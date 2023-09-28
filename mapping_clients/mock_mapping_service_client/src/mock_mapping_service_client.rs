// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::config::Config;
use freyja_common::{config_utils, out_dir, retry_utils::execute_with_retry};
use freyja_contracts::mapping_client::*;

const CONFIG_FILE: &str = "mock_mapping_client_config";
const CONFIG_EXT: &str = "json";

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
    /// Creates a new instance of a MockMappingServiceClient using a config file.
    ///
    /// # Arguments
    /// - `config`: the config
    pub fn from_config(config: Config) -> Self {
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
    /// Creates a new instance of a MockMappingServiceClient with default settings
    fn create_new() -> Result<Self, MappingClientError> {
        let config = config_utils::read_from_files(
            CONFIG_FILE,
            CONFIG_EXT,
            out_dir!(),
            MappingClientError::io,
            MappingClientError::deserialize,
        )?;

        Ok(Self::from_config(config))
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
