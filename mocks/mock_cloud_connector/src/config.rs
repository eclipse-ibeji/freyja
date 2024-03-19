// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the mock cloud connector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The server authority for hosting a gRPC server
    pub server_authority: String,
}
