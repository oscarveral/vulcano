use crate::{Builder, Error, Gate, GateHandle, InputHandle, OutputHandle};

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
fn new_circuit() {
    let circuit: Builder<TestGate> = Builder::new();
    assert_eq!(circuit.gate_count(), 0);
    assert_eq!(circuit.input_count(), 0);
    assert_eq!(circuit.output_count(), 0);
}

#[test]
fn with_capacity() {
    let circuit: Builder<TestGate> = Builder::with_capacity(100);
    assert_eq!(circuit.gate_count(), 0);
    assert_eq!(circuit.input_count(), 0);
    assert_eq!(circuit.output_count(), 0);
}

#[test]
fn add_gate() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(3));

    assert_eq!(circuit.gate_count(), 2);
    assert_eq!(gate1, GateHandle(0));
    assert_eq!(gate2, GateHandle(1));
}

#[test]
fn add_input() {
    let mut circuit: Builder<TestGate> = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();

    assert_eq!(circuit.input_count(), 2);
    assert_eq!(input1, InputHandle(0));
    assert_eq!(input2, InputHandle(1));
}

#[test]
fn add_output() {
    let mut circuit: Builder<TestGate> = Builder::new();
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    assert_eq!(circuit.output_count(), 2);
    assert_eq!(output1, OutputHandle(0));
    assert_eq!(output2, OutputHandle(1));
}

#[test]
fn connect_input_to_gate() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_input_to_gate(input, gate);
    assert!(result.is_ok());
}

#[test]
fn connect_input_to_nonexistent_gate() {
    let mut circuit: Builder<TestGate> = Builder::new();
    let input = circuit.add_input();
    let nonexistent_gate = GateHandle(99);

    let result = circuit.connect_input_to_gate(input, nonexistent_gate);
    assert_eq!(result, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn connect_nonexistent_input_to_gate() {
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(2));
    let nonexistent_input = InputHandle(99);

    let result = circuit.connect_input_to_gate(nonexistent_input, gate);
    assert_eq!(result, Err(Error::NonExistentInput(nonexistent_input)));
}

#[test]
fn connect_too_many_inputs_to_gate() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let input3 = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));

    assert!(circuit.connect_input_to_gate(input1, gate).is_ok());
    assert!(circuit.connect_input_to_gate(input2, gate).is_ok());

    let result = circuit.connect_input_to_gate(input3, gate);
    assert_eq!(result, Err(Error::TooManyConnections { gate, arity: 2 }));
}

#[test]
fn connect_gate_to_gate() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_gate_to_gate(gate1, gate2);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_gate() {
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let nonexistent_gate = GateHandle(99);

    let result1 = circuit.connect_gate_to_gate(nonexistent_gate, gate);
    assert_eq!(result1, Err(Error::NonExistentGate(nonexistent_gate)));

    let result2 = circuit.connect_gate_to_gate(gate, nonexistent_gate);
    assert_eq!(result2, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn connect_gate_to_itself() {
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_gate_to_gate(gate, gate);
    assert_eq!(result, Err(Error::SelfConnection(gate)));
}

#[test]
fn connect_too_many_gates_to_gate() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let target_gate = circuit.add_gate(TestGate::new(2));

    assert!(circuit.connect_gate_to_gate(gate1, target_gate).is_ok());
    assert!(circuit.connect_gate_to_gate(gate2, target_gate).is_ok());

    let result = circuit.connect_gate_to_gate(gate3, target_gate);
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
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    let result = circuit.connect_gate_to_output(gate, output);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_output() {
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let nonexistent_output = OutputHandle(99);

    let result = circuit.connect_gate_to_output(gate, nonexistent_output);
    assert_eq!(result, Err(Error::NonExistentOutput(nonexistent_output)));
}

#[test]
fn connect_nonexistent_gate_to_output() {
    let mut circuit: Builder<TestGate> = Builder::new();
    let output = circuit.add_output();
    let nonexistent_gate = GateHandle(99);

    let result = circuit.connect_gate_to_output(nonexistent_gate, output);
    assert_eq!(result, Err(Error::NonExistentGate(nonexistent_gate)));
}

#[test]
fn output_already_connected() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    assert!(circuit.connect_gate_to_output(gate1, output).is_ok());

    let result = circuit.connect_gate_to_output(gate2, output);
    assert_eq!(result, Err(Error::OutputAlreadyConnectedToGate(output)));
}

#[test]
fn gate_cannot_connect_to_multiple_outputs() {
    let mut circuit = Builder::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    assert!(circuit.connect_gate_to_output(gate, output1).is_ok());

    let result = circuit.connect_gate_to_output(gate, output2);
    assert_eq!(result, Err(Error::GateAlreadyConnectedToOutput(gate)));
}

#[test]
fn mixed_connections() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(2));
    let gate3 = circuit.add_gate(TestGate::new(2));
    let output = circuit.add_output();

    assert!(circuit.connect_input_to_gate(input1, gate1).is_ok());
    assert!(circuit.connect_input_to_gate(input2, gate1).is_ok());
    assert!(circuit.connect_gate_to_gate(gate1, gate3).is_ok());
    assert!(circuit.connect_gate_to_gate(gate2, gate3).is_ok());
    assert!(circuit.connect_gate_to_output(gate3, output).is_ok());
}

