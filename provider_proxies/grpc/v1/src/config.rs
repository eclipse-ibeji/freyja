// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// The GRPC provider proxy config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The hosting address
    pub consumer_address: String,

    /// The advertised address given to providers as the callback address
    /// If not specified, the `consumer_address` will be used
    pub advertised_consumer_address: Option<String>,
}

impl Config {
    /// Gets the advertised address.
    /// Returns the value of `self.advertised_consumer_address` if it's not `None`,
    /// otherwise returns `self.consumer_address`.
    pub fn get_advertised_address(&self) -> &String {
        self.advertised_consumer_address.as_ref().unwrap_or(&self.consumer_address)
    }
}