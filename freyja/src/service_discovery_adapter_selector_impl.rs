// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use freyja_common::{
    service_discovery_adapter::{
        ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError, ServiceDiscoveryAdapterErrorKind,
    },
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};

/// Selector for `ServiceDiscoveryAdapters`
pub struct ServiceDiscoveryAdapterSelectorImpl {
    adapters: Vec<Box<dyn ServiceDiscoveryAdapter + Send + Sync>>,
}

impl ServiceDiscoveryAdapterSelectorImpl {
    /// Creates a new instance of a `ServiceDiscoveryAdapterSelectorImpl`
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }
}

#[async_trait]
impl ServiceDiscoveryAdapterSelector for ServiceDiscoveryAdapterSelectorImpl {
    /// Registers a `ServiceDiscoveryAdapter` with this selector
    ///
    /// # Arguments
    /// - `adapter`: the adapter to register
    fn register(
        &mut self,
        adapter: Box<dyn ServiceDiscoveryAdapter + Send + Sync>,
    ) -> Result<(), ServiceDiscoveryAdapterError> {
        self.adapters.push(adapter);

        Ok(())
    }

    /// Gets the URI for the requested service.
    /// Adapters will be checked in registration order, and the first successful result will be returned.
    /// If no adapters can successfully retrieve the URI, a `NotFound` error will be returned.
    ///
    /// # Arguments
    /// - `id`: the service identifier
    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError> {
        for adapter in self.adapters.iter() {
            log::debug!(
                "Attempting to discover uri for service {id} from adapter {}...",
                adapter.get_adapter_name()
            );
            match adapter.get_service_uri(id).await {
                Ok(uri) => {
                    log::debug!("Discovered uri for service {id}");
                    return Ok(uri);
                }
                Err(e) => {
                    log::debug!("Failed to discover service uri: {e:?}. Trying next adapter...")
                }
            }
        }

        Err(ServiceDiscoveryAdapterErrorKind::NotFound.into())
    }
}
