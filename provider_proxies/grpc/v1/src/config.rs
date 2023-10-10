// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The GRPC provider proxy config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The set of config values
    pub consumer_address: String,
}
