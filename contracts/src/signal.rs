// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::{entity::Entity, conversion::Conversion};

/// Conveys information about a signal, its current state, and how the data should be emitted
#[derive(Clone, Default)]
pub struct Signal {
    pub id: String,
    pub value: Option<String>,
    pub source: Entity,
    pub target: Target,
    pub emission: Emission,
}

#[derive(Clone, Default)]
pub struct Target {
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Default)]
pub struct Emission {
    pub policy: EmissionPolicy,
    // Note that the default for this value is 0, which means that signals are emitted ASAP
    pub next_emission_ms: u64,
    pub last_emitted_value: Option<String>,
}

#[derive(Clone, Default)]
pub struct EmissionPolicy {
    pub interval_ms: u64,
    pub emit_only_if_changed: bool,
    pub conversion: Conversion,
}