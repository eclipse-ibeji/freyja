// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod cartographer;
mod emitter;

use std::{
    collections::HashMap,
    env,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam::queue::SegQueue;
use env_logger::Target;
use log::LevelFilter;
use tokio::sync::mpsc;

use cartographer::Cartographer;
use freyja_contracts::provider_proxy_request::{
    ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
};
use freyja_contracts::{
    cloud_adapter::CloudAdapter, digital_twin_adapter::DigitalTwinAdapter,
    digital_twin_map_entry::DigitalTwinMapEntry, entity::*, mapping_client::MappingClient,
    provider_proxy::SignalValue,
};
use emitter::Emitter;
use provider_proxy_selector::provider_proxy_selector::ProviderProxySelector;

use freyja_deps::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: HashMap<String, String> = env::args()
        .skip(1)
        .map(|arg| {
            let mut split = arg.split('=');
            let key = split
                .next()
                .expect("Couldn't parse argument key")
                .to_owned();
            let val = split
                .next()
                .expect("Couldn't parse argument value")
                .to_owned();
            if split.next().is_some() {
                panic!("Too many pieces in argument");
            }

            (key, val)
        })
        .collect();

    // Setup logging
    let log_level = LevelFilter::from_str(args.get("log-level").unwrap_or(&"info".to_owned()))
        .expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    // Setup shared resources
    let map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Setup interfaces
    let dt_adapter = DigitalTwinAdapterImpl::create_new().unwrap();
    let mapping_client = MappingClientImpl::create_new().unwrap();
    let cloud_adapter: Box<dyn CloudAdapter + Send + Sync> =
        CloudAdapterImpl::create_new().unwrap();

    // Setup cartographer
    let cartographer_poll_interval = Duration::from_secs(5);
    let cartographer = Cartographer::new(map.clone(), mapping_client, cartographer_poll_interval);

    // Setup emitter
    let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
    let (tx_provider_proxy_selector_request, rx_provider_proxy_selector_request) =
        mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
    let provider_proxy_selector_request_sender = Arc::new(ProviderProxySelectorRequestSender::new(
        tx_provider_proxy_selector_request,
    ));

    let emitter = Emitter::new(
        map,
        cloud_adapter,
        entity_map.clone(),
        provider_proxy_selector_request_sender.clone(),
        signal_values_queue.clone(),
    );

    let provider_proxy_selector = ProviderProxySelector::new();
    tokio::select! {
        Err(e) = dt_adapter.run(entity_map, Duration::from_secs(5), provider_proxy_selector_request_sender) => { println!("[main] digital twin adapter terminated with error {e:?}"); Err(e)? },
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        Err(e) = provider_proxy_selector.run(rx_provider_proxy_selector_request, signal_values_queue) => {  Err(e)? }
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
