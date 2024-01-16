// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The Managed Subscribe Data Adapter config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The type of frequency constraint to use. Defaults to `"frequency_ms"`.
    pub frequency_constraint_type: String,

    /// The frequency at which the data is transferred. Defaults to `3000` ms.
    pub frequency_constraint_value: String,
}
