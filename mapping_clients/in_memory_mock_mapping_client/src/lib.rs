// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod config;
pub mod in_memory_mock_mapping_client;

pub use crate::in_memory_mock_mapping_client::InMemoryMockMappingClient as MappingClientImpl;
