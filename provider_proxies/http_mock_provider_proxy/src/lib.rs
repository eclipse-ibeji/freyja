// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
pub mod http_mock_provider_proxy;
pub mod http_mock_provider_proxy_factory;

const HTTP_PROTOCOL: &str = "http";
const GET_OPERATION: &str = "Get";
const SUBSCRIBE_OPERATION: &str = "Subscribe";