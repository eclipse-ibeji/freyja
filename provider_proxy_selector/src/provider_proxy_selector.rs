// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::{hash_map::Entry::Occupied, HashMap},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crossbeam::queue::SegQueue;
use log::{debug, info, warn};
use strum_macros::{Display, EnumString};
use tokio::{sync::mpsc::UnboundedReceiver, time::Duration};

use dts_contracts::{
    entity::{Entity, EntityID, ProviderURI},
    provider_proxy::{OperationKind, ProviderProxy, ProviderProxyError, SignalValue},
    provider_proxy_request::ProviderProxySelectorRequestKind,
};
use grpc_provider_proxy_v1::grpc_provider_proxy::GRPCProviderProxy;
use http_mock_provider_proxy::http_mock_provider_proxy::HttpMockProviderProxy;
use in_memory_mock_provider_proxy::in_memory_provider_proxy::InMemoryMockProviderProxy;

type ProviderProxyImpl = Arc<Box<dyn ProviderProxy + Send + Sync>>;

/// Provider Proxy for matching the message delivery model
#[derive(Debug, Clone, EnumString, Display, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum ProviderProxyKind {
    #[strum(serialize = "grpc")]
    GRPCProviderProxy,

    #[strum(serialize = "mqtt")]
    MqttProviderProxy,

    #[strum(serialize = "in-memory")]
    InMemoryMockProviderProxy,

    #[strum(serialize = "http")]
    HttpProviderProxy,
}

impl ProviderProxyKind {
    /// Handles the result from a provider proxy's instantiation
    ///
    /// # Arguments
    /// - `provider_proxy_result`: the result of creating a provider proxy
    /// - `provider_proxy_kind`: the provider proxy kind
    fn handle_provider_proxy_result(
        provider_proxy_result: Result<Box<dyn ProviderProxy + Send + Sync>, ProviderProxyError>,
        provider_proxy_kind: ProviderProxyKind,
    ) -> Result<ProviderProxyImpl, ProviderProxySelectorError> {
        if let Err(error) = provider_proxy_result {
            warn!("Cannot create a {provider_proxy_kind} due to {error:?}");
            return Err(ProviderProxySelectorError::communication(error));
        }

        Ok(Arc::new(provider_proxy_result.unwrap()))
    }

    /// Instantiates the respective provider proxy using protocol and operation
    ///
    /// # Arguments
    /// - `protocol`: the protocol for identifying the provider proxy
    /// - `operation`: the operation for identifying the provider proxy
    /// - `provider_uri`: the provider uri to contact
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    async fn create_provider_proxy(
        protocol: &str,
        operation: &OperationKind,
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<ProviderProxyImpl, ProviderProxySelectorError> {
        // Take the protocol match it to the ProviderProxyKind concrete implementation
        // With the concrete implementation, check if our operation matches with anything that the provider has
        let protocol_kind = ProviderProxyKind::from_str(protocol)
            .map_err(ProviderProxySelectorError::protocol_not_supported)?;

        let provider_proxy: ProviderProxyImpl;
        match protocol_kind {
            ProviderProxyKind::GRPCProviderProxy
                if GRPCProviderProxy::is_operation_supported(operation) =>
            {
                info!("Creating a GRPCProviderProxy");

                let grpc_provider_proxy_result =
                    GRPCProviderProxy::create_new(provider_uri, signal_values_queue);

                provider_proxy = ProviderProxyKind::handle_provider_proxy_result(
                    grpc_provider_proxy_result,
                    ProviderProxyKind::GRPCProviderProxy,
                )?;
            }
            ProviderProxyKind::InMemoryMockProviderProxy
                if InMemoryMockProviderProxy::is_operation_supported(operation) =>
            {
                info!("Creating an InMemoryProviderProxy");

                let in_memory_mock_provider_proxy_result =
                    InMemoryMockProviderProxy::create_new(provider_uri, signal_values_queue);

                provider_proxy = ProviderProxyKind::handle_provider_proxy_result(
                    in_memory_mock_provider_proxy_result,
                    ProviderProxyKind::InMemoryMockProviderProxy,
                )?;
            }
            ProviderProxyKind::HttpProviderProxy
                if HttpMockProviderProxy::is_operation_supported(operation) =>
            {
                info!("Creating an HttpProviderProxy");
                let http_provider_proxy_result =
                    HttpMockProviderProxy::create_new(provider_uri, signal_values_queue);

                provider_proxy = ProviderProxyKind::handle_provider_proxy_result(
                    http_provider_proxy_result,
                    ProviderProxyKind::HttpProviderProxy,
                )?;
            }
            _ => {
                return Err(ProviderProxySelectorError::operation_not_supported(
                    "operation not supported",
                ))
            }
        }
        Ok(provider_proxy)
    }
}

/// The provider proxy selector selects which provider proxy to create based on protocol and operation
pub struct ProviderProxySelector {
    /// A map of entity uri to provider proxy
    pub provider_proxies: Arc<Mutex<HashMap<ProviderURI, ProviderProxyImpl>>>,

