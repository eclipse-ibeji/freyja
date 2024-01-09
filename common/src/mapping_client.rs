// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::digital_twin_map_entry::DigitalTwinMapEntry;

/// Client interface for communicating with a mapping service
#[async_trait]
pub trait MappingClient {
    /// Creates a new instance of a MappingClient with default settings
    fn create_new() -> Result<Self, MappingClientError>
    where
        Self: Sized;

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    ///
    /// # Arguments
    ///
    /// - `request`: the request to send
    async fn check_for_work(
        &self,
        request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingClientError>;

    /// Sends the provider inventory to the mapping service
    ///
    /// # Arguments
    ///
    /// - `inventory`: the request to send
    async fn send_inventory(
        &self,
        _inventory: SendInventoryRequest,
    ) -> Result<SendInventoryResponse, MappingClientError> {
        Ok(SendInventoryResponse {})
    }

    /// Gets the mapping from the mapping service
    ///
    /// # Arguments
    ///
    /// - `request`: the request to send
    async fn get_mapping(
        &self,
        request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingClientError>;
}

/// A request for the check for work api
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckForWorkRequest {}

/// A response for the check for work api
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckForWorkResponse {
    /// Whether or not there is work for the caller
    pub has_work: bool,
}

/// A request for sending inventory
#[derive(Debug, Serialize, Deserialize)]
pub struct SendInventoryRequest {
    pub inventory: HashSet<String>,
}

/// A response to sending inventory
#[derive(Debug, Serialize, Deserialize)]
pub struct SendInventoryResponse {}

/// A request for a mapping
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMappingRequest {}

/// A response with a mapping
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMappingResponse {
    /// The map
    pub map: HashMap<String, DigitalTwinMapEntry>,
}

proc_macros::error! {
    MappingClientError {
        Io,
        Serialize,
        Deserialize,
        Communication,
        Unknown
    }
}
