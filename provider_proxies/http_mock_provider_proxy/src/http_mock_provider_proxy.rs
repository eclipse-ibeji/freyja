// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use async_trait::async_trait;
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use crossbeam::queue::SegQueue;
use freyja_common::{config_utils, out_dir};
use freyja_contracts::entity::EntityEndpoint;
use log::{debug, error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::{GET_OPERATION, SUBSCRIBE_OPERATION};
use freyja_contracts::provider_proxy::{
    ProviderProxy, ProviderProxyError, ProviderProxyErrorKind, SignalValue,
};

const CONFIG_FILE_STEM: &str = "http_mock_provider_proxy";
const CALLBACK_FOR_VALUES_PATH: &str = "/value";

macro_rules! ok {
    () => {
        (axum::http::StatusCode::OK, axum::Json("")).into_response()
    };
    ($body:expr) => {
        (axum::http::StatusCode::OK, axum::Json($body)).into_response()
    };
}

/// A request for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueRequest {
    /// The entity's ID
    pub entity_id: String,

    /// The callback uri for a provider to send data back
    pub callback_uri: String,
}

/// A response for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueResponse {
    // The id of the entity
    pub entity_id: String,

    /// The value of the entity
    pub value: String,
}

/// A provider proxy for our HTTP mocks/mock_digital_twin
#[derive(Debug)]
pub struct HttpMockProviderProxy {
    /// Async Reqwest HTTP Client
    client: Client,

    /// Local cache for keeping track of which entities this provider proxy contains
    entity_operation_map: Mutex<HashMap<String, String>>,

    /// Shared queue for all proxies to push new signal values
    signal_values_queue: Arc<SegQueue<SignalValue>>,

    /// The proxy configuration
    config: Config,

    /// The uri of the provider
    provider_uri: String,
}

impl HttpMockProviderProxy {
    /// Constructs a callback uri using a callback server authority and path
    ///
    /// # Arguments
    /// - `provider_callback_authority`: the callback server authority for receiving values from the mock digital twin
    fn construct_callback_uri(provider_callback_authority: &str) -> String {
        format!(
            "http://{}{CALLBACK_FOR_VALUES_PATH}", // Devskim: ignore DS137138
            String::from(provider_callback_authority)
        )
    }

    /// Receive signal handler for the value listener to handle incoming values
    ///
    /// # Arguments
    /// - `signal_values`: shared map of provider IDs to provider values
    /// - `value`: the value received from a provider
    async fn receive_value_handler(
        State(signal_values_queue): State<Arc<SegQueue<SignalValue>>>,
        Json(value): Json<EntityValueResponse>,
    ) -> Response {
        let EntityValueResponse { entity_id, value } = value;

        debug!("Received a response for entity id {entity_id} with the value {value}");

        let new_signal_value = SignalValue { entity_id, value };
        signal_values_queue.push(new_signal_value);

        ok!()
    }

    /// Run a listener, so the providers' server can publish data back
    ///
    /// # Arguments
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    async fn run_signal_values_listener(
        &self,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<(), ProviderProxyError> {
        let server_endpoint_addr = SocketAddr::from_str(&self.config.proxy_callback_address)
            .map_err(ProviderProxyError::parse)?;
        // Start a listener server to have a digital twin provider push data
        // http://{provider_callback_authority}/value
        // POST request where the json body is GetSignalValueResponse
        // Set up router path
        let router = Router::new()
            .route(CALLBACK_FOR_VALUES_PATH, post(Self::receive_value_handler))
            .with_state(signal_values_queue);

        // Run the listener
        let builder = axum::Server::try_bind(&server_endpoint_addr)
            .map_err(ProviderProxyError::communication)?;
        builder
            .serve(router.into_make_service())
            .await
            .map_err(ProviderProxyError::communication)?;

        info!(
            "Http Provider Proxy listening at http://{}", // Devskim: ignore DS137138
            self.config.proxy_callback_address
        );
        Ok(())
    }
}

#[async_trait]
impl ProviderProxy for HttpMockProviderProxy {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    fn create_new(
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError>
    where
        Self: Sized,
    {
        let config = config_utils::read_from_files(
            CONFIG_FILE_STEM,
            config_utils::JSON_EXT,
            out_dir!(),
            ProviderProxyError::io,
            ProviderProxyError::deserialize,
        )?;

        Ok(Self {
            signal_values_queue,
            config,
            provider_uri: provider_uri.to_string(),
            client: reqwest::Client::new(),
            entity_operation_map: Mutex::new(HashMap::new()),
        })
        .map(|r| Arc::new(r) as _)
    }

    /// Runs a provider proxy
    async fn run(&self) -> Result<(), ProviderProxyError> {
        info!("Started an HttpProviderProxy!");
        self.run_signal_values_listener(self.signal_values_queue.clone())
            .await?;
        Ok(())
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), ProviderProxyError> {
        let operation_result;
        {
            let lock = self.entity_operation_map.lock().unwrap();
            operation_result = lock.get(entity_id).cloned();
        }

        if operation_result.is_none() {
            return Err(ProviderProxyError::unknown(format!(
                "Entity {entity_id} does not have an operation registered"
            )));
        }

        // Only need to handle Get operations since subscribe has already happened
        let operation = operation_result.unwrap();
        if operation == GET_OPERATION {
            info!("Sending a get request to {entity_id}");

            let request = EntityValueRequest {
                entity_id: String::from(entity_id),
                callback_uri: Self::construct_callback_uri(&self.config.proxy_callback_address),
            };
            let server_endpoint = self.provider_uri.clone();

            self.client
                .post(&server_endpoint)
                .json(&request)
                .send()
                .await
                .map_err(ProviderProxyError::communication)?
                .error_for_status()
                .map_err(ProviderProxyError::unknown)?;
        }

        Ok(())
    }

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<(), ProviderProxyError> {
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

            result
                .ok_or::<ProviderProxyError>(ProviderProxyErrorKind::OperationNotSupported.into())?
        };

        self.entity_operation_map
            .lock()
            .unwrap()
            .insert(String::from(entity_id), String::from(selected_operation));

        if selected_operation == SUBSCRIBE_OPERATION {
            let request = EntityValueRequest {
                entity_id: String::from(entity_id),
                callback_uri: Self::construct_callback_uri(&self.config.proxy_callback_address),
            };

            let subscribe_endpoint_for_entity = self.provider_uri.clone();
            let result = self
                .client
                .post(&subscribe_endpoint_for_entity)
                .json(&request)
                .send()
                .await
                .map_err(ProviderProxyError::communication)?
                .error_for_status()
                .map_err(ProviderProxyError::unknown);

            // Remove from map if the subscribe operation fails
            if result.is_err() {
                error!("Cannot subscribe to {entity_id} due to {result:?}");
                self.entity_operation_map.lock().unwrap().remove(entity_id);
            }
        }

        Ok(())
    }
}
