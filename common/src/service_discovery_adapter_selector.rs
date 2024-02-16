// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use crate::service_discovery_adapter::{ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError};

/// Selector for `ServiceDiscoveryAdapters`
#[async_trait]
pub trait ServiceDiscoveryAdapterSelector {
    /// Registers a `ServiceDiscoveryAdapter` with this selector
    /// 
    /// # Arguments
    /// - `adapter`: the adapter to register
    fn register(
        &mut self,
        adapter: Box<dyn ServiceDiscoveryAdapter + Send + Sync>,
    ) -> Result<(), ServiceDiscoveryAdapterError>;

    /// Gets the URI for the requested service.
    ///
    /// # Arguments
    /// - `id`: the service identifier
    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError>;
}
