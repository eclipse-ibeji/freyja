// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::Mutex;

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils, out_dir,
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, FindByIdRequest, FindByIdResponse,
    },
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};
use mock_digital_twin::ENTITY_QUERY_PATH;

/// Mocks a Digital Twin Adapter that calls the mocks/mock_digital_twin
/// to get entity access info.
pub struct MockDigitalTwinAdapter {
    /// The adapter config
    config: Config,

    /// Async Reqwest HTTP Client
    client: Client,
}

impl MockDigitalTwinAdapter {
    /// Creates a new MockDigitalTwinAdapter with the specified config
    ///
    /// # Arguments
    /// - `config`: the config to use
    pub fn from_config(config: Config) -> Result<Self, DigitalTwinAdapterError> {
        Ok(Self {
            config,
            client: Client::new(),
        })
    }

    /// Helper to map HTTP error codes to our own error type
    ///
    /// # Arguments
    /// - `error`: the HTTP error to translate
    fn map_status_err(error: reqwest::Error) -> DigitalTwinAdapterError {
        match error.status() {
            Some(reqwest::StatusCode::NOT_FOUND) => {
                DigitalTwinAdapterError::entity_not_found(error)
            }
            _ => DigitalTwinAdapterError::communication(error),
        }
    }
}

#[async_trait]
impl DigitalTwinAdapter for MockDigitalTwinAdapter {
    /// Creates a new instance of a MockDigitalTwinAdapter
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

    /// Gets the info of an entity via an HTTP request.
    ///
    /// # Arguments
    /// - `request`: the request to send to the mock digital twin server
    async fn find_by_id(
        &self,
        request: FindByIdRequest,
    ) -> Result<FindByIdResponse, DigitalTwinAdapterError> {
        let target = format!(
            "{}{ENTITY_QUERY_PATH}{}",
            self.config.digital_twin_service_uri, request.entity_id
        );

        self.client
            .get(&target)
            .send()
            .await
            .map_err(DigitalTwinAdapterError::communication)?
            .error_for_status()
            .map_err(Self::map_status_err)?
            .json::<FindByIdResponse>()
            .await
            .map_err(DigitalTwinAdapterError::deserialize)
    }
}
