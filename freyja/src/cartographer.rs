// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use freyja_common::signal_store::SignalStore;
use log::info;

use freyja_contracts::{
    conversion::Conversion,
    digital_twin_adapter::{DigitalTwinAdapter, GetDigitalTwinProviderRequest},
    mapping_client::{CheckForWorkRequest, GetMappingRequest, MappingClient},
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
    signal::{Emission, EmissionPolicy, Signal, Target},
};

/// Manages mappings from the mapping service
pub struct Cartographer {
    /// The shared signal store
    signals: Arc<SignalStore>,

    /// The mapping client
    mapping_client: Box<dyn MappingClient>,

    /// The digital twin client
    digital_twin_client: Box<dyn DigitalTwinAdapter>,

    /// The provider proxy selector client
    provider_proxy_selector_client: ProviderProxySelectorRequestSender,

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
        digital_twin_client: Box<dyn DigitalTwinAdapter>,
        provider_proxy_selector_client: ProviderProxySelectorRequestSender,
        poll_interval: Duration,
    ) -> Self {
        Self {
            signals,
            mapping_client,
            digital_twin_client,
            provider_proxy_selector_client,
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
                let mut signals: Vec<_> = self
                    .mapping_client
                    .get_mapping(GetMappingRequest {})
                    .await?
                    .map
                    .into_iter()
                    .map(|(id, entry)| Signal {
                        // TODO: Should this id be here or is it part of the entity?
                        id,
                        target: Target {
                            metadata: entry.target,
                        },
                        emission: Emission {
                            policy: EmissionPolicy {
                                interval_ms: entry.interval_ms,
                                emit_only_if_changed: entry.emit_on_change,
                                conversion: Conversion::default(),
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .collect();

                // Some of these API calls are not really necessary, but this code gets executed
                // infrequently enough that the sub-optimal performance is not a major concern.
                // If Ibeji had a bulk find_by_id API there would be even less of a concern.
                // TODO: punt stuff to the dt client and we call find_all here
                // TODO: if there's a bulk api for providers then there probably needs to be a bulk api for proxies
                // TODO: handle errors
                for signal in signals.iter_mut() {
                    signal.source = self
                        .digital_twin_client
                        .find_by_id(GetDigitalTwinProviderRequest {
                            entity_id: signal.id.clone(),
                        })
                        .await?
                        .entity;

                    let request = ProviderProxySelectorRequestKind::CreateOrUpdateProviderProxy {
                        entity_id: signal.source.id.clone(),
                        uri: signal.source.uri.clone(),
                        protocol: signal.source.protocol.clone(),
                        operation: signal.source.operation.clone(),
                    };

                    self.provider_proxy_selector_client
                        .send_request_to_provider_proxy_selector(request);
                }

                self.signals.do_the_thing(signals.into_iter());
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
