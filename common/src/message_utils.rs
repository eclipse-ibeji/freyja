// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use log::{debug, warn};

const METADATA_KEY: &str = "$metadata";

/// Parses a value published by a provider.
/// The current implementation is a workaround for the current Ibeji sample provider implementation,
/// which uses a non-consistent contract as follows:
///
/// ```ignore
/// {
///     "{propertyName}": "value",
///     "$metadata": {...}
/// }
/// ```
///
/// Note that `{propertyName}` is replaced by the name of the property that the provider published.
/// This function will extract the value from the first property satisfying the following conditions:
/// - The property is not named `$metadata`
/// - The property value is a non-null primitive JSON type (string, bool, or number)
///
/// If any part of parsing fails, a warning is logged and the original value is returned.
///
/// # Arguments
/// - `value`: the value to attempt to parse
pub fn parse_value(value: String) -> String {
    match serde_json::from_str::<serde_json::Value>(&value) {
        Ok(v) => {
            let property_map = match v.as_object() {
                Some(o) => o,
                None => {
                    debug!("Could not parse value as JSON object");
                    return value;
                }
            };

            for property in property_map.iter() {
                if property.0 == METADATA_KEY {
                    continue;
                }

                let selected_value = match property.1 {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => continue,
                };

                let metadata_descriptor = if property_map.contains_key(&METADATA_KEY.to_string()) {
                    "has"
                } else {
                    "does not have"
                };

                debug!(
                    "Value contained {} properties and {metadata_descriptor} a {METADATA_KEY} property. Selecting property with key {} as the signal value",
                    property_map.len(),
                    property.0
                );

                return selected_value;
            }

            warn!("Could not find a property that was parseable as a value");
            value
        }
        Err(e) => {
            warn!("Failed to parse value |{value}|: {e}");
            value
        }
    }
}

#[cfg(test)]
mod message_utils_tests {
    use super::*;

    #[test]
    fn parse_value_returns_input_when_parse_fails() {
        let input = r#"invalid json"#;
        let result = parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_is_plain_string() {
        let input = r#""value""#;
        let result = parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_has_zero_properties() {
        let input = r#"{}"#;
        let result = parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_has_no_valid_properties() {
        let input = format!(r#"{{"{METADATA_KEY}": "foo"}}"#);
        let result = parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_property_value_is_not_string() {
        let input = r#"{"property": ["value"]}"#;
        let result = parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_correct_value_for_strings() {
        let expected_value = "value";
        let input = format!(r#"{{"property": "{expected_value}", "{METADATA_KEY}": "foo"}}"#);
        let result = parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_returns_correct_value_for_bools() {
        let expected_value = "true";
        let input = format!(r#"{{"property": {expected_value}, "{METADATA_KEY}": "foo"}}"#);
        let result = parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_returns_correct_value_for_numbers() {
        let expected_value = "123.456";
        let input = format!(r#"{{"property": {expected_value}, "{METADATA_KEY}": "foo"}}"#);
        let result = parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_skips_metadata_property() {
        let expected_value = "value";
        let input = format!(r#"{{"{METADATA_KEY}": "foo", "property": "{expected_value}"}}"#);
        let result = parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_skips_non_primitive_properties() {
        let expected_value = "value";
        let input = format!(
            r#"{{"foo": ["bar"], "property": "{expected_value}", "{METADATA_KEY}": "foo"}}"#
        );
        let result = parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }
}
