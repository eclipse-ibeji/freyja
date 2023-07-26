// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod mqtt_connector;
mod mqtt_connector_config;

use std::{fs, path::Path};

use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use tonic::transport::Server;

use azure_cloud_connector_proto::azure_cloud_connector::azure_cloud_connector_server::AzureCloudConnectorServer;
use mqtt_connector::MQTTConnector;
use mqtt_connector_config::{
    GRPCConfigItem, MQTTConfigItem, CLOUD_CONNECTOR_CONFIG_FILENAME,
    MQTT_FILE_RELATIVE_TO_OUTPUT_DIR, OUTPUT_DIR_PATH,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load the MQTT config
    let mqtt_config_file =
        fs::read_to_string(Path::new(OUTPUT_DIR_PATH).join(MQTT_FILE_RELATIVE_TO_OUTPUT_DIR))
            .unwrap();
    let mqtt_config: MQTTConfigItem = serde_json::from_str(&mqtt_config_file).unwrap();

    // Load the gRPC config
    let grpc_config_file =
        fs::read_to_string(Path::new(OUTPUT_DIR_PATH).join(CLOUD_CONNECTOR_CONFIG_FILENAME))
            .unwrap();
    let grpc_config: GRPCConfigItem = serde_json::from_str(&grpc_config_file).unwrap();
    let grpc_server_authority = grpc_config.grpc_server_authority.clone();

    // Setup logging
    Builder::new()
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout)
        .init();

    info!("Starting the Azure MQTT Cloud Connector.");

    // Start a gRPC server and MQTT client
    let mqtt_connector =
        MQTTConnector::new(mqtt_config).expect("Please make sure you have edited the target/debug/mqtt_config.json");
    Server::builder()
        .add_service(AzureCloudConnectorServer::new(mqtt_connector))
        .serve(grpc_server_authority.parse()?)
        .await?;
    Ok(())
}
