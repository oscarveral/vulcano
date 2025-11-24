use std::any::TypeId;

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analyzer, analyses::reachability::Reachability},
        circuit::Circuit,
    },
};

/// Optimization pass that eliminates dead gates from the circuit.
///
/// Dead gates refer to gates that are not reachable from the circuit inputs
/// and cannot reach the circuit outputs. This pass relies on the
/// [`Reachability`] analysis to identify such gates.
pub fn dead_gate_elimination<T: Gate>(
    mut circuit: Circuit<T>,
    analyzer: &mut Analyzer<T>,
) -> Result<(Circuit<T>, Vec<TypeId>)> {
    // Get the set of reachable gates from the Reachability analysis.
    let reachable_gates = analyzer.get::<Reachability>(&circuit)?;

    // Retain only the gates that are reachable.
    circuit.gate_entries = circuit
        .gate_entries
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| reachable_gates.contains(idx))
        .map(|(_, entry)| entry)
        .collect();

    // Invalidate all analyses since the circuit has changed.
    Ok((circuit, Vec::new()))
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::Analyzer, builder::Builder, circuit::Circuit,
            optimizer::passes::dead::dead_gate_elimination,
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
    fn no_dead_gates() {
        // Simple circuit with no dead code.
        // Layout: input -> negate -> output

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let gate_count_before = circuit.gate_entries.len();

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), gate_count_before);
        assert_eq!(optimized.gate_entries.len(), 1);
    }

    #[test]
    fn all_gates_reachable() {
        // Complex circuit where all gates are reachable.
        // Layout:
        //
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
        let gate_count_before = circuit.gate_entries.len();

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), gate_count_before);
        assert_eq!(optimized.gate_entries.len(), 3);
    }

    #[test]
    fn unreachable_from_outputs() {
        // Circuit with gates not reaching outputs (dead branch).
        // Layout:
        //
        // input -> negate1 -> negate2 (not connected to output - DEAD)
        //       -> addition -> output
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
        let gate_count_before = circuit.gate_entries.len();
        assert_eq!(gate_count_before, 3); // negate1, negate2, addition

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), 1);
        assert_eq!(optimized.gate_entries[0].0.name(), "Addition");
    }

    #[test]
    fn unreachable_from_inputs() {
        // Circuit with gates not reachable from inputs.
        // Manually constructed to have unreachable gates.

        let negate1_gate = TestGate::Negate;
        let negate2_gate = TestGate::Negate;
        let addition_gate = TestGate::Addition;

        let circuit_input = Input::new(0);

        let circuit = Circuit {
            gate_entries: vec![
                (negate1_gate, vec![Source::Input(circuit_input)]),
                (
                    negate2_gate,
                    vec![Source::Input(Input::new(999))], // Unreachable: non-existent input
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

        let gate_count_before = circuit.gate_entries.len();
        assert_eq!(gate_count_before, 3);

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), 2);

        let names: Vec<&str> = optimized
            .gate_entries
            .iter()
            .map(|(g, _)| g.name())
            .collect();
        assert!(names.contains(&"Negate"));
        assert!(names.contains(&"Addition"));
    }

    #[test]
    fn multiple_outputs_partial_dead() {
        // Circuit with multiple outputs where some branches are used.
        // Layout:
        // input -> negate1 -> output1
        //       -> negate2 -> negate3

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let negate3 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_output(negate1, output1).unwrap();
        builder.connect_input_to_gate(input, negate2).unwrap();
        builder.connect_gate_to_gate(negate2, negate3).unwrap();

        let circuit = builder.finalize().unwrap();
        let gate_count_before = circuit.gate_entries.len();
        assert_eq!(gate_count_before, 3);

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), 1);
        assert_eq!(optimized.gate_entries[0].0.name(), "Negate");
    }

    #[test]
    fn diamond_pattern_no_dead() {
        // Diamond pattern - all gates reachable.
        // Layout:
        //
        // input -> negate1 -> \
        //       -> negate2 -> / -> addition -> output

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
        let gate_count_before = circuit.gate_entries.len();

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), gate_count_before);
        assert_eq!(optimized.gate_entries.len(), 3);
    }

    #[test]
    fn complex_circuit_with_dead_branches() {
        // Complex circuit with multiple dead branches.
        // Layout:
        //
        //     input1 -> negate1 -> \
        //                           addition1 -> output
        //     input2 -> negate2 -> /
        //
        //     input3 -> negate3 -> negate4

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let input3 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let negate3 = builder.add_gate(TestGate::Negate);
        let negate4 = builder.add_gate(TestGate::Negate);
        let addition1 = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        // Live branch.
        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_input_to_gate(input2, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition1).unwrap();
        builder.connect_gate_to_gate(negate2, addition1).unwrap();
        builder.connect_gate_to_output(addition1, output).unwrap();

        // Dead branch.
        builder.connect_input_to_gate(input3, negate3).unwrap();
        builder.connect_gate_to_gate(negate3, negate4).unwrap();

        let circuit = builder.finalize().unwrap();
        let gate_count_before = circuit.gate_entries.len();
        assert_eq!(gate_count_before, 5);

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), 3);

        let names: Vec<&str> = optimized
            .gate_entries
            .iter()
            .map(|(g, _)| g.name())
            .collect();
        assert!(names.contains(&"Negate"));
        assert!(names.contains(&"Addition"));
    }

    #[test]
    fn multiple_outputs_all_used() {
        // Circuit with multiple outputs, all gates used.
        // Layout:
        //
        // input -> negate1 -> output1
        //              |-> negate2 -> output2

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
        let gate_count_before = circuit.gate_entries.len();

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        assert_eq!(optimized.gate_entries.len(), gate_count_before);
        assert_eq!(optimized.gate_entries.len(), 2);
    }

    #[test]
    fn preserves_circuit_structure() {
        // Verify that dead code elimination preserves other circuit properties.

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let dead_gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        // Dead gate with dummy input.
        builder.connect_input_to_gate(input, dead_gate).unwrap();

        let circuit = builder.finalize().unwrap();

        let input_count = circuit.input_count;
        let output_count = circuit.connected_outputs.len();

        let mut analyzer = Analyzer::new();
        let (optimized, _) = dead_gate_elimination(circuit, &mut analyzer).unwrap();

        // Verify inputs and outputs are preserved.
        assert_eq!(optimized.input_count, input_count);
        assert_eq!(optimized.connected_outputs.len(), output_count);

        // Only the live gate remains.
        assert_eq!(optimized.gate_entries.len(), 1);
    }
}