    /// A map of entity id to provider uri
    pub entity_map: Arc<Mutex<HashMap<EntityID, ProviderURI>>>,
}

impl ProviderProxySelector {
    /// Instantiates the provider proxy selector
    pub fn new() -> Self {
        ProviderProxySelector {
            provider_proxies: Arc::new(Mutex::new(HashMap::new())),
            entity_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Handles incoming requests to the provider proxy selector
    ///
    /// # Arguments
    /// - `rx_provider_proxy_selector_request`: receiver of the provider proxy selector request channel
    /// - `signal_values_queue`: shared queue for all provider proxies to push new signal values of entities
    async fn handle_incoming_requests(
        &self,
        mut rx_provider_proxy_selector_request: UnboundedReceiver<ProviderProxySelectorRequestKind>,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<(), ProviderProxySelectorError> {
        loop {
            let message = rx_provider_proxy_selector_request.recv().await;
            if message.is_none() {
                warn!("Channel is closed, aborting receive provider proxy selector request responder...");
                break;
            }
            let message = message.unwrap();
            debug!("Handling new request {:?}", message);

            match message {
                ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(
                    entity_id,
                    provider_uri,
                    protocol,
                    operation,
                ) => {
                    let entity = Entity {
                        id: entity_id,
                        uri: provider_uri,
                        name: None,
                        description: None,
                        operation,
                        protocol,
                    };
                    let _ = self
                        .handle_create_update_provider_proxy_request(
                            signal_values_queue.clone(),
                            &entity,
                        )
                        .await;
                }
                ProviderProxySelectorRequestKind::GetEntityValue(entity_id) => {
                    let provider_uri;
                    {
                        let lock = self.entity_map.lock().unwrap();
                        let entity_uri_option = lock.get(&entity_id);
                        if entity_uri_option.is_none() {
                            debug!("Unable to retrieve entity uri for {entity_id}");
                            continue;
                        }
                        provider_uri = String::from(entity_uri_option.unwrap());
                    }

                    let mut provider_proxy_option: Option<ProviderProxyImpl> = None;
                    {
                        let mut provider_proxies = self.provider_proxies.lock().unwrap();
                        if let Occupied(provider_proxy) = provider_proxies.entry(provider_uri) {
                            provider_proxy_option = Some(provider_proxy.get().clone());
                        }
                    }

                    if provider_proxy_option.is_none() {
                        warn!("Provider proxy for {entity_id} is not available");
                        continue;
                    }
                    provider_proxy_option
                        .unwrap()
                        .send_request_to_provider(&entity_id)
                        .await
                        .map_err(ProviderProxySelectorError::communication)?;
                }
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        Ok(())
    }

    /// Runs the provider proxy selector
    ///
    /// # Arguments
    /// - `rx_provider_proxy_selector_request`: receiver of the provider proxy selector request channel
    /// - `signal_values_queue`: shared queue for all provider proxies to push new signal values of entities
    pub async fn run(
        &self,
        rx_provider_proxy_selector_request: UnboundedReceiver<ProviderProxySelectorRequestKind>,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<(), ProviderProxySelectorError> {
        self.handle_incoming_requests(rx_provider_proxy_selector_request, signal_values_queue)
            .await
    }

    /// Retrieves the provider proxy for entity uri
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri
    fn retrieve_provider_proxy(&self, provider_uri: &str) -> Option<ProviderProxyImpl> {
        let provider_proxies = self.provider_proxies.lock().unwrap();
        provider_proxies.get(provider_uri).cloned()
    }

    /// Maps the entity id to the provider uri
    ///
    /// # Arguments
    /// - `entity_id`: the entity id is the key
    /// - `provider_uri`: the provider uri
    fn insert_entity_id_with_uri_to_entity_map(&self, entity_id: &str, provider_uri: &str) {
        let mut map = self.entity_map.lock().unwrap();
        map.insert(String::from(entity_id), String::from(provider_uri));
    }

    /// Handles requests for creating or updating the provider proxy
    /// Creates a provider proxy if a proxy doesn't exist for an entity uri
    /// Otherwise, gets the existing provider proxy for an entity uri
    ///
    /// # Arguments
    /// - `signal_values_queue`: shared queue for all provider proxies to push new signal values of entities
    /// - `entity`: start a provider proxy for this entity
    async fn handle_create_update_provider_proxy_request(
        &self,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
        entity: &Entity,
    ) -> Result<(), ProviderProxySelectorError> {
        let (entity_id, provider_uri, operation, protocol) =
            (&entity.id, &entity.uri, &entity.operation, &entity.protocol);

        // If a provider proxy already exists for this uri,
        // then we notify that proxy to include this new entity_id
        if let Some(provider_proxy) = self.retrieve_provider_proxy(provider_uri) {
            debug!("A provider proxy for {provider_uri} already exists");
            self.insert_entity_id_with_uri_to_entity_map(entity_id, provider_uri);
            return provider_proxy
                .register_entity(entity_id, operation)
                .await
                .map_err(ProviderProxySelectorError::communication);
        }

        match ProviderProxyKind::create_provider_proxy(
            protocol,
            operation,
            provider_uri,
            signal_values_queue,
        )
        .await
        {
            Ok(provider_proxy) => {
                // If we're able to create a provider_proxy then map the
                // provider uri to that created proxy
                {
                    let mut provider_proxies = self.provider_proxies.lock().unwrap();
                    provider_proxies.insert(provider_uri.clone(), provider_proxy.clone());
                }

                self.insert_entity_id_with_uri_to_entity_map(entity_id, provider_uri);

                let proxy = provider_proxy.clone();
                tokio::spawn(async move {
                    let _ = proxy.run().await;
                });

                let _ = provider_proxy.register_entity(entity_id, operation).await;
            }
            Err(err) => {
                return Err(err);
            }
        }
        Ok(())
    }
}

impl Default for ProviderProxySelector {
    fn default() -> Self {
        Self::new()
    }
}

proc_macros::error! {
    ProviderProxySelectorError {
        ProtocolNotSupported,
        OperationNotSupported,
        Io,
        Serialize,
        Deserialize,
        Communication,
        Unknown
    }
}

#[cfg(test)]
mod provider_proxy_selector_tests {
    use super::*;

    use dts_contracts::provider_proxy::OperationKind;

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

    #[tokio::test]
    async fn handle_start_provider_proxy_request_return_err_test() {
        let protocol_selector = ProviderProxySelector::new();

        let entity = Entity {
            id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
            uri: String::new(),
            name: None,
            description: None,
            operation: OperationKind::Subscribe,
            protocol: String::from("grpc"),
        };

        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
        let result = protocol_selector
            .handle_create_update_provider_proxy_request(signal_values_queue, &entity)
            .await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ProviderProxySelectorErrorKind::Communication
        );
    }

    #[test]
    fn protocol_kind_match_test() {
        let mut grpc = String::from("grpc");
        let mut protocol_kind = ProviderProxyKind::from_str(&grpc).unwrap();
        assert_eq!(protocol_kind, ProviderProxyKind::GRPCProviderProxy);

        grpc = String::from("gRPC");
        protocol_kind = ProviderProxyKind::from_str(&grpc).unwrap();
        assert_eq!(protocol_kind, ProviderProxyKind::GRPCProviderProxy);

        let mut mqtt = String::from("mqtt");
        protocol_kind = ProviderProxyKind::from_str(&mqtt).unwrap();
        assert_eq!(protocol_kind, ProviderProxyKind::MqttProviderProxy);

        mqtt = String::from("mQTT");
        protocol_kind = ProviderProxyKind::from_str(&mqtt).unwrap();
        assert_eq!(protocol_kind, ProviderProxyKind::MqttProviderProxy);
    }
}
