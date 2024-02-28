// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use log::info;
use samples_protobuf_data_access::sample_grpc::v1::{
    digital_twin_consumer::digital_twin_consumer_server::DigitalTwinConsumerServer,
    digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient,
    digital_twin_provider::{GetRequest, SubscribeRequest},
};
use tonic::transport::{Channel, Server};

use crate::{config::Config, grpc_client_impl::GRPCClientImpl, GET_OPERATION, SUBSCRIBE_OPERATION};
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterErrorKind, EntityRegistration},
    entity::EntityEndpoint,
    out_dir,
    signal_store::SignalStore,
};

/// Interfaces with providers which support GRPC. Based on the Ibeji mixed sample.
pub struct SampleGRPCDataAdapter {
    /// The adapter config
    config: Config,

    /// Client for connecting to a provider
    provider_client: DigitalTwinProviderClient<Channel>,

    /// Local cache for keeping track of which entities this data adapter contains
    entity_operation_map: Mutex<HashMap<String, String>>,

    /// Shared signal store for all data adapters to push new signal values
    signals: Arc<SignalStore>,
}

#[async_trait]
impl DataAdapter for SampleGRPCDataAdapter {
    /// Creates a data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signals`: the shared signal store
    fn create_new(provider_uri: &str, signals: Arc<SignalStore>) -> Result<Self, DataAdapterError>
    where
        Self: Sized,
    {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            DataAdapterError::io,
            DataAdapterError::deserialize,
        )?;

        let provider_client = futures::executor::block_on(async {
            DigitalTwinProviderClient::connect(String::from(provider_uri))
                .await
                .map_err(DataAdapterError::communication)
        })?;

        Ok(Self {
            config,
            provider_client,
            entity_operation_map: Mutex::new(HashMap::new()),
            signals,
        })
    }

    /// Starts a data adapter
    async fn start(&self) -> Result<(), DataAdapterError> {
        let addr: SocketAddr = self
            .config
            .consumer_address
            .parse()
            .map_err(DataAdapterError::parse)
            .unwrap();

        let consumer_impl = GRPCClientImpl {
            signals: self.signals.clone(),
        };
        let server_future = Server::builder()
            .add_service(DigitalTwinConsumerServer::new(consumer_impl))
            .serve(addr);

        tokio::spawn(async move {
            let _ = server_future.await;
        });

        info!("Started a GRPCDataAdapter!");

        Ok(())
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), DataAdapterError> {
        let consumer_uri = format!("http://{}", self.config.get_advertised_address()); // Devskim: ignore DS137138

        let operation_result;
        {
            let lock = self.entity_operation_map.lock().unwrap();
            operation_result = lock.get(entity_id).cloned();
        }

        if operation_result.is_none() {
            let message = format!("Entity {entity_id} does not have an operation registered");
            info!("{message}");
            return Err(DataAdapterError::unknown(message));
        }

        // Only need to handle Get operations since subscribe has already happened
        let operation = operation_result.unwrap();
        if operation == GET_OPERATION {
            let mut client = self.provider_client.clone();
            let request = tonic::Request::new(GetRequest {
                entity_id: String::from(entity_id),
                consumer_uri,
            });
            client
                .get(request)
                .await
                .map_err(DataAdapterError::communication)?;
        }

        Ok(())
    }

    /// Registers an entity id to a local cache inside a data adapter to keep track of which entities a data adapter contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<EntityRegistration, DataAdapterError> {
        // Prefer subscribe if present
        let selected_operation = {
            let mut result = None;
            for operation in endpoint.operations.iter() {
                if operation == SUBSCRIBE_OPERATION {
                    result = Some(SUBSCRIBE_OPERATION);
                    break;
                } else if operation == GET_OPERATION {
                    // Set result, but don't break the loop in case there's a subscribe operation later in the list
                    result = Some(GET_OPERATION);
                }
            }

            result.ok_or::<DataAdapterError>(DataAdapterErrorKind::OperationNotSupported.into())?
        };

        self.entity_operation_map
            .lock()
            .unwrap()
            .insert(String::from(entity_id), String::from(selected_operation));

        if selected_operation == SUBSCRIBE_OPERATION {
            let consumer_uri = format!("http://{}", self.config.get_advertised_address()); // Devskim: ignore DS137138
            let mut client = self.provider_client.clone();
            let request = tonic::Request::new(SubscribeRequest {
                entity_id: String::from(entity_id),
                consumer_uri,
            });

            let result = client
                .subscribe(request)
                .await
                .map_err(DataAdapterError::communication);

            // Remove from map if subscribing to the provider fails
            if result.is_err() {
                self.entity_operation_map.lock().unwrap().remove(entity_id);
            }
        }

        Ok(EntityRegistration::Registered)
    }
}

#[cfg(test)]
mod sample_grpc_data_adapter_tests {
    use std::pin::Pin;

