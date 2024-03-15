// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// This module contains constants related to remote protobuf files as well as helpers for referencing them.
// This also helps manage interface versions from a central location, which is particularly helpful for Ibeji since
// the interfaces are referenced in two different crates.

pub const GITHUB_BASE_URL: &str = "https://raw.githubusercontent.com";

pub mod ibeji {
    pub const REPO_NAME: &str = "eclipse-ibeji/ibeji";
    pub const VERSION: &str = "0.1.1";

    pub mod interfaces {
        pub const INVEHICLE_DIGITAL_TWIN_INTERFACE: &str =
            "interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto";
        pub const MANAGED_SUBSCRIBE_INTERFACE: &str =
            "interfaces/module/managed_subscribe/v1/managed_subscribe.proto";
        pub const SAMPLE_CONSUMER_INTERFACE: &str =
            "samples/interfaces/sample_grpc/v1/digital_twin_consumer.proto";
        pub const SAMPLE_PROVIDER_INTERFACE: &str =
            "samples/interfaces/sample_grpc/v1/digital_twin_provider.proto";
    }
}

pub mod chariott {
    pub const REPO_NAME: &str = "eclipse-chariott/chariott";
    pub const VERSION: &str = "0.2.1";

    pub mod interfaces {
        pub const SERVICE_REGISTRY_INTERFACE: &str =
            "service_discovery/proto/core/v1/service_registry.proto";
    }
}

/// Macro that simplifies the construction of URLs for remote protobuf interfaces.
///
/// # Arguments
/// - `service`: the service name. Corresponds to one of the submodules of `proto_common`.
/// - `interface`: the interface name. Corresponds to one of the constants in the `interfaces` sub-module of the `service` module.
#[macro_export]
macro_rules! interface_url {
    ($service:ident, $interface:ident) => {
        format!(
            "{}/{}/{}/{}",
            proto_common::GITHUB_BASE_URL,
            proto_common::$service::REPO_NAME,
            proto_common::$service::VERSION,
            proto_common::$service::interfaces::$interface
        )
    };
}
