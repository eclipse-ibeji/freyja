// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use freyja_common::signal_store::SignalStore;
use log::{info, warn};

use freyja_contracts::{
    conversion::Conversion,
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, DigitalTwinAdapterErrorKind, FindByIdRequest,
    },
    mapping_client::{CheckForWorkRequest, GetMappingRequest, MappingClient},
    provider_proxy_selector::ProviderProxySelector,
    signal::{EmissionPolicy, SignalPatch, Target},
};
use tokio::sync::Mutex;

/// Manages mappings from the mapping service
pub struct Cartographer<TMappingClient, TDigitalTwinAdapter, TProviderProxySelector> {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The mapping client
    mapping_client: TMappingClient,

    /// The digital twin client
    digital_twin_client: TDigitalTwinAdapter,

    /// The provider proxy selector
    provider_proxy_selector: Arc<Mutex<TProviderProxySelector>>,

    /// The mapping service polling interval
    poll_interval: Duration,
}

impl<
        TMappingClient: MappingClient,
        TDigitalTwinAdapter: DigitalTwinAdapter,
        TProviderProxySelector: ProviderProxySelector,
    > Cartographer<TMappingClient, TDigitalTwinAdapter, TProviderProxySelector>
{
    /// Create a new instance of a Cartographer
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `mapping_client`: the client for the mapping service
    /// - `digital_twin_client`: the client for the digital twin service
    /// - `provider_proxy_selector`: the provider proxy selector
    /// - `poll_interval`: the interval at which the cartographer should poll for changes
    pub fn new(
        signals: Arc<SignalStore>,
        mapping_client: TMappingClient,
        digital_twin_client: TDigitalTwinAdapter,
        provider_proxy_selector: Arc<Mutex<TProviderProxySelector>>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            signals,
            mapping_client,
            digital_twin_client,
            provider_proxy_selector,
            poll_interval,
        }
    }

    /// Run the cartographer. This will do the following in a loop:
    ///
    /// 1. Check to see if the mapping service has more work. If not, skip to the last step
    /// 1. ~~Send the new inventory to the mapping service~~
    /// 1. Get the new mapping from the mapping service
    /// 1. Query the digital twin service for entity information
    /// 1. Create or update provider proxies for the new entities
    /// 1. Update the signal store with the new data
    /// 1. Sleep until the next iteration
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let mapping_client_result = self
                .mapping_client
                .check_for_work(CheckForWorkRequest {})
                .await;

            if mapping_client_result.is_err() {
                let error = mapping_client_result.err().unwrap();
                log::error!(
                    "Failed to check for mapping work; will try again later. Error: {error}"
                );
                continue;
            }

            if mapping_client_result.unwrap().has_work {
                info!("Cartographer detected mapping work");

                let patches_result = self.get_mapping_as_signal_patches().await;
                if patches_result.is_err() {
                    let error = patches_result.err().unwrap();
                    log::error!("Falied to get mapping from mapping client: {error}");
                    continue;
                }

                let mut patches = patches_result.unwrap();
                let mut failed_signals = Vec::new();

                for patch in patches.iter_mut() {
                    // Many of the API calls in populate_entity are probably unnecessary, but this code gets executed
                    // infrequently enough that the sub-optimal performance is not a major concern.
                    // A bulk find_by_id API in the digital twin service would make this a non-issue
                    let populate_result = self.populate_source(patch).await;

                    if populate_result.is_err() {
                        match populate_result
                            .err()
                            .unwrap()
                            .downcast::<DigitalTwinAdapterError>()
                        {
                            Ok(e) if e.kind() == DigitalTwinAdapterErrorKind::EntityNotFound => {
                                warn!("Entity not found for signal {}", patch.id);
                            }
                            Ok(e) => {
                                log::error!("Error fetching entity for signal {}: {e:?}", patch.id);
                            }
                            Err(e) => {
                                log::error!("Error fetching entity for signal {}: {e:?}", patch.id);
                            }
                        }

                        failed_signals.push(patch.id.clone());
                    }
                }

                self.signals.sync(
                    patches
                        .into_iter()
                        .filter(|s| !failed_signals.contains(&s.id)),
                );
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Gets the mapping from the mapping client and returns a corresponding list of signal patches.
    async fn get_mapping_as_signal_patches(
        &self,
    ) -> Result<Vec<SignalPatch>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .mapping_client
            .get_mapping(GetMappingRequest {})
            .await?
            .map
            .into_iter()
            .map(|(id, entry)| SignalPatch {
                id,
                // this gets populated later, set to default for now
                source: Default::default(),
                target: Target {
                    metadata: entry.target,
                },
                emission_policy: EmissionPolicy {
                    interval_ms: entry.interval_ms,
                    emit_only_if_changed: entry.emit_on_change,
                    conversion: Conversion::default(),
                },
            })
            .collect())
    }

    /// Populates the source of the provided signal with data retrieved from the digital twin service.
    /// This will also create or update a proxy to handle incoming requests from the provider.
    ///
    /// Arguments
    /// - `signal`: The signal patch to update
    async fn populate_source(
        &self,
        signal: &mut SignalPatch,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        signal.source = self
            .digital_twin_client
            .find_by_id(FindByIdRequest {
                entity_id: signal.id.clone(),
            })
            .await?
            .entity;

        {
            let provider_proxy_selector = self.provider_proxy_selector.lock().await;
            provider_proxy_selector
                .create_or_update_proxy(&signal.source)
                .await
                .map_err(|e| format!("Error sending request to provider proxy selector: {e}"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod cartographer_tests {
    use super::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use mockall::{predicate::eq, *};

    use freyja_contracts::{
        digital_twin_adapter::{DigitalTwinAdapterError, FindByIdResponse},
        digital_twin_map_entry::DigitalTwinMapEntry,
        entity::{Entity, EntityEndpoint},
        mapping_client::{
            CheckForWorkResponse, GetMappingResponse, MappingClientError, SendInventoryRequest,
            SendInventoryResponse,
        },
        provider_proxy_selector::ProviderProxySelectorError,
    };

    mock! {
        pub DigitalTwinAdapterImpl {}

        #[async_trait]
        impl DigitalTwinAdapter for DigitalTwinAdapterImpl {
            fn create_new() -> Result<Self, DigitalTwinAdapterError>
            where
                Self: Sized;

            async fn find_by_id(
                &self,
                request: FindByIdRequest,
            ) -> Result<FindByIdResponse, DigitalTwinAdapterError>;
        }
    }

    mock! {
        pub MappingClientImpl {}

        #[async_trait]
        impl MappingClient for MappingClientImpl {
            fn create_new() -> Result<Self, MappingClientError>
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

    mock! {
        pub ProviderProxySelector {}

        #[async_trait]
        impl ProviderProxySelector for ProviderProxySelector {
            async fn create_or_update_proxy(&self, entity: &Entity) -> Result<(), ProviderProxySelectorError>;
            async fn request_entity_value(&self, entity_id: &str) -> Result<(), ProviderProxySelectorError>;
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

        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_client: mock_mapping_client,
            digital_twin_client: MockDigitalTwinAdapterImpl::new(),
            provider_proxy_selector: Arc::new(Mutex::new(MockProviderProxySelector::new())),
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.get_mapping_as_signal_patches().await;

        assert!(result.is_ok());
        let mut signals = result.unwrap();
        assert_eq!(signals.len(), 1);
        let signal = signals.pop().unwrap();
        assert_eq!(signal.id, ID.to_string());
        assert_eq!(signal.target.metadata, test_map_entry.target);
        assert_eq!(
            signal.emission_policy.interval_ms,
            test_map_entry.interval_ms
        );
        assert_eq!(
            signal.emission_policy.emit_only_if_changed,
            test_map_entry.emit_on_change
        );
        assert_eq!(signal.emission_policy.conversion, test_map_entry.conversion);
    }

    #[tokio::test]
    async fn populate_source_tests() {
        const ID: &str = "testid";
        let test_entity = Entity {
            id: ID.to_string(),
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            endpoints: vec![EntityEndpoint {
                operations: vec!["FooOperation".to_string()],
                protocol: "in-memory".to_string(),
                uri: "uri".to_string(),
            }],
        };

        let test_signal_patch = &mut SignalPatch {
            id: ID.to_string(),
            ..Default::default()
        };

        let test_entity_clone = test_entity.clone();

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_create_or_update_proxy()
            .with(eq(test_entity.clone()))
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_dt_adapter = MockDigitalTwinAdapterImpl::new();
        mock_dt_adapter.expect_find_by_id().returning(move |_| {
            Ok(FindByIdResponse {
                entity: test_entity_clone.clone(),
            })
        });

        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_client: MockMappingClientImpl::new(),
            digital_twin_client: mock_dt_adapter,
            provider_proxy_selector,
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.populate_source(test_signal_patch).await;

        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(test_signal_patch.source, test_entity);
    }
}
