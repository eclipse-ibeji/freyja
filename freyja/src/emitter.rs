// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{cmp::min, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use log::{info, warn};
use time::OffsetDateTime;
use tokio::time::sleep;

use freyja_common::signal_store::SignalStore;
use freyja_contracts::{
    cloud_adapter::{CloudAdapter, CloudMessageRequest, CloudMessageResponse},
    provider_proxy::SignalValue,
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
    signal::Signal,
};

/// Data emitter for the digital twin sync project
/// Emits sensor data at regular intervals as defined by the map
pub struct Emitter {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The cloud adapter used to emit data to the cloud
    cloud_adapter: Box<dyn CloudAdapter + Sync + Send>,

    /// Sends requests to the provider proxy selector
    provider_proxy_selector_request_sender: ProviderProxySelectorRequestSender,

    /// Shared message queue for obtaining new signal values
    signal_values_queue: Arc<SegQueue<SignalValue>>,
}

impl Emitter {
    /// Creates a new instance of emitter
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `cloud_adapter`: the cloud adapter used to emit to the cloud
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    /// - `signal_values_queue`: queue for receiving signal values
    pub fn new(
        signals: Arc<SignalStore>,
        cloud_adapter: Box<dyn CloudAdapter + Sync + Send>,
        provider_proxy_selector_request_sender: ProviderProxySelectorRequestSender,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Self {
        Self {
            signals,
            cloud_adapter,
            provider_proxy_selector_request_sender,
            signal_values_queue,
        }
    }

    /// Execute this Emitter
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        const DEFAULT_SLEEP_INTERVAL_MS: u64 = 1000;
        loop {
            self.update_signal_values();

            let signals = self.signals.get_all();
            let mut sleep_interval = u64::MAX;

            if signals.is_empty() {
                sleep_interval = DEFAULT_SLEEP_INTERVAL_MS;
            } else {
                info!("********************BEGIN EMISSION********************");

                for signal in signals {
                    // Submit a request for a new value for the next iteration.
                    // This approach to requesting signal values introduces an inherent delay in uploading data
                    // and needs to be revisited.
                    let request = ProviderProxySelectorRequestKind::GetEntityValue {
                        entity_id: signal.id.clone(),
                    };
                    self.provider_proxy_selector_request_sender
                        .send_request_to_provider_proxy_selector(request);

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

                    self.send_to_cloud(signal).await?;
                }

                info!("*********************END EMISSION*********************");

                // Update the emission times for the next loop.
                // Note that the signal set could actually have changed since we originally queried them!
                // Updating new signals will be a no-op since they get initialized with next_emission_ms = 0.
                // If all of the signals corresponding to this value were removed from tracking
                // then the loop will wake up early and emit nothing after sleeping for this interval.
                self.signals.update_next_emission_times(sleep_interval);
            }

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

    /// Applies conversion implicitly to a signal value and sends it to the cloud
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
        CloudError,
    }
}
