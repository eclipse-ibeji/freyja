// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU8, Ordering},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use freyja_common::{config_utils, out_dir, signal_store::SignalStore};
use log::{info, warn};
use tokio::sync::Mutex;

use crate::{
    config::{Config, EntityConfig},
    GET_OPERATION, SUBSCRIBE_OPERATION,
};

use freyja_build_common::config_file_stem;
use freyja_common::{
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterErrorKind, EntityRegistration},
    entity::EntityEndpoint,
};

pub struct InMemoryMockDataAdapter {
    /// Maps the number of calls to each provider so we can mock changing behavior
    data: Arc<Mutex<HashMap<String, (EntityConfig, AtomicU8)>>>,

    /// Local cache for keeping track of which entities this data adapter contains
    entity_operation_map: Arc<Mutex<HashMap<String, String>>>,

    /// Shared queue for all data adapters to push new signal values of entities
    signals: Arc<SignalStore>,

    /// The frequency between updates to signal values
    signal_update_frequency: Duration,
}

impl InMemoryMockDataAdapter {
    /// Creates a new InMemoryMockDigitalTwinAdapter with the specified config
    ///
    /// # Arguments
    /// - `config`: the config to use
    /// - `signals`: the shared signal store
    /// - `interval_between_signal_generation_ms`: the interval in milliseconds between signal value generation
    pub fn from_config(
        config: Config,
        signals: Arc<SignalStore>,
    ) -> Result<Self, DataAdapterError> {
        let data = config
            .entities
            .into_iter()
            .map(|c| (c.entity_id.clone(), (c, AtomicU8::new(0))))
            .collect();

        Ok(Self {
            entity_operation_map: Arc::new(Mutex::new(HashMap::new())),
            data: Arc::new(Mutex::new(data)),
            signals,
            signal_update_frequency: Duration::from_millis(config.signal_update_frequency_ms),
        })
    }

    /// Generates signal value for an entity id
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a signal value
    /// - `signals`: the shared signal store
    /// - `data`: the current data of a provider
    fn generate_signal_value(
        entity_id: &str,
        signals: Arc<SignalStore>,
        data: &HashMap<String, (EntityConfig, AtomicU8)>,
    ) -> Result<(), DataAdapterError> {
        let (entity_config, counter) = data
            .get(entity_id)
            .ok_or_else(|| format!("Cannot find {entity_id}"))
            .map_err(DataAdapterError::entity_not_found)?;
        let n = counter.fetch_add(1, Ordering::SeqCst);

        let value = entity_config.values.get_nth(n).to_string();
        let entity_id = String::from(entity_id);

        signals
            .set_value(entity_id, value)
            .map(|_| ())
            .ok_or(DataAdapterErrorKind::EntityNotFound.into())
    }
}

#[async_trait]
impl DataAdapter for InMemoryMockDataAdapter {
    /// Creates a data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signals`: the shared signal store
    fn create_new(_provider_uri: &str, signals: Arc<SignalStore>) -> Result<Self, DataAdapterError>
    where
        Self: Sized,
    {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            DataAdapterError::io,
            DataAdapterError::deserialize,
        )?;

        Self::from_config(config, signals)
    }

    /// Starts a data adapter
    async fn start(&self) -> Result<(), DataAdapterError> {
        let entity_operation_map = self.entity_operation_map.clone();
        let signals = self.signals.clone();
        let signal_update_frequency = self.signal_update_frequency;
        let data = self.data.clone();

        tokio::spawn(async move {
            loop {
                let entities_with_subscribe: Vec<String>;

                {
                    entities_with_subscribe = entity_operation_map
                        .lock()
                        .await
                        .clone()
                        .into_iter()
                        .filter(|(_, operation)| *operation == SUBSCRIBE_OPERATION)
                        .map(|(entity_id, _)| entity_id)
                        .collect();
                }

                {
                    let data = data.lock().await;
                    for entity_id in entities_with_subscribe {
                        if let Err(e) =
                            Self::generate_signal_value(&entity_id, signals.clone(), &data)
                        {
                            warn!("Attempt to set value for non-existent entity {entity_id}: {e}");
                        }
                    }
                }

                tokio::time::sleep(signal_update_frequency).await;
            }
        });

        info!("Started an InMemoryMockDataAdapter!");

        Ok(())
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), DataAdapterError> {
        let operation_result;
        {
            let lock = self.entity_operation_map.lock().await;
            operation_result = lock.get(entity_id).cloned();
        }

        if operation_result.is_none() {
            return Err(DataAdapterError::unknown(format!(
                "Entity {entity_id} does not have an operation registered"
            )));
        }

        // Only need to handle Get operations since subscribe has already happened
        let operation = operation_result.unwrap();

        let data = self.data.lock().await;
        if operation == GET_OPERATION {
            let _ = Self::generate_signal_value(entity_id, self.signals.clone(), &data);
        }

        Ok(())
    }

    /// Registers an entity id to a local cache inside a data adapter to keep track of which entities a data adapter contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<EntityRegistration, DataAdapterError> {
        // Prefer subscribe if present
        let selected_operation = {
            let mut result = None;
            for operation in endpoint.operations.iter() {
                if operation == SUBSCRIBE_OPERATION {
                    result = Some(SUBSCRIBE_OPERATION);
                    break;
                } else if operation == GET_OPERATION {
                    // Set result, but don't break the loop in case there's a subscribe operation later in the list
                    result = Some(GET_OPERATION);
                }
            }

            result.ok_or::<DataAdapterError>(DataAdapterErrorKind::OperationNotSupported.into())?
        };

        self.entity_operation_map
            .lock()
            .await
            .insert(String::from(entity_id), String::from(selected_operation));

        Ok(EntityRegistration::Registered)
    }
}

