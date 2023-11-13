// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::debug;
use mqtt_provider_proxy::mqtt_provider_proxy_factory::MqttProviderProxyFactory;
use tokio::sync::Mutex;

use freyja_contracts::{
    entity::Entity,
    provider_proxy::{ProviderProxy, ProviderProxyFactory, SignalValue},
    provider_proxy_selector::{
        ProviderProxySelector, ProviderProxySelectorError, ProviderProxySelectorErrorKind,
    },
};
use grpc_provider_proxy_v1::grpc_provider_proxy_factory::GRPCProviderProxyFactory;
use http_mock_provider_proxy::http_mock_provider_proxy_factory::HttpMockProviderProxyFactory;
use in_memory_mock_provider_proxy::in_memory_mock_provider_proxy_factory::InMemoryMockProviderProxyFactory;

/// Represents the state of the ProviderProxySelector and allows for simplified access through a mutex
struct ProviderProxySelectorState {
    /// A map of entity uri to provider proxy
    provider_proxies: HashMap<String, Arc<dyn ProviderProxy + Send + Sync>>,

    /// A map of entity id to provider uri
    entity_map: HashMap<String, String>,
}

/// The provider proxy selector selects which provider proxy to create based on protocol and operation.
/// This struct is **not** thread-safe and should be shared with `Arc<Mutex<ProviderProxySelectorImpl>>`.
pub struct ProviderProxySelectorImpl {
    /// The set of factories that have been registered
    factories: Vec<Box<dyn ProviderProxyFactory + Send + Sync>>,

    /// The ProviderPrxySelector's state
    state: Mutex<ProviderProxySelectorState>,

    /// The signal values queue used for creating the proxies
    signal_values_queue: Arc<SegQueue<SignalValue>>,
}

impl ProviderProxySelectorImpl {
    /// Instantiates the provider proxy selector
    ///
    /// # Arguments
    /// - `signal_values_queue`: The queue that is passed to proxies andused to update the emitter
    pub fn new(signal_values_queue: Arc<SegQueue<SignalValue>>) -> Self {
        let factories: Vec<Box<dyn ProviderProxyFactory + Send + Sync>> = vec![
            Box::new(GRPCProviderProxyFactory {}),
            Box::new(HttpMockProviderProxyFactory {}),
            Box::new(InMemoryMockProviderProxyFactory {}),
            Box::new(MqttProviderProxyFactory {}),
        ];

        ProviderProxySelectorImpl {
            factories,
            state: Mutex::new(ProviderProxySelectorState {
                provider_proxies: HashMap::new(),
                entity_map: HashMap::new(),
            }),
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
        &self,
        entity: &Entity,
    ) -> Result<(), ProviderProxySelectorError> {
        let mut state = self.state.lock().await;

        // If a provider proxy already exists for one of this entity's uris,
        // then we notify that proxy to include this new entity
        for endpoint in entity.endpoints.iter() {
            if let Some(provider_proxy) = state.provider_proxies.get(&endpoint.uri) {
                debug!("A provider proxy for {} already exists", &endpoint.uri);

                let result = provider_proxy
                    .register_entity(&entity.id, endpoint)
                    .await
                    .map_err(ProviderProxySelectorError::communication);

                state
                    .entity_map
                    .insert(String::from(&entity.id), String::from(&endpoint.uri));

                return result;
            }
        }

        // If there's not a proxy we can reuse, find the right factory to create a new one
        let (provider_proxy, endpoint) = {
            let mut result = None;
            for factory in self.factories.iter() {
                if let Some(endpoint) = factory.is_supported(entity) {
                    let proxy = factory
                        .create_proxy(&endpoint.uri, self.signal_values_queue.clone())
                        .map_err(ProviderProxySelectorError::provider_proxy_error)?;
                    result = Some((proxy, endpoint));
                }
            }

            result.ok_or::<ProviderProxySelectorError>(
                ProviderProxySelectorErrorKind::OperationNotSupported.into(),
            )?
        };

        // If we're able to create a proxy then map the
        // provider uri to that created proxy
        state
            .entity_map
            .insert(entity.id.clone(), endpoint.uri.clone());

        let provider_proxy_clone = provider_proxy.clone();
        tokio::spawn(async move {
            let _ = provider_proxy_clone.run().await;
        })
        .await
        .map_err(ProviderProxySelectorError::provider_proxy_error)?;

        provider_proxy
            .register_entity(&entity.id, &endpoint)
            .await
            .map_err(ProviderProxySelectorError::provider_proxy_error)?;

        state
            .provider_proxies
            .insert(endpoint.uri.clone(), provider_proxy);

        Ok(())
    }

    /// Requests that the value of an entity be published as soon as possible
    ///
    /// # Arguments
    /// - `entity_id`: the entity to request
    async fn request_entity_value(
        &self,
        entity_id: &str,
    ) -> Result<(), ProviderProxySelectorError> {
        let mut state = self.state.lock().await;

        let provider_uri = state
            .entity_map
            .get(entity_id)
            .ok_or(ProviderProxySelectorError::entity_not_found(format!(
                "Unable to retrieve entity uri for {entity_id}"
            )))?
            .to_owned();

        match state.provider_proxies.entry(provider_uri) {
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
        entity::EntityEndpoint, provider_proxy_selector::ProviderProxySelectorErrorKind,
    };

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
    const OPERATION: &str = "Subscribe";

    #[tokio::test]
    async fn handle_start_provider_proxy_request_return_err_test() {
        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
        let uut = ProviderProxySelectorImpl::new(signal_values_queue);

        let entity = Entity {
            id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
            name: None,
            description: None,
            endpoints: vec![EntityEndpoint {
                operations: vec![OPERATION.to_string()],
                // Emtpy URI for GRPC will cause the test to fail when creating a new proxy
                uri: String::new(),
                protocol: String::from("grpc"),
                context: String::from("context"),
            }],
        };

        let result = uut.create_or_update_proxy(&entity).await;
        println!("{result:?}");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            ProviderProxySelectorErrorKind::ProviderProxyError
        );
    }
}
