use crate::{
    error::Error,
    gate::Gate,
    graph::builder::{Builder, Source},
    handles::{Input, Operation, Output},
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
fn creation() {
    let builder: Builder<TestGate> = Builder::new();

    assert_eq!(builder.gate_entries.len(), 0);
    assert_eq!(builder.connected_inputs.len(), 0);
    assert_eq!(builder.connected_outputs.len(), 0);
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 0);
}

#[test]
fn add_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let _ = builder.add_gate(TestGate::Negate);
    let _ = builder.add_gate(TestGate::Addition);

    assert_eq!(builder.gate_entries.len(), 2);
    assert_eq!(builder.connected_inputs.len(), 0);
    assert_eq!(builder.connected_outputs.len(), 0);
    assert_eq!(builder.gate_count(), 2);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 0);

    assert_eq!(builder.gate_entries[0].0.name(), "Negate");
    assert_eq!(builder.gate_entries[1].0.name(), "Addition");
    assert_eq!(builder.gate_entries[0].1.len(), 0);
    assert_eq!(builder.gate_entries[1].1.len(), 0);
}

#[test]
fn add_input() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();

    assert_eq!(builder.gate_entries.len(), 0);
    assert_eq!(builder.connected_inputs.len(), 2);
    assert_eq!(builder.connected_outputs.len(), 0);
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 2);
    assert_eq!(builder.output_count(), 0);

    assert!(!builder.connected_inputs[input1.id()]);
    assert!(!builder.connected_inputs[input2.id()]);
}

#[test]
fn add_output() {
    let mut builder: Builder<TestGate> = Builder::new();

    let output1 = builder.add_output();
    let output2 = builder.add_output();

    assert_eq!(builder.gate_entries.len(), 0);
    assert_eq!(builder.connected_inputs.len(), 0);
    assert_eq!(builder.connected_outputs.len(), 2);
    assert_eq!(builder.gate_count(), 0);
    assert_eq!(builder.input_count(), 0);
    assert_eq!(builder.output_count(), 2);

    assert_eq!(builder.connected_outputs[output1.id()], None);
    assert_eq!(builder.connected_outputs[output2.id()], None);
}

#[test]
fn consistency() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Addition);
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    assert_eq!(builder.gate_count(), 2);
    assert_eq!(builder.input_count(), 2);
    assert_eq!(builder.output_count(), 2);
    assert_eq!(builder.gate_entries.len(), builder.gate_count());
    assert_eq!(builder.connected_inputs.len(), builder.input_count());
    assert_eq!(builder.connected_outputs.len(), builder.output_count());

    assert!(!builder.connected_inputs[input1.id()]);
    assert!(!builder.connected_inputs[input2.id()]);

    assert_eq!(builder.connected_outputs[output1.id()], None);
    assert_eq!(builder.connected_outputs[output2.id()], None);

    assert_eq!(builder.gate_entries[gate1.id()].0.name(), "Negate");
    assert_eq!(builder.gate_entries[gate2.id()].0.name(), "Addition");
    assert_eq!(builder.gate_entries[gate1.id()].1.len(), 0);
    assert_eq!(builder.gate_entries[gate2.id()].1.len(), 0);
}

#[test]
fn non_existent_input() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate = builder.add_gate(TestGate::Negate);
    let input = Input::new(999);

    let result = builder.connect_input_to_gate(input, gate);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NonExistentInput(input));
}

#[test]
fn non_existent_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input = builder.add_input();
    let gate = Operation::new(999);

    let result = builder.connect_input_to_gate(input, gate);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NonExistentGate(gate));
}

#[test]
fn input_arity_over_limit_using_input() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);

    let op = builder.connect_input_to_gate(input1, gate);

    assert!(op.is_ok());

    let result = builder.connect_input_to_gate(input2, gate);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InputArityOverLimit(gate));
}

