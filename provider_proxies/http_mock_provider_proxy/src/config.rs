// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE: &str = "config.json";
pub(crate) const CALLBACK_FOR_VALUES_PATH: &str = "/value";

/// Settings for http provider proxy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    /// the callback server authority for receiving signals from the mock digital twin
    pub provider_callback_authority: String,
}
