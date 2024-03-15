// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::{compile_remote_proto, SERDE_DERIVE_ATTRIBUTE};
use proto_common::interface_url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(
        interface_url!(ibeji, INVEHICLE_DIGITAL_TWIN_INTERFACE),
        &[
            ("EndpointInfo", SERDE_DERIVE_ATTRIBUTE),
            ("EntityAccessInfo", SERDE_DERIVE_ATTRIBUTE),
        ],
    )?;
    compile_remote_proto(
        interface_url!(ibeji, MANAGED_SUBSCRIBE_INTERFACE),
        &[
            ("Constraint", SERDE_DERIVE_ATTRIBUTE),
            ("CallbackPayload", SERDE_DERIVE_ATTRIBUTE),
            ("SubscriptionInfo", SERDE_DERIVE_ATTRIBUTE),
        ],
    )?;

    Ok(())
}
