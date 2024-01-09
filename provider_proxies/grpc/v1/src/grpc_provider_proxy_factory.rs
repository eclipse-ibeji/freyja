// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use freyja_common::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory}, signal_store::SignalStore,
};

use crate::{
    grpc_provider_proxy::GRPCProviderProxy, GET_OPERATION, GRPC_PROTOCOL, SUBSCRIBE_OPERATION,
};

/// Factory for creating GRPCProviderProxies
pub struct GRPCProviderProxyFactory {}

impl ProviderProxyFactory for GRPCProviderProxyFactory {
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
        entity.is_supported(&[GRPC_PROTOCOL], &[GET_OPERATION, SUBSCRIBE_OPERATION])
    }

    /// Create a new proxy
    ///
    /// # Arguments
    /// - `provider_uri`: The provider URI to associate with this proxy
    /// - `signals`: The shared signal store
    fn create_proxy(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError> {
        let proxy = GRPCProviderProxy::create_new(provider_uri, signals)?;
        Ok(Arc::new(proxy))
    }
}
