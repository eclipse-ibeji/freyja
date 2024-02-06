// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Configuration for the Ibeji Adapter.
/// Supports two different schemas based on the service discovery method.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "service_discovery_method")]
pub enum Config {
    /// Use a URI from the config for the In-Vehicle Digital Twin Service
    FromConfig {
        /// The URI for the In-Vehicle Digital Twin Service
        uri: String,

        /// The maximum number of retries for communication attempts
        max_retries: u32,

        /// The duration between retries in milliseconds
        retry_interval_ms: u64,
    },

    /// Use Chariott's Service Discovery system to discover the In-Vehicle Digital Twin Service
    ChariottServiceDiscovery {
        /// The URI for the Chariott Discovery Service
        uri: String,

        /// The maximum number of retries for communication attempts
        max_retries: u32,

        /// The duration between retries in milliseconds
        retry_interval_ms: u64,

        /// The request to send to Chariott
        discover_request: ChariottDiscoverRequest,
    },
}

/// A Chariott Service Discovery request
#[derive(Clone, Serialize, Deserialize)]
pub struct ChariottDiscoverRequest {
    /// The service namespace
    pub namespace: String,

    /// The service name
    pub name: String,

    /// The service version
    pub version: String,
}
