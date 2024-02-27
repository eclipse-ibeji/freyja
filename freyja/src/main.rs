// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use file_service_discovery_adapter::file_service_discovery_adapter::FileServiceDiscoveryAdapter;
use grpc_cloud_adapter::grpc_cloud_adapter::GRPCCloudAdapter;
use grpc_digital_twin_adapter::grpc_digital_twin_adapter::GRPCDigitalTwinAdapter;
use grpc_mapping_adapter::grpc_mapping_adapter::GRPCMappingAdapter;
use grpc_service_discovery_adapter::grpc_service_discovery_adapter::GRPCServiceDiscoveryAdapter;
use managed_subscribe_data_adapter::managed_subscribe_data_adapter_factory::ManagedSubscribeDataAdapterFactory;
use mqtt_data_adapter::mqtt_data_adapter_factory::MqttDataAdapterFactory;
use sample_grpc_data_adapter::sample_grpc_data_adapter_factory::SampleGRPCDataAdapterFactory;

freyja::freyja_main! {
    GRPCDigitalTwinAdapter,
    GRPCCloudAdapter,
    GRPCMappingAdapter,
    [SampleGRPCDataAdapterFactory, MqttDataAdapterFactory, ManagedSubscribeDataAdapterFactory],
    [GRPCServiceDiscoveryAdapter, FileServiceDiscoveryAdapter],
}
