// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use service_discovery_proto::service_registry::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use tonic::{transport::Channel, Code, Request};

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

    fn get_adapter_name(&self) -> String {
        String::from("ChariottServiceDiscoveryAdapter")
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

        let result = execute_with_retry(
            self.config.max_retries,
            Duration::from_millis(self.config.retry_interval_ms),
            || async {
                match self
                    .client
                    .clone()
                    .discover(Request::new(request.clone()))
                    .await
                {
                    Ok(response) => {
                        let uri = response
                            .into_inner()
                            .service
                            .ok_or_else(|| {
                                ServiceDiscoveryAdapterError::communication(format!(
                                    "Cannot discover uri for service {id}"
                                ))
                            })?
                            .uri;
                        Ok(Ok(uri))
                    },
                    // This branch returns Ok(Err(_)) to indicate to the execute_with_retry wrapper that processing should stop
                    Err(status) if status.code() == Code::NotFound => Ok(Err(ServiceDiscoveryAdapterError::not_found(status))),
                    // This branch returns Err(_) to indicate to the execute_with_retry wrapper that the request should be retried
                    Err(e) => Err(ServiceDiscoveryAdapterError::communication(e)),
                }
            },
            Some("Retrieving service uri".into()),
        )
        .await;

        // This is implemented with the `flatten` method in nightly rust toolchains, which should be used here once stable
        // See https://doc.rust-lang.org/std/result/enum.Result.html#method.flatten
        match result {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e),
        }
    }
}
