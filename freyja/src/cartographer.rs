// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use freyja_common::signal_store::SignalStore;
use log::{info, warn};

use freyja_contracts::{
    conversion::Conversion,
    digital_twin_adapter::{DigitalTwinAdapter, GetDigitalTwinProviderRequest, DigitalTwinAdapterError, DigitalTwinAdapterErrorKind},
    mapping_client::{CheckForWorkRequest, GetMappingRequest, MappingClient},
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
    signal::{Emission, EmissionPolicy, Signal, Target},
};

/// Manages mappings from the mapping service
pub struct Cartographer {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The mapping client
    mapping_client: Box<dyn MappingClient>,

    /// The digital twin client
    digital_twin_client: Box<dyn DigitalTwinAdapter>,

    /// The provider proxy selector client
    provider_proxy_selector_client: ProviderProxySelectorRequestSender,

    /// The mapping service polling interval
    poll_interval: Duration,
}

impl Cartographer {
    /// Create a new instance of a Cartographer
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `mapping_client`: the client for the mapping service
    /// - `poll_interval`: the interval at which the cartographer should poll for changes
    pub fn new(
        signals: Arc<SignalStore>,
        mapping_client: Box<dyn MappingClient>,
        digital_twin_client: Box<dyn DigitalTwinAdapter>,
        provider_proxy_selector_client: ProviderProxySelectorRequestSender,
        poll_interval: Duration,
    ) -> Self {
        Self {
            signals,
            mapping_client,
            digital_twin_client,
            provider_proxy_selector_client,
            poll_interval,
        }
    }

    /// Run the cartographer. This will do the following in a loop:
    ///
    /// 1. Check to see if the mapping service has more work. If not, skip to step 5
    /// 2. Send the new inventory to the mapping service
    /// 3. Get the new mapping from the mapping service
    /// 4. Update the shared map for the emitter
    /// 5. Sleep until the next iteration
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let mapping_client_result = self
                .mapping_client
                .check_for_work(CheckForWorkRequest {})
                .await;
            
            if mapping_client_result.is_err() {
                log::error!("Failed to check for mapping work; will try again later. Error: {mapping_client_result:?}");
                continue;
            }

