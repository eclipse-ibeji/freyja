// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use log::{debug, info, warn};

use freyja_common::signal_store::SignalStore;
use freyja_common::{
    conversion::Conversion,
    data_adapter_selector::DataAdapterSelector,
    digital_twin_adapter::{
        DigitalTwinAdapter, DigitalTwinAdapterError, DigitalTwinAdapterErrorKind, FindByIdRequest,
    },
    mapping_adapter::{CheckForWorkRequest, GetMappingRequest, MappingAdapter},
    signal::{EmissionPolicy, SignalPatch, Target},
};

/// Manages mappings from the mapping service
pub struct Cartographer<TMappingAdapter, TDigitalTwinAdapter, TDataAdapterSelector> {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The mapping adapter
    mapping_adapter: TMappingAdapter,

    /// The digital twin adapter
    digital_twin_adapter: TDigitalTwinAdapter,

    /// The data adapter selector
    data_adapter_selector: Arc<Mutex<TDataAdapterSelector>>,

    /// The mapping service polling interval
    poll_interval: Duration,
}

impl<
        TMappingAdapter: MappingAdapter,
        TDigitalTwinAdapter: DigitalTwinAdapter,
        TDataAdapterSelector: DataAdapterSelector,
    > Cartographer<TMappingAdapter, TDigitalTwinAdapter, TDataAdapterSelector>
{
    /// Create a new instance of a Cartographer
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `mapping_adapter`: the adapter for the mapping service
    /// - `digital_twin_adapter`: the adapter for the digital twin service
    /// - `data_adapter_selector`: the data adapter selector
    /// - `poll_interval`: the interval at which the cartographer should poll for changes
    pub fn new(
        signals: Arc<SignalStore>,
        mapping_adapter: TMappingAdapter,
        digital_twin_adapter: TDigitalTwinAdapter,
        data_adapter_selector: Arc<Mutex<TDataAdapterSelector>>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            signals,
            mapping_adapter,
            digital_twin_adapter,
            data_adapter_selector,
            poll_interval,
        }
    }

    /// Run the cartographer. This will do the following in a loop:
    ///
    /// 1. Check to see if the mapping service has more work
    ///     - If there is work, do the following:
    ///         1. Clear the list of previously failed signals
    ///         1. ~~Send the new inventory to the mapping service~~
    ///         1. Get the new mapping from the mapping service
    ///         1. Query the digital twin service for entity information
    ///         1. Create or update data adapters for the new entities
    ///         1. Update the signal store with the new data and track any failed signals for future iterations
    ///     - If there is no work but previous attempts to start up adapters failed,
    ///         execute the steps above starting from step 4 for these failed cases
    ///     - If the check failed, log the error
    /// 1. Sleep until the next iteration
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut failed_signals: Vec<SignalPatch> = Vec::new();
        loop {
            let mut successes = Vec::new();

            // Check for new work from the mapping service
            match self
                .mapping_adapter
                .check_for_work(CheckForWorkRequest {})
                .await
            {
                Ok(r) if r.has_work => {
                    info!("Cartographer detected mapping work");

                    match self.get_mapping_as_signal_patches().await {
                        Ok(p) => {
                            // We clear the failed signals here because the incoming mapping is used as the source of truth,
                            // so anything left over from previous mappings shouldn't get used.
                            failed_signals.clear();
                            self.process_signal_patches(&p, &mut successes, &mut failed_signals)
                                .await;
                            self.signals.sync(successes.into_iter());
                        }
                        Err(e) => log::error!("Failed to get mapping from mapping adapter: {e}"),
                    }
                }
                Ok(_) if !failed_signals.is_empty() => {
                    info!("No new mappings found, but some mappings failed to be created in previous iterations");

                    // Retry previously failed signals
                    let mut failures = Vec::new();
                    self.process_signal_patches(&failed_signals, &mut successes, &mut failures)
                        .await;

                    self.signals.add(successes.into_iter());
                    failed_signals = failures;
                }
                Ok(_) => debug!("No work for cartographer"),
                Err(e) => log::error!(
                    "Failed to check for mapping work; will try again later. Error: {e}"
                ),
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Processes a list of signal patches by calling `populate_source` for each one.
    /// The signals for which this call succeeds are pushed into `successes`, while others are put into `failures`.
    ///
    /// # Arguments
    /// - `patches`: the list of signal patches to process
    /// - `successes`: the list to update with successful signals
    /// - `failures`: the list to update with failed signals
    async fn process_signal_patches(
        &self,
        patches: &[SignalPatch],
        successes: &mut Vec<SignalPatch>,
        failures: &mut Vec<SignalPatch>,
    ) {
        for patch in patches.iter() {
            // Many of the API calls in populate_entity are probably unnecessary, but this code gets executed
            // infrequently enough that the sub-optimal performance is not a major concern.
            // A bulk find_by_id API in the digital twin service would make this a non-issue
            let mut patch = patch.clone();
            match self.populate_source(&mut patch).await {
                Ok(_) => successes.push(patch),
                Err(e) => {
                    match e.downcast::<DigitalTwinAdapterError>() {
                        Ok(e) if e.kind() == DigitalTwinAdapterErrorKind::EntityNotFound => {
                            warn!("Entity not found for signal {}", patch.id);
                        }
                        Ok(e) => {
                            log::error!("Error fetching entity for signal {}: {e:?}", patch.id);
                        }
                        Err(e) => {
                            log::error!("Error fetching entity for signal {}: {e:?}", patch.id);
                        }
                    }

                    failures.push(patch);
                }
            }
        }
    }

    /// Gets the mapping from the mapping adapter and returns a corresponding list of signal patches.
    async fn get_mapping_as_signal_patches(
        &self,
    ) -> Result<Vec<SignalPatch>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .mapping_adapter
            .get_mapping(GetMappingRequest {})
            .await?
            .map
            .into_iter()
            .map(|(id, entry)| SignalPatch {
                id,
                // this gets populated later, set to default for now
                source: Default::default(),
                target: Target {
                    metadata: entry.target,
                },
                emission_policy: EmissionPolicy {
                    interval_ms: entry.interval_ms,
                    emit_only_if_changed: entry.emit_on_change,
                    conversion: Conversion::default(),
                },
            })
            .collect())
    }

    /// Populates the source of the provided signal with data retrieved from the digital twin service.
    /// This will also create or update a data adapter to handle incoming requests from the provider.
    ///
    /// Arguments
    /// - `signal`: The signal patch to update
    async fn populate_source(
        &self,
        signal: &mut SignalPatch,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        signal.source = self
            .digital_twin_adapter
            .find_by_id(FindByIdRequest {
                entity_id: signal.id.clone(),
            })
            .await?
            .entity;

        {
            let data_adapter_selector = self.data_adapter_selector.lock().await;
            data_adapter_selector
                .create_or_update_adapter(&signal.source)
                .await
                .map_err(|e| format!("Error sending request to data adapter selector: {e:?}"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod cartographer_tests {
    use super::*;

    use std::collections::HashMap;

    use freyja_common::{
        digital_twin_adapter::FindByIdResponse,
        digital_twin_map_entry::DigitalTwinMapEntry,
        entity::{Entity, EntityEndpoint},
        mapping_adapter::GetMappingResponse,
    };
    use freyja_test_common::{
        mockall::predicate::eq,
        mocks::{MockDataAdapterSelector, MockDigitalTwinAdapter, MockMappingAdapter},
    };

    #[tokio::test]
    async fn get_mapping_as_signals_returns_correct_value() {
        const ID: &str = "testid";
        let test_map_entry = DigitalTwinMapEntry {
            source: ID.to_string(),
            target: HashMap::new(),
            interval_ms: 42,
            conversion: Default::default(),
            emit_on_change: true,
        };

        let test_map_entry_clone = test_map_entry.clone();

        let mut mock_mapping_adapter = MockMappingAdapter::new();
        mock_mapping_adapter
            .expect_get_mapping()
            .returning(move |_| {
                Ok(GetMappingResponse {
                    map: [(ID.to_string(), test_map_entry_clone.clone())]
                        .into_iter()
                        .collect(),
                })
            });

        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_adapter: mock_mapping_adapter,
            digital_twin_adapter: MockDigitalTwinAdapter::new(),
            data_adapter_selector: Arc::new(Mutex::new(MockDataAdapterSelector::new())),
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.get_mapping_as_signal_patches().await;

        assert!(result.is_ok());
        let mut signals = result.unwrap();
        assert_eq!(signals.len(), 1);
        let signal = signals.pop().unwrap();
        assert_eq!(signal.id, ID.to_string());
        assert_eq!(signal.target.metadata, test_map_entry.target);
        assert_eq!(
            signal.emission_policy.interval_ms,
            test_map_entry.interval_ms
        );
        assert_eq!(
            signal.emission_policy.emit_only_if_changed,
            test_map_entry.emit_on_change
        );
        assert_eq!(signal.emission_policy.conversion, test_map_entry.conversion);
    }

    #[tokio::test]
    async fn populate_source_tests() {
        const ID: &str = "testid";
        let test_entity = Entity {
            id: ID.to_string(),
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            endpoints: vec![EntityEndpoint {
                operations: vec!["FooOperation".to_string()],
                protocol: "in-memory".to_string(),
                uri: "uri".to_string(),
                context: "context".to_string(),
            }],
        };

        let test_signal_patch = &mut SignalPatch {
            id: ID.to_string(),
            ..Default::default()
        };

        let test_entity_clone = test_entity.clone();

        let mut mock_data_adapter_selector = MockDataAdapterSelector::new();
        mock_data_adapter_selector
            .expect_create_or_update_adapter()
            .with(eq(test_entity.clone()))
            .once()
            .returning(|_| Ok(()));
        let data_adapter_selector = Arc::new(Mutex::new(mock_data_adapter_selector));

        let mut mock_dt_adapter = MockDigitalTwinAdapter::new();
        mock_dt_adapter.expect_find_by_id().returning(move |_| {
            Ok(FindByIdResponse {
                entity: test_entity_clone.clone(),
            })
        });

        let uut = Cartographer {
            signals: Arc::new(SignalStore::new()),
            mapping_adapter: MockMappingAdapter::new(),
            digital_twin_adapter: mock_dt_adapter,
            data_adapter_selector,
            poll_interval: Duration::from_secs(1),
        };

        let result = uut.populate_source(test_signal_patch).await;

        uut.data_adapter_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(test_signal_patch.source, test_entity);
    }
}
