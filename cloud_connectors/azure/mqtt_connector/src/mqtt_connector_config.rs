// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub(crate) const OUTPUT_DIR_PATH: &str = env!("OUT_DIR");
pub(crate) const MQTT_FILE_RELATIVE_TO_OUTPUT_DIR: &str = "../../../mqtt_config.json";

/// Configuration for the MQTT Connector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
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

    /// The mqtt topic to use for updating an Azure Digital Twin instance.
    pub mqtt_event_grid_topic: String,

    /// The mqtt event grid hostname. Obtained
    pub mqtt_event_grid_host_name: String,
}
