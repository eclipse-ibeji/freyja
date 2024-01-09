// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::{conversion::Conversion, entity::Entity};

/// Conveys information about a signal, its current state, and how the data should be emitted
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Signal {
    /// The signal id. In most cases, this should be the same as the source id
    pub id: String,
    /// The signal's current value, if it's been set
    pub value: Option<String>,
    /// The signal's source entity information
    pub source: Entity,
    /// The signal's target mapping information
    pub target: Target,
    /// The signal's emission metadata
    pub emission: Emission,
}

/// A partial signal representation used in the signal store's sync API
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignalPatch {
    /// The signal id. In most cases, this should be the same as the source id
    pub id: String,
    /// The signal's source entity information
    pub source: Entity,
    /// The signal's target mapping information
    pub target: Target,
    /// The signal's emission metadata
    pub emission_policy: EmissionPolicy,
}

/// A signal's target mapping information
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Target {
    /// Metadata that will be passed to the cloud adapter to perform the mapping
    pub metadata: HashMap<String, String>,
}

/// Metadata about a signal's emission
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emission {
    /// The emission policy
    pub policy: EmissionPolicy,
    /// The time until the signal's next emission
    /// Note that the default for this value is 0, which the emitter will interpret as ready to emit ASAP
    pub next_emission_ms: u64,
    /// The last emitted value
    pub last_emitted_value: Option<String>,
}

/// A signal's emission policy
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmissionPolicy {
    /// The interal at which the signal should be emitted
    pub interval_ms: u64,
    /// Indicates whether the signal data should only be emitted if the value has changed
    pub emit_only_if_changed: bool,
    /// A conversion to apply to the signal before emission
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
