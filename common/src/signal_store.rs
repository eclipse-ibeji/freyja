// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, sync::RwLock};

use freyja_contracts::signal::{Emission, Signal, SignalPatch};

/// Stores signals and allows access in a thread-safe manner with support for multiple concurrent readers.
/// Suitable for use as `Arc<SignalStore>`.
pub struct SignalStore {
    /// The data being stored
    signals: RwLock<HashMap<String, Signal>>,
}

impl SignalStore {
    /// Creates an empty SignalStore
    pub fn new() -> Self {
        Self {
            signals: RwLock::new(HashMap::new()),
        }
    }

    /// Get a value from the store. Returns `None` if the signal was not found.
    /// Acquires a read lock.
    ///
    /// # Arguments
    /// - `id`: The id of the entity to retrieve
    pub fn get(&self, id: &String) -> Option<Signal> {
        let signals = self.signals.read().unwrap();
        signals.get(id).cloned()
    }

    /// Gets a `Vec` containing copies all of the signals in the store.
    /// Acquires a read lock.
    pub fn get_all(&self) -> Vec<Signal> {
        let signals = self.signals.read().unwrap();
        signals.iter().map(|(_, signal)| signal.clone()).collect()
    }

    /// For each signal in the input:
    /// - If the incoming signal is already in the data store, apply the patch.
    /// - If the incoming signal is not in the data store, create a new signal from the patch.
    ///
    /// # Arguments
    /// - `incoming_signals`: The signal patches used to generate the new signal
    pub fn add<SyncIterator, IntoSignalPatch>(&self, incoming_signals: SyncIterator)
    where
        SyncIterator: Iterator<Item = IntoSignalPatch>,
        IntoSignalPatch: Into<SignalPatch>,
    {
        let mut signals = self.signals.write().unwrap();
        for value in incoming_signals {
            let SignalPatch {
                id,
                source,
                target,
                emission_policy,
            } = value.into();

            signals
                .entry(id.clone())
                // If the incoming signal is already in the data store, update only its target and emission policy
                .and_modify(|s| {
                    s.source = source.clone();
                    s.target = target.clone();
                    s.emission.policy = emission_policy.clone();
                })
                // If the incoming signal is not in the data store, insert a new one
                .or_insert(Signal {
                    id,
                    source,
                    target,
                    emission: Emission {
                        policy: emission_policy,
                        ..Default::default()
                    },
                    ..Default::default()
                });
        }
    }

    /// For each signal in the input:
    /// - If the incoming signal is already in the data store, apply the patch.
    /// - If the incoming signal is not in the data store, create a new signal from the patch.
    ///
    /// For each signal in the data store:
    /// - If the stored signal is not in the input, delete it
    ///
    /// The previous state of the store is discarded.
    /// Acquires a write lock.
    ///
    /// # Arguments
    /// - `incoming_signals`: The list of input signals
    pub fn sync<SyncIterator, IntoSignalPatch>(&self, incoming_signals: SyncIterator)
    where
        SyncIterator: Iterator<Item = IntoSignalPatch>,
        IntoSignalPatch: Into<SignalPatch>,
    {
        let mut signals = self.signals.write().unwrap();

        // This algorithm avoids trying to iterate over incoming_signals multiple times since iterators are consumed in this process.
        // If the iterator were cloneable then the implementation could be better, but in general that's not always a feasible constraint.
        // This function isn't invoked very often (only when we have a new mapping), so less-than-optimal efficiency is less of a concern.
        let size_hint = incoming_signals.size_hint();
        let mut incoming_ids = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for value in incoming_signals {
            let SignalPatch {
                id,
                source,
                target,
                emission_policy,
            } = value.into();

            // We'll use these ids later to only retain entries in the store which were in the incoming list.
            // We track it separately from the input iterator since we can't reuse the iterator.
            incoming_ids.push(id.clone());

            signals
                .entry(id.clone())
                // If the incoming signal is already in the data store, update only its target and emission policy
                .and_modify(|s| {
                    s.source = source.clone();
                    s.target = target.clone();
                    s.emission.policy = emission_policy.clone();
                })
                // If the incoming signal is not in the data store, insert a new one
                .or_insert(Signal {
                    id,
                    source,
                    target,
                    emission: Emission {
                        policy: emission_policy,
                        ..Default::default()
                    },
                    ..Default::default()
                });
        }

        // Delete signals in the store but not in the incoming list
        signals.retain(|id, _| incoming_ids.contains(id));
    }

