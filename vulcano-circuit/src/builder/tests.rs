use crate::{
    builder::Builder,
    error::Error,
    gate::Gate,
    handles::{Input, Node, Output},
};

#[derive(Debug, Clone)]
struct TestGate {
    arity: usize,
}

impl TestGate {
    fn new(arity: usize) -> Self {
        Self { arity }
    }
}

impl Gate for TestGate {
    fn arity(&self) -> usize {
        self.arity
    }
}

#[test]
fn new_builder() {
    let builder: Builder<TestGate> = Builder::new();
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 0);
}

#[test]
fn with_capacity() {
    let builder: Builder<TestGate> = Builder::with_capacity(100, 50, 50);
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 0);
}

#[test]
fn add_gate() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(2));
    let gate2 = builder.add_gate(TestGate::new(3));

    assert_eq!(builder.gate_count(), 2);
    assert_eq!(gate1, Node(0));
    assert_eq!(gate2, Node(1));
}

#[test]
fn add_input() {
    let mut builder: Builder<TestGate> = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();

    assert_eq!(builder.input_count(), 2);
    assert_eq!(input1, Input(0));
    assert_eq!(input2, Input(1));
}

#[test]
fn add_output() {
    let mut builder: Builder<TestGate> = Builder::new();
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    assert_eq!(builder.output_count(), 2);
    assert_eq!(output1, Output(0));
    assert_eq!(output2, Output(1));
}

#[test]
fn connect_input_to_gate() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::new(2));

    let result = builder.connect_input_to_gate(input, gate);
    assert!(result.is_ok());
}

#[test]
fn connect_input_to_nonexistent_gate() {
    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let nonexistent_gate = Node(99);

    let result = builder.connect_input_to_gate(input, nonexistent_gate);
    assert_eq!(result, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn connect_nonexistent_input_to_gate() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(2));
    let nonexistent_input = Input(99);

    let result = builder.connect_input_to_gate(nonexistent_input, gate);
    assert_eq!(result, Err(Error::NonExistentInput(nonexistent_input)));
}

#[test]
fn connect_too_many_inputs_to_gate() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let input3 = builder.add_input();
    let gate = builder.add_gate(TestGate::new(2));

    assert!(builder.connect_input_to_gate(input1, gate).is_ok());
    assert!(builder.connect_input_to_gate(input2, gate).is_ok());

    let result = builder.connect_input_to_gate(input3, gate);
    assert_eq!(result, Err(Error::TooManyConnections { gate, arity: 2 }));
}

#[test]
fn connect_gate_to_gate() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(2));

    let result = builder.connect_gate_to_gate(gate1, gate2);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_gate() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(1));
    let nonexistent_gate = Node(99);

    let result1 = builder.connect_gate_to_gate(nonexistent_gate, gate);
    assert_eq!(result1, Err(Error::NonExistentGate(nonexistent_gate)));

    let result2 = builder.connect_gate_to_gate(gate, nonexistent_gate);
    assert_eq!(result2, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn connect_gate_to_itself() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(2));

    let result = builder.connect_gate_to_gate(gate, gate);
    assert_eq!(result, Err(Error::SelfConnection(gate)));
}

#[test]
fn connect_too_many_gates_to_gate() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(1));
    let target_gate = builder.add_gate(TestGate::new(2));

    assert!(builder.connect_gate_to_gate(gate1, target_gate).is_ok());
    assert!(builder.connect_gate_to_gate(gate2, target_gate).is_ok());

    let result = builder.connect_gate_to_gate(gate3, target_gate);
    assert_eq!(
        result,
        Err(Error::TooManyConnections {
            gate: target_gate,
            arity: 2
        })
    );
}

#[test]
fn connect_gate_to_output() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    let result = builder.connect_gate_to_output(gate, output);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_output() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(1));
    let nonexistent_output = Output(99);

    let result = builder.connect_gate_to_output(gate, nonexistent_output);
    assert_eq!(result, Err(Error::NonExistentOutput(nonexistent_output)));
}

