//! Key type for the arena.

/// A key with index and version for arena access.
///
/// Keys are stable references to slots in the arena. Even after deletion
/// and reuse of a slot, old keys will fail to access the new data due to
/// version mismatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    /// Index into the arena's slot array.
    pub(crate) index: usize,
    /// Version counter to detect stale keys.
    pub(crate) version: usize,
}

impl Key {
    /// Returns the index portion of the key.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the version portion of the key.
    pub fn version(&self) -> usize {
        self.version
    }
}
