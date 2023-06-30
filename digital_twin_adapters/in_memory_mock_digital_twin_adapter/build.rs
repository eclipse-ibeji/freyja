// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

fn main() {
    // Current directory of the build script is the package's root directory
    let config_path = env::current_dir().unwrap().join("res").join("config.json");

    let target_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&target_dir).join("config.json");

    fs::copy(&config_path, dest_path).unwrap();

    println!("cargo:rerun-if-changed={}", config_path.to_str().unwrap());
}
