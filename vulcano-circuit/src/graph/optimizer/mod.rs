//! Module for optimizing circuits.
//!
//! This module provides functionality to optimize computation
//! circuits represented as graphs. The optimizations can leverage
//! analyses provided by the [`Analyzer`] module to make informed
//! decisions about circuit transformations.

pub mod passes;

use std::any::TypeId;

use crate::{
    error::Result,
    gate::Gate,
    graph::{analyzer::Analyzer, circuit::Circuit},
};

/// A type alias for an optimizer pass function.
///
/// Passes return a tuple containing the optimized circuit and a Vec of [`TypeId`] representing
/// the analyses they preserve.
type OptimizerPass<T> = fn(Circuit<T>, &mut Analyzer) -> Result<(Circuit<T>, Vec<TypeId>)>;

/// Struct that manages and applies optimization passes to circuits.
pub struct Optimizer<T: Gate> {
    analyzer: Analyzer,
    passes: Vec<OptimizerPass<T>>,
}

impl<T: Gate> Optimizer<T> {
    /// Creates a new optimizer instance.
    pub fn new() -> Self {
        Self {
            analyzer: Analyzer::new(),
            passes: Vec::new(),
        }
    }

    /// Adds an optimization pass to the optimizer.
    pub fn add_pass(&mut self, pass: OptimizerPass<T>) {
        self.passes.push(pass);
    }

    /// Runs all optimization passes on the given circuit.
    pub fn optimize(&mut self, mut circuit: Circuit<T>) -> Result<Circuit<T>> {
        for pass in &self.passes {
            let (optimized_circuit, preserved_analyses) = pass(circuit, &mut self.analyzer)?;
            circuit = optimized_circuit;
            // Invalidate analyses not in preserved_analyses.
            self.analyzer
                .invalidate_except(preserved_analyses.as_slice());
        }
        Ok(circuit)
    }
}

impl<T: Gate> Default for Optimizer<T> {
    fn default() -> Self {
        Self::new()
    }
}
