// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use service_discovery_proto::service_registry::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use tonic::{transport::Channel, Request};

use crate::config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils, out_dir,
    retry_utils::execute_with_retry,
    service_discovery_adapter::{
        ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError, ServiceDiscoveryAdapterErrorKind,
    },
};

pub struct ChariottServiceDiscoveryAdapter {
    config: Config,

    client: ServiceRegistryClient<Channel>,
}

#[async_trait]
impl ServiceDiscoveryAdapter for ChariottServiceDiscoveryAdapter {
    fn create_new() -> Result<Self, ServiceDiscoveryAdapterError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            ServiceDiscoveryAdapterError::io,
            ServiceDiscoveryAdapterError::deserialize,
        )?;

        let client = futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || async {
                    ServiceRegistryClient::connect(config.uri.clone())
                        .await
                        .map_err(ServiceDiscoveryAdapterError::communication)
                },
                Some("Connecting to Chariott Service Discovery".into()),
            )
            .await
        })?;

        Ok(Self { config, client })
    }

    async fn get_service_uri(&self, id: &String) -> Result<String, ServiceDiscoveryAdapterError> {
        let pieces = id.split('/').collect::<Vec<_>>();
        if pieces.len() != 3 {
            return Err(ServiceDiscoveryAdapterErrorKind::InvalidId.into());
        }

        let request = DiscoverRequest {
            namespace: pieces[0].into(),
            name: pieces[1].into(),
            version: pieces[2].into(),
        };

        execute_with_retry(
            self.config.max_retries,
            Duration::from_millis(self.config.retry_interval_ms),
            || async {
                let uri = self
                    .client
                    .clone()
                    .discover(Request::new(request.clone()))
                    .await
                    .map_err(ServiceDiscoveryAdapterError::communication)?
                    .into_inner()
                    .service
                    .ok_or_else(|| {
                        ServiceDiscoveryAdapterError::communication(format!(
                            "Cannot discover uri for service {id}"
                        ))
                    })?
                    .uri;

                Ok(uri)
            },
            Some("Retrieving service uri".into()),
        )
        .await
    }
}
