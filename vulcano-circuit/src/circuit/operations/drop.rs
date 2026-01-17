//! Drop operations.
//!
//! This module provides the internal representation of drop operations
//! and the index type used to identify them. Drops are operations that
//! consume one value and produce nothing.

use vulcano_arena::Key;

use crate::circuit::{subcircuit::CircuitId, value::ValueId};

/// Handle identifying a drop operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DropId {
    /// The circuit this drop belongs to.
    circuit: CircuitId,
    /// Specific drop in the circuit.
    key: Key,
}

impl DropId {
    /// Create a new drop id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self { circuit, key }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Return the circuit this drop belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// Drop operation: consume a value, produce nothing.
pub struct DropOp {
    /// The input value.
    input: ValueId,
}

impl DropOp {
    /// Create a new drop operation.
    pub fn new(input: ValueId) -> Self {
        Self { input }
    }

    /// Get the input value.
    pub fn get_input(&self) -> ValueId {
        self.input
    }

    /// Set the input value.
    pub fn set_input(&mut self, value: ValueId) {
        self.input = value;
    }
}
