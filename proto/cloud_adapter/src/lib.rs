// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub use prost_types;

pub mod v1 {
    tonic::include_proto!("cloud_adapter");
}