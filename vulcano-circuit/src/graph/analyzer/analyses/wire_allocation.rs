//! Wire allocation analysis for circuits.
//!
//! This module assigns wire IDs to operations and inputs, minimizing total wires
//! through graph coloring based on liveness interference. Gates can reuse wires from their
//! last-used inputs for aggressive optimization.

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{
            Analysis, Analyzer,
            analyses::{
                last_use::{LastUseAnalysis, LastUseInfo},
                liveness::{LivenessAnalysis, LivenessInfo},
                subcircuit::{SubCircuitAnalysis, SubCircuitInfo},
            },
        },
        circuit::Circuit,
    },
    handles::{GateId, InputId, Value, Wire},
};

/// Analysis that computes optimal wire assignments for a circuit.
pub struct WireAllocationAnalysis;

/// Wire assignment information for a circuit.
#[derive(Debug, Clone)]
pub struct WireAllocation {
    /// Wire assignments for each operation's output.
    pub operation_wires: HashMap<GateId, Wire>,
    /// Wire assignments for each circuit input.
    pub input_wires: HashMap<InputId, Wire>,
    /// Total number of wires needed.
    pub wire_count: usize,
}

impl WireAllocation {
    /// Get the wire assigned to an operation's output.
    pub fn operation_wire(&self, op: &GateId) -> Option<Wire> {
        self.operation_wires.get(op).copied()
    }

    /// Get the wire assigned to a circuit input.
    pub fn input_wire(&self, input: &InputId) -> Option<Wire> {
        self.input_wires.get(input).copied()
    }

    /// Returns the total number of wires allocated.
    pub fn total_wires(&self) -> usize {
        self.wire_count
    }
}

impl Analysis for WireAllocationAnalysis {
    type Output = WireAllocation;

    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let liveness = analyzer.get::<LivenessAnalysis>(circuit)?;
        let last_use = analyzer.get::<LastUseAnalysis>(circuit)?;
        let subcircuit = analyzer.get::<SubCircuitAnalysis>(circuit)?;

        // Collect all values.
        let values = collect_values(circuit);

        // Build interference graph with aggressive reuse.
        let interference =
            build_interference_graph(&values, &liveness, &last_use, &subcircuit, circuit)?;

        // Assign colors (wires) using greedy coloring.
        let coloring = greedy_coloring(&values, &interference)?;

        // Convert coloring to wire assignments.
        convert_to_wire_allocation(values, coloring)
    }
}

/// Collect all values (operations and inputs) from the circuit.
fn collect_values<T: Gate>(circuit: &Circuit<T>) -> Vec<Value> {
    let mut values = Vec::new();

    for op in circuit.operations() {
        values.push(Value::Gate(op));
    }

    for input in circuit.inputs() {
        values.push(Value::Input(input));
    }

    values
}

/// Build interference graph where edges connect values that cannot share a wire.
///
/// Two values interfere if:
/// 1. They belong to the same sub-circuit (values in different sub-circuits never interfere).
/// 2. Their live ranges overlap.
/// 3. Neither can reuse the other's wire.
fn build_interference_graph<T: Gate>(
    values: &[Value],
    liveness: &LivenessInfo,
    last_use: &LastUseInfo,
    subcircuit: &SubCircuitInfo,
    circuit: &Circuit<T>,
) -> Result<HashMap<Value, HashSet<Value>>> {
    let mut graph: HashMap<Value, HashSet<Value>> =
        values.iter().map(|&v| (v, HashSet::new())).collect();

    // Check all pairs for interference.
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            if values_interfere(
                values[i], values[j], liveness, last_use, subcircuit, circuit,
            )? {
                graph
                    .get_mut(&values[i])
                    .ok_or(Error::WireAllocationNoColorAvailable)?
                    .insert(values[j]);

                graph
                    .get_mut(&values[j])
                    .ok_or(Error::WireAllocationNoColorAvailable)?
                    .insert(values[i]);
            }
        }
    }

    Ok(graph)
}

/// Check if two values interfere and cannot share a wire.
fn values_interfere<T: Gate>(
    v1: Value,
    v2: Value,
    liveness: &LivenessInfo,
    last_use: &LastUseInfo,
    subcircuit: &SubCircuitInfo,
    circuit: &Circuit<T>,
) -> Result<bool> {
    // Values in different sub-circuits never interfere.
    if !same_subcircuit(v1, v2, subcircuit)? {
        return Ok(false);
    }
    // Check for aggressive reuse. Can one value reuse the other's wire?
    if can_reuse_wire(v1, v2, last_use, circuit) || can_reuse_wire(v2, v1, last_use, circuit) {
        return Ok(false);
    }

    // Check if live ranges overlap.
    ranges_overlap(v1, v2, liveness)
}

