// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use mock_digital_twin_adapter::mock_digital_twin_adapter::MockDigitalTwinAdapter;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use mock_mapping_service_client::mock_mapping_service_client::MockMappingServiceClient;

freyja::freyja_main!{MockDigitalTwinAdapter, InMemoryMockCloudAdapter, MockMappingServiceClient}