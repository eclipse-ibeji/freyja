// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, DigitalTwinAdapterErrorKind, FindByIdRequest,
        FindByIdResponse,
    },
    out_dir,
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};

/// In-memory mock that mocks finding endpoint info about entities
/// through find by id
pub struct InMemoryMockDigitalTwinAdapter {
    /// The adapter config
    config: Config,
}

impl InMemoryMockDigitalTwinAdapter {
    /// Creates a new InMemoryMockDigitalTwinAdapter with the specified config
    ///
    /// # Arguments
    /// - `config`: the config to use
    pub fn from_config(config: Config) -> Result<Self, DigitalTwinAdapterError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl DigitalTwinAdapter for InMemoryMockDigitalTwinAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    ///
    /// # Arguments
    /// - `_selector`: the service discovery adapter selector to use (unused by this adapter)
    fn create_new(
        _selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, DigitalTwinAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            DigitalTwinAdapterError::io,
            DigitalTwinAdapterError::deserialize,
        )?;

        Self::from_config(config)
    }

    /// Gets the entity information based on the request
    ///
    /// # Arguments
    /// - `request`: the request to send
    async fn find_by_id(
        &self,
        request: FindByIdRequest,
    ) -> Result<FindByIdResponse, DigitalTwinAdapterError> {
        self.config
            .values
            .iter()
            .find(|entity_config| entity_config.entity.id == request.entity_id)
            .map(|entity_config| FindByIdResponse {
                entity: entity_config.entity.clone(),
            })
            .ok_or(DigitalTwinAdapterErrorKind::EntityNotFound.into())
    }
}

#[cfg(test)]
mod in_memory_mock_digital_twin_adapter_tests {
    use super::*;

    use crate::config::EntityConfig;
    use freyja_common::entity::{Entity, EntityEndpoint};
    use freyja_test_common::mocks::MockServiceDiscoveryAdapterSelector;

    const OPERATION: &str = "Subscribe";

    #[test]
    fn can_create_new() {
        let result = InMemoryMockDigitalTwinAdapter::create_new(Arc::new(Mutex::new(
            MockServiceDiscoveryAdapterSelector::new(),
        )));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_by_id_test() {
        const ENTITY_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

        let config = Config {
            values: vec![EntityConfig {
                entity: Entity {
                    name: None,
                    id: ENTITY_ID.to_string(),
                    description: None,
                    endpoints: vec![EntityEndpoint {
                        protocol: String::from("in-memory"),
                        operations: vec![OPERATION.to_string()],
                        uri: String::from("http://0.0.0.0:1111"), // Devskim: ignore DS137138
                        context: String::from("context"),
                    }],
                },
            }],
        };

        let in_memory_digital_twin_adapter = InMemoryMockDigitalTwinAdapter { config };
        let request = FindByIdRequest {
            entity_id: String::from(ENTITY_ID),
        };

        let response = in_memory_digital_twin_adapter
            .find_by_id(request)
            .await
            .unwrap();

        assert_eq!(response.entity.id, ENTITY_ID);
        assert_eq!(response.entity.endpoints.len(), 1);
        let endpoint = response.entity.endpoints.first().unwrap();
        assert_eq!(endpoint.operations.len(), 1);
        let operation = endpoint.operations.first().unwrap();
        assert_eq!(operation, OPERATION);
    }
}
