// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    fs,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use core_protobuf_data_access::digital_twin::v1::{
    digital_twin_client::DigitalTwinClient, EndpointInfo, FindByIdRequest,
};
use log::{debug, error, warn};
use service_discovery_proto::service_registry::v1::service_registry_client::ServiceRegistryClient;
use service_discovery_proto::service_registry::v1::DiscoverRequest;
use tonic::{transport::Channel, Request};

use crate::config::{IbejiDiscoveryMetadata, Settings, CONFIG_FILE};
use dts_contracts::{
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, GetDigitalTwinProviderRequest,
        GetDigitalTwinProviderResponse,
    },
    entity::{Entity, EntityID},
    provider_proxy::OperationKind,
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
};

const GET_OPERATION: &str = "Get";
const SUBSCRIBE_OPERATION: &str = "Subscribe";

/// Contacts the In-Vehicle Digital Twin Service in Ibeji
pub struct IbejiAdapter {
    client: DigitalTwinClient<Channel>,
}

impl IbejiAdapter {
    /// Creates or updates a provider proxy for each entity using its info then caches the info in entity_map
    ///
    /// # Arguments
    /// - `entity_map`: shared map of entity ID to entity information
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    async fn init_provider_proxies_with_entities(
        &self,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
    ) -> Result<(), DigitalTwinAdapterError> {
        let mut entity_map_update;
        {
            entity_map_update = entity_map.lock().unwrap().clone();
        }

        // Update the copy of entity map if it contains an entity that has no information
        for (entity_id, entity) in entity_map_update.iter_mut() {
            if entity.is_some() {
                continue;
            }

            // Update the copy of entity map if we're able to find the info of an entity after doing find_by_id
            let request = GetDigitalTwinProviderRequest {
                entity_id: entity_id.clone(),
            };

            match self.find_by_id(request).await {
                Ok(response) => {
                    let entity_info = response.entity.clone();
                    let (id, uri, protocol, operation) = (
                        entity_info.id,
                        entity_info.uri,
                        entity_info.protocol,
                        entity_info.operation,
                    );
                    *entity = Some(response.entity);

                    // Notify the provider proxy selector to start a proxy
                    let request = ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(
                        id, uri, protocol, operation,
                    );
                    provider_proxy_selector_request_sender
                        .send_request_to_provider_proxy_selector(request);
                }
                Err(err) => {
                    error!("{err}");
                    *entity = None
                }
            };
        }

        *entity_map.lock().unwrap() = entity_map_update;

        Ok(())
    }

