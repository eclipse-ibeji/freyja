// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use cloud_connector_proto::v1::{
    cloud_connector_server::CloudConnector, UpdateDigitalTwinRequest, UpdateDigitalTwinResponse,
};
use log::info;
use tonic::{Request, Response, Status};

/// Implements a Mock Cloud Connector
pub struct MockCloudConnectorImpl {}

#[async_trait]
impl CloudConnector for MockCloudConnectorImpl {
    /// Update the digital twin
    ///
    /// # Arguments
    /// - `request`: the update request+
    async fn update_digital_twin(
        &self,
        request: Request<UpdateDigitalTwinRequest>,
    ) -> Result<Response<UpdateDigitalTwinResponse>, Status> {
        let message_json = serde_json::to_string_pretty(&request.into_inner())
            .map_err(|_| Status::invalid_argument("Could not parse request"))?;

        info!("Mock Cloud Connector received a message!\n{message_json}");

        Ok(Response::new(UpdateDigitalTwinResponse {}))
    }
}
