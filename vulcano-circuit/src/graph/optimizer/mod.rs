//! Module for optimizing circuits represented as graphs.
//! This module provides functionality to perform various
//! optimization passes on computation circuits, such as
//! topological sorting of gates or removal of unreachable gates.

mod topological_sort;

use crate::{error::Result, gate::Gate, graph::circuit::Circuit};

/// Tracks the optimization state of a circuit.
/// TODO: Analysis manager approach. Use analysis traits and caching
/// for analyses between passes and dependency tracking.
struct OptimizationState {
    /// Whether the circuit's gates are ordered topologically.
    ordered: bool,
    /// Whether circuit optimization has been finalized.
    finalized: bool,
}

impl OptimizationState {
    /// Create a new [`OptimizationState`] instance.
    fn new() -> Self {
        Self {
            ordered: false,
            finalized: false,
        }
    }
}

/// Catalogue of available optimization passes.
pub enum OptimizationPass {
    /// Topological sort pass.
    TopologicalSort,
}

/// Optimizer struct that applies a series of optimization passes
/// to a given circuit.
pub struct Optimizer {
    /// List of optimization passes to apply.
    passes: Vec<OptimizationPass>,
}

impl Optimizer {
    /// Create a new [`Optimizer`] instance.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Add an optimization pass to the optimizer from the catalogue.
    pub fn add_pass(&mut self, pass: OptimizationPass) {
        self.passes.push(pass);
    }

    /// Optimize the given circuit by applying all registered
    /// optimization passes in sequence.
    pub fn optimize<T: Gate>(&self, mut circuit: Circuit<T>) -> Result<Circuit<T>> {
        let mut state = OptimizationState::new();
        for pass in &self.passes {
            match pass {
                OptimizationPass::TopologicalSort => {
                    topological_sort::topological_sort(&mut circuit, &mut state)?
                }
            }
        }
        Ok(circuit)
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}
