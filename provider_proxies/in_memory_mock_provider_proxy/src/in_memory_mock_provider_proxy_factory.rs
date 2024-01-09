// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crossbeam::queue::SegQueue;
use freyja_common::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory, SignalValue},
};

use crate::{
    in_memory_mock_provider_proxy::InMemoryMockProviderProxy, GET_OPERATION, IN_MEMORY_PROTOCOL,
    SUBSCRIBE_OPERATION,
};

/// Factory for creating InMemoryMockProviderProxies
pub struct InMemoryMockProviderProxyFactory {}

impl ProviderProxyFactory for InMemoryMockProviderProxyFactory {
    /// Create a new `GRPCProviderProxyFactory`
    fn create_new() -> Result<Self, ProviderProxyError> {
        Ok(Self {})
    }

    /// Check to see whether this factory can create a proxy for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[IN_MEMORY_PROTOCOL], &[GET_OPERATION, SUBSCRIBE_OPERATION])
    }

    /// Create a new proxy
    ///
    /// # Arguments
    /// - `provider_uri`: The provider URI to associate with this proxy
    /// - `signal_values_queue`: The queue into which new signal values wil lbe published
    fn create_proxy(
        &self,
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError> {
        let proxy = InMemoryMockProviderProxy::create_new(provider_uri, signal_values_queue)?;
        Ok(Arc::new(proxy))
    }
}
