// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use log::{debug, info};
use paho_mqtt::{Client, QOS_1};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::Config, MQTT_PROTOCOL, SUBSCRIBE_OPERATION};
use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    entity::EntityEndpoint,
    message_utils,
    out_dir,
    provider_proxy::{
        EntityRegistration, ProviderProxy, ProviderProxyError, ProviderProxyErrorKind,
    },
    signal_store::SignalStore,
};

const MQTT_CLIENT_ID_PREFIX: &str = "freyja-mqtt-proxy";

/// Interfaces with providers which support GRPC. Based on the Ibeji mixed sample.
/// Note that the current implementation works on the assumption that there is a
/// one-to-one mapping of topic to entity id.
pub struct MqttProviderProxy {
    /// The proxy config
    config: Config,

    /// The MQTT client
    client: Arc<Mutex<Client>>,

    /// Maps subscribed topics to their associated entity id
    subscriptions: Arc<Mutex<HashMap<String, String>>>,

    /// Shared signal store for all proxies to push new signal values
    signals: Arc<SignalStore>,
}

#[async_trait]
impl ProviderProxy for MqttProviderProxy {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signals`: The shared signal store
    fn create_new(
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Self, ProviderProxyError>
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

        let client =
            paho_mqtt::Client::new(create_options).map_err(ProviderProxyError::communication)?;

        Ok(MqttProviderProxy {
            config,
            client: Arc::new(Mutex::new(client)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            signals,
        })
    }

    /// Starts a provider proxy
    async fn start(&self) -> Result<(), ProviderProxyError> {
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
            let _ = client
                .connect(connection_options)
                .map_err(ProviderProxyError::communication);
        }

        let client = self.client.clone();
        let subscriptions = self.subscriptions.clone();
        let signals = self.signals.clone();

        // Start the thread for handling publishes from providers
        tokio::spawn(async move {
            info!("Started MQTT listener");
            for msg in receiver.iter() {
                if let Some(m) = msg {
                    let subsciptions = subscriptions.lock().await;
                    let entity_id = subsciptions.get(m.topic()).unwrap().clone();
                    let value = message_utils::parse_value(m.payload_str().to_string());
                    if signals.set_value(entity_id, value).is_none() {
                        log::warn!("Attempt to set value for non-existent signal");
                    }
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
                            }
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

        info!("Started an MQTTProviderProxy!");

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
    ) -> Result<EntityRegistration, ProviderProxyError> {
        // Verify that the endpoint has the expected data.
        // This shouldn't be necessary since it's first verified by the factory,
        // but this ensures we don't get hit by an edge case
        if endpoint.protocol != MQTT_PROTOCOL
            || !endpoint
                .operations
                .contains(&SUBSCRIBE_OPERATION.to_string())
        {
            return Err(ProviderProxyErrorKind::OperationNotSupported.into());
        }

        // Topic comes from the endpoint context
        let topic = endpoint.context.clone();
        debug!("Subscribing to topic {topic}");

        let client = self.client.lock().await;
        client
            .subscribe(&topic, QOS_1)
            .map_err(ProviderProxyError::communication)?;
        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.insert(topic, entity_id.to_string());

        Ok(EntityRegistration::Registered)
    }
}
