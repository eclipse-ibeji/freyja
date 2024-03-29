// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
pub mod in_memory_mock_data_adapter;
pub mod in_memory_mock_data_adapter_factory;

const IN_MEMORY_PROTOCOL: &str = "in-memory";
const GET_OPERATION: &str = "Get";
const SUBSCRIBE_OPERATION: &str = "Subscribe";
