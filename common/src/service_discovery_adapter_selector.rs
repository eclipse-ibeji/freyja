// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use crate::service_discovery_adapter::{ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError};

#[async_trait]
pub trait ServiceDiscoveryAdapterSelector {
    async fn register<T: ServiceDiscoveryAdapter + Send + Sync + 'static>(&mut self) -> Result<(), ServiceDiscoveryAdapterError>;

    /// Gets the URI for the requested service
    /// 
    /// # Arguments
    /// - `id`: the service identifier
    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError>;
}