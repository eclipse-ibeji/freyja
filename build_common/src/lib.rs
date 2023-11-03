// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

const OUT_DIR: &str = "OUT_DIR";
const CONFIG_FILE_STEM: &str = "CONFIG_FILE_STEM";
const RES_DIR: &str = "res";
const DEFAULT_CONFIG_EXT: &str = ".default.json";

/// Expands to `env!("CONFIG_FILE_STEM")`.
/// Since we cannot use a constant in the `env!` macro,
/// this is the next best option to avoid duplicating the `"CONFIG_FILE_STEM"` literal.
#[macro_export]
macro_rules! config_file_stem {
    () => {
        env!("CONFIG_FILE_STEM")
    };
}

/// Copies a file to the build output in `OUT_DIR`.
/// Includes a `cargo:rerun-if-changed` instruction for use in `build.rs` scripts.
/// This will likely panic outside of a build script and is not recommended for use in services.
///
/// # Arguments:
/// - `source_path`: The source file to copy
/// - `dest_filename`: The filename for the destination

/// Copies a config file to the build output in `OUT_DIR`.
/// Includes a `cargo:rerun-if-changed` instruction for use in `build.rs` scripts.
/// Also includes a `cargo:rustc-env` instruction to set the `CONFIG_FILE_STEM` enivornment variable,
/// which makes it possible to define the config stem in one place (the build script)
/// and share it with the source code via the `config_file_stem!` macro.
///
/// This will likely panic outside of a build script and is not recommended for use in services.
///
/// # Arguments
/// - `config_file_stem`: the config filename without an extension.
pub fn copy_config(config_file_stem: &str) {
    let default_config_filename = format!("{config_file_stem}{DEFAULT_CONFIG_EXT}");

    // Current directory of the build script is the package's root directory
    let config_path = env::current_dir()
        .unwrap()
        .join(RES_DIR)
        .join(&default_config_filename);

    let target_dir = env::var(OUT_DIR).unwrap();
    let destination = Path::new(&target_dir).join(default_config_filename);

    fs::copy(&config_path, destination).unwrap();

    // Only rerun the build script if the config changes
    println!("cargo:rerun-if-changed={}", config_path.to_str().unwrap());

    // Set the CONFIG_FILE_STEM environment variable for compilation
    println!("cargo:rustc-env={}={}", CONFIG_FILE_STEM, config_file_stem);
}