#[test]
fn connect_input_to_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::Addition);

    let result1 = builder.connect_input_to_gate(input1, gate);

    assert!(result1.is_ok());
    assert!(builder.connected_inputs[input1.id()]);
    assert_eq!(builder.gate_entries[gate.id()].1.len(), 1);
    assert_eq!(builder.gate_entries[gate.id()].1[0], Source::Input(input1));

    let result2 = builder.connect_input_to_gate(input2, gate);

    assert!(result2.is_ok());
    assert!(builder.connected_inputs[input2.id()]);
    assert_eq!(builder.gate_entries[gate.id()].1.len(), 2);
    assert_eq!(builder.gate_entries[gate.id()].1[1], Source::Input(input2));
}

#[test]
fn source_gate_non_existent() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate = builder.add_gate(TestGate::Addition);
    let source_gate = Operation::new(999);

    let result = builder.connect_gate_to_gate(source_gate, gate);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NonExistentGate(source_gate));
}

#[test]
fn destination_gate_non_existent() {
    let mut builder: Builder<TestGate> = Builder::new();

    let source_gate = builder.add_gate(TestGate::Negate);
    let destination_gate = Operation::new(999);

    let result = builder.connect_gate_to_gate(source_gate, destination_gate);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Error::NonExistentGate(destination_gate)
    );
}

#[test]
fn self_connection() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate = builder.add_gate(TestGate::Negate);

    let result = builder.connect_gate_to_gate(gate, gate);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::SelfConnection(gate));
}

#[test]
fn input_arity_over_limit_using_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Negate);

    let op = builder.connect_gate_to_gate(gate1, gate2);

    assert!(op.is_ok());

    let result = builder.connect_gate_to_gate(gate1, gate2);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InputArityOverLimit(gate2));
}

#[test]
fn allow_repeated_connections() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Addition);

    let op1 = builder.connect_gate_to_gate(gate1, gate2);
    assert!(op1.is_ok());

    let op2 = builder.connect_gate_to_gate(gate1, gate2);
    assert!(op2.is_ok());
}

#[test]
fn connect_gate_to_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Addition);

    let result1 = builder.connect_gate_to_gate(gate1, gate2);

    assert!(result1.is_ok());
    assert_eq!(builder.gate_entries[gate2.id()].1.len(), 1);
    assert_eq!(builder.gate_entries[gate2.id()].1[0], Source::Gate(gate1));

    let result2 = builder.connect_gate_to_gate(gate1, gate2);

    assert!(result2.is_ok());
    assert_eq!(builder.gate_entries[gate2.id()].1.len(), 2);
    assert_eq!(builder.gate_entries[gate2.id()].1[1], Source::Gate(gate1));
}

#[test]
fn mixed_source_connections() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input = builder.add_input();
    let gate1 = builder.add_gate(TestGate::Negate);
    let gate2 = builder.add_gate(TestGate::Addition);

    let result1 = builder.connect_input_to_gate(input, gate2);

    assert!(result1.is_ok());
    assert!(builder.connected_inputs[input.id()]);
    assert_eq!(builder.gate_entries[gate2.id()].1.len(), 1);
    assert_eq!(builder.gate_entries[gate2.id()].1[0], Source::Input(input));

    let result2 = builder.connect_gate_to_gate(gate1, gate2);
    assert!(result2.is_ok());
    assert_eq!(builder.gate_entries[gate2.id()].1.len(), 2);
    assert_eq!(builder.gate_entries[gate2.id()].1[1], Source::Gate(gate1));

    let result3 = builder.connect_gate_to_gate(gate1, gate2);
    assert!(result3.is_err());
    assert_eq!(result3.unwrap_err(), Error::InputArityOverLimit(gate2));
}

#[test]
fn output_non_existent_gate() {
    let mut builder: Builder<TestGate> = Builder::new();

    let output = builder.add_output();
    let gate = Operation::new(999);

    let result = builder.connect_gate_to_output(gate, output);

    assert!(result.is_err());

    assert_eq!(result.unwrap_err(), Error::NonExistentGate(gate));
}

#[test]
fn non_existent_output() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate = builder.add_gate(TestGate::Negate);
    let output = Output::new(999);

    let result = builder.connect_gate_to_output(gate, output);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NonExistentOutput(output));
}

