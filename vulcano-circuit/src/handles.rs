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
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

/// Handle identifying an input slot for the circuit.
///
/// An [`Input`] represents an externally-provided input value. It is used
/// when wiring builders or when mapping runtime inputs into the
/// execution plan.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

/// Handle identifying an exported output slot of the circuit.
///
/// An [`Output`] represents an externally-visible output value. It is used
/// when wiring builders or when mapping runtime outputs from the
/// execution plan.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

/// Low-level runtime handle representing storage (a wire/register).
///
/// A [`Wire`] represents a runtime storage location used to hold
/// intermediate values produced and consumed by gates during execution.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
