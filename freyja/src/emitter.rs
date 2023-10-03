// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{cmp::min, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use log::{info, warn};
use time::OffsetDateTime;
use tokio::{time::sleep, sync::Mutex};

use freyja_common::signal_store::SignalStore;
use freyja_contracts::{
    cloud_adapter::{CloudAdapter, CloudMessageRequest, CloudMessageResponse},
    provider_proxy::SignalValue,
    signal::Signal, provider_proxy_selector::ProviderProxySelector,
};

const DEFAULT_SLEEP_INTERVAL_MS: u64 = 1000;

/// Emits sensor data at regular intervals as configured in the store
pub struct Emitter<TCloudAdapter, TProviderProxySelector> {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The cloud adapter used to emit data to the cloud
    cloud_adapter: TCloudAdapter,

    /// The provider proxy selector
    provider_proxy_selector: Arc<Mutex<TProviderProxySelector>>,

    /// Shared message queue for obtaining new signal values
    signal_values_queue: Arc<SegQueue<SignalValue>>,
}

impl<TCloudAdapter: CloudAdapter, TProviderProxySelector: ProviderProxySelector> Emitter<TCloudAdapter, TProviderProxySelector> {
    /// Creates a new instance of the Emitter
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `cloud_adapter`: the cloud adapter used to emit to the cloud
    /// - `provider_proxy_selector`: the provider proxy selector
    /// - `signal_values_queue`: queue for receiving signal values
    pub fn new(
        signals: Arc<SignalStore>,
        cloud_adapter: TCloudAdapter,
        provider_proxy_selector: Arc<Mutex<TProviderProxySelector>>,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Self {
        Self {
            signals,
            cloud_adapter,
            provider_proxy_selector,
            signal_values_queue,
        }
    }

    /// Execute this Emitter
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sleep_interval = u64::MAX;
        loop {
            self.update_signal_values();

            // Update the emission times and get the list of all signals.
            // This is performed as a single operation to minimize the impact of changes to the signal set during processing.
            // Note that the first time the loop is executed sleep_interval will still be u64::MAX,
            // which will have the effect of force-emitting every signal in the store (though typically there won't be anything).
            // After that, the intervals will be no more than the max configured interval.
            let signals = self
                .signals
                .update_emission_times_and_get_all(sleep_interval);

            sleep_interval = self.emit_data(signals).await?;

            info!("Checking for next emission in {sleep_interval}ms\n");
            sleep(Duration::from_millis(sleep_interval)).await;
        }
    }

    /// Updates the signal values map.
    /// This will eventually get removed and provider proxies will update the store directly,
    /// but it remains temporarily to scope work down a bit.
    fn update_signal_values(&self) {
        while !self.signal_values_queue.is_empty() {
            let SignalValue { entity_id, value } = self.signal_values_queue.pop().unwrap();
            if self.signals.set_value(entity_id.clone(), value).is_none() {
                warn!("Attempted to update signal {entity_id} but it wasn't found")
            }
        }
    }

    /// Performs data emissions of the provided signals.
    /// Returns the amount of time that the main emitter loop should sleep before the next iteration.
    ///
    /// # Arguments
    /// - `signals`: The set of signals to emit
    async fn emit_data(&self, signals: Vec<Signal>) -> Result<u64, EmitterError> {
        if signals.is_empty() {
            Ok(DEFAULT_SLEEP_INTERVAL_MS)
        } else {
            info!("********************BEGIN EMISSION********************");
            let mut sleep_interval = u64::MAX;

            for signal in signals {
                if signal.emission.next_emission_ms > 0 {
                    // Don't emit this signal on this iteration, but use the value to update the sleep interval
                    sleep_interval = min(sleep_interval, signal.emission.next_emission_ms);

                    // Go to next signal
                    continue;
                } else {
                    // We will emit this signal since the timer is expired,
                    // but need to also check the new interval in case it's smaller than the remaining intervals
                    sleep_interval = min(sleep_interval, signal.emission.policy.interval_ms);
                }

                // Submit a request for a new value for the next iteration.
                // This approach to requesting signal values introduces an inherent delay in uploading data
                // of signal.emission.policy.interval_ms and needs to be revisited.
                let proxy_result = {
                    let mut provider_proxy_selector = self.provider_proxy_selector.lock().await;
                    provider_proxy_selector
                        .request_entity_value(&signal.id)
                        .await
                        .map_err(EmitterError::provider_proxy_error)
                };

                if proxy_result.is_err() {
                    log::error!("Error submitting request for signal value while processing signal {}: {:?}", signal.id, proxy_result.err());
                }

                if signal.value.is_none() {
                    info!(
                        "No signal value for {} in our cache. Skipping emission for this signal.",
                        signal.id
                    );

                    // Go to the next signal
                    continue;
                }

                if signal.emission.policy.emit_only_if_changed
                    && signal.emission.last_emitted_value.is_some()
                    && signal.value == signal.emission.last_emitted_value
                {
                    info!("Signal {} did not change and has already been emitted. Skipping emission for this signal.", signal.id);

                    // Go to next signal
                    continue;
                }

                let signal_id = signal.id.clone();
                let send_to_cloud_result = self.send_to_cloud(signal).await;

                if send_to_cloud_result.is_err() {
                    log::error!(
                        "Error sending data to cloud while processing signal {}: {:?}",
                        signal_id,
                        send_to_cloud_result.err()
                    );
                }
            }

            info!("*********************END EMISSION*********************");

            Ok(sleep_interval)
        }
    }

