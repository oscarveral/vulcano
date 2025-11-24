//! Reachability analysis for circuits.
//!
//! This analysis computes which gates are reachable from the circuit inputs and outputs,
//! i.e., which gates contribute to the final computation. Gates that are not
//! reachable are "dead" and can be eliminated by optimization passes.

use std::collections::{HashSet, VecDeque};

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::Circuit,
    },
    handles::Source,
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
        // Step 1: Forward reachability - BFS from inputs.
        let mut forward_reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from gates that consume valid circuit inputs.
        for (gate_idx, (_, sources)) in circuit.gate_entries.iter().enumerate() {
            let has_input_dependency = sources.iter().any(|source| {
                if let Source::Input(input) = source {
                    input.id() < circuit.input_count
                } else {
                    false
                }
            });

            if has_input_dependency {
                forward_reachable.insert(gate_idx);
                queue.push_back(gate_idx);
            }
        }

        // BFS forward: mark all gates reachable from inputs.
        while let Some(producer_idx) = queue.pop_front() {
            // Find gates that consume this gate's output.
            for (consumer_idx, (_, sources)) in circuit.gate_entries.iter().enumerate() {
                if forward_reachable.contains(&consumer_idx) {
                    // Already visited.
                    continue;
                }

                let consumes_output = sources
                    .iter()
                    .any(|source| matches!(source, Source::Gate(op) if op.id() == producer_idx));

                if consumes_output {
                    forward_reachable.insert(consumer_idx);
                    queue.push_back(consumer_idx);
                }
            }
        }

        // Step 2: Backward reachability - BFS from outputs.
        let mut backward_reachable = HashSet::new();

        // Start from gates that produce circuit outputs.
        for output_op in &circuit.connected_outputs {
            let gate_idx = output_op.id();
            if backward_reachable.insert(gate_idx) {
                queue.push_back(gate_idx);
            }
        }

        // BFS backward: mark all gates that outputs depend on.
        while let Some(gate_idx) = queue.pop_front() {
            let sources = &circuit.gate_entries[gate_idx].1;
            for source in sources {
                // If this source is another gate, mark it reachable.
                if let Source::Gate(producer_op) = source {
                    let producer_idx = producer_op.id();
                    if backward_reachable.insert(producer_idx) {
                        queue.push_back(producer_idx);
                    }
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
