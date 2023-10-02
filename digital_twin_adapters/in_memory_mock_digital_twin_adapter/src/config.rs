// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use freyja_contracts::entity::Entity;

/// The in-memory mock digital twin's config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The set of config values
    pub values: Vec<EntityConfig>,
}

/// Configuration for a entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityConfig {
    // The entity
    pub entity: Entity,
}
