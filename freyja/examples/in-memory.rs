// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_mapping_client::in_memory_mock_mapping_client::InMemoryMockMappingClient;

proc_macros::freyja_main!{InMemoryMockDigitalTwinAdapter, InMemoryMockCloudAdapter, InMemoryMockMappingClient}