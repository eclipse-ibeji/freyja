// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Represents an entity
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    /// The entity's id
    pub id: String,

    // The entity's name
    pub name: Option<String>,

    /// The provider's uri
    pub uri: String,

    /// The entity's description
    pub description: Option<String>,

    /// The operation that we will use for this entity
    pub operation: String,

    /// The protocol to use to contact this entity
    pub protocol: String,
}
