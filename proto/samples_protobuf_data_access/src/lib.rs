// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod sample_grpc {
    pub mod v1 {
        pub mod digital_twin_consumer {
            tonic::include_proto!("digital_twin_consumer");
        }

        pub mod digital_twin_provider {
            tonic::include_proto!("digital_twin_provider");
        }
    }
}
