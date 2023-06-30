// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use dts_contracts::entity::Entity;

/// Configuration for a entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityConfig {
    // The entity
    pub entity: Entity,
}
