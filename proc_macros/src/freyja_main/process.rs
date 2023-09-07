// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use super::parse::FreyjaMainArgs;

/// Process data for the freyja_main! macro
/// No additional processing is currently necessary for ths macro, so the args are just being repackaged.
///
/// # Arguments
///
/// - `args`: the input arguments
pub(crate) fn process(args: FreyjaMainArgs) -> FreyjaMainOutput {
    FreyjaMainOutput { args }
}

/// An intermediate representation of the use_env output
#[derive(Debug)]
pub(crate) struct FreyjaMainOutput {
    pub args: FreyjaMainArgs,
}
