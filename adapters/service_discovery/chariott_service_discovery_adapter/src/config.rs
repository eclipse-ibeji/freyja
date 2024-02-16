// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Configuration for the File Service Discovery Adapter
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The Chariott uri
    pub uri: String,

    /// The maximum number of retries for communication attempts
    pub max_retries: u32,

    /// The duration between retries in milliseconds
    pub retry_interval_ms: u64,
}
