// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, path::Path};

use async_trait::async_trait;
use reqwest::Client;
use serde_json;

use crate::config::{Settings, CONFIG_FILE};
use dts_contracts::digital_twin_adapter::{
    DigitalTwinAdapter, DigitalTwinAdapterError, GetDigitalTwinProviderRequest,
    GetDigitalTwinProviderResponse,
};
use dts_contracts::entity::{Entity, EntityID};
use dts_contracts::provider_proxy_request::ProviderProxySelectorRequestSender;
use mock_digital_twin::ENTITY_QUERY_PATH;

/// Mocks a Digital Twin Adapter that calls the mocks/mock_digital_twin
/// to get entity access info.
pub struct MockDigitalTwinAdapter {
    /// Base uri for finding an entity's info
    base_uri_for_digital_twin_server: String,

    /// Async Reqwest HTTP Client
    client: Client,
}

impl MockDigitalTwinAdapter {
    /// Creates a new instance of a MockDigitalTwinAdapter
    ///
    /// # Arguments
    /// - `base_uri_for_entity_info`: the base uri for finding entities' access info
    pub fn with_uri(base_uri_for_entity_info: &str) -> Self {
        Self {
            base_uri_for_digital_twin_server: String::from(base_uri_for_entity_info),
            client: reqwest::Client::new(),
        }
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
    fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError> {
        let settings_content = fs::read_to_string(Path::new(env!("OUT_DIR")).join(CONFIG_FILE))
            .map_err(DigitalTwinAdapterError::io)?;
        let settings: Settings = serde_json::from_str(settings_content.as_str())
            .map_err(DigitalTwinAdapterError::deserialize)?;

        Ok(Box::new(Self::with_uri(
            &settings.base_uri_for_digital_twin_server,
        )))
    }

    /// Gets the info of an entity via an HTTP request.
    ///
    /// # Arguments
    /// - `request`: the request to send to the mock digital twin server
    async fn find_by_id(
        &self,
        request: GetDigitalTwinProviderRequest,
    ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError> {
        let target = format!(
            "{}{ENTITY_QUERY_PATH}{}",
            self.base_uri_for_digital_twin_server, request.entity_id
        );
        self.client
            .get(&target)
            .send()
            .await
            .map_err(DigitalTwinAdapterError::communication)?
            .error_for_status()
            .map_err(Self::map_status_err)?
            .json::<GetDigitalTwinProviderResponse>()
            .await
            .map_err(DigitalTwinAdapterError::deserialize)
    }

    /// Run as a client to contact the mocks/mock_digital_twin for finding an entity's info
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
            self.update_entity_map(entity_map.clone(), provider_proxy_selector_request_sender.clone()).await?;
            tokio::time::sleep(sleep_interval).await;
        }
    }
}
