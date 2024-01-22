// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export these items for convenience so users don't need to manually import them
pub use freyja_common;
pub use proc_macros::freyja_main;
pub use tokio;

mod cartographer;
mod data_adapter_selector_impl;
mod emitter;

use std::{env, sync::Arc, time::Duration};

use env_logger::Target;
use log::LevelFilter;
use crate::tokio::sync::Mutex;

use cartographer::Cartographer;
use emitter::Emitter;
use freyja_common::{
    cloud_adapter::CloudAdapter,
    cmd_utils::{get_log_level, parse_args},
    data_adapter::DataAdapterFactory,
    data_adapter_selector::DataAdapterSelector,
    digital_twin_adapter::DigitalTwinAdapter,
    mapping_adapter::MappingAdapter,
    signal_store::SignalStore,
};

use crate::data_adapter_selector_impl::DataAdapterSelectorImpl;

pub async fn freyja_main<
    TDigitalTwinAdapter: DigitalTwinAdapter,
    TCloudAdapter: CloudAdapter,
    TMappingAdapter: MappingAdapter,
>(
    factories: Vec<Box<dyn DataAdapterFactory + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = parse_args(env::args()).expect("Failed to parse args");

    // Setup logging
    let log_level = get_log_level(&args, LevelFilter::Info).expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let signal_store = Arc::new(SignalStore::new());

    let mut data_adapter_selector = DataAdapterSelectorImpl::new(signal_store.clone());
    for factory in factories.into_iter() {
        data_adapter_selector
            .register(factory)
            .expect("Could not register factory");
    }

    let data_adapter_selector = Arc::new(Mutex::new(data_adapter_selector));

    // Setup cartographer
    let cartographer_poll_interval = Duration::from_secs(5);
    let cartographer = Cartographer::new(
        signal_store.clone(),
        TMappingAdapter::create_new().unwrap(),
        TDigitalTwinAdapter::create_new().unwrap(),
        data_adapter_selector.clone(),
        cartographer_poll_interval,
    );

    // Setup emitter
    let emitter = Emitter::new(
        signal_store.clone(),
        TCloudAdapter::create_new().unwrap(),
        data_adapter_selector.clone(),
    );

    crate::tokio::select! {
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
