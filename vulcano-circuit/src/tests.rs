use super::*;
use std::num::NonZeroUsize;

#[derive(Debug, Clone)]
struct TestGate {
    arity: NonZeroUsize,
}

impl TestGate {
    fn new(arity: usize) -> Self {
        Self {
            arity: NonZeroUsize::new(arity).expect("arity must be > 0"),
        }
    }
}

impl Gate for TestGate {
    fn arity(&self) -> NonZeroUsize {
        self.arity
    }
}

#[test]
fn new_circuit() {
    let circuit: Circuit<TestGate> = Circuit::new();
    assert_eq!(circuit.gate_count(), 0);
    assert_eq!(circuit.input_count(), 0);
    assert_eq!(circuit.output_count(), 0);
}

#[test]
fn with_capacity() {
    let circuit: Circuit<TestGate> = Circuit::with_capacity(100);
    assert_eq!(circuit.gate_count(), 0);
    assert_eq!(circuit.input_count(), 0);
    assert_eq!(circuit.output_count(), 0);
}

#[test]
fn add_gate() {
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(2));
    let gate2 = circuit.add_gate(TestGate::new(3));

    assert_eq!(circuit.gate_count(), 2);
    assert_eq!(gate1, GateHandle(0));
    assert_eq!(gate2, GateHandle(1));
}

#[test]
fn add_input() {
    let mut circuit: Circuit<TestGate> = Circuit::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();

    assert_eq!(circuit.input_count(), 2);
    assert_eq!(input1, InputHandle(0));
    assert_eq!(input2, InputHandle(1));
}

#[test]
fn add_output() {
    let mut circuit: Circuit<TestGate> = Circuit::new();
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    assert_eq!(circuit.output_count(), 2);
    assert_eq!(output1, OutputHandle(0));
    assert_eq!(output2, OutputHandle(1));
}

#[test]
fn connect_input_to_gate() {
    let mut circuit = Circuit::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_input_to_gate(input, gate);
    assert!(result.is_ok());
}

#[test]
fn connect_input_to_nonexistent_gate() {
    let mut circuit: Circuit<TestGate> = Circuit::new();
    let input = circuit.add_input();
    let nonexistent_gate = GateHandle(99);

    let result = circuit.connect_input_to_gate(input, nonexistent_gate);
    assert_eq!(result, Err(CircuitError::NonExistentGate(nonexistent_gate)));
}

#[test]
fn connect_nonexistent_input_to_gate() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(2));
    let nonexistent_input = InputHandle(99);

    let result = circuit.connect_input_to_gate(nonexistent_input, gate);
    assert_eq!(
        result,
        Err(CircuitError::NonExistentInput(nonexistent_input))
    );
}

#[test]
fn connect_too_many_inputs_to_gate() {
    let mut circuit = Circuit::new();
    let input1 = circuit.add_input();
    let input2 = circuit.add_input();
    let input3 = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(2));

    assert!(circuit.connect_input_to_gate(input1, gate).is_ok());
    assert!(circuit.connect_input_to_gate(input2, gate).is_ok());

    let result = circuit.connect_input_to_gate(input3, gate);
    assert_eq!(
        result,
        Err(CircuitError::TooManyConnections { gate, arity: 2 })
    );
}

#[test]
fn connect_gate_to_gate() {
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_gate_to_gate(gate1, gate2);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_gate() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let nonexistent_gate = GateHandle(99);

    let result1 = circuit.connect_gate_to_gate(nonexistent_gate, gate);
    assert_eq!(
        result1,
        Err(CircuitError::NonExistentGate(nonexistent_gate))
    );

    let result2 = circuit.connect_gate_to_gate(gate, nonexistent_gate);
    assert_eq!(
        result2,
        Err(CircuitError::NonExistentGate(nonexistent_gate))
    );
}

#[test]
fn connect_gate_to_itself() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(2));

    let result = circuit.connect_gate_to_gate(gate, gate);
    assert_eq!(result, Err(CircuitError::SelfConnection(gate)));
}

#[test]
fn connect_too_many_gates_to_gate() {
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));
    let target_gate = circuit.add_gate(TestGate::new(2));

    assert!(circuit.connect_gate_to_gate(gate1, target_gate).is_ok());
    assert!(circuit.connect_gate_to_gate(gate2, target_gate).is_ok());

    let result = circuit.connect_gate_to_gate(gate3, target_gate);
    assert_eq!(
        result,
        Err(CircuitError::TooManyConnections {
            gate: target_gate,
            arity: 2
        })
    );
}

