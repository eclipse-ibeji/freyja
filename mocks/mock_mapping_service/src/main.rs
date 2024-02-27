// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
mod mock_mapping_service_impl;

use std::{
    env, io,
    sync::{Arc, Mutex},
    thread,
};

use env_logger::Target;
use log::{info, LevelFilter};
use mapping_service_proto::v1::mapping_service_server::MappingServiceServer;
use tonic::transport::Server;

use config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    cmd_utils::{get_log_level, parse_args},
    config_utils, out_dir,
};

use crate::mock_mapping_service_impl::MockMappingServiceImpl;

/// Stores the state of the mapping service
struct MappingState {
    /// An internal count that dictates which mappings are enabled
    count: u8,

    /// Indicates whether or not there is an update to the mapping that a client has not yet consumed
    pending_work: bool,

    /// The mapping service config
    config: Config,

    /// Whether or not the application is in interactive mode
    interactive: bool,
}

/// Starts the following threads and tasks:
/// - A thread which listens for input from the command window
/// - A GRPC server to accept incoming requests
#[tokio::main]
async fn main() {
    let args = parse_args(env::args()).expect("Failed to parse args");

    // Setup logging
    let log_level = get_log_level(&args, LevelFilter::Info).expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let interactive = args.get("interactive").is_some();

    let config: Config = config_utils::read_from_files(
        config_file_stem!(),
        config_utils::JSON_EXT,
        out_dir!(),
        |e| log::error!("{}", e),
        |e| log::error!("{}", e),
    )
    .unwrap();

    let server_endpoint = config.mapping_server_authority.clone();

    let state = Arc::new(Mutex::new(MappingState {
        count: 0,
        pending_work: check_for_work(&config, 0, interactive),
        config: config.clone(),
        interactive,
    }));

    let state_clone = state.clone();

    {
        let initial_work = state.lock().unwrap().pending_work;
        info!("Initial work? {initial_work}");
    }

    if interactive {
        // stdin setup
        thread::spawn(move || -> std::io::Result<usize> {
            let mut buffer = String::new();
            loop {
                io::stdin().read_line(&mut buffer)?;

                let mut state = state_clone.lock().unwrap();
                state.count += 1;
                let new_work = check_for_work(&config, state.count, state.interactive);

                state.pending_work |= new_work;
                info!(
                    "New count: {}. Work available? {}",
                    state.count, state.pending_work
                );

                if state.pending_work {
                    let work_available_state: Vec<String> = state
                        .config
                        .values
                        .iter()
                        .filter(|c| state.count == c.begin)
                        .map(|v| v.value.source.clone())
                        .collect();

                    info!("New work available for {work_available_state:?}");
                }
            }
        });
    }

    // Server setup
    info!("Mock Mapping Server starting at {}", server_endpoint);

    let addr = server_endpoint
        .parse()
        .expect("Unable to parse server address");

    let mock_mapping_service = MockMappingServiceImpl {
        state: state.clone(),
    };

    Server::builder()
        .add_service(MappingServiceServer::new(mock_mapping_service))
        .serve(addr)
        .await
        .unwrap();
}

/// Checks to see if there is pending work
///
/// # Arguments
/// - `config`: the mapping service config
/// - `n`: the current count
/// - `interactive`: whether or not the service is running in interactive mode
fn check_for_work(config: &Config, n: u8, interactive: bool) -> bool {
    config.values.iter().any(|c| {
        (!interactive && n == 0)
            || match c.end {
                Some(end) => {
                    if n == end {
                        info!("End of {} for mapping", c.value.source);
                    }
                    n == end || n == c.begin
                }

                None => n == c.begin,
            }
    })
}
