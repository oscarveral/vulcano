//! Handles used throughout the crate
//!
//! This module defines strongly-typed indices for circuit elements.
//! Each handle wraps a numeric index and prevents accidental mixing.

/// Handle identifying a gate in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct GateId(usize);

impl GateId {
    /// Create a new gate id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Handle identifying a clone operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct CloneId(usize);

impl CloneId {
    /// Create a new clone id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Handle identifying a drop operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct DropId(usize);

impl DropId {
    /// Create a new drop id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Handle identifying an SSA value in the circuit.
///
/// Each value is defined exactly once and consumed exactly once.
/// A value can be borrowed any number of times before being consumed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct ValueId(usize);

impl ValueId {
    /// Create a new value id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Handle identifying a circuit input.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct InputId(usize);

impl InputId {
    /// Create a new input id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
        self.0
    }
}

/// Handle identifying a circuit output.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct OutputId(usize);

impl OutputId {
    /// Create a new output id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub(super) fn index(self) -> usize {
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
