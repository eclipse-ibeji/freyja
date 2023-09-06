// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::atomic::{AtomicU8, Ordering},
    sync::Arc,
    sync::Mutex,
    time::Duration,
};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::info;

use crate::config::{EntityConfig, Settings};
use freyja_contracts::provider_proxy::{
    OperationKind, ProviderProxy, ProviderProxyError, SignalValue,
};

const CONFIG_FILE: &str = "config.json";
const SUPPORTED_OPERATIONS: &[OperationKind] = &[OperationKind::Get, OperationKind::Subscribe];

#[derive(Debug)]
pub struct InMemoryMockProviderProxy {
    /// Maps the number of calls to each provider so we can mock changing behavior
    data: HashMap<String, (EntityConfig, AtomicU8)>,

    /// Local cache for keeping track of which entities this provider proxy contains
    entity_operation_map: Arc<Mutex<HashMap<String, OperationKind>>>,

    /// Shared queue for all proxies to push new signal values of entities
    signal_values_queue: Arc<SegQueue<SignalValue>>,

    /// interval_between_signal_generation_ms`: the interval in milliseconds between signal value generation
    interval_between_signal_generation_ms: Duration,
}

impl InMemoryMockProviderProxy {
    /// Creates a new InMemoryMockDigitalTwinAdapter with config from the specified file
    ///
    /// # Arguments
    /// - `config_path`: the path to the config to use
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    pub fn from_config_file<P: AsRef<Path>>(
        config_path: P,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Self, ProviderProxyError> {
        let settings_contents = fs::read_to_string(config_path).map_err(ProviderProxyError::io)?;

        let settings: Settings = serde_json::from_str(settings_contents.as_str())
            .map_err(ProviderProxyError::deserialize)?;
        let config: Vec<EntityConfig> = settings.entity_configs.clone();
        let interval_between_signal_generation_ms = settings.interval_between_signal_generation_ms;

        Self::from_config(
            config,
            signal_values_queue,
            interval_between_signal_generation_ms,
        )
    }

    /// Creates a new InMemoryMockDigitalTwinAdapter with the specified config
    ///
    /// # Arguments
    /// - `config_path`: the config to use
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    /// - `interval_between_signal_generation_ms`: the interval in milliseconds between signal value generation
    pub fn from_config(
        config: Vec<EntityConfig>,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
        interval_between_signal_generation_ms: u64,
    ) -> Result<Self, ProviderProxyError> {
        Ok(Self {
            entity_operation_map: Arc::new(Mutex::new(HashMap::new())),
            data: config
                .into_iter()
                .map(|c| (c.entity_id.clone(), (c, AtomicU8::new(0))))
                .collect(),
            signal_values_queue,
            interval_between_signal_generation_ms: Duration::from_millis(
                interval_between_signal_generation_ms,
            ),
        })
    }

    /// Generates signal value for an entity id
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a signal value
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    /// - `data`: the current data of a provider
    fn generate_signal_value(
        entity_id: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
        data: &HashMap<String, (EntityConfig, AtomicU8)>,
    ) -> Result<(), ProviderProxyError> {
        let (entity_config, counter) = data
            .get(entity_id)
            .ok_or_else(|| format!("Cannot find {entity_id}"))
            .map_err(ProviderProxyError::entity_not_found)?;
        let n = counter.fetch_add(1, Ordering::SeqCst);

        let value = entity_config.values.get_nth(n).to_string();
        let entity_id = String::from(entity_id);

        let new_signal_value = SignalValue { entity_id, value };
        signal_values_queue.push(new_signal_value);
        Ok(())
    }
}

#[async_trait]
impl ProviderProxy for InMemoryMockProviderProxy {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    fn create_new(
        _provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Box<dyn ProviderProxy + Send + Sync>, ProviderProxyError>
    where
        Self: Sized,
    {
        Self::from_config_file(
            Path::new(env!("OUT_DIR")).join(CONFIG_FILE),
            signal_values_queue,
        )
        .map(|r| Box::new(r) as _)
    }

    /// Runs a provider proxy
    async fn run(&self) -> Result<(), ProviderProxyError> {
        info!("Started an InMemoryMockProviderProxy!");

        loop {
            let entities_with_subscribe: Vec<String>;

            {
                entities_with_subscribe = self
                    .entity_operation_map
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .filter(|(_, operation)| *operation == OperationKind::Subscribe)
                    .map(|(entity_id, _)| entity_id)
                    .collect();
            }

            for entity_id in entities_with_subscribe {
                let _ = Self::generate_signal_value(
                    &entity_id,
                    self.signal_values_queue.clone(),
                    &self.data,
                );
            }
            tokio::time::sleep(self.interval_between_signal_generation_ms).await;
        }
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), ProviderProxyError> {
        let operation_result;
        {
            let lock = self.entity_operation_map.lock().unwrap();
            operation_result = lock.get(entity_id).cloned();
        }

        if operation_result.is_none() {
            return Err(ProviderProxyError::unknown(format!(
                "Entity {entity_id} does not have an operation registered"
            )));
        }

        // Only need to handle Get operations since subscribe has already happened
        let operation = operation_result.unwrap();
        if operation == OperationKind::Get {
            let _ = Self::generate_signal_value(
                entity_id,
                self.signal_values_queue.clone(),
                &self.data,
            );
        }

        Ok(())
    }

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `operation`: the operation that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        operation: &OperationKind,
    ) -> Result<(), ProviderProxyError> {
        self.entity_operation_map
            .lock()
            .unwrap()
            .insert(String::from(entity_id), operation.clone());
        Ok(())
    }

