// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::{compile_remote_proto, SERDE_DERIVE_ATTRIBUTE};

const IBEJI_INTERFACES_BASE_URI: &str =
    "https://raw.githubusercontent.com/eclipse-ibeji/ibeji/main/interfaces";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(
        format!(
            "{IBEJI_INTERFACES_BASE_URI}/invehicle_digital_twin/v1/invehicle_digital_twin.proto"
        ),
        &[
            ("EndpointInfo", SERDE_DERIVE_ATTRIBUTE),
            ("EntityAccessInfo", SERDE_DERIVE_ATTRIBUTE),
        ],
    )?;
    compile_remote_proto(
        format!("{IBEJI_INTERFACES_BASE_URI}/module/managed_subscribe/v1/managed_subscribe.proto"),
        &[
            ("Constraint", SERDE_DERIVE_ATTRIBUTE),
            ("CallbackPayload", SERDE_DERIVE_ATTRIBUTE),
            ("SubscriptionInfo", SERDE_DERIVE_ATTRIBUTE),
        ],
    )?;

    Ok(())
}
