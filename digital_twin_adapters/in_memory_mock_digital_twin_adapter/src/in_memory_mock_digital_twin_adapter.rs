// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fs, path::Path, sync::Arc, sync::Mutex, time::Duration};

use async_trait::async_trait;
use log::error;

use crate::config::EntityConfig;
use dts_contracts::{
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, DigitalTwinAdapterErrorKind,
        GetDigitalTwinProviderRequest, GetDigitalTwinProviderResponse,
    },
    entity::{Entity, EntityID},
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
};

const CONFIG_FILE: &str = "config.json";

/// In-memory mock that mocks finding endpoint info about entities
/// through find by id
pub struct InMemoryMockDigitalTwinAdapter {
    /// Stores configs about entities
    data: Vec<EntityConfig>,
}

impl InMemoryMockDigitalTwinAdapter {
    /// Creates a new InMemoryMockDigitalTwinAdapter with config from the specified file
    ///
    /// # Arguments
    /// - `config_path`: the path to the config to use
    pub fn from_config_file<P: AsRef<Path>>(
        config_path: P,
    ) -> Result<Self, DigitalTwinAdapterError> {
        let config_contents =
            fs::read_to_string(config_path).map_err(DigitalTwinAdapterError::io)?;

        let config: Vec<EntityConfig> = serde_json::from_str(config_contents.as_str())
            .map_err(DigitalTwinAdapterError::deserialize)?;

        Self::from_config(config)
    }

    /// Creates a new InMemoryMockDigitalTwinAdapter with the specified config
    ///
    /// # Arguments
    /// - `config`: the config to use
    pub fn from_config(config: Vec<EntityConfig>) -> Result<Self, DigitalTwinAdapterError> {
        Ok(Self { data: config })
    }
}

#[async_trait]
impl DigitalTwinAdapter for InMemoryMockDigitalTwinAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError> {
        Self::from_config_file(Path::new(env!("OUT_DIR")).join(CONFIG_FILE))
            .map(|r| Box::new(r) as _)
    }

    /// Gets the entity information based on the request
    ///
    /// # Arguments
    /// - `request`: the request to send
    async fn find_by_id(
        &self,
        request: GetDigitalTwinProviderRequest,
    ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError> {
        self.data
            .iter()
            .find(|entity_config| entity_config.entity.id == request.entity_id)
            .map(|entity_config| GetDigitalTwinProviderResponse {
                entity: entity_config.entity.clone(),
            })
            .ok_or(DigitalTwinAdapterErrorKind::EntityNotFound.into())
    }

    /// Run as a client to the mock in-vehicle digital twin provider
    ///
    /// # Arguments
    /// - `entity_map`: shared map of entity ID to entity information
    /// - `sleep_interval`: the interval in milliseconds between finding the access info of entities
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    async fn run(
        &self,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        sleep_interval: Duration,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
    ) -> Result<(), DigitalTwinAdapterError> {
        loop {
            let mut entity_map_update;
            {
                entity_map_update = entity_map.lock().unwrap().clone();
            }

            // Update the copy of entity map if it contains an entity that has no information
            for (entity_id, entity) in entity_map_update.iter_mut() {
                if entity.is_some() {
                    continue;
                }

                // Update the copy of entity map if we're able to find the info of an entity after doing find_by_id
                let request = GetDigitalTwinProviderRequest {
                    entity_id: entity_id.clone(),
                };

                match self.find_by_id(request).await {
                    Ok(response) => {
                        let entity_info = response.entity.clone();
                        let (id, uri, protocol, operation) = (
                            entity_info.id,
                            entity_info.uri,
                            entity_info.protocol,
                            entity_info.operation,
                        );
                        *entity = Some(response.entity);

                        // Notify the provider proxy selector to start a proxy
                        let request = ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(
                            id, uri, protocol, operation,
                        );
                        provider_proxy_selector_request_sender
                            .send_request_to_provider_proxy_selector(request);
                    }
                    Err(err) => {
                        error!("{err}");
                        *entity = None
                    }
                };
            }

            {
                *entity_map.lock().unwrap() = entity_map_update;
            }
            tokio::time::sleep(sleep_interval).await;
        }
    }
}

#[cfg(test)]
mod in_memory_mock_digital_twin_adapter_tests {
    use super::*;

    use dts_contracts::provider_proxy::OperationKind;

    #[test]
    fn from_config_file_returns_err_on_nonexistent_file() {
        let result = InMemoryMockDigitalTwinAdapter::from_config_file("fake_file.foo");
        assert!(result.is_err());
    }

    #[test]
    fn can_get_default_config() {
        let result = InMemoryMockDigitalTwinAdapter::create_new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_by_id_test() {
        const ENTITY_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

        let data = vec![EntityConfig {
            entity: Entity {
                id: String::from(ENTITY_ID),
                name: None,
                uri: String::from("http://0.0.0.0:1111"), // Devskim: ignore DS137138
                description: None,
                operation: OperationKind::Subscribe,
                protocol: String::from("in-memory"),
            },
        }];

        let in_memory_digital_twin_adapter = InMemoryMockDigitalTwinAdapter { data };
        let request = GetDigitalTwinProviderRequest {
            entity_id: String::from(ENTITY_ID),
        };
        let response = in_memory_digital_twin_adapter
            .find_by_id(request)
            .await
            .unwrap();
        assert_eq!(response.entity.id, ENTITY_ID);
        assert_eq!(response.entity.operation, OperationKind::Subscribe);
    }
}
