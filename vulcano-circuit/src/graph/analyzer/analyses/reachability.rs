//! Reachability analysis for circuits.
//!
//! This analysis computes which gates are reachable from the circuit inputs and outputs,
//! i.e., which gates contribute to the final computation. Gates that are not
//! reachable are "dead" and can be eliminated by optimization passes.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::{Circuit, Use},
    },
};

/// Analysis that computes which gates are reachable in the circuit.
///
/// A gate is considered reachable if:
/// 1. It is reachable from circuit inputs (forward reachability), AND
/// 2. It can reach circuit outputs (backward reachability)
///
/// Gates that don't satisfy both conditions are "dead" and can be eliminated.
pub struct Reachability;

impl Analysis for Reachability {
    /// Set of gate indices that are reachable.
    type Output = HashSet<usize>;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let n = circuit.gate_entries.len();

        // Build a mapping from wire ID to the gate that produces it.
        let mut wire_to_gate: HashMap<usize, usize> = HashMap::with_capacity(n);
        for (gate_idx, entry) in circuit.gate_entries.iter().enumerate() {
            let output_wire = entry.2.id();
            wire_to_gate.insert(output_wire, gate_idx);
        }

        // Build wire set for circuit inputs.
        let input_wires: HashSet<usize> = circuit.connected_inputs.iter().map(|w| w.id()).collect();

        // Step 1: Forward reachability - BFS from inputs.
        let mut forward_reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from gates that consume circuit inputs.
        for (gate_idx, entry) in circuit.gate_entries.iter().enumerate() {
            let inputs = &entry.1;
            let has_input_dependency = inputs.iter().any(|use_item| {
                let wire_id = match use_item {
                    Use::Read(w) | Use::Consume(w) => w.id(),
                };
                input_wires.contains(&wire_id)
            });

            if has_input_dependency {
                forward_reachable.insert(gate_idx);
                queue.push_back(gate_idx);
            }
        }

        // BFS forward: mark all gates reachable from inputs.
        while let Some(gate_idx) = queue.pop_front() {
            let output_wire = circuit.gate_entries[gate_idx].2.id();

            // Find gates that consume this gate's output.
            for (consumer_idx, entry) in circuit.gate_entries.iter().enumerate() {
                if forward_reachable.contains(&consumer_idx) {
                    // Already visited.
                    continue;
                }

                let consumes_output = entry.1.iter().any(|use_item| {
                    let wire_id = match use_item {
                        Use::Read(w) | Use::Consume(w) => w.id(),
                    };
                    wire_id == output_wire
                });

                if consumes_output {
                    forward_reachable.insert(consumer_idx);
                    queue.push_back(consumer_idx);
                }
            }
        }

        // Step 2: Backward reachability - BFS from outputs.
        let mut backward_reachable = HashSet::new();

        // Start from gates that produce circuit outputs.
        for output_wire in &circuit.connected_outputs {
            if let Some(&gate_idx) = wire_to_gate.get(&output_wire.id())
                && backward_reachable.insert(gate_idx)
            {
                queue.push_back(gate_idx);
            }
        }

        // BFS backward: mark all gates that outputs depend on.
        while let Some(gate_idx) = queue.pop_front() {
            let inputs = &circuit.gate_entries[gate_idx].1;
            for input_use in inputs {
                let input_wire_id = match input_use {
                    Use::Read(w) | Use::Consume(w) => w.id(),
                };

                // If this input is produced by another gate, mark it reachable.
                if let Some(&producer_idx) = wire_to_gate.get(&input_wire_id)
                    && backward_reachable.insert(producer_idx)
                {
                    queue.push_back(producer_idx);
                }
            }
        }

        // Step 3: Intersection - gates must be reachable both ways.
        let reachable: HashSet<usize> = forward_reachable
            .intersection(&backward_reachable)
            .copied()
            .collect();

        Ok(reachable)
    }
}
