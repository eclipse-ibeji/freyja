// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this macro for convenience so users don't need to manually import the proc_macros crate
pub use proc_macros::freyja_main;

mod cartographer;
mod emitter;

use std::{env, sync::Arc, time::Duration};

use env_logger::Target;
use log::LevelFilter;
use tokio::sync::Mutex;

use cartographer::Cartographer;
use emitter::Emitter;
use freyja_common::{
    cloud_adapter::CloudAdapter,
    cmd_utils::{get_log_level, parse_args},
    digital_twin_adapter::DigitalTwinAdapter,
    mapping_adapter::MappingAdapter,
    data_adapter_selector::DataAdapterSelector,
    signal_store::SignalStore,
};
use data_adapter_selector::data_adapter_selector_impl::DataAdapterSelectorImpl;

use grpc_data_adapter::grpc_data_adapter_factory::GRPCDataAdapterFactory;
use http_mock_data_adapter::http_mock_data_adapter_factory::HttpMockDataAdapterFactory;
use in_memory_mock_data_adapter::in_memory_mock_data_adapter_factory::InMemoryMockDataAdapterFactory;
use managed_subscribe_data_adapter::managed_subscribe_data_adapter_factory::ManagedSubscribeDataAdapterFactory;
use mqtt_data_adapter::mqtt_data_adapter_factory::MqttDataAdapterFactory;

pub async fn freyja_main<
    TDigitalTwinAdapter: DigitalTwinAdapter,
    TCloudAdapter: CloudAdapter,
    TMappingAdapter: MappingAdapter,
>() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = parse_args(env::args()).expect("Failed to parse args");

    // Setup logging
    let log_level = get_log_level(&args, LevelFilter::Info).expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let signal_store = Arc::new(SignalStore::new());

    let mut data_adapter_selector = DataAdapterSelectorImpl::new(signal_store.clone());
    data_adapter_selector.register::<GRPCDataAdapterFactory>()?;
    data_adapter_selector.register::<HttpMockDataAdapterFactory>()?;
    data_adapter_selector.register::<InMemoryMockDataAdapterFactory>()?;
    data_adapter_selector.register::<ManagedSubscribeDataAdapterFactory>()?;
    data_adapter_selector.register::<MqttDataAdapterFactory>()?;

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

    tokio::select! {
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