    /// Applies a conversion implicitly to a signal value and sends it to the cloud
    ///
    /// # Arguments
    /// - `signal`: The signal to emit
    async fn send_to_cloud(&self, signal: Signal) -> Result<CloudMessageResponse, EmitterError> {
        let value = signal
            .value
            .clone()
            // This error case should actually be unreachable, but always good to check!
            .ok_or::<EmitterError>(EmitterErrorKind::SignalValueEmpty.into())?;

        let converted = value.parse::<f32>().map_or(value.clone(), |v| {
            signal.emission.policy.conversion.apply(v).to_string()
        });

        info!(
            "Digital Twin Instance {:?}: {}",
            signal.target.metadata, converted
        );

        info!("\t(from {}: {:?})", signal.source.id, signal.value);

        let cloud_message = CloudMessageRequest {
            cloud_signal: signal.target.metadata.clone(),
            signal_value: converted,
            signal_timestamp: OffsetDateTime::now_utc().to_string(),
        };

        let response = self
            .cloud_adapter
            .send_to_cloud(cloud_message)
            .await
            .map_err(EmitterError::cloud_error)?;

        // We don't set the last emitted value to the converted value so that we can meaningfully compare
        // this value with the value coming directly from the signal.
        self.signals.set_last_emitted_value(signal.id, value);

        Ok(response)
    }
}

proc_macros::error! {
    EmitterError {
        SignalValueEmpty,
        ProviderProxyError,
        CloudError,
    }
}

#[cfg(test)]
mod emitter_tests {
    use super::*;
    use mockall::*;

    use async_trait::async_trait;

    use freyja_contracts::{
        cloud_adapter::{CloudAdapterError, CloudAdapterErrorKind},
        signal::{Emission, EmissionPolicy}, entity::Entity, provider_proxy_selector::ProviderProxySelectorError,
    };

    mock! {
        pub CloudAdapter {}

        #[async_trait]
        impl CloudAdapter for CloudAdapter {
            fn create_new() -> Result<Self, CloudAdapterError>
            where
                Self: Sized;

            async fn send_to_cloud(
                &self,
                cloud_message: CloudMessageRequest,
            ) -> Result<CloudMessageResponse, CloudAdapterError>;
        }
    }

    mock! {
        pub ProviderProxySelector {}

        #[async_trait]
        impl ProviderProxySelector for ProviderProxySelector {
            async fn create_or_update_proxy(&mut self, entity: &Entity) -> Result<(), ProviderProxySelectorError>;
            async fn request_entity_value(&mut self, entity_id: &String) -> Result<(), ProviderProxySelectorError>;
        }
    }

    #[tokio::test]
    async fn emit_data_returns_default_on_empty_input() {
        let uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: MockCloudAdapter::new(),
            provider_proxy_selector: Arc::new(Mutex::new(MockProviderProxySelector::new())),
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let result = uut.emit_data(vec![]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DEFAULT_SLEEP_INTERVAL_MS);
    }

