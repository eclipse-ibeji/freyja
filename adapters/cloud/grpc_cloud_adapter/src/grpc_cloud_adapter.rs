// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use cloud_adapter_proto::v1::UpdateDigitalTwinRequest;
use freyja_build_common::config_file_stem;
use freyja_common::retry_utils::execute_with_retry;
use freyja_common::{config_utils, out_dir};
use log::debug;

use freyja_common::cloud_adapter::{
    CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse,
};
use cloud_adapter_proto::prost_types::{Timestamp, Value};

use cloud_adapter_proto::v1::cloud_adapter_service_client::CloudAdapterServiceClient;
use tonic::transport::Channel;

use crate::config::Config;

/// Mocks a cloud adapter in memory
pub struct GRPCCloudAdapter {
    // Adapter config
    _config: Config,

    // The gRPC client
    client: CloudAdapterServiceClient<Channel>,
}

#[async_trait]
impl CloudAdapter for GRPCCloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Self, CloudAdapterError> {
        let config: Config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            CloudAdapterError::io,
            CloudAdapterError::deserialize,
        )?;

        let client = futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || CloudAdapterServiceClient::connect(config.target_uri.clone()),
                Some(String::from("Cloud adapter connection retry")))
            .await
            .map_err(CloudAdapterError::communication)
        })?;

        Ok(Self {
            _config: config,
            client,
        })
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

        let request = tonic::Request::new(
            UpdateDigitalTwinRequest {
                value: Some(Value {
                    kind: Some(cloud_adapter_proto::prost_types::value::Kind::StringValue(cloud_message.signal_value))
                }),
                timestamp: Some(timestamp),
                metadata: cloud_message.metadata,
            }
        );

        let response = self
            .client
            .clone()
            .update_digital_twin(request)
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

        use std::collections::HashMap;
        use std::sync::Arc;

        use cloud_adapter_proto::v1::cloud_adapter_service_server::{
            CloudAdapterService,
            CloudAdapterServiceServer,
        };
        use cloud_adapter_proto::v1::UpdateDigitalTwinResponse;
        use tempfile::TempPath;
        use time::OffsetDateTime;
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::transport::{Channel, Endpoint, Server, Uri};
        use tonic::{Request, Response, Status};
        use tower::service_fn;

        pub struct MockCloudConnector {}

        #[tonic::async_trait]
        impl CloudAdapterService for MockCloudConnector {
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

        async fn create_test_grpc_client(
            bind_path: Arc<TempPath>,
        ) -> CloudAdapterServiceClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let bind_path = bind_path.clone();
                    async move { UnixStream::connect(bind_path.as_ref()).await }
                }))
                .await
                .unwrap();

            CloudAdapterServiceClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_azure_connector = MockCloudConnector {};
            Server::builder()
                .add_service(CloudAdapterServiceServer::new(mock_azure_connector))
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

                let timestamp = OffsetDateTime::now_utc();

                let request = tonic::Request::new(UpdateDigitalTwinRequest {
                    value: Some(Value {
                        kind: Some(cloud_adapter_proto::prost_types::value::Kind::StringValue("foo".to_owned()))
                    }),
                    timestamp: Some(
                        Timestamp::date_time(
                            timestamp.year().into(),
                            timestamp.month().into(),
                            timestamp.day(),
                            timestamp.hour(),
                            timestamp.minute(),
                            timestamp.second(),
                        ).unwrap()
                    ),
                    metadata: HashMap::from([
                        ("model_id".to_owned(), "dtmi:sdv:Cloud:Vehicle:Cabin:HVAC:AmbientAirTemperature;1".to_owned()),
                        ("instance_id".to_owned(), "hvac".to_owned()),
                        ("instance_property_path".to_owned(), "/AmbientAirTemperature".to_owned()),
                    ]),
                });
                assert!(client.update_digital_twin(request).await.is_ok())
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }

            std::fs::remove_file(bind_path.as_ref()).unwrap();
        }
    }
}