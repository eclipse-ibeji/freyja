// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::conversion::Conversion;

/// Represents a mapping from the device digital twin to the cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalTwinMapEntry {
    /// The name of the source signal provider
    pub source: String,

    /// A map containing metadata for identifying a digital twin instance
    pub target: HashMap<String, String>,

    /// The interval at which the signal data should be sent
    pub interval_ms: u64,

    /// A conversion to apply to the signal data
    pub conversion: Conversion,

    /// Specifies whether to emit the signal when there's a change
    pub emit_on_change: bool,
}

impl Default for DigitalTwinMapEntry {
    fn default() -> Self {
        DigitalTwinMapEntry {
            source: String::new(),
            target: HashMap::new(),
            interval_ms: 0,
            conversion: Conversion::None,
            emit_on_change: false,
        }
    }
}
