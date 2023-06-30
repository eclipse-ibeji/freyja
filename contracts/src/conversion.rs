// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// A conversion from one value to another
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum Conversion {
    /// No conversion
    None,
    /// A conversion from x to y in the form y = mul * x + offset
    Linear { mul: f32, offset: f32 },
}

impl Conversion {
    /// Creates a LinearConversion for Celsius to Fahrenheit
    pub fn c_to_f() -> Self {
        Self::Linear {
            mul: 9.0 / 5.0,
            offset: 32.0,
        }
    }

    /// Creates a LinearConversion for Fahrenheit to Celsius
    pub fn f_to_c() -> Self {
        Self::c_to_f().inverse()
    }

    /// Inverts a Conversion
    ///
    /// Note that this may not yield the exact inverse due to floating-point errors
    ///
    /// # Example
    /// ```rust
    /// use dts_contracts::conversion::Conversion;
    /// let c2f = Conversion::c_to_f();
    /// assert!(42.0 == c2f.inverse().apply(c2f.apply(42.0)));
    /// ```
    pub fn inverse(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Linear { mul: m, offset: o } => Self::Linear {
                mul: 1.0 / m,
                offset: -o / m,
            },
        }
    }

    /// Converts the input
    ///
    /// # Arguments
    ///
    /// - `input`: the value to convert
    ///
    /// # Example
    /// ```rust
    /// use dts_contracts::conversion::Conversion;
    /// let c2f = Conversion::c_to_f();
    /// assert!(32.0 == c2f.apply(0.0));
    /// assert!(212.0 == c2f.apply(100.0));
    /// ```
    pub fn apply(&self, input: f32) -> f32 {
        match self {
            Self::None => input,
            Self::Linear { mul: m, offset: o } => input * m + o,
        }
    }
}

#[cfg(test)]
mod conversion_tests {
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
    fn can_invert_none() {
        let c = Conversion::None;
        assert_eq!(c.inverse(), Conversion::None);
    }

    #[test]
    fn can_invert_linear() {
        // These numbers were chosen to avoid floating-point errors since 1/8 = 0.125 can be represented with a terminating binary number
        let c1 = Conversion::Linear {
            mul: 8.0,
            offset: 8.0,
        };
        let c2 = Conversion::Linear {
            mul: 0.125,
            offset: -1.0,
        };
        assert_eq!(c1, c2.inverse());
        assert_eq!(c1.inverse(), c2);
    }

    #[test]
    fn can_apply_none() {
        let c = Conversion::None;

        // Try multiple values to make sure it works across various f32 and not just one input
        let vals = vec![
            0.0,
            -0.0,
            -1.23,
            42.0,
            77.7,
            f32::NAN,
            f32::INFINITY,
            std::f32::consts::PI,
        ];
        for v in vals.into_iter() {
            assert!(f32_close_enough(v, c.apply(v), 0.001));
        }
    }

    #[test]
    fn can_apply_linear() {
        let (mul, offset) = (0.125, 1.0);
        let c = Conversion::Linear { mul, offset };

        // Try multiple values to make sure it works across various f32 and not just one input
        let vals = vec![
            0.0,
            -0.0,
            -1.23,
            42.0,
            77.7,
            f32::NAN,
            f32::INFINITY,
            std::f32::consts::PI,
        ];
        for v in vals.into_iter() {
            let expected = v * mul + offset;
            assert!(f32_close_enough(expected, c.apply(v), 0.001));
        }
    }

    #[test]
    fn inverse_returns_correct_value() {
        let c = Conversion::Linear {
            mul: 8.0,
            offset: 8.0,
        };
        let i = c.inverse();

        // Try multiple values to make sure it works across various f32 and not just one input
        let vals = vec![
            0.0,
            -0.0,
            -1.23,
            42.0,
            77.7,
            f32::NAN,
            f32::INFINITY,
            std::f32::consts::PI,
        ];
        for v in vals.into_iter() {
            assert!(f32_close_enough(v, i.apply(c.apply(v)), 0.001));
        }
    }

    #[test]
    fn can_convert_temperatures() {
        let c2f = Conversion::c_to_f();
        let f2c = Conversion::f_to_c();

        let vals = vec![
            (-459.67, -273.15),
            (-40.0, -40.0),
            (32.0, 0.0),
            (77.0, 25.0),
            (212.0, 100.0),
        ];

        for (f, c) in vals.into_iter() {
            assert!(f32_close_enough(f, c2f.apply(c), 0.001));
            assert!(f32_close_enough(f2c.apply(f), c, 0.001));
        }
    }
}
