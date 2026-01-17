//! Output operations.
//!
//! This module provides the internal representation of output operations
//! and the index type used to identify them. Outputs are operations that
//! consume one value and produce nothing. The outputs are circuit outputs.

use crate::circuit::{subcircuit::CircuitId, value::ValueId};

use vulcano_arena::Key;

/// Handle identifying a circuit output.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OutputId {
    /// The circuit this output belongs to.
    circuit: CircuitId,
    /// Specific output in the circuit.
    key: Key,
}

impl OutputId {
    /// Create a new output id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self { circuit, key }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Return the circuit this output belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// Output operation: circuit output, consumes one value.
pub struct OutputOp {
    /// The input value.
    input: ValueId,
}

impl OutputOp {
    /// Create a new output operation.
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
