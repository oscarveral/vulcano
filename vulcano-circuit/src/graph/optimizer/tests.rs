use crate::{
    gate::Gate,
    graph::{analyzer::Analyzer, builder::Builder, circuit::Circuit, optimizer::Optimizer},
};
use std::any::TypeId;

#[allow(dead_code)]
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

// Create a simple no-op pass for testing.
fn noop_pass<T: Gate>(
    circuit: Circuit<T>,
    _analyzer: &mut Analyzer<T>,
) -> crate::error::Result<(Circuit<T>, Vec<TypeId>)> {
    Ok((circuit, vec![]))
}

#[test]
fn creation() {
    let optimizer: Optimizer<TestGate> = Optimizer::new();
    assert_eq!(optimizer.passes.len(), 0);
}

#[test]
fn default() {
    let optimizer: Optimizer<TestGate> = Optimizer::default();
    assert_eq!(optimizer.passes.len(), 0);
}

#[test]
fn add_single_pass() {
    let mut optimizer: Optimizer<TestGate> = Optimizer::new();
    optimizer.add_pass(noop_pass);

    assert_eq!(optimizer.passes.len(), 1);
}

#[test]
fn add_multiple_passes() {
    let mut optimizer: Optimizer<TestGate> = Optimizer::new();

    optimizer.add_pass(noop_pass);
    optimizer.add_pass(noop_pass);

    assert_eq!(optimizer.passes.len(), 2);
}

#[test]
fn optimize_no_passes() {
    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.finalize().unwrap();
    let gate_count = circuit.gate_entries.len();

    let mut optimizer: Optimizer<TestGate> = Optimizer::new();
    let optimized = optimizer.optimize(circuit).unwrap();

    assert_eq!(optimized.gate_entries.len(), gate_count);
}

#[test]
fn custom_pass() {
    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.finalize().unwrap();
    let gate_count = circuit.gate_entries.len();

    let mut optimizer: Optimizer<TestGate> = Optimizer::new();
    optimizer.add_pass(noop_pass);

    let optimized = optimizer.optimize(circuit).unwrap();

    assert_eq!(optimized.gate_entries.len(), gate_count);
}

#[test]
fn pass_with_preserved_analyses() {
    fn pass_with_preserved<T: Gate>(
        circuit: Circuit<T>,
        _analyzer: &mut Analyzer<T>,
    ) -> crate::error::Result<(Circuit<T>, Vec<TypeId>)> {
        // Return some TypeIds as preserved.
        let preserved = vec![TypeId::of::<u32>(), TypeId::of::<String>()];
        Ok((circuit, preserved))
    }

    let mut builder: Builder<TestGate> = Builder::new();
    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    builder.connect_input_to_gate(input, gate).unwrap();
    builder.connect_gate_to_output(gate, output).unwrap();

    let circuit = builder.finalize().unwrap();

    let mut optimizer: Optimizer<TestGate> = Optimizer::new();
    optimizer.add_pass(pass_with_preserved);

    let optimized = optimizer.optimize(circuit).unwrap();

    assert_eq!(optimized.gate_entries.len(), 1);
}

#[test]
fn preserves_circuit_structure() {
    let mut builder: Builder<TestGate> = Builder::new();
    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Negate);
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    builder.connect_input_to_gate(input1, gate1).unwrap();
    builder.connect_input_to_gate(input2, gate2).unwrap();
    builder.connect_gate_to_output(gate1, output1).unwrap();
    builder.connect_gate_to_output(gate2, output2).unwrap();

    let circuit = builder.finalize().unwrap();
    let input_count = circuit.input_count;
    let output_count = circuit.connected_outputs.len();
    let gate_count = circuit.gate_entries.len();

    let mut optimizer: Optimizer<TestGate> = Optimizer::new();
    optimizer.add_pass(noop_pass);

    let optimized = optimizer.optimize(circuit).unwrap();

    assert_eq!(optimized.input_count, input_count);
    assert_eq!(optimized.connected_outputs.len(), output_count);
    assert_eq!(optimized.gate_entries.len(), gate_count);
}
