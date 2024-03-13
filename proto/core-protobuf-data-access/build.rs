// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::compile_remote_proto;

const IBEJI_INVEHICLE_INTERFACE_URI: &str = "https://raw.githubusercontent.com/eclipse-ibeji/ibeji/main/interfaces";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(
        format!("{IBEJI_INVEHICLE_INTERFACE_URI}/invehicle_digital_twin/v1/invehicle_digital_twin.proto"),
        &["EndpointInfo", "EntityAccessInfo"])?;
    compile_remote_proto(
        format!("{IBEJI_INVEHICLE_INTERFACE_URI}/module/managed_subscribe/v1/managed_subscribe.proto"),
        &["Constraint", "CallbackPayload", "SubscriptionInfo"])?;

    Ok(())
}
