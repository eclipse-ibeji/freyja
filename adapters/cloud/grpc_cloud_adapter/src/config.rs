// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the GRPCCloudAdapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The service discovery id of the cloud connector
    pub service_discovery_id: String,

    /// Max retries for contacting the server
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,
}
