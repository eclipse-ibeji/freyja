// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    collections::{hash_map::Entry::Occupied, hash_map::Entry::Vacant, HashMap},
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam::queue::SegQueue;
use log::{debug, info};
use time::OffsetDateTime;
use tokio::time::sleep;

use freyja_contracts::{
    cloud_adapter::{CloudAdapter, CloudAdapterError, CloudMessageRequest, CloudMessageResponse},
    digital_twin_map_entry::DigitalTwinMapEntry,
    entity::{Entity, EntityID},
    provider_proxy::SignalValue,
    provider_proxy_request::{
        ProviderProxySelectorRequestKind, ProviderProxySelectorRequestSender,
    },
};

type TimeMsLeft = u64;

/// Data emitter for the digital twin sync project
/// Emits sensor data at regular intervals as defined by the map
pub struct Emitter {
    /// The mapping is shared with the cartographer
    dt_map_entries: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>>,

    /// Shared map of entity ID to entity info
    entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,

    /// The cloud adapter used to emit data to the cloud
    cloud_adapter: Box<dyn CloudAdapter + Sync + Send>,

    /// Sends requests to the provider proxy selector
    provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,

    /// Shared message queue for obtaining new signal values
    signal_values_queue: Arc<SegQueue<SignalValue>>,
}

/// The payload for an emission
#[derive(Debug, Clone)]
pub(crate) struct EmissionPayload {
    /// The time left until emission
    time_ms_left: u64,

    // States whether this signal has been emitted
    has_signal_been_emitted: bool,

    // Stores state of previous signal value emitted
    previous_signal_value: Option<String>,

    // States whether this provider value has changed
    did_signal_value_change: bool,
}

impl Emitter {
    /// Creates a new instance of emitter
    ///
    /// # Arguments
    /// - `dt_map_entries`: shared hashmap with Cartographer for storing provider mapping info
    /// - `cloud_adapter`: the cloud adapter used to emit to the cloud
    /// - `entity_map`: shared map of entity ID to entity information
    /// - `provider_proxy_selector_request_sender`: sends requests to the provider proxy selector
    /// - `signal_values_queue`: queue for receiving signal values
    pub fn new(
        dt_map_entries: Arc<Mutex<HashMap<EntityID, DigitalTwinMapEntry>>>,
        cloud_adapter: Box<dyn CloudAdapter + Sync + Send>,
        entity_map: Arc<Mutex<HashMap<EntityID, Option<Entity>>>>,
        provider_proxy_selector_request_sender: Arc<ProviderProxySelectorRequestSender>,
        signal_values_queue: Arc<SegQueue<SignalValue>>,
    ) -> Self {
        Self {
            dt_map_entries,
            cloud_adapter,
            entity_map,
            provider_proxy_selector_request_sender,
            signal_values_queue,
        }
    }

    /// Updates the emissions hashmap when there are changes to the dt_map_entries hashmap
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals to emit
    /// - `dt_map_entries`: cloned hashmap for storing mapping information
    fn update_emissions(
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
        dt_map_entries: &HashMap<EntityID, DigitalTwinMapEntry>,
    ) {
        for (signal_id, entry) in dt_map_entries.iter() {
            // Insert into emissions if unique mapping
            emissions_map
                .entry(signal_id.clone())
                .or_insert_with(|| EmissionPayload {
                    time_ms_left: entry.interval_ms,
                    has_signal_been_emitted: false,
                    previous_signal_value: None,
                    did_signal_value_change: false,
                });
        }

        // If we have a mapping that doesn't exist anymore in map,
        // but still exists in emissions, we remove it
        emissions_map.retain(|signal_id, _| dt_map_entries.contains_key(signal_id));
    }

    /// Creates a key to the shared entity_map when there are new emissions
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals to emit
    async fn create_key_to_entity_map(
        &self,
        emissions_map: &HashMap<EntityID, EmissionPayload>,
    ) -> Result<(), String> {
        // Check to see if an emission entry requires a key
        let mut entity_map = self.entity_map.lock().unwrap();

        // Create a key if a key-value doesn't exist for entity_map
        for (signal_id, _) in emissions_map.iter() {
            entity_map.entry(signal_id.clone()).or_insert_with(|| None);
        }

        // Entity Map may be outdated due to change in emissions,
        // so only retain the entities for signals that need to be emitted
        entity_map.retain(|signal_id, _| emissions_map.contains_key(signal_id));

        Ok(())
    }

