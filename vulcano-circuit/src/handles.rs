//! Handles used throughout the crate
//!
//! This module defines the small, opaque handle types used by the
//! builder and circuit layers. Each handle is a thin wrapper around a
//! numeric index and is intentionally small and cheap to copy.

//! These types are intentionally minimal; they exist to make APIs more
//! self-documenting and to prevent accidental mixing of indexes.

/// Handle identifying a gate/operation in a circuit.
///
/// An [`Operation`] is a compact newtype-like wrapper around a numeric
/// index. Use [`Operation`] when referring to the producer of a value
/// (the gate).
pub struct Operation {
    id: usize,
}

impl Operation {
    /// Create a new [`Operation`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation({})", self.id)
    }
}

impl std::cmp::PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Eq for Operation {}

impl std::clone::Clone for Operation {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::marker::Copy for Operation {}

/// Handle identifying an input slot for the circuit.
///
/// An [`Input`] represents an externally-provided input value. It is used
/// when wiring builders or when mapping runtime inputs into the
/// execution plan.
pub struct Input {
    id: usize,
}

impl Input {
    /// Create a new [`Input`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Input({})", self.id)
    }
}

impl std::clone::Clone for Input {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::marker::Copy for Input {}

/// Handle identifying an exported output slot of the circuit.
///
/// An [`Output`] represents an externally-visible output value. It is used
/// when wiring builders or when mapping runtime outputs from the
/// execution plan.
pub struct Output {
    id: usize,
}

impl Output {
    /// Create a new [`Output`] handle from a numeric index.
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    /// Return the numeric index used internally for this handle.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Output({})", self.id)
    }
}

impl std::clone::Clone for Output {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::marker::Copy for Output {}

/// Low-level runtime handle representing storage (a wire/register).
///
/// A [`Wire`] represents a runtime storage location used to hold
/// intermediate values produced and consumed by gates during execution.
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

impl std::fmt::Debug for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wire({})", self.id)
    }
}

impl std::clone::Clone for Wire {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::marker::Copy for Wire {}
