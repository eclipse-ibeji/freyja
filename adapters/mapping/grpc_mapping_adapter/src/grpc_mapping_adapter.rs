// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use log::debug;
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
    fn create_new() -> Result<Self, MappingAdapterError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            MappingAdapterError::io,
            MappingAdapterError::deserialize,
        )?;

        let client = futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || MappingServiceClient::connect(config.target_uri.clone()),
                Some("Mapping adapter initial connection".into()),
            )
            .await
            .map_err(MappingAdapterError::communication)
        })?;

        Ok(Self { config, client })
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
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

    /// Gets the mapping from the mapping service
    /// Returns the values that are configured to exist for the current internal count
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

        use std::sync::Arc;

        use mapping_service_proto::v1::{
            mapping_service_server::{MappingService, MappingServiceServer},
            CheckForWorkResponse as ProtoCheckForWorkResponse,
            GetMappingResponse as ProtoGetMappingResponse,
        };
        use tempfile::TempPath;
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::{
            transport::{Channel, Endpoint, Server, Uri},
            Request, Response, Status,
        };
        use tower::service_fn;

        pub struct MockMappingService {}

        #[tonic::async_trait]
        impl MappingService for MockMappingService {
            async fn check_for_work(
                &self,
                _request: Request<ProtoCheckForWorkRequest>,
            ) -> Result<Response<ProtoCheckForWorkResponse>, Status> {
                let response = ProtoCheckForWorkResponse::default();
                Ok(Response::new(response))
            }

            async fn get_mapping(
                &self,
                _request: Request<ProtoGetMappingRequest>,
            ) -> Result<Response<ProtoGetMappingResponse>, Status> {
                let response = ProtoGetMappingResponse::default();
                Ok(Response::new(response))
            }
        }

        async fn create_test_grpc_client(
            bind_path: Arc<TempPath>,
        ) -> MappingServiceClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let bind_path = bind_path.clone();
                    async move { UnixStream::connect(bind_path.as_ref()).await }
                }))
                .await
                .unwrap();

            MappingServiceClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_azure_connector = MockMappingService {};
            Server::builder()
                .add_service(MappingServiceServer::new(mock_azure_connector))
                .serve_with_incoming(uds_stream)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn send_request_to_provider() {
            // Create the Unix Socket
            let bind_path = Arc::new(tempfile::NamedTempFile::new().unwrap().into_temp_path());
            let uds = match UnixListener::bind(bind_path.as_ref()) {
                Ok(unix_listener) => unix_listener,
                Err(_) => {
                    std::fs::remove_file(bind_path.as_ref()).unwrap();
                    UnixListener::bind(bind_path.as_ref()).unwrap()
                }
            };
            let uds_stream = UnixListenerStream::new(uds);

            let request_future = async {
                let mut client = create_test_grpc_client(bind_path.clone()).await;

                let request = ProtoGetMappingRequest::default();

                let request = tonic::Request::new(request);
                assert!(client.get_mapping(request).await.is_ok())
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }

            std::fs::remove_file(bind_path.as_ref()).unwrap();
        }
    }
}
