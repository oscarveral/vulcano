//! Liveness analysis for circuits.
//!
//! This module provides functionality to compute live ranges for values in a circuit.
//! A live range represents the span of execution steps during which a value must be
//! kept alive (from production to last use).

use std::collections::HashMap;

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{
            Analysis, Analyzer,
            analyses::{topological::TopologicalOrder, use_count::UseCountAnalysis},
        },
        circuit::Circuit,
    },
    handles::{Input, Operation, Source},
};

/// Represents the live range of a value.
///
/// A live range spans from when a value is produced (start) to when it is last consumed (end).
/// Step indices correspond to positions in the topological order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveRange {
    /// Step index where the value is produced/becomes live.
    pub start: usize,
    /// Step index where the value is last used/dies.
    pub end: usize,
}

impl LiveRange {
    /// Check if this live range overlaps with another.
    ///
    /// Two live ranges overlap if they have any step in common.
    pub fn overlaps(&self, other: &LiveRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Return the length of this live range (number of steps it spans).
    pub fn length(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }
}

/// Analysis that computes live ranges for all values in a circuit.
pub struct LivenessAnalysis;

/// Information about live ranges in a circuit.
#[derive(Debug, Clone)]
pub struct LivenessInfo {
    /// Live ranges for each operation's output.
    pub operation_ranges: HashMap<Operation, LiveRange>,
    /// Live ranges for each circuit input.
    pub input_ranges: HashMap<Input, LiveRange>,
}

impl LivenessInfo {
    /// Get the live range for an operation's output.
    pub fn operation_range(&self, op: &Operation) -> Option<&LiveRange> {
        self.operation_ranges.get(op)
    }

    /// Get the live range for a circuit input.
    pub fn input_range(&self, input: &Input) -> Option<&LiveRange> {
        self.input_ranges.get(input)
    }

    /// Check if two operations' outputs have overlapping live ranges.
    pub fn operations_overlap(&self, op1: &Operation, op2: &Operation) -> bool {
        if let (Some(range1), Some(range2)) = (self.operation_range(op1), self.operation_range(op2))
        {
            range1.overlaps(range2)
        } else {
            false
        }
    }
}

impl Analysis for LivenessAnalysis {
    type Output = LivenessInfo;

    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        // Clone the topological order to avoid borrow conflicts.
        let topo_order = analyzer.get::<TopologicalOrder>(circuit)?.clone();
        let use_counts = analyzer.get::<UseCountAnalysis>(circuit)?;

        let mut operation_ranges: HashMap<Operation, LiveRange> = HashMap::new();
        let mut input_ranges: HashMap<Input, LiveRange> = HashMap::new();

        // Initialize live ranges for operations.
        // Start time is when the operation executes in topological order.
        for (step_idx, &gate_idx) in topo_order.iter().enumerate() {
            let op = Operation::new(gate_idx);
            operation_ranges.insert(
                op,
                LiveRange {
                    start: step_idx,
                    end: step_idx,
                },
            );
        }

        // Initialize live ranges for inputs.
        // Inputs are "produced" at step 0 (before any gate executes).
        for input_idx in 0..circuit.input_count() {
            let input = Input::new(input_idx);
            if use_counts.is_input_used(&input) {
                input_ranges.insert(input, LiveRange { start: 0, end: 0 });
            }
        }

        // Extend live ranges based on uses.
        // Scan through gates in topological order and extend ranges to cover uses.
        for (step_idx, &gate_idx) in topo_order.iter().enumerate() {
            let (_, sources) = &circuit.gate_entries[gate_idx];

            for source in sources {
                match source {
                    Source::Input(input) => {
                        // Extend input's live range to this step.
                        if let Some(range) = input_ranges.get_mut(input) {
                            range.end = range.end.max(step_idx);
                        }
                    }
                    Source::Gate(producer_op) => {
                        // Extend producer's live range to this step.
                        if let Some(range) = operation_ranges.get_mut(producer_op) {
                            range.end = range.end.max(step_idx);
                        }
                    }
                }
            }
        }

        // Note: Circuit outputs do not extend live ranges.
        // Outputs can be returned immediately when the operation executes,
        // so they don't need to keep the value alive more than necessary.

        Ok(LivenessInfo {
            operation_ranges,
            input_ranges,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::{
                Analyzer,
                analyses::liveness::{LiveRange, LivenessAnalysis},
            },
            builder::Builder,
        },
        handles::Operation,
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
        // Expected:
        //   input: [0, 0]
        //   negate: [0, 0]

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let liveness = analyzer.get::<LivenessAnalysis>(&circuit).unwrap();

        // Check input liveness.
        let input_range = liveness.input_range(&input).unwrap();
        assert_eq!(input_range.start, 0);
        assert_eq!(input_range.end, 0);

        // Check gate liveness.
        let gate_range = liveness
            .operation_range(&Operation::new(gate.id()))
            .unwrap();
        assert_eq!(gate_range.start, 0);
        assert_eq!(gate_range.end, 0);
    }

