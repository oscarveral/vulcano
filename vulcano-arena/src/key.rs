//! Key type for the arena.

/// A key with index and generation for arena access.
///
/// Keys are stable references to slots in the arena. Even after deletion
/// and reuse of a slot, old keys will fail to access the new data due to
/// generation mismatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    /// Index into the arena's slot array.
    pub(crate) index: u32,
    /// Generation counter to detect stale keys.
    pub(crate) generation: u32,
}

impl Key {
    /// Returns the index portion of the key.
    pub fn index(&self) -> usize {
        self.index as usize
    }

    /// Returns the generation portion of the key.
    pub fn generation(&self) -> u32 {
        self.generation
    }
}
