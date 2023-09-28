// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod config_utils;
pub mod retry_utils;
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
