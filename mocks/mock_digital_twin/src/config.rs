// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use freyja_common::entity::Entity;

/// Config for the mock digital twin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The digital twin server authority for hosting a gRPC server
    pub digital_twin_server_authority: String,

    /// The list of entities
    pub entities: Vec<EntityConfig>,
}

/// A config entry for the MockDigitalTwinAdapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityConfig {
    /// Start emitting the value after this many calls to the client
    pub begin: u8,

    /// Stop emitting the value after this many calls to the client (or don't stop emitting if None)
    pub end: Option<u8>,

    /// The entity to provide
    pub entity: Entity,

    /// The config for the sensor values
    pub values: SensorValueConfig,
}

/// Configuration for a mock sensor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SensorValueConfig {
    /// A static value which does not change
    Static(String),

    /// A value which changes according to a fixed interval with set start and end values
    Stepwise { start: f32, end: f32, delta: f32 },
}

impl SensorValueConfig {
    /// Gets the nth value of the sensor
    ///
    /// # Arguments
    /// - `n`: the seed for the value
    pub fn get_nth(&self, n: u8) -> String {
        match self {
            Self::Static(val) => val.clone(),
            Self::Stepwise { start, end, delta } => match start + delta * n as f32 {
                val if val > *end && *delta > 0.0 => end.to_string(),
                val if val < *end && *delta < 0.0 => end.to_string(),
                val => val.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod config_item_tests {
    use super::*;

    /// Valdiates that abs(lhs - rhs) < epsilon, or that lhs and rhs are both f32::NAN or infinite with the same sign
    fn f32_close_enough(lhs: f32, rhs: f32, epsilon: f32) -> bool {
        f32::abs(lhs - rhs) < epsilon
            || lhs.is_nan() && rhs.is_nan()
            || lhs.is_infinite()
                && rhs.is_infinite()
                && lhs.is_sign_positive()
                && rhs.is_sign_positive()
    }

    #[test]
    fn get_nth_returns_static_value() {
        let val = 42.0;
        let c = SensorValueConfig::Static(val.to_string());

        for i in 0..10 {
            assert!(f32_close_enough(val, c.get_nth(i).parse().unwrap(), 0.001));
        }
    }

    #[test]
    fn get_nth_returns_stepwise_values() {
        let (start, end, delta) = (42.0, 44.4, 0.1);
        let c = SensorValueConfig::Stepwise { start, end, delta };

        let iters_to_end = ((end - start) / delta) as u8;

        // First check for values less than end
        for i in 0..iters_to_end {
            assert!(f32_close_enough(
                start + delta * i as f32,
                c.get_nth(i).parse().unwrap(),
                0.001
            ));
        }

        // Now validate behavior past end
        for i in iters_to_end..(iters_to_end + 10) {
            assert!(f32_close_enough(end, c.get_nth(i).parse().unwrap(), 0.001));
        }
    }
}