    #[test]
    fn multi_step_circuit() {
        // Circuit: input -> negate1 -> negate2 -> output
        // Expected:
        //   input: [0, 0]
        //   negate1: [0, 1]
        //   negate2: [1, 1]

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let liveness = analyzer.get::<LivenessAnalysis>(&circuit).unwrap();

        let input_range = liveness.input_range(&input).unwrap();
        assert_eq!(input_range.start, 0);
        assert_eq!(input_range.end, 0);

        let negate1_range = liveness
            .operation_range(&Operation::new(negate1.id()))
            .unwrap();
        assert_eq!(negate1_range.start, 0);
        assert_eq!(negate1_range.end, 1);

        let negate2_range = liveness
            .operation_range(&Operation::new(negate2.id()))
            .unwrap();
        assert_eq!(negate2_range.start, 1);
        assert_eq!(negate2_range.end, 1);
    }

    #[test]
    fn fan_out_circuit() {
        // Circuit: input -> negate1 -> negate2
        //                          \-> addition -> output
        // negate1 is used by both negate2 and addition

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
        let liveness = analyzer.get::<LivenessAnalysis>(&circuit).unwrap();

        // negate1 should be live until the last use (addition, which comes after negate2).
        let negate1_range = liveness
            .operation_range(&Operation::new(negate1.id()))
            .unwrap();
        assert_eq!(negate1_range.start, 0);
        // end should be at least 2 (when addition executes).
        assert!(negate1_range.end >= 2);
    }

    #[test]
    fn diamond_pattern() {
        // Circuit:
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
        let mut analyzer = Analyzer::new();
        let liveness = analyzer.get::<LivenessAnalysis>(&circuit).unwrap();

        // Input should be live until both negate gates have consumed it.
        let input_range = liveness.input_range(&input).unwrap();
        assert_eq!(input_range.start, 0);
        // end should be at least 1 (both negates execute).
        assert!(input_range.end >= 1);

        // Both negates should be live until addition executes.
        let negate1_range = liveness
            .operation_range(&Operation::new(negate1.id()))
            .unwrap();
        let negate2_range = liveness
            .operation_range(&Operation::new(negate2.id()))
            .unwrap();

        // Both should die at the addition step.
        assert_eq!(negate1_range.end, negate2_range.end);
    }

    #[test]
    fn overlap_detection() {
        // Partial overlaps
        let range1 = LiveRange { start: 0, end: 2 };
        let range2 = LiveRange { start: 1, end: 3 };
        assert!(range1.overlaps(&range2)); // [0,2] overlaps [1,3]
        assert!(range2.overlaps(&range1)); // Symmetric.

        // Touching at boundary (should overlap)
        let range3 = LiveRange { start: 3, end: 5 };
        assert!(range2.overlaps(&range3)); // [1,3] overlaps [3,5] (touch at 3)
        assert!(range3.overlaps(&range2)); // Symmetric.

        // Non-overlapping (gap between)
        let range4 = LiveRange { start: 6, end: 8 };
        assert!(!range1.overlaps(&range4)); // [0,2] doesn't overlap [6,8]
        assert!(!range4.overlaps(&range1)); // Symmetric.
        assert!(!range3.overlaps(&range4)); // [3,5] doesn't overlap [6,8]

        // Complete containment (one range inside another)
        let outer = LiveRange { start: 0, end: 10 };
        let inner = LiveRange { start: 3, end: 7 };
        assert!(outer.overlaps(&inner)); // outer contains inner.   
        assert!(inner.overlaps(&outer)); // inner is contained in outer.

        // Same range
        let same1 = LiveRange { start: 2, end: 5 };
        let same2 = LiveRange { start: 2, end: 5 };
        assert!(same1.overlaps(&same2));

        // Single-step ranges
        let single1 = LiveRange { start: 3, end: 3 };
        let single2 = LiveRange { start: 3, end: 3 };
        let single3 = LiveRange { start: 4, end: 4 };
        assert!(single1.overlaps(&single2)); // Same step.
        assert!(!single1.overlaps(&single3)); // Adjacent steps.

        // Adjacent non-overlapping
        let adj1 = LiveRange { start: 0, end: 2 };
        let adj2 = LiveRange { start: 3, end: 5 };
        assert!(!adj1.overlaps(&adj2)); // [0,2] and [3,5] don't overlap.
    }

    #[test]
    fn range_length() {
        assert_eq!(LiveRange { start: 0, end: 0 }.length(), 1);
        assert_eq!(LiveRange { start: 0, end: 5 }.length(), 6);
        assert_eq!(LiveRange { start: 3, end: 7 }.length(), 5);
    }
}
