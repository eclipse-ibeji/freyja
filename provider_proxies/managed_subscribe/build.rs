// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::copy_config;

const CONFIG_FILE_STEM: &str = "managed_subscribe_proxy_config";

fn main() {
    copy_config(CONFIG_FILE_STEM);
}