    /// Sets the value of the signal with the given id to the requested value.
    /// Returns the old value, or `None` if the signal could not be found.
    /// Acquires a write lock.
    ///
    /// # Arguments
    /// - `id`: The id of the signal to edit
    /// - `value`: The new value to assign to the signal
    pub fn set_value(&self, id: String, value: String) -> Option<Option<String>> {
        let mut signals = self.signals.write().unwrap();

        let mut result = None;
        signals.entry(id).and_modify(|s| {
            result = Some(s.value.clone());
            s.value = Some(value);
        });

        result
    }

    /// Sets the last emitted value of the signal with the given id to the requested value
    /// and resets its `next_emssion_ms` based on the emission policy.
    /// Returns the old value, or `None` if the signal could not be found.
    /// Acquires a write lock.
    ///
    /// # Arguments
    /// - `id`: The id of the signal to edit
    /// - `value`: The new value to assign to the signal's last emitted value
    pub fn set_last_emitted_value(&self, id: String, value: String) -> Option<Option<String>> {
        let mut signals = self.signals.write().unwrap();

        let mut result = None;
        signals.entry(id).and_modify(|s| {
            result = Some(s.emission.last_emitted_value.clone());
            s.emission.last_emitted_value = Some(value);
            s.emission.next_emission_ms = s.emission.policy.interval_ms;
        });

        result
    }

    /// Adjusts the emission times of all signals in the store by subtracting the provided interval from next_emission_ms.
    /// If overflow would occur, the value saturates at `u64::MIN` (`0`).
    /// Returns the updated list of all signals.
    /// Acquires a write lock.
    ///
    /// # Arguments
    /// - `interval_ms`: The value to subtract from each signal's next_emission_ms value
    pub fn update_emission_times_and_get_all(&self, interval_ms: u64) -> Vec<Signal> {
        let mut signals = self.signals.write().unwrap();
        let mut result = Vec::new();

        for (_, signal) in signals.iter_mut() {
            signal.emission.next_emission_ms =
                signal.emission.next_emission_ms.saturating_sub(interval_ms);
            result.push(signal.clone());
        }

        result
    }
}

impl Default for SignalStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod signal_store_tests {
    use super::*;

    use std::collections::HashSet;

    use freyja_contracts::{
        conversion::Conversion,
        entity::{Entity, EntityEndpoint},
        signal::{Emission, EmissionPolicy, Target},
    };

    const GET_OPERATION: &str = "Get";

    #[test]
    fn get_returns_existing_signal() {
        const ID: &str = "testid";

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        let result = uut.get(&ID.to_string());
        assert!(result.is_some());
        assert_eq!(result.unwrap().id.as_str(), ID);
    }

    #[test]
    fn get_returns_none_for_non_existent_signal() {
        const ID: &str = "testid";

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        let result = uut.get(&String::from("invalid"));
        assert!(result.is_none());
    }

    #[test]
    fn get_all_returns_all_signals() {
        let mut ids = HashSet::new();
        for id in &["1", "2", "3"] {
            ids.insert(id.to_string());
        }

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();

            for id in ids.iter() {
                let signal = Signal {
                    id: id.clone(),
                    ..Default::default()
                };

                signals.insert(id.clone(), signal);
            }
        }

        let result = uut.get_all();

        assert!(result.len() == ids.len());

