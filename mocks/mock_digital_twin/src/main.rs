// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
mod mock_digital_twin_impl;
mod mock_provider;

use std::{
    collections::{HashMap, HashSet},
    env, io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use core_protobuf_data_access::invehicle_digital_twin::v1::invehicle_digital_twin_server::InvehicleDigitalTwinServer;
use env_logger::Target;
use log::{debug, info, warn, LevelFilter};
use samples_protobuf_data_access::sample_grpc::v1::{
    digital_twin_consumer::{
        digital_twin_consumer_client::DigitalTwinConsumerClient, PublishRequest,
    },
    digital_twin_provider::digital_twin_provider_server::DigitalTwinProviderServer,
};
use tokio::sync::{mpsc, mpsc::UnboundedSender};
use tonic::{transport::Server, Request};

use crate::{
    config::{Config, EntityConfig},
    mock_digital_twin_impl::MockDigitalTwinImpl,
    mock_provider::MockProvider,
};
use freyja_build_common::config_file_stem;
use freyja_common::{
    cmd_utils::{get_log_level, parse_args},
    config_utils, out_dir,
};

/// Stores the state of active entities, subscribers, and relays responses
/// for getting/subscribing to an entity.
pub(crate) struct DigitalTwinAdapterState {
    /// An internal count that dictates which entites are enabled
    count: u8,

    /// The list of configured entities paired with the number of times that entity has published a value
    entities: Vec<(EntityConfig, u8)>,

    /// Maps entities to their subscribers
    subscriptions: HashMap<String, HashSet<String>>,

    /// A sender for manual publish requests
    response_channel_sender: UnboundedSender<(String, PublishRequest)>,

    /// Whether or not the application is in interactive mode
    interactive: bool,
}

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

    let interactive = args.get("interactive").is_some();

    let config: Config = config_utils::read_from_files(
        config_file_stem!(),
        config_utils::JSON_EXT,
        out_dir!(),
        |e| log::error!("{}", e),
        |e| log::error!("{}", e),
    )
    .unwrap();

    let (sender, mut receiver) = mpsc::unbounded_channel::<(String, PublishRequest)>();

    let state = Arc::new(Mutex::new(DigitalTwinAdapterState {
        count: 0,
        entities: config.entities.iter().map(|c| (c.clone(), 0)).collect(),
        subscriptions: config
            .entities
            .iter()
            .map(|c| (c.entity.id.clone(), HashSet::new()))
            .collect(),
        response_channel_sender: sender,
        interactive,
    }));

    let console_listener_state = state.clone();
    let subscribe_loop_state = state.clone();

    {
        let initial_state = state.lock().unwrap();
        info!(
            "Initial entity list: {:?}",
            get_active_entity_names(&initial_state)
        );
    }

    if interactive {
        // stdin setup
        thread::spawn(move || -> std::io::Result<usize> {
            let mut buffer = String::new();
            loop {
                io::stdin().read_line(&mut buffer)?;

                let mut state = console_listener_state.lock().unwrap();
                state.count += 1;
                info!(
                    "New count: {}. Active entities {:?}",
                    state.count,
                    get_active_entity_names(&state)
                );
            }
        });
    }

    // Get responder setup
    tokio::spawn(async move {
        loop {
            let message = receiver.recv().await;
            if message.is_none() {
                debug!("Channel is closed, aborting get responder...");
                break;
            }

            let request = message.unwrap();
            info!("Handling GET for request {request:?}...");
            let (consumer_uri, request) = request.clone();

            let mut client = match DigitalTwinConsumerClient::connect(consumer_uri).await {
                Ok(client) => client,
                Err(e) => {
                    log::error!("Error creating DigitalTwinConsumerClient: {e:?}");
                    continue;
                }
            };

            match client.publish(Request::new(request.clone())).await {
                Ok(_) => info!("Successfully sent value for request {request:?}"),
                Err(e) => log::error!("Failed to send value to {request:?}: {e}"),
            }
        }
    });

    // Subscriber publish setup
    tokio::spawn(async move {
        loop {
            debug!("Beginning subscribe loop...");

            let subscriptions = {
                let state = subscribe_loop_state.lock().unwrap();
                state.subscriptions.clone()
            };

            for (entity_id, subscribers) in subscriptions {
                // Get provider value
                let value = {
                    let mut state = subscribe_loop_state.lock().unwrap();
                    get_entity_value(&mut state, &entity_id).unwrap_or(String::new())
                };

                if value.is_empty() && !subscribers.is_empty() {
                    warn!("Entity {entity_id} has subscriptions but wasn't found!");
                    continue;
                }

                for subscriber in subscribers {
                    let request = PublishRequest {
                        entity_id: entity_id.clone(),
                        value: value.clone(),
                    };

                    let mut client = match DigitalTwinConsumerClient::connect(subscriber).await {
                        Ok(client) => client,
                        Err(e) => {
                            log::error!("Error creating DigitalTwinConsumerClient: {e:?}");
                            continue;
                        }
                    };

                    match client.publish(Request::new(request.clone())).await {
                        Ok(_) => info!("Successfully sent value for request {request:?}"),
                        Err(e) => log::error!("Failed to send value to {request:?}: {e}"),
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(3000)).await;
        }
    });

    // Server setup
    info!(
        "Mock Digital Twin Server starting at {}",
        config.digital_twin_server_authority
    );

    let addr = config
        .digital_twin_server_authority
        .parse()
        .expect("Unable to parse server address");

    let mock_digital_twin = MockDigitalTwinImpl {
        state: state.clone(),
    };

    let mock_provider = MockProvider {
        state: state.clone(),
    };

    Server::builder()
        .add_service(InvehicleDigitalTwinServer::new(mock_digital_twin))
        .add_service(DigitalTwinProviderServer::new(mock_provider))
        .serve(addr)
        .await
        .unwrap();
}

/// Checks if a value is within bounds
///
/// # Arguments
/// - `value`: the value to check within bounds
/// - `begin`: the start of a boundary
/// - `end`: the end of a boundary
/// - `interactive`: whether or not the application is running in interactive mode
fn within_bounds(value: u8, begin: u8, end: Option<u8>, interactive: bool) -> bool {
    !interactive
        || match end {
            Some(end) => value >= begin && value < end,
            None => value >= begin,
        }
}

/// Gets active entity names for this mock provider
///
/// # Arguments
/// - `state`: the state of the DigitalTwinAdapter which consists of active entities
fn get_active_entity_names(state: &DigitalTwinAdapterState) -> Vec<String> {
    state
        .entities
        .iter()
        .filter_map(|(config_item, _)| {
            if within_bounds(
                state.count,
                config_item.begin,
                config_item.end,
                state.interactive,
            ) {
                Some(
                    config_item
                        .entity
                        .name
                        .clone()
                        .unwrap_or_else(|| config_item.entity.id.clone()),
                )
            } else {
                None
            }
        })
        .collect()
}

/// Finds an entity using an entity's ID
///
/// # Arguments
/// - `state`: the state of the DigitalTwinAdapter which consists of active entities
/// - `id`: the entity's ID
fn find_entity<'a>(
    state: &'a DigitalTwinAdapterState,
    id: &'a String,
) -> Option<&'a (EntityConfig, u8)> {
    state
        .entities
        .iter()
        .filter(|(config_item, _)| {
            within_bounds(
                state.count,
                config_item.begin,
                config_item.end,
                state.interactive,
            )
        })
        .find(|(config_item, _)| config_item.entity.id == *id)
}

/// Gets an entity's value
///
/// # Arguments
/// - `state`: the state of the DigitalTwinAdapter which consists of active entities
/// - `id`: the entity's ID
fn get_entity_value(state: &mut DigitalTwinAdapterState, id: &str) -> Option<String> {
    let n = state.count;
    state
        .entities
        .iter_mut()
        .filter(|(config_item, _)| {
            within_bounds(n, config_item.begin, config_item.end, state.interactive)
        })
        .find(|(config_item, _)| config_item.entity.id == *id)
        .map(|p| {
            p.1 += 1;
            p.0.values.get_nth(p.1 - 1)
        })
}
