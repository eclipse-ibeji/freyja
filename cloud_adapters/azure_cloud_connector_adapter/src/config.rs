// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE_NAME: &str = "config.json";

/// A config entry for the Azure Cloud Connector Adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    /// The url for the cloud connector server
    pub cloud_connector_url: String,
}
