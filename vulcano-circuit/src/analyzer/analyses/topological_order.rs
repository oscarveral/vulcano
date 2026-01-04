//! Topological Order Analysis
//!
//! Computes a valid execution order for circuit operations using Kahn's algorithm.
//! The order respects data dependencies: an operation appears after all operations
//! that produce its input values.

use std::collections::{HashMap, VecDeque};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{Circuit, Operation},
    error::{Error, Result},
    gate::Gate,
};

/// Result of topological order analysis.
pub struct TopologicalOrder {
    /// Operations in valid execution order.
    order: Vec<Operation>,
}

impl TopologicalOrder {
    /// Get the operations in topological order.
    pub fn operations(&self) -> &[Operation] {
        &self.order
    }

    /// Iterate over operations in topological order.
    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.order.iter()
    }
}

impl<G: Gate> Analysis<G> for TopologicalOrder {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        // Step 1. Storage used to map each operation to its in-degree.
        let mut in_degree: HashMap<Operation, usize> = HashMap::new();

        // Step 2. Initialize all operations with zero in-degree.
        for op in circuit.all_operations() {
            in_degree.insert(op, 0);
        }

        // Step 3. Build edges: for each value, increment in-degree for each consumer.
        for (_, value) in circuit.all_values() {
            for usage in value.get_uses() {
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
            for value_id in circuit.produced_values(op) {
                let value = circuit.value(value_id)?;
                for usage in value.get_uses() {
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

        Ok(TopologicalOrder { order })
    }
}
