// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crossbeam::queue::SegQueue;
use log::{debug, warn};
use serde_json::Value;
use tonic::{Request, Response, Status};

use freyja_contracts::provider_proxy::SignalValue;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_consumer::{
    digital_twin_consumer_server::DigitalTwinConsumer, PublishRequest, PublishResponse,
    RespondRequest, RespondResponse,
};

const METADATA_KEY: &str = "$metadata";

/// Struct which implements the DigitalTwinConsumer trait for GRPC clients
#[derive(Debug, Default)]
pub struct GRPCClientImpl {
    /// The queue on which incoming signal values should be published
    pub signal_values_queue: Arc<SegQueue<SignalValue>>,
}

impl GRPCClientImpl {
    /// Parses the value published by a provider.
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
    fn parse_value(value: String) -> String {
        match serde_json::from_str::<Value>(&value) {
            Ok(v) => {
                let property_map = match v.as_object() {
                    Some(o) => o,
                    None => {
                        warn!("Could not parse value as JSON object");
                        return value;
                    }
                };

                let mut selected_property = None;
                for property in property_map.iter() {
                    if property.0 == METADATA_KEY {
                        continue;
                    }

                    let selected_value = match property.1 {
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        _ => continue,
                    };

                    selected_property = Some((property.0, selected_value));
                    break;
                }

                match selected_property {
                    Some((k,v)) => {
                        let metadata_descriptor = if property_map.contains_key(&METADATA_KEY.to_string()) {
                            "has"
                        } else {
                            "doesn't have"
                        };

                        debug!(
                            "Value contained {} properties and {metadata_descriptor} a $metadata property. Selecting property with key {} as the signal value",
                            property_map.len(),
                            k
                        );

                        v
                    },
                    None => {
                        warn!("Could not find a property that was parseable as a value");
                        value
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse value: {e}");
                value
            }
        }
    }
}

#[tonic::async_trait]
impl DigitalTwinConsumer for GRPCClientImpl {
    /// Publish implementation.
    ///
    /// # Arguments
    /// * `request` - Publish request.
    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let PublishRequest { entity_id, value } = request.into_inner();

        debug!("Received a publish for entity id {entity_id} with the value {value}");

        let value = Self::parse_value(value);

        let new_signal_value = SignalValue { entity_id, value };
        self.signal_values_queue.push(new_signal_value);
        let response = PublishResponse {};
        Ok(Response::new(response))
    }

    /// Respond implementation.
    ///
    /// # Arguments
    /// * `request` - Respond request.
    async fn respond(
        &self,
        request: Request<RespondRequest>,
    ) -> Result<Response<RespondResponse>, Status> {
        warn!("Got a response request: {request:?}");

        Err(Status::unimplemented("respond has not been implemented"))
    }
}

#[cfg(test)]
mod grpc_client_impl_tests {
    use super::*;

    #[tokio::test]
    async fn publish_test() {
        let consumer_impl = GRPCClientImpl {
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let entity_id = String::from("some-id");
        let value = String::from("some-value");

        let request = tonic::Request::new(PublishRequest { entity_id, value });
        let result = consumer_impl.publish(request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn parse_value_returns_input_when_parse_fails() {
        let input = r#"invalid json"#;
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_is_plain_string() {
        let input = r#""value""#;
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_has_zero_properties() {
        let input = r#"{}"#;
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_input_has_no_valid_properties() {
        let input = format!(r#"{{"{METADATA_KEY}": "foo"}}"#);
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_input_when_property_value_is_not_string() {
        let input = r#"{"property": ["value"]}"#;
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, input);
    }

    #[test]
    fn parse_value_returns_correct_value_for_strings() {
        let expected_value = "value";
        let input = format!(r#"{{"property": "{expected_value}", "{METADATA_KEY}": "foo"}}"#);
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_returns_correct_value_for_bools() {
        let expected_value = "true";
        let input = format!(r#"{{"property": {expected_value}, "{METADATA_KEY}": "foo"}}"#);
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_returns_correct_value_for_numbers() {
        let expected_value = "123.456";
        let input = format!(r#"{{"property": {expected_value}, "{METADATA_KEY}": "foo"}}"#);
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }

    #[test]
    fn parse_value_skips_non_primitive_properties() {
        let expected_value = "value";
        let input = format!(r#"{{"foo": ["bar"], "property": "{expected_value}", "{METADATA_KEY}": "foo"}}"#);
        let result = GRPCClientImpl::parse_value(input.to_string());

        assert_eq!(result, expected_value);
    }
}
