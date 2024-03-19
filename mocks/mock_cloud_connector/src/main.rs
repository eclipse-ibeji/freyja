// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
mod mock_cloud_connector_impl;

use std::env;

use cloud_connector_proto::v1::cloud_connector_server::CloudConnectorServer;
use env_logger::Target;
use log::{info, LevelFilter};
use tonic::transport::Server;

use crate::{config::Config, mock_cloud_connector_impl::MockCloudConnectorImpl};
use freyja_build_common::config_file_stem;
use freyja_common::{
    cmd_utils::{get_log_level, parse_args},
    config_utils, out_dir,
};

/// Starts the following threads and tasks:
/// - A thread which listens for input from the command window
/// - A task which handles async get responses
/// - A task which handles publishing to subscribers
/// - A gRPC server to accept incoming requests
#[tokio::main]
async fn main() {
    let args = parse_args(env::args()).expect("Failed to parse args");

    // Setup logging
    let log_level = get_log_level(&args, LevelFilter::Info).expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let config: Config = config_utils::read_from_files(
        config_file_stem!(),
        config_utils::JSON_EXT,
        out_dir!(),
        |e| log::error!("{}", e),
        |e| log::error!("{}", e),
    )
    .unwrap();

    // Server setup
    info!(
        "Mock Cloud Connector Server starting at {}",
        config.server_authority
    );

    let addr = config
        .server_authority
        .parse()
        .expect("Unable to parse server address");

    let mock_cloud_connector = MockCloudConnectorImpl {};

    Server::builder()
        .add_service(CloudConnectorServer::new(mock_cloud_connector))
        .serve(addr)
        .await
        .unwrap();
}