    /// Execute this Emitter
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut min_interval_ms = None;
        let mut emissions_map: HashMap<EntityID, EmissionPayload> = HashMap::new();
        let mut signal_values_map = HashMap::new();

        loop {
            let mut dt_map_entries_clone: HashMap<EntityID, DigitalTwinMapEntry>;
            // Note: since this a sync lock, do not introduce async calls to this block without switching to an async lock!
            {
                let dt_map_entries = self.dt_map_entries.lock().unwrap();
                dt_map_entries_clone = dt_map_entries.clone();
            }

            Self::update_emissions(&mut emissions_map, &dt_map_entries_clone);

            self.create_key_to_entity_map(&emissions_map).await?;

            self.emit_signals(
                &mut emissions_map,
                &mut signal_values_map,
                &mut dt_map_entries_clone,
                &mut min_interval_ms,
            )
            .await?;

            let sleep_interval = min_interval_ms.unwrap_or(1000);
            info!("Checking for next emission in {sleep_interval}ms\n");
            sleep(Duration::from_millis(sleep_interval)).await;
        }
    }

    /// Updates the signal values map
    ///
    /// # Arguments
    /// - `signal_values_map`: a map for storing a signal value
    fn update_signal_values_map(&self, signal_values_map: &mut HashMap<EntityID, Option<String>>) {
        while !self.signal_values_queue.is_empty() {
            let SignalValue { entity_id, value } = self.signal_values_queue.pop().unwrap();
            signal_values_map.insert(entity_id, Some(value));
        }
    }

    /// Applies conversion implicitly to a signal_value and sends it to the cloud
    ///
    /// # Arguments
    /// - `dt_map_entry`: the digital twin map entry to send to cloud
    /// - `signal_value`: the signal value
    async fn send_to_cloud(
        &self,
        dt_map_entry: &DigitalTwinMapEntry,
        signal_value: String,
    ) -> Result<CloudMessageResponse, CloudAdapterError> {
        let mut converted = signal_value.clone();
        if let Ok(value) = signal_value.parse::<f32>() {
            converted = dt_map_entry.conversion.apply(value).to_string();
        }

        info!(
            "Digital Twin Instance {:?}: {}",
            dt_map_entry.target, converted
        );
        info!("\t(from {}: {})", dt_map_entry.source, signal_value);

        let cloud_message = CloudMessageRequest {
            cloud_signal: dt_map_entry.target.clone(),
            signal_value: converted,
            signal_timestamp: OffsetDateTime::now_utc().to_string(),
        };
        let response = self.cloud_adapter.send_to_cloud(cloud_message).await?;
        Ok(response)
    }

    /// Gets a signal value from a signal_values hashmap
    ///
    /// # Arguments
    /// - `signal_values_map`: hashmap for storing signal values
    /// - `signal_id`: the signal id
    fn get_signal_value_with_entity_id(
        signal_values_map: &mut HashMap<EntityID, Option<String>>,
        signal_id: &str,
    ) -> Option<String> {
        match signal_values_map.entry(signal_id.to_string()) {
            Occupied(value) => value.get().clone(),
            Vacant(_) => None,
        }
    }

    /// Gets a digital twin map entry from a dt_map_entries hashmap
    ///
    /// # Arguments
    /// - `dt_map_entries`: hashmap for storing mapping information
    fn get_dt_map_entry(
        dt_map_entries: &mut HashMap<EntityID, DigitalTwinMapEntry>,
        signal_id: &str,
    ) -> Option<DigitalTwinMapEntry> {
        match dt_map_entries.entry(signal_id.to_string()) {
            Occupied(dt_map_entry) => Some(dt_map_entry.get().clone()),
            Vacant(_) => None,
        }
    }

    /// Updates did_signal_value_change for each emission in emissions_map
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals to emit
    /// - `signal_values_map`: hashmap for storing signal values
    fn update_emissions_signal_value_change_status(
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
        signal_values_map: &mut HashMap<EntityID, Option<String>>,
    ) {
        for (signal_id, emission_payload) in emissions_map.iter_mut() {
            let previous_signal_value = emission_payload.previous_signal_value.clone();
            let current_signal_value =
                Self::get_signal_value_with_entity_id(signal_values_map, signal_id);

            // Update this emission payload inside emissions if
            // previous value is different
            if previous_signal_value != current_signal_value {
                debug!("{signal_id}, previous signal value {previous_signal_value:?}, current signal_value {current_signal_value:?}");
                emission_payload.previous_signal_value = current_signal_value;
                emission_payload.did_signal_value_change = true;
            } else {
                emission_payload.did_signal_value_change = false;
            }
        }
    }

    /// Emit signals based on the dt_map_entries mapping and the emissions
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals to emit
    /// - `signal_values_map`: hashmap for storing signal values
    /// - `dt_map_entries`: hashmap for storing mapping information
    /// - `min_interval_ms`: minimum interval in milliseconds
    async fn emit_signals(
        &self,
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
        signal_values_map: &mut HashMap<EntityID, Option<String>>,
        dt_map_entries: &mut HashMap<EntityID, DigitalTwinMapEntry>,
        min_interval_ms: &mut Option<TimeMsLeft>,
    ) -> Result<HashMap<EntityID, TimeMsLeft>, Box<dyn std::error::Error + Send + Sync>> {
        // Signals that match the minimum interval
        let emissions_with_min_interval =
            Self::get_emissions_with_min_interval(emissions_map, min_interval_ms);

        if !emissions_map.is_empty() {
            info!("********************BEGIN EMISSION********************");
        }

        // Update our signal values cache
        // After getting signal values, check if a value has changed in
        // emissions and update the did_signal_value_change field for each emission
        self.update_signal_values_map(signal_values_map);
        Self::update_emissions_signal_value_change_status(emissions_map, signal_values_map);

        // Emit the signal values that have min_interval_ms
        for signal_id in emissions_with_min_interval.keys() {
            let signal_value =
                match Self::get_signal_value_with_entity_id(signal_values_map, signal_id) {
                    Some(signal_value) => signal_value,
                    None => {
                        info!(
                        "No signal value for {} in our cache. Skipping emission for this signal.",
                        signal_id
                    );

                        // Send request again for a new signal value since our cache
                        // doesn't contain a signal value for this signal
                        let request = ProviderProxySelectorRequestKind::GetEntityValue(
                            String::from(signal_id),
                        );

                        self.provider_proxy_selector_request_sender
                            .send_request_to_provider_proxy_selector(request);
                        continue;
                    }
                };

            // Acquire the signal entry for the signal id.
            // This is guaranteed to exist since emissions_with_min_interval is a subset of dt_map_entries.
            let dt_map_entry = Self::get_dt_map_entry(dt_map_entries, signal_id).unwrap();

            // Get a new value for this current entity in emission
            // if an entity only supports subscribe this call shouldn't do anything
            let request = ProviderProxySelectorRequestKind::GetEntityValue(String::from(signal_id));
            self.provider_proxy_selector_request_sender
                .send_request_to_provider_proxy_selector(request);

            // Checks if this digital twin map entry requires emit on change
            // then handles the digital twin map entry for emitting on change
            // if the signal has been emitted and its value did not change then skip emission
            if Self::should_emit_signal_for_emit_on_change(&dt_map_entry, emissions_map) {
                Self::set_emission_has_signal_been_emitted(
                    emissions_map,
                    &dt_map_entry.source,
                    false,
                )?;
            } else {
                info!("Signal {} did not change and has already been emitted. Skipping emission for this signal.", dt_map_entry.source);
                continue;
            }

            self.send_to_cloud(&dt_map_entry, signal_value).await?;
            Self::set_emission_has_signal_been_emitted(emissions_map, signal_id, true)?
        }

        if !emissions_map.is_empty() {
            info!("*********************END EMISSION*********************");
        }

        // Emitted the previous signals, so now we update the time left based on new signals in emissions
        Self::update_emission_payload_time_left(dt_map_entries, emissions_map, min_interval_ms);
        debug!("Signals in the emissions after updating {emissions_map:?}");

        Ok(emissions_with_min_interval)
    }

    /// Updates the time left in an emission payload
    /// and sets a new min_interval_ms
    ///
    /// # Arguments
    /// - `dt_map_entries`: hashmap for storing signals
    /// - `emissions_map`: hashmap for storing signals to emit
    /// - `min_interval_ms`: minimum interval in milliseconds
    fn update_emission_payload_time_left(
        dt_map_entries: &mut HashMap<EntityID, DigitalTwinMapEntry>,
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
        min_interval_ms: &mut Option<TimeMsLeft>,
    ) {
        let mut new_min_interval: Option<TimeMsLeft> = None;

        // Update the intervals based on time elapsed
        for (signal_id, emission_payload) in emissions_map.iter_mut() {
            if let Some(interval) = min_interval_ms {
                if emission_payload.time_ms_left >= *interval {
                    emission_payload.time_ms_left -= *interval;
                }

                // Reset the time_ms_left
                if emission_payload.time_ms_left == 0 {
                    let Some(dt_map_entry) = Self::get_dt_map_entry(dt_map_entries, signal_id)
                    else {
                        continue;
                    };
                    emission_payload.time_ms_left = dt_map_entry.interval_ms;
                }
            }

            new_min_interval = match new_min_interval {
                Some(interval) => Some(TimeMsLeft::min(interval, emission_payload.time_ms_left)),
                None => Some(emission_payload.time_ms_left),
            }
        }

        *min_interval_ms = new_min_interval;
    }

    /// Gets the emissions with all entries that have the minimum sleep interval
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals to emit
    /// - `min_interval_ms`: minimum interval in milliseconds
    fn get_emissions_with_min_interval(
        emissions_map: &HashMap<EntityID, EmissionPayload>,
        min_interval_ms: &Option<TimeMsLeft>,
    ) -> HashMap<EntityID, TimeMsLeft> {
        emissions_map
            .clone()
            .into_iter()
            .filter(|(_, emission_payload)| match min_interval_ms {
                Some(interval) => emission_payload.time_ms_left == *interval,
                None => true,
            })
            .map(|(signal_id, emission_payload)| (signal_id, emission_payload.time_ms_left))
            .collect()
    }

    /// Determines if a signal should emit if it requries to be emit on change
    ///
    /// # Arguments
    /// - `dt_map_entry`: digital twin map entry
    /// - `emissions_map`: hashmap for storing signals
    fn should_emit_signal_for_emit_on_change(
        dt_map_entry: &DigitalTwinMapEntry,
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
    ) -> bool {
        // Skip if the digital twin map entry doesn't require emit on change
        if !dt_map_entry.emit_on_change {
            return true;
        }

        match emissions_map.entry(dt_map_entry.source.clone()) {
            // If this digital_twin map entry requires to only emit on change,
            // then if it's signal value did not change and signal has already been emitted
            // we skip this signal's emission
            Occupied(emission_payload) => {
                if !emission_payload.get().did_signal_value_change
                    && emission_payload.get().has_signal_been_emitted
                {
                    return false;
                }
                true
            }
            Vacant(_) => false,
        }
    }

    /// Sets an entry value in emission to has_signal_been_emitted
    ///
    /// # Arguments
    /// - `emissions_map`: hashmap for storing signals
    /// - `signal_id`: the signal id to set the value to has_signal_been_emitted
    /// - `has_signal_been_emitted`: states whether the signal has been emitted
    fn set_emission_has_signal_been_emitted(
        emissions_map: &mut HashMap<EntityID, EmissionPayload>,
        signal_id: &str,
        has_signal_been_emitted: bool,
    ) -> Result<(), EmitterError> {
        match emissions_map.entry(String::from(signal_id)) {
            Occupied(emission_payload) => {
                emission_payload.into_mut().has_signal_been_emitted = has_signal_been_emitted;
                Ok(())
            }
            Vacant(_) => {
                let error_message = format!("Cannot set has_signal_been_emitted = {has_signal_been_emitted} for {signal_id}");
                Err(EmitterError::cannot_set_signal_to_emitted(error_message))
            }
        }
    }
}

