//! Handles used throughout the crate
//!
//! This module defines the small, opaque handle types used by the
//! builder and circuit layers. Each handle is a thin wrapper around a
//! numeric index and is intentionally small and cheap to copy.
//! These types are intentionally minimal; they exist to make APIs more
//! self-documenting and to prevent accidental mixing of indexes.

/// Handle identifying a gate/operation in a circuit.
///
/// A [`GateId`] is a compact newtype-like wrapper around a numeric
/// index. Use [`GateId`] when referring to the producer of a value
/// (the gate).
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct GateId {
    id: usize,
}

impl GateId {
    /// Create a new [`GateId`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying an input slot for the circuit.
///
/// An [`InputId`] represents an externally-provided input value. It is used
/// when wiring builders or when mapping runtime inputs into the
/// execution plan.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct InputId {
    id: usize,
}

impl InputId {
    /// Create a new [`InputId`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Handle identifying an exported output slot of the circuit.
///
/// An [`OutputId`] represents an externally-visible output value. It is used
/// when wiring builders or when mapping runtime outputs from the
/// execution plan.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct OutputId {
    id: usize,
}

impl OutputId {
    /// Create a new [`OutputId`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Low-level runtime handle representing storage (a wire/register).
///
/// A [`Wire`] represents a runtime storage location used to hold
/// intermediate values produced and consumed by gates during execution.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct Wire {
    id: usize,
}

impl Wire {
    /// Create a new [`Wire`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Represents a value flowing through the circuit.
///
/// A [`Value`] can be either a circuit input or the output of a gate.
/// This is the primary way dependencies between gates are represented in the IR.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Value {
    /// The value comes from a circuit input slot.
    Input(InputId),
    /// The value comes from a gate's output.
    Gate(GateId),
}
