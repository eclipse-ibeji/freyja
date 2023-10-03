// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this macro for convenience so users don't need to manually import the proc_macros crate
pub use proc_macros::freyja_main;
use tokio::sync::Mutex;

mod cartographer;
mod emitter;

use std::{collections::HashMap, env, str::FromStr, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use env_logger::Target;
use log::LevelFilter;

use cartographer::Cartographer;
use emitter::Emitter;
use freyja_common::signal_store::SignalStore;
use freyja_contracts::{
    cloud_adapter::CloudAdapter,
    digital_twin_adapter::DigitalTwinAdapter,
    mapping_client::MappingClient,
    provider_proxy::SignalValue,
};
use provider_proxy_selector::provider_proxy_selector_impl::ProviderProxySelectorImpl;

pub async fn freyja_main<
    TDigitalTwinAdapter: DigitalTwinAdapter,
    TCloudAdapter: CloudAdapter,
    TMappingClient: MappingClient,
>() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());
    let provider_proxy_selector = Arc::new(Mutex::new(ProviderProxySelectorImpl::new(signal_values_queue.clone())));

    // Setup cartographer
    let cartographer_poll_interval = Duration::from_secs(5);
    let cartographer = Cartographer::new(
        signal_store.clone(),
        TMappingClient::create_new().unwrap(),
        TDigitalTwinAdapter::create_new().unwrap(),
        provider_proxy_selector.clone(),
        cartographer_poll_interval,
    );

    // Setup emitter
    let emitter = Emitter::new(
        signal_store.clone(),
        TCloudAdapter::create_new().unwrap(),
        provider_proxy_selector.clone(),
        signal_values_queue.clone(),
    );

    tokio::select! {
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
