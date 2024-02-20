// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use file_service_discovery_adapter::file_service_discovery_adapter::FileServiceDiscoveryAdapter;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_data_adapter::in_memory_mock_data_adapter_factory::InMemoryMockDataAdapterFactory;
use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_mapping_adapter::in_memory_mock_mapping_adapter::InMemoryMockMappingAdapter;

freyja::freyja_main! {
    InMemoryMockDigitalTwinAdapter,
    InMemoryMockCloudAdapter,
    InMemoryMockMappingAdapter,
    [InMemoryMockDataAdapterFactory],
    [FileServiceDiscoveryAdapter],
}
