// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::{conversion::Conversion, entity::Entity};

/// Conveys information about a signal, its current state, and how the data should be emitted
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Signal {
    pub id: String,
    pub value: Option<String>,
    pub source: Entity,
    pub target: Target,
    pub emission: Emission,
}

/// A partial signal representation used in the signal store's sync API
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignalPatch {
    pub id: String,
    pub source: Entity,
    pub target: Target,
    pub emission_policy: EmissionPolicy,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Target {
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emission {
    pub policy: EmissionPolicy,
    // Note that the default for this value is 0, which the emitter will interpret as ready to emit ASAP
    pub next_emission_ms: u64,
    pub last_emitted_value: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmissionPolicy {
    pub interval_ms: u64,
    pub emit_only_if_changed: bool,
    pub conversion: Conversion,
}

impl From<Signal> for SignalPatch {
    fn from(value: Signal) -> Self {
        Self {
            id: value.id,
            source: value.source,
            target: value.target,
            emission_policy: value.emission.policy,
        }
    }
}