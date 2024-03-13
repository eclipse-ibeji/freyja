// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::compile_remote_proto;

const CHARIOTT_SERVICE_DISCOVERY_INTERFACES_BASE_URI: &str =
    "https://raw.githubusercontent.com/eclipse-chariott/chariott/main/service_discovery/proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(
        format!("{CHARIOTT_SERVICE_DISCOVERY_INTERFACES_BASE_URI}/core/v1/service_registry.proto"),
        &[],
    )?;

    Ok(())
}
