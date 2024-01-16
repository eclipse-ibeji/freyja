// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod cloud_adapter;
pub mod cmd_utils;
pub mod config_utils;
pub mod conversion;
pub mod digital_twin_adapter;
pub mod digital_twin_map_entry;
pub mod entity;
pub mod http_utils;
pub mod mapping_adapter;
pub mod message_utils;
pub mod provider_proxy;
pub mod provider_proxy_selector;
pub mod retry_utils;
pub mod signal;
pub mod signal_store;

/// Expands to `env!("OUT_DIR")`.
/// Since we cannot use a constant in the `env!` macro,
/// this is the next best option to avoid duplicating the `"OUT_DIR"` literal.
#[macro_export]
macro_rules! out_dir {
    () => {
        env!("OUT_DIR")
    };
}
