// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    out_dir, service_discovery_adapter::{ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError, ServiceDiscoveryAdapterErrorKind},
};

pub struct FileServiceDiscoveryAdapter {
    config: Config,
}

#[async_trait]
impl ServiceDiscoveryAdapter for FileServiceDiscoveryAdapter {
    fn create_new() -> Result<Self, ServiceDiscoveryAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            ServiceDiscoveryAdapterError::io,
            ServiceDiscoveryAdapterError::deserialize,
        )?;

        Ok(Self{config})
    }

    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError> {
        self.config.services.get(id).cloned().ok_or(ServiceDiscoveryAdapterErrorKind::NotFound.into())
    }
}
