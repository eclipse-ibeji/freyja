// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE: &str = "config.json";

/// Configuration settings for Ibeji Adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Ibeji's In-vehicle Digital Twin Service uri
    /// If this uri is not null, then Chariott's Service Discovery uri is ignored
    pub invehicle_digital_twin_service_uri: Option<String>,

    /// Chariott's Service Discovery uri
    pub chariott_service_discovery_uri: Option<String>,
}