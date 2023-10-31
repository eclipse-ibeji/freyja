// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Represents an entity
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    // The entity's name
    pub name: Option<String>,

    /// The entity's id
    pub id: String,

    /// The entity's description
    pub description: Option<String>,

    /// The list of supported endpoints
    pub endpoints: Vec<EntityEndpoint>,
}

/// Represents an entity's endpoint for communication
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityEndpoint {
    /// The protocol to use to contact this entity
    pub protocol: String,

    /// The operations that this entity supports
    pub operations: Vec<String>,

    /// The provider's uri
    pub uri: String,
}
