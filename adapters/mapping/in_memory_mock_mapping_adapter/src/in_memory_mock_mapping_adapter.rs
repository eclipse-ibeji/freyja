// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::config::Config;
use freyja_build_common::config_file_stem;

use freyja_common::{
    config_utils,
    mapping_adapter::{
        CheckForWorkRequest, CheckForWorkResponse, GetMappingRequest, GetMappingResponse,
        MappingAdapter, MappingAdapterError,
    },
    out_dir,
    service_discovery_adapter_selector::ServiceDiscoveryAdapterSelector,
};

/// Mocks a mapping provider in memory
pub struct InMemoryMockMappingAdapter {
    /// The mock's config
    config: Config,

    /// An internal counter which controls which mappings are available
    counter: AtomicU8,
}

impl InMemoryMockMappingAdapter {
    /// Creates a new InMemoryMockMappingAdapter with the specified config
    ///
    /// # Arguments
    ///
    /// - `config`: the config to use
    pub fn from_config(config: Config) -> Result<Self, MappingAdapterError> {
        Ok(Self {
            config,
            counter: AtomicU8::new(0),
        })
    }
}

#[async_trait]
impl MappingAdapter for InMemoryMockMappingAdapter {
    /// Creates a new instance of an InMemoryMockMappingAdapter with default settings
    ///
    /// # Arguments
    /// - `_selector`: the service discovery adapter selector to use (unused by this adapter)
    fn create_new(
        _selector: Arc<Mutex<dyn ServiceDiscoveryAdapterSelector>>,
    ) -> Result<Self, MappingAdapterError> {
        let config = config_utils::read_from_files(
            config_file_stem!(),
            config_utils::JSON_EXT,
            out_dir!(),
            MappingAdapterError::io,
            MappingAdapterError::deserialize,
        )?;

        Self::from_config(config)
    }

    /// Checks for any additional work that the mapping service requires.
    /// For example, the cloud digital twin has changed and a new mapping needs to be generated
    /// Increments the internal counter and returns true if this would affect the result of get_mapping compared to the previous call
    async fn check_for_work(
        &self,
        _request: CheckForWorkRequest,
    ) -> Result<CheckForWorkResponse, MappingAdapterError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);

        Ok(CheckForWorkResponse {
            has_work: self.config.values.iter().any(|c| match c.end {
                Some(end) => n == end || n == c.begin,
                None => n == c.begin,
            }),
        })
    }

    /// Gets the mapping from the mapping service
    /// Returns the values that are configured to exist for the current internal count
    async fn get_mapping(
        &self,
        _request: GetMappingRequest,
    ) -> Result<GetMappingResponse, MappingAdapterError> {
        let n = self.counter.load(Ordering::SeqCst);

        Ok(GetMappingResponse {
            map: self
                .config
                .values
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
mod in_memory_mock_mapping_adapter_tests {
    use super::*;

    use std::collections::HashMap;

    use freyja_common::{conversion::Conversion, digital_twin_map_entry::DigitalTwinMapEntry};
    use freyja_test_common::mocks::MockServiceDiscoveryAdapterSelector;

    use crate::config::ConfigItem;

    #[test]
    fn can_create_new() {
        let result = InMemoryMockMappingAdapter::create_new(Arc::new(Mutex::new(
            MockServiceDiscoveryAdapterSelector::new(),
        )));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_for_work_returns_correct_values() {
        let config = Config {
            values: vec![
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
            ],
        };

        let uut = InMemoryMockMappingAdapter::from_config(config).unwrap();

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
        let config = Config {
            values: vec![
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
            ],
        };

        let uut = InMemoryMockMappingAdapter::from_config(config).unwrap();

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
}
