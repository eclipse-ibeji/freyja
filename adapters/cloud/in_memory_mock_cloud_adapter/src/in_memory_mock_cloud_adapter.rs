// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use log::{debug, info};

use freyja_common::cloud_adapter::{
    CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse,
};

/// Mocks a cloud adapter in memory
pub struct InMemoryMockCloudAdapter {}

#[async_trait]
impl CloudAdapter for InMemoryMockCloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Self, CloudAdapterError> {
        Ok(Self {})
    }

    /// Sends the signal to the cloud
    ///
    /// # Arguments
    ///
    /// - `cloud_message`: represents a message to send to the cloud canonical model
    async fn send_to_cloud(
        &self,
        cloud_message: CloudMessageRequest,
    ) -> Result<CloudMessageResponse, CloudAdapterError> {
        debug!("Received a request to send to the cloud");
        let cloud_message_json =
            serde_json::to_string_pretty(&cloud_message).map_err(CloudAdapterError::serialize)?;

        info!("Cloud canonical value:\n{cloud_message_json}");

        Ok(CloudMessageResponse {})
    }
}

#[cfg(test)]
mod in_memory_mock_cloud_adapter_tests {
    use super::*;

    use std::collections::HashMap;

    use time::OffsetDateTime;

    #[test]
    fn can_get_new() {
        let result = InMemoryMockCloudAdapter::create_new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn can_send_to_cloud() {
        let cloud_adapter = InMemoryMockCloudAdapter::create_new().unwrap();

        let cloud_message = CloudMessageRequest {
            metadata: HashMap::new(),
            signal_value: String::from("72"),
            signal_timestamp: OffsetDateTime::now_utc().to_string(),
        };

        assert!(cloud_adapter.send_to_cloud(cloud_message).await.is_ok());
    }
}
