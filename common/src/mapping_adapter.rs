// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::digital_twin_map_entry::DigitalTwinMapEntry;

/// Interface for communicating with a mapping service
#[async_trait]
pub trait MappingAdapter {
    /// Creates a new instance of a MappingAdapter with default settings
    fn create_new() -> Result<Self, MappingAdapterError>
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
    ) -> Result<CheckForWorkResponse, MappingAdapterError>;

    /// Gets the mapping from the mapping service
    ///
    /// # Arguments
    ///
    /// - `request`: the request to send
    async fn get_mapping(
        &self,
        request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingAdapterError>;
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
    MappingAdapterError {
        Io,
        Serialize,
        Deserialize,
        Communication,
        Unknown
    }
}
