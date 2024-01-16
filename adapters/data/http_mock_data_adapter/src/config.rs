// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the http mock data adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    /// The callback address for receiving signals from the mock digital twin
    pub callback_address: String,

    /// The starting port number
    pub starting_port: u16,
}
