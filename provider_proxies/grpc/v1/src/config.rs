// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The in-memory mock mapping client's config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The set of config values
    pub consumer_address: String,
}