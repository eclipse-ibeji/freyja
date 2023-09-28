// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

const OUT_DIR: &str = "OUT_DIR";

/// Copies a file to the build output in `OUT_DIR`.
/// Includes a `cargo:rerun-if-changed` instruction for use in `build.rs` scripts.
/// This will likely panic outside of a build script and is not recommended for use in services.
///
/// # Arguments:
/// - `source_path`: The source file to copy
/// - `dest_filename`: The filename for the destination
pub fn copy_to_build_out_dir<P: AsRef<Path>>(source_path: P, dest_filename: &str) {
    let target_dir = env::var(OUT_DIR).unwrap();
    let destination = Path::new(&target_dir).join(dest_filename);

    fs::copy(&source_path, destination).unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        source_path.as_ref().to_str().unwrap()
    );
}
