// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::copy_config;

const CONFIG_FILE_STEM: &str = "mock_mapping_client_config";

fn main() {
    copy_config(CONFIG_FILE_STEM);
}
