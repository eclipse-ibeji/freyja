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
    cloud_adapter::CloudAdapter, digital_twin_adapter::DigitalTwinAdapter,
    mapping_client::MappingClient, provider_proxy_selector::ProviderProxySelector,
};
use freyja_common::{
    cmd_utils::{get_log_level, parse_args},
    signal_store::SignalStore,
};
use provider_proxy_selector::provider_proxy_selector_impl::ProviderProxySelectorImpl;

use grpc_provider_proxy_v1::grpc_provider_proxy_factory::GRPCProviderProxyFactory;
use http_mock_provider_proxy::http_mock_provider_proxy_factory::HttpMockProviderProxyFactory;
use in_memory_mock_provider_proxy::in_memory_mock_provider_proxy_factory::InMemoryMockProviderProxyFactory;
use managed_subscribe_provider_proxy::managed_subscribe_provider_proxy_factory::ManagedSubscribeProviderProxyFactory;
use mqtt_provider_proxy::mqtt_provider_proxy_factory::MqttProviderProxyFactory;

pub async fn freyja_main<
    TDigitalTwinAdapter: DigitalTwinAdapter,
    TCloudAdapter: CloudAdapter,
    TMappingClient: MappingClient,
>() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = parse_args(env::args()).expect("Failed to parse args");

    // Setup logging
    let log_level = get_log_level(&args, LevelFilter::Info).expect("Could not parse log level");
    env_logger::Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let signal_store = Arc::new(SignalStore::new());

    let mut provider_proxy_selector = ProviderProxySelectorImpl::new(signal_store.clone());
    provider_proxy_selector.register::<GRPCProviderProxyFactory>()?;
    provider_proxy_selector.register::<HttpMockProviderProxyFactory>()?;
    provider_proxy_selector.register::<InMemoryMockProviderProxyFactory>()?;
    provider_proxy_selector.register::<ManagedSubscribeProviderProxyFactory>()?;
    provider_proxy_selector.register::<MqttProviderProxyFactory>()?;

    let provider_proxy_selector = Arc::new(Mutex::new(provider_proxy_selector));

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
    );

    tokio::select! {
        Err(e) = cartographer.run() => { println!("[main] cartographer terminated with error {e:?}"); Err(e) },
        Err(e) = emitter.run() => { println!("[main] emitter terminated with error {e:?}"); Err(e) },
        else => { println!("[main] all operations terminated successfully"); Ok(()) },
    }
}