#[test]
fn handles_equality() {
    let gate1 = GateHandle(0);
    let gate2 = GateHandle(0);
    let gate3 = GateHandle(1);

    assert_eq!(gate1, gate2);
    assert_ne!(gate1, gate3);

    let input1 = InputHandle(0);
    let input2 = InputHandle(0);
    let input3 = InputHandle(1);

    assert_eq!(input1, input2);
    assert_ne!(input1, input3);

    let output1 = OutputHandle(0);
    let output2 = OutputHandle(0);
    let output3 = OutputHandle(1);

    assert_eq!(output1, output2);
    assert_ne!(output1, output3);
}

#[test]
fn default_circuit() {
    let circuit: Builder<TestGate> = Builder::default();
    assert_eq!(circuit.gate_count(), 0);
    assert_eq!(circuit.input_count(), 0);
    assert_eq!(circuit.output_count(), 0);
}

#[test]
fn circuit_error_display() {
    let gate = GateHandle(5);
    let input = InputHandle(3);
    let output = OutputHandle(7);

    assert_eq!(
        format!("{}", Error::NonExistentGate(gate)),
        "Gate GateHandle(5) does not exist"
    );

    assert_eq!(
        format!("{}", Error::NonExistentInput(input)),
        "Input InputHandle(3) does not exist"
    );

    assert_eq!(
        format!("{}", Error::NonExistentOutput(output)),
        "Output OutputHandle(7) does not exist"
    );

    assert_eq!(
        format!("{}", Error::TooManyConnections { gate, arity: 2 }),
        "Gate GateHandle(5) already has 2 connections (max)"
    );

    assert_eq!(
        format!("{}", Error::SelfConnection(gate)),
        "Gate GateHandle(5) cannot connect to itself"
    );

    assert_eq!(
        format!("{}", Error::OutputAlreadyConnectedToGate(output)),
        "Output OutputHandle(7) is already connected"
    );

    assert_eq!(
        format!("{}", Error::GateAlreadyConnectedToOutput(gate)),
        "Gate GateHandle(5) is already connected to an output"
    );
}

#[test]
fn gate_with_arity_one() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(1));

    assert!(circuit.connect_input_to_gate(input, gate).is_ok());

    let input2 = circuit.add_input();
    let result = circuit.connect_input_to_gate(input2, gate);
    assert_eq!(result, Err(Error::TooManyConnections { gate, arity: 1 }));
}

#[test]
fn large_circuit() {
    let mut circuit = Builder::new();

    let inputs: Vec<_> = (0..100).map(|_| circuit.add_input()).collect();
    let gates: Vec<_> = (0..100)
        .map(|_| circuit.add_gate(TestGate::new(1)))
        .collect();
    let outputs: Vec<_> = (0..100).map(|_| circuit.add_output()).collect();

    for (input, gate) in inputs.iter().zip(gates.iter()) {
        assert!(circuit.connect_input_to_gate(*input, *gate).is_ok());
    }

    for (gate, output) in gates.iter().zip(outputs.iter()) {
        assert!(circuit.connect_gate_to_output(*gate, *output).is_ok());
    }

    assert_eq!(circuit.gate_count(), 100);
    assert_eq!(circuit.input_count(), 100);
    assert_eq!(circuit.output_count(), 100);
}

#[test]
fn gate_can_have_multiple_forward_connections() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));

    assert!(circuit.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(circuit.connect_gate_to_gate(gate1, gate3).is_ok());
}

#[test]
fn gate_can_connect_to_gates_and_output() {
    let mut circuit = Builder::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    assert!(circuit.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(circuit.connect_gate_to_output(gate1, output).is_ok());
}

#[test]
fn validate_simple_valid_circuit() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate).unwrap();
    circuit.connect_gate_to_output(gate, output).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_unused_input() {
    let mut circuit = Builder::new();
    let _input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    let input2 = circuit.add_input();
    circuit.connect_input_to_gate(input2, gate).unwrap();
    circuit.connect_gate_to_output(gate, output).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::UnusedInput(InputHandle(0))));
}

#[test]
fn validate_unused_output() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let _output = circuit.add_output();

    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_input_to_gate(input2, gate2).unwrap();

    let output2 = circuit.add_output();
    circuit.connect_gate_to_output(gate1, output2).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::UnusedOutput(OutputHandle(0))));
}

#[test]
fn validate_zero_arity_gate() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(0));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_output(gate1, output).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::ZeroArityGate(gate2)));
}

#[test]
fn validate_too_little_connections() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate).unwrap();
    circuit.connect_gate_to_output(gate, output).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::TooLittleConnections { gate, arity: 2 }));
}