#[test]
fn connect_gate_to_output() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    let result = circuit.connect_gate_to_output(gate, output);
    assert!(result.is_ok());
}

#[test]
fn connect_gate_to_nonexistent_output() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let nonexistent_output = OutputHandle(99);

    let result = circuit.connect_gate_to_output(gate, nonexistent_output);
    assert_eq!(
        result,
        Err(CircuitError::NonExistentOutput(nonexistent_output))
    );
}

#[test]
fn connect_nonexistent_gate_to_output() {
    let mut circuit: Circuit<TestGate> = Circuit::new();
    let output = circuit.add_output();
    let nonexistent_gate = GateHandle(99);

    let result = circuit.connect_gate_to_output(nonexistent_gate, output);
    assert_eq!(result, Err(CircuitError::NonExistentGate(nonexistent_gate)));
}

#[test]
fn output_already_connected() {
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    assert!(circuit.connect_gate_to_output(gate1, output).is_ok());

    let result = circuit.connect_gate_to_output(gate2, output);
    assert_eq!(
        result,
        Err(CircuitError::OutputAlreadyConnectedToGate(output))
    );
}

#[test]
fn gate_cannot_connect_to_multiple_outputs() {
    let mut circuit = Circuit::new();
    let gate = circuit.add_gate(TestGate::new(1));
    let output1 = circuit.add_output();
    let output2 = circuit.add_output();

    assert!(circuit.connect_gate_to_output(gate, output1).is_ok());

    let result = circuit.connect_gate_to_output(gate, output2);
    assert_eq!(
        result,
        Err(CircuitError::GateAlreadyConnectedToOutput(gate))
    );
}

#[test]
fn mixed_connections() {
    let mut circuit = Circuit::new();
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
    let circuit: Circuit<TestGate> = Circuit::default();
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
        format!("{}", CircuitError::NonExistentGate(gate)),
        "Gate GateHandle(5) does not exist"
    );

    assert_eq!(
        format!("{}", CircuitError::NonExistentInput(input)),
        "Input InputHandle(3) does not exist"
    );

    assert_eq!(
        format!("{}", CircuitError::NonExistentOutput(output)),
        "Output OutputHandle(7) does not exist"
    );

    assert_eq!(
        format!("{}", CircuitError::TooManyConnections { gate, arity: 2 }),
        "Gate GateHandle(5) already has 2 connections (max)"
    );

    assert_eq!(
        format!("{}", CircuitError::SelfConnection(gate)),
        "Gate GateHandle(5) cannot connect to itself"
    );

    assert_eq!(
        format!("{}", CircuitError::OutputAlreadyConnectedToGate(output)),
        "Output OutputHandle(7) is already connected"
    );

    assert_eq!(
        format!("{}", CircuitError::GateAlreadyConnectedToOutput(gate)),
        "Gate GateHandle(5) is already connected to an output"
    );
}

#[test]
fn gate_with_arity_one() {
    let mut circuit = Circuit::new();
    let input = circuit.add_input();
    let gate = circuit.add_gate(TestGate::new(1));

    assert!(circuit.connect_input_to_gate(input, gate).is_ok());

    let input2 = circuit.add_input();
    let result = circuit.connect_input_to_gate(input2, gate);
    assert_eq!(
        result,
        Err(CircuitError::TooManyConnections { gate, arity: 1 })
    );
}

#[test]
fn large_circuit() {
    let mut circuit = Circuit::new();

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
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let gate3 = circuit.add_gate(TestGate::new(1));

    assert!(circuit.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(circuit.connect_gate_to_gate(gate1, gate3).is_ok());
}

#[test]
fn gate_can_connect_to_gates_and_output() {
    let mut circuit = Circuit::new();
    let gate1 = circuit.add_gate(TestGate::new(1));
    let gate2 = circuit.add_gate(TestGate::new(1));
    let output = circuit.add_output();

    assert!(circuit.connect_gate_to_gate(gate1, gate2).is_ok());
    assert!(circuit.connect_gate_to_output(gate1, output).is_ok());
}
