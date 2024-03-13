// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::compile_remote_proto;

const IBEJI_SAMPLE_INTERFACES_BASE_URI: &str =
    "https://raw.githubusercontent.com/eclipse-ibeji/ibeji/main/samples/interfaces";
const SAMPLE_GRPC_INTERFACE_PATH: &str = "sample_grpc/v1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(
        format!("{IBEJI_SAMPLE_INTERFACES_BASE_URI}/{SAMPLE_GRPC_INTERFACE_PATH}/digital_twin_consumer.proto"),
        &[])?;
    compile_remote_proto(
        format!("{IBEJI_SAMPLE_INTERFACES_BASE_URI}/{SAMPLE_GRPC_INTERFACE_PATH}/digital_twin_provider.proto"),
        &[])?;

    Ok(())
}
