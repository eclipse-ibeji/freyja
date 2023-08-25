// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{env, fs};

use async_trait::async_trait;

use crate::config::ConfigItem;

use freyja_contracts::mapping_client::*;

const CONFIG_FILE: &str = "config.json";

/// Mocks a mapping provider in memory
pub struct InMemoryMockMappingClient {
    /// The mock's config
    config: Vec<ConfigItem>,

    /// An internal counter which controls which mappings are available
    counter: AtomicU8,
}

impl InMemoryMockMappingClient {
    /// Creates a new InMemoryMockMappingClient with config from the specified file
    ///
    /// # Arguments
    ///
    /// - `config_path`: the path to the config to use
    pub fn from_config_file<P: AsRef<Path>>(config_path: P) -> Result<Self, MappingClientError> {
        let config_contents = fs::read_to_string(config_path).map_err(MappingClientError::io)?;
        let config: Vec<ConfigItem> = serde_json::from_str(config_contents.as_str())
            .map_err(MappingClientError::deserialize)?;

        Self::from_config(config)
    }

    /// Creates a new InMemoryMockMappingClient with the specified config
    ///
    /// # Arguments
    ///
    /// - `config_path`: the config to use
    pub fn from_config(config: Vec<ConfigItem>) -> Result<Self, MappingClientError> {
        Ok(Self {
            config,
            counter: AtomicU8::new(0),
        })
    }
}

#[async_trait]
impl MappingClient for InMemoryMockMappingClient {
    /// Creates a new instance of an InMemoryMockMappingClient with default settings
    fn create_new() -> Result<Box<dyn MappingClient>, MappingClientError> {
        Self::from_config_file(Path::new(env!("OUT_DIR")).join(CONFIG_FILE))
            .map(|r| Box::new(r) as _)
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
    async fn check_for_work(
        &self,
        _request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingClientError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);

        Ok(CheckForWorkResponse {
            has_work: self.config.iter().any(|c| match c.end {
                Some(end) => n == end || n == c.begin,
                None => n == c.begin,
            }),
        })
    }

    /// Sends the provider inventory to the mapping service
    ///
    /// # Arguments
    ///
    /// - `inventory`: the providers to send
    async fn send_inventory(
        &self,
        _inventory: SendInventoryRequest,
    ) -> Result<SendInventoryResponse, MappingClientError> {
        Ok(SendInventoryResponse {})
    }

    /// Gets the mapping from the mapping service
    /// Returns the values that are configured to exist for the current internal count
    async fn get_mapping(
        &self,
        _request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingClientError> {
        let n = self.counter.load(Ordering::SeqCst);

        Ok(GetMappingResponse {
            map: self
                .config
                .iter()
                .filter_map(|c| match c.end {
                    Some(end) if n >= c.begin && n < end => {
                        Some((c.value.source.clone(), c.value.clone()))
                    }
                    None if n >= c.begin => Some((c.value.source.clone(), c.value.clone())),
                    _ => None,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod in_memory_mock_mapping_client_tests {
    use super::*;

    use std::collections::{HashMap, HashSet};

    use freyja_contracts::{conversion::Conversion, digital_twin_map_entry::DigitalTwinMapEntry};

    #[test]
    fn from_config_file_returns_err_on_nonexistent_file() {
        let result = InMemoryMockMappingClient::from_config_file("fake_file.foo");
        assert!(result.is_err());
    }

    #[test]
    fn can_get_default_config() {
        let result = InMemoryMockMappingClient::create_new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_for_work_returns_correct_values() {
        let config = vec![
            ConfigItem {
                begin: 0,
                end: None,
                value: DigitalTwinMapEntry {
                    source: String::from("always-active"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
            ConfigItem {
                begin: 10,
                end: None,
                value: DigitalTwinMapEntry {
                    source: String::from("delayed-activaction"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
            ConfigItem {
                begin: 0,
                end: Some(20),
                value: DigitalTwinMapEntry {
                    source: String::from("not-always-active"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
        ];

        let uut = InMemoryMockMappingClient::from_config(config).unwrap();

        for i in 0..30 {
            let result = uut
                .check_for_work(CheckForWorkRequest {})
                .await
                .unwrap()
                .has_work;
            assert!(match i {
                0 | 10 | 20 => result,
                _ => !result,
            });
        }
    }

    #[tokio::test]
    async fn get_mapping_returns_correct_values() {
        let config = vec![
            ConfigItem {
                begin: 0,
                end: None,
                value: DigitalTwinMapEntry {
                    source: String::from("always-active"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
            ConfigItem {
                begin: 10,
                end: None,
                value: DigitalTwinMapEntry {
                    source: String::from("delayed-activation"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
            ConfigItem {
                begin: 0,
                end: Some(20),
                value: DigitalTwinMapEntry {
                    source: String::from("not-always-active"),
                    target: HashMap::new(),
                    interval_ms: 0,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            },
        ];

        let uut = InMemoryMockMappingClient::from_config(config).unwrap();

        for _ in 0..9 {
            uut.check_for_work(CheckForWorkRequest {})
                .await
                .expect("check_for_work failed");
            let mapping = uut.get_mapping(GetMappingRequest {}).await.unwrap().map;
            assert_eq!(2, mapping.len());
            assert!(mapping.iter().any(|p| *p.0 == "always-active"));
            assert!(!mapping.iter().any(|p| *p.0 == "delayed-activation"));
            assert!(mapping.iter().any(|p| *p.0 == "not-always-active"));
        }

        for _ in 10..20 {
            uut.check_for_work(CheckForWorkRequest {})
                .await
                .expect("check_for_work failed");
            let mapping = uut.get_mapping(GetMappingRequest {}).await.unwrap().map;
            assert_eq!(3, mapping.len());
            assert!(mapping.iter().any(|p| *p.0 == "always-active"));
            assert!(mapping.iter().any(|p| *p.0 == "delayed-activation"));
            assert!(mapping.iter().any(|p| *p.0 == "not-always-active"));
        }

        for _ in 21..30 {
            uut.check_for_work(CheckForWorkRequest {})
                .await
                .expect("check_for_work failed");
            let mapping = uut.get_mapping(GetMappingRequest {}).await.unwrap().map;
            assert_eq!(2, mapping.len());
            assert!(mapping.iter().any(|p| *p.0 == "always-active"));
            assert!(mapping.iter().any(|p| *p.0 == "delayed-activation"));
            assert!(!mapping.iter().any(|p| *p.0 == "not-always-active"));
        }
    }

    #[tokio::test]
    async fn send_inventory_is_ok() {
        let uut = InMemoryMockMappingClient::create_new().unwrap();
        assert!(uut
            .send_inventory(SendInventoryRequest {
                inventory: HashSet::new()
            })
            .await
            .is_ok());
    }
}
