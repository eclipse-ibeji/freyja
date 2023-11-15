// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The Managed Subscribe provider proxy config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub frequency_constraint_type: String,
    pub frequency_constraint_value: String,
}
