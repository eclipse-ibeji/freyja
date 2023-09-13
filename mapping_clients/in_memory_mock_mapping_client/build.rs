// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

const OUT_DIR: &str = "OUT_DIR";
const RES_DIR_NAME: &str = "res";
const DEFAULT_CONFIG_FILE: &str = "mapping_client_config.default.json";

fn main() {
    // Current directory of the build script is the package's root directory
    let config_path = env::current_dir()
        .unwrap()
        .join(RES_DIR_NAME)
        .join(DEFAULT_CONFIG_FILE);

    let target_dir = env::var(OUT_DIR).unwrap();
    let dest_path = Path::new(&target_dir).join(DEFAULT_CONFIG_FILE);

    fs::copy(&config_path, dest_path).unwrap();

    println!("cargo:rerun-if-changed={}", config_path.to_str().unwrap());
}
