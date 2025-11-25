//! Circuit representation module.
//!
//! This module defines the core [`Circuit`] structure used to
//! represent computation graphs. A [`Circuit`] is composed of gates
//! and their dependencies. The circuit representation is generic
//! over the gate type, allowing different kinds of gates to be used
//! within the same circuit framework.
//!
//! The circuit uses a Static Single Assignment (SSA) style representation
//! where each gate produces exactly one value, and dependencies are
//! tracked through direct references to other gates or circuit inputs
//! via the [`Source`] enum.

use crate::{
    gate::Gate,
    handles::{Input, Operation, Output, Source},
};

/// Represents a computation circuit as a directed acyclic graph (DAG).
///
/// The circuit is generic over the gate type `T`, which must implement
/// the [`Gate`] trait. Each gate in the circuit is associated with its
/// input dependencies (as a list of [`Source`] instances).
pub struct Circuit<T: Gate> {
    /// Per-gate storage.
    ///
    /// Each gate has a list of sources that feed its inputs. The order
    /// of sources corresponds to the gate's input positions.
    pub(super) gate_entries: Vec<(T, Vec<Source>)>,

    /// Number of circuit inputs.
    ///
    /// Circuit inputs are numbered from 0 to `input_count - 1`.
    pub(super) input_count: usize,

    /// Gates that produce circuit outputs, indexed by output position.
    ///
    /// Each output is connected to exactly one gate that produces its value.
    pub connected_outputs: Vec<Operation>,
}

impl<T: Gate> Circuit<T> {
    /// Create a new [`Circuit`] instance.
    pub fn new(
        gate_entries: Vec<(T, Vec<Source>)>,
        input_count: usize,
        connected_outputs: Vec<Operation>,
    ) -> Self {
        Self {
            gate_entries,
            input_count,
            connected_outputs,
        }
    }

    /// Returns the number of gates in the circuit.
    pub fn gate_count(&self) -> usize {
        self.gate_entries.len()
    }

    /// Returns the number of circuit inputs.
    pub fn input_count(&self) -> usize {
        self.input_count
    }

    /// Returns the number of circuit outputs.
    pub fn output_count(&self) -> usize {
        self.connected_outputs.len()
    }

    /// Returns an iterator over all circuit input handles.
    ///
    /// This is a convenience method that creates [`Input`] handles for all
    /// inputs in the circuit (from 0 to `input_count - 1`).
    pub fn inputs(&self) -> impl Iterator<Item = Input> {
        (0..self.input_count).map(Input::new)
    }

    /// Returns an iterator over all operation handles in the circuit.
    ///
    /// This is a convenience method that creates [`Operation`] handles for all
    /// gates in the circuit (from 0 to `gate_count - 1`).
    pub fn operations(&self) -> impl Iterator<Item = Operation> {
        (0..self.gate_count()).map(Operation::new)
    }

    /// Returns an iterator over all circuit output handles.
    ///
    /// This is a convenience method that creates [`Output`] handles for all
    /// outputs in the circuit (from 0 to `output_count - 1`).
    pub fn outputs(&self) -> impl Iterator<Item = Output> {
        (0..self.output_count()).map(Output::new)
    }
}
