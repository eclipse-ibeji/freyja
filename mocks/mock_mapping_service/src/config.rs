// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use freyja_common::digital_twin_map_entry::DigitalTwinMapEntry;

/// The mock mapping service's config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The mapping server authority
    pub mapping_server_authority: String,

    /// The set of config values
    pub values: Vec<ConfigItem>,
}

/// A config item for the mock mapping service
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    /// Start emitting the value after this many calls to the service
    pub begin: u8,

    /// Stop emitting the value after this many calls to the service (or don't stop emitting if None)
    pub end: Option<u8>,

    /// The mapping to apply
    pub value: DigitalTwinMapEntry,
}
