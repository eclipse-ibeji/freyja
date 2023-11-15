// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![allow(unused_variables)]

use std::sync::Arc;

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::info;
use core_protobuf_data_access::module::managed_subscribe::v1::{
    managed_subscribe_client::ManagedSubscribeClient, SubscriptionInfoRequest, Constraint,
};
use tonic::transport::Channel;

use crate::{config::Config, GRPC_PROTOCOL, MQTT_PROTOCOL, MANAGED_SUBSCRIBE_OPERATION, SUBSCRIBE_OPERATION};
use freyja_build_common::config_file_stem;
use freyja_common::{config_utils, out_dir};
use freyja_contracts::{
    entity::{EntityEndpoint, Entity},
    provider_proxy::{ProviderProxy, ProviderProxyError, ProviderProxyErrorKind, SignalValue, EntityRegistration},
};

/// Interfaces with providers which support GRPC. Based on the Ibeji mixed sample.
/// Note that the current implementation works on the assumption that there is a
/// one-to-one mapping of topic to entity id.
pub struct ManagedSubscribeProviderProxy {
    /// The proxy config
    config: Config,

    /// Client for connecting to the Managed Subscribe service.
    client: ManagedSubscribeClient<Channel>,
}

#[async_trait]
impl ProviderProxy for ManagedSubscribeProviderProxy {
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

        let client = futures::executor::block_on(async {
            ManagedSubscribeClient::connect(String::from(provider_uri))
                .await
                .map_err(ProviderProxyError::communication)
        })?;

        Ok(ManagedSubscribeProviderProxy {
            config,
            client,
        })
        .map(|r| Arc::new(r) as _)
    }

    /// Starts a provider proxy
    async fn start(&self) -> Result<(), ProviderProxyError> {
        // Not relevant for this proxy, so passthrough.
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

    /// Loopback proxy that calls the 'Managed Subscribe' module in Ibeji to retrieve correct subscription endpoint.
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
        if endpoint.protocol != GRPC_PROTOCOL
            || !endpoint
                .operations
                .contains(&MANAGED_SUBSCRIBE_OPERATION.to_string())
        {
            return Err(ProviderProxyErrorKind::OperationNotSupported.into());
        }

        let mut client = self.client.clone();

        let default_freq_constraint = Constraint {
            r#type: self.config.frequency_constraint_type.clone(),
            value: self.config.frequency_constraint_value.clone(),
        };

        let request = tonic::Request::new(SubscriptionInfoRequest {
            entity_id: entity_id.to_string(),
            constraints: vec![default_freq_constraint],
        });

        info!("Calling Managed Subscribe with {request:?}.");

        let result = client
            .get_subscription_info(request)
            .await
            .map_err(ProviderProxyError::communication)?;

        info!("Managed Subscribe returned {result:?}.");

        let sub_info = result.into_inner();

        let mut protocol = sub_info.protocol;

        if protocol.contains(MQTT_PROTOCOL) {
            protocol = MQTT_PROTOCOL.to_string();
        }

        let endpoint = EntityEndpoint {
            protocol,
            operations: vec![SUBSCRIBE_OPERATION.to_string()],
            uri: sub_info.uri,
            context: sub_info.context,
        };

        let new_entity = Entity {
            name: Some(entity_id.to_string()),
            id: entity_id.to_string(),
            description: None,
            endpoints: vec![endpoint],
        };

        info!("New Entity constructed: {new_entity:?}");

        Ok(EntityRegistration::Loopback(new_entity))
    }
}
