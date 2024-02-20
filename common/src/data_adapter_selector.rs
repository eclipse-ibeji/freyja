// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use crate::{data_adapter::DataAdapterFactory, entity::Entity};

/// Manages a collection of data adapters and provides access to them.
/// Conceptually similar to a gateway for the adapters.
#[async_trait]
pub trait DataAdapterSelector {
    /// Registers a `DataAdapterFactory` with this selector.
    ///
    /// # Arguments
    /// - `factory`: the factory to register
    fn register(
        &mut self,
        factory: Box<dyn DataAdapterFactory + Send + Sync>,
    ) -> Result<(), DataAdapterSelectorError>;

    /// Updates an existing data adapter to include an entity if possible,
    /// otherwise creates a new data adapter to handle that entity.
    ///
    /// # Arguments
    /// - `entity`: the entity that the adapter should handle
    async fn create_or_update_adapter(
        &self,
        entity: &Entity,
    ) -> Result<(), DataAdapterSelectorError>;

    /// Requests that the value of an entity be published as soon as possible
    ///
    /// # Arguments
    /// - `entity_id`: the entity to request
    async fn request_entity_value(&self, entity_id: &str) -> Result<(), DataAdapterSelectorError>;
}

proc_macros::error! {
    DataAdapterSelectorError {
        DataAdapterError,
        EntityNotFound,
        ProtocolNotSupported,
        OperationNotSupported,
        Io,
        Serialize,
        Deserialize,
        Communication,
        Unknown
    }
}
