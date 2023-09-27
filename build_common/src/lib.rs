// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

/// Copies a file to the build output in `OUT_DIR`.
/// Includes a `cargo:rerun-if-changed` instruction for use in build.rs scripts.
///
/// # Arguments:
/// - `source`: The source file to copy
/// - `dest_filename`: The filename for the destination
pub fn copy_to_build_out_dir<P: AsRef<Path>>(source: P, dest_filename: &str) {
    const OUT_DIR: &str = "OUT_DIR";

    let target_dir = env::var(OUT_DIR).unwrap();
    let destination = Path::new(&target_dir).join(dest_filename);

    fs::copy(&source, destination).unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        source.as_ref().to_str().unwrap()
    );
}
