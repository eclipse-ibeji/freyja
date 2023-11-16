// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

use crossbeam::queue::SegQueue;

use crate::{
    config::Config, http_mock_provider_proxy::HttpMockProviderProxy, GET_OPERATION, HTTP_PROTOCOL,
    SUBSCRIBE_OPERATION,
};
use freyja_build_common::config_file_stem;
use freyja_common::{config_utils, out_dir};
use freyja_contracts::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory, SignalValue},
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
    /// - `signal_values_queue`: The queue into which new signal values wil lbe published
    fn create_proxy(
        &self,
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError> {
        let mut proxy = HttpMockProviderProxy::create_new(provider_uri, signal_values_queue)?;
        proxy.set_callback_server_port(self.current_port.fetch_add(1, Ordering::SeqCst));
        Ok(Arc::new(proxy))
    }
}
