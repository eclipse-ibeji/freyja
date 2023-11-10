// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The GRPC provider proxy config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The keep alive interval in seconds
    pub keep_alive_interval_s: u64,
}