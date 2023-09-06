// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod cartographer;
mod emitter;

use std::{collections::HashMap, env, str::FromStr, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use env_logger::Target;
use log::LevelFilter;
use tokio::sync::mpsc;

use cartographer::Cartographer;
use emitter::Emitter;
use freyja_common::signal_store::SignalStore;
use freyja_contracts::{
    cloud_adapter::CloudAdapter,
    digital_twin_adapter::DigitalTwinAdapter,
    mapping_client::MappingClient,
    provider_proxy::SignalValue,
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
};
use freyja_deps::*;
use provider_proxy_selector::provider_proxy_selector::ProviderProxySelector;

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

    let signal_store = Arc::new(SignalStore::new());
    let (tx_provider_proxy_selector_request, rx_provider_proxy_selector_request) =
        mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
    let provider_proxy_selector_request_sender =
        ProviderProxySelectorRequestSender::new(tx_provider_proxy_selector_request);

    // Setup cartographer
    let cartographer_poll_interval = Duration::from_secs(5);
    let cartographer = Cartographer::new(
        signal_store.clone(),
        MappingClientImpl::create_new().unwrap(),
        DigitalTwinAdapterImpl::create_new().unwrap(),
        provider_proxy_selector_request_sender.clone(),
        cartographer_poll_interval,
    );

    // Setup emitter
    let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
    let emitter = Emitter::new(
        signal_store.clone(),
        CloudAdapterImpl::create_new().unwrap(),
        provider_proxy_selector_request_sender.clone(),
        signal_values_queue.clone(),
    );

    let provider_proxy_selector = ProviderProxySelector::new();
    tokio::select! {
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        Err(e) = provider_proxy_selector.run(rx_provider_proxy_selector_request, signal_values_queue) => {  Err(e)? }
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
