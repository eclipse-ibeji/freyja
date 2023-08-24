// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path, time::Duration};

use async_trait::async_trait;
use azure_cloud_connector_proto::azure_cloud_connector::{
    azure_cloud_connector_client::AzureCloudConnectorClient, UpdateDigitalTwinRequest,
};
use log::debug;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;

use crate::azure_cloud_connector_adapter_config::{Config, CONFIG_FILE};
use common::utils::execute_with_retry;
use dts_contracts::cloud_adapter::{
    CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse,
};

const MODEL_ID_KEY: &str = "model_id";
const INSTANCE_ID_KEY: &str = "instance_id";
const INSTANCE_PROPERTY_PATH_KEY: &str = "instance_property_path";

/// The Cloud Connector Adapter for communicating with the Cloud Connector
pub struct AzureCloudConnectorAdapter {
    // A gRPC Client for communicating with the Azure Cloud Connector
    cloud_connector_client: AzureCloudConnectorClient<Channel>,
}

/// Contains info about a digital twin instance
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct CloudDigitalTwinInstance {
    /// The id of the target signal's model
    pub model_id: String,

    /// The id of the target signal's instance
    pub instance_id: String,

    /// The path of the property within the instance to target
    pub instance_property_path: String,
}

impl AzureCloudConnectorAdapter {
    /// Gets info about an instance from a cloud message
    ///
    /// # Arguments
    /// - `cloud_message`: represents a message to send to the cloud canonical model
    fn get_instance_info_from_message(
        cloud_message: &CloudMessageRequest,
    ) -> Result<CloudDigitalTwinInstance, CloudAdapterError> {
        Ok(CloudDigitalTwinInstance {
            model_id: cloud_message
                .cloud_signal
                .get(MODEL_ID_KEY)
                .ok_or_else(|| {
                    CloudAdapterError::key_not_found(format!("Cannot find key: {MODEL_ID_KEY:}"))
                })?
                .clone(),
            instance_id: cloud_message
                .cloud_signal
                .get(INSTANCE_ID_KEY)
                .ok_or_else(|| {
                    CloudAdapterError::key_not_found(format!("Cannot find key: {INSTANCE_ID_KEY:}"))
                })?
                .clone(),
            instance_property_path: cloud_message
                .cloud_signal
                .get(INSTANCE_PROPERTY_PATH_KEY)
                .ok_or_else(|| {
                    CloudAdapterError::key_not_found(format!(
                        "Cannot find key: {INSTANCE_PROPERTY_PATH_KEY:}"
                    ))
                })?
                .clone(),
        })
    }
}

#[async_trait]
impl CloudAdapter for AzureCloudConnectorAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Box<dyn CloudAdapter + Send + Sync>, CloudAdapterError> {
        let cloud_connector_client = futures::executor::block_on(async {
            let config_file =
                fs::read_to_string(Path::new(env!("OUT_DIR")).join(CONFIG_FILE)).unwrap();

            // Load the config
            let config: Config = serde_json::from_str(&config_file).unwrap();

            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || AzureCloudConnectorClient::connect(config.cloud_connector_url.clone()),
                Some(String::from(
                    "Connection retry for connecting to Azure Cloud Connector",
                )),
            )
            .await
            .map_err(CloudAdapterError::communication)
        })?;

        Ok(Box::new(Self {
            cloud_connector_client,
        }))
    }

    /// Sends the signal to the cloud
    ///
    /// # Arguments
    /// - `cloud_message`: represents a message to send to the cloud canonical model
    async fn send_to_cloud(
        &self,
        cloud_message: CloudMessageRequest,
    ) -> Result<CloudMessageResponse, CloudAdapterError> {
        debug!("Received a request to send to the cloud");
        let cloud_message_string =
            serde_json::to_string_pretty(&cloud_message).map_err(CloudAdapterError::serialize)?;
        debug!("Cloud canonical value:\n{cloud_message_string}");

        let cloud_digital_twin_instance = Self::get_instance_info_from_message(&cloud_message)?;

        let request = tonic::Request::new(UpdateDigitalTwinRequest {
            model_id: cloud_digital_twin_instance.model_id,
            instance_id: cloud_digital_twin_instance.instance_id,
            instance_property_path: cloud_digital_twin_instance.instance_property_path,
            data: cloud_message.signal_value,
        });

        let response = self
            .cloud_connector_client
            .clone()
            .update_digital_twin(request)
            .await
            .map_err(CloudAdapterError::communication)?;
        debug!("Response from cloud connector {response:?}");

        Ok(CloudMessageResponse {})
    }
}

