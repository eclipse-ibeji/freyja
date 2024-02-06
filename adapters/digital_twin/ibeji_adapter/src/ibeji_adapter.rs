// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use async_trait::async_trait;
use core_protobuf_data_access::invehicle_digital_twin::v1::{
    invehicle_digital_twin_client::InvehicleDigitalTwinClient,
    FindByIdRequest as IbejiFindByIdRequest,
};
use log::info;
use service_discovery_proto::service_registry::v1::{
    service_registry_client::ServiceRegistryClient, DiscoverRequest,
};
use tonic::{transport::Channel, Request};

use crate::config::{ChariottDiscoverRequest, Config};
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, FindByIdRequest, FindByIdResponse,
    },
    entity::{Entity, EntityEndpoint},
    out_dir,
    retry_utils::execute_with_retry,
};

/// Contacts the In-Vehicle Digital Twin Service in Ibeji
pub struct IbejiAdapter {
    client: InvehicleDigitalTwinClient<Channel>,
}

impl IbejiAdapter {
    /// Retrieves Ibeji's In-Vehicle Digital Twin URI from Chariott
    ///
    /// # Arguments
    /// - `chariott_discovery_request`: the uri for Chariott's service discovery
    /// - `metadata`: optional configuration metadata for discovering Ibeji using Chariott
    async fn retrieve_ibeji_invehicle_digital_twin_uri_from_chariott(
        chariott_service_discovery_uri: &str,
        chariott_discovery_request: ChariottDiscoverRequest,
    ) -> Result<String, DigitalTwinAdapterError> {
        let mut service_registry_client =
            ServiceRegistryClient::connect(chariott_service_discovery_uri.to_owned())
                .await
                .map_err(DigitalTwinAdapterError::communication)?;

        let discover_request = Request::new(DiscoverRequest {
            namespace: chariott_discovery_request.namespace,
            name: chariott_discovery_request.name,
            version: chariott_discovery_request.version,
        });

        let service = service_registry_client
            .discover(discover_request)
            .await
            .map_err(DigitalTwinAdapterError::communication)?
            .into_inner()
            .service
            .ok_or_else(|| {
                DigitalTwinAdapterError::communication(
                    "Cannot discover the uri of Ibeji's In-Vehicle Digital Twin Service",
                )
            })?;

        Ok(service.uri)
    }
}

#[async_trait]
impl DigitalTwinAdapter for IbejiAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    fn create_new() -> Result<Self, DigitalTwinAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            DigitalTwinAdapterError::io,
            DigitalTwinAdapterError::deserialize,
        )?;

        let (invehicle_digital_twin_service_uri, max_retries, retry_interval_ms) = match config {
            Config::FromConfig {
                uri,
                max_retries,
                retry_interval_ms,
            } => (uri, max_retries, retry_interval_ms),
            Config::ChariottServiceDiscovery {
                uri,
                max_retries,
                retry_interval_ms,
                discover_request,
            } => {
                let invehicle_digital_twin_service_uri = futures::executor::block_on(async {
                    execute_with_retry(
                        max_retries,
                        Duration::from_millis(retry_interval_ms),
                        || {
                            Self::retrieve_ibeji_invehicle_digital_twin_uri_from_chariott(
                                &uri,
                                discover_request.clone(),
                            )
                        },
                        Some(String::from("Connection retry for connecting to Chariott")),
                    )
                    .await
                })
                .map_err(DigitalTwinAdapterError::communication)?;

                info!("Discovered the uri of the In-Vehicle Digital Twin Service via Chariott: {invehicle_digital_twin_service_uri}");

                (
                    invehicle_digital_twin_service_uri,
                    max_retries,
                    retry_interval_ms,
                )
            }
        };

        let client = futures::executor::block_on(async {
            execute_with_retry(
                max_retries,
                Duration::from_millis(retry_interval_ms),
                || InvehicleDigitalTwinClient::connect(invehicle_digital_twin_service_uri.clone()),
                Some(String::from("Connection retry for connecting to Ibeji")),
            )
            .await
            .map_err(DigitalTwinAdapterError::communication)
        })
        .map_err(DigitalTwinAdapterError::communication)?;

        Ok(Self { client })
    }

    /// Gets entity access information
    ///
    /// # Arguments
    /// - `request`: the request for finding an entity's access information
    async fn find_by_id(
        &self,
        request: FindByIdRequest,
    ) -> Result<FindByIdResponse, DigitalTwinAdapterError> {
        let entity_id = request.entity_id;
        let request = tonic::Request::new(IbejiFindByIdRequest {
            id: entity_id.clone(),
        });

        let response = self
            .client
            .clone()
            .find_by_id(request)
            .await
            .map_err(DigitalTwinAdapterError::entity_not_found)?;

        // Extract the response from find_by_id
        let entity_access_info = response
            .into_inner()
            .entity_access_info
            .ok_or(format!("Cannot find {entity_id} with find_by_id"))
            .map_err(DigitalTwinAdapterError::entity_not_found)?;

        Ok(FindByIdResponse {
            entity: Entity {
                id: entity_access_info.id,
                name: Some(entity_access_info.name),
                description: Some(entity_access_info.description),
                endpoints: entity_access_info
                    .endpoint_info_list
                    .into_iter()
                    .map(|e| EntityEndpoint {
                        protocol: e.protocol,
                        operations: e.operations,
                        uri: e.uri,
                        context: e.context,
                    })
                    .collect(),
            },
        })
    }
}

