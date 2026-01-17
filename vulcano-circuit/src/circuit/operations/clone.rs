//! Clone operations.
//!
//! This module provides the internal representation of clone operations
//! and the index type used to identify them. Clones are operations that
//! borrow one value and produce N copies of it.

use vulcano_arena::Key;

use crate::circuit::{subcircuit::CircuitId, value::ValueId};

/// Handle identifying a clone operation in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CloneId {
    /// The circuit this clone belongs to.
    circuit: CircuitId,
    /// Specific clone in the circuit.
    key: Key,
}

impl CloneId {
    /// Create a new clone id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self { circuit, key }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Return the circuit this clone belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// Clone operation: borrow one value, produce N copies.
pub struct CloneOp {
    /// The input value.
    input: ValueId,
    /// The output values.
    outputs: Vec<ValueId>,
}

impl CloneOp {
    /// Create a new clone operation.
    pub fn new(input: ValueId, outputs: Vec<ValueId>) -> Self {
        Self { input, outputs }
    }

    /// Get the input value.
    pub fn get_input(&self) -> ValueId {
        self.input
    }

    /// Get the output values.
    pub fn get_outputs(&self) -> &[ValueId] {
        &self.outputs
    }

    /// Get the number of output copies.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}
