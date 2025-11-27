//! Last use analysis for circuits.
//!
//! This module tracks which gate is the last consumer of each value (input or operation output).
//! This information is used by wire allocation to enable aggressive wire reuse: when a gate
//! is the last consumer of a value, it can reuse that value's wire for its own output.

use std::collections::HashMap;

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer, analyses::topological::TopologicalOrder},
        circuit::Circuit,
    },
    handles::{GateId, InputId, Value},
};

/// Analysis that identifies the last use of each value in a circuit.
pub struct LastUseAnalysis;

/// Information about last uses in a circuit.
#[derive(Debug, Clone)]
pub struct LastUseInfo {
    /// For each operation, maps it to the last gate that uses its output (if any).
    pub operation_last_use: HashMap<GateId, GateId>,
    /// For each input, maps it to the last gate that uses it (if any).
    pub input_last_use: HashMap<InputId, GateId>,
}

impl LastUseInfo {
    /// Check if a gate is the last user of a given operation's output.
    pub fn is_last_use_of_operation(&self, consumer: &GateId, producer: &GateId) -> bool {
        self.operation_last_use.get(producer) == Some(consumer)
    }

    /// Check if a gate is the last user of a given input.
    pub fn is_last_use_of_input(&self, consumer: &GateId, input: &InputId) -> bool {
        self.input_last_use.get(input) == Some(consumer)
    }

    /// Get the last gate that uses an operation's output.
    pub fn get_operation_last_use(&self, op: &GateId) -> Option<GateId> {
        self.operation_last_use.get(op).copied()
    }

    /// Get the last gate that uses an input.
    pub fn get_input_last_use(&self, input: &InputId) -> Option<GateId> {
        self.input_last_use.get(input).copied()
    }
}

impl Analysis for LastUseAnalysis {
    type Output = LastUseInfo;

    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let topo_order = analyzer.get::<TopologicalOrder>(circuit)?;

        let mut operation_last_use: HashMap<GateId, GateId> = HashMap::new();
        let mut input_last_use: HashMap<InputId, GateId> = HashMap::new();

        // Scan through gates in topological order.
        // For each value used, update its last use to the current gate.
        for &consumer_op in topo_order.iter() {
            let (_, sources) = &circuit.gate_entries[consumer_op.id()];

            for source in sources {
                match source {
                    Value::Input(input) => {
                        // Update last use of this input.
                        input_last_use.insert(*input, consumer_op);
                    }
                    Value::Gate(producer_op) => {
                        // Update last use of this operation's output.
                        operation_last_use.insert(*producer_op, consumer_op);
                    }
                }
            }
        }

        Ok(LastUseInfo {
            operation_last_use,
            input_last_use,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::last_use::LastUseAnalysis},
            builder::Builder,
        },
        handles::GateId,
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
    fn simple_linear_circuit() {
        // Circuit: input -> negate -> output
        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let last_use = analyzer.get::<LastUseAnalysis>(&circuit).unwrap();

        // InputId's last use is by the negate gate.
        assert_eq!(
            last_use.get_input_last_use(&input),
            Some(GateId::new(gate.id()))
        );
        assert!(last_use.is_last_use_of_input(&GateId::new(gate.id()), &input));
    }

    #[test]
    fn diamond_pattern() {
        // Circuit: input -> negate1 -> \
        //               -> negate2 -> / -> addition -> output

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
        let last_use = analyzer.get::<LastUseAnalysis>(&circuit).unwrap();

        // InputId is used by both negate1 and negate2, last use depends on topological order.
        let input_last_user = last_use.get_input_last_use(&input).unwrap();
        assert!(
            input_last_user == GateId::new(negate1.id())
                || input_last_user == GateId::new(negate2.id())
        );

        // Both negate gates are last used by addition.
        assert_eq!(
            last_use.get_operation_last_use(&GateId::new(negate1.id())),
            Some(GateId::new(addition.id()))
        );
        assert_eq!(
            last_use.get_operation_last_use(&GateId::new(negate2.id())),
            Some(GateId::new(addition.id()))
        );
    }

    #[test]
    fn fan_out() {
        // Circuit: input -> negate1 -> negate2
        //                          \-> addition -> output
        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition).unwrap();
        builder.connect_input_to_gate(input2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let last_use = analyzer.get::<LastUseAnalysis>(&circuit).unwrap();

        // negate1's output is used by both negate2 and addition.
        // Last use depends on topological order.
        let negate1_last_user = last_use
            .get_operation_last_use(&GateId::new(negate1.id()))
            .unwrap();
        assert!(
            negate1_last_user == GateId::new(negate2.id())
                || negate1_last_user == GateId::new(addition.id())
        );
    }
}
