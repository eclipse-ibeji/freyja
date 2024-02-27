// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use file_service_discovery_adapter::file_service_discovery_adapter::FileServiceDiscoveryAdapter;
use freyja_common::{
    data_adapter::DataAdapterFactory, service_discovery_adapter::ServiceDiscoveryAdapter,
};
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_data_adapter::in_memory_mock_data_adapter_factory::InMemoryMockDataAdapterFactory;
use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_mapping_adapter::in_memory_mock_mapping_adapter::InMemoryMockMappingAdapter;
use managed_subscribe_data_adapter::managed_subscribe_data_adapter_factory::ManagedSubscribeDataAdapterFactory;
use mqtt_data_adapter::mqtt_data_adapter_factory::MqttDataAdapterFactory;
use sample_grpc_data_adapter::sample_grpc_data_adapter_factory::SampleGRPCDataAdapterFactory;

// This example shows how you can use the freyja_main function manually rather than using the freyja_main! macro.
// This is useful when you need to do some additional work such as complex adapter setup or dependency resolution before invoking freyja_main.
// The following code is functionally equivalent to the expanded macro.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_adapter_factories: Vec<Box<dyn DataAdapterFactory + Send + Sync>> = vec![
        Box::new(
            SampleGRPCDataAdapterFactory::create_new()
                .expect("Could not create SampleGRPCDataAdapterFactory"),
        ),
        Box::new(
            InMemoryMockDataAdapterFactory::create_new()
                .expect("Could not create InMemoryMockDataAdapterFactory"),
        ),
        Box::new(
            ManagedSubscribeDataAdapterFactory::create_new()
                .expect("Could not create ManagedSubscribeDataAdapterFactory"),
        ),
        Box::new(
            MqttDataAdapterFactory::create_new().expect("Could not create MqttDataAdapterFactory"),
        ),
    ];

    let service_discovery_adapters: Vec<Box<dyn ServiceDiscoveryAdapter + Send + Sync>> =
        vec![Box::new(
            FileServiceDiscoveryAdapter::create_new()
                .expect("Could not create FileServiceDiscoveryAdapter"),
        )];

    freyja::freyja_main::<
        InMemoryMockDigitalTwinAdapter,
        InMemoryMockCloudAdapter,
        InMemoryMockMappingAdapter,
    >(data_adapter_factories, service_discovery_adapters)
    .await
}
