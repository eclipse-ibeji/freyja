// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod invehicle_digital_twin {
    pub mod v1 {
        tonic::include_proto!("invehicle_digital_twin");
    }
}

pub mod module {
    pub mod managed_subscribe {
        pub mod v1 {
            tonic::include_proto!("managed_subscribe");
        }
    }
}