/// Check if two values belong to the same sub-circuit.
fn same_subcircuit(v1: Value, v2: Value, subcircuit: &SubCircuitInfo) -> Result<bool> {
    match (v1, v2) {
        (Value::Gate(op1), Value::Gate(op2)) => subcircuit.same_subcircuit_ops(&op1, &op2),
        (Value::Gate(op), Value::Input(input)) | (Value::Input(input), Value::Gate(op)) => {
            subcircuit.same_subcircuit_op_input(&op, &input)
        }
        (Value::Input(input1), Value::Input(input2)) => {
            subcircuit.same_subcircuit_inputs(&input1, &input2)
        }
    }
}

/// Check if live ranges of two values overlap.
fn ranges_overlap(v1: Value, v2: Value, liveness: &LivenessInfo) -> Result<bool> {
    let range1 = match v1 {
        Value::Gate(op) => Some(liveness.operation_range(&op)?),
        Value::Input(input) => Some(liveness.input_range(&input)?),
    };

    let range2 = match v2 {
        Value::Gate(op) => Some(liveness.operation_range(&op)?),
        Value::Input(input) => Some(liveness.input_range(&input)?),
    };

    Ok(matches!((range1, range2), (Some(r1), Some(r2)) if r1.overlaps(r2)))
}

/// Check if `gate_output` can reuse `input_or_producer`'s wire.
fn can_reuse_wire<T: Gate>(
    gate_output: Value,
    input_or_producer: Value,
    last_use: &LastUseInfo,
    circuit: &Circuit<T>,
) -> bool {
    // gate_output must be an operation.
    let Value::Gate(gate_op) = gate_output else {
        return false;
    };

    // Get gate's input sources.
    let (_, sources) = &circuit.gate_entries[gate_op.id()];

    match input_or_producer {
        Value::Input(input) => {
            sources.contains(&Value::Input(input))
                && last_use.is_last_use_of_input(&gate_op, &input)
        }
        Value::Gate(producer_op) => {
            sources.contains(&Value::Gate(producer_op))
                && last_use.is_last_use_of_operation(&gate_op, &producer_op)
        }
    }
}

/// Assign colors (wires) to values using greedy coloring.
///
/// Orders values by degree (most constrained first) for better results.
fn greedy_coloring(
    values: &[Value],
    interference: &HashMap<Value, HashSet<Value>>,
) -> Result<HashMap<Value, usize>> {
    let mut coloring = HashMap::new();

    // Validate all values are in interference graph before sorting.
    for value in values {
        if !interference.contains_key(value) {
            return Err(Error::WireAllocationNoColorAvailable);
        }
    }

    // Sort by degree (most constrained first).
    // Safety: We validated all values exist in the graph above.
    let mut sorted_values: Vec<Value> = values.to_vec();
    sorted_values.sort_by_key(|v| Reverse(interference[v].len()));

    // Assign colors greedily.
    for &value in &sorted_values {
        let neighbors = interference
            .get(&value)
            .ok_or(Error::WireAllocationNoColorAvailable)?;

        let used_colors: HashSet<&usize> = neighbors
            .iter()
            .filter_map(|neighbor| coloring.get(neighbor))
            .collect();

        // Find smallest available color.
        let color = (0..=used_colors.len())
            .find(|c| !used_colors.contains(c))
            .ok_or(Error::WireAllocationNoColorAvailable)?;

        coloring.insert(value, color);
    }

    Ok(coloring)
}

