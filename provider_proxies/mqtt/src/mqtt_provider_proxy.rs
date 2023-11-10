// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    sync::Arc, time::Duration,
};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::{info, debug, warn};
use paho_mqtt::{Client, QOS_1};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::Config, SUBSCRIBE_OPERATION, MQTT_PROTOCOL};
use freyja_build_common::config_file_stem;
use freyja_common::{config_utils, out_dir};
use freyja_contracts::{
    entity::EntityEndpoint,
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyErrorKind, SignalValue},
};

const METADATA_KEY: &str = "$metadata";
const MQTT_CLIENT_ID_PREFIX: &str = "freyja-mqtt-proxy";

/// Interfaces with providers which support GRPC. Based on the Ibeji mixed sample.
/// Note that the current implementation works on the assumption that there is a
/// one-to-one mapping of topic to entity id.
/// TODO: can this deadlock?
pub struct MqttProviderProxy {
    /// The proxy config
    config: Config,
    
    /// The MQTT client
    client: Arc<Mutex<Client>>,

    /// Maps subscribed topics to their associated entity id
    subscriptions: Arc<Mutex<HashMap<String, String>>>,

    /// Shared queue for all proxies to push new signal values of entities
    signal_values_queue: Arc<SegQueue<SignalValue>>,
}

// TODO: make this common?
impl MqttProviderProxy {
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
        match serde_json::from_str::<serde_json::Value>(&value) {
            Ok(v) => {
                let property_map = match v.as_object() {
                    Some(o) => o,
                    None => {
                        warn!("Could not parse value as JSON object");
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

                    let metadata_descriptor =
                        if property_map.contains_key(&METADATA_KEY.to_string()) {
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
                warn!("Failed to parse value: {e}");
                value
            }
        }
    }
}

#[async_trait]
impl ProviderProxy for MqttProviderProxy {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_values_queue`: shared queue for all proxies to push new signal values of entities
    fn create_new(
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError>
    where
        Self: Sized,
    {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            ProviderProxyError::io,
            ProviderProxyError::deserialize,
        )?;

        let client_id = format!("{MQTT_CLIENT_ID_PREFIX}_{}", Uuid::new_v4());
        let create_options = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(provider_uri)
            .client_id(client_id)
            .finalize();

        let client = paho_mqtt::Client::new(create_options)
            .map_err(ProviderProxyError::communication)?;

        Ok(MqttProviderProxy {
            config,
            client: Arc::new(Mutex::new(client)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            signal_values_queue,
        })
        .map(|r| Arc::new(r) as _)
    }

    /// Runs a provider proxy
    async fn run(&self) -> Result<(), ProviderProxyError> {
        info!("Started an MQTTProviderProxy!");

        let lwt = paho_mqtt::MessageBuilder::new()
            .topic("test")
            .payload("Receiver lost connection")
            .finalize();

        let connection_options = paho_mqtt::ConnectOptionsBuilder::new_v5()
            .keep_alive_interval(Duration::from_secs(self.config.keep_alive_interval_s))
            .clean_session(false)
            .will_message(lwt)
            .finalize();
        
        let receiver;
        {
            let client = self.client.lock().await;
            receiver = client.start_consuming();
            let _ = client.connect(connection_options)
                .map_err(ProviderProxyError::communication);
        }

        let client = self.client.clone();
        let subscriptions = self.subscriptions.clone();
        let signal_values_queue = self.signal_values_queue.clone();

        // Start the thread for handling publishes from providers
        tokio::spawn(async move {
            for msg in receiver.iter() {
                if let Some(m) = msg {
                    let subsciptions = subscriptions.lock().await;
                    let entity_id = subsciptions.get(m.topic()).unwrap().clone();
                    // TODO: additional parsing for value?
                    let value = Self::parse_value(m.to_string());
                    signal_values_queue.push(SignalValue { entity_id, value });
                } else {
                    let client = client.lock().await;
                    if !client.is_connected() {
                        match client.reconnect() {
                            Ok(_) => {
                                let subscriptions = subscriptions.lock().await;
                                for (topic, _) in subscriptions.iter() {
                                    if let Err(e) = client.subscribe(topic, QOS_1) {
                                        log::error!("Error resubscribing to topic {topic}: {e}");
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("Fatal error trying to reconnect to mqtt client: {e}");
                                break;
                            }
                        }
                    }
                }
            }

            let client = client.lock().await;
            if client.is_connected() {
                debug!("Disconnecting from MQTT client...");
                let subscriptions = subscriptions.lock().await;
                for (topic, _) in subscriptions.iter() {
                    if let Err(e) = client.unsubscribe(topic) {
                        log::error!("Error unsubscribing from topic {topic}: {e}");
                    }
                }

                if let Err(e) = client.disconnect(None) {
                    log::error!("Error disconnecting from MQTT client: {e}");
                }
            }
        });

        Ok(())
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, _entity_id: &str) -> Result<(), ProviderProxyError> {
        // No actions for this provider when calling this function
        Ok(())
    }

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<(), ProviderProxyError> {
        // Verify that the endpoint has the expected data.
        // This shouldn't be necessary since it's first verified by the factory,
        // but this ensures we don't get hit by an edge case
        if endpoint.protocol != MQTT_PROTOCOL || !endpoint.operations.contains(&SUBSCRIBE_OPERATION.to_string()) {
            return Err(ProviderProxyErrorKind::OperationNotSupported.into());
        }
        
        // Topic comes from the endpoint context
        let topic = endpoint.context.clone();
        let client = self.client.lock().await;
        client.subscribe(&topic, QOS_1).map_err(ProviderProxyError::communication)?;
        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.insert(topic, entity_id.to_string());

        Ok(())
    }
}