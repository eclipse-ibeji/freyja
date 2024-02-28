// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use core_protobuf_data_access::invehicle_digital_twin::v1::{
    invehicle_digital_twin_server::InvehicleDigitalTwin, FindByIdRequest as IbejiFindByIdRequest,
    FindByIdResponse as IbejiFindByIdResponse, RegisterRequest, RegisterResponse,
};
use mockall::*;
use tonic::{Request, Response, Status};

use cloud_connector_proto::v1::{
    cloud_connector_server::CloudConnector, UpdateDigitalTwinRequest, UpdateDigitalTwinResponse,
};
use freyja_common::{
    cloud_adapter::{CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse},
    data_adapter::DataAdapterFactory,
    data_adapter_selector::{DataAdapterSelector, DataAdapterSelectorError},
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, FindByIdRequest, FindByIdResponse,
    },
    entity::Entity,
    mapping_adapter::{
        CheckForWorkRequest, CheckForWorkResponse, GetMappingRequest, GetMappingResponse,
        MappingAdapter, MappingAdapterError,
    },
    service_discovery_adapter::{ServiceDiscoveryAdapter, ServiceDiscoveryAdapterError},
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};
use mapping_service_proto::v1::{
    mapping_service_server::MappingService, CheckForWorkRequest as ProtoCheckForWorkRequest,
    CheckForWorkResponse as ProtoCheckForWorkResponse, GetMappingRequest as ProtoGetMappingRequest,
    GetMappingResponse as ProtoGetMappingResponse,
};

mock! {
    pub CloudAdapter {}

    #[async_trait]
    impl CloudAdapter for CloudAdapter {
        fn create_new(
            selector: Arc<tokio::sync::Mutex<dyn ServiceDiscoveryAdapterSelector>>,
        ) -> Result<Self, CloudAdapterError>
        where
            Self: Sized;

        async fn send_to_cloud(
            &self,
            cloud_message: CloudMessageRequest,
        ) -> Result<CloudMessageResponse, CloudAdapterError>;
    }
}

mock! {
    pub DataAdapterSelector {}

    #[async_trait]
    impl DataAdapterSelector for DataAdapterSelector {
        fn register(
            &mut self,
            factory: Box<dyn DataAdapterFactory + Send + Sync>
        ) -> Result<(), DataAdapterSelectorError>;

        async fn create_or_update_adapter(
            &self,
            entity: &Entity
        ) -> Result<(), DataAdapterSelectorError>;

        async fn request_entity_value(
            &self,
            entity_id: &str
        ) -> Result<(), DataAdapterSelectorError>;
    }
}

mock! {
    pub DigitalTwinAdapter {}

    #[async_trait]
    impl DigitalTwinAdapter for DigitalTwinAdapter {
        fn create_new(
            selector: Arc<tokio::sync::Mutex<dyn ServiceDiscoveryAdapterSelector>>
        ) -> Result<Self, DigitalTwinAdapterError>
        where
            Self: Sized;

        async fn find_by_id(
            &self,
            request: FindByIdRequest,
        ) -> Result<FindByIdResponse, DigitalTwinAdapterError>;
    }
}

mock! {
    pub MappingAdapter {}

    #[async_trait]
    impl MappingAdapter for MappingAdapter {
        fn create_new(
            selector: Arc<tokio::sync::Mutex<dyn ServiceDiscoveryAdapterSelector>>
        ) -> Result<Self, MappingAdapterError>
        where
            Self: Sized;

        async fn check_for_work(
            &self,
            request: CheckForWorkRequest,
        ) -> Result<CheckForWorkResponse, MappingAdapterError>;

        async fn get_mapping(
            &self,
            request: GetMappingRequest,
        ) -> Result<GetMappingResponse, MappingAdapterError>;
    }
}

mock! {
    pub ServiceDiscoveryAdapterSelector {}

    #[async_trait]
    impl ServiceDiscoveryAdapterSelector for ServiceDiscoveryAdapterSelector {
        fn register(
            &mut self,
            adapter: Box<dyn ServiceDiscoveryAdapter + Send + Sync>
        ) -> Result<(), ServiceDiscoveryAdapterError>;

        async fn get_service_uri<'a>(
            &self,
            id: &'a str
        ) -> Result<String, ServiceDiscoveryAdapterError>;
    }
}

mock! {
    pub CloudConnector {}

    #[async_trait]
    impl CloudConnector for CloudConnector {
        async fn update_digital_twin(
            &self,
            _request: Request<UpdateDigitalTwinRequest>,
        ) -> Result<Response<UpdateDigitalTwinResponse>, Status>;
    }
}

mock! {
    pub MappingService {}

    #[async_trait]
    impl MappingService for MappingService {
        async fn check_for_work(
            &self,
            _request: Request<ProtoCheckForWorkRequest>,
        ) -> Result<Response<ProtoCheckForWorkResponse>, Status>;

        async fn get_mapping(
            &self,
            _request: Request<ProtoGetMappingRequest>,
        ) -> Result<Response<ProtoGetMappingResponse>, Status>;
    }
}

mock! {
    pub InVehicleDigitalTwin {}

    #[async_trait]
    impl InvehicleDigitalTwin for InVehicleDigitalTwin {
        async fn find_by_id(
            &self,
            request: Request<IbejiFindByIdRequest>,
        ) -> Result<Response<IbejiFindByIdResponse>, Status>;

        async fn register(
            &self,
            _request: Request<RegisterRequest>,
        ) -> Result<Response<RegisterResponse>, Status>;
    }
}
