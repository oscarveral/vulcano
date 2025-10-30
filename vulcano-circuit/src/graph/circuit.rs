//! Circuit representation module.
//!
//! This module defines the core [`Circuit`] structure used to
//! represent computation graphs. A [`Circuit`] is composed of gates,
//! wires, inputs and outputs. The circuit representation is generic
//! over the gate type, allowing different kinds of gates to be used
//! within the same circuit framework.

use crate::{gate::Gate, handles::Wire};

/// Represents how a wire is used in the circuit.
pub enum Use {
    /// A read operation from the wire.
    Read(Wire),
    /// A consume operation from the wire.
    Consume(Wire),
}

/// Represents a computation circuit composed of gates and wires.
///
/// The circuit is generic over the gate type `T`, which must implement
/// the [`Gate`] trait. Each gate in the circuit is associated with its
/// inputs (as a list of [`Use`] instances) and the output wire it
/// produces.
pub struct Circuit<T: Gate> {
    /// Per-gate storage: (gate, list of inputs, output).
    pub(super) gate_entries: Vec<(T, Vec<Use>, Wire)>,
    /// Wires connected to the circuit's inputs.
    pub connected_inputs: Vec<Wire>,
    /// Wires connected to the circuit's outputs.
    pub connected_outputs: Vec<Wire>,
    /// Total number of wires in the circuit.
    pub wire_count: usize,
}

impl<T: Gate> Circuit<T> {
    /// Create a new [`Circuit`] instance.
    pub fn new(
        gate_entries: Vec<(T, Vec<Use>, Wire)>,
        connected_inputs: Vec<Wire>,
        connected_outputs: Vec<Wire>,
        wire_count: usize,
    ) -> Self {
        Self {
            gate_entries,
            connected_inputs,
            connected_outputs,
            wire_count,
        }
    }
}
