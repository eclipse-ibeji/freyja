// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crossbeam::queue::SegQueue;
use freyja_contracts::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory, SignalValue},
};

use crate::{
    managed_subscribe_provider_proxy::ManagedSubscribeProviderProxy, GRPC_PROTOCOL,
    MANAGED_SUBSCRIBE_OPERATION,
};

/// Factory for creating ManagedSubscribeProviderProxies
pub struct ManagedSubscribeProviderProxyFactory {}

impl ProviderProxyFactory for ManagedSubscribeProviderProxyFactory {
    /// Create a new `ManagedSubscribeProviderProxyFactory`
    fn new() -> Self {
        Self {}
    }

    /// Check to see whether this factory can create a proxy for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[GRPC_PROTOCOL], &[MANAGED_SUBSCRIBE_OPERATION])
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
        ManagedSubscribeProviderProxy::create_new(provider_uri, signal_values_queue)
    }
}
