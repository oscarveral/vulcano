//! Key type for the arena.

use std::marker::PhantomData;

/// A key with index and generation for type-safe arena access.
///
/// Keys are stable references to slots in the arena. Even after deletion
/// and reuse of a slot, old keys will fail to access the new data due to
/// generation mismatch.
#[derive(Debug)]
pub struct Key<T> {
    /// Index into the arena's slot array.
    pub(crate) index: u32,
    /// Generation counter to detect stale keys.
    pub(crate) generation: u32,
    /// Marker for the value type (prevents mixing keys from different arenas).
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Key<T> {}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Key<T> {}

impl<T> std::hash::Hash for Key<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> Key<T> {
    /// Returns the index portion of the key.
    pub fn index(&self) -> usize {
        self.index as usize
    }

    /// Returns the generation portion of the key.
    pub fn generation(&self) -> u32 {
        self.generation
    }
}
