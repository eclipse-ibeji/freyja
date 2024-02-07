// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
mod grpc_client_impl;
pub mod sample_grpc_data_adapter;
pub mod sample_grpc_data_adapter_factory;

const GRPC_PROTOCOL: &str = "grpc";
const GET_OPERATION: &str = "Get";
const SUBSCRIBE_OPERATION: &str = "Subscribe";
