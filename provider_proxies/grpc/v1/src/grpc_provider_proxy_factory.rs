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
    grpc_provider_proxy::GRPCProviderProxy, GET_OPERATION, GRPC_PROTOCOL, SUBSCRIBE_OPERATION,
};

/// Factory for creating GRPCProviderProxies
pub struct GRPCProviderProxyFactory {}

impl ProviderProxyFactory for GRPCProviderProxyFactory {
    /// Create a new `GRPCProviderProxyFactory`
    fn new() -> Self {
        Self {}
    }

    /// Check to see whether this factory can create a proxy for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[GRPC_PROTOCOL], &[GET_OPERATION, SUBSCRIBE_OPERATION])
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
        GRPCProviderProxy::create_new(provider_uri, signal_values_queue)
    }
}
