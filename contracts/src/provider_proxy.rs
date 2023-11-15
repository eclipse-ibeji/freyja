// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use strum_macros::Display;

use crate::entity::{Entity, EntityEndpoint};

/// Represents a signal value
pub struct SignalValue {
    /// The entity's id
    pub entity_id: String,

    /// The entity's value
    pub value: String,
}

#[derive(Clone, Debug, Display, Eq, PartialEq)]
/// Successful return types when a proxy registers an entity
pub enum EntityRegistration {
    /// The Entity has been successfully registered by the proxy
    Registered,
    /// The proxy has requested a loopback with new information for the proxy selector
    Loopback(Entity),
}

/// Consumes data from a provider and acts as a proxy for its interface
#[async_trait]
pub trait ProviderProxy {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_values_queue`: shared queue for all provider proxies to push new signal values of entities
    fn create_new(
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError>
    where
        Self: Sized;

    /// Starts a provider proxy.
    /// This should not block once intialization is complete, so anything that needs to run indefinitely
    /// (such as a server or a listener) should spawn its own task.
    async fn start(&self) -> Result<(), ProviderProxyError>;

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), ProviderProxyError>;

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    /// Some proxies may return a 'Loopback' with new entity information for the proxy selector to use to select a different proxy.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<EntityRegistration, ProviderProxyError>;
}

/// Factory for creating ProviderProxies
pub trait ProviderProxyFactory {
    /// Create a new factory
    fn new() -> Self
    where
        Self: Sized;

    /// Check to see whether this factory can create a proxy for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint>;

    /// Create a new proxy
    ///
    /// # Arguments
    /// - `provider_uri`: The provider URI to associate with this proxy
    /// - `signal_values_queue`: The queue into which new signal values will be published
    fn create_proxy(
        &self,
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError>;
}

proc_macros::error! {
    ProviderProxyError {
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
