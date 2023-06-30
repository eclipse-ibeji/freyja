// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, Clone, Serialize, Deserialize, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum OperationKind {
    #[strum(serialize = "Get")]
    Get,
    #[strum(serialize = "Subscribe")]
    Subscribe,
}

/// Represesnts a new signal value
pub struct SignalValue {
    /// The entity's id
    pub entity_id: String,

    /// The entity's value
    pub value: String,
}

#[async_trait]
pub trait ProviderProxy: Debug {
    /// Creates a provider proxy
    ///
    /// # Arguments
    /// - `provider_uri`: the provider uri for accessing an entity's information
    /// - `signal_values_queue`: shared queue for all provider proxies to push new signal values of entities
    fn create_new(
        provider_uri: &str,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Result<Box<dyn ProviderProxy + Send + Sync>, ProviderProxyError>
    where
        Self: Sized;

    /// Runs a provider proxy
    async fn run(&self) -> Result<(), ProviderProxyError>;

    /// Sends a request to a provider for obtaining the value of an entity
    ///
    /// # Arguments
    /// - `entity_id`: the entity id that needs a value
    async fn send_request_to_provider(&self, entity_id: &str) -> Result<(), ProviderProxyError>;

    /// Registers an entity id to a local cache inside a provider proxy to keep track of which entities a provider proxy contains.
    /// If the operation is Subscribe for an entity, the expectation is subscribe will happen in this function after registering an entity.
    ///
    /// # Arguments
    /// - `entity_id`: the entity id to add
    /// - `operation`: the operation that this entity supports
    async fn register_entity(
        &self,
        entity_id: &str,
        operation: &OperationKind,
    ) -> Result<(), ProviderProxyError>;

    /// Checks if this operation is supported
    ///
    /// # Arguments
    /// - `operation`: check to see if this operation is supported by this provider proxy
    fn is_operation_supported(operation: &OperationKind) -> bool
    where
        Self: Sized + Send + Sync;
}

proc_macros::error! {
    ProviderProxyError {
        Io,
        Parse,
        Serialize,
        Deserialize,
        Communication,
        EntityNotFound,
        Unknown
    }
}

#[cfg(test)]
mod provider_proxy_tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn provider_proxy_kind_match_test() {
        let mut subscribe = String::from("sUbScriBe");
        let mut operation_kind = OperationKind::from_str(&subscribe).unwrap();
        assert_eq!(operation_kind, OperationKind::Subscribe);

        subscribe = String::from("Subscribe");
        operation_kind = OperationKind::from_str(&subscribe).unwrap();
        assert_eq!(operation_kind, OperationKind::Subscribe);

        let mut get = String::from("gET");
        operation_kind = OperationKind::from_str(&get).unwrap();
        assert_eq!(operation_kind, OperationKind::Get);

        get = String::from("Get");
        operation_kind = OperationKind::from_str(&get).unwrap();
        assert_eq!(operation_kind, OperationKind::Get);
    }
}
