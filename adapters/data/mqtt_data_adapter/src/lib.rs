// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
pub mod mqtt_data_adapter;
pub mod mqtt_data_adapter_factory;

const MQTT_PROTOCOL: &str = "mqtt";
const SUBSCRIBE_OPERATION: &str = "Subscribe";
