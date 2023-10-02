// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::env;

use freyja_build_common::copy_to_build_out_dir;

const RES_DIR_NAME: &str = "res";
const DEFAULT_CONFIG_FILE: &str = "mock_digital_twin_adapter_config.default.json";

fn main() {
    // Current directory of the build script is the package's root directory
    let config_path = env::current_dir()
        .unwrap()
        .join(RES_DIR_NAME)
        .join(DEFAULT_CONFIG_FILE);

    copy_to_build_out_dir(config_path, DEFAULT_CONFIG_FILE);
}
