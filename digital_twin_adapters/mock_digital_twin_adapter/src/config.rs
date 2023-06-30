// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "config.json";

/// Settings for http provider proxy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    /// the base uri for the digital twin server
    pub base_uri_for_digital_twin_server: String,
}
