//! Scheduler for converting circuits into execution plans.
//!
//! The scheduler takes an optimized circuit and produces an execution plan
//! by running necessary analyses and organizing gates into partitions and layers.

use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{
            Analyzer,
            analyses::{
                subcircuit::SubCircuitAnalysis,
                topological::TopologicalOrder,
                wire_allocation::{WireAllocation, WireAllocationAnalysis},
            },
        },
        circuit::Circuit,
        plan::{ExecutionPlan, Layer, Partition, Step},
    },
    handles::{GateId, InputId, OutputId, Value, Wire},
};

/// Scheduler that converts circuits into execution plans.
///
/// The scheduler reuses the analyzer from the optimizer to avoid
/// recomputing expensive analyses.
pub struct Scheduler<T: Gate> {
    analyzer: Analyzer<T>,
}

/// Context containing all data needed to build partitions.
struct SchedulerContext<'a, T> {
    num_subcircuits: usize,
    gate_entries: Vec<(T, Vec<Value>)>,
    output_connections: Vec<GateId>,
    topo_order: &'a [GateId],
    wire_allocation: &'a WireAllocation,
    subcircuit_gates: HashMap<usize, Vec<GateId>>,
    subcircuit_inputs: HashMap<usize, Vec<InputId>>,
    subcircuit_outputs: HashMap<usize, Vec<OutputId>>,
}

impl<T: Gate> Scheduler<T> {
    /// Create a new scheduler with the given analyzer.
    pub fn new(analyzer: Analyzer<T>) -> Self {
        Self { analyzer }
    }

    /// Convert a circuit into an execution plan.
    ///
    /// This consumes both the scheduler and the circuit, producing
    /// a ready-to-execute plan.
    pub fn schedule(mut self, circuit: Circuit<T>) -> Result<ExecutionPlan<T>> {
        // Run necessary analyses
        let subcircuit_info = self.analyzer.get::<SubCircuitAnalysis>(&circuit)?;
        let topo_order = self.analyzer.get::<TopologicalOrder>(&circuit)?;
        let wire_allocation = self.analyzer.get::<WireAllocationAnalysis>(&circuit)?;

        // Group gates by subcircuit.
        let mut subcircuit_gates: HashMap<usize, Vec<GateId>> = HashMap::new();
        for gate in circuit.operations() {
            let subcircuit_id = subcircuit_info.operation_subcircuit(&gate)?;
            subcircuit_gates
                .entry(subcircuit_id)
                .or_default()
                .push(gate);
        }

        // Group inputs by subcircuit.
        let mut subcircuit_inputs: HashMap<usize, Vec<InputId>> = HashMap::new();
        for input in circuit.inputs() {
            let subcircuit_id = subcircuit_info.input_subcircuit(&input)?;
            subcircuit_inputs
                .entry(subcircuit_id)
                .or_default()
                .push(input);
        }

        // Group outputs by subcircuit (via their connected gates).
        let mut subcircuit_outputs: HashMap<usize, Vec<OutputId>> = HashMap::new();
        for output in circuit.outputs() {
            let gate = circuit.output_connections()[output.id()];
            let subcircuit_id = subcircuit_info.operation_subcircuit(&gate)?;
            subcircuit_outputs
                .entry(subcircuit_id)
                .or_default()
                .push(output);
        }

        // Consume circuit to get all components.
        let (gate_entries, _input_count, output_connections) = circuit.into_parts();

        // Build all partitions.
        let context = SchedulerContext {
            num_subcircuits: subcircuit_info.subcircuit_count,
            gate_entries,
            output_connections,
            topo_order: &topo_order,
            wire_allocation: &wire_allocation,
            subcircuit_gates,
            subcircuit_inputs,
            subcircuit_outputs,
        };

        let partitions = self.build_all_partitions(context)?;

        Ok(ExecutionPlan { partitions })
    }