            if mapping_client_result.unwrap().has_work {
                info!("Cartographer detected mapping work");

                // TODO: will this notion of checking and sending inventory exist?
                // self.mapping_client.send_inventory(SendInventoryRequest { inventory: self.known_providers.clone() }).await?;

                let signals_result = self.get_mapping_as_signals().await;
                if signals_result.is_err() {
                    log::error!("Falied to get mapping from mapping client: {signals_result:?}");
                    continue;
                }
                
                let mut signals = signals_result.unwrap();
                let mut failed_signals = Vec::new();

                // Some of these API calls are not really necessary, but this code gets executed
                // infrequently enough that the sub-optimal performance is not a major concern.
                // If Ibeji had a bulk find_by_id API there would be even less of a concern.
                // TODO: punt stuff to the dt client and we call find_all here
                // TODO: if there's a bulk api for providers then there probably needs to be a bulk api for proxies
                // TODO: handle errors
                for signal in signals.iter_mut() {
                    let populate_result = self.populate_entity(signal).await;

                    if populate_result.is_err() {
                        match populate_result.err().unwrap().downcast::<DigitalTwinAdapterError>() {
                            Ok(e) if e.kind() == DigitalTwinAdapterErrorKind::EntityNotFound => {
                                warn!("Entity not found for signal {}", signal.id);
                            },
                            Ok(e) => {
                                log::error!("Error fetching entity for signal {}: {e:?}", signal.id);
                            },
                            Err(e) => {
                                log::error!("Error fetching entity for signal {}: {e:?}", signal.id);
                            },
                        }

                        failed_signals.push(signal.id.clone());
                    }
                }

                self.signals.sync(signals.into_iter().filter(|s| !failed_signals.contains(&s.id)));
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Gets the mapping from the mapping client and returns a corresponding list of signals.
    async fn get_mapping_as_signals(
        &self,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .mapping_client
            .get_mapping(GetMappingRequest {})
            .await?
            .map
            .into_iter()
            .map(|(id, entry)| Signal {
                id,
                target: Target {
                    metadata: entry.target,
                },
                emission: Emission {
                    policy: EmissionPolicy {
                        interval_ms: entry.interval_ms,
                        emit_only_if_changed: entry.emit_on_change,
                        conversion: Conversion::default(),
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .collect())
    }

    async fn populate_entity(
        &self,
        signal: &mut Signal,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        signal.source = self
            .digital_twin_client
            .find_by_id(GetDigitalTwinProviderRequest {
                entity_id: signal.id.clone(),
            })
            .await?
            .entity;

        let request = ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy {
            entity_id: signal.source.id.clone(),
            uri: signal.source.uri.clone(),
            protocol: signal.source.protocol.clone(),
            operation: signal.source.operation.clone(),
        };

        self.provider_proxy_selector_client
            .send_request_to_provider_proxy_selector(request)
            .map_err(|e| format!("Error sending request to provider proxy selector: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod cartographer_tests {
    use std::collections::HashMap;

    use super::*;

    use async_trait::async_trait;
    use mockall::*;

    use freyja_contracts::{
        digital_twin_adapter::{DigitalTwinAdapterError, GetDigitalTwinProviderResponse},
        digital_twin_map_entry::DigitalTwinMapEntry,
        entity::Entity,
        mapping_client::{
            CheckForWorkResponse, GetMappingResponse, MappingClientError, SendInventoryRequest,
            SendInventoryResponse,
        },
        provider_proxy::OperationKind,
    };
    use tokio::sync::mpsc;

    mock! {
        pub DigitalTwinAdapterImpl {}

        #[async_trait]
        impl DigitalTwinAdapter for DigitalTwinAdapterImpl {
            fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError>
            where
                Self: Sized;

            async fn find_by_id(
                &self,
                request: GetDigitalTwinProviderRequest,
            ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError>;
        }
    }

    mock! {
        pub MappingClientImpl {}

        #[async_trait]
        impl MappingClient for MappingClientImpl {
            fn create_new() -> Result<Box<dyn MappingClient>, MappingClientError>
            where
                Self: Sized;

            async fn check_for_work(
                &self,
                request: CheckForWorkRequest,
            ) -> Result<CheckForWorkResponse, MappingClientError>;

            async fn send_inventory(
                &self,
                inventory: SendInventoryRequest,
            ) -> Result<SendInventoryResponse, MappingClientError>;

            async fn get_mapping(
                &self,
                request: GetMappingRequest,
            ) -> Result<GetMappingResponse, MappingClientError>;
        }
    }

    #[tokio::test]
    async fn get_mapping_as_signals_returns_correct_value() {
        const ID: &str = "testid";
        let test_map_entry = DigitalTwinMapEntry {
            source: ID.to_string(),
            target: HashMap::new(),
            interval_ms: 42,
            conversion: Default::default(),
            emit_on_change: true,
        };

        let test_map_entry_clone = test_map_entry.clone();

        let mut mock_mapping_client = MockMappingClientImpl::new();
        mock_mapping_client
            .expect_get_mapping()
            .returning(move |_| {
                Ok(GetMappingResponse {
                    map: [(ID.to_string(), test_map_entry_clone.clone())]
                        .into_iter()
                        .collect(),
                })
            });

        let (tx, _) = mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let provider_proxy_selector_client = ProviderProxySelectorRequestSender::new(tx);
        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_client: Box::new(mock_mapping_client),
            digital_twin_client: Box::new(MockDigitalTwinAdapterImpl::new()),
            provider_proxy_selector_client,
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.get_mapping_as_signals().await;
        assert!(result.is_ok());
        let mut signals = result.unwrap();
        assert_eq!(signals.len(), 1);
        let signal = signals.pop().unwrap();
        assert_eq!(signal.id, ID.to_string());
        assert_eq!(signal.target.metadata, test_map_entry.target);
        assert_eq!(
            signal.emission.policy.interval_ms,
            test_map_entry.interval_ms
        );
        assert_eq!(
            signal.emission.policy.emit_only_if_changed,
            test_map_entry.emit_on_change
        );
        assert_eq!(signal.emission.policy.conversion, test_map_entry.conversion);
    }

    #[tokio::test]
    async fn populate_entity_tests() {
        const ID: &str = "testid";
        let test_entity = Entity {
            id: ID.to_string(),
            name: Some("name".to_string()),
            uri: "uri".to_string(),
            description: Some("description".to_string()),
            operation: OperationKind::Get,
            protocol: "protocol".to_string(),
        };

        let test_signal = &mut Signal {
            id: ID.to_string(),
            ..Default::default()
        };

        let test_entity_clone = test_entity.clone();

        let mut mock_dt_adapter = MockDigitalTwinAdapterImpl::new();
        mock_dt_adapter.expect_find_by_id().returning(move |_| {
            Ok(GetDigitalTwinProviderResponse {
                entity: test_entity_clone.clone(),
            })
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let provider_proxy_selector_client = ProviderProxySelectorRequestSender::new(tx);
        let listener_handler = tokio::spawn(async move { rx.recv().await });

        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_client: Box::new(MockMappingClientImpl::new()),
            digital_twin_client: Box::new(mock_dt_adapter),
            provider_proxy_selector_client,
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.populate_entity(test_signal).await;
        let join_result = listener_handler.await;

        assert!(result.is_ok());
        assert!(join_result.is_ok());
        assert_eq!(test_signal.source, test_entity);

        let proxy_request = join_result.unwrap();
        assert!(proxy_request.is_some());
        let proxy_request = proxy_request.as_ref().unwrap();
        match proxy_request {
            ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy {
                entity_id,
                uri,
                protocol,
                operation,
            } => {
                assert_eq!(*entity_id, test_entity.id);
                assert_eq!(*uri, test_entity.uri);
                assert_eq!(*protocol, test_entity.protocol);
                assert_eq!(*operation, test_entity.operation);
            }
            _ => panic!("Unexpected proxy request kind: {proxy_request:?}"),
        }
    }
}
