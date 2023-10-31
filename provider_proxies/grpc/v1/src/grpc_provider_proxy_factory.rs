// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crossbeam::queue::SegQueue;
use freyja_contracts::{
    entity::{Entity, EntityEndpoint},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyFactory, SignalValue},
};

use crate::{
    grpc_provider_proxy::GRPCProviderProxy, GET_OPERATION, GRPC_PROTOCOL, SUBSCRIBE_OPERATION,
};

pub struct GRPCProviderProxyFactory {}

impl ProviderProxyFactory for GRPCProviderProxyFactory {
    fn is_supported(&self, entity: &Entity) -> Option<EntityEndpoint> {
        for endpoint in entity.endpoints.iter() {
            if endpoint.protocol == GRPC_PROTOCOL {
                for operation in endpoint.operations.iter() {
                    if operation == GET_OPERATION || operation == SUBSCRIBE_OPERATION {
                        // This entity is supported! The proxy will worry about how to handle operations,
                        // right now we just need to know if it can do anything at all with this entity.
                        return Some(endpoint.clone());
                    }
                }
            }
        }

        None
    }

    fn create_proxy(
        &self,
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Arc<dyn ProviderProxy + Send + Sync>, ProviderProxyError> {
        GRPCProviderProxy::create_new(provider_uri, signal_values_queue)
    }
}