#[test]
fn used_output() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    let op = builder.connect_gate_to_output(gate, output);

    assert!(op.is_ok());
    assert_eq!(builder.connected_outputs[output.id()], Some(gate));

    let result = builder.connect_gate_to_output(gate, output);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::UsedOutput(output));
}

#[test]
fn output_arity_over_limit() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate1 = builder.add_gate(TestGate::Negate);
    let output1 = builder.add_output();
    let output2 = builder.add_output();

    let op = builder.connect_gate_to_output(gate1, output1);

    assert!(op.is_ok());
    assert_eq!(builder.connected_outputs[output1.id()], Some(gate1));

    let result = builder.connect_gate_to_output(gate1, output2);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::OutputArityOverLimit(gate1));
}

#[test]
fn connect_gate_to_output() {
    let mut builder: Builder<TestGate> = Builder::new();

    let gate1 = builder.add_gate(TestGate::Negate);
    let output1 = builder.add_output();

    let result1 = builder.connect_gate_to_output(gate1, output1);

    assert!(result1.is_ok());
    assert_eq!(builder.connected_outputs[output1.id()], Some(gate1));
}

#[test]
fn validate_used_inputs() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::Addition);

    let _ = builder.connect_input_to_gate(input1, gate);

    let result = builder.validate();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::UnusedInput(input2));
}

#[test]
fn validate_used_outputs() {
    let mut builder: Builder<TestGate> = Builder::new();

    let _ = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    let result = builder.validate();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::UnusedOutput(output));
}

#[test]
fn validate_empty_circuit() {
    let builder: Builder<TestGate> = Builder::new();

    let result = builder.validate();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::EmptyCircuit);
}

#[test]
fn validate_input_arity_under_limit() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Addition);

    let _ = builder.connect_input_to_gate(input, gate);

    let result = builder.validate();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InputArityUnderLimit(gate));
}

#[test]
fn validate_input_arity_over_limit() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input1 = builder.add_input();
    let input2 = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);

    let _ = builder.connect_input_to_gate(input1, gate);

    // Manually connect another input to exceed arity.
    builder.gate_entries[gate.id()]
        .1
        .push(Source::Input(input2));
    builder.connected_inputs[input2.id()] = true;

    let result = builder.validate();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::InputArityOverLimit(gate));
}

#[test]
fn simple_circuit() {
    let mut builder: Builder<TestGate> = Builder::new();

    let input = builder.add_input();
    let gate = builder.add_gate(TestGate::Negate);
    let output = builder.add_output();

    let r1 = builder.connect_input_to_gate(input, gate);
    assert!(r1.is_ok());
    assert!(builder.connected_inputs[input.id()]);
    assert_eq!(builder.gate_entries[gate.id()].1.len(), 1);
    assert_eq!(builder.gate_entries[gate.id()].1[0], Source::Input(input));

    let r2 = builder.connect_gate_to_output(gate, output);
    assert!(r2.is_ok());
    assert_eq!(builder.connected_outputs[output.id()], Some(gate));

    let v = builder.validate();
    assert!(v.is_ok());
}

#[test]
fn complex_circuit() {
    let mut builder: Builder<TestGate> = Builder::new();

    // input1 -> negate1 -> \
    //                        addition -> output
    // input2 -> negate2 -> /

    let input1 = builder.add_input();
    let input2 = builder.add_input();

    let negate1 = builder.add_gate(TestGate::Negate);
    let negate2 = builder.add_gate(TestGate::Negate);
    let addition = builder.add_gate(TestGate::Addition);

    let output = builder.add_output();

    assert!(builder.connect_input_to_gate(input1, negate1).is_ok());
    assert!(builder.connect_input_to_gate(input2, negate2).is_ok());

    assert!(builder.connect_gate_to_gate(negate1, addition).is_ok());
    assert!(builder.connect_gate_to_gate(negate2, addition).is_ok());

    assert!(builder.connect_gate_to_output(addition, output).is_ok());

    // Verify internal wiring.
    let srcs = &builder.gate_entries[addition.id()].1;
    assert_eq!(srcs.len(), 2);
    assert_eq!(srcs[0], Source::Gate(negate1));
    assert_eq!(srcs[1], Source::Gate(negate2));

    assert_eq!(builder.connected_outputs[output.id()], Some(addition));

    // Validation should succeed for a well-formed circuit.
    assert!(builder.validate().is_ok());
}

