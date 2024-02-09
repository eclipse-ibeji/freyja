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
        pub fn new() -> Self {
            Self::default()
        }

        pub fn build(self) -> UpdateDigitalTwinRequest {
            self.request
        }

        pub fn null_value(mut self) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::NullValue(0))
            });

            self
        }

        pub fn number_value(mut self, n: f64) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::NumberValue(n))
            });

            self
        }

        pub fn string_value(mut self, s: String) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::StringValue(s))
            });

            self
        }

        pub fn bool_value(mut self, b: bool) -> Self {
            self.request.value = Some(Value {
                kind: Some(Kind::BoolValue(b))
            });

            self
        }

        pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
            self.request.timestamp = Some(timestamp);
            self
        }

        pub fn timestamp_now(mut self) -> Self {
            let timestamp = OffsetDateTime::now_utc();
            self.request.timestamp = Some(Timestamp::date_time(
                timestamp.year().into(),
                timestamp.month().into(),
                timestamp.day(),
                timestamp.hour(),
                timestamp.minute(),
                timestamp.second(),
                ).unwrap()
            );

            self
        }

        pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
            self.request.metadata = metadata;
            self
        }

        pub fn add_metadata(mut self, key: String, value: String) -> Self {
            self.request.metadata.insert(key, value);
            self
        }
    }
}