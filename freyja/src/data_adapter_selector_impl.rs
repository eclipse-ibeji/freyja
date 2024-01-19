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
    data_adapter::{DataAdapter, DataAdapterFactory, EntityRegistration},
    data_adapter_selector::{
        DataAdapterSelector, DataAdapterSelectorError, DataAdapterSelectorErrorKind,
    },
    entity::Entity,
    signal_store::SignalStore,
};

const LOOPBACK_MAX: u16 = 10;

/// Represents the state of the DataAdapterSelector and allows for simplified access through a mutex
struct DataAdapterSelectorState {
    /// A map of entity uris to data adapters
    data_adapters: HashMap<String, Arc<dyn DataAdapter + Send + Sync>>,

    /// A map of entity id to provider uri
    entity_map: HashMap<String, String>,
}

/// The data adapter selector selects which data adapter to create based on protocol and operation.
/// This struct is **not** thread-safe and should be shared with `Arc<Mutex<DataAdapterSelectorImpl>>`.
pub struct DataAdapterSelectorImpl {
    /// The set of factories that have been registered
    factories: Vec<Box<dyn DataAdapterFactory + Send + Sync>>,

    /// The DataAdapterSelector's state
    state: Mutex<DataAdapterSelectorState>,

    /// The signal store used for creating the adapters
    signals: Arc<SignalStore>,
}

impl DataAdapterSelectorImpl {
    /// Instantiates the data adapter selector
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    pub fn new(signals: Arc<SignalStore>) -> Self {
        DataAdapterSelectorImpl {
            factories: Vec::new(),
            state: Mutex::new(DataAdapterSelectorState {
                data_adapters: HashMap::new(),
                entity_map: HashMap::new(),
            }),
            signals,
        }
    }
}

#[async_trait]
impl DataAdapterSelector for DataAdapterSelectorImpl {
    /// Registers a `DataAdapterFactory` with this selector.
    ///
    /// # Arguments
    /// - `factory`: the factory to register
    fn register(
        &mut self,
        factory: Box<dyn DataAdapterFactory + Send + Sync + 'static>,
    ) -> Result<(), DataAdapterSelectorError> {
        self.factories.push(factory);
        Ok(())
    }

    /// Updates an existing data adapter to include an entity if possible,
    /// otherwise creates a new data adapter to handle that entity.
    ///
    /// # Arguments
    /// - `entity`: the entity that the adapter should handle
    async fn create_or_update_adapter(
        &self,
        entity: &Entity,
    ) -> Result<(), DataAdapterSelectorError> {
        // Keeps track of max depth loopback can reach.
        let mut loopback_count = 0;
        let mut current_entity = entity.to_owned();

        // The selector will loop (up to max attempts) until a data adapter registers the entity.
        // Will break out of loop on an error.
        'loopback: while loopback_count < LOOPBACK_MAX {
            let mut state = self.state.lock().await;

            // If a data adapter already exists for one of this entity's uris,
            // then we notify that adapter to include this new entity
            for endpoint in current_entity.endpoints.iter() {
                if let Some(data_adapter) = state.data_adapters.get(&endpoint.uri) {
                    debug!("A data adapter for {} already exists", &endpoint.uri);

                    let entity_registration = data_adapter
                        .register_entity(&current_entity.id, endpoint)
                        .await
                        .map_err(DataAdapterSelectorError::communication)?;

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
                            // The adapter is requesting a loopback with new entity information
                            current_entity = new_entity.to_owned();
                            loopback_count += 1;

                            debug!("Loopback requested with: {current_entity:?}. Loopback count is: {loopback_count}.");

                            continue 'loopback;
                        }
                    }
                }
            }

            // If there's not a data adapter we can reuse, find the right factory to create a new one
            let (data_adapter, endpoint) = {
                let mut result = None;
                for factory in self.factories.iter() {
                    if let Some(endpoint) = factory.is_supported(&current_entity) {
                        let adapter = factory
                            .create_adapter(&endpoint.uri, self.signals.clone())
                            .map_err(DataAdapterSelectorError::data_adapter_error)?;
                        result = Some((adapter, endpoint));
                    }
                }

                result.ok_or::<DataAdapterSelectorError>(
                    DataAdapterSelectorErrorKind::OperationNotSupported.into(),
                )?
            };

            // Start the data adapter
            data_adapter
                .start()
                .await
                .map_err(DataAdapterSelectorError::data_adapter_error)?;

            // Register the entity with the data adapter
            let entity_registration = data_adapter
                .register_entity(&current_entity.id, &endpoint)
                .await
                .map_err(DataAdapterSelectorError::data_adapter_error)?;

            // As long as there was not an error with registration, add the adapter to the map
            state
                .data_adapters
                .insert(endpoint.uri.clone(), data_adapter);

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
                    // The adapter is requesting a loopback with new entity information
                    current_entity = new_entity.to_owned();
                    loopback_count += 1;

                    debug!("Loopback requested with: {current_entity:?}. Loopback count is: {loopback_count}.");

                    continue 'loopback;
                }
            }
        }

        Err(DataAdapterSelectorError::data_adapter_error(format!(
            "Unable to select a data adapter: reached maximum loopback attempts ({LOOPBACK_MAX})."
        )))
    }

    /// Requests that the value of an entity be published as soon as possible
    ///
    /// # Arguments
    /// - `entity_id`: the entity to request
    async fn request_entity_value(&self, entity_id: &str) -> Result<(), DataAdapterSelectorError> {
        let mut state = self.state.lock().await;

        let provider_uri = state
            .entity_map
            .get(entity_id)
            .ok_or(DataAdapterSelectorError::entity_not_found(format!(
                "Unable to retrieve entity uri for {entity_id}"
            )))?
            .to_owned();

        match state.data_adapters.entry(provider_uri) {
            Entry::Occupied(data_adapter) => data_adapter
                .get()
                .send_request_to_provider(entity_id)
                .await
                .map_err(DataAdapterSelectorError::communication),
            Entry::Vacant(_) => Err(DataAdapterSelectorError::entity_not_found(format!(
                "Data adapter for {entity_id} is not available"
            ))),
        }
    }
}

#[cfg(test)]
mod data_adapter_selector_tests {
    use super::*;

    use freyja_common::{
        data_adapter_selector::DataAdapterSelectorErrorKind, entity::EntityEndpoint,
    };
    use grpc_data_adapter::grpc_data_adapter_factory::GRPCDataAdapterFactory;

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";
    const OPERATION: &str = "Subscribe";

    #[tokio::test]
    async fn handle_start_data_adapter_request_return_err_test() {
        let signals: Arc<SignalStore> = Arc::new(SignalStore::new());
        let mut uut = DataAdapterSelectorImpl::new(signals);
        uut.register(Box::new(GRPCDataAdapterFactory::create_new().unwrap()))
            .unwrap();

        let entity = Entity {
            id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
            name: None,
            description: None,
            endpoints: vec![EntityEndpoint {
                operations: vec![OPERATION.to_string()],
                // Emtpy URI for GRPC will cause the test to fail when creating a new adapter
                uri: String::new(),
                protocol: String::from("grpc"),
                context: String::from("context"),
            }],
        };

        let result = uut.create_or_update_adapter(&entity).await;
        println!("{result:?}");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            DataAdapterSelectorErrorKind::DataAdapterError
        );
    }
}
