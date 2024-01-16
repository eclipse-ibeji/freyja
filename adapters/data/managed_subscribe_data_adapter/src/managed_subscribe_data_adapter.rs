// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use core_protobuf_data_access::module::managed_subscribe::v1::{
    managed_subscribe_client::ManagedSubscribeClient, Constraint, SubscriptionInfoRequest,
};
use log::{debug, info};
use tonic::transport::Channel;

use crate::{
    config::Config, GRPC_PROTOCOL, MANAGED_SUBSCRIBE_OPERATION, MQTT_PROTOCOL, SUBSCRIBE_OPERATION,
};

use freyja_build_common::config_file_stem;
use freyja_common::{
    config_utils,
    data_adapter::{DataAdapter, DataAdapterError, DataAdapterErrorKind, EntityRegistration},
    entity::{Entity, EntityEndpoint},
    out_dir,
    signal_store::SignalStore,
};

/// Interfaces with providers which utilize 'Managed Subscribe'. Based on the Ibeji managed
/// subscribe sample. Note that the current implementation works on the assumption that there is a
/// one-to-one mapping of topic to entity id.
pub struct ManagedSubscribeDataAdapter {
    /// The adapter config
    config: Config,

    /// Client for connecting to the Managed Subscribe service.
    client: ManagedSubscribeClient<Channel>,
}

#[async_trait]
impl DataAdapter for ManagedSubscribeDataAdapter {
    /// Creates a data adapter
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `_signals`: the shared signal store (unused in this adapter)
    fn create_new(provider_uri: &str, _signals: Arc<SignalStore>) -> Result<Self, DataAdapterError>
    where
        Self: Sized,
    {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            DataAdapterError::io,
            DataAdapterError::deserialize,
        )?;

        let client = futures::executor::block_on(async {
            ManagedSubscribeClient::connect(String::from(provider_uri))
                .await
                .map_err(DataAdapterError::communication)
        })?;

        Ok(ManagedSubscribeDataAdapter { config, client })
    }

    /// Starts a data adapter
    async fn start(&self) -> Result<(), DataAdapterError> {
        // Not relevant for this adapter as the adapter is just retrieving the subscription information
        // and has no persistent state.
        Ok(())
    }

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, _entity_id: &str) -> Result<(), DataAdapterError> {
        // No actions for this provider when calling this function
        Ok(())
    }

    /// Calls the `Managed Subscribe` module in Ibeji to retrieve correct subscription information
    /// and returns a `Loopback` request to the data adapter selector
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to get information for
    /// - `endpoint`: the endpoint that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        endpoint: &EntityEndpoint,
    ) -> Result<EntityRegistration, DataAdapterError> {
        // Verify that the endpoint has the expected data.
        // This shouldn't be necessary since it's first verified by the factory,
        // but this ensures we don't get hit by an edge case
        if endpoint.protocol != GRPC_PROTOCOL
            || !endpoint
                .operations
                .contains(&MANAGED_SUBSCRIBE_OPERATION.to_string())
        {
            return Err(DataAdapterErrorKind::OperationNotSupported.into());
        }

        let mut client = self.client.clone();

        // Set the default frequency to recieve data at
        let default_freq_constraint = Constraint {
            r#type: self.config.frequency_constraint_type.clone(),
            value: self.config.frequency_constraint_value.clone(),
        };

        let request = tonic::Request::new(SubscriptionInfoRequest {
            entity_id: entity_id.to_string(),
            constraints: vec![default_freq_constraint],
        });

        let result = client
            .get_subscription_info(request)
            .await
            .map_err(DataAdapterError::communication)?;

        let sub_info = result.into_inner();

        // The MQTT data adapter supports v5 and v3 so do not need to make a distinction
        let mut protocol = sub_info.protocol;
        if protocol.contains(MQTT_PROTOCOL) {
            protocol = MQTT_PROTOCOL.to_string();
        }

        // Construct endpoint information from returned result
        let endpoint = EntityEndpoint {
            protocol,
            operations: vec![SUBSCRIBE_OPERATION.to_string()],
            uri: sub_info.uri,
            context: sub_info.context,
        };

        // Create new entity object with updated endpoint information.
        let new_entity = Entity {
            name: Some(entity_id.to_string()),
            id: entity_id.to_string(),
            description: None,
            endpoints: vec![endpoint],
        };

        info!("Loopback requested.");
        debug!("Looback request contains Entity: {new_entity:?}");

        Ok(EntityRegistration::Loopback(new_entity))
    }
}
