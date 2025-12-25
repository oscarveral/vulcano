//! Topological analysis
//!
//! The analysis computes a total ordering of all nodes in the circuit
//! (inputs, gates, clones, outputs) that respects data dependencies.
//! - Inputs: positioned just before their first consumer
//! - Gates/Clones: topologically ordered by dependencies
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
/// Contains a total ordering of all nodes (inputs, gates, clones, outputs) in execution order.
/// Processing nodes in this order guarantees that all dependencies are satisfied.
pub(super) struct TopologicalOrder {
    /// The order of all nodes (NodeId), including inputs, gates, clones, and outputs.
    order: Vec<NodeId>,
}

impl TopologicalOrder {
    /// Returns the topological order of all nodes.
    ///
    /// The returned vector contains all node IDs (inputs, gates, clones, outputs) in execution order.
    pub(super) fn get_order(&self) -> &Vec<NodeId> {
        &self.order
    }
}

impl Analysis for TopologicalOrder {
    type Output = Self;

    /// Compute the topological order of all nodes using Kahn's algorithm.
    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        // Step 1. Calculate in-degrees for processing nodes (gates, clones).
        let mut in_degrees: HashMap<NodeId, usize> = HashMap::new();

        for gate_id in circuit.get_gate_ids() {
            in_degrees.insert(gate_id, 0);
        }
        for clone_id in circuit.get_clone_ids() {
            in_degrees.insert(clone_id, 0);
        }

        // Count incoming edges from gates.
        for node_id in circuit.get_gate_ids() {
            let node = circuit.get_gate(node_id)?;
            for (_, consumer) in node.get_destinations() {
                if let Some(degree) = in_degrees.get_mut(&consumer) {
                    *degree += 1;
                }
            }
        }

        // Count incoming edges from clones.
        for clone_id in circuit.get_clone_ids() {
            let clone_node = circuit.get_clone(clone_id)?;
            for (_, consumer) in clone_node.get_destinations() {
                if let Some(degree) = in_degrees.get_mut(&consumer) {
                    *degree += 1;
                }
            }
        }

        // Count incoming edges from inputs.
        for input_id in circuit.get_input_ids() {
            let input = circuit.get_input(input_id)?;
            for consumer in input.get_destinations() {
                if let Some(degree) = in_degrees.get_mut(&consumer) {
                    *degree += 1;
                }
            }
        }

        // Step 2. Initialize queue with nodes having in-degree 0.
        let mut queue = VecDeque::new();
        for (node_id, &degree) in &in_degrees {
            if degree == 0 {
                queue.push_back(*node_id);
            }
        }

        // Step 3. Process queue.
        let mut final_order = Vec::with_capacity(circuit.node_count());
        let mut processed_inputs = HashSet::new();
        let mut processed_outputs = HashSet::new();

        while let Some(node_id) = queue.pop_front() {
            let node = circuit.get_node(node_id)?;
            match node {
                Node::Gate {
                    node: gate_internal,
                } => {
                    // A. Add any input sources not yet processed.
                    for source in gate_internal.get_sources() {
                        if circuit.is_input(source)? && !processed_inputs.contains(&source) {
                            final_order.push(source);
                            processed_inputs.insert(source);
                        }
                    }

                    // B. Add the node itself.
                    final_order.push(node_id);

                    // C. Add any output destinations not yet processed.
                    for (_, dest) in gate_internal.get_destinations() {
                        if circuit.is_output(dest)? && !processed_outputs.contains(&dest) {
                            final_order.push(dest);
                            processed_outputs.insert(dest);
                        }
                    }

                    // D. Decrease in-degrees and enqueue ready nodes.
                    for (_, consumer) in gate_internal.get_destinations() {
                        if let Some(degree) = in_degrees.get_mut(&consumer) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(consumer);
                            }
                        }
                    }
                }
                Node::Clone {
                    node: clone_internal,
                } => {
                    // A. Add any input source not yet processed.
                    if let Some(source) = clone_internal.get_source() {
                        if circuit.is_input(source)? && !processed_inputs.contains(&source) {
                            final_order.push(source);
                            processed_inputs.insert(source);
                        }
                    }

                    // B. Add the clone itself.
                    final_order.push(node_id);

                    // C. Add any output destinations not yet processed.
                    for (_, dest) in clone_internal.get_destinations() {
                        if circuit.is_output(dest)? && !processed_outputs.contains(&dest) {
                            final_order.push(dest);
                            processed_outputs.insert(dest);
                        }
                    }

                    // D. Decrease in-degrees and enqueue ready nodes.
                    for (_, consumer) in clone_internal.get_destinations() {
                        if let Some(degree) = in_degrees.get_mut(&consumer) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(consumer);
                            }
                        }
                    }
                }
                Node::Input { .. } | Node::Output { .. } => {
                    // These should not appear in queue.
                    return Err(Error::InconsistentOrder);
                }
            }
        }

        // Step 4. Add any remaining inputs not consumed by any gate/clone.
        for input_id in circuit.get_input_ids() {
            if !processed_inputs.contains(&input_id) {
                final_order.push(input_id);
            }
        }

        // Step 5. Add any remaining outputs not sourced by any gate/clone.
        for output_id in circuit.get_output_ids() {
            if !processed_outputs.contains(&output_id) {
                final_order.push(output_id);
            }
        }

        // Step 6. Check for cycles.
        if final_order.len() != circuit.node_count() {
            return Err(Error::CycleDetected);
        }

        Ok(TopologicalOrder { order: final_order })
    }
}
