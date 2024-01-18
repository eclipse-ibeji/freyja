// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Config for the in-memory mock data adapter
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The frequency between updates to signal values in milliseconds
    pub signal_update_frequency_ms: u64,

    /// The entites to mock
    pub entities: Vec<EntityConfig>,
}

/// Configuration for a entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityConfig {
    /// The entity id
    pub entity_id: String,

    /// The config for the sensor values
    pub values: SensorValueConfig,
}

/// Configuration for a mock sensor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SensorValueConfig {
    /// A static value which does not change
    Static(f32),

    /// A value which changes according to a fixed interval with set start and end values
    Stepwise { start: f32, end: f32, delta: f32 },
}

impl SensorValueConfig {
    /// Gets the nth value of the sensor
    ///
    /// # Arguments
    ///
    /// - `n`: the seed for the value
    pub fn get_nth(&self, n: u8) -> f32 {
        match self {
            Self::Static(val) => *val,
            Self::Stepwise { start, end, delta } => match start + delta * n as f32 {
                val if val > *end && *delta > 0.0 => *end,
                val if val < *end && *delta < 0.0 => *end,
                val => val,
            },
        }
    }
}

#[cfg(test)]
mod config_item_tests {
    use super::*;

    /// Valdiates that abs(lhs - rhs) < epsilon, or that lhs and rhs are both f32::NAN or infinite with the same sign
    fn f32_approx_eq(lhs: f32, rhs: f32, epsilon: f32) -> bool {
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
        let c = SensorValueConfig::Static(val);

        for i in 0..10 {
            assert!(f32_approx_eq(val, c.get_nth(i), 0.001));
        }
    }

    #[test]
    fn get_nth_returns_stepwise_values() {
        let (start, end, delta) = (42.0, 44.4, 0.1);
        let c = SensorValueConfig::Stepwise { start, end, delta };

        let iters_to_end = ((end - start) / delta) as u8;

        // First check for values less than end
        for i in 0..iters_to_end {
            assert!(f32_approx_eq(start + delta * i as f32, c.get_nth(i), 0.001));
        }

        // Now validate behavior past end
        for i in iters_to_end..(iters_to_end + 10) {
            assert!(f32_approx_eq(end, c.get_nth(i), 0.001));
        }
    }
}
