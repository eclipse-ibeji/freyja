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

impl Entity {
    /// Checks to see if this entity supports the requested protocol(s) and operation(s).
    /// Returns the first endpoint found that contains one of the requested protocols and operations,
    /// or `None` if no such endpoint could be found.
    ///
    /// # Arguments
    /// - `protocols`: the list of protocols to check
    /// - `operations`: the list of operations to check
    pub fn is_supported(&self, protocols: &[&str], operations: &[&str]) -> Option<EntityEndpoint> {
        for endpoint in self.endpoints.iter() {
            if protocols.contains(&endpoint.protocol.as_str()) {
                for operation in endpoint.operations.iter() {
                    if operations.contains(&operation.as_str()) {
                        return Some(endpoint.clone());
                    }
                }
            }
        }

        None
    }
}
