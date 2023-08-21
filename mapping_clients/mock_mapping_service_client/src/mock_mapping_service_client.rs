// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{fs, path::Path, time::Duration};

use async_trait::async_trait;
use reqwest::Client;

use crate::mock_mapping_service_client_config::{Config, CONFIG_FILE};
use common::utils::execute_with_retry;
use dts_contracts::mapping_client::*;

/// Mocks a mapping provider in memory
pub struct MockMappingServiceClient {
    /// The base URL for requests
    base_url: String,

    /// An internal HTTP client
    client: Client,

    /// Max retries for connecting to Mock Mapping Service
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,
}

impl MockMappingServiceClient {
    /// Creates a new instance of a CloudAdapter with default settings
    pub fn with_url(config: Config) -> Self {
        Self {
            base_url: config.mock_mapping_service_url,
            client: reqwest::Client::new(),
            max_retries: config.max_retries,
            retry_interval_ms: config.retry_interval_ms,
        }
    }
}

#[async_trait]
impl MappingClient for MockMappingServiceClient {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Box<dyn MappingClient>, MappingClientError> {
        let config_contents =
            fs::read_to_string(Path::new(env!("OUT_DIR")).join(CONFIG_FILE)).unwrap();
        let config: Config = serde_json::from_str(config_contents.as_str()).unwrap();

        Ok(Box::new(Self::with_url(config)))
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
    async fn check_for_work(
        &self,
        _request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingClientError> {
        let target = format!("{}/work", self.base_url);

        execute_with_retry(
            self.max_retries,
            Duration::from_millis(self.retry_interval_ms),
            || self.client.get(&target).send(),
            Some(String::from("Checking for work from the mapping service")),
        )
        .await
        .map_err(MappingClientError::communication)?
        .error_for_status()
        .map_err(MappingClientError::communication)?
        .json::<CheckForWorkResponse>()
        .await
        .map_err(MappingClientError::deserialize)
    }

    /// Sends the provider inventory to the mapping service
    ///
    /// # Arguments
    ///
    /// - `inventory`: the providers to send
    async fn send_inventory(
        &self,
        inventory: SendInventoryRequest,
    ) -> Result<SendInventoryResponse, MappingClientError> {
        let target = format!("{}/inventory", self.base_url);
        self.client
            .post(&target)
            .json(&inventory)
            .send()
            .await
            .map_err(MappingClientError::communication)?
            .error_for_status()
            .map_err(MappingClientError::communication)?
            .json::<SendInventoryResponse>()
            .await
            .map_err(MappingClientError::deserialize)
    }

    /// Gets the mapping from the mapping service
    /// Returns the values that are configured to exist for the current internal count
    async fn get_mapping(
        &self,
        _request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingClientError> {
        let target = format!("{}/mapping", self.base_url);

        execute_with_retry(
            self.max_retries,
            Duration::from_millis(self.retry_interval_ms),
            || self.client.get(&target).send(),
            Some(String::from(
                "Getting mapping info from the mapping service",
            )),
        )
        .await
        .map_err(MappingClientError::communication)?
        .error_for_status()
        .map_err(MappingClientError::communication)?
        .json::<GetMappingResponse>()
        .await
        .map_err(MappingClientError::deserialize)
    }
}
