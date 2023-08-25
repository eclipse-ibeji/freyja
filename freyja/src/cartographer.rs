// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, time::Duration};

use log::info;

use freyja_contracts::digital_twin_map_entry::DigitalTwinMapEntry;
use freyja_contracts::mapping_client::{CheckForWorkRequest, GetMappingRequest, MappingClient};

/// Manages mappings from the mapping service
pub struct Cartographer {
    /// The mapping shared with the emitter
    map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>>,

    /// The mapping client
    mapping_client: Box<dyn MappingClient>,

    /// The mapping service polling interval
    poll_interval: Duration,
}

impl Cartographer {
    /// Create a new instance of a Cartographer
    ///
    /// # Arguments
    /// - `map`: the shared map instance to update with new changes
    /// - `mapping_client`: the client for the mapping service
    /// - `poll_interval`: the interval at which the cartographer should poll for changes
    pub fn new(
        map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>>,
        mapping_client: Box<dyn MappingClient>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            map,
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
                let mapping_response = self
                    .mapping_client
                    .get_mapping(GetMappingRequest {})
                    .await?;

                // Note: since this a sync lock, do not introduce async calls to this block without switching to an async lock!
                *self.map.lock().unwrap() = mapping_response.map;
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
