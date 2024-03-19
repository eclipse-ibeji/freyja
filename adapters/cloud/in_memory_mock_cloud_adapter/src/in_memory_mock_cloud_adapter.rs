// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, info};
use tokio::sync::Mutex;

use freyja_common::{
    cloud_adapter::{CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse},
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};

/// Mocks a cloud adapter in memory
pub struct InMemoryMockCloudAdapter {}

#[async_trait]
impl CloudAdapter for InMemoryMockCloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    ///
    /// # Arguments
    /// - `_selector`: the service discovery adapter selector to use (unused by this adapter)
    fn create_new(
        _selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, CloudAdapterError> {
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

    use freyja_test_common::mocks::MockServiceDiscoveryAdapterSelector;

    #[test]
    fn can_get_new() {
        let result = InMemoryMockCloudAdapter::create_new(Arc::new(Mutex::new(
            MockServiceDiscoveryAdapterSelector::new(),
        )));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn can_send_to_cloud() {
        let cloud_adapter = InMemoryMockCloudAdapter::create_new(Arc::new(Mutex::new(
            MockServiceDiscoveryAdapterSelector::new(),
        )))
        .unwrap();

        let cloud_message = CloudMessageRequest {
            metadata: HashMap::new(),
            signal_value: String::from("72"),
            signal_timestamp: OffsetDateTime::now_utc(),
        };

        assert!(cloud_adapter.send_to_cloud(cloud_message).await.is_ok());
    }
}