#[cfg(test)]
mod ibeji_digital_twin_adapter_tests {
    use core_protobuf_data_access::invehicle_digital_twin::v1::{
        invehicle_digital_twin_server::InvehicleDigitalTwin, EndpointInfo, EntityAccessInfo,
        FindByIdResponse as IbejiFindByIdResponse, RegisterRequest, RegisterResponse,
    };
    use tonic::{Response, Status};

    use super::*;

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

    pub struct MockInVehicleTwin {}

    #[tonic::async_trait]
    impl InvehicleDigitalTwin for MockInVehicleTwin {
        async fn find_by_id(
            &self,
            request: Request<IbejiFindByIdRequest>,
        ) -> Result<Response<IbejiFindByIdResponse>, Status> {
            let entity_id = request.into_inner().id;

            if entity_id != AMBIENT_AIR_TEMPERATURE_ID {
                return Err(Status::not_found(
                    "Unable to find the entity with id {entity_id}",
                ));
            }

            let endpoint_info = EndpointInfo {
                protocol: String::from("grpc"),
                uri: String::from("http://[::1]:40010"), // Devskim: ignore DS137138
                context: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
                operations: vec![String::from("Get"), String::from("Subscribe")],
            };

            let entity_access_info = EntityAccessInfo {
                name: String::from("AmbientAirTemperature"),
                id: String::from("dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1"),
                description: String::from("Ambient air temperature"),
                endpoint_info_list: vec![endpoint_info],
            };

            let response = IbejiFindByIdResponse {
                entity_access_info: Some(entity_access_info),
            };

            Ok(Response::new(response))
        }

        async fn register(
            &self,
            _request: Request<RegisterRequest>,
        ) -> Result<Response<RegisterResponse>, Status> {
            let response = RegisterResponse {};
            Ok(Response::new(response))
        }
    }

    /// The tests below uses Unix sockets to create a channel between a gRPC client and a gRPC server.
    /// Unix sockets are more ideal than using TCP/IP sockets since Rust tests will run in parallel
    /// so you would need to set an arbitrary port per test for TCP/IP sockets.
    #[cfg(unix)]
    mod unix_tests {
        use super::*;

        use std::sync::Arc;

        use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
        use tempfile::TempPath;
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::transport::{Channel, Endpoint, Server, Uri};
        use tower::service_fn;

        async fn create_test_grpc_client(
            bind_path: Arc<TempPath>,
        ) -> InvehicleDigitalTwinClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let bind_path = bind_path.clone();
                    async move { UnixStream::connect(bind_path.as_ref()).await }
                }))
                .await
                .unwrap();

            InvehicleDigitalTwinClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_in_vehicle_twin = MockInVehicleTwin {};
            Server::builder()
                .add_service(InvehicleDigitalTwinServer::new(mock_in_vehicle_twin))
                .serve_with_incoming(uds_stream)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn find_by_id_test() {
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
                let client = create_test_grpc_client(bind_path.clone()).await;
                let ibeji_digital_twin_adapter = IbejiAdapter { client };

                let request = FindByIdRequest {
                    entity_id: String::from("invalid_entity"),
                };

                let result = ibeji_digital_twin_adapter.find_by_id(request).await;

                assert!(result.is_err());

                let request = FindByIdRequest {
                    entity_id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
                };

                let result = ibeji_digital_twin_adapter.find_by_id(request).await;
                assert!(result.is_ok());
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }

            std::fs::remove_file(bind_path.as_ref()).unwrap();
        }
    }
}
