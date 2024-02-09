// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["../../interfaces/cloud_connector/v1/cloud_connector.proto"],
        &["../../interfaces/cloud_connector/v1/"],
    )?;

    Ok(())
}
