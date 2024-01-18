// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use strum_macros::Display;

use crate::{
    entity::{Entity, EntityEndpoint},
    signal_store::SignalStore,
};

#[derive(Clone, Debug, Display, Eq, PartialEq)]
/// Return options for when a data adapter attempts to register an entity
pub enum EntityRegistration {
    /// The Entity has been successfully registered by the data adapter
    Registered,
    /// The data adapter has requested a loopback with new information for the selector
    Loopback(Entity),
}

/// Interfacess with a data source, such as a digital twin provider
#[async_trait]
pub trait DataAdapter {
    /// Creates a data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_store`: the shared signal store
    fn create_new(provider_uri: &str, signals: Arc<SignalStore>) -> Result<Self, DataAdapterError>
    where
        Self: Sized;

    /// Starts a data adapter.
    /// This should not block once intialization is complete, so anything that needs to run indefinitely
    /// (such as a server or a listener) should spawn its own task.
    async fn start(&self) -> Result<(), DataAdapterError>;

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), DataAdapterError>;

    /// Registers an entity id to a local cache inside a data adapter to keep track of which entities a data adapter contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    /// Some adapters may return a 'Loopback' with new entity information for the selector to use to select a different adapter.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<EntityRegistration, DataAdapterError>;
}

/// Factory for creating DataAdapters
pub trait DataAdapterFactory {
    /// Create a new factory
    fn create_new() -> Result<Self, DataAdapterError>
    where
        Self: Sized;

    /// Check to see whether this factory can create a data adapter for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint>;

    /// Create a new data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider URI to associate with this data adapter
    /// - `signals`: the shared signal store
    fn create_adapter(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn DataAdapter + Send + Sync>, DataAdapterError>;
}

proc_macros::error! {
    DataAdapterError {
        Io,
        Parse,
        Serialize,
        Deserialize,
        Communication,
        EntityNotFound,
        OperationNotSupported,
        Unknown
    }
}
