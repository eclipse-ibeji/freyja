// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::mapping_adapter::*;
use freyja_common::{config_utils, out_dir, retry_utils::execute_with_retry};

/// Mocks a mapping provider in memory
pub struct MockMappingServiceAdapter {
    /// The base URL for requests
    base_url: String,

    /// An internal HTTP client
    client: Client,

    /// Max retries for connecting to Mock Mapping Service
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,
}

impl MockMappingServiceAdapter {
    /// Creates a new instance of a MockMappingServiceAdapter using a config file.
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
impl MappingAdapter for MockMappingServiceAdapter {
    /// Creates a new instance of a MockMappingServiceAdapter with default settings
    fn create_new() -> Result<Self, MappingAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            MappingAdapterError::io,
            MappingAdapterError::deserialize,
        )?;

        Ok(Self::from_config(config))
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
    async fn check_for_work(
        &self,
        _request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingAdapterError> {
        let target = format!("{}/work", self.base_url);

        execute_with_retry(
            self.max_retries,
            Duration::from_millis(self.retry_interval_ms),
            || self.client.get(&target).send(),
            Some(String::from("Checking for work from the mapping service")),
        )
        .await
        .map_err(MappingAdapterError::communication)?
        .error_for_status()
        .map_err(MappingAdapterError::communication)?
        .json::<CheckForWorkResponse>()
        .await
        .map_err(MappingAdapterError::deserialize)
    }

    /// Sends the provider inventory to the mapping service
    ///
    /// # Arguments
    ///
    /// - `inventory`: the providers to send
    async fn send_inventory(
        &self,
        inventory: SendInventoryRequest,
    ) -> Result<SendInventoryResponse, MappingAdapterError> {
        let target = format!("{}/inventory", self.base_url);
        self.client
            .post(&target)
            .json(&inventory)
            .send()
            .await
            .map_err(MappingAdapterError::communication)?
            .error_for_status()
            .map_err(MappingAdapterError::communication)?
            .json::<SendInventoryResponse>()
            .await
            .map_err(MappingAdapterError::deserialize)
    }

    /// Gets the mapping from the mapping service
    /// Returns the values that are configured to exist for the current internal count
    async fn get_mapping(
        &self,
        _request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingAdapterError> {
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
        .map_err(MappingAdapterError::communication)?
        .error_for_status()
        .map_err(MappingAdapterError::communication)?
        .json::<GetMappingResponse>()
        .await
        .map_err(MappingAdapterError::deserialize)
    }
}
