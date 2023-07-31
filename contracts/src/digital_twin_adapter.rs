// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityID},
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
};

/// Provides digital twin data
#[async_trait]
pub trait DigitalTwinAdapter {
    /// Creates a new instance of a DigitalTwinAdapter with default settings
    fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError>
    where
        Self: Sized;

    /// Gets entity access information
    ///
    /// # Arguments
    /// - `request`: the request for finding an entity's access information
    async fn find_by_id(
        &self,
        request: GetDigitalTwinProviderRequest,
    ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError>;

    /// Run as a client to the in-vehicle digital twin provider
    ///
    /// # Arguments
    /// - `entity_map`: shared map of entity ID to entity information
    /// - `sleep_interval`: the interval in milliseconds between finding the access info of entities
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    async fn run(
        &self,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        sleep_interval: Duration,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
    ) -> Result<(), DigitalTwinAdapterError>;

    /// Updates a shared entity map to populate empty values with provider information fetched from the digital twin service.
    /// This default implementation is shared for all providers.
    ///
    /// # Arguments
    /// - `entity_map`: the map to update
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    async fn update_entity_map(
        &self,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
    ) -> Result<(), DigitalTwinAdapterError>
    where
        Self: Sized,
    {
        // Copy the shared map
        let mut updated_entities = {
            let map = entity_map.lock().unwrap();
            map.clone()
        };

        // Update any entries which don't have an entity yet
        for (entity_id, entity) in updated_entities.iter_mut().filter(|(_, e)| e.is_none()) {
            let request = GetDigitalTwinProviderRequest {
                entity_id: entity_id.clone(),
            };

            match self.find_by_id(request).await {
                Ok(response) => {
                    *entity = Some(response.entity.clone());

                    // Notify the provider proxy selector to start a proxy
                    let Entity {
                        id,
                        uri,
                        operation,
                        protocol,
                        ..
                    } = response.entity;
                    let request = ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(
                        id, uri, protocol, operation,
                    );

                    provider_proxy_selector_request_sender
                        .send_request_to_provider_proxy_selector(request);
                }
                Err(err) => {
                    log::error!("{err}");
                }
            };
        }

        // Update the shared map
        {
            let mut map = entity_map.lock().unwrap();
            *map = updated_entities;
        }

        Ok(())
    }
}

/// A request for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDigitalTwinProviderRequest {
    /// The entity's id to inquire about
    pub entity_id: String,
}

/// The response for digital twin providers
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDigitalTwinProviderResponse {
    /// Entity information
    pub entity: Entity,
}

/// A request for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueRequest {
    /// The entity's ID
    pub entity_id: String,

    /// The callback uri for a provider to send data back
    pub callback_uri: String,
}

/// A response for an entity's value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityValueResponse {
    // The id of the entity
    pub entity_id: String,

    /// The value of the entity
    pub value: String,
}

proc_macros::error! {
    DigitalTwinAdapterError {
        EntityNotFound,
        Io,
        Serialize,
        Deserialize,
        Communication,
        ParseError,
        Unknown
    }
}

#[cfg(test)]
mod digital_twin_adapter_tests {
    use super::*;

    use crate::provider_proxy::OperationKind;

    use rstest::*;
    use tokio::{
        sync::mpsc::{self},
        task::JoinHandle,
    };

    struct TestDigitalTwinAdapter {
        entity: Entity,
    }

    #[async_trait]
    impl DigitalTwinAdapter for TestDigitalTwinAdapter {
        fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError>
        {
            Err(DigitalTwinAdapterError::unknown("not implemented"))
        }

        async fn find_by_id(
            &self,
            request: GetDigitalTwinProviderRequest,
        ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError> {
            if self.entity.id == request.entity_id {
                Ok(GetDigitalTwinProviderResponse {
                    entity: self.entity.clone(),
                })
            } else {
                Err(DigitalTwinAdapterError::entity_not_found("not found"))
            }
        }

        async fn run(
            &self,
            _entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
            _sleep_interval: Duration,
            _provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
        ) -> Result<(), DigitalTwinAdapterError> {
            Err(DigitalTwinAdapterError::unknown("not implemented"))
        }
    }

