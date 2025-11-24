use crate::{
    error::Error,
    gate::Gate,
    graph::{
        analyzer::{
            Analyzer,
            analyses::{reachability::Reachability, topological::TopologicalOrder},
        },
        builder::Builder,
    },
};

enum TestGate {
    Negate,
    Addition,
    Rotate,
}

impl Gate for TestGate {
    fn arity(&self) -> usize {
        match self {
            TestGate::Negate => 1,
            TestGate::Addition => 2,
            TestGate::Rotate => 2,
        }
    }

    fn name(&self) -> &str {
        match self {
            TestGate::Negate => "Negate",
            TestGate::Addition => "Addition",
            TestGate::Rotate => "Rotate",
        }
    }
}

#[test]
fn reachability_simple_circuit() {
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
    assert!(reachable.contains(&gate.id()));
}

#[test]
fn reachability_complex_circuit_all_reachable() {
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
    assert!(reachable.contains(&negate1.id()));
    assert!(reachable.contains(&negate2.id()));
    assert!(reachable.contains(&addition.id()));
}

#[test]
fn reachability_unreachable_from_inputs() {
    // Manually construct a circuit where negate2 is not reachable from inputs.
    //
    // input -> negate1 -> addition -> output
    //          negate2 -/

    use crate::graph::circuit::Circuit;
    use crate::handles::{Input, Operation, Source};

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
    assert!(reachable.contains(&0));
    assert!(reachable.contains(&2));
    assert!(!reachable.contains(&1));
}

#[test]
fn reachability_unreachable_from_outputs() {
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
    assert!(reachable.contains(&addition.id()));
    assert!(!reachable.contains(&negate1.id()));
    assert!(!reachable.contains(&negate2.id()));
}

#[test]
fn reachability_multiple_outputs() {
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
    assert!(reachable.contains(&negate1.id()));
    assert!(reachable.contains(&negate2.id()));
}

#[test]
fn reachability_diamond_pattern() {
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
    assert!(reachable.contains(&negate1.id()));
    assert!(reachable.contains(&negate2.id()));
    assert!(reachable.contains(&addition.id()));
}

#[test]
fn topological_simple_circuit() {
    // Layout: input -> negate -> output

    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.finalize().unwrap();
    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 1);
    assert_eq!(topo[0], gate.id());
}

#[test]
fn topological_linear_circuit() {
    // Layout: input -> negate1 -> negate2 -> negate3 -> output

    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let negate1 = builder.add_gate(TestGate::Negate);
    let negate2 = builder.add_gate(TestGate::Negate);
    let negate3 = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input, negate1).unwrap();
    builder.connect_gate_to_gate(negate1, negate2).unwrap();
    builder.connect_gate_to_gate(negate2, negate3).unwrap();
    builder.connect_gate_to_output(negate3, output).unwrap();

    let circuit = builder.finalize().unwrap();
    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 3);
    let pos1 = topo.iter().position(|&x| x == negate1.id()).unwrap();
    let pos2 = topo.iter().position(|&x| x == negate2.id()).unwrap();
    let pos3 = topo.iter().position(|&x| x == negate3.id()).unwrap();
    assert!(pos1 < pos2);
    assert!(pos2 < pos3);
}

#[test]
fn topological_complex_circuit() {
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
    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 3);
    let pos1 = topo.iter().position(|&x| x == negate1.id()).unwrap();
    let pos2 = topo.iter().position(|&x| x == negate2.id()).unwrap();
    let pos_add = topo.iter().position(|&x| x == addition.id()).unwrap();
    assert!(pos1 < pos_add);
    assert!(pos2 < pos_add);
}

#[test]
fn topological_diamond_pattern() {
    // Layout:
    //
    // input -> negate1 -> \
    //       \-> negate2 -> / -> addition -> output

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
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 3);
    let pos1 = topo.iter().position(|&x| x == negate1.id()).unwrap();
    let pos2 = topo.iter().position(|&x| x == negate2.id()).unwrap();
    let pos_add = topo.iter().position(|&x| x == addition.id()).unwrap();
    assert!(pos1 < pos_add);
    assert!(pos2 < pos_add);
}

