// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use core_protobuf_data_access::invehicle_digital_twin::v1::{
    invehicle_digital_twin_server::InvehicleDigitalTwin, EndpointInfo, EntityAccessInfo,
    FindByIdRequest, FindByIdResponse, RegisterRequest, RegisterResponse,
};
use log::info;
use tonic::{Request, Response, Status};

use crate::{find_entity, DigitalTwinAdapterState};

/// Implements an In-Vehicle Digital Twin Server
pub struct MockDigitalTwinImpl {
    /// The server state
    pub(crate) state: Arc<Mutex<DigitalTwinAdapterState>>,
}

#[async_trait]
impl InvehicleDigitalTwin for MockDigitalTwinImpl {
    /// Find-by-id implementation.
    ///
    /// # Arguments
    /// - `request`: Find-by-id request.
    async fn find_by_id(
        &self,
        request: Request<FindByIdRequest>,
    ) -> Result<Response<FindByIdResponse>, Status> {
        let request = request.into_inner();
        info!("Received request to get entity: {}", request.id);
        let state = self.state.lock().unwrap();
        find_entity(&state, &request.id)
            .map(|(config_item, _)| {
                let endpoint_info_list = config_item
                    .entity
                    .endpoints
                    .iter()
                    .map(|e| EndpointInfo {
                        protocol: e.protocol.clone(),
                        operations: e.operations.clone(),
                        uri: e.uri.clone(),
                        context: e.context.clone(),
                    })
                    .collect();

                let access_info = EntityAccessInfo {
                    name: config_item.entity.name.clone().unwrap_or_default(),
                    id: config_item.entity.id.clone(),
                    description: config_item.entity.description.clone().unwrap_or_default(),
                    endpoint_info_list,
                };

                Ok(Response::new(FindByIdResponse {
                    entity_access_info: Some(access_info),
                }))
            })
            .unwrap_or(Err(Status::not_found("Entity not found")))
    }

    /// Register implementation.
    ///
    /// # Arguments
    /// - `request`: Register request.
    async fn register(
        &self,
        _request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        Err(Status::unimplemented(
            "Register is not supported for the mock digital twin",
        ))
    }
}
