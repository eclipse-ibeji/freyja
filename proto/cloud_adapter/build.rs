// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["../../interfaces/cloud_adapter/v1/cloud_adapter.proto"],
            &["../../interfaces/cloud_adapter/v1/"],
        )?;

    Ok(())
}