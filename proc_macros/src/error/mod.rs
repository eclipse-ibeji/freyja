// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod generate;
mod parse;
mod process;

use proc_macro2::TokenStream;

use generate::generate;
use parse::parse;
use process::process;

/// Implements the error macro
///
/// # Arguments:
///
/// - `ts`: The token stream input
pub fn error(ts: TokenStream) -> TokenStream {
    let args = parse(ts);
    let ir = process(args);
    generate(ir)
}