    /// Checks if the operation is supported
    ///
    /// # Arguments
    /// - `operation`: check to see if this operation is supported by this provider proxy
    fn is_operation_supported(operation: &OperationKind) -> bool {
        SUPPORTED_OPERATIONS.contains(operation)
    }
}

#[cfg(test)]
mod in_memory_mock_digital_twin_adapter_tests {
    use super::*;

    use crate::config::SensorValueConfig;

    #[test]
    fn from_config_file_returns_err_on_nonexistent_file() {
        let signal_values_queue = Arc::new(SegQueue::new());
        let result =
            InMemoryMockProviderProxy::from_config_file("fake_file.foo", signal_values_queue);
        assert!(result.is_err());
    }

    #[test]
    fn can_get_default_config() {
        let signal_values_queue = Arc::new(SegQueue::new());
        let result = InMemoryMockProviderProxy::create_new("FAKE_URI", signal_values_queue);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_signal_value_returns_correct_values() {
        const STATIC_ID: &str = "static";
        const INCREASING_ID: &str = "increasing";
        const DECREASING_ID: &str = "decreasing";

        let (start, end, delta) = (0.0, 5.0, 1.0);
        let config = vec![
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
        ];

        let signal_values_queue = Arc::new(SegQueue::new());
        let interval_between_signal_generation_ms: u64 = 1000;
        let in_memory_mock_provider_proxy = InMemoryMockProviderProxy::from_config(
            config,
            signal_values_queue.clone(),
            interval_between_signal_generation_ms,
        )
        .unwrap();

        const END_OF_SENSOR_VALUE_CONFIG_ITERATION: i32 = 5;

        // First for loop, we generate signal values for each entity until we've reached the end value of each
        // entity that has the stepwise functionality configured.
        for i in 0..END_OF_SENSOR_VALUE_CONFIG_ITERATION {
            let result = InMemoryMockProviderProxy::generate_signal_value(
                STATIC_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let static_value = signal_values_queue.pop().unwrap();
            assert_eq!(static_value.entity_id, STATIC_ID);
            assert_eq!(static_value.value.parse::<f32>().unwrap(), 42.0);

            let result = InMemoryMockProviderProxy::generate_signal_value(
                INCREASING_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let increasing_value = signal_values_queue.pop().unwrap();
            assert_eq!(increasing_value.entity_id, INCREASING_ID);
            assert_eq!(
                increasing_value.value.parse::<f32>().unwrap(),
                start + delta * i as f32
            );

            let result = InMemoryMockProviderProxy::generate_signal_value(
                DECREASING_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let decreasing_value = signal_values_queue.pop().unwrap();
            assert_eq!(decreasing_value.entity_id, DECREASING_ID);
            assert_eq!(
                decreasing_value.value.parse::<f32>().unwrap(),
                start - delta * i as f32
            );
        }

        // Validating each entity that has the stepwise functionality configured is at its end value
        for _ in 0..END_OF_SENSOR_VALUE_CONFIG_ITERATION {
            let result = InMemoryMockProviderProxy::generate_signal_value(
                STATIC_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let static_value = signal_values_queue.pop().unwrap();
            assert_eq!(static_value.entity_id, STATIC_ID);
            assert_eq!(static_value.value.parse::<f32>().unwrap(), 42.0);

            let result = InMemoryMockProviderProxy::generate_signal_value(
                INCREASING_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let increasing_value = signal_values_queue.pop().unwrap();
            assert_eq!(increasing_value.entity_id, INCREASING_ID);
            assert_eq!(increasing_value.value.parse::<f32>().unwrap(), end);

            let result = InMemoryMockProviderProxy::generate_signal_value(
                DECREASING_ID,
                signal_values_queue.clone(),
                &in_memory_mock_provider_proxy.data,
            );
            assert!(result.is_ok());

            let decreasing_value = signal_values_queue.pop().unwrap();
            assert_eq!(decreasing_value.entity_id, DECREASING_ID);
            assert_eq!(decreasing_value.value.parse::<f32>().unwrap(), -end);
        }
    }
}
