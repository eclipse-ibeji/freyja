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
use mqtt_connector_config::{Config, CONFIG_FILE};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load the configuration settings
    let config_file = fs::read_to_string(Path::new(env!("OUT_DIR")).join(CONFIG_FILE)).unwrap();
    let config: Config = serde_json::from_str(&config_file).unwrap();

    // Setup logging
    Builder::new()
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout)
        .init();

    info!("Starting the Azure MQTT Cloud Connector.");

    // Start a gRPC server and MQTT client
    let grpc_server_authority = config.grpc_server_authority.parse()?;
    let mqtt_connector = MQTTConnector::new(config).expect("Unable to read MQTT config");
    Server::builder()
        .add_service(AzureCloudConnectorServer::new(mqtt_connector))
        .serve(grpc_server_authority)
        .await?;
    Ok(())
}
