// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter;
use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;
use in_memory_mock_mapping_client::in_memory_mock_mapping_client::InMemoryMockMappingClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This example shows how you can use the freyja_main function manually rather than using the freyja_main! macro.
    // This is useful when you need to do some additional work such as complex adapter setup or dependency resolution before invoking freyja_main.
    freyja::freyja_main::<InMemoryMockDigitalTwinAdapter, InMemoryMockCloudAdapter, InMemoryMockMappingClient>().await
}