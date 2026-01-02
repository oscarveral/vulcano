//! Handles used throughout the crate
//!
//! This module defines strongly-typed indices for circuit elements.
//! Each handle wraps a generational key and prevents accidental mixing.

use vulcano_arena::Key;

/// Handle identifying a gate in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateId(Key);

impl GateId {
    /// Create a new gate id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying a clone operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CloneId(Key);

impl CloneId {
    /// Create a new clone id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying a drop operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DropId(Key);

impl DropId {
    /// Create a new drop id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying an SSA value in the circuit.
///
/// Each value is defined exactly once and consumed exactly once.
/// A value can be borrowed any number of times before being consumed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ValueId(Key);

impl ValueId {
    /// Create a new value id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying a circuit input.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InputId(Key);

impl InputId {
    /// Create a new input id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying a circuit output.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OutputId(Key);

impl OutputId {
    /// Create a new output id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// Handle identifying a port (input or output slot).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct PortId(usize);

impl PortId {
    /// Create a new port id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Ownership mode for a use of a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Ownership {
    /// Value is borrowed. Remains available after use.
    Borrow,
    /// Value is moved. Consumed, no longer available.
    Move,
}
