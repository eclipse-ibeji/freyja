// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use freyja_common::signal_store::SignalStore;
use log::info;

use freyja_contracts::{mapping_client::{CheckForWorkRequest, GetMappingRequest, MappingClient}, signal::{Signal, Target, EmissionPolicy, Emission}};

/// Manages mappings from the mapping service
pub struct Cartographer {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The mapping client
    mapping_client: Box<dyn MappingClient>,

    /// The mapping service polling interval
    poll_interval: Duration,
}

impl Cartographer {
    /// Create a new instance of a Cartographer
    ///
    /// # Arguments
    /// - `signals`: the shared signal store
    /// - `mapping_client`: the client for the mapping service
    /// - `poll_interval`: the interval at which the cartographer should poll for changes
    pub fn new(
        signals: Arc<SignalStore>,
        mapping_client: Box<dyn MappingClient>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            signals,
            mapping_client,
            poll_interval,
        }
    }

    /// Run the cartographer. This will do the following in a loop:
    ///
    /// 1. Check to see if the mapping service has more work. If not, skip to step 5
    /// 2. Send the new inventory to the mapping service
    /// 3. Get the new mapping from the mapping service
    /// 4. Update the shared map for the emitter
    /// 5. Sleep until the next iteration
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let mapping_client_result = self
                .mapping_client
                .check_for_work(CheckForWorkRequest {})
                .await;

            let mapping_work = mapping_client_result?;

            if mapping_work.has_work {
                info!("Cartographer detected mapping work");

                // TODO: will this notion of checking and sending inventory exist?
                // self.mapping_client.send_inventory(SendInventoryRequest { inventory: self.known_providers.clone() }).await?;

                // TODO: waiting/retry logic?
                let signals = self
                    .mapping_client
                    .get_mapping(GetMappingRequest {})
                    .await?
                    .map
                    .into_iter()
                    .map(|(id, entry)| Signal {
                        id,
                        target: Target {
                            metadata: entry.target,
                        },
                        emission: Emission {
                            policy: EmissionPolicy {
                                interval_ms: entry.interval_ms,
                                emit_on_change: entry.emit_on_change,
                                conversion: entry.conversion,
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                self.signals.do_the_thing(signals);
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
