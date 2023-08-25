// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use reqwest::Client;

use freyja_contracts::mapping_client::*;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8888"; // Devskim: ignore DS137138

/// Mocks a mapping provider in memory
pub struct MockMappingServiceClient<'a> {
    /// The base URL for requests
    base_url: &'a str,
    /// An internal HTTP client
    client: Client,
}

impl<'a> MockMappingServiceClient<'a> {
    /// Creates a new instance of a CloudAdapter with default settings
    pub fn with_url(base_url: &'a str) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MappingClient for MockMappingServiceClient<'static> {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Box<dyn MappingClient>, MappingClientError> {
        Ok(Box::new(Self::with_url(DEFAULT_ENDPOINT)))
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
    async fn check_for_work(
        &self,
        _request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingClientError> {
        let target = format!("{}/work", self.base_url);
        self.client
            .get(&target)
            .send()
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
        self.client
            .get(&target)
            .send()
            .await
            .map_err(MappingClientError::communication)?
            .error_for_status()
            .map_err(MappingClientError::communication)?
            .json::<GetMappingResponse>()
            .await
            .map_err(MappingClientError::deserialize)
    }
}
