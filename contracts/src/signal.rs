// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::{entity::Entity, conversion::Conversion};

/// Conveys information about a signal, its current state, and how the data should be emitted
#[derive(Clone)]
pub struct Signal {
    pub id: String,
    pub value: Option<String>,
    pub source: Entity,
    pub target: Target,
    pub emission: Emission,
}

#[derive(Clone)]
pub struct Target {
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Emission {
    pub policy: EmissionPolicy,
    pub next_emission_ms: u64,
    pub last_emitted_value: Option<String>,
}

#[derive(Clone)]
pub struct EmissionPolicy {
    pub interval_ms: u64,
    pub emit_on_change: bool,
    pub conversion: Conversion,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            id: String::default(),
            value: None,
            source: Entity::default(),
            target: Target::default(),
            emission: Emission::default(),
        }
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            metadata: HashMap::default(),
        }
    }
}

impl Default for Emission {
    fn default() -> Self {
        Self {
            policy: EmissionPolicy::default(),
            // Note that setting the default next emission interval to 0 means that signals are emitted ASAP
            next_emission_ms: u64::default(),
            last_emitted_value: None,
        }
    }
}

impl Default for EmissionPolicy {
    fn default() -> Self {
        Self {
            interval_ms: u64::default(),
            emit_on_change: bool::default(),
            conversion: Conversion::None,
        }
    }
}