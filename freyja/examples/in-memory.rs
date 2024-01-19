// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_mapping_adapter::in_memory_mock_mapping_adapter::InMemoryMockMappingAdapter;
use grpc_data_adapter::grpc_data_adapter_factory::GRPCDataAdapterFactory;

freyja::freyja_main! {
    InMemoryMockDigitalTwinAdapter,
    InMemoryMockCloudAdapter,
    InMemoryMockMappingAdapter,
    [GRPCDataAdapterFactory],
}
