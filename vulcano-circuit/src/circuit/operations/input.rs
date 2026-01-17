//! Input operations.
//!
//! This module provides the internal representation of input operations
//! and the index type used to identify them. Inputs are operations that
//! produce one value and consume nothing. The inputs are external circuit
//! inputs.

use crate::circuit::{subcircuit::CircuitId, value::ValueId};

use vulcano_arena::Key;

/// Handle identifying a circuit input.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InputId {
    /// The circuit this input belongs to.
    circuit: CircuitId,
    /// Specific input in the circuit.
    key: Key,
}

impl InputId {
    /// Create a new input id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self { circuit, key }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Return the circuit this input belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// Input operation: external circuit input, produces one value.
pub struct InputOp {
    /// The output value.
    output: ValueId,
}

impl InputOp {
    /// Create a new input operation.
    pub fn new(output: ValueId) -> Self {
        Self { output }
    }

    /// Get the output value.
    pub fn get_output(&self) -> ValueId {
        self.output
    }
}
