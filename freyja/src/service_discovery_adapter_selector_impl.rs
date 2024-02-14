// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use freyja_common::{service_discovery_adapter::{ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError, ServiceDiscoveryAdapterErrorKind}, service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector};

pub struct ServiceDiscoveryAdapterSelectorImpl {
    adapters: Vec<Box<dyn ServiceDiscoveryAdapter + Send + Sync>>,
}

#[async_trait]
impl ServiceDiscoveryAdapterSelector for ServiceDiscoveryAdapterSelectorImpl {
    async fn register<T: ServiceDiscoveryAdapter + Send + Sync + 'static>(&mut self) -> Result<(), ServiceDiscoveryAdapterError> {
        let adapter = T::create_new()?;

        self.adapters.push(Box::new(adapter));

        Ok(())
    }

    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError> {
        for adapter in self.adapters.iter() {
            match adapter.get_service_uri(id).await {
                Ok(uri) => return Ok(uri),
                Err(_) => {}
            }
        }

        Err(ServiceDiscoveryAdapterErrorKind::NotFound.into())
    }
}