// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
    sync::Arc,
};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::{debug, info, warn};
use strum_macros::{Display, EnumString};

use freyja_contracts::{
    entity::Entity,
    provider_proxy::{OperationKind, ProviderProxy, ProviderProxyError, SignalValue},
    provider_proxy_selector::{ProviderProxySelector, ProviderProxySelectorError},
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

/// The provider proxy selector selects which provider proxy to create based on protocol and operation.
/// This struct is **not** thread-safe and should be shared with `Arc<Mutex<ProviderProxySelectorImpl>>`.
pub struct ProviderProxySelectorImpl {
    /// A map of entity uri to provider proxy
    pub provider_proxies: HashMap<String, ProviderProxyImpl>,

    /// A map of entity id to provider uri
    pub entity_map: HashMap<String, String>,

    /// The signal values queue used for creating the proxies
    pub signal_values_queue: Arc<SegQueue<SignalValue>>,
}

impl ProviderProxySelectorImpl {
    /// Instantiates the provider proxy selector
    pub fn new(signal_values_queue: Arc<SegQueue<SignalValue>>) -> Self {
        ProviderProxySelectorImpl {
            provider_proxies: HashMap::new(),
            entity_map: HashMap::new(),
            signal_values_queue,
        }
    }
}

#[async_trait]
impl ProviderProxySelector for ProviderProxySelectorImpl {
    /// Updates an existing proxy for an entity if possible,
    /// otherwise creates a new proxy to handle that entity.
    ///
    /// # Arguments
    /// - `entity`: the entity that the proxy should handle
    async fn create_or_update_proxy(
        &mut self,
        entity: &Entity,
    ) -> Result<(), ProviderProxySelectorError> {
        let (entity_id, provider_uri, operation, protocol) =
            (&entity.id, &entity.uri, &entity.operation, &entity.protocol);

        // If a provider proxy already exists for this uri,
        // then we notify that proxy to include this new entity_id
        if let Some(provider_proxy) = self.provider_proxies.get(provider_uri).cloned() {
            debug!("A provider proxy for {provider_uri} already exists");
            self.entity_map
                .insert(String::from(entity_id), String::from(provider_uri));
            return provider_proxy
                .register_entity(entity_id, operation)
                .await
                .map_err(ProviderProxySelectorError::communication);
        }

        let provider_proxy = ProviderProxyKind::create_provider_proxy(
            protocol,
            operation,
            provider_uri,
            self.signal_values_queue.clone(),
        )
        .await?;

        // If we're able to create a provider_proxy then map the
        // provider uri to that created proxy
        self.provider_proxies
            .insert(provider_uri.clone(), provider_proxy.clone());

        self.entity_map
            .insert(String::from(entity_id), String::from(provider_uri));

        let proxy = provider_proxy.clone();
        tokio::spawn(async move {
            let _ = proxy.run().await;
        });

        provider_proxy
            .register_entity(entity_id, operation)
            .await
            .map_err(ProviderProxySelectorError::provider_proxy_error)
    }

    /// Requests that the value of an entity be published as soon as possible
    ///
    /// # Arguments
    /// - `entity_id`: the entity to request
    async fn request_entity_value(
        &mut self,
        entity_id: &str,
    ) -> Result<(), ProviderProxySelectorError> {
        let provider_uri = {
            self.entity_map
                .get(entity_id)
                .ok_or(ProviderProxySelectorError::entity_not_found(format!(
                    "Unable to retrieve entity uri for {entity_id}"
                )))?
                .to_owned()
        };

        match self.provider_proxies.entry(provider_uri) {
            Entry::Occupied(provider_proxy) => provider_proxy
                .get()
                .send_request_to_provider(entity_id)
                .await
                .map_err(ProviderProxySelectorError::communication),
            Entry::Vacant(_) => Err(ProviderProxySelectorError::entity_not_found(format!(
                "Provider proxy for {entity_id} is not available"
            ))),
        }
    }
}

#[cfg(test)]
mod provider_proxy_selector_tests {
    use super::*;

    use freyja_contracts::{
        provider_proxy::OperationKind, provider_proxy_selector::ProviderProxySelectorErrorKind,
    };

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

    #[tokio::test]
    async fn handle_start_provider_proxy_request_return_err_test() {
        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
        let mut uut = ProviderProxySelectorImpl::new(signal_values_queue);

        let entity = Entity {
            id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
            uri: String::new(),
            name: None,
            description: None,
            operation: OperationKind::Subscribe,
            protocol: String::from("grpc"),
        };

        let result = uut.create_or_update_proxy(&entity).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
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
