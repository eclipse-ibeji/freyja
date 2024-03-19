// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector;

#[async_trait]
pub trait CloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    ///
    /// # Arguments
    /// - `selector`: the service discovery adapter selector to use
    fn create_new(
        selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, CloudAdapterError>
    where
        Self: Sized;

    /// Sends the signal to the cloud
    ///
    /// # Arguments
    /// - `cloud_message`: represents a message to send to the cloud canonical model
    async fn send_to_cloud(
        &self,
        cloud_message: CloudMessageRequest,
    ) -> Result<CloudMessageResponse, CloudAdapterError>;
}

/// Represents a message to send to the cloud canonical model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMessageRequest {
    /// A map containing metadata to help identify the signal in the cloud
    pub metadata: HashMap<String, String>,

    // The signal value
    pub signal_value: String,

    // Timestamp of when the signal was emitted
    pub signal_timestamp: OffsetDateTime,
}

/// Represents a response to a message sent to the cloud digital twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMessageResponse {}

proc_macros::error! {
    CloudAdapterError {
        Io,
        Serialize,
        Deserialize,
        Communication,
        KeyNotFound,
        Unknown
    }
}
