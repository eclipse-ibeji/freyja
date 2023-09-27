// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;

use std::{
    io,
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
use freyja_common::config_utils;
use log::{info, LevelFilter};

use config::Config;
use freyja_contracts::mapping_client::{
    CheckForWorkResponse, GetMappingResponse, SendInventoryRequest, SendInventoryResponse,
};

const CONFIG_FILE: &str = "mock_mapping_config";
const CONFIG_EXT: &str = "json";

struct MappingState {
    count: u8,
    pending_work: bool,
    config: Config,
}

macro_rules! ok {
    () => {
        (axum::http::StatusCode::OK, axum::Json("")).into_response()
    };
    ($body:expr) => {
        (axum::http::StatusCode::OK, axum::Json($body)).into_response()
    };
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout)
        .init();

    let config = config_utils::read_from_files(
        CONFIG_FILE,
        CONFIG_EXT,
        env!("OUT_DIR"),
        |e| log::error!("{}", e),
        |e| log::error!("{}", e),
    )
    .unwrap();

    const SERVER_ENDPOINT: &str = "127.0.0.1:8888";

    let state = Arc::new(Mutex::new(MappingState {
        count: 0,
        pending_work: check_for_work(&config, 0),
        config: config.clone(),
    }));

    let state_clone = state.clone();

    {
        let initial_work = state.lock().unwrap().pending_work;
        info!("Initial work? {initial_work}");
    }

    // stdin setup
    thread::spawn(move || -> std::io::Result<usize> {
        let mut buffer = String::new();
        loop {
            io::stdin().read_line(&mut buffer)?;

            let mut state = state_clone.lock().unwrap();
            state.count += 1;
            let new_work = check_for_work(&config, state.count);

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

                info!("Work available for {work_available_state:?}");
            }
        }
    });

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
            .filter_map(|c| match c.end {
                Some(end) if state.count >= c.begin && state.count < end => {
                    Some((c.value.source.clone(), c.value.clone()))
                }
                None if state.count >= c.begin => Some((c.value.source.clone(), c.value.clone())),
                _ => None,
            })
            .collect(),
    };

    ok!(response)
}

fn check_for_work(config: &Config, n: u8) -> bool {
    config.values.iter().any(|c| match c.end {
        Some(end) => {
            if n == end {
                info!("End of {} for mapping", c.value.source);
            }
            n == end || n == c.begin
        }

        None => n == c.begin,
    })
}