#[test]
fn topological_multiple_outputs() {
    // Layout:
    //
    // input -> negate1 -> output1
    //                  \-> negate2 -> output2

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
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 2);
    let pos1 = topo.iter().position(|&x| x == negate1.id()).unwrap();
    let pos2 = topo.iter().position(|&x| x == negate2.id()).unwrap();
    assert!(pos1 < pos2);
}

#[test]
fn topological_cycle_detection_indirect() {
    // This test creates a circuit with a cycle by manually building the circuit
    // and bypassing validation.

    use crate::graph::circuit::Circuit;
    use crate::handles::{Operation, Source};

    let gate1 = TestGate::Negate;
    let gate2 = TestGate::Negate;
    let gate3 = TestGate::Negate;

    let circuit = Circuit {
        gate_entries: vec![
            (gate1, vec![Source::Gate(Operation::new(2))]), // gate1 depends on gate3
            (gate2, vec![Source::Gate(Operation::new(0))]), // gate2 depends on gate1
            (gate3, vec![Source::Gate(Operation::new(1))]), // gate3 depends on gate2 -> cycle!
        ],
        input_count: 0,
        connected_outputs: vec![Operation::new(2)],
    };

    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::CycleDetected(ops) => {
            assert_eq!(ops.len(), 3);
        }
        _ => panic!("Expected CycleDetected error"),
    }
}

#[test]
fn topological_cycle_detection_self_loop() {
    // Create a gate that references its own output.

    use crate::graph::circuit::Circuit;
    use crate::handles::{Operation, Source};

    let gate = TestGate::Negate;

    // gate uses its own output
    let circuit = Circuit {
        gate_entries: vec![(gate, vec![Source::Gate(Operation::new(0))])],
        input_count: 0,
        connected_outputs: vec![Operation::new(0)],
    };

    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::CycleDetected(ops) => {
            assert_eq!(ops.len(), 1);
        }
        _ => panic!("Expected CycleDetected error"),
    }
}

#[test]
fn topological_complex_valid_dag() {
    // More complex DAG:
    //     input1 -> negate1 -> \
    //                           addition1 -> rotate -> output
    //     input2 -> negate2 -> /              /
    //     input3 --------------------------> /

    let mut builder: Builder<TestGate> = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let input3 = builder.add_input();
    let negate1 = builder.add_gate(TestGate::Negate);
    let negate2 = builder.add_gate(TestGate::Negate);
    let addition1 = builder.add_gate(TestGate::Addition);
    let rotate = builder.add_gate(TestGate::Rotate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input1, negate1).unwrap();
    builder.connect_input_to_gate(input2, negate2).unwrap();
    builder.connect_gate_to_gate(negate1, addition1).unwrap();
    builder.connect_gate_to_gate(negate2, addition1).unwrap();
    builder.connect_gate_to_gate(addition1, rotate).unwrap();
    builder.connect_input_to_gate(input3, rotate).unwrap();
    builder.connect_gate_to_output(rotate, output).unwrap();

    let circuit = builder.finalize().unwrap();
    let mut analyzer = Analyzer::new();
    let result = analyzer.get::<TopologicalOrder>(&circuit);

    assert!(result.is_ok());
    let topo = result.unwrap();
    assert_eq!(topo.len(), 4);

    let pos_neg1 = topo.iter().position(|&x| x == negate1.id()).unwrap();
    let pos_neg2 = topo.iter().position(|&x| x == negate2.id()).unwrap();
    let pos_add = topo.iter().position(|&x| x == addition1.id()).unwrap();
    let pos_rot = topo.iter().position(|&x| x == rotate.id()).unwrap();

    assert!(pos_neg1 < pos_add);
    assert!(pos_neg2 < pos_add);
    assert!(pos_add < pos_rot);
}
