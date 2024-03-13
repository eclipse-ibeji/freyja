// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod service_registry {
    pub mod v1 {
        tonic::include_proto!("service_registry");
    }
}
