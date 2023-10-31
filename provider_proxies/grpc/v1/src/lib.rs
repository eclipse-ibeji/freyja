// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
mod grpc_client_impl;
pub mod grpc_provider_proxy;
pub mod grpc_provider_proxy_factory;

const GRPC_PROTOCOL: &str = "grpc";
const GET_OPERATION: &str = "Get";
const SUBSCRIBE_OPERATION: &str = "Subscribe";