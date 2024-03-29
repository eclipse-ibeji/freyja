// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use freyja_common::{
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterFactory},
    entity::{Entity, EntityEndpoint},
    signal_store::SignalStore,
};

use crate::{mqtt_data_adapter::MqttDataAdapter, MQTT_PROTOCOL, SUBSCRIBE_OPERATION};

/// Factory for creating MqttDataAdapters
pub struct MqttDataAdapterFactory {}

impl DataAdapterFactory for MqttDataAdapterFactory {
    /// Create a new `MqttDataAdapterFactory`
    fn create_new() -> Result<Self, DataAdapterError> {
        Ok(Self {})
    }

    /// Check to see whether this factory can create a data adapter for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[MQTT_PROTOCOL], &[SUBSCRIBE_OPERATION])
    }

    /// Create a new data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider URI to associate with this data adapter
    /// - `signals`: the shared signal store
    fn create_adapter(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn DataAdapter + Send + Sync>, DataAdapterError> {
        let adapter = MqttDataAdapter::create_new(provider_uri, signals)?;
        Ok(Arc::new(adapter))
    }
}
