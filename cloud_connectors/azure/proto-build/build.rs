// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() {
    tonic_build::compile_protos("../proto/azure_cloud_connector.proto").unwrap();
}
