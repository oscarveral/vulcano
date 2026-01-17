//! Topological Order Analysis
//!
//! Computes a valid execution order for circuit operations using Kahn's algorithm.
//! The order respects data dependencies: an operation appears after all operations
//! that produce its input values.

use std::collections::{HashMap, VecDeque};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{
        Circuit,
        operations::Operation,
        subcircuit::{CircuitId, Subcircuit},
    },
    error::{Error, Result},
    gate::Gate,
};

/// Per-subcircuit topological order result.
pub struct SubcircuitOrder {
    /// Operations in valid execution order.
    order: Vec<Operation>,
}

impl SubcircuitOrder {
    /// Get the operations in topological order.
    pub fn operations(&self) -> &[Operation] {
        &self.order
    }

    /// Iterate over operations in topological order.
    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.order.iter()
    }
}

/// Result of topological order analysis.
pub struct TopologicalOrder {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitOrder>,
}

impl TopologicalOrder {
    /// Get the topological order for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitOrder> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitOrder)> {
        self.results.iter().map(|(&id, order)| (id, order))
    }
}

impl<G: Gate> Analysis<G> for TopologicalOrder {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let order = compute_topological_order(subcircuit)?;
            results.insert(subcircuit.id(), order);
        }

        Ok(TopologicalOrder { results })
    }
}

/// Compute topological order for a single subcircuit using Kahn's algorithm.
fn compute_topological_order<G: Gate>(subcircuit: &Subcircuit<G>) -> Result<SubcircuitOrder> {
    // Step 1. Storage used to map each operation to its in-degree.
    let mut in_degree: HashMap<Operation, usize> = HashMap::new();

    // Step 2. Initialize all operations with zero in-degree.
    for op in subcircuit.all_operations() {
        in_degree.insert(op, 0);
    }

    // Step 3. Build edges: for each value, increment in-degree for each consumer.
    for (_, value) in subcircuit.all_values() {
        for usage in value.get_destinations() {
            let consumer_op: Operation = usage.get_consumer().into();
            // Each consumer depends on the producer.
            *in_degree.entry(consumer_op).or_insert(0) += 1;
        }
    }

    // Step 4. Kahn's algorithm.
    let mut queue: VecDeque<Operation> = VecDeque::new();
    let mut order: Vec<Operation> = Vec::new();

    // Substep A. Start with operations that have no dependencies.
    for (&op, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(op);
        }
    }

    // Substep B. Process each operation in the queue.
    while let Some(op) = queue.pop_front() {
        order.push(op);

        // Substep C. Find all values produced by this operation and reduce in-degree of consumers.
        for value_id in subcircuit.produced_values(op)? {
            let value = subcircuit.value(value_id)?;
            for usage in value.get_destinations() {
                let consumer_op: Operation = usage.get_consumer().into();
                if let Some(deg) = in_degree.get_mut(&consumer_op) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(consumer_op);
                    }
                }
            }
        }
    }

    // Step 5. Check for cycles.
    if order.len() != in_degree.len() {
        let cycle_ops: Vec<Operation> = in_degree
            .into_iter()
            .filter(|(_, deg)| *deg > 0)
            .map(|(op, _)| op)
            .collect();
        return Err(Error::CycleDetected(cycle_ops));
    }

    Ok(SubcircuitOrder { order })
}
