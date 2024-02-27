// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use log::info;
use samples_protobuf_data_access::sample_grpc::v1::{
    digital_twin_consumer::PublishRequest,
    digital_twin_provider::{
        digital_twin_provider_server::DigitalTwinProvider, GetRequest, GetResponse, InvokeRequest,
        InvokeResponse, SetRequest, SetResponse, StreamRequest, StreamResponse, SubscribeRequest,
        SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
    },
};
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::{find_entity, get_entity_value, DigitalTwinAdapterState};

/// Implements a digital twin provider
pub struct MockProvider {
    /// The server state
    pub(crate) state: Arc<Mutex<DigitalTwinAdapterState>>,
}

#[async_trait]
impl DigitalTwinProvider for MockProvider {
    type StreamStream = Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send>>;

    /// Subscribe implementation.
    ///
    /// # Arguments
    /// - `request`: Subscribe request.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        let request = request.into_inner();
        info!("Received subscribe request: {request:?}");
        let mut state = self.state.lock().unwrap();

        match find_entity(&state, &request.entity_id) {
            Some(_) => {
                state
                    .subscriptions
                    .entry(request.entity_id)
                    .and_modify(|e| {
                        e.insert(request.consumer_uri);
                    });
                Ok(Response::new(SubscribeResponse {}))
            }
            None => Err(Status::not_found("Entity not found")),
        }
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// - `request`: Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request = request.into_inner();
        info!("Received request to get value: {request:?}");
        let mut state = self.state.lock().unwrap();
        match get_entity_value(&mut state, &request.entity_id) {
            Some(value) => {
                let publish_request = PublishRequest {
                    entity_id: request.entity_id,
                    value,
                };

                info!("Submitting request...");
                match state
                    .response_channel_sender
                    .send((request.consumer_uri, publish_request))
                {
                    Ok(_) => Ok(Response::new(GetResponse {})),
                    Err(e) => Err(Status::internal(format!("Request value error: {e:?}"))),
                }
            }
            None => Err(Status::not_found("Entity not found")),
        }
    }

    /// Unsubscribe implementation.
    ///
    /// # Arguments
    /// - `request`: Unsubscribe request.
    async fn unsubscribe(
        &self,
        _request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        Err(Status::unimplemented(
            "Unsubscribe is not supported for the mock digital twin",
        ))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// - `request`: Set request.
    async fn set(&self, _request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        Err(Status::unimplemented(
            "Set is not supported for the mock digital twin",
        ))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// - `request`: Invoke request.
    async fn invoke(
        &self,
        _request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        Err(Status::unimplemented(
            "Invoke is not supported for the mock digital twin",
        ))
    }

    /// Stream implementation.
    ///
    /// # Arguments
    /// - `request`: OpenStream request.
    async fn stream(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamStream>, Status> {
        Err(Status::unimplemented(
            "Stream is not supported for the mock digital twin",
        ))
    }
}
