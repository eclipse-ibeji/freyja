// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

use crate::{
    config::Config, http_mock_provider_proxy::HttpMockProviderProxy, GET_OPERATION, HTTP_PROTOCOL,
    SUBSCRIBE_OPERATION,
};
use freyja_build_common::config_file_stem;
use freyja_common::{config_utils, out_dir, signal_store::SignalStore};
use freyja_common::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory},
};

/// Factory for creating HttpMockProviderProxies
pub struct HttpMockProviderProxyFactory {
    /// The current port to use for a new proxy
    current_port: AtomicU16,
}

impl ProviderProxyFactory for HttpMockProviderProxyFactory {
    /// Create a new `GRPCProviderProxyFactory`
    fn create_new() -> Result<Self, ProviderProxyError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            ProviderProxyError::io,
            ProviderProxyError::deserialize,
        )?;

        Ok(Self {
            current_port: AtomicU16::new(config.starting_port),
        })
    }

    /// Check to see whether this factory can create a proxy for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[HTTP_PROTOCOL], &[GET_OPERATION, SUBSCRIBE_OPERATION])
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
        let mut proxy = HttpMockProviderProxy::create_new(provider_uri, signals)?;
        proxy.set_callback_server_port(self.current_port.fetch_add(1, Ordering::SeqCst));
        Ok(Arc::new(proxy))
    }
}