proc_macros::error! {
    EmitterError {
        CannotSetSignalToEmitted
    }
}

#[cfg(test)]
mod emitter_tests {
    use super::*;

    use std::collections::HashSet;

    use core::panic;
    use tokio::sync::mpsc;

    use freyja_contracts::conversion::Conversion;
    use in_memory_mock_cloud_adapter::in_memory_mock_cloud_adapter::InMemoryMockCloudAdapter;

    mod fixture {
        use super::*;
        /// Fixture for struct Emitter
        ///
        /// # Arguments
        ///
        /// - `digital_twin_map_entries`: vector of digital twin map entries, where each entry contains mapping info
        /// - `emissions_map`: Hashmap where the key is a map entry to emit, and the value is the time left in milliseconds to emit
        /// - `min_interval_ms`: minimum interval in milliseconds from a list of signal intervals
        pub(crate) struct EmitterFixture {
            pub digital_twin_map_entries: HashMap<EntityID, DigitalTwinMapEntry>,
            pub emissions_map: HashMap<EntityID, EmissionPayload>,
            pub min_interval_ms: Option<u64>,
        }

        impl EmitterFixture {
            /// Setting up the Emitter Test Fixture
            ///
            /// # Arguments
            ///
            /// - `intervals_ms`: vector of requested signal intervals in milliseconds to emit
            pub fn setup(intervals_ms: Vec<u64>) -> Self {
                let digital_twin_map_entries = generate_digital_twin_map_entries(&intervals_ms);
                let emissions_map = insert_digital_map_entries(&digital_twin_map_entries);
                let min_interval_ms = None;
                let mut operations = HashSet::new();
                operations.insert(String::from("Subscribe"));

                EmitterFixture {
                    digital_twin_map_entries,
                    emissions_map,
                    min_interval_ms,
                }
            }

