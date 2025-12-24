//! Handles used throughout the crate
//!
//! This module defines the small, opaque handle types used by the
//! builder and circuit layers. Each handle is a thin wrapper around a
//! numeric index and is intentionally small and cheap to copy.
//! These types are intentionally minimal; they exist to make APIs more
//! self-documenting and to prevent accidental mixing of indexes.

/// Handle identifying a gate/operation in a circuit.
///
/// A gate id is a compact newtype-like wrapper around a numeric
/// index. Use gate ids when referring to the producer of a value
/// (the gate).
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) struct GateId {
    id: usize,
}

impl GateId {
    /// Create a new gate id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub(super) fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying an input slot for the circuit.
///
/// An input id represents an externally-provided input value. It is used
/// when wiring builders or when mapping runtime inputs into the
/// execution plan.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) struct InputId {
    id: usize,
}

impl InputId {
    /// Create a new input id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub(super) fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying an exported output slot of the circuit.
///
/// An output id represents an externally-visible output value. It is used
/// when wiring builders or when mapping runtime outputs from the
/// execution plan.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) struct OutputId {
    id: usize,
}

impl OutputId {
    /// Create a new output id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub(super) fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying a internal node in the circuit.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub(super) struct NodeId {
    id: usize,
}

impl NodeId {
    /// Create a new node id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub(super) fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying a internal port in the circuit.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) struct PortId {
    id: usize,
}

impl PortId {
    /// Create a new port id from a numeric index.
    pub(super) fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub(super) fn id(&self) -> usize {
        self.id
    }
}

/// A source of a value in the circuit.
pub(super) enum Source {
    Gate { gate: GateId, port: PortId },
    Input(InputId),
}

impl From<(GateId, PortId)> for Source {
    fn from(value: (GateId, PortId)) -> Self {
        Self::Gate {
            gate: value.0,
            port: value.1,
        }
    }
}

impl From<InputId> for Source {
    fn from(value: InputId) -> Self {
        Self::Input(value)
    }
}

/// A destination of a value in the circuit.
pub(super) enum Destination {
    Gate { gate: GateId, port: PortId },
    Output(OutputId),
}

impl From<(GateId, PortId)> for Destination {
    fn from(value: (GateId, PortId)) -> Self {
        Self::Gate {
            gate: value.0,
            port: value.1,
        }
    }
}

impl From<OutputId> for Destination {
    fn from(value: OutputId) -> Self {
        Self::Output(value)
    }
}

/// Access mode for a value in the circuit.
/// Use by the gates to indicate how they will access the value.
pub(super) enum AccessMode {
    /// The specified value is borrowed.
    Borrow,
    /// The specified value is moved, an therefore consumed by the gate.
    Move,
}
