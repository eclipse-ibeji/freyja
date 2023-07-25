// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    const OUTPUT_DIR: &str = "OUT_DIR";
    const CONFIG_FILE_IN_RESOURCE: &str = "res/config.json";
    const CONFIG_FILE: &str = "config.json";

    // Copy the Cloud Connector config to the target/debug output directory.
    let cloud_connector_config_path = env::current_dir().unwrap().join(CONFIG_FILE_IN_RESOURCE);
    let target_dir = env::var(OUTPUT_DIR).unwrap();
    let dest_path = Path::new(&target_dir).join(CONFIG_FILE);
    copy_file_to_path(cloud_connector_config_path, dest_path);

    // Copy the mqtt_config.sample.json template to target/debug
    const MQTT_CONFIG_SAMPLE_IN_RESOURCE: &str = "res/mqtt_config.sample.json";
    const MQTT_FILE_RELATIVE_TO_OUTPUT_DIR: &str = "../../../mqtt_config.json";
    let mqtt_config_sample = env::current_dir()
        .unwrap()
        .join(MQTT_CONFIG_SAMPLE_IN_RESOURCE);
    let dest_path = Path::new(&target_dir).join(MQTT_FILE_RELATIVE_TO_OUTPUT_DIR);
    copy_file_to_path(mqtt_config_sample, dest_path);
}

/// Copies a file to the destination path.
///
/// # Arguments
/// - `file_path`: the path to the file you wish to copy
/// - `dest_path`: the destination path.
fn copy_file_to_path(file_path: PathBuf, dest_path: PathBuf) {
    fs::copy(&file_path, dest_path).unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        file_path
            .to_str()
            .ok_or(format!("Check the file {file_path:?}"))
            .unwrap()
    );
}
