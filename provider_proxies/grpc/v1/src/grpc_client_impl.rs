// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crossbeam::queue::SegQueue;
use freyja_common::message_utils;
use log::{debug, warn};
use tonic::{Request, Response, Status};

use freyja_common::provider_proxy::SignalValue;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::{
    digital_twin_consumer_server::DigitalTwinConsumer, PublishRequest, PublishResponse,
    RespondRequest, RespondResponse,
};

/// Struct which implements the DigitalTwinConsumer trait for gRPC clients
#[derive(Debug, Default)]
pub struct GRPCClientImpl {
    /// The queue on which incoming signal values should be published
    pub signal_values_queue: Arc<SegQueue<SignalValue>>,
}

#[tonic::async_trait]
impl DigitalTwinConsumer for GRPCClientImpl {
    /// Publish implementation.
    ///
    /// # Arguments
    /// * `request` - Publish request.
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let PublishRequest { entity_id, value } = request.into_inner();

        debug!("Received a publish for entity id {entity_id} with the value {value}");

        let value = message_utils::parse_value(value);

        let new_signal_value = SignalValue { entity_id, value };
        self.signal_values_queue.push(new_signal_value);
        let response = PublishResponse {};
        Ok(Response::new(response))
    }

    /// Respond implementation.
    ///
    /// # Arguments
    /// * `request` - Respond request.
    async fn respond(
        &self,
        request: Request<RespondRequest>,
    ) -> Result<Response<RespondResponse>, Status> {
        warn!("Got a response request: {request:?}");

        Err(Status::unimplemented("respond has not been implemented"))
    }
}

#[cfg(test)]
mod grpc_client_impl_tests {
    use super::*;

    #[tokio::test]
    async fn publish_test() {
        let consumer_impl = GRPCClientImpl {
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let entity_id = String::from("some-id");
        let value = String::from("some-value");

        let request = tonic::Request::new(PublishRequest { entity_id, value });
        let result = consumer_impl.publish(request).await;
        assert!(result.is_ok());
    }
}
