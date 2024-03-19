// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod v1 {
    use std::collections::HashMap;

    use prost_types::{value::Kind, Timestamp, Value};
    use serde::ser::{Serialize, Serializer, SerializeStruct};
    use time::OffsetDateTime;

    tonic::include_proto!("cloud_connector");

    // Because the members of UpdateDigitalTwinRequest do not implement serialize
    // and are not owned by this project, implementing Serialize has to be done manually.
    impl Serialize for UpdateDigitalTwinRequest {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            let serialize_none = |state: &mut <S as Serializer>::SerializeStruct, key: &'static str| {
                state.serialize_field(key, &None::<()>)
            };

            let mut state = serializer.serialize_struct("UpdateDigitalTwinRequest", 3)?;

            // Serialize value
            const VALUE_FIELD: &str = "value";
            match self.value.as_ref() {
                None => serialize_none(&mut state, VALUE_FIELD)?,
                Some(v) => match v.kind.as_ref() {
                    None => serialize_none(&mut state, VALUE_FIELD)?,
                    Some(k) => match k {
                        Kind::NullValue(_) => serialize_none(&mut state, VALUE_FIELD)?,
                        Kind::NumberValue(n) => state.serialize_field(VALUE_FIELD, &n)?,
                        Kind::StringValue(s) => state.serialize_field(VALUE_FIELD, &s)?,
                        Kind::BoolValue(b) => state.serialize_field(VALUE_FIELD, &b)?,
                        _ => serialize_none(&mut state, VALUE_FIELD)?,
                    }
                }
            }

            // Serialize timestamp
            // Note that the nanos are discarded for simplicity
            const TIMESTAMP_FIELD: &str = "timestamp";
            match self.timestamp {
                None => serialize_none(&mut state, TIMESTAMP_FIELD)?,
                Some(Timestamp { seconds, .. }) => {
                    let timestamp = OffsetDateTime::from_unix_timestamp(seconds).unwrap();
                    state.serialize_field(TIMESTAMP_FIELD, &timestamp)?;
                },
            }

            // Serialize metadata
            state.serialize_field("metadata", &self.metadata)?;

            // End serialization
            state.end()
        }
    }

    /// Helper class to deal with the verbose contracts that tonic generates
    #[derive(Default)]
    pub struct UpdateDigitalTwinRequestBuilder {
        /// The request that's being built
        request: UpdateDigitalTwinRequest,
    }

    impl UpdateDigitalTwinRequestBuilder {
        /// Create a new `UpdateDigitalTwinRequestBuilder` with default values
        pub fn new() -> Self {
            Self::default()
        }

        /// Build the request
        pub fn build(self) -> UpdateDigitalTwinRequest {
            self.request
        }

        /// Set the request value to null
        pub fn null_value(mut self) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::NullValue(0)),
            });

            self
        }

        /// Set the request value to a number
        ///
        /// # Arguments
        /// - `n`: the value to set
        pub fn number_value(mut self, n: f64) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::NumberValue(n)),
            });

            self
        }

        /// Set the request value to a string
        ///
        /// # Arguments
        /// - `s`: the value to set
        pub fn string_value(mut self, s: String) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::StringValue(s)),
            });

            self
        }

        /// Set the request value to a boolean
        ///
        /// # Arguments
        /// - `b`: the value to set
        pub fn bool_value(mut self, b: bool) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::BoolValue(b)),
            });

            self
        }

        /// Set the request timestamp
        ///
        /// # Arguments
        /// - `timestamp`: the timestamp to set
        pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
            self.request.timestamp = Some(timestamp);
            self
        }

        /// Set the request timestamp to the current time
        pub fn timestamp_now(mut self) -> Self {
            let timestamp = OffsetDateTime::now_utc();
            self.request.timestamp = Some(
                Timestamp::date_time(
                    timestamp.year().into(),
                    timestamp.month().into(),
                    timestamp.day(),
                    timestamp.hour(),
                    timestamp.minute(),
                    timestamp.second(),
                )
                .unwrap(),
            );

            self
        }

        /// Set the request metadata. This overwrites any previously set metadata
        ///
        /// # Arguments
        /// - `metadata`: the metdata to set
        pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
            self.request.metadata = metadata;
            self
        }

        /// Add an entry to the request metadata
        ///
        /// # Arguments
        /// - `key`: the key of the metadata entry
        /// - `value`: the value of the metadata entry
        pub fn add_metadata(mut self, key: String, value: String) -> Self {
            self.request.metadata.insert(key, value);
            self
        }
    }
}

#[cfg(test)]
mod cloud_connector_tests {
    use serde_json::{json, Map, Value};

    use self::v1::UpdateDigitalTwinRequestBuilder;

    use super::*;

    use crate::v1::UpdateDigitalTwinRequest;

    fn validate_request_json(request: UpdateDigitalTwinRequest, key: &str, value: Value) {
        let serialize_result = serde_json::to_string(&request);
        assert!(serialize_result.is_ok());
        let serialized = serialize_result.unwrap();

        let deserialize_result = serde_json::from_str(&serialized);
        assert!(deserialize_result.is_ok());
        let json: Value = deserialize_result.unwrap();

        assert_eq!(json[key], value);
    }

    #[test]
    fn test_serialize_no_value() {
        let request = UpdateDigitalTwinRequestBuilder::new()
            .build();
        
        validate_request_json(request, "value", Value::Null);
    }

    #[test]
    fn test_serialize_null() {
        let request = UpdateDigitalTwinRequestBuilder::new()
            .null_value()
            .build();
        
        validate_request_json(request, "value", Value::Null);
    }

    #[test]
    fn test_serialize_bool() {
        let b = true;
        let request = UpdateDigitalTwinRequestBuilder::new()
            .bool_value(b)
            .build();
        
        validate_request_json(request, "value", Value::Bool(b));
    }

    #[test]
    fn test_serialize_number() {
        let n = 42.0;
        let request = UpdateDigitalTwinRequestBuilder::new()
            .number_value(n)
            .build();
        
        validate_request_json(request, "value", json!(n));
    }

    #[test]
    fn test_serialize_string() {
        let s = "foo";
        let request = UpdateDigitalTwinRequestBuilder::new()
            .string_value(s.into())
            .build();
        
        validate_request_json(request, "value", Value::String(s.into()));
    }

    #[test]
    fn test_serialize_no_timestamp() {
        let request = UpdateDigitalTwinRequestBuilder::new()
            .build();
        
        validate_request_json(request, "timestamp", Value::Null);
    }

    #[test]
    fn test_serialize_timestamp() {
        let request = UpdateDigitalTwinRequestBuilder::new()
            .timestamp_now()
            .build();

        let serialize_result = serde_json::to_string(&request);
        assert!(serialize_result.is_ok());
        let serialized = serialize_result.unwrap();

        let deserialize_result = serde_json::from_str(&serialized);
        assert!(deserialize_result.is_ok());
        let json: Value = deserialize_result.unwrap();

        assert_ne!(json["timestamp"], Value::Null);
    }

    #[test]
    fn test_serialize_no_metadata() {
        let request = UpdateDigitalTwinRequestBuilder::new()
            .build();
        
        validate_request_json(request, "metadata", Value::Object(Map::new()));
    }

    #[test]
    fn test_serialize_metadata() {
        let metadata = ("foo", "bar");
        let request = UpdateDigitalTwinRequestBuilder::new()
            .add_metadata(metadata.0.into(), metadata.1.into())
            .build();

        let mut map = Map::new();
        map.insert(metadata.0.into(), Value::String(metadata.1.into()));
        validate_request_json(request, "metadata", Value::Object(map));
    }
}