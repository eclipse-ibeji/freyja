// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const CONFIG_FILE: &str = "mqtt_config.json";

/// Configuration for the MQTT Connector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Max retries for connecting to Azure Event Grid
    pub max_retries: u32,

    /// Retry interval in milliseconds
    pub retry_interval_ms: u64,

    /// gRPC Server Authority
    pub grpc_server_authority: String,

    /// Absolute path to certificate
    pub cert_path: String,

    /// Absolute path to private key
    pub private_key_path: String,

    /// The mqtt client id
    pub mqtt_client_id: String,

    /// The client authentication name to use, which is different from mqtt_client_id.
    /// The mqtt_client_id field is used to identify the client, whereas this field
    /// is used for authentication purposes.
    pub mqtt_client_authentication_name: String,

    /// The Event Grid topic to use for updating an Azure Digital Twin instance.
    pub event_grid_topic: String,

    /// The Event Grid Namespace hostname.
    pub event_grid_namespace_host_name: String,

    /// The Event Grid port number
    pub event_grid_port: String,
}
