// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the http mock provider proxy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    /// The callback address for receiving signals from the mock digital twin
    pub proxy_callback_address: String,
}
