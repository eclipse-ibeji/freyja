// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use file_service_discovery_adapter::file_service_discovery_adapter::FileServiceDiscoveryAdapter;
use grpc_digital_twin_adapter::grpc_digital_twin_adapter::GRPCDigitalTwinAdapter;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_mapping_adapter::in_memory_mock_mapping_adapter::InMemoryMockMappingAdapter;
use managed_subscribe_data_adapter::managed_subscribe_data_adapter_factory::ManagedSubscribeDataAdapterFactory;
use mqtt_data_adapter::mqtt_data_adapter_factory::MqttDataAdapterFactory;
use sample_grpc_data_adapter::sample_grpc_data_adapter_factory::SampleGRPCDataAdapterFactory;

freyja::freyja_main! {
    GRPCDigitalTwinAdapter,
    InMemoryMockCloudAdapter,
    InMemoryMockMappingAdapter,
    [SampleGRPCDataAdapterFactory, MqttDataAdapterFactory, ManagedSubscribeDataAdapterFactory],
    [FileServiceDiscoveryAdapter],
}
