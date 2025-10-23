use crate::{builder::Builder, gate::Gate};

#[derive(Debug, Clone)]
struct TestGate {
    name: &'static str,
    arity: usize,
}

impl TestGate {
    fn new(name: &'static str, arity: usize) -> Self {
        Self { name, arity }
    }
}

impl Gate for TestGate {
    fn arity(&self) -> usize {
        self.arity
    }

    fn name(&self) -> &str {
        self.name
    }
}

#[test]
fn ssa_simple_circuit() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::new("NOT", 1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("; Circuit with 1 inputs, 1 gates, 1 outputs"));
    assert!(ssa.contains("%w0 = input @i0"));
    assert!(ssa.contains("NOT(%w0)"));
    assert!(ssa.contains("output @o0 = %w1"));
}

#[test]
fn ssa_multiple_inputs() {
    let mut builder = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::new("AND", 2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input1, gate).unwrap();
    builder.connect_input_to_gate(input2, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("%w0 = input @i0"));
    assert!(ssa.contains("%w1 = input @i1"));
    assert!(ssa.contains("(%w0, %w1)"));
    assert!(ssa.contains("output @o0 = %w2"));
}

#[test]
fn ssa_chain_circuit() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new("NOT", 1));
    let gate2 = builder.add_gate(TestGate::new("NOT", 1));
    let gate3 = builder.add_gate(TestGate::new("NOT", 1));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_gate_to_output(gate3, output).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("; Circuit with 1 inputs, 3 gates, 1 outputs"));
    assert!(ssa.contains("%w0 = input @i0"));
    assert!(ssa.contains("%w1"));
    assert!(ssa.contains("%w2"));
    assert!(ssa.contains("%w3"));
    assert!(ssa.contains("output @o0 = %w3"));
}

#[test]
fn ssa_multiple_outputs() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new("BUF", 1));
    let gate2 = builder.add_gate(TestGate::new("NOT", 1));
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();
    builder.connect_gate_to_output(gate2, output2).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("output @o0 = %w1"));
    assert!(ssa.contains("output @o1 = %w2"));
}

#[test]
fn ssa_diamond_dag() {
    let mut builder = Builder::new();
    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::new("BUF", 1));
    let gate2 = builder.add_gate(TestGate::new("NOT", 1));
    let gate3 = builder.add_gate(TestGate::new("NOT", 1));
    let gate4 = builder.add_gate(TestGate::new("AND", 2));
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate1).unwrap();
    builder.connect_gate_to_gate(gate1, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate3).unwrap();
    builder.connect_gate_to_gate(gate2, gate4).unwrap();
    builder.connect_gate_to_gate(gate3, gate4).unwrap();
    builder.connect_gate_to_output(gate4, output).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("; Circuit with 1 inputs, 4 gates, 1 outputs"));
    assert!(ssa.contains("(%w0)"));
    assert!(ssa.contains("output @o0"));
}

#[test]
fn ssa_empty_circuit() {
    let builder: Builder<TestGate> = Builder::new();
    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("; Circuit with 0 inputs, 0 gates, 0 outputs"));
}

#[test]
fn ssa_complex_circuit() {
    let mut builder = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let input3 = builder.add_input();

    let gate1 = builder.add_gate(TestGate::new("AND", 2));
    let gate2 = builder.add_gate(TestGate::new("OR", 2));
    let gate3 = builder.add_gate(TestGate::new("XOR", 3));

    let output = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate2).unwrap();
    builder.connect_input_to_gate(input3, gate2).unwrap();
    builder.connect_gate_to_gate(gate1, gate3).unwrap();
    builder.connect_gate_to_gate(gate2, gate3).unwrap();
    builder.connect_input_to_gate(input1, gate3).unwrap();
    builder.connect_gate_to_output(gate3, output).unwrap();

    let circuit = builder.build().unwrap();
    let ssa = circuit.to_ssa();

    assert!(ssa.contains("; Circuit with 3 inputs, 3 gates, 1 outputs"));
    assert!(ssa.contains("%w0 = input @i0"));
    assert!(ssa.contains("%w1 = input @i1"));
    assert!(ssa.contains("%w2 = input @i2"));
    assert!(ssa.contains("output @o0"));
}
