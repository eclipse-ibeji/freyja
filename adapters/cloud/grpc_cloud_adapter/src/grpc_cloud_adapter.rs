// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{str::FromStr, sync::Arc};
use std::time::Duration;

use async_trait::async_trait;
use freyja_common::service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector;
use log::debug;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use cloud_connector_proto::{
    prost_types::Timestamp,
    v1::{cloud_connector_client::CloudConnectorClient, UpdateDigitalTwinRequestBuilder},
};
use freyja_build_common::config_file_stem;
use freyja_common::{
    cloud_adapter::{CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse},
    config_utils, out_dir,
    retry_utils::execute_with_retry,
};

use crate::config::Config;

/// A "standard" cloud adapter which communicates over gRPC
pub struct GRPCCloudAdapter {
    // Adapter config
    config: Config,

    // The gRPC client
    client: CloudConnectorClient<Channel>,
}

#[async_trait]
impl CloudAdapter for GRPCCloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    ///
    /// # Arguments
    /// - `selector`: the service discovery adapter selector to use
    fn create_new(
        selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, CloudAdapterError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            CloudAdapterError::io,
            CloudAdapterError::deserialize,
        )?;

        let cloud_connector_uri = futures::executor::block_on(async {
            let selector = selector.lock().await;
            selector.get_service_uri(&config.service_discovery_id).await
        })
        .map_err(CloudAdapterError::communication)?;

        let client = futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || CloudConnectorClient::connect(cloud_connector_uri.clone()),
                Some("Cloud adapter initial connection".into()),
            )
            .await
            .map_err(CloudAdapterError::communication)
        })?;

        Ok(Self { config, client })
    }

    /// Sends the signal to the cloud
    ///
    /// # Arguments
    ///
    /// - `cloud_message`: represents a message to send to the cloud canonical model
    async fn send_to_cloud(
        &self,
        cloud_message: CloudMessageRequest,
    ) -> Result<CloudMessageResponse, CloudAdapterError> {
        debug!("Received a request to send to the cloud");

        let timestamp = Timestamp::from_str(cloud_message.signal_timestamp.as_str())
            .map_err(CloudAdapterError::deserialize)?;

        let request = UpdateDigitalTwinRequestBuilder::new()
            .string_value(cloud_message.signal_value)
            .timestamp(timestamp)
            .metadata(cloud_message.metadata)
            .build();

        let response = execute_with_retry(
            self.config.max_retries,
            Duration::from_millis(self.config.retry_interval_ms),
            || async {
                let request = tonic::Request::new(request.clone());
                self.client
                    .clone()
                    .update_digital_twin(request)
                    .await
                    .map_err(CloudAdapterError::communication)
            },
            Some("Cloud adapter request".into()),
        )
        .await
        .map_err(CloudAdapterError::communication)?;

        debug!("Cloud adapter response: {response:?}");

        Ok(CloudMessageResponse {})
    }
}

#[cfg(test)]
mod grpc_cloud_adapter_tests {
    use super::*;

    /// The tests below uses Unix sockets to create a channel between a gRPC client and a gRPC server.
    /// Unix sockets are more ideal than using TCP/IP sockets since Rust tests will run in parallel
    /// so you would need to set an arbitrary port per test for TCP/IP sockets.
    #[cfg(unix)]
    mod unix_tests {
        use super::*;

        use std::{
            io::{stderr, Write},
            path::PathBuf,
        };

        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::{
            transport::{Channel, Endpoint, Server, Uri},
            Request, Response, Status,
        };
        use tower::service_fn;
        use uuid::Uuid;

        use cloud_connector_proto::v1::{
            cloud_connector_server::{CloudConnector, CloudConnectorServer},
            UpdateDigitalTwinRequest, UpdateDigitalTwinResponse,
        };

        pub struct TestFixture {
            pub socket_path: PathBuf,
        }

        impl TestFixture {
            fn new() -> Self {
                Self {
                    socket_path: std::env::temp_dir()
                        .as_path()
                        .join(Uuid::new_v4().as_hyphenated().to_string()),
                }
            }
        }

        impl Drop for TestFixture {
            fn drop(&mut self) {
                match std::fs::remove_file(&self.socket_path) {
                    Ok(_) => {}
                    Err(e) => {
                        write!(stderr(), "Error cleaning up `TestFixture`: {e:?}")
                            .expect("Error writing to stderr");
                    }
                }
            }
        }

        pub struct MockCloudConnector {}

        #[tonic::async_trait]
        impl CloudConnector for MockCloudConnector {
            /// Updates a digital twin instance
            ///
            /// # Arguments
            /// - `request`: the request to send
            async fn update_digital_twin(
                &self,
                _request: Request<UpdateDigitalTwinRequest>,
            ) -> Result<Response<UpdateDigitalTwinResponse>, Status> {
                let response = UpdateDigitalTwinResponse {};
                Ok(Response::new(response))
            }
        }

        async fn create_test_grpc_client(socket_path: PathBuf) -> CloudConnectorClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let socket_path = socket_path.clone();
                    async move { UnixStream::connect(socket_path).await }
                }))
                .await
                .unwrap();

            CloudConnectorClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_azure_connector = MockCloudConnector {};
            Server::builder()
                .add_service(CloudConnectorServer::new(mock_azure_connector))
                .serve_with_incoming(uds_stream)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn send_request_to_provider() {
            let fixture = TestFixture::new();

            // Create the Unix Socket
            let uds = UnixListener::bind(&fixture.socket_path).unwrap();
            let uds_stream = UnixListenerStream::new(uds);

            let request_future = async {
                let mut client = create_test_grpc_client(fixture.socket_path.clone()).await;

                let request = UpdateDigitalTwinRequestBuilder::new()
                    .string_value("foo".into())
                    .timestamp_now()
                    .add_metadata(
                        "model_id".into(),
                        "dtmi:sdv:Cloud:Vehicle:Cabin:HVAC:AmbientAirTemperature;1".into(),
                    )
                    .add_metadata("instance_id".into(), "hvac".into())
                    .add_metadata(
                        "instance_property_path".into(),
                        "/AmbientAirTemperature".into(),
                    )
                    .build();

                let request = tonic::Request::new(request);
                assert!(client.update_digital_twin(request).await.is_ok())
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }
        }
    }
}