        // HashSet equality checks that both operands have the same contents
        assert_eq!(
            result
                .into_iter()
                .map(|s| s.id)
                .collect::<HashSet<String>>(),
            ids
        );
    }

    #[test]
    fn sync_updates_correct_properties() {
        const ID: &str = "id";
        const ORIGINAL: &str = "original";
        const INCOMING: &str = "incoming";

        let original_signal = Signal {
            id: ID.to_string(),
            value: Some(ORIGINAL.to_string()),
            source: Entity {
                name: Some(ORIGINAL.to_string()),
                id: ID.to_string(),
                description: Some(ORIGINAL.to_string()),
                endpoints: vec![EntityEndpoint {
                    protocol: ORIGINAL.to_string(),
                    operations: vec![GET_OPERATION.to_string()],
                    uri: ORIGINAL.to_string(),
                    context: ORIGINAL.to_string(),
                }],
            },
            target: Target {
                metadata: [(ORIGINAL.to_string(), ORIGINAL.to_string())]
                    .into_iter()
                    .collect(),
            },
            emission: Emission {
                policy: EmissionPolicy {
                    interval_ms: 42,
                    emit_only_if_changed: false,
                    conversion: Conversion::None,
                },
                next_emission_ms: 42,
                last_emitted_value: Some(ORIGINAL.to_string()),
            },
        };

        // Note that everything in this signal is different compared to original_signal
        // (except the id)
        let incoming_signal = Signal {
            id: ID.to_string(),
            value: Some(INCOMING.to_string()),
            source: Entity {
                name: Some(INCOMING.to_string()),
                id: ID.to_string(),
                description: Some(INCOMING.to_string()),
                endpoints: vec![EntityEndpoint {
                    protocol: INCOMING.to_string(),
                    operations: vec!["FooOperation".to_string()],
                    uri: INCOMING.to_string(),
                    context: INCOMING.to_string(),
                }],
            },
            target: Target {
                metadata: [(INCOMING.to_string(), INCOMING.to_string())]
                    .into_iter()
                    .collect(),
            },
            emission: Emission {
                policy: EmissionPolicy {
                    interval_ms: 123,
                    emit_only_if_changed: true,
                    conversion: Conversion::Linear {
                        mul: 1.2,
                        offset: 3.4,
                    },
                },
                next_emission_ms: 123,
                last_emitted_value: Some(INCOMING.to_string()),
            },
        };

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            signals.insert(ID.to_string(), original_signal.clone());
        }

        uut.sync([incoming_signal.clone()].into_iter());
        let updated_signal = uut.get(&ID.to_string()).expect("Test signal should exist");

        // The following fields should have changed to match the incoming signal:
        // - source.*
        // - target.*
        // - emission.policy.*
        assert_eq!(updated_signal.source, incoming_signal.source);
        assert_eq!(updated_signal.target, incoming_signal.target);
        assert_eq!(
            updated_signal.emission.policy,
            incoming_signal.emission.policy
        );

        // The following fields should NOT have changed to match the incoming signal:
        // - value
        // - emission.next_emission_ms
        // - emission.last_emitted_value
        assert_eq!(updated_signal.value, original_signal.value);
        assert_eq!(
            updated_signal.emission.next_emission_ms,
            original_signal.emission.next_emission_ms
        );
        assert_eq!(
            updated_signal.emission.last_emitted_value,
            original_signal.emission.last_emitted_value
        );
    }

    #[test]
    fn sync_inserts_new_signal() {
        const ID: &str = "id";
        const INCOMING: &str = "incoming";

        let incoming_signal = Signal {
            id: ID.to_string(),
            value: Some(INCOMING.to_string()),
            source: Entity {
                name: Some(INCOMING.to_string()),
                id: ID.to_string(),
                description: Some(INCOMING.to_string()),
                endpoints: vec![EntityEndpoint {
                    protocol: INCOMING.to_string(),
                    operations: vec![GET_OPERATION.to_string()],
                    uri: INCOMING.to_string(),
                    context: INCOMING.to_string(),
                }],
            },
            target: Target {
                metadata: [(INCOMING.to_string(), INCOMING.to_string())]
                    .into_iter()
                    .collect(),
            },
            emission: Emission {
                policy: EmissionPolicy {
                    interval_ms: 123,
                    emit_only_if_changed: true,
                    conversion: Conversion::Linear {
                        mul: 1.2,
                        offset: 3.4,
                    },
                },
                next_emission_ms: 123,
                last_emitted_value: Some(INCOMING.to_string()),
            },
        };

        let uut = SignalStore::new();

        uut.sync([incoming_signal.clone()].into_iter());
        let updated_signal = uut.get(&ID.to_string()).expect("Test signal should exist");

        // The following fields should match the incoming signal:
        // - source.*
        // - target.*
        // - emission.policy.*
        assert_eq!(updated_signal.source, incoming_signal.source);
        assert_eq!(updated_signal.target, incoming_signal.target);
        assert_eq!(
            updated_signal.emission.policy,
            incoming_signal.emission.policy
        );

        // The following fields should be initialized to default:
        // - value
        // - emission.next_emission_ms
        // - emission.last_emitted_value
        assert_eq!(updated_signal.value, Default::default());
        assert_eq!(updated_signal.emission.next_emission_ms, u64::default());
        assert_eq!(
            updated_signal.emission.last_emitted_value,
            Default::default()
        );
    }

    #[test]
    fn sync_deletes_signals_not_in_input() {
        const ID: &str = "id";
        const ORIGINAL: &str = "original";

        let original_signal = Signal {
            id: ID.to_string(),
            value: Some(ORIGINAL.to_string()),
            source: Entity {
                name: Some(ORIGINAL.to_string()),
                id: ID.to_string(),
                description: Some(ORIGINAL.to_string()),
                endpoints: vec![EntityEndpoint {
                    protocol: ORIGINAL.to_string(),
                    operations: vec![GET_OPERATION.to_string()],
                    uri: ORIGINAL.to_string(),
                    context: ORIGINAL.to_string(),
                }],
            },
            target: Target {
                metadata: [(ORIGINAL.to_string(), ORIGINAL.to_string())]
                    .into_iter()
                    .collect(),
            },
            emission: Emission {
                policy: EmissionPolicy {
                    interval_ms: 42,
                    emit_only_if_changed: false,
                    conversion: Conversion::None,
                },
                next_emission_ms: 42,
                last_emitted_value: Some(ORIGINAL.to_string()),
            },
        };

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            signals.insert(ID.to_string(), original_signal.clone());
        }

        uut.sync(Vec::<SignalPatch>::new().into_iter());
        let maybe_updated_signal = uut.get(&ID.to_string());
        assert!(maybe_updated_signal.is_none());
    }

    #[test]
    fn set_value_tests() {
        const ID: &str = "testid";

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        // Test first set returns Some(None) and changes state
        let value = String::from("value");
        let result = uut.set_value(ID.to_string(), value.clone());
        assert!(result.is_some());
        assert!(result.unwrap().is_none());
        {
            let signals = uut.signals.read().unwrap();
            assert_eq!(
                signals.get(&ID.to_string()).unwrap().value,
                Some(value.clone())
            );
        }

        // Test setting non-existent value returns None doesn't change state
        let result = uut.set_value(String::from("foo"), String::from("foo"));
        assert!(result.is_none());
        {
            let signals = uut.signals.read().unwrap();
            assert_eq!(
                signals.get(&ID.to_string()).unwrap().value,
                Some(value.clone())
            );
        }

        // Test second set returns Some(Some("value")) and changes state
        let result = uut.set_value(ID.to_string(), String::from("new value"));
        assert!(result.is_some());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap(), value);
        {
            let signals = uut.signals.read().unwrap();
            assert_ne!(
                signals.get(&ID.to_string()).unwrap().value,
                Some(value.clone())
            );
        }
    }

    #[test]
    fn set_last_emitted_value_tests() {
        const ID: &str = "testid";
        const INTERVAL: u64 = 42;
        const UPDATED_EMISSION_TIME: u64 = 20;

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                emission: Emission {
                    policy: EmissionPolicy {
                        interval_ms: INTERVAL,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        // Test first set returns Some(None) and changes state
        let value = String::from("value");
        let result = uut.set_last_emitted_value(ID.to_string(), value.clone());
        assert!(result.is_some());
        assert!(result.unwrap().is_none());
        {
            let signals = uut.signals.read().unwrap();
            let signal = signals.get(&ID.to_string()).unwrap();
            assert_eq!(signal.emission.last_emitted_value, Some(value.clone()));
            assert_eq!(signal.emission.next_emission_ms, INTERVAL);
        }

        {
            // Simulate something changing next_emission_ms, such as the emitter
            let mut signals = uut.signals.write().unwrap();
            signals
                .entry(ID.to_string())
                .and_modify(|s| s.emission.next_emission_ms = UPDATED_EMISSION_TIME);
        }

        // Test setting non-existent value returns None doesn't change state
        let result = uut.set_last_emitted_value(String::from("foo"), String::from("foo"));
        assert!(result.is_none());
        {
            let signals = uut.signals.read().unwrap();
            let signal = signals.get(&ID.to_string()).unwrap();
            assert_eq!(signal.emission.last_emitted_value, Some(value.clone()));
            assert_eq!(signal.emission.next_emission_ms, UPDATED_EMISSION_TIME);
        }

        {
            // Simulate something changing next_emission_ms, such as the emitter
            let mut signals = uut.signals.write().unwrap();
            signals
                .entry(ID.to_string())
                .and_modify(|s| s.emission.next_emission_ms = UPDATED_EMISSION_TIME);
        }

        // Test second set returns Some(Some("value")) and changes state
        let result = uut.set_last_emitted_value(ID.to_string(), String::from("new value"));
        assert!(result.is_some());
        assert!(result.as_ref().unwrap().is_some());
        assert_eq!(result.unwrap().unwrap(), value);
        {
            let signals = uut.signals.read().unwrap();
            let signal = signals.get(&ID.to_string()).unwrap();
            assert_ne!(signal.emission.last_emitted_value, Some(value.clone()));
            assert_eq!(signal.emission.next_emission_ms, INTERVAL);
        }
    }

    #[test]
    fn update_emission_times_and_get_all_sets_correct_value() {
        const ID: &str = "testid";
        const ORIGINAL_VALUE: u64 = 42;
        const INTERVAL: u64 = 20;

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                emission: Emission {
                    next_emission_ms: ORIGINAL_VALUE,
                    ..Default::default()
                },
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        let mut result = uut.update_emission_times_and_get_all(INTERVAL);

        // Validate the values in the result
        assert_eq!(result.len(), 1);
        let signal = result.pop().unwrap();
        assert_eq!(signal.id, ID.to_string());
        assert_eq!(signal.emission.next_emission_ms, ORIGINAL_VALUE - INTERVAL);

        // Validate the values in the store itself
        {
            let signals = uut.signals.read().unwrap();
            assert_eq!(signals.len(), 1);
            assert!(signals.contains_key(&ID.to_string()));
            let signal = signals.get(&ID.to_string()).unwrap();
            assert_eq!(signal.id, ID.to_string());
            assert_eq!(signal.emission.next_emission_ms, ORIGINAL_VALUE - INTERVAL);
        }
    }

    #[test]
    fn update_emission_times_and_get_all_saturates_overflowed_value() {
        const ID: &str = "testid";
        const ORIGINAL_VALUE: u64 = 20;
        const INTERVAL: u64 = u64::MAX;

        let uut = SignalStore::new();
        {
            let mut signals = uut.signals.write().unwrap();
            let signal = Signal {
                id: ID.to_string(),
                emission: Emission {
                    next_emission_ms: ORIGINAL_VALUE,
                    ..Default::default()
                },
                ..Default::default()
            };

            signals.insert(ID.to_string(), signal);
        }

        let mut result = uut.update_emission_times_and_get_all(INTERVAL);

        // Validate the values in the result
        assert_eq!(result.len(), 1);
        let signal = result.pop().unwrap();
        assert_eq!(signal.id, ID.to_string());
        assert_eq!(signal.emission.next_emission_ms, 0);

        // Validate the values in the store itself
        {
            let signals = uut.signals.read().unwrap();
            assert_eq!(signals.len(), 1);
            assert!(signals.contains_key(&ID.to_string()));
            let signal = signals.get(&ID.to_string()).unwrap();
            assert_eq!(signal.id, ID.to_string());
            assert_eq!(signal.emission.next_emission_ms, 0);
        }
    }
}