#[test]
fn validate_cycle_two_gates() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_gate_to_gate(gate2, gate1).unwrap();
    circuit.connect_gate_to_output(gate2, output).unwrap();

    let result = circuit.validate();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn validate_cycle_three_gates() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_gate_to_gate(gate2, gate3).unwrap();
    circuit.connect_gate_to_gate(gate3, gate1).unwrap();
    circuit.connect_gate_to_output(gate2, output).unwrap();

    let result = circuit.validate();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn validate_cycle_in_disconnected_subgraph() {
    let mut circuit = Builder::new();

    let input1 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_gate_to_output(gate1, output1).unwrap();

    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));
    circuit.connect_gate_to_gate(gate2, gate3).unwrap();
    circuit.connect_gate_to_gate(gate3, gate2).unwrap();

    let result = circuit.validate();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn validate_unreachable_gate_simple() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(2));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_output(gate1, output1).unwrap();

    circuit.connect_gate_to_gate(gate3, gate2).unwrap();
    circuit.connect_gate_to_gate(gate3, gate2).unwrap();

    circuit.connect_gate_to_gate(gate2, gate3).unwrap();
    circuit.connect_gate_to_output(gate2, output2).unwrap();

    let result = circuit.validate();

    assert!(matches!(
        result,
        Err(Error::CycleDetected(_)) | Err(Error::UnreachableGate(_))
    ));
}

#[test]
fn validate_dead_end_gate() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_gate_to_output(gate1, output).unwrap();

    circuit.connect_input_to_gate(input2, gate2).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::DeadEndGate(gate2)));
}

#[test]
fn validate_complex_valid_circuit() {
    let mut circuit = Builder::new();

    let input1 = circuit.add_input();
    let input2 = circuit.add_input();

    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(2));

    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_input_to_gate(input2, gate2).unwrap();
    circuit.connect_gate_to_gate(gate1, gate3).unwrap();
    circuit.connect_gate_to_gate(gate2, gate3).unwrap();
    circuit.connect_gate_to_output(gate1, output1).unwrap();
    circuit.connect_gate_to_output(gate3, output2).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_gate_with_multiple_forward_edges() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_gate_to_gate(gate1, gate3).unwrap();
    circuit.connect_gate_to_output(gate2, output1).unwrap();
    circuit.connect_gate_to_output(gate3, output2).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_dag_no_cycle() {
    let mut circuit = Builder::new();

    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(2));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_gate_to_gate(gate1, gate3).unwrap();
    circuit.connect_gate_to_gate(gate3, gate2).unwrap();
    circuit.connect_gate_to_output(gate2, output).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_multiple_inputs_to_same_gate() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input1, gate).unwrap();
    circuit.connect_input_to_gate(input2, gate).unwrap();
    circuit.connect_gate_to_output(gate, output).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_mixed_input_and_gate_connections() {
    let mut circuit = Builder::new();
    let input = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(2));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input, gate1).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_input_to_gate(input, gate2).unwrap();
    circuit.connect_gate_to_output(gate2, output).unwrap();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_large_valid_circuit() {
    let mut circuit = Builder::new();

    let inputs: Vec<_> = (0..10).map(|_| circuit.add_input()).collect();
    let gates: Vec<_> = (0..10)
        .map(|_| circuit.add_gate(TestGate::new(1)))
        .collect();
    let outputs: Vec<_> = (0..10).map(|_| circuit.add_output()).collect();

    for (input, gate) in inputs.iter().zip(gates.iter()) {
        circuit.connect_input_to_gate(*input, *gate).unwrap();
    }

    for (gate, output) in gates.iter().zip(outputs.iter()) {
        circuit.connect_gate_to_output(*gate, *output).unwrap();
    }

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_cycle_after_valid_path() {
    let mut circuit = Builder::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(2));
    let output = circuit.add_output();

    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_input_to_gate(input2, gate2).unwrap();
    circuit.connect_gate_to_gate(gate1, gate2).unwrap();
    circuit.connect_gate_to_gate(gate2, gate1).unwrap(); // Creates cycle
    circuit.connect_gate_to_output(gate2, output).unwrap();

    let result = circuit.validate();
    assert!(matches!(result, Err(Error::CycleDetected(_))));
}

#[test]
fn validate_empty_circuit_passes() {
    let circuit: Builder<TestGate> = Builder::new();

    assert!(circuit.validate().is_ok());
}

#[test]
fn validate_partial_connectivity() {
    let mut circuit = Builder::new();

    let input1 = circuit.add_input();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    circuit.connect_input_to_gate(input1, gate1).unwrap();
    circuit.connect_gate_to_output(gate1, output1).unwrap();

    let input2 = circuit.add_input();
    let gate2 = circuit.add_gate(TestGate::new(1));
    circuit.connect_input_to_gate(input2, gate2).unwrap();

    let result = circuit.validate();
    assert_eq!(result, Err(Error::DeadEndGate(gate2)));
}