            pub fn set_digital_twin_map_entry_to_emit_on_change(
                dt_map_entry: &mut DigitalTwinMapEntry,
            ) {
                dt_map_entry.emit_on_change = true;
            }

            pub fn check_signal_time_left(&self, signal_id: &str, time_left_ms: u64) -> bool {
                self.emissions_map
                    .get(&String::from(signal_id))
                    .unwrap()
                    .time_ms_left
                    == time_left_ms
            }
        }
    }

    fn generate_digital_twin_map_entries(
        intervals_ms: &[u64],
    ) -> HashMap<EntityID, DigitalTwinMapEntry> {
        let mut digital_twin_map_entries: HashMap<EntityID, DigitalTwinMapEntry> = HashMap::new();

        for (index, interval_ms) in intervals_ms.iter().enumerate() {
            let (source, target_id) = (format!("test_{index}"), format!("test_target_{index}"));

            let mut target = HashMap::new();
            target.insert(target_id, String::new());
            digital_twin_map_entries.insert(
                source.clone(),
                DigitalTwinMapEntry {
                    source,
                    target,
                    interval_ms: *interval_ms,
                    conversion: Conversion::None,
                    emit_on_change: false,
                },
            );
        }

        digital_twin_map_entries
    }

    fn insert_digital_map_entries(
        digital_map_entries: &HashMap<EntityID, DigitalTwinMapEntry>,
    ) -> HashMap<EntityID, EmissionPayload> {
        let mut emissions_map: HashMap<EntityID, EmissionPayload> = HashMap::new();

        for entry in digital_map_entries.values() {
            let emission_payload = EmissionPayload {
                time_ms_left: entry.interval_ms,
                has_signal_been_emitted: false,
                previous_signal_value: None,
                did_signal_value_change: false,
            };
            emissions_map.insert(entry.source.clone(), emission_payload);
        }

        emissions_map
    }

