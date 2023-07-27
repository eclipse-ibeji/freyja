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
    provider_proxy_request::{ProviderProxySelectorRequestSender, ProviderProxySelectorRequestKind},
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

    /// Updates a shared entity map to populate empty values with provider information.
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
        Self: Sized
    {
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
                    let Entity { id, uri, operation, protocol, .. } = response.entity;
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
    use crate::provider_proxy::OperationKind;

    use super::*;

    use tokio::sync::mpsc;
    
    fn get_test_entity(id: &String) -> Entity
    {
        Entity { 
            id: id.clone(), 
            name: Some("name".to_string()), 
            uri: "uri".to_string(), 
            description: Some("description".to_string()), 
            operation: OperationKind::Get,
            protocol: "protocol".to_string(),
        } 
    }

    struct TestDigitalTwinAdapter {}

    #[async_trait]
    impl DigitalTwinAdapter for TestDigitalTwinAdapter {
        fn create_new() -> Result<Box<dyn DigitalTwinAdapter + Send + Sync>, DigitalTwinAdapterError> {
            Ok(Box::new(Self {}))
        }

        async fn find_by_id(
            &self,
            request: GetDigitalTwinProviderRequest,
        ) -> Result<GetDigitalTwinProviderResponse, DigitalTwinAdapterError> {
            Ok(GetDigitalTwinProviderResponse { 
                entity: get_test_entity(&request.entity_id),
            })
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

    #[tokio::test]
    async fn update_entity_map_updates_none_value() {
        // Setup
        let id = String::from("id");
        let mut entites: HashMap<EntityID, Option<Entity>> = HashMap::new();
        entites.insert(id.clone(), None);
        let shared_map = Arc::new(Mutex::new(entites));

        let (sender, mut receiver) =
            mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let ppsrs = ProviderProxySelectorRequestSender::new(sender);

        let data_received: Arc<Mutex<Option<ProviderProxySelectorRequestKind>>> = Arc::new(Mutex::new(None));
        let data_received_clone = data_received.clone();

        let listener = tokio::spawn(async move {
            let value = receiver.recv().await;
            let mut data = data_received_clone.lock().unwrap();
            *data = value;
        });
        
        let uut = TestDigitalTwinAdapter {};

        // Test
        let update_result = uut.update_entity_map(shared_map.clone(), Arc::new(ppsrs)).await;
        let join_result = listener.await;

        // Verify
        assert!(update_result.is_ok());
        assert!(join_result.is_ok());

        let guard = shared_map.lock().unwrap();
        let value = guard.get(&id);
        assert!(value.is_some());
        assert!(value.unwrap().is_some());
        let entity = value.unwrap().as_ref().unwrap();
        assert_eq!(entity.id, id);

        let data = data_received.lock().unwrap();
        assert!(data.is_some());
        match data.as_ref().unwrap() {
            ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy(entity_id, _, _, _) => assert_eq!(*entity_id, id),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    async fn update_entity_map_ignores_some_value() {
        // Setup
        let id = String::from("id");
        let mut entites: HashMap<EntityID, Option<Entity>> = HashMap::new();
        entites.insert(id.clone(), Some(get_test_entity(&id)));
        let shared_map = Arc::new(Mutex::new(entites));

        let (sender, mut receiver) =
            mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let ppsrs = ProviderProxySelectorRequestSender::new(sender);

        let data_received: Arc<Mutex<Option<ProviderProxySelectorRequestKind>>> = Arc::new(Mutex::new(None));
        let data_received_clone = data_received.clone();

        let listener = tokio::spawn(async move {
            let value = receiver.recv().await;
            let mut data = data_received_clone.lock().unwrap();
            *data = value;
        });
        
        let uut = TestDigitalTwinAdapter {};

        // Test
        let update_result = uut.update_entity_map(shared_map.clone(), Arc::new(ppsrs)).await;
        let join_result = listener.await;

        // Verify
        assert!(update_result.is_ok());
        assert!(join_result.is_ok());

        let guard = shared_map.lock().unwrap();
        let value = guard.get(&id);
        assert!(value.is_some());
        assert!(value.unwrap().is_some());
        let entity = value.unwrap().as_ref().unwrap();
        assert_eq!(entity.id, id);

        let data = data_received.lock().unwrap();
        assert!(data.is_none());
    }
}