    use super::*;

    use tokio_stream::Stream;
    use tonic::{Request, Response, Status};

    use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
        digital_twin_provider_server::{DigitalTwinProvider, DigitalTwinProviderServer},
        GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse, StreamRequest,
        StreamResponse, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
    };

    pub struct MockProvider {}

    #[tonic::async_trait]
    impl DigitalTwinProvider for MockProvider {
        // This is required by the Ibeji contract
        type StreamStream = Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send>>;
        async fn subscribe(
            &self,
            _request: Request<SubscribeRequest>,
        ) -> Result<Response<SubscribeResponse>, Status> {
            let response = SubscribeResponse {};
            Ok(Response::new(response))
        }

        async fn unsubscribe(
            &self,
            _request: Request<UnsubscribeRequest>,
        ) -> Result<Response<UnsubscribeResponse>, Status> {
            Err(Status::unimplemented(
                "unsubscribe has not been implemented",
            ))
        }

        async fn get(
            &self,
            _request: Request<GetRequest>,
        ) -> Result<Response<GetResponse>, Status> {
            let response = GetResponse {};
            Ok(Response::new(response))
        }

        async fn set(
            &self,
            _request: Request<SetRequest>,
        ) -> Result<Response<SetResponse>, Status> {
            Err(Status::unimplemented("set has not been implemented"))
        }

        async fn invoke(
            &self,
            _request: Request<InvokeRequest>,
        ) -> Result<Response<InvokeResponse>, Status> {
            Err(Status::unimplemented("invoke has not been implemented"))
        }

        async fn stream(
            &self,
            _request: Request<StreamRequest>,
        ) -> Result<Response<Self::StreamStream>, Status> {
            Err(Status::unimplemented("stream has not been implemented"))
        }
    }

    /// The tests below uses Unix sockets to create a channel between a gRPC client and a gRPC server.
    /// Unix sockets are more ideal than using TCP/IP sockets since Rust tests will run in parallel
    /// so you would need to set an arbitrary port per test for TCP/IP sockets.
    #[cfg(unix)]
    mod unix_tests {
        use crate::GRPC_PROTOCOL;

        use super::*;

        use std::{path::PathBuf, sync::Arc};

        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use tonic::transport::{Channel, Endpoint, Server, Uri};
        use tower::service_fn;

        use freyja_test_common::fixtures::GRPCTestFixture;

        async fn create_test_grpc_client(
            socket_path: PathBuf,
        ) -> DigitalTwinProviderClient<Channel> {
            let channel = Endpoint::try_from("http://URI_IGNORED") // Devskim: ignore DS137138
                .unwrap()
                .connect_with_connector(service_fn(move |_: Uri| {
                    let socket_path = socket_path.clone();
                    async move { UnixStream::connect(socket_path).await }
                }))
                .await
                .unwrap();

            DigitalTwinProviderClient::new(channel)
        }

        async fn run_test_grpc_server(uds_stream: UnixListenerStream) {
            let mock_provider = MockProvider {};
            Server::builder()
                .add_service(DigitalTwinProviderServer::new(mock_provider))
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
                let client = create_test_grpc_client(fixture.socket_path.clone()).await;
                let grpc_data_adapter = SampleGRPCDataAdapter {
                    config: Config {
                        consumer_address: "[::1]:60010".to_string(),
                        advertised_consumer_address: None,
                    },
                    provider_client: client,
                    entity_operation_map: Mutex::new(HashMap::new()),
                    signals: Arc::new(SignalStore::new()),
                };
                assert!(grpc_data_adapter
                    .send_request_to_provider("unknown_entity_id")
                    .await
                    .is_err());

                let entity_id = "operation_get_entity_id";

                let result = grpc_data_adapter
                    .register_entity(
                        entity_id,
                        &EntityEndpoint {
                            protocol: GRPC_PROTOCOL.to_string(),
                            operations: vec![GET_OPERATION.to_string()],
                            uri: "foo".to_string(),
                            context: String::from("context"),
                        },
                    )
                    .await;
                assert!(result.is_ok());
                assert!(grpc_data_adapter
                    .send_request_to_provider(entity_id)
                    .await
                    .is_ok());

                let entity_id = "operation_subscribe_entity_id";
                let result = grpc_data_adapter
                    .register_entity(
                        entity_id,
                        &EntityEndpoint {
                            protocol: GRPC_PROTOCOL.to_string(),
                            operations: vec![SUBSCRIBE_OPERATION.to_string()],
                            uri: "foo".to_string(),
                            context: String::from("context"),
                        },
                    )
                    .await;
                assert!(result.is_ok());
                assert!(grpc_data_adapter
                    .send_request_to_provider(entity_id)
                    .await
                    .is_ok());
            };

            tokio::select! {
                _ = run_test_grpc_server(uds_stream) => (),
                _ = request_future => ()
            }
        }
    }
}
