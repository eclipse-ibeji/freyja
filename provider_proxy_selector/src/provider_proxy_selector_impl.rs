// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use log::debug;
use tokio::sync::Mutex;

use freyja_common::{
    entity::Entity,
    provider_proxy::{EntityRegistration, ProviderProxy, ProviderProxyFactory},
    provider_proxy_selector::{
        ProviderProxySelector, ProviderProxySelectorError, ProviderProxySelectorErrorKind,
    },
    signal_store::SignalStore,
};

use crate::PROXY_SELECTOR_LOOPBACK_MAX;

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

    /// The signal store used for creating the proxies
    signals: Arc<SignalStore>,
}

impl ProviderProxySelectorImpl {
    /// Instantiates the provider proxy selector
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    pub fn new(signals: Arc<SignalStore>) -> Self {
        ProviderProxySelectorImpl {
            factories: Vec::new(),
            state: Mutex::new(ProviderProxySelectorState {
                provider_proxies: HashMap::new(),
                entity_map: HashMap::new(),
            }),
            signals,
        }
    }
}

#[async_trait]
impl ProviderProxySelector for ProviderProxySelectorImpl {
    /// Registers a `ProviderProxyFactory` with this selector.
    fn register<TFactory: ProviderProxyFactory + Send + Sync + 'static>(
        &mut self,
    ) -> Result<(), ProviderProxySelectorError> {
        let factory =
            TFactory::create_new().map_err(ProviderProxySelectorError::provider_proxy_error)?;
        self.factories.push(Box::new(factory));
        Ok(())
    }

    /// Updates an existing proxy for an entity if possible,
    /// otherwise creates a new proxy to handle that entity.
    ///
    /// # Arguments
    /// - `entity`: the entity that the proxy should handle
    async fn create_or_update_proxy(
        &self,
        entity: &Entity,
    ) -> Result<(), ProviderProxySelectorError> {
        // Keeps track of max depth loopback can reach.
        let mut loopback_count = 0;
        let mut current_entity = entity.to_owned();

        // Proxy selector will loop (up to max attempts) until a proxy registers the entity.
        // Will break out of loop on an error.
        'loopback: while loopback_count < PROXY_SELECTOR_LOOPBACK_MAX {
            let mut state = self.state.lock().await;

            // If a provider proxy already exists for one of this entity's uris,
            // then we notify that proxy to include this new entity
            for endpoint in current_entity.endpoints.iter() {
                if let Some(provider_proxy) = state.provider_proxies.get(&endpoint.uri) {
                    debug!("A provider proxy for {} already exists", &endpoint.uri);

                    let entity_registration = provider_proxy
                        .register_entity(&current_entity.id, endpoint)
                        .await
                        .map_err(ProviderProxySelectorError::communication)?;

                    match entity_registration {
                        EntityRegistration::Registered => {
                            // There was a successful registration of the entity.
                            // The entity is added to the map and the selector returns.
                            state.entity_map.insert(
                                String::from(&current_entity.id),
                                String::from(&endpoint.uri),
                            );

                            return Ok(());
                        }
                        EntityRegistration::Loopback(new_entity) => {
                            // The proxy is requesting a loopback with new entity information
                            current_entity = new_entity.to_owned();
                            loopback_count += 1;

                            debug!("Loopback requested with: {current_entity:?}. Loopback count is: {loopback_count}.");

                            continue 'loopback;
                        }
                    }
                }
            }

            // If there's not a proxy we can reuse, find the right factory to create a new one
            let (provider_proxy, endpoint) = {
                let mut result = None;
                for factory in self.factories.iter() {
                    if let Some(endpoint) = factory.is_supported(&current_entity) {
                        let proxy = factory
                            .create_proxy(&endpoint.uri, self.signals.clone())
                            .map_err(ProviderProxySelectorError::provider_proxy_error)?;
                        result = Some((proxy, endpoint));
                    }
                }

                result.ok_or::<ProviderProxySelectorError>(
                    ProviderProxySelectorErrorKind::OperationNotSupported.into(),
                )?
            };

            // Start the provider proxy
            provider_proxy
                .start()
                .await
                .map_err(ProviderProxySelectorError::provider_proxy_error)?;

            // Register the entity with the provider proxy
            let entity_registration = provider_proxy
                .register_entity(&current_entity.id, &endpoint)
                .await
                .map_err(ProviderProxySelectorError::provider_proxy_error)?;

            // As long as there was not an error with registration, add proxy to map
            state
                .provider_proxies
                .insert(endpoint.uri.clone(), provider_proxy);

            match entity_registration {
                EntityRegistration::Registered => {
                    // There was a successful registration of the entity.
                    // The entity is added to the map and the selector returns.
                    state.entity_map.insert(
                        String::from(&current_entity.id),
                        String::from(&endpoint.uri),
                    );

                    return Ok(());
                }
                EntityRegistration::Loopback(new_entity) => {
                    // The proxy is requesting a loopback with new entity information
                    current_entity = new_entity.to_owned();
                    loopback_count += 1;

                    debug!("Loopback requested with: {current_entity:?}. Loopback count is: {loopback_count}.");

                    continue 'loopback;
                }
            }
        }

        Err(ProviderProxySelectorError::provider_proxy_error(format!(
            "Unable to select proxy, reached max attempts of: {PROXY_SELECTOR_LOOPBACK_MAX}."
        )))
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

    use freyja_common::{
        entity::EntityEndpoint, provider_proxy_selector::ProviderProxySelectorErrorKind,
    };
    use grpc_provider_proxy_v1::grpc_provider_proxy_factory::GRPCProviderProxyFactory;

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
    const OPERATION: &str = "Subscribe";

    #[tokio::test]
    async fn handle_start_provider_proxy_request_return_err_test() {
        let signals: Arc<SignalStore> = Arc::new(SignalStore::new());
        let mut uut = ProviderProxySelectorImpl::new(signals);
        uut.register::<GRPCProviderProxyFactory>().unwrap();

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
