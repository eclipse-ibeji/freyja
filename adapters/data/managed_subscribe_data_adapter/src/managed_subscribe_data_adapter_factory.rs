// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use freyja_common::{
    entity::{Entity, EntityEndpoint},
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterFactory},
    signal_store::SignalStore,
};

use crate::{
    managed_subscribe_data_adapter::ManagedSubscribeDataAdapter, GRPC_PROTOCOL,
    MANAGED_SUBSCRIBE_OPERATION,
};

/// Factory for creating ManagedSubscribeDataAdapters
pub struct ManagedSubscribeDataAdapterFactory {}

impl DataAdapterFactory for ManagedSubscribeDataAdapterFactory {
    /// Create a new `ManagedSubscribeDataAdapterFactory`
    fn create_new() -> Result<Self, DataAdapterError> {
        Ok(Self {})
    }

    /// Check to see whether this factory can create a data adapter for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[GRPC_PROTOCOL], &[MANAGED_SUBSCRIBE_OPERATION])
    }

    /// Create a new data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider URI to associate with this adapter
    /// - `signals`: the shared signal store
    fn create_adapter(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn DataAdapter + Send + Sync>, DataAdapterError> {
        let adapter = ManagedSubscribeDataAdapter::create_new(provider_uri, signals)?;
        Ok(Arc::new(adapter))
    }
}
