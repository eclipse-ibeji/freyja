// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;

use crate::entity::{EntityEndpoint, Entity};

/// Represents a signal value
pub struct SignalValue {
    /// The entity's id
    pub entity_id: String,

    /// The entity's value
    pub value: String,
}

#[async_trait]
pub trait ProviderProxy: Debug {
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

    /// Runs a provider proxy.
    async fn run(&self) -> Result<(), ProviderProxyError>;

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), ProviderProxyError>;

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<(), ProviderProxyError>;
}

pub trait ProviderProxyFactory {
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint>;

    fn create_proxy(&self, provider_uri: &str, signal_values_queue: Arc<SegQueue<SignalValue>>) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError>;
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
