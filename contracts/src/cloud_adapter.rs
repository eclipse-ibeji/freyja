// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait CloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Self, CloudAdapterError>
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
    /// A map containing metadata to identify a cloud canonical model signal
    pub cloud_signal: HashMap<String, String>,

    // The signal value
    pub signal_value: String,

    // Timestamp of when the signal was emitted
    pub signal_timestamp: String,
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
