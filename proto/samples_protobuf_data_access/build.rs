// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use freyja_build_common::compile_remote_proto;
use proto_common::interface_url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_remote_proto(interface_url!(ibeji, SAMPLE_CONSUMER_INTERFACE), &[])?;
    compile_remote_proto(interface_url!(ibeji, SAMPLE_PROVIDER_INTERFACE), &[])?;

    Ok(())
}
