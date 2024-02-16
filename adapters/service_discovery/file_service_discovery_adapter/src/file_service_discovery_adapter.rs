// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils, out_dir,
    service_discovery_adapter::{
        ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError, ServiceDiscoveryAdapterErrorKind,
    },
};

/// Uses a static config file for service discovery
pub struct FileServiceDiscoveryAdapter {
    /// The adapter config
    config: Config,
}

#[async_trait]
impl ServiceDiscoveryAdapter for FileServiceDiscoveryAdapter {
    /// Creates a new instance of a `ServiceDiscoveryAdapter` with default settings
    fn create_new() -> Result<Self, ServiceDiscoveryAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            ServiceDiscoveryAdapterError::io,
            ServiceDiscoveryAdapterError::deserialize,
        )?;

        Ok(Self { config })
    }

    /// Gets the name of this adapter. Used for diagnostic purposes.
    fn get_adapter_name(&self) -> String {
        String::from("FileServiceDiscoveryAdapter")
    }

    /// Gets the URI for the requested service
    ///
    /// # Arguments
    /// - `id`: the service identifier
    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError> {
        self.config
            .services
            .get(id)
            .cloned()
            .ok_or(ServiceDiscoveryAdapterErrorKind::NotFound.into())
    }
}
