// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE_RELATIVE_TO_OUTPUT_DIR: &str = "../../../ibeji_adapter_config.json";

/// Configuration setting variants for selecting the service
/// that the Ibeji Adapter should communicate with to interact with Ibeji
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "service_type")]
pub enum Settings {
    /// In-Vehicle Digital Twin Service
    InVehicleDigitalTwinService {
        uri: String,
        max_retries: u32,
        retry_interval_ms: u64,
    },

    /// Chariott's Service Discovery to discover Ibeji
    ChariottDiscoveryService {
        uri: String,
        max_retries: u32,
        retry_interval_ms: u64,
        metadata: IbejiDiscoveryMetadata,
    },
}

/// Configuration metadata for discovering Ibeji using Chariott
#[derive(Clone, Serialize, Deserialize)]
pub struct IbejiDiscoveryMetadata {
    pub namespace: String,
    pub name: String,
    pub version: String,
}
