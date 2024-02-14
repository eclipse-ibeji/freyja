// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Configuration for the Ibeji Adapter.
/// Supports two different schemas based on the service discovery method.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The URI for the In-Vehicle Digital Twin Service
    pub service_discovery_id: String,

    /// The maximum number of retries for communication attempts
    pub max_retries: u32,

    /// The duration between retries in milliseconds
    pub retry_interval_ms: u64,
}
