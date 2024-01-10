// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use freyja_common::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory},
    signal_store::SignalStore,
};

use crate::{
    managed_subscribe_provider_proxy::ManagedSubscribeProviderProxy, GRPC_PROTOCOL,
    MANAGED_SUBSCRIBE_OPERATION,
};

/// Factory for creating ManagedSubscribeProviderProxies
pub struct ManagedSubscribeProviderProxyFactory {}

impl ProviderProxyFactory for ManagedSubscribeProviderProxyFactory {
    /// Create a new `ManagedSubscribeProviderProxyFactory`
    fn create_new() -> Result<Self, ProviderProxyError> {
        Ok(Self {})
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
    /// - `provider_uri`: the provider URI to associate with this proxy
    /// - `signals`: the shared signal store
    fn create_proxy(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError> {
        let proxy = ManagedSubscribeProviderProxy::create_new(provider_uri, signals)?;
        Ok(Arc::new(proxy))
    }
}
