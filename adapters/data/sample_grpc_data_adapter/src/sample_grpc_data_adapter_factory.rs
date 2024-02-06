// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use freyja_common::{
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterFactory},
    entity::{Entity, EntityEndpoint},
    signal_store::SignalStore,
};

use crate::{
    sample_grpc_data_adapter::SampleGRPCDataAdapter, GET_OPERATION, GRPC_PROTOCOL,
    SUBSCRIBE_OPERATION,
};

/// Factory for creating SampleGRPCDataAdapters
pub struct SampleGRPCDataAdapterFactory {}

impl DataAdapterFactory for SampleGRPCDataAdapterFactory {
    /// Create a new `GRPCDataAdapterFactory`
    fn create_new() -> Result<Self, DataAdapterError> {
        Ok(Self {})
    }

    /// Check to see whether this factory can create an adapter for the requested entity.
    /// Returns the first endpoint found that is supported by this factory.
    ///
    /// # Arguments
    /// - `entity`: the entity to check for compatibility
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        entity.is_supported(&[GRPC_PROTOCOL], &[GET_OPERATION, SUBSCRIBE_OPERATION])
    }

    /// Create a new adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider URI to associate with this adapter
    /// - `signals`: the shared signal store
    fn create_adapter(
        &self,
        provider_uri: &str,
        signals: Arc<SignalStore>,
    ) -> Result<Arc<dyn DataAdapter + Send + Sync>, DataAdapterError> {
        let adapter = SampleGRPCDataAdapter::create_new(provider_uri, signals)?;
        Ok(Arc::new(adapter))
    }
}
