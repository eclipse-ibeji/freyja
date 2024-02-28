// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use log::debug;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    mapping_adapter::{
        CheckForWorkRequest, CheckForWorkResponse, GetMappingRequest, GetMappingResponse,
        MappingAdapter, MappingAdapterError,
    },
    out_dir,
    retry_utils::execute_with_retry,
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};
use mapping_service_proto::v1::{
    mapping_service_client::MappingServiceClient, CheckForWorkRequest as ProtoCheckForWorkRequest,
    GetMappingRequest as ProtoGetMappingRequest,
};

use crate::config::Config;

/// A "standard" mapping adapter which communicates over gRPC
pub struct GRPCMappingAdapter {
    // Adapter config
    config: Config,

    // The gRPC client
    client: MappingServiceClient<Channel>,
}

#[async_trait]
impl MappingAdapter for GRPCMappingAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    ///
    /// # Arguments
    /// - `selector`: the service discovery adapter selector to use
    fn create_new(
        selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, MappingAdapterError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            MappingAdapterError::io,
            MappingAdapterError::deserialize,
        )?;

        let mapping_service_uri = futures::executor::block_on(async {
            let selector = selector.lock().await;
            selector.get_service_uri(&config.service_discovery_id).await
        })
        .map_err(MappingAdapterError::communication)?;

        let client = futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || MappingServiceClient::connect(mapping_service_uri.clone()),
                Some("Mapping adapter initial connection".into()),
            )
            .await
            .map_err(MappingAdapterError::communication)
        })?;

        Ok(Self { config, client })
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated.
    async fn check_for_work(
        &self,
        request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingAdapterError> {
        debug!("Received check for work request");

        let request: ProtoCheckForWorkRequest = request.into();

        let response = execute_with_retry(
            self.config.max_retries,
            Duration::from_millis(self.config.retry_interval_ms),
            || async {
                let request = tonic::Request::new(request.clone());
                self.client
                    .clone()
                    .check_for_work(request)
                    .await
                    .map_err(MappingAdapterError::communication)
            },
            Some(String::from("Mapping adapter check for work request")),
        )
        .await
        .map_err(MappingAdapterError::communication)?
        .into_inner();

        debug!("Check for work response: {response:?}");

        Ok(response.into())
    }

    /// Gets the mapping from the mapping service.
    async fn get_mapping(
        &self,
        request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingAdapterError> {
        debug!("Received get mapping request");

        let request: ProtoGetMappingRequest = request.into();

        let response = execute_with_retry(
            self.config.max_retries,
            Duration::from_millis(self.config.retry_interval_ms),
            || async {
                let request = tonic::Request::new(request.clone());
                self.client
                    .clone()
                    .get_mapping(request)
                    .await
                    .map_err(MappingAdapterError::communication)
            },
            Some(String::from("Mapping adapter get mapping request")),
        )
        .await
        .map_err(MappingAdapterError::communication)?
        .into_inner();

        debug!("Get mapping response: {response:?}");

        Ok(response.into())
    }
}

#[cfg(test)]
mod grpc_mapping_adapter_tests {
    use super::*;

    /// The tests below uses Unix sockets to create a channel between a gRPC client and a gRPC server.
    /// Unix sockets are more ideal than using TCP/IP sockets since Rust tests will run in parallel
    /// so you would need to set an arbitrary port per test for TCP/IP sockets.
    #[cfg(unix)]
    mod unix_tests {
        use super::*;

        use std::path::PathBuf;

        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::{
            transport::{Channel, Endpoint, Server, Uri},
            Response,
        };
        use tower::service_fn;

        use freyja_test_common::{fixtures::GRPCTestFixture, mocks::MockMappingService};
        use mapping_service_proto::v1::{
            mapping_service_server::MappingServiceServer,
            GetMappingResponse as ProtoGetMappingResponse,
        };

        async fn create_test_grpc_client(socket_path: PathBuf) -> MappingServiceClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let socket_path = socket_path.clone();
                    async move { UnixStream::connect(socket_path).await }
                }))
                .await
                .unwrap();

            MappingServiceClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mut mock_mapping_service = MockMappingService::new();
            mock_mapping_service
                .expect_get_mapping()
                .returning(|_| Ok(Response::new(ProtoGetMappingResponse::default())));

            Server::builder()
                .add_service(MappingServiceServer::new(mock_mapping_service))
                .serve_with_incoming(uds_stream)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn send_request_to_provider() {
            let fixture = GRPCTestFixture::new();

            // Create the Unix Socket
            let uds = UnixListener::bind(&fixture.socket_path).unwrap();
            let uds_stream = UnixListenerStream::new(uds);

            let request_future = async {
                let mut client = create_test_grpc_client(fixture.socket_path.clone()).await;

                let request = ProtoGetMappingRequest::default();

                let request = tonic::Request::new(request);
                assert!(client.get_mapping(request).await.is_ok())
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }
        }
    }
}
