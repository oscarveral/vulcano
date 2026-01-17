//! Element Reachability Analysis
//!
//! Computes which values and operations are reachable from circuit outputs.
//! An element is reachable if it contributes (directly or transitively) to an output.

use std::collections::{HashMap, HashSet};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{
        Circuit,
        operations::{Operation, Producer},
        subcircuit::{CircuitId, Subcircuit},
        value::ValueId,
    },
    error::Result,
    gate::Gate,
};

/// Per-subcircuit reachability result.
pub struct SubcircuitReachability {
    /// Values reachable from circuit outputs.
    values: HashSet<ValueId>,
    /// Operations reachable from circuit outputs.
    operations: HashSet<Operation>,
}

impl SubcircuitReachability {
    /// Check if a value is reachable.
    pub fn is_value_reachable(&self, value: ValueId) -> bool {
        self.values.contains(&value)
    }

    /// Check if an operation is reachable.
    pub fn is_operation_reachable(&self, op: Operation) -> bool {
        self.operations.contains(&op)
    }

    /// Get all reachable values.
    pub fn reachable_values(&self) -> &HashSet<ValueId> {
        &self.values
    }

    /// Get all reachable operations.
    pub fn reachable_operations(&self) -> &HashSet<Operation> {
        &self.operations
    }
}

/// Result of element reachability analysis.
pub struct ElementReachability {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitReachability>,
}

impl ElementReachability {
    /// Get the reachability for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitReachability> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitReachability)> {
        self.results.iter().map(|(&id, reach)| (id, reach))
    }
}

impl<G: Gate> Analysis<G> for ElementReachability {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let reachability = compute_reachability(subcircuit)?;
            results.insert(subcircuit.id(), reachability);
        }

        Ok(ElementReachability { results })
    }
}

/// Compute reachability for a single subcircuit.
fn compute_reachability<G: Gate>(subcircuit: &Subcircuit<G>) -> Result<SubcircuitReachability> {
    let mut values = HashSet::new();
    let mut operations = HashSet::new();
    let mut worklist: Vec<ValueId> = Vec::new();

    // Seed with output operations and their input values.
    for (output_id, output) in subcircuit.all_outputs() {
        operations.insert(Operation::Output(output_id));
        let value_id = output.get_input();
        if values.insert(value_id) {
            worklist.push(value_id);
        }
    }

    // Walk backwards through producers.
    while let Some(value_id) = worklist.pop() {
        let value = subcircuit.value(value_id)?;

        match value.get_product().get_producer() {
            Producer::Input(input_id) => {
                operations.insert(Operation::Input(input_id));
            }
            Producer::Gate(gate_id) => {
                operations.insert(Operation::Gate(gate_id));
                let gate = subcircuit.gate_op(gate_id)?;
                for &input_value in gate.get_inputs() {
                    if values.insert(input_value) {
                        worklist.push(input_value);
                    }
                }
            }
            Producer::Clone(clone_id) => {
                operations.insert(Operation::Clone(clone_id));
                let clone = subcircuit.clone_op(clone_id)?;
                let input_value = clone.get_input();
                if values.insert(input_value) {
                    worklist.push(input_value);
                }
            }
        }
    }

    Ok(SubcircuitReachability { values, operations })
}
