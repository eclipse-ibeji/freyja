// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Configuration for the File Service Discovery Adapter
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// A map of service ids to uris
    pub services: HashMap<String, String>,
}
