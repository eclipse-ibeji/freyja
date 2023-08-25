// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE: &str = "azure_cloud_connector_adapter_config.json";

/// A config entry for the Azure Cloud Connector Adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Max retries for connecting to an Azure Cloud Connector
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,

    /// The url for the cloud connector server
    pub cloud_connector_url: String,
}
