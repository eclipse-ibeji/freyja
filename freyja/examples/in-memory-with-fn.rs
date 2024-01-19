// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_common::data_adapter::DataAdapterFactory;
use grpc_data_adapter::grpc_data_adapter_factory::GRPCDataAdapterFactory;
use http_mock_data_adapter::http_mock_data_adapter_factory::HttpMockDataAdapterFactory;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_data_adapter::in_memory_mock_data_adapter_factory::InMemoryMockDataAdapterFactory;
use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_mapping_adapter::in_memory_mock_mapping_adapter::InMemoryMockMappingAdapter;
use managed_subscribe_data_adapter::managed_subscribe_data_adapter_factory::ManagedSubscribeDataAdapterFactory;
use mqtt_data_adapter::mqtt_data_adapter_factory::MqttDataAdapterFactory;

// This example shows how you can use the freyja_main function manually rather than using the freyja_main! macro.
// This is useful when you need to do some additional work such as complex adapter setup or dependency resolution before invoking freyja_main.
// The following code is functionally equivalent to the expanded macro.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut factories: Vec<Box<dyn DataAdapterFactory + Send + Sync>> = Vec::new();
    factories.push(Box::new(
        GRPCDataAdapterFactory::create_new().expect("Could not create factory"),
    ));
    factories.push(Box::new(
        HttpMockDataAdapterFactory::create_new().expect("Could not create factory"),
    ));
    factories.push(Box::new(
        InMemoryMockDataAdapterFactory::create_new().expect("Could not create factory"),
    ));
    factories.push(Box::new(
        ManagedSubscribeDataAdapterFactory::create_new().expect("Could not create factory"),
    ));
    factories.push(Box::new(
        MqttDataAdapterFactory::create_new().expect("Could not create factory"),
    ));

    freyja::freyja_main::<
        InMemoryMockDigitalTwinAdapter,
        InMemoryMockCloudAdapter,
        InMemoryMockMappingAdapter,
    >(factories)
    .await
}
