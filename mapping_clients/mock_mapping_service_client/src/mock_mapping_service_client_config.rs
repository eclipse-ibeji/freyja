// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE: &str = "mock_mapping_service_client_config.json";

/// Configuration metadata for discovering Ibeji using Chariott
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// Max retries for connecting to
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,

    /// The url for the mock mapping service
    pub mock_mapping_service_url: String,
}
