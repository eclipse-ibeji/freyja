// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

#[async_trait]
pub trait ServiceDiscoveryAdapter {
    /// Creates a new instance of a ServiceDiscoveryAdapter with default settings
    fn create_new() -> Result<Self, ServiceDiscoveryAdapterError>
    where
        Self: Sized;
    
    /// Gets the name of this adapter. Used for diagnositc purposes.
    fn get_adapter_name(&self) -> String;

    /// Gets the URI for the requested service
    ///
    /// # Arguments
    /// - `id`: the service identifier
    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError>;
}

proc_macros::error! {
    ServiceDiscoveryAdapterError {
        Io,
        Serialize,
        Deserialize,
        Communication,
        NotFound,
        InvalidId,
        Unknown
    }
}