    /// Retrieves Ibeji's In-Vehicle Digital Twin URI from Chariott
    ///
    /// # Arguments
    /// - `chariott_service_discovery_uri`: the uri for Chariott's service discovery
    /// - `metadata`: optional configuration metadata for discovering Ibeji using Chariott
    async fn retrieve_ibeji_invehicle_digital_twin_uri_from_chariott(
        chariott_service_discovery_uri: &str,
        chariott_ibeji_config: IbejiDiscoveryMetadata,
    ) -> Result<String, DigitalTwinAdapterError> {
        let mut service_registry_client =
            ServiceRegistryClient::connect(String::from(chariott_service_discovery_uri))
                .await
                .map_err(DigitalTwinAdapterError::communication)?;

        let discover_request = Request::new(DiscoverRequest {
            namespace: chariott_ibeji_config.namespace,
            name: chariott_ibeji_config.name,
            version: chariott_ibeji_config.version,
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
    fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError> {
        let settings_content =
            fs::read_to_string(Path::new(env!("OUT_DIR")).join(CONFIG_FILE)).unwrap();
        let settings: Settings = serde_json::from_str(settings_content.as_str()).unwrap();

        let invehicle_digital_twin_service_uri = match settings {
            Settings::InVehicleDigitalTwinService { uri } => uri,
            Settings::ChariottDiscoveryService { uri, metadata } => {
                futures::executor::block_on(async {
                    Self::retrieve_ibeji_invehicle_digital_twin_uri_from_chariott(&uri, metadata)
                        .await
                })
                .unwrap()
            }
        };
        debug!("Discovered the uri of the In-Vehicle Digital Twin Service via Chariott: {invehicle_digital_twin_service_uri}");

        let client = futures::executor::block_on(async {
            DigitalTwinClient::connect(invehicle_digital_twin_service_uri)
                .await
                .map_err(DigitalTwinAdapterError::communication)
        })
        .unwrap();

        Ok(Box::new(Self { client }))
    }

    /// Gets entity access information
    ///
    /// # Arguments
    /// - `request`: the request for finding an entity's access information
    async fn find_by_id(
        &self,
        request: GetDigitalTwinProviderRequest,
    ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError> {
        let entity_id = request.entity_id;
        let request = tonic::Request::new(FindByIdRequest {
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
        let entity_endpoint_info_list = entity_access_info.endpoint_info_list;

        let endpoint: Option<(EndpointInfo, String)> = entity_endpoint_info_list
            .into_iter()
            .find_map(|endpoint_info| {
                endpoint_info.operations.iter().find_map(|operation| {
                    if *operation == SUBSCRIBE_OPERATION || *operation == GET_OPERATION {
                        return Some((endpoint_info.clone(), operation.clone()));
                    }
                    None
                })
            });

        if endpoint.is_none() {
            let message = format!("No access info to connect with {entity_id}");
            warn!("{message}");
            return Err(DigitalTwinAdapterError::communication(message));
        }

        let (endpoint, _) = endpoint.unwrap();

        // If both Subscribe and Get are supported, then we pick Subscribe over Get
        let operation = if endpoint
            .operations
            .iter()
            .any(|op| op == SUBSCRIBE_OPERATION)
        {
            String::from(SUBSCRIBE_OPERATION)
        } else {
            String::from(GET_OPERATION)
        };

        let operation =
            OperationKind::from_str(&operation).map_err(DigitalTwinAdapterError::parse_error)?;
        let entity = Entity {
            id: entity_id,
            description: Some(entity_access_info.description),
            name: Some(entity_access_info.name),
            operation,
            uri: endpoint.uri,
            protocol: endpoint.protocol,
        };
        Ok(GetDigitalTwinProviderResponse { entity })
    }

    /// Run as a client to the in-vehicle digital twin provider
    ///
    /// # Arguments
    /// - `entity_map`: shared map of entity ID to entity information
    /// - `sleep_interval`: the interval in milliseconds between finding the access info of entities
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    async fn run(
        &self,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        sleep_interval: Duration,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
    ) -> Result<(), DigitalTwinAdapterError> {
        loop {
            self.init_provider_proxies_with_entities(
                entity_map.clone(),
                provider_proxy_selector_request_sender.clone(),
            )
            .await?;
            tokio::time::sleep(sleep_interval).await;
        }
    }
}

#[cfg(test)]
mod ibeji_digital_twin_adapter_tests {
    use super::*;

    use core_protobuf_data_access::digital_twin::v1::{
        digital_twin_server::DigitalTwin, EntityAccessInfo, FindByIdRequest, FindByIdResponse,
        RegisterRequest, RegisterResponse,
    };
    use tonic::{Request, Response, Status};

    const AMBIENT_AIR_TEMPERATURE_ID: &str = "dtmi:sdv:Vehicle:Cabin:HVAC:AmbientAirTemperature;1";

    pub struct MockInVehicleTwin {}

    #[tonic::async_trait]
    impl DigitalTwin for MockInVehicleTwin {
        async fn find_by_id(
            &self,
            request: Request<FindByIdRequest>,
        ) -> Result<Response<FindByIdResponse>, Status> {
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

            let response = FindByIdResponse {
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

        use core_protobuf_data_access::digital_twin::v1::digital_twin_server::DigitalTwinServer;
        use tempfile::TempPath;
        use tokio::{
            net::{UnixListener, UnixStream},
            sync::mpsc,
        };
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::transport::{Channel, Endpoint, Server, Uri};
        use tower::service_fn;

        async fn create_test_grpc_client(bind_path: Arc<TempPath>) -> DigitalTwinClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let bind_path = bind_path.clone();
                    async move { UnixStream::connect(bind_path.as_ref()).await }
                }))
                .await
                .unwrap();

            DigitalTwinClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_in_vehicle_twin = MockInVehicleTwin {};
            Server::builder()
                .add_service(DigitalTwinServer::new(mock_in_vehicle_twin))
                .serve_with_incoming(uds_stream)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn init_provider_proxies_with_entities_test() {
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

                let (tx_provider_proxy_selector_request, _rx_provider_proxy_selector_request) =
                    mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();

                let entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>> =
                    Arc::new(Mutex::new(HashMap::new()));

                let provider_proxy_selector_request_sender = Arc::new(
                    ProviderProxySelectorRequestSender::new(tx_provider_proxy_selector_request),
                );
                let result = ibeji_digital_twin_adapter
                    .init_provider_proxies_with_entities(
                        entity_map,
                        provider_proxy_selector_request_sender,
                    )
                    .await;
                assert!(result.is_ok());
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }

            std::fs::remove_file(bind_path.as_ref()).unwrap();
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

                let request = GetDigitalTwinProviderRequest {
                    entity_id: String::from("invalid_entity"),
                };

                let result = ibeji_digital_twin_adapter.find_by_id(request).await;

                assert!(result.is_err());

                let request = GetDigitalTwinProviderRequest {
                    entity_id: String::from(AMBIENT_AIR_TEMPERATURE_ID),
                };
                let result = ibeji_digital_twin_adapter.find_by_id(request).await;
                assert!(result.is_ok());

                let response = result.unwrap();
                assert_eq!(response.entity.operation, OperationKind::Subscribe);
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }

            std::fs::remove_file(bind_path.as_ref()).unwrap();
        }
    }
}
