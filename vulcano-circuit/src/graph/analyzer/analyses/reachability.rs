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
    handles::{Operation, Source},
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
    /// Set of operations that are reachable.
    type Output = HashSet<Operation>;

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
        let reachable: HashSet<Operation> = forward_reachable
            .intersection(&backward_reachable)
            .map(|&idx| Operation::new(idx))
            .collect();

        Ok(reachable)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::reachability::Reachability},
            builder::Builder,
            circuit::Circuit,
        },
        handles::{Input, Operation, Source},
    };

    enum TestGate {
        Negate,
        Addition,
    }

    impl Gate for TestGate {
        fn arity(&self) -> usize {
            match self {
                TestGate::Negate => 1,
                TestGate::Addition => 2,
            }
        }

        fn name(&self) -> &str {
            match self {
                TestGate::Negate => "Negate",
                TestGate::Addition => "Addition",
            }
        }
    }

    #[test]
    fn simple_circuit() {
        // Layout: input -> negate -> output.

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&gate));
    }

    #[test]
    fn complex_circuit_all_reachable() {
        // Layout:
        // input1 -> negate1 -> \
        //                        addition -> output
        // input2 -> negate2 -> /

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_input_to_gate(input2, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition).unwrap();
        builder.connect_gate_to_gate(negate2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&negate1));
        assert!(reachable.contains(&negate2));
        assert!(reachable.contains(&addition));
    }

    #[test]
    fn unreachable_from_inputs() {
        // Manually construct a circuit where negate2 is not reachable from inputs.
        //
        // input -> negate1 -> addition -> output
        //          negate2 -/

        let negate1_gate = TestGate::Negate;
        let negate2_gate = TestGate::Negate;
        let addition_gate = TestGate::Addition;

        let circuit_input = Input::new(0);

        let circuit = Circuit {
            gate_entries: vec![
                (negate1_gate, vec![Source::Input(circuit_input)]),
                (
                    negate2_gate,
                    vec![Source::Input(Input::new(999))], // Unreachable: depends on non-existent input
                ),
                (
                    addition_gate,
                    vec![
                        Source::Gate(Operation::new(0)), // negate1
                        Source::Gate(Operation::new(1)), // negate2
                    ],
                ),
            ],
            input_count: 1,
            connected_outputs: vec![Operation::new(2)], // addition
        };

        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&Operation::new(0)));
        assert!(reachable.contains(&Operation::new(2)));
        assert!(!reachable.contains(&Operation::new(1)));
    }

    #[test]
    fn unreachable_from_outputs() {
        // Layout:
        //
        // input -> negate1 -> negate2 (no output connection)
        //       \-> addition -> output

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_input_to_gate(input1, addition).unwrap();
        builder.connect_input_to_gate(input2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&addition));
        assert!(!reachable.contains(&negate1));
        assert!(!reachable.contains(&negate2));
    }

    #[test]
    fn multiple_outputs() {
        // Layout:
        //
        // input -> negate1 -> output1
        //       \-> negate2 -> output2

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        let output2 = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_output(negate1, output1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&negate1));
        assert!(reachable.contains(&negate2));
    }

    #[test]
    fn diamond_pattern() {
        // Layout:
        //
        // input -> negate1 -> \
        //      \-> negate2 -> / -> addition -> output

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_input_to_gate(input, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition).unwrap();
        builder.connect_gate_to_gate(negate2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<Reachability>(&circuit);

        assert!(result.is_ok());
        let reachable = result.unwrap();
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&negate1));
        assert!(reachable.contains(&negate2));
        assert!(reachable.contains(&addition));
    }
}
