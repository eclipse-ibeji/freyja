// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::time::Duration;

use log::info;
use paho_mqtt::{self as mqtt, MQTT_VERSION_5};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

use crate::mqtt_connector_config::Config;
use azure_cloud_connector_proto::azure_cloud_connector::azure_cloud_connector_server::AzureCloudConnector;
use azure_cloud_connector_proto::azure_cloud_connector::{
    UpdateDigitalTwinRequest, UpdateDigitalTwinResponse,
};
use freyja_common::utils::execute_with_retry;

/// Implementation of the MQTTConnector gRPC trait
pub struct MQTTConnector {
    mqtt_event_grid_client: mqtt::AsyncClient,
    mqtt_event_grid_topic: String,
}

/// The event grid payload for updating a digital twin instance
#[derive(Debug, Serialize, Deserialize)]
struct EventGridDigitalTwinPayload {
    model_id: String,
    instance_id: String,
    instance_property_path: String,
    data: String,
}

impl MQTTConnector {
    /// Creates an instance of MQTTConnector
    ///
    /// # Arguments
    /// - `config`: the config file
    pub fn new(config: Config) -> Result<Self, MQTTConnectorError> {
        let event_grid_mqtt_uri = format!(
            "mqtts://{}:{}",
            config.event_grid_namespace_host_name, config.event_grid_port
        );

        let mqtt_event_grid_client = mqtt::CreateOptionsBuilder::new()
            .server_uri(event_grid_mqtt_uri)
            .client_id(config.mqtt_client_id)
            .mqtt_version(MQTT_VERSION_5)
            .max_buffered_messages(100)
            .create_client()
            .map_err(MQTTConnectorError::communication)?;

        // The key_store option uses a self-signed certificate
        let ssl_options = mqtt::SslOptionsBuilder::new()
            .key_store(config.cert_path)
            .map_err(MQTTConnectorError::io)?
            .private_key(config.private_key_path)
            .map_err(MQTTConnectorError::io)?
            .finalize();
        let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
            .ssl_options(ssl_options)
            .user_name(config.mqtt_client_authentication_name)
            .clean_start(true)
            .finalize();

        futures::executor::block_on(async {
            execute_with_retry(
                config.max_retries,
                Duration::from_millis(config.retry_interval_ms),
                || mqtt_event_grid_client.connect(conn_opts.clone()),
                Some(String::from(
                    "Connection retry for connecting to your Azure Event Grid",
                )),
            )
            .await
            .map_err(MQTTConnectorError::communication)
        })?;

        Ok(MQTTConnector {
            mqtt_event_grid_client,
            mqtt_event_grid_topic: config.event_grid_topic,
        })
    }
}

#[tonic::async_trait]
impl AzureCloudConnector for MQTTConnector {
    /// Updates a digital twin instance
    ///
    /// # Arguments
    /// - `request`: the request to send
    async fn update_digital_twin(
        &self,
        request: Request<UpdateDigitalTwinRequest>,
    ) -> Result<Response<UpdateDigitalTwinResponse>, Status> {
        let request_inner = request.into_inner();

        let mqtt_payload = EventGridDigitalTwinPayload {
            model_id: request_inner.model_id.clone(),
            instance_id: request_inner.instance_id.clone(),
            instance_property_path: request_inner.instance_property_path.clone(),
            data: request_inner.data.clone(),
        };

        let message = mqtt::MessageBuilder::new()
            .topic(self.mqtt_event_grid_topic.clone())
            .payload(
                serde_json::to_vec(&mqtt_payload)
                    .map_err(|error| Status::failed_precondition(error.to_string()))?,
            )
            .qos(1)
            .finalize();

        self.mqtt_event_grid_client
            .publish(message)
            .await
            .map_err(|error| Status::internal(error.to_string()))?;

        let reply = format!(
            "Successfully set {}{} based on model {} to {}",
            request_inner.instance_id,
            request_inner.instance_property_path,
            request_inner.model_id,
            request_inner.data
        );

        info!("{reply}");

        Ok(Response::new(UpdateDigitalTwinResponse { reply }))
    }
}

#[cfg(test)]
mod azure_cloud_connector_tests {
    use super::*;

    #[tokio::test]
    async fn update_digital_twin_with_no_broker_test() {
        let consumer_impl = MQTTConnector {
            mqtt_event_grid_client: mqtt::CreateOptionsBuilder::new().create_client().unwrap(),
            mqtt_event_grid_topic: String::new(),
        };

        let request = tonic::Request::new(UpdateDigitalTwinRequest {
            model_id: String::new(),
            instance_id: String::new(),
            instance_property_path: String::new(),
            data: String::new(),
        });

        let result = consumer_impl.update_digital_twin(request).await;

        assert!(result.is_err());
    }
}

proc_macros::error! {
    MQTTConnectorError {
        Io,
        Communication,
    }
}
