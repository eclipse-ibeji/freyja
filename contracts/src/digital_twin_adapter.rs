// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::entity::Entity;

/// Provides digital twin data
#[async_trait]
pub trait DigitalTwinAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    fn create_new() -> Result<Self, DigitalTwinAdapterError>
    where
        Self: Sized;

    /// Gets entity access information
    ///
    /// # Arguments
    /// - `request`: the request for finding an entity's access information
    async fn find_by_id(
        &self,
        request: GetDigitalTwinProviderRequest,
    ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError>;
}

/// A request for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDigitalTwinProviderRequest {
    /// The entity's id to inquire about
    pub entity_id: String,
}

/// The response for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDigitalTwinProviderResponse {
    /// Entity information
    pub entity: Entity,
}

/// A request for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueRequest {
    /// The entity's ID
    pub entity_id: String,

    /// The callback uri for a provider to send data back
    pub callback_uri: String,
}

/// A response for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueResponse {
    // The id of the entity
    pub entity_id: String,

    /// The value of the entity
    pub value: String,
}

proc_macros::error! {
    DigitalTwinAdapterError {
        EntityNotFound,
        Io,
        Serialize,
        Deserialize,
        Communication,
        ParseError,
        Unknown
    }
}
