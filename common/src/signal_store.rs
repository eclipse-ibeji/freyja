// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{sync::RwLock, collections::HashMap};

use freyja_contracts::signal::Signal;

/// Stores signals in a thread-safe manner.
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

    /// Get a value from the store.
    /// Acquires a read lock.
    ///
    /// # Arguments
    /// - `id`: The id of the entity to retrieve
    pub fn get(&self, id: &String) -> Option<Signal> {
        let signals = self.signals.read().unwrap();
        signals.get(id).cloned()
    }

    /// Gets a Vec containing copies all of the signals in the store.
    /// Acquires a read lock.
    pub fn get_all(&self) -> Vec<Signal> {
        let signals = self.signals.read().unwrap();
        signals.iter().map(|(_, signal)| signal.clone()).collect()
    }

    /// TODO: Needs an actual name, and maybe the input should be a subset of signal
    /// For each signal in the input:
    /// - If the incoming signal is already in the data store, update only its source, target, and emission policy.
    ///     We don't update any of the other data that's being managed by the emitter to avoid untimely or incorrect emissions.
    /// - If the incoming signal is not in the data store, insert it
    /// 
    /// For each signal in the data store:
    /// - If the stored signal is not in the input, delete it
    /// 
    /// The previous state of the store is discarded.
    /// Acquires a write lock.
    /// 
    /// # Arguments
    /// - `incoming_signals`: The list of input signals
    pub fn do_the_thing<SignalIterator>(&self, incoming_signals: SignalIterator)
    where
        SignalIterator: Iterator<Item = Signal>
    {
        // This algorithm avoids trying to iterate over incoming_signals multiple times since iterators are consumed in this process.
        // If the iterator were cloneable then the implementation would be a bit nicer, but in general that's not always possible
        // (and in particular, it's not possible with the iterator being passed to this function in its usage).
        // This function isn't invoked very often (only when we have a new mapping), so less-than-optimal efficiency is less of a concern.
        let mut signals = self.signals.write().unwrap();

        let size_hint = incoming_signals.size_hint();
        let mut incoming_ids = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for incoming_signal in incoming_signals {
            // We'll use these ids later to only retain entries in the store which were in the incoming list.
            // We track it separately from the input iterator since we can't reuse the iterator.
            incoming_ids.push(incoming_signal.id.clone());

            signals
                .entry(incoming_signal.id.clone())
                // If the incoming signal is already in the data store, update only its target and emission policy
                .and_modify(|e| {
                    e.source = incoming_signal.source.clone();
                    e.target = incoming_signal.target.clone();
                    e.emission.policy = incoming_signal.emission.policy.clone();
                })
                // If the incoming signal is not in the data store, insert it
                .or_insert(incoming_signal);
        }

        // Delete signals in the store but not in the incoming list
        signals.retain(|id, _| incoming_ids.contains(id));
    }

    /// Sets the value of the signal with the given id to the requested value.
    /// Returns the old value, or None if the signal could not be found.
    /// Acquires a write lock.
    /// 
    /// # Arguments
    /// - `id`: The id of the signal to edit
    /// - `value`: The new value to assign to the signal
    pub fn set_value(&self, id: String, value: String) -> Option<Option<String>> {
        let mut signals = self.signals.write().unwrap();

        let mut result = None;
        signals
            .entry(id)
            .and_modify(|s| {
                result = Some(s.value.clone());
                s.value = Some(value);
            });

        result
    }

    /// Sets the last emitted value of the signal with the given id to the requested value.
    /// Returns the old value, or None if the signal could not be found.
    /// Acquires a write lock.
    /// 
    /// # Arguments
    /// - `id`: The id of the signal to edit
    /// - `value`: The new value to assign to the signal's last emitted value
    pub fn set_last_emitted_value(&self, id: String, value: String) -> Option<Option<String>> {
        let mut signals = self.signals.write().unwrap();

        let mut result = None;
        signals
            .entry(id)
            .and_modify(|s| {
                result = Some(s.emission.last_emitted_value.clone());
                s.emission.last_emitted_value = Some(value);
            });

        result
    }
}

impl Default for SignalStore {
    fn default() -> Self {
        Self::new()
    }
}