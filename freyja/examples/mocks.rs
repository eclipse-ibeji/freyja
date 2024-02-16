// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use file_service_discovery_adapter::file_service_discovery_adapter::FileServiceDiscoveryAdapter;
use http_mock_data_adapter::http_mock_data_adapter_factory::HttpMockDataAdapterFactory;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use mock_digital_twin_adapter::mock_digital_twin_adapter::MockDigitalTwinAdapter;
use mock_mapping_service_adapter::mock_mapping_service_adapter::MockMappingServiceAdapter;

freyja::freyja_main! {
    MockDigitalTwinAdapter,
    InMemoryMockCloudAdapter,
    MockMappingServiceAdapter,
    [HttpMockDataAdapterFactory],
    [FileServiceDiscoveryAdapter],
}