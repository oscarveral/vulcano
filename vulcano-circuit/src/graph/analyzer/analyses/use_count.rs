//! Use count analysis for circuits.
//!
//! This module provides functionality to count how many times each
//! operation's output and each circuit input is used by other gates
//! or circuit outputs.

use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::Circuit,
    },
    handles::{Input, Operation, Source},
};

/// Analysis that computes how many times each value is used.
///
/// This analysis counts:
/// - How many times each operation's output is consumed.
/// - How many times each circuit input is consumed.
pub struct UseCountAnalysis;

/// Information about how many times each value is used.
#[derive(Debug, Clone)]
pub struct UseCountInfo {
    /// Number of times each operation's output is used.
    ///
    /// This includes uses by other gates and uses as circuit outputs.
    pub operation_uses: HashMap<Operation, usize>,

    /// Number of times each circuit input is used.
    ///
    /// Inputs not in this map have a use count of 0 (unused inputs).
    pub input_uses: HashMap<Input, usize>,
}

impl UseCountInfo {
    /// Get the use count for an operation.
    ///
    /// Returns an error if the operation is not found.
    pub fn operation_use_count(&self, op: &Operation) -> Result<usize> {
        self.operation_uses
            .get(op)
            .copied()
            .ok_or(Error::UseCountOperationNotFound(*op))
    }

    /// Get the use count for an input.
    ///
    /// Returns an error if the input is not found.
    pub fn input_use_count(&self, input: &Input) -> Result<usize> {
        self.input_uses
            .get(input)
            .copied()
            .ok_or(Error::UseCountInputNotFound(*input))
    }

    /// Check if an operation's output is used at least once.
    pub fn is_operation_used(&self, op: &Operation) -> Result<bool> {
        Ok(self.operation_use_count(op)? > 0)
    }

    /// Check if an input is used at least once.
    pub fn is_input_used(&self, input: &Input) -> Result<bool> {
        Ok(self.input_use_count(input)? > 0)
    }
}

impl Analysis for UseCountAnalysis {
    type Output = UseCountInfo;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let mut operation_uses: HashMap<Operation, usize> = HashMap::new();
        let mut input_uses: HashMap<Input, usize> = HashMap::new();

        // Count uses in gate inputs.
        for (_, sources) in &circuit.gate_entries {
            for source in sources {
                match source {
                    Source::Input(input) => {
                        *input_uses.entry(*input).or_insert(0) += 1;
                    }
                    Source::Gate(op) => {
                        *operation_uses.entry(*op).or_insert(0) += 1;
                    }
                }
            }
        }

        // Count uses in circuit outputs.
        for output_op in &circuit.connected_outputs {
            *operation_uses.entry(*output_op).or_insert(0) += 1;
        }

        Ok(UseCountInfo {
            operation_uses,
            input_uses,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::use_count::UseCountAnalysis},
            circuit::Circuit,
        },
        handles::{Input, Operation, Source},
    };

    /// Simple test gate for demonstration
    enum TestGate {
        Add,
        Mul,
        Constant(()),
    }

    impl Gate for TestGate {
        fn arity(&self) -> usize {
            match self {
                TestGate::Add | TestGate::Mul => 2,
                TestGate::Constant(_) => 0,
            }
        }

        fn name(&self) -> &str {
            match self {
                TestGate::Add => "add",
                TestGate::Mul => "mul",
                TestGate::Constant(_) => "const",
            }
        }
    }

    #[test]
    fn simple_use_counts() {
        // Circuit: (input0 + input1) * constant.
        // Gates:
        //   0: Add(input0, input1)
        //   1: Constant(2)
        //   2: Mul(gate0, gate1)
        // Output: gate2

        let gate_entries = vec![
            (
                TestGate::Add,
                vec![Source::Input(Input::new(0)), Source::Input(Input::new(1))],
            ),
            (TestGate::Constant(()), vec![]),
            (
                TestGate::Mul,
                vec![
                    Source::Gate(Operation::new(0)),
                    Source::Gate(Operation::new(1)),
                ],
            ),
        ];

        let circuit = Circuit::new(gate_entries, 2, vec![Operation::new(2)]);

        let mut analyzer = Analyzer::new();
        let use_counts = analyzer.get::<UseCountAnalysis>(&circuit).unwrap();

        // Check input use counts.
        assert_eq!(use_counts.input_use_count(&Input::new(0)).unwrap(), 1);
        assert_eq!(use_counts.input_use_count(&Input::new(1)).unwrap(), 1);

        // Check operation use counts.
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(0)).unwrap(),
            1
        );
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(1)).unwrap(),
            1
        );
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(2)).unwrap(),
            1
        );

        // Check that all are used.
        assert!(use_counts.is_operation_used(&Operation::new(0)).unwrap());
        assert!(use_counts.is_operation_used(&Operation::new(1)).unwrap());
        assert!(use_counts.is_operation_used(&Operation::new(2)).unwrap());
    }

    #[test]
    fn fan_out_use_counts() {
        // Circuit: input0 is used by two gates
        // Gates:
        //   0: Add(input0, input0)  <- input0 used twice in same gate
        //   1: Mul(input0, gate0)   <- input0 used again
        // Output: gate1

        let gate_entries = vec![
            (
                TestGate::Add,
                vec![Source::Input(Input::new(0)), Source::Input(Input::new(0))],
            ),
            (
                TestGate::Mul,
                vec![
                    Source::Input(Input::new(0)),
                    Source::Gate(Operation::new(0)),
                ],
            ),
        ];

        let circuit = Circuit::new(gate_entries, 1, vec![Operation::new(1)]);

        let mut analyzer = Analyzer::new();
        let use_counts = analyzer.get::<UseCountAnalysis>(&circuit).unwrap();

        // input0 is used 3 times total (twice in Add, once in Mul).
        assert_eq!(use_counts.input_use_count(&Input::new(0)).unwrap(), 3);

        // gate0 is used once (by Mul).
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(0)).unwrap(),
            1
        );

        // gate1 is used once (by output).
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(1)).unwrap(),
            1
        );
    }

    #[test]
    fn multiple_outputs() {
        // Circuit with multiple outputs using same gate:
        // Gates:
        //   0: Add(input0, input1)
        // Outputs: [gate0, gate0]

        let gate_entries = vec![(
            TestGate::Add,
            vec![Source::Input(Input::new(0)), Source::Input(Input::new(1))],
        )];

        let circuit = Circuit::new(gate_entries, 2, vec![Operation::new(0), Operation::new(0)]);

        let mut analyzer = Analyzer::new();
        let use_counts = analyzer.get::<UseCountAnalysis>(&circuit).unwrap();

        // Gate 0 is used twice (by both outputs).
        assert_eq!(
            use_counts.operation_use_count(&Operation::new(0)).unwrap(),
            2
        );
    }
}