#[test]
fn connect_nonexistent_gate_to_output() {
    let mut builder: Builder<TestGate> = Builder::new();
    let output = builder.add_output();
    let nonexistent_gate = Node(99);

    let result = builder.connect_gate_to_output(nonexistent_gate, output);
    assert_eq!(result, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn output_already_connected() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    assert!(builder.connect_gate_to_output(gate1, output).is_ok());

    let result = builder.connect_gate_to_output(gate2, output);
    assert_eq!(result, Err(Error::OutputAlreadyConnectedToGate(output)));
}

#[test]
fn gate_cannot_connect_to_multiple_outputs() {
    let mut builder = Builder::new();
    let gate = builder.add_gate(TestGate::new(1));
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    assert!(builder.connect_gate_to_output(gate, output1).is_ok());

    let result = builder.connect_gate_to_output(gate, output2);
    assert_eq!(result, Err(Error::GateAlreadyConnectedToOutput(gate)));
}

#[test]
fn mixed_connections() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(2));
    let gate2 = builder.add_gate(TestGate::new(2));
    let gate3 = builder.add_gate(TestGate::new(2));
    let output = builder.add_output();

    assert!(builder.connect_input_to_gate(input1, gate1).is_ok());
    assert!(builder.connect_input_to_gate(input2, gate1).is_ok());
    assert!(builder.connect_gate_to_gate(gate1, gate3).is_ok());
    assert!(builder.connect_gate_to_gate(gate2, gate3).is_ok());
    assert!(builder.connect_gate_to_output(gate3, output).is_ok());
}

#[test]
fn handles_equality() {
    let gate1 = Node(0);
    let gate2 = Node(0);
    let gate3 = Node(1);

    assert_eq!(gate1, gate2);
    assert_ne!(gate1, gate3);

    let input1 = Input(0);
    let input2 = Input(0);
    let input3 = Input(1);

    assert_eq!(input1, input2);
    assert_ne!(input1, input3);

    let output1 = Output(0);
    let output2 = Output(0);
    let output3 = Output(1);

    assert_eq!(output1, output2);
    assert_ne!(output1, output3);
}

#[test]
fn default_builder() {
    let builder: Builder<TestGate> = Builder::default();
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 0);
}

#[test]
fn gate_with_arity_one() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::new(1));

    assert!(builder.connect_input_to_gate(input, gate).is_ok());

    let input2 = builder.add_input();
    let result = builder.connect_input_to_gate(input2, gate);
    assert_eq!(result, Err(Error::TooManyConnections { gate, arity: 1 }));
}

#[test]
fn large_builder() {
    let mut builder = Builder::new();

    let inputs: Vec<_> = (0..100).map(|_| builder.add_input()).collect();
    let gates: Vec<_> = (0..100)
        .map(|_| builder.add_gate(TestGate::new(1)))
        .collect();
    let outputs: Vec<_> = (0..100).map(|_| builder.add_output()).collect();

    for (input, gate) in inputs.iter().zip(gates.iter()) {
        assert!(builder.connect_input_to_gate(*input, *gate).is_ok());
    }

    for (gate, output) in gates.iter().zip(outputs.iter()) {
        assert!(builder.connect_gate_to_output(*gate, *output).is_ok());
    }

    assert_eq!(builder.gate_count(), 100);
    assert_eq!(builder.input_count(), 100);
    assert_eq!(builder.output_count(), 100);
}

#[test]
fn gate_can_have_multiple_forward_connections() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(1));

    assert!(builder.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(builder.connect_gate_to_gate(gate1, gate3).is_ok());
}

#[test]
fn gate_can_connect_to_gates_and_output() {
    let mut builder = Builder::new();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    assert!(builder.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(builder.connect_gate_to_output(gate1, output).is_ok());
}

#[test]
fn build_simple_valid_builder() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_unused_input() {
    let mut builder = Builder::new();
    let _input = builder.add_input();
    let gate = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    let input2 = builder.add_input();
    builder.connect_input_to_gate(input2, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::UnusedInput(Input(0))));
}

#[test]
fn build_unused_output() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let _output = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate2).unwrap();

    let output2 = builder.add_output();
    builder.connect_gate_to_output(gate1, output2).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::UnusedOutput(Output(0))));
}

#[test]
fn build_zero_arity_gate() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(0));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_output(gate1, output).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::ZeroArityGate(gate2)));
}

