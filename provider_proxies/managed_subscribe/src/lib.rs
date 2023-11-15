// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
pub mod managed_subscribe_provider_proxy;
pub mod managed_subscribe_provider_proxy_factory;

const GRPC_PROTOCOL: &str = "grpc";
const MQTT_PROTOCOL: &str = "mqtt";
const MANAGED_SUBSCRIBE_OPERATION: &str = "ManagedSubscribe";
const SUBSCRIBE_OPERATION: &str = "Subscribe";