    #[test]
    fn update_emission_payload_time_left_test() {
        let intervals_ms = vec![2000, 3000];
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms.clone());
        let mut signal_ids = emitter_fixture
            .emissions_map
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        // Sort since collecting keys as a vector is random
        signal_ids.sort();

        let [test_1_entry, test_2_entry] = &signal_ids[..] else {
            panic! {"Cannot get digital twin entries"}
        };

        Emitter::update_emission_payload_time_left(
            &mut emitter_fixture.digital_twin_map_entries,
            &mut emitter_fixture.emissions_map,
            &mut intervals_ms.into_iter().min(),
        );

        println!("{test_1_entry}");
        assert_eq!(
            emitter_fixture
                .emissions_map
                .get(test_1_entry)
                .unwrap()
                .time_ms_left,
            2000
        );
        assert_eq!(
            emitter_fixture
                .emissions_map
                .get(test_2_entry)
                .unwrap()
                .time_ms_left,
            1000
        );

        Emitter::update_emission_payload_time_left(
            &mut emitter_fixture.digital_twin_map_entries,
            &mut emitter_fixture.emissions_map,
            &mut None,
        );
        assert_eq!(
            emitter_fixture
                .emissions_map
                .get(test_1_entry)
                .unwrap()
                .time_ms_left,
            2000
        );
        assert_eq!(
            emitter_fixture
                .emissions_map
                .get(test_2_entry)
                .unwrap()
                .time_ms_left,
            1000
        );
    }

    #[test]
    fn update_emissions_did_signal_value_change_test() {
        let intervals_ms = vec![2000, 3000];
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms);
        let mut signal_values_map: HashMap<EntityID, Option<String>> = emitter_fixture
            .emissions_map
            .keys()
            .map(|signal_id| (signal_id.clone(), Some(String::from("0.00"))))
            .collect();

        Emitter::update_emissions_signal_value_change_status(
            &mut emitter_fixture.emissions_map,
            &mut signal_values_map,
        );

        for emission_payload in emitter_fixture.emissions_map.values() {
            assert!(emission_payload.did_signal_value_change);
        }

        Emitter::update_emissions_signal_value_change_status(
            &mut emitter_fixture.emissions_map,
            &mut signal_values_map,
        );
        for emission_payload in emitter_fixture.emissions_map.values() {
            assert!(!emission_payload.did_signal_value_change);
        }
    }

    #[test]
    fn check_emit_for_emit_on_change_test() {
        let intervals_ms = vec![2000, 3000];
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms);

        for (signal_id, dt_map_entry) in emitter_fixture.digital_twin_map_entries.iter_mut() {
            assert!(
                !emitter_fixture
                    .emissions_map
                    .get(signal_id)
                    .unwrap()
                    .has_signal_been_emitted
            );
            assert!(
                !emitter_fixture
                    .emissions_map
                    .get(signal_id)
                    .unwrap()
                    .did_signal_value_change
            );
            assert!(!dt_map_entry.emit_on_change);
            let mut result = Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emitter_fixture.emissions_map,
            );
            assert!(result);

            dt_map_entry.emit_on_change = true;
            emitter_fixture
                .emissions_map
                .get_mut(signal_id)
                .unwrap()
                .did_signal_value_change = true;
            assert!(
                !emitter_fixture
                    .emissions_map
                    .get(signal_id)
                    .unwrap()
                    .has_signal_been_emitted
            );
            result = Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emitter_fixture.emissions_map,
            );
            assert!(result);

            emitter_fixture
                .emissions_map
                .get_mut(signal_id)
                .unwrap()
                .did_signal_value_change = false;
            emitter_fixture
                .emissions_map
                .get_mut(signal_id)
                .unwrap()
                .has_signal_been_emitted = true;
            result = Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emitter_fixture.emissions_map,
            );
            assert!(!result);
        }
    }

    #[tokio::test]
    async fn emit_two_signals_test() {
        let intervals_ms = vec![3, 2];
        let mut time_index = 0;
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms);

        // The setup below is for instantiating an emitter
        let map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let cloud_adapter: Box<dyn CloudAdapter + Send + Sync> =
            InMemoryMockCloudAdapter::create_new().unwrap();

        let entity_map: Arc<Mutex<HashMap<String, Option<Entity>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (tx_provider_proxy_selector_request, _rx_provider_proxy_selector_request) =
            mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let provider_proxy_selector_request_sender = Arc::new(
            ProviderProxySelectorRequestSender::new(tx_provider_proxy_selector_request),
        );
        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());

        let emitter = Emitter::new(
            map,
            cloud_adapter,
            entity_map,
            provider_proxy_selector_request_sender,
            signal_values_queue,
        );
        for _ in 0..10 {
            let mut signal_values_map = HashMap::new();
            let emissions_with_min_interval = emitter
                .emit_signals(
                    &mut emitter_fixture.emissions_map,
                    &mut signal_values_map,
                    &mut emitter_fixture.digital_twin_map_entries,
                    &mut emitter_fixture.min_interval_ms,
                )
                .await
                .unwrap();
            let time_sleep_duration = &emitter_fixture.min_interval_ms;

            // Signal emitting scenarios
            match time_index {
                // Initially we send all the signals and sleep for 2ms
                0 => {
                    assert_eq!(emissions_with_min_interval.len(), 2);
                    assert_eq!(time_sleep_duration.unwrap(), 2)
                }
                // Check if we're only emitting only 1 signal and that we're only sleeping for 1 ms
                2 | 3 | 8 | 9 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 1);
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 2ms
                4 | 10 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 2);
                }
                // Check if we're emitting 2 signals and that we're only sleeping for 2ms
                6 => {
                    assert_eq!(emissions_with_min_interval.len(), 2);
                    assert_eq!(time_sleep_duration.unwrap(), 2);
                }
                _ => {}
            }
            // Simulate sleep
            time_index += time_sleep_duration.unwrap();
        }
    }

    #[tokio::test]
    async fn emit_multiple_signals_test() {
        let intervals_ms = vec![4, 7, 9];
        let mut time_index = 0;
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms);

        // The setup below is for instantiating an emitter
        let map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let cloud_adapter: Box<dyn CloudAdapter + Send + Sync> =
            InMemoryMockCloudAdapter::create_new().unwrap();

        let entity_map: Arc<Mutex<HashMap<String, Option<Entity>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (tx_provider_proxy_selector_request, _rx_provider_proxy_selector_request) =
            mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let provider_proxy_selector_request_sender = Arc::new(
            ProviderProxySelectorRequestSender::new(tx_provider_proxy_selector_request),
        );
        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());

        let emitter = Emitter::new(
            map,
            cloud_adapter,
            entity_map,
            provider_proxy_selector_request_sender,
            signal_values_queue,
        );

        for _ in 0..10 {
            let mut signal_values_map = HashMap::new();
            let emissions_with_min_interval = emitter
                .emit_signals(
                    &mut emitter_fixture.emissions_map,
                    &mut signal_values_map,
                    &mut emitter_fixture.digital_twin_map_entries,
                    &mut emitter_fixture.min_interval_ms,
                )
                .await
                .unwrap();
            let time_sleep_duration = &emitter_fixture.min_interval_ms;

            // Signal emitting scenarios
            match time_index {
                // Initially we send all the signals and sleep for 4 ms
                0 => {
                    assert_eq!(emissions_with_min_interval.len(), 3);
                    assert_eq!(time_sleep_duration.unwrap(), 4)
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 3ms
                4 | 9 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 3);
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 1ms
                7 | 8 | 20 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 1);
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 2ms
                // For time_index 12, 14, 16, 18
                (12..=18) if time_index % 2 == 0 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 2);
                }
                _ => {}
            }
            // Simulate sleep
            time_index += time_sleep_duration.unwrap();
        }
    }

    #[test]
    fn all_signals_emit_on_change_value_change_test() {
        let intervals_ms = vec![4, 7, 9, 2, 8];
        let mut digital_twin_map_entries =
            fixture::EmitterFixture::setup(intervals_ms).digital_twin_map_entries;

        // Set all entries to emit on change
        for (_, dt_map_entry) in digital_twin_map_entries.iter_mut() {
            fixture::EmitterFixture::set_digital_twin_map_entry_to_emit_on_change(dt_map_entry);
            assert!(dt_map_entry.emit_on_change);
        }

        let mut emissions: HashMap<EntityID, EmissionPayload> = HashMap::new();
        for (signal_id, _) in digital_twin_map_entries.iter_mut() {
            let emission_payload = EmissionPayload {
                time_ms_left: 0,
                has_signal_been_emitted: false,
                previous_signal_value: None,
                did_signal_value_change: false,
            };
            emissions.insert(signal_id.clone(), emission_payload);
            assert!(
                Emitter::set_emission_has_signal_been_emitted(&mut emissions, signal_id, true)
                    .is_ok()
            );
            let payload = emissions.get(signal_id).unwrap();
            assert!(payload.has_signal_been_emitted);
        }

        // Test to see if signals with emit on change policy are correctly handled
        for (signal_id, dt_map_entry) in digital_twin_map_entries.iter() {
            assert!(!Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emissions
            ));
            let emission_payload = emissions.get_mut(&signal_id.clone()).unwrap();
            emission_payload.did_signal_value_change = true;
            assert!(Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emissions
            ));
        }
    }

    #[test]
    fn some_signal_emit_on_change_value_change_test() {
        // Initial setup
        let intervals_ms = vec![4, 7, 9, 2, 8];
        let mut digital_twin_map_entries =
            fixture::EmitterFixture::setup(intervals_ms).digital_twin_map_entries;

        for (counter, (_, dt_map_entry)) in digital_twin_map_entries.iter_mut().enumerate() {
            // Set every even counter number to emit on change
            if counter % 2 == 0 {
                fixture::EmitterFixture::set_digital_twin_map_entry_to_emit_on_change(dt_map_entry);
                assert!(dt_map_entry.emit_on_change);
            }
        }

        let mut emissions: HashMap<EntityID, EmissionPayload> = HashMap::new();
        for (signal_id, dt_map_entry) in digital_twin_map_entries.iter_mut() {
            let emission_payload = EmissionPayload {
                time_ms_left: 0,
                has_signal_been_emitted: dt_map_entry.emit_on_change,
                previous_signal_value: None,
                did_signal_value_change: false,
            };
            emissions.insert(signal_id.clone(), emission_payload);
            assert!(
                Emitter::set_emission_has_signal_been_emitted(&mut emissions, signal_id, true)
                    .is_ok()
            );
            let payload = emissions.get(signal_id).unwrap();
            assert!(payload.has_signal_been_emitted);
        }

        // Test to see signals are correctly handled
        for (counter, (signal_id, dt_map_entry)) in digital_twin_map_entries.iter().enumerate() {
            if counter % 2 != 0 {
                assert!(Emitter::should_emit_signal_for_emit_on_change(
                    dt_map_entry,
                    &mut emissions
                ));
            } else {
                assert!(!Emitter::should_emit_signal_for_emit_on_change(
                    dt_map_entry,
                    &mut emissions
                ));
            }
            let emission_payload = emissions.get_mut(&signal_id.clone()).unwrap();
            emission_payload.did_signal_value_change = true;
            assert!(Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emissions
            ));
        }
    }

    #[test]
    fn signal_emit_on_change_no_value_change_err_expect_test() {
        let intervals_ms = vec![4, 7, 9, 2, 8];

        let mut digital_twin_map_entries =
            fixture::EmitterFixture::setup(intervals_ms).digital_twin_map_entries;

        for (_, dt_map_entry) in digital_twin_map_entries.iter_mut() {
            fixture::EmitterFixture::set_digital_twin_map_entry_to_emit_on_change(dt_map_entry);
        }

        let mut emissions: HashMap<EntityID, EmissionPayload> = HashMap::new();
        for (signal_id, dt_map_entry) in digital_twin_map_entries.iter_mut() {
            let emission_payload = EmissionPayload {
                time_ms_left: 0,
                has_signal_been_emitted: dt_map_entry.emit_on_change,
                previous_signal_value: None,
                did_signal_value_change: false,
            };
            emissions.insert(signal_id.clone(), emission_payload);
            assert!(
                Emitter::set_emission_has_signal_been_emitted(&mut emissions, signal_id, true)
                    .is_ok()
            );
            let payload = emissions.get(signal_id).unwrap();
            assert!(payload.has_signal_been_emitted);
        }

        for (_, dt_map_entry) in digital_twin_map_entries.iter() {
            assert!(!Emitter::should_emit_signal_for_emit_on_change(
                dt_map_entry,
                &mut emissions
            ));
        }
    }

    #[tokio::test]
    async fn emit_multiple_signals_on_change_test() {
        let intervals_ms = vec![4, 7, 9];
        let mut time_index = 0;
        let mut emitter_fixture = fixture::EmitterFixture::setup(intervals_ms);

        // The setup below is for instantiating an emitter
        let map: Arc<Mutex<HashMap<String, DigitalTwinMapEntry>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let cloud_adapter: Box<dyn CloudAdapter + Send + Sync> =
            InMemoryMockCloudAdapter::create_new().unwrap();
        let entity_map: Arc<Mutex<HashMap<String, Option<Entity>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (tx_provider_proxy_selector_request, _rx_provider_proxy_selector_request) =
            mpsc::unbounded_channel::<ProviderProxySelectorRequestKind>();
        let provider_proxy_selector_request_sender = Arc::new(
            ProviderProxySelectorRequestSender::new(tx_provider_proxy_selector_request),
        );
        let signal_values_queue: Arc<SegQueue<SignalValue>> = Arc::new(SegQueue::new());

        let emitter = Emitter::new(
            map,
            cloud_adapter,
            entity_map,
            provider_proxy_selector_request_sender,
            signal_values_queue,
        );

        let first_map_entry = emitter_fixture
            .digital_twin_map_entries
            .entry(String::from("test_0"))
            .or_default();
        fixture::EmitterFixture::set_digital_twin_map_entry_to_emit_on_change(first_map_entry);
        assert!(Emitter::set_emission_has_signal_been_emitted(
            &mut emitter_fixture.emissions_map,
            "test_0",
            true
        )
        .is_ok());
        for _ in 0..10 {
            let mut signal_values_map = HashMap::new();
            let emissions_with_min_interval = emitter
                .emit_signals(
                    &mut emitter_fixture.emissions_map,
                    &mut signal_values_map,
                    &mut emitter_fixture.digital_twin_map_entries,
                    &mut emitter_fixture.min_interval_ms,
                )
                .await
                .unwrap();
            let time_sleep_duration = &emitter_fixture.min_interval_ms;

            // Signal emitting scenarios
            match time_index {
                // Initially we send all the signals and sleep for 4 ms
                0 => {
                    assert_eq!(emissions_with_min_interval.len(), 3);
                    assert_eq!(time_sleep_duration.unwrap(), 4)
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 3ms
                4 | 9 => {
                    if time_index == 4 {
                        assert!(emitter_fixture.check_signal_time_left("test_1", 3))
                    };
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 3);
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 1ms
                7 | 8 | 20 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 1);
                }
                // Check if we're emitting 1 signal and that we're only sleeping for 2ms
                // For time_index 12, 14, 16, 18
                (12..=18) if time_index % 2 == 0 => {
                    assert_eq!(emissions_with_min_interval.len(), 1);
                    assert_eq!(time_sleep_duration.unwrap(), 2);
                }
                _ => {}
            }
            // Simulate sleep
            time_index += time_sleep_duration.unwrap();
        }
    }
}
