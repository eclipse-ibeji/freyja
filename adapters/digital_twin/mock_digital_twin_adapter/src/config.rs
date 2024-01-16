// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the mock digital twin adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// the base uri for the digital twin service
    pub digital_twin_service_uri: String,
}