#[test]
fn build_too_little_connections() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::new(2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::TooLittleConnections { gate, arity: 2 }));
}

#[test]
fn build_cycle_two_gates() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(2));
    let gate2 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate2, gate1).unwrap();
    builder.connect_gate_to_output(gate2, output).unwrap();

    let result = builder.build();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn build_cycle_three_gates() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(2));
    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_gate_to_gate(gate3, gate1).unwrap();
    builder.connect_gate_to_output(gate2, output).unwrap();

    let result = builder.build();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn build_cycle_in_disconnected_subgraph() {
    let mut builder = Builder::new();

    let input1 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let output1 = builder.add_output();
    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();

    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(1));
    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_gate_to_gate(gate3, gate2).unwrap();

    let result = builder.build();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn build_unreachable_gate_simple() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(2));
    let gate3 = builder.add_gate(TestGate::new(1));
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();

    builder.connect_gate_to_gate(gate3, gate2).unwrap();
    builder.connect_gate_to_gate(gate3, gate2).unwrap();

    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_gate_to_output(gate2, output2).unwrap();

    let result = builder.build();

    assert!(matches!(
        result,
        Err(Error::CycleDetected(_)) | Err(Error::UnreachableGate(_))
    ));
}

#[test]
fn build_dead_end_gate() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_gate_to_output(gate1, output).unwrap();

    builder.connect_input_to_gate(input2, gate2).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::DeadEndGate(gate2)));
}

#[test]
fn build_complex_valid_builder() {
    let mut builder = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();

    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(2));

    let output1 = builder.add_output();
    let output2 = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate3).unwrap();
    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();
    builder.connect_gate_to_output(gate3, output2).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_gate_with_multiple_forward_edges() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(1));
    let gate3 = builder.add_gate(TestGate::new(1));
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate3).unwrap();
    builder.connect_gate_to_output(gate2, output1).unwrap();
    builder.connect_gate_to_output(gate3, output2).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_dag_no_cycle() {
    let mut builder = Builder::new();

    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(2));
    let gate3 = builder.add_gate(TestGate::new(1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate3).unwrap();
    builder.connect_gate_to_gate(gate3, gate2).unwrap();
    builder.connect_gate_to_output(gate2, output).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_multiple_inputs_to_same_gate() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::new(2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input1, gate).unwrap();
    builder.connect_input_to_gate(input2, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_mixed_input_and_gate_connections() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let gate2 = builder.add_gate(TestGate::new(2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_input_to_gate(input, gate2).unwrap();
    builder.connect_gate_to_output(gate2, output).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn build_large_valid_builder() {
    let mut builder = Builder::new();

    let inputs: Vec<_> = (0..10).map(|_| builder.add_input()).collect();
    let gates: Vec<_> = (0..10)
        .map(|_| builder.add_gate(TestGate::new(1)))
        .collect();
    let outputs: Vec<_> = (0..10).map(|_| builder.add_output()).collect();

    for (input, gate) in inputs.iter().zip(gates.iter()) {
        builder.connect_input_to_gate(*input, *gate).unwrap();
    }

    for (gate, output) in gates.iter().zip(outputs.iter()) {
        builder.connect_gate_to_output(*gate, *output).unwrap();
    }

    assert!(builder.build().is_ok());
}

#[test]
fn build_cycle_after_valid_path() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(2));
    let gate2 = builder.add_gate(TestGate::new(2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate2, gate1).unwrap(); // Creates cycle
    builder.connect_gate_to_output(gate2, output).unwrap();

    let result = builder.build();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn build_empty_builder_passes() {
    let builder: Builder<TestGate> = Builder::new();

    assert!(builder.build().is_ok());
}

#[test]
fn build_partial_connectivity() {
    let mut builder = Builder::new();

    let input1 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new(1));
    let output1 = builder.add_output();
    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();

    let input2 = builder.add_input();
    let gate2 = builder.add_gate(TestGate::new(1));
    builder.connect_input_to_gate(input2, gate2).unwrap();

    let result = builder.build();
    assert_eq!(result, Err(Error::DeadEndGate(gate2)));
}
