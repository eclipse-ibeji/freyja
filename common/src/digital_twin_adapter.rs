// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{entity::Entity, service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector};

/// Provides digital twin data
#[async_trait]
pub trait DigitalTwinAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    fn create_new(
        selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, DigitalTwinAdapterError>
    where
        Self: Sized;

    /// Gets entity access information
    ///
    /// # Arguments
    /// - `request`: the request for finding an entity's access information
    async fn find_by_id(
        &self,
        request: FindByIdRequest,
    ) -> Result<FindByIdResponse, DigitalTwinAdapterError>;
}

/// A request for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct FindByIdRequest {
    /// The entity's id to inquire about
    pub entity_id: String,
}

/// The response for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct FindByIdResponse {
    /// Entity information
    pub entity: Entity,
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
