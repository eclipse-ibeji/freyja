// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Configuration for the File Service Discovery Adapter
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub uri: String,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
}