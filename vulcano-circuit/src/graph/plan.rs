//! Execution plan data structures for runtime execution.
//!
//! This module defines the structures used to represent a compiled execution plan
//! that can be efficiently executed by a runtime. The plan is organized hierarchically:
//! - `ExecutionPlan`: Top-level container with multiple independent partitions
//! - `Partition`: Self-contained execution unit for a subcircuit
//! - `Layer`: Sequence of steps that can potentially be parallelized
//! - `Step`: Single gate operation with its inputs and output wire

use crate::handles::{InputId, OutputId, Wire};

/// Execution plan for a circuit.
///
/// Contains multiple independent partitions that can be executed in parallel.
/// Each partition corresponds to a disjoint subcircuit.
#[derive(Debug, Clone)]
pub struct ExecutionPlan<T> {
    /// Independent partitions that can be executed in parallel.
    pub partitions: Vec<Partition<T>>,
}

/// Self-contained execution plan for a single subcircuit.
///
/// A partition has its own memory space (wires) and can be executed
/// independently of other partitions.
#[derive(Debug, Clone)]
pub struct Partition<T> {
    /// Sequence of layers to execute in order.
    pub layers: Vec<Layer<T>>,
    /// Number of wires needed for this partition's execution.
    pub memory_size: usize,
    /// Mapping from global input IDs to local wire assignments.
    pub input_bindings: Vec<(InputId, Wire)>,
    /// Mapping from global output IDs to local wire assignments.
    pub output_bindings: Vec<(OutputId, Wire)>,
}

/// A layer of operations that can potentially be executed in parallel.
///
/// Currently, each layer contains a single step, but the structure
/// supports future fine-grained parallelism.
#[derive(Debug, Clone)]
pub struct Layer<T> {
    /// Steps that are independent and can be executed concurrently.
    pub steps: Vec<Step<T>>,
}

/// A single gate operation with its wire assignments.
#[derive(Debug, Clone)]
pub struct Step<T> {
    /// The gate operation to execute.
    pub gate: T,
    /// Input wires for this operation.
    pub inputs: Vec<Wire>,
    /// Output wire where the result is stored.
    pub output: Wire,
}
