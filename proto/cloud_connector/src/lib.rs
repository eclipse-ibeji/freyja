// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod v1 {
    use std::collections::HashMap;

    use prost_types::{value::Kind, Timestamp, Value};
    use time::OffsetDateTime;

    tonic::include_proto!("cloud_connector");

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