#[cfg(test)]
mod in_memory_mock_data_adapter_tests {
    use freyja_common::signal::SignalPatch;

    use super::*;

    use crate::config::SensorValueConfig;

    fn validate_signal(signals: Arc<SignalStore>, id: &str, value: f32) {
        let signal = signals.get(&id.to_owned());
        assert!(signal.is_some());

        let signal = signal.unwrap();
        assert_eq!(signal.id, id);
        assert!(signal.value.is_some());

        let signal_value = signal.value.unwrap().parse::<f32>();
        assert!(signal_value.is_ok());
        assert_eq!(signal_value.unwrap(), value);
    }

    #[test]
    fn can_create_new() {
        let signals = Arc::new(SignalStore::new());
        let result = InMemoryMockDataAdapter::create_new("FAKE_URI", signals);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_signal_value_returns_correct_values() {
        const STATIC_ID: &str = "static";
        const INCREASING_ID: &str = "increasing";
        const DECREASING_ID: &str = "decreasing";

        let (start, end, delta) = (0.0, 5.0, 1.0);
        let config = Config {
            signal_update_frequency_ms: 1000,
            entities: vec![
                EntityConfig {
                    entity_id: String::from(STATIC_ID),
                    values: SensorValueConfig::Static(42.0),
                },
                EntityConfig {
                    entity_id: String::from(INCREASING_ID),
                    values: SensorValueConfig::Stepwise { start, end, delta },
                },
                EntityConfig {
                    entity_id: String::from(DECREASING_ID),
                    values: SensorValueConfig::Stepwise {
                        start,
                        end: -end,
                        delta: -delta,
                    },
                },
            ],
        };

        let signals = Arc::new(SignalStore::new());
        signals.add(
            [
                SignalPatch {
                    id: STATIC_ID.to_owned(),
                    ..Default::default()
                },
                SignalPatch {
                    id: INCREASING_ID.to_owned(),
                    ..Default::default()
                },
                SignalPatch {
                    id: DECREASING_ID.to_owned(),
                    ..Default::default()
                },
            ]
            .into_iter(),
        );
        let in_memory_mock_data_adapter =
            InMemoryMockDataAdapter::from_config(config, signals.clone()).unwrap();

        const END_OF_SENSOR_VALUE_CONFIG_ITERATION: i32 = 5;

        // First for loop, we generate signal values for each entity until we've reached the end value of each
        // entity that has the stepwise functionality configured.
        let data = in_memory_mock_data_adapter.data.lock().await;
        for i in 0..END_OF_SENSOR_VALUE_CONFIG_ITERATION {
            let result =
                InMemoryMockDataAdapter::generate_signal_value(STATIC_ID, signals.clone(), &data);
            assert!(result.is_ok());

            validate_signal(signals.clone(), STATIC_ID, 42.0);

            let result = InMemoryMockDataAdapter::generate_signal_value(
                INCREASING_ID,
                signals.clone(),
                &data,
            );
            assert!(result.is_ok());

            validate_signal(signals.clone(), INCREASING_ID, start + delta * (i as f32));

            let result = InMemoryMockDataAdapter::generate_signal_value(
                DECREASING_ID,
                signals.clone(),
                &data,
            );
            assert!(result.is_ok());

            validate_signal(signals.clone(), DECREASING_ID, start - delta * (i as f32));
        }

        // Validating each entity that has the stepwise functionality configured is at its end value
        for _ in 0..END_OF_SENSOR_VALUE_CONFIG_ITERATION {
            let result =
                InMemoryMockDataAdapter::generate_signal_value(STATIC_ID, signals.clone(), &data);
            assert!(result.is_ok());

            validate_signal(signals.clone(), STATIC_ID, 42.0);

            let result = InMemoryMockDataAdapter::generate_signal_value(
                INCREASING_ID,
                signals.clone(),
                &data,
            );
            assert!(result.is_ok());

            validate_signal(signals.clone(), INCREASING_ID, end);

            let result = InMemoryMockDataAdapter::generate_signal_value(
                DECREASING_ID,
                signals.clone(),
                &data,
            );
            assert!(result.is_ok());

            validate_signal(signals.clone(), DECREASING_ID, -end);
        }
    }
}
