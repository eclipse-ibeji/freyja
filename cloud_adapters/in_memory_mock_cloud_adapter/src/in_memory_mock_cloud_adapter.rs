// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs, path::Path};

use async_trait::async_trait;
use log::{debug, info};

use crate::config_item::ConfigItem;
use freyja_contracts::cloud_adapter::{
    CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse,
};

const CONFIG_FILE: &str = "config.json";

/// Mocks a cloud adapter in memory
pub struct InMemoryMockCloudAdapter {
    /// The mock's config
    pub config: ConfigItem,
}

impl InMemoryMockCloudAdapter {
    /// Creates a new InMemoryMockCloudAdapter with config from the specified file
    ///
    /// # Arguments
    ///
    /// - `config_path`: the path to the config to use
    pub fn from_config_file<P: AsRef<Path>>(config_path: P) -> Result<Self, CloudAdapterError> {
        let config_contents = fs::read_to_string(config_path).map_err(CloudAdapterError::io)?;
        let config: ConfigItem = serde_json::from_str(config_contents.as_str())
            .map_err(CloudAdapterError::deserialize)?;

        Self::from_config(config)
    }

    /// Creates a new InMemoryMockCloudAdapter with the specified config
    ///
    /// # Arguments
    ///
    /// - `config_path`: the config to use
    pub fn from_config(config: ConfigItem) -> Result<Self, CloudAdapterError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl CloudAdapter for InMemoryMockCloudAdapter {
    /// Creates a new instance of a CloudAdapter with default settings
    fn create_new() -> Result<Self, CloudAdapterError> {
        Self::from_config_file(Path::new(env!("OUT_DIR")).join(CONFIG_FILE))
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
        let instance_end_point = format!(
            "https://{}/{:?}",
            self.config.host_connection_string, cloud_message.cloud_signal
        );

        debug!(
            "Sending signal update to the {} endpoint:\n{}",
            self.config.cloud_service_name, instance_end_point,
        );

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
    fn from_config_file_returns_err_on_nonexistent_file() {
        let result = InMemoryMockCloudAdapter::from_config_file("fake_file.foo");
        assert!(result.is_err());
    }

    #[test]
    fn can_get_default_config() {
        let result = InMemoryMockCloudAdapter::create_new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn can_send_to_cloud() {
        let cloud_adapter = InMemoryMockCloudAdapter::create_new().unwrap();

        let cloud_message = CloudMessageRequest {
            cloud_signal: HashMap::new(),
            signal_value: String::from("72"),
            signal_timestamp: OffsetDateTime::now_utc().to_string(),
        };

        assert!(cloud_adapter.send_to_cloud(cloud_message).await.is_ok());
    }
}