/// Convert color assignments to wire allocations.
fn convert_to_wire_allocation(
    values: Vec<Value>,
    coloring: HashMap<Value, usize>,
) -> Result<WireAllocation> {
    // Validate all values were colored.
    for value in &values {
        if !coloring.contains_key(value) {
            return Err(Error::WireAllocationValueNotColored);
        }
    }

    let mut operation_wires = HashMap::new();
    let mut input_wires = HashMap::new();
    let mut max_color = 0;

    for (value, color) in coloring {
        max_color = max_color.max(color);
        let wire = Wire::new(color);

        match value {
            Value::Gate(op) => {
                operation_wires.insert(op, wire);
            }
            Value::Input(input) => {
                input_wires.insert(input, wire);
            }
        }
    }

    Ok(WireAllocation {
        operation_wires,
        input_wires,
        wire_count: max_color + 1,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::wire_allocation::WireAllocationAnalysis},
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
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        assert_eq!(allocation.wire_count, 1);
        assert!(allocation.input_wire(&input).is_some());
        assert!(allocation.operation_wire(&GateId::new(gate.id())).is_some());

        let input_wire = allocation.input_wire(&input).unwrap();
        let gate_wire = allocation.operation_wire(&GateId::new(gate.id())).unwrap();
        assert_eq!(input_wire, gate_wire);
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
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        assert_eq!(allocation.wire_count, 2);
    }

    #[test]
    fn multi_step_circuit() {
        // Circuit: input -> negate1 -> negate2 -> output
        // Expected: 1 wire (sequential execution, no overlap).

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
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        // Sequential circuit should reuse wires efficiently
        assert!(allocation.wire_count <= 2);
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
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        // All values should get wire assignments.
        assert!(allocation.input_wire(&input1).is_some());
        assert!(allocation.input_wire(&input2).is_some());
        assert!(
            allocation
                .operation_wire(&GateId::new(negate1.id()))
                .is_some()
        );
    }

    #[test]
    fn delayed_input_reuses_wire() {
        // Circuit: input1 -> negate1 -> negate2
        //          input2 ----------------> addition -> output
        //
        // input2 is only used at the last step, so it should be able to reuse
        // a wire that was freed by negate1 (which dies after negate2 reads it).

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_gate(negate2, addition).unwrap();
        builder.connect_input_to_gate(input2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        // With the fixed liveness analysis, input2 starts living at step 2 (when addition executes)
        // By that time, input1 and negate1 are dead, so their wires can be reused
        // We should need at most 2-3 wires total (not 5).
        assert!(
            allocation.wire_count <= 3,
            "Expected at most 3 wires, got {}",
            allocation.wire_count
        );

        // Verify all values got assignments.
        assert!(allocation.input_wire(&input1).is_some());
        assert!(allocation.input_wire(&input2).is_some());
        assert!(
            allocation
                .operation_wire(&GateId::new(negate1.id()))
                .is_some()
        );
        assert!(
            allocation
                .operation_wire(&GateId::new(negate2.id()))
                .is_some()
        );
        assert!(
            allocation
                .operation_wire(&GateId::new(addition.id()))
                .is_some()
        );
    }

    #[test]
    fn disjoint_circuits_reuse_wires() {
        // Circuit:
        // input1 -> negate1 -> negate2 -> output1
        // input2 -> negate3 -> negate4 -> output2
        //
        // Two completely independent linear circuits.
        // Each circuit needs 1 wire when executed sequentially.
        // Since they're disjoint, they should be able to share wires.
        // Total wire count should be 1, not 2.

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let negate3 = builder.add_gate(TestGate::Negate);
        let negate4 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        let output2 = builder.add_output();

        // First circuit: input1 -> negate1 -> negate2 -> output1
        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output1).unwrap();

        // Second circuit: input2 -> negate3 -> negate4 -> output2
        builder.connect_input_to_gate(input2, negate3).unwrap();
        builder.connect_gate_to_gate(negate3, negate4).unwrap();
        builder.connect_gate_to_output(negate4, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let allocation = analyzer.get::<WireAllocationAnalysis>(&circuit).unwrap();

        // Each linear circuit needs 1 wire. Since they're disjoint, they can share.
        // Total should be 1, not 2.
        assert_eq!(
            allocation.wire_count, 1,
            "Expected 1 wire for disjoint circuits, got {}",
            allocation.wire_count
        );

        // Verify all values got assignments.
        assert!(allocation.input_wire(&input1).is_some());
        assert!(allocation.input_wire(&input2).is_some());
        assert!(
            allocation
                .operation_wire(&GateId::new(negate1.id()))
                .is_some()
        );
        assert!(
            allocation
                .operation_wire(&GateId::new(negate2.id()))
                .is_some()
        );
        assert!(
            allocation
                .operation_wire(&GateId::new(negate3.id()))
                .is_some()
        );
        assert!(
            allocation
                .operation_wire(&GateId::new(negate4.id()))
                .is_some()
        );
    }
}
