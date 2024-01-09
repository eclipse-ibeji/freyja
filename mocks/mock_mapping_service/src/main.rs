// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;

use std::{
    env, io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use env_logger::Target;
use log::{info, LevelFilter};

use config::Config;
use freyja_build_common::config_file_stem;
use freyja_common::{
    cmd_utils::{get_log_level, parse_args},
    config_utils, out_dir,
};
use freyja_common::{
    mapping_client::{
        CheckForWorkResponse, GetMappingResponse, SendInventoryRequest, SendInventoryResponse,
    },
    ok,
};

struct MappingState {
    count: u8,
    pending_work: bool,
    config: Config,
    interactive: bool,
}

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

    let config = config_utils::read_from_files(
        config_file_stem!(),
        config_utils::JSON_EXT,
        out_dir!(),
        |e| log::error!("{}", e),
        |e| log::error!("{}", e),
    )
    .unwrap();

    const SERVER_ENDPOINT: &str = "127.0.0.1:8888";

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

    info!("Mock Mapping Server starting at {SERVER_ENDPOINT}");

    // HTTP server setup
    let app = Router::new()
        .route("/work", get(get_work))
        .route("/inventory", post(send_inventory))
        .route("/mapping", get(get_mapping))
        .with_state(state);

    Server::bind(
        &SERVER_ENDPOINT
            .parse::<SocketAddr>()
            .expect("unable to parse socket address"),
    )
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn get_work(State(state): State<Arc<Mutex<MappingState>>>) -> Response {
    let mut state = state.lock().unwrap();
    if state.pending_work {
        info!("Work consumed");
        state.pending_work = false;
        ok!(CheckForWorkResponse { has_work: true })
    } else {
        ok!(CheckForWorkResponse { has_work: false })
    }
}

async fn send_inventory(
    State(_state): State<Arc<Mutex<MappingState>>>,
    Json(body): Json<SendInventoryRequest>,
) -> Response {
    info!("Got {} items in body", body.inventory.len());
    ok!(SendInventoryResponse {})
}

async fn get_mapping(State(state): State<Arc<Mutex<MappingState>>>) -> Response {
    let state = state.lock().unwrap();
    let response = GetMappingResponse {
        map: state
            .config
            .values
            .iter()
            .filter_map(|c| {
                if state.interactive {
                    Some((c.value.source.clone(), c.value.clone()))
                } else {
                    match c.end {
                        Some(end) if state.count >= c.begin && state.count < end => {
                            Some((c.value.source.clone(), c.value.clone()))
                        }
                        None if state.count >= c.begin => {
                            Some((c.value.source.clone(), c.value.clone()))
                        }
                        _ => None,
                    }
                }
            })
            .collect(),
    };

    ok!(response)
}

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
