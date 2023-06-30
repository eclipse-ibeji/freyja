// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// A configuration item for the in-memory mock cloud adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    /// The cloud service name
    pub cloud_service_name: String,
    /// The connection string to the cloud service
    pub host_connection_string: String,
}