    struct TestFixture {
        adapter: TestDigitalTwinAdapter,
        entity_id: String,
        entity: Entity,
        map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        sender: Arc<ProviderProxySelectorRequestSender>,
        listener_handler: JoinHandle<Option<ProviderProxySelectorRequestKind>>,
    }

    #[fixture]
    fn fixture() -> TestFixture {
        let entity = Entity {
            id: "entity_id".to_string(),
            name: Some("name".to_string()),
            uri: "uri".to_string(),
            description: Some("description".to_string()),
            operation: OperationKind::Get,
            protocol: "protocol".to_string(),
        };

        let (sender, mut receiver) = mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();

        let listener_handler = tokio::spawn(async move { receiver.recv().await });

        TestFixture {
            adapter: TestDigitalTwinAdapter {
                entity: entity.clone(),
            },
            entity_id: entity.id.clone(),
            entity,
            map: Arc::new(Mutex::new(HashMap::new())),
            sender: Arc::new(ProviderProxySelectorRequestSender::new(sender)),
            listener_handler,
        }
    }

    fn assert_entry_is_in_map(
        entry: (String, Option<Entity>),
        map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
    ) {
        let (id, entity) = entry;
        let map = map.lock().unwrap();
        let value = map.get(&id);
        assert!(value.is_some());

        match entity {
            Some(entity) => {
                assert!(value.unwrap().is_some());
                let retrieved_entity = value.unwrap().as_ref().unwrap();
                assert_eq!(entity, *retrieved_entity);
            }
            None => {
                assert!(value.unwrap().is_none());
            }
        }
    }

    // Variation of assert_entry_is_in_map for conveneince
    fn assert_entity_is_in_map(entity: Entity, map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>) {
        assert_entry_is_in_map((entity.id.clone(), Some(entity)), map)
    }

    #[rstest]
    #[tokio::test]
    async fn update_entity_map_updates_none_value(fixture: TestFixture) {
        // Setup
        {
            let mut map = fixture.map.lock().unwrap();
            map.insert(fixture.entity_id.clone(), None);
        }

        // Test
        let update_result = fixture
            .adapter
            .update_entity_map(fixture.map.clone(), fixture.sender)
            .await;
        let join_result = fixture.listener_handler.await;

        // Verify
        assert!(update_result.is_ok());
        assert!(join_result.is_ok());

        assert_entity_is_in_map(fixture.entity.clone(), fixture.map.clone());

        let proxy_request = join_result.unwrap();
        assert!(proxy_request.is_some());
        let proxy_request = proxy_request.as_ref().unwrap();
        match proxy_request {
            ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(
                entity_id,
                uri,
                protocol,
                operation,
            ) => {
                assert_eq!(*entity_id, fixture.entity_id);
                assert_eq!(*uri, fixture.entity.uri);
                assert_eq!(*protocol, fixture.entity.protocol);
                assert_eq!(*operation, fixture.entity.operation);
            }
            _ => panic!("Unexpected proxy request kind: {proxy_request:?}"),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn update_entity_map_skips_existing_values(fixture: TestFixture) {
        // Setup
        {
            let mut map = fixture.map.lock().unwrap();
            map.insert(fixture.entity_id, Some(fixture.entity.clone()));
        }

        // Test
        let update_result = fixture
            .adapter
            .update_entity_map(fixture.map.clone(), fixture.sender)
            .await;
        let join_result = fixture.listener_handler.await;

        // Verify
        assert!(update_result.is_ok());
        assert!(join_result.is_ok());

        assert_entity_is_in_map(fixture.entity, fixture.map.clone());

        let proxy_request = join_result.unwrap();
        assert!(proxy_request.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn update_entity_map_handles_entity_not_found(fixture: TestFixture) {
        // Setup
        let non_existent_id = String::from("fooid");

        {
            let mut map = fixture.map.lock().unwrap();
            map.insert(non_existent_id.clone(), None);
        }

        // Test
        let update_result = fixture
            .adapter
            .update_entity_map(fixture.map.clone(), fixture.sender)
            .await;
        let join_result = fixture.listener_handler.await;

        // Verify
        assert!(update_result.is_ok());
        assert!(join_result.is_ok());

        assert_entry_is_in_map((non_existent_id, None), fixture.map.clone());

        let proxy_request = join_result.unwrap();
        assert!(proxy_request.is_none());
    }
}
