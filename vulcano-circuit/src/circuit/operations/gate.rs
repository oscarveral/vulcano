//! Gate operations.
//!
//! This module provides the internal representation of gate operations
//! and the index type used to identify them. Gates are user-defined
//! computations that take a number of input values and produce a number of
//! output values.

use vulcano_arena::Key;

use crate::{
    circuit::{subcircuit::CircuitId, value::ValueId},
    error::{Error, Result},
    gate::Gate,
};

/// Handle identifying a gate in a circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateId {
    /// The circuit this gate belongs to.
    circuit: CircuitId,
    /// Specific gate in the circuit.
    key: Key,
}

impl GateId {
    /// Create a new gate id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self { circuit, key }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.key
    }

    /// Return the circuit this gate belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// A gate operation. User-defined computation.
pub struct GateOp<G: Gate> {
    /// The gate descriptor.
    gate: G,
    /// Input values.
    inputs: Vec<ValueId>,
    /// Output values.
    outputs: Vec<ValueId>,
}

impl<G: Gate> GateOp<G> {
    /// Create a new gate operation.
    pub fn new(gate: G, inputs: Vec<ValueId>, outputs: Vec<ValueId>) -> Result<Self> {
        // Check compatibility.
        if inputs.len() != gate.input_count() {
            return Err(Error::InvalidInputCount(gate.input_count(), inputs.len()));
        }
        if outputs.len() != gate.output_count() {
            return Err(Error::InvalidOutputCount(
                gate.output_count(),
                outputs.len(),
            ));
        }

        Ok(Self {
            gate,
            inputs,
            outputs,
        })
    }

    /// Get the gate descriptor.
    pub fn get_gate(&self) -> &G {
        &self.gate
    }

    /// Get the input values.
    pub fn get_inputs(&self) -> &[ValueId] {
        &self.inputs
    }

    /// Get the output values.
    pub fn get_outputs(&self) -> &[ValueId] {
        &self.outputs
    }
}
