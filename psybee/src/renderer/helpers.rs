// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{atomic::AtomicBool, Arc},
};

/// A trait for types that can't reasonably be hashed, but you need a fast fingerprint for.
pub trait Fingerprint {
    fn fingerprint(&self) -> u64;
}

// implement Fingerprint for Tuples of Fingerprint types
impl<A, B> Fingerprint for (A, B)
where
    A: Fingerprint,
    B: Fingerprint,
{
    // hash the two fingerprints together
    fn fingerprint(&self) -> u64 {
        let (a, b) = self;
        let mut state = DefaultHasher::new();
        a.fingerprint().hash(&mut state);
        b.fingerprint().hash(&mut state);
        state.finish()
    }
}

// atomatic cache entry. counter increments on clone, decrements on drop
#[derive(Clone, Debug)]
pub struct CacheEntry(Arc<u128>, Arc<AtomicBool>);

impl Hash for CacheEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Eq for CacheEntry {}

impl PartialEq for CacheEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl CacheEntry {
    pub fn new() -> Self {
        // create random u128
        let u = rand::random();
        Self(Arc::new(u), Arc::new(AtomicBool::new(false)))
    }

    pub fn id(&self) -> u128 {
        *self.0
    }

    pub fn n_refs(&self) -> usize {
        Arc::strong_count(&self.0)
    }

    pub fn is_sole_ref(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }

    pub fn is_dirty(&self) -> bool {
        self.1.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_dirty(&self, dirty: bool) {
        self.1.store(dirty, std::sync::atomic::Ordering::Relaxed);
    }
}

pub trait Cacheable {
    fn cache_id(&self) -> CacheEntry;
}

pub struct Cache<T> {
    data: HashMap<CacheEntry, (T, u64)>,
}

impl<T> Cache<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<K>(&mut self, entry: &K, value: T)
    where
        K: Cacheable + Fingerprint,
    {
        self.data
            .insert(entry.cache_id().clone(), (value, entry.fingerprint()));
    }

    pub fn get<K>(&self, entry: &K) -> Option<&T>
    where
        K: Cacheable + Fingerprint,
    {
        // check if the fingerprint matches
        if let Some(v) = self.data.get(&entry.cache_id()) {
            if v.1 != entry.fingerprint() {
                // // remove the entry if the fingerprint doesn't match
                // self.data.remove(&entry.cache_id());
                return None;
            }
        } else {
            // no entry found
            return None;
        }

        // return the value
        Some(&self.data.get(&entry.cache_id()).unwrap().0)
    }

    pub fn get_sweep<K>(&mut self, entry: &K) -> Option<&T>
    where
        K: Cacheable + Fingerprint,
    {
        // check if the fingerprint matches
        if let Some(v) = self.data.get(&entry.cache_id()) {
            if v.1 != entry.fingerprint() {
                // remove the entry if the fingerprint doesn't match
                self.data.remove(&entry.cache_id());
                return None;
            }
        } else {
            // no entry found
            return None;
        }

        // return the value
        Some(&self.data.get(&entry.cache_id()).unwrap().0)
    }

    pub fn contains<K>(&self, entry: K) -> bool
    where
        K: Cacheable + Fingerprint,
    {
        self.data.contains_key(&entry.cache_id())
    }

    // clear the cache of all entries that have no references outside the cache
    pub fn sweep(&mut self) {
        self.data.retain(|k, _| !k.is_sole_ref());
    }
}
