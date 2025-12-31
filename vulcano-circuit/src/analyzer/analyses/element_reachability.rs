//! Element Reachability Analysis
//!
//! Computes which values and operations are reachable from circuit outputs.
//! An element is reachable if it contributes (directly or transitively) to an output.

use std::collections::HashSet;

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{Circuit, Operation, Producer},
    error::Result,
    gate::Gate,
    handles::ValueId,
};

/// Result of element reachability analysis.
pub(crate) struct ElementReachability {
    /// Values reachable from circuit outputs.
    values: HashSet<ValueId>,
    /// Operations reachable from circuit outputs.
    operations: HashSet<Operation>,
}

impl ElementReachability {
    /// Check if a value is reachable.
    pub(crate) fn is_value_reachable(&self, value: ValueId) -> bool {
        self.values.contains(&value)
    }

    /// Check if an operation is reachable.
    pub(crate) fn is_operation_reachable(&self, op: Operation) -> bool {
        self.operations.contains(&op)
    }

    /// Get all reachable values.
    pub(crate) fn reachable_values(&self) -> &HashSet<ValueId> {
        &self.values
    }

    /// Get all reachable operations.
    pub(crate) fn reachable_operations(&self) -> &HashSet<Operation> {
        &self.operations
    }
}

impl Analysis for ElementReachability {
    type Output = Self;

    fn run<G: Gate>(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut values = HashSet::new();
        let mut operations = HashSet::new();
        let mut worklist: Vec<ValueId> = Vec::new();

        // Seed with output operations and their input values.
        for (output_id, output) in circuit.all_outputs() {
            operations.insert(Operation::Output(output_id));
            let value_id = output.get_input();
            if values.insert(value_id) {
                worklist.push(value_id);
            }
        }

        // Walk backwards through producers.
        while let Some(value_id) = worklist.pop() {
            let value = circuit.value(value_id)?;

            match value.get_producer() {
                Producer::Input(input_id) => {
                    operations.insert(Operation::Input(input_id));
                }
                Producer::Gate(gate_id) => {
                    operations.insert(Operation::Gate(gate_id));
                    let gate = circuit.gate_op(gate_id)?;
                    for &input_value in gate.get_inputs() {
                        if values.insert(input_value) {
                            worklist.push(input_value);
                        }
                    }
                }
                Producer::Clone(clone_id) => {
                    operations.insert(Operation::Clone(clone_id));
                    let clone = circuit.clone_op(clone_id)?;
                    let input_value = clone.get_input();
                    if values.insert(input_value) {
                        worklist.push(input_value);
                    }
                }
            }
        }

        Ok(ElementReachability { values, operations })
    }
}
