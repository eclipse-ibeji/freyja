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

pub(crate) mod chariott_ibeji_config {
    pub const DIGITAL_TWIN_SERVICE_NAME: &str = "digital_twin";
    pub const DIGITAL_TWIN_SERVICE_VERSION: &str = "1.0";
    pub const CHARIOTT_NAMESPACE_FOR_IBEJI: &str = "sdv.ibeji";
}