    #[tokio::test]
    async fn emit_data_handles_nonzero_next_emission_time() {
        const NEXT_EMISSION_MS: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector.expect_request_entity_value().never();
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter.expect_send_to_cloud().never();

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            emission: Emission {
                next_emission_ms: NEXT_EMISSION_MS,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NEXT_EMISSION_MS);
    }

    #[tokio::test]
    async fn emit_data_handles_zero_next_emission_time() {
        const INTERVAL: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter
            .expect_send_to_cloud()
            .once()
            .returning(|_| Ok(CloudMessageResponse {}));

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            value: Some("foo".to_string()),
            emission: Emission {
                next_emission_ms: 0,
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), INTERVAL);
    }

    #[tokio::test]
    async fn emit_data_doesnt_emit_when_value_empty() {
        const INTERVAL: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter.expect_send_to_cloud().never();

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            value: None,
            emission: Emission {
                next_emission_ms: 0,
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), INTERVAL);
    }

    #[tokio::test]
    async fn emit_data_doesnt_emit_when_value_not_changed() {
        const INTERVAL: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter.expect_send_to_cloud().never();

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let value = Some("foo".to_string());
        let test_signal = Signal {
            value: value.clone(),
            emission: Emission {
                next_emission_ms: 0,
                last_emitted_value: value,
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    emit_only_if_changed: true,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), INTERVAL);
    }

    #[tokio::test]
    async fn emit_data_emits_when_value_changed() {
        const INTERVAL: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter
            .expect_send_to_cloud()
            .once()
            .returning(|_| Ok(CloudMessageResponse {}));

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            value: Some("foo".to_string()),
            emission: Emission {
                next_emission_ms: 0,
                last_emitted_value: Some("bar".to_string()),
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    emit_only_if_changed: true,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), INTERVAL);
    }

    #[tokio::test]
    async fn emit_data_emits_when_last_value_empty() {
        const INTERVAL: u64 = 42;

        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .once()
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter
            .expect_send_to_cloud()
            .once()
            .returning(|_| Ok(CloudMessageResponse {}));

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            value: Some("foo".to_string()),
            emission: Emission {
                next_emission_ms: 0,
                last_emitted_value: None,
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    emit_only_if_changed: true,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), INTERVAL);
    }

    #[tokio::test]
    async fn cloud_adapter_error_doesnt_prevent_further_emission_attempts() {
        let mut mock_provider_proxy_selector = MockProviderProxySelector::new();
        mock_provider_proxy_selector
            .expect_request_entity_value()
            .times(2)
            .returning(|_| Ok(()));
        let provider_proxy_selector = Arc::new(Mutex::new(mock_provider_proxy_selector));

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter
            .expect_send_to_cloud()
            .times(2)
            .returning(|_| Err(CloudAdapterErrorKind::Unknown.into()));

        let mut uut = Emitter {
            signals: Arc::new(SignalStore::new()),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector,
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let test_signal = Signal {
            value: Some("foo".to_string()),
            ..Default::default()
        };

        let result = uut.emit_data(vec![test_signal.clone(), test_signal]).await;

        uut.cloud_adapter.checkpoint();
        uut.provider_proxy_selector.lock().await.checkpoint();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_to_cloud_updates_signal_store() {
        const ID: &str = "testid";
        const INTERVAL: u64 = 42;

        let mut mock_cloud_adapter = MockCloudAdapter::new();
        mock_cloud_adapter
            .expect_send_to_cloud()
            .returning(|_| Ok(CloudMessageResponse {}));

        let test_signal = Signal {
            id: ID.to_string(),
            value: Some("foo".to_string()),
            emission: Emission {
                policy: EmissionPolicy {
                    interval_ms: INTERVAL,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let signals = SignalStore::new();
        signals.sync([test_signal.clone()].into_iter());

        let uut = Emitter {
            signals: Arc::new(signals),
            cloud_adapter: mock_cloud_adapter,
            provider_proxy_selector: Arc::new(Mutex::new(MockProviderProxySelector::new())),
            signal_values_queue: Arc::new(SegQueue::new()),
        };

        let result = uut.send_to_cloud(test_signal).await;

        assert!(result.is_ok());

        // Ideally the signal store should be mockable so we can just verify call count
        let signal = uut.signals.get(&ID.to_string());
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert!(signal.emission.last_emitted_value.is_some());
        assert_eq!(signal.emission.next_emission_ms, INTERVAL);
    }
}
