//! Topological analysis
//!
//! The analysis computes a total ordering of all nodes in the circuit
//! (inputs, gates, clones, drops, outputs) that respects data dependencies.
//! - Inputs: positioned just before their first consumer
//! - Gates/Clones: topologically ordered by dependencies
//! - Drops: positioned after all their dependencies complete
//! - Outputs: positioned just after their source

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{Circuit, Node},
    error::{Error, Result},
    gate::Gate,
    handles::NodeId,
};

/// Topological order of all nodes in the circuit.
///
/// Contains a total ordering of all nodes (inputs, gates, clones, drops, outputs) in execution order.
/// Processing nodes in this order guarantees that all dependencies are satisfied.
pub(super) struct TopologicalOrder {
    /// The order of all nodes (NodeId), including inputs, gates, clones, drops, and outputs.
    order: Vec<NodeId>,
}

impl TopologicalOrder {
    /// Returns the topological order of all nodes.
    ///
    /// The returned vector contains all node IDs (inputs, gates, clones, drops, outputs) in execution order.
    pub(super) fn get_order(&self) -> &Vec<NodeId> {
        &self.order
    }
}

impl Analysis for TopologicalOrder {
    type Output = Self;

    /// Compute the topological order of all nodes using Kahn's algorithm.
    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        // Step 1. Calculate in-degrees for processing nodes (gates, clones, drops).
        let mut in_degrees: HashMap<NodeId, usize> = HashMap::new();

        // Initialize all gates and clones with in-degree 0.
        for gate_id in circuit.get_gate_ids() {
            in_degrees.insert(gate_id, 0);
        }
        for clone_id in circuit.get_clone_ids() {
            in_degrees.insert(clone_id, 0);
        }
        // Initialize drops with in-degree = number of dependencies.
        for drop_id in circuit.get_drop_ids() {
            let drop_node = circuit.get_drop(drop_id)?;
            let dep_count = drop_node.get_dependencies().count();
            in_degrees.insert(drop_id, dep_count);
        }

        // Count incoming edges from gates to gates/clones.
        for node_id in circuit.get_gate_ids() {
            let node = circuit.get_gate(node_id)?;
            for (_, consumer) in node.get_destinations() {
                if circuit.is_gate(consumer)? || circuit.is_clone(consumer)? {
                    *in_degrees.entry(consumer).or_insert(0) += 1;
                }
            }
        }

        // Count incoming edges from clones to gates/clones.
        for clone_id in circuit.get_clone_ids() {
            let clone_node = circuit.get_clone(clone_id)?;
            for (_, consumer) in clone_node.get_destinations() {
                if circuit.is_gate(consumer)? || circuit.is_clone(consumer)? {
                    *in_degrees.entry(consumer).or_insert(0) += 1;
                }
            }
        }

        // Count incoming edges from inputs to gates/clones.
        for input_id in circuit.get_input_ids() {
            let input = circuit.get_input(input_id)?;
            for consumer in input.get_destinations() {
                if circuit.is_gate(consumer)? || circuit.is_clone(consumer)? {
                    *in_degrees.entry(consumer).or_insert(0) += 1;
                }
            }
        }

        // Step 2. Track first consumer for each input.
        let mut first_consumer = HashMap::new();
        for input_id in circuit.get_input_ids() {
            let input = circuit.get_input(input_id)?;
            let mut destinations = input.get_destinations();
            first_consumer.insert(input_id, destinations.next());
        }

        // Step 3. Track source for each output.
        let mut output_source = HashMap::new();
        for output_id in circuit.get_output_ids() {
            let output = circuit.get_output(output_id)?;
            output_source.insert(output_id, output.get_source());
        }

        // Step 4. Build reverse dependency map: node -> drops that depend on it.
        let mut drop_dependents: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for drop_id in circuit.get_drop_ids() {
            let drop_node = circuit.get_drop(drop_id)?;
            for dep in drop_node.get_dependencies() {
                drop_dependents.entry(dep).or_default().push(drop_id);
            }
        }

        // Step 5. Initialize queue with nodes having in-degree 0.
        let mut queue = VecDeque::new();
        for (node_id, &degree) in &in_degrees {
            if degree == 0 {
                queue.push_back(*node_id);
            }
        }