#[cfg(test)]
mod azure_cloud_connector_tests {
    use super::*;

    use std::collections::HashMap;

    #[tokio::test]
    async fn get_instance_info_from_message_test() {
        let cloud_message = CloudMessageRequest {
            cloud_signal: HashMap::new(),
            signal_value: String::new(),
            signal_timestamp: String::new(),
        };
        let cloud_digital_twin_instance =
            AzureCloudConnectorAdapter::get_instance_info_from_message(&cloud_message);
        assert!(cloud_digital_twin_instance.is_err());

        let mut cloud_signal_map = HashMap::new();
        cloud_signal_map.insert(String::from(MODEL_ID_KEY), String::from("some-model-id"));
        cloud_signal_map.insert(
            String::from(INSTANCE_ID_KEY),
            String::from("some-instance-id"),
        );
        cloud_signal_map.insert(
            String::from(INSTANCE_PROPERTY_PATH_KEY),
            String::from("some-instance-property-path"),
        );
    }

    /// The tests below uses Unix sockets to create a channel between a gRPC client and a gRPC server.
    /// Unix sockets are more ideal than using TCP/IP sockets since Rust tests will run in parallel
    /// so you would need to set an arbitrary port per test for TCP/IP sockets.
    #[cfg(unix)]
    mod unix_tests {
        use super::*;

        use std::sync::Arc;

        use azure_cloud_connector_proto::azure_cloud_connector::azure_cloud_connector_server::{
            AzureCloudConnector, AzureCloudConnectorServer,
        };
        use azure_cloud_connector_proto::azure_cloud_connector::UpdateDigitalTwinResponse;
        use tempfile::TempPath;
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::transport::{Channel, Endpoint, Server, Uri};
        use tonic::{Request, Response, Status};
        use tower::service_fn;

        pub struct MockAzureConnector {}

        #[tonic::async_trait]
        impl AzureCloudConnector for MockAzureConnector {
            /// Updates a digital twin instance
            ///
            /// # Arguments
            /// - `request`: the request to send
            async fn update_digital_twin(
                &self,
                _request: Request<UpdateDigitalTwinRequest>,
            ) -> Result<Response<UpdateDigitalTwinResponse>, Status> {
                let response = UpdateDigitalTwinResponse {
                    reply: String::new(),
                };
                Ok(Response::new(response))
            }
        }

        async fn create_test_grpc_client(
            bind_path: Arc<TempPath>,
        ) -> AzureCloudConnectorClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let bind_path = bind_path.clone();
                    async move { UnixStream::connect(bind_path.as_ref()).await }
                }))
                .await
                .unwrap();

            AzureCloudConnectorClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_azure_connector = MockAzureConnector {};
            Server::builder()
                .add_service(AzureCloudConnectorServer::new(mock_azure_connector))
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

                let request = tonic::Request::new(UpdateDigitalTwinRequest {
                    model_id: String::from(
                        "dtmi:sdv:Cloud:Vehicle:Cabin:HVAC:AmbientAirTemperature;1",
                    ),
                    instance_id: String::from("hvac"),
                    instance_property_path: String::from("/AmbientAirTemperature"),
                    data: String::from("12.00"),
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
