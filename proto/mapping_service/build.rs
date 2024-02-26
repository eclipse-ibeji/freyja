// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["../../interfaces/mapping_service/v1/mapping_service.proto"],
        &["../../interfaces/mapping_service/v1/"],
    )?;

    Ok(())
}
