// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    env, fs,
    path::{Path, PathBuf},
};

const OUTPUT_DIR: &str = "OUT_DIR";
const MQTT_CONFIG_SAMPLE_IN_RESOURCE: &str = "res/mqtt_config.template.json";
const MQTT_CONFIG_FILE_RELATIVE_TO_OUTPUT_DIR: &str = "../../../mqtt_config.json";

fn main() {
    let target_dir = env::var(OUTPUT_DIR).unwrap();

    // Copy the mqtt_config.template.json template to target/debug
    let config_template = env::current_dir()
        .unwrap()
        .join(MQTT_CONFIG_SAMPLE_IN_RESOURCE);
    let dest_path = Path::new(&target_dir).join(MQTT_CONFIG_FILE_RELATIVE_TO_OUTPUT_DIR);
    copy(config_template, dest_path);
}

/// Copies a file to the destination path.
///
/// # Arguments
/// - `source_path`: the source path to a file.
/// - `dest_path`: the destination path.
fn copy(source_path: PathBuf, dest_path: PathBuf) {
    fs::copy(&source_path, dest_path).unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        source_path
            .to_str()
            .ok_or(format!("Check the file {source_path:?}"))
            .unwrap()
    );
}
