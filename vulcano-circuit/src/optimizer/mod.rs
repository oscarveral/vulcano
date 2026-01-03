//! Optimizer framework
//!
//! This module provides functionality to optimize circuits.
//! Optimizations can leverage analyses provided by the Analyzer.

pub mod passes;

use std::any::TypeId;

use crate::{analyzer::Analyzer, circuit::Circuit, error::Result, gate::Gate};

/// A type alias for an optimizer pass function.
///
/// Passes return a tuple containing the optimized circuit and a Vec of TypeIds
/// representing the analyses they preserve.
pub type OptimizerPass<T> = fn(Circuit<T>, &mut Analyzer<T>) -> Result<(Circuit<T>, Vec<TypeId>)>;

/// Manages and applies optimization passes to circuits.
pub struct Optimizer<T: Gate> {
    analyzer: Analyzer<T>,
    passes: Vec<OptimizerPass<T>>,
}

impl<T: Gate> Optimizer<T> {
    /// Create a new optimizer.
    pub fn new() -> Self {
        Self {
            analyzer: Analyzer::new(),
            passes: Vec::new(),
        }
    }

    /// Add an optimization pass.
    pub fn add_pass(&mut self, pass: OptimizerPass<T>) {
        self.passes.push(pass);
    }

    /// Run all optimization passes on the circuit.
    pub fn optimize(&mut self, mut circuit: Circuit<T>) -> Result<Circuit<T>> {
        for pass in &self.passes {
            let (optimized_circuit, preserved_analyses) = pass(circuit, &mut self.analyzer)?;
            circuit = optimized_circuit;
            self.analyzer.invalidate_except(&preserved_analyses);
        }
        Ok(circuit)
    }
}

impl<T: Gate> Default for Optimizer<T> {
    fn default() -> Self {
        Self::new()
    }
}