    /// Build all partitions from circuit components.
    fn build_all_partitions(&self, context: SchedulerContext<T>) -> Result<Vec<Partition<T>>> {
        // Convert gate_entries Vec into HashMap for easy extraction.
        let mut gate_map: HashMap<usize, (T, Vec<Value>)> =
            context.gate_entries.into_iter().enumerate().collect();

        let mut partitions = Vec::with_capacity(context.num_subcircuits);

        for subcircuit_id in 0..context.num_subcircuits {
            let gates = context
                .subcircuit_gates
                .get(&subcircuit_id)
                .map(|v| v.as_slice())
                .ok_or(Error::SubCircuitNotFound(subcircuit_id))?;

            let inputs = context
                .subcircuit_inputs
                .get(&subcircuit_id)
                .map(|v| v.as_slice())
                .ok_or(Error::SubCircuitNotFound(subcircuit_id))?;

            let outputs = context
                .subcircuit_outputs
                .get(&subcircuit_id)
                .map(|v| v.as_slice())
                .ok_or(Error::SubCircuitNotFound(subcircuit_id))?;

            // Filter topological order to only gates in this subcircuit.
            let ordered_gates: Vec<GateId> = context
                .topo_order
                .iter()
                .filter(|gate| gates.contains(gate))
                .copied()
                .collect();

            // Build layers for this partition.
            let mut layers: Vec<Layer<T>> = Vec::with_capacity(ordered_gates.len());

            for gate_id in ordered_gates {
                let gate_idx = gate_id.id();
                // Remove and take ownership of the gate and its sources from the map.
                let (gate, sources) = gate_map
                    .remove(&gate_idx)
                    .ok_or(Error::SubCircuitNotFound(subcircuit_id))?;

                // Get input wires.
                let input_wires: Vec<Wire> = sources
                    .iter()
                    .map(|value| match value {
                        Value::Input(input) => context.wire_allocation.input_wire(input),
                        Value::Gate(op) => context.wire_allocation.operation_wire(op),
                    })
                    .collect::<Option<Vec<_>>>()
                    .ok_or(Error::WireAllocationValueNotColored)?;

                // Get output wire.
                let output_wire = context
                    .wire_allocation
                    .operation_wire(&gate_id)
                    .ok_or(Error::WireAllocationValueNotColored)?;

                let step = Step {
                    gate,
                    inputs: input_wires,
                    output: output_wire,
                };

                layers.push(Layer { steps: vec![step] });
            }

            // Get memory size for this subcircuit.
            let memory_size = context
                .wire_allocation
                .subcircuit_wire_count(subcircuit_id)
                .ok_or(Error::SubCircuitNotFound(subcircuit_id))?;

            // Build input bindings.
            let input_bindings: Vec<(InputId, Wire)> = inputs
                .iter()
                .filter_map(|input| {
                    context
                        .wire_allocation
                        .input_wire(input)
                        .map(|wire| (*input, wire))
                })
                .collect();

            // Build output bindings.
            let output_bindings: Vec<(OutputId, Wire)> = outputs
                .iter()
                .filter_map(|output| {
                    let gate = context.output_connections[output.id()];
                    context
                        .wire_allocation
                        .operation_wire(&gate)
                        .map(|wire| (*output, wire))
                })
                .collect();

            partitions.push(Partition {
                layers,
                memory_size,
                input_bindings,
                output_bindings,
            });
        }

        Ok(partitions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{builder::Builder, optimizer::Optimizer},
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
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
    fn basic_pipeline_test() {
        // Circuit: input -> negate -> output
        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut optimizer = Optimizer::new();
        let circuit = optimizer.optimize(circuit).unwrap();
        let scheduler = optimizer.into_scheduler();
        let plan = scheduler.schedule(circuit).unwrap();

        // Should have 1 partition (single connected circuit).
        assert_eq!(plan.partitions.len(), 1);

        let partition = &plan.partitions[0];
        // Should have 1 layer with 1 step.
        assert_eq!(partition.layers.len(), 1);
        assert_eq!(partition.layers[0].steps.len(), 1);

        // Should need only 1 wire (input and output reuse same wire).
        assert_eq!(partition.memory_size, 1);

        // Should have 1 input binding and 1 output binding.
        assert_eq!(partition.input_bindings.len(), 1);
        assert_eq!(partition.output_bindings.len(), 1);
    }

    #[test]
    fn multiple_partitions_test() {
        // Two independent circuits:
        // input1 -> negate1 -> output1
        // input2 -> negate2 -> output2
        let mut builder: Builder<TestGate> = Builder::new();

        let input1 = builder.add_input();
        let gate1 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        builder.connect_input_to_gate(input1, gate1).unwrap();
        builder.connect_gate_to_output(gate1, output1).unwrap();

        let input2 = builder.add_input();
        let gate2 = builder.add_gate(TestGate::Negate);
        let output2 = builder.add_output();
        builder.connect_input_to_gate(input2, gate2).unwrap();
        builder.connect_gate_to_output(gate2, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut optimizer = Optimizer::new();
        let circuit = optimizer.optimize(circuit).unwrap();
        let scheduler = optimizer.into_scheduler();
        let plan = scheduler.schedule(circuit).unwrap();

        // Should have 2 partitions (disjoint circuits).
        assert_eq!(plan.partitions.len(), 2);

        // Each partition should be independent.
        for partition in &plan.partitions {
            assert_eq!(partition.layers.len(), 1);
            assert_eq!(partition.layers[0].steps.len(), 1);
            // Each should need only 1 wire.
            assert_eq!(partition.memory_size, 1);
        }
    }

    #[test]
    fn complex_circuit_test() {
        // Diamond pattern with wire reuse:
        // input -> negate1 -> addition -> output
        //       -> negate2 ->
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
        let mut optimizer = Optimizer::new();
        let circuit = optimizer.optimize(circuit).unwrap();
        let scheduler = optimizer.into_scheduler();
        let plan = scheduler.schedule(circuit).unwrap();

        // Should have 1 partition.
        assert_eq!(plan.partitions.len(), 1);

        let partition = &plan.partitions[0];
        // Should have 3 layers (negate1, negate2, addition).
        assert_eq!(partition.layers.len(), 3);

        // Verify topological order is preserved
        // (negates before addition).
        assert_eq!(partition.layers[0].steps[0].gate, TestGate::Negate);
        assert_eq!(partition.layers[1].steps[0].gate, TestGate::Negate);
        assert_eq!(partition.layers[2].steps[0].gate, TestGate::Addition);

        // Wire allocation should be efficient (at most 2-3 wires).
        assert!(partition.memory_size <= 3);
    }

    #[test]
    fn input_output_binding_test() {
        // Verify input_bindings and output_bindings are correct.
        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut optimizer = Optimizer::new();
        let circuit = optimizer.optimize(circuit).unwrap();
        let scheduler = optimizer.into_scheduler();
        let plan = scheduler.schedule(circuit).unwrap();

        let partition = &plan.partitions[0];

        // Check input bindings.
        assert_eq!(partition.input_bindings.len(), 1);
        assert_eq!(partition.input_bindings[0].0, input);

        // Check output bindings.
        assert_eq!(partition.output_bindings.len(), 1);
        assert_eq!(partition.output_bindings[0].0, output);

        // Input and output should use the same wire (reuse).
        assert_eq!(
            partition.input_bindings[0].1,
            partition.output_bindings[0].1
        );
    }
}