        // Step 6. Process queue with special handling.
        let mut final_order = Vec::with_capacity(circuit.node_count());
        let mut processed_inputs = HashSet::new();
        let mut processed_outputs = HashSet::new();

        while let Some(node_id) = queue.pop_front() {
            let node = circuit.get_node(node_id)?;
            match node {
                Node::Gate {
                    node: gate_internal,
                } => {
                    // A. Add inputs where this node is the first consumer.
                    for input_id in circuit.get_input_ids() {
                        if !processed_inputs.contains(&input_id) {
                            if let Some(first) = first_consumer.get(&input_id) {
                                if *first == Some(node_id) {
                                    final_order.push(input_id);
                                    processed_inputs.insert(input_id);
                                }
                            }
                        }
                    }

                    // B. Add the node itself.
                    final_order.push(node_id);

                    // C. Add outputs where this node is the source.
                    for output_id in circuit.get_output_ids() {
                        if !processed_outputs.contains(&output_id) {
                            if let Some(source) = output_source.get(&output_id) {
                                if *source == Some(node_id) {
                                    final_order.push(output_id);
                                    processed_outputs.insert(output_id);
                                }
                            }
                        }
                    }

                    // D. Decrease in-degrees for data consumers and enqueue ready nodes.
                    for (_, consumer) in gate_internal.get_destinations() {
                        if let Some(degree) = in_degrees.get_mut(&consumer) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(consumer);
                            }
                        }
                    }

                    // E. Decrease in-degrees for drops depending on this node.
                    if let Some(drops) = drop_dependents.get(&node_id) {
                        for &drop_id in drops {
                            if let Some(degree) = in_degrees.get_mut(&drop_id) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(drop_id);
                                }
                            }
                        }
                    }
                }
                Node::Clone {
                    node: clone_internal,
                } => {
                    // A. Add inputs where this clone is the first consumer.
                    for input_id in circuit.get_input_ids() {
                        if !processed_inputs.contains(&input_id) {
                            if let Some(first) = first_consumer.get(&input_id) {
                                if *first == Some(node_id) {
                                    final_order.push(input_id);
                                    processed_inputs.insert(input_id);
                                }
                            }
                        }
                    }

                    // B. Add the clone itself.
                    final_order.push(node_id);

                    // C. Add outputs where this clone is the source.
                    for output_id in circuit.get_output_ids() {
                        if !processed_outputs.contains(&output_id) {
                            if let Some(source) = output_source.get(&output_id) {
                                if *source == Some(node_id) {
                                    final_order.push(output_id);
                                    processed_outputs.insert(output_id);
                                }
                            }
                        }
                    }

                    // D. Decrease in-degrees for data consumers and enqueue ready nodes.
                    for (_, consumer) in clone_internal.get_destinations() {
                        if let Some(degree) = in_degrees.get_mut(&consumer) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(consumer);
                            }
                        }
                    }

                    // E. Decrease in-degrees for drops depending on this node.
                    if let Some(drops) = drop_dependents.get(&node_id) {
                        for &drop_id in drops {
                            if let Some(degree) = in_degrees.get_mut(&drop_id) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(drop_id);
                                }
                            }
                        }
                    }
                }
                Node::Drop { .. } => {
                    // Drop nodes: add to order when all dependencies complete.
                    final_order.push(node_id);
                }
                Node::Input { .. } => {
                    // Inputs should not appear in queue - they're added before first consumer.
                    return Err(Error::InconsistentOrder);
                }
                Node::Output { .. } => {
                    // Outputs should not appear in queue - they're added after source.
                    return Err(Error::InconsistentOrder);
                }
            }
        }

        // Step 7. Add any remaining inputs that weren't consumed by any node.
        for input_id in circuit.get_input_ids() {
            if !processed_inputs.contains(&input_id) {
                final_order.push(input_id);
            }
        }

        // Step 8. Check for cycles.
        if final_order.len() != circuit.node_count() {
            return Err(Error::CycleDetected);
        }

        Ok(TopologicalOrder { order: final_order })
    }
}