#[test]
fn multiple_outputs_from_different_gates() {
    let mut builder: Builder<TestGate> = Builder::new();

    // input ---> negate1 -> output1
    //        \-> negate1 -> negate2 -> output2

    let input = builder.add_input();
    let negate1 = builder.add_gate(TestGate::Negate);
    let negate2 = builder.add_gate(TestGate::Negate);

    let output1 = builder.add_output();
    let output2 = builder.add_output();

    assert!(builder.connect_input_to_gate(input, negate1).is_ok());

    assert!(builder.connect_gate_to_output(negate1, output1).is_ok());
    assert_eq!(builder.connected_outputs[output1.id()], Some(negate1));

    assert!(builder.connect_gate_to_gate(negate1, negate2).is_ok());

    assert!(builder.connect_gate_to_output(negate2, output2).is_ok());
    assert_eq!(builder.connected_outputs[output2.id()], Some(negate2));

    // All gates have required inputs and all outputs are used.
    assert!(builder.validate().is_ok());

    // Check gate_entries sources.
    let s_neg2 = &builder.gate_entries[negate2.id()].1;
    assert_eq!(s_neg2.len(), 1);
    assert_eq!(s_neg2[0], Source::Gate(negate1));
}

#[test]
fn vector_accumulation_circuit() {
    let mut builder: Builder<TestGate> = Builder::new();

    // Complex vector of length 6 accumulation circuit can be built here.

    // input --------------> addition --------------> addition -------> addition ---> output
    //   |-> rotate by 3 -|        |-> rotate by 1 -|                |
    //                                              |-> rotate by 1 -|

    let input = builder.add_input();

    // Rotations need the vector and the rotation index.
    let rotation_idx_1 = builder.add_input();
    let rotation_idx_3 = builder.add_input();

    // Need 3 rotation gates to move all 6 elements.
    let rotate1_1 = builder.add_gate(TestGate::Rotate);
    let rotate2_2 = builder.add_gate(TestGate::Rotate);
    let rotate3 = builder.add_gate(TestGate::Rotate);

    // Need addition gates to accumulate results.
    let addition1 = builder.add_gate(TestGate::Addition);
    let addition2 = builder.add_gate(TestGate::Addition);
    let addition3 = builder.add_gate(TestGate::Addition);

    let output = builder.add_output();

    assert!(builder.connect_input_to_gate(input, rotate3).is_ok());
    assert!(
        builder
            .connect_input_to_gate(rotation_idx_3, rotate3)
            .is_ok()
    );

    assert!(builder.connect_gate_to_gate(rotate3, addition1).is_ok());
    assert!(builder.connect_input_to_gate(input, addition1).is_ok());

    assert!(builder.connect_gate_to_gate(addition1, rotate1_1).is_ok());
    assert!(
        builder
            .connect_input_to_gate(rotation_idx_1, rotate1_1)
            .is_ok()
    );

    assert!(builder.connect_gate_to_gate(rotate1_1, addition2).is_ok());
    assert!(builder.connect_gate_to_gate(addition1, addition2).is_ok());

    assert!(builder.connect_gate_to_gate(rotate1_1, rotate2_2).is_ok());
    assert!(
        builder
            .connect_input_to_gate(rotation_idx_1, rotate2_2)
            .is_ok()
    );

    assert!(builder.connect_gate_to_gate(rotate2_2, addition3).is_ok());
    assert!(builder.connect_gate_to_gate(addition2, addition3).is_ok());

    assert!(builder.connect_gate_to_output(addition3, output).is_ok());

    // All gates have required inputs and all outputs are used.
    assert!(builder.validate().is_ok());

    // Assert construction.
    assert!(builder.finalize().is_ok());
}
