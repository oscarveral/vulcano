//! Dead Code Elimination Pass
//!
//! Removes unreachable operations and values from the circuit.
//! Rebuilds the circuit with only elements that contribute to outputs.

use std::any::TypeId;
use std::collections::HashMap;

use crate::{
    analyzer::{Analyzer, analyses::element_reachability::ElementReachability},
    circuit::{Circuit, Operation},
    error::Result,
    gate::Gate,
    handles::{CloneId, DropId, GateId, InputId, OutputId, ValueId},
};

/// Eliminate dead code by rebuilding the circuit with only reachable elements.
pub(crate) fn dead_code_elimination<G: Gate>(
    circuit: Circuit<G>,
    analyzer: &mut Analyzer<G>,
) -> Result<(Circuit<G>, Vec<TypeId>)> {
    let reachability = analyzer.get::<ElementReachability>(&circuit)?;

    // If everything is reachable, return unchanged.
    let total_ops = circuit.all_operations().count();
    if reachability.reachable_operations().len() == total_ops {
        return Ok((circuit, Vec::from([TypeId::of::<ElementReachability>()])));
    }

    // Consume the circuit to get ownership of all parts.
    let (gates, clones, drops, inputs, outputs, values) = circuit.into_parts();

    // Rebuild circuit with only reachable elements.
    let mut new_circuit = Circuit::<G>::new();
    let mut value_map: HashMap<ValueId, ValueId> = HashMap::new();

    // Step 1: Rebuild inputs (move value types).
    for (idx, input_op) in inputs.into_iter().enumerate() {
        let input_id = InputId::new(idx);
        if reachability.is_operation_reachable(Operation::Input(input_id)) {
            let old_value = input_op.get_output();
            let value_type = values[old_value.index()].value_type;
            let (_, new_value) = new_circuit.add_input(value_type);
            value_map.insert(old_value, new_value);
        }
    }

    // Step 2: Rebuild gates (move gate descriptors).
    for (idx, gate_op) in gates.into_iter().enumerate() {
        let gate_id = GateId::new(idx);
        if reachability.is_operation_reachable(Operation::Gate(gate_id)) {
            let new_inputs: Vec<ValueId> =
                gate_op.inputs.iter().map(|old| value_map[old]).collect();

            // Move the gate descriptor, no clone.
            let (_, new_outputs) = new_circuit.add_gate(gate_op.gate, new_inputs)?;

            for (old, new) in gate_op.outputs.iter().zip(new_outputs) {
                value_map.insert(*old, new);
            }
        }
    }

    // Step 3: Rebuild clones.
    for (idx, clone_op) in clones.into_iter().enumerate() {
        let clone_id = CloneId::new(idx);
        if reachability.is_operation_reachable(Operation::Clone(clone_id)) {
            let new_input = value_map[&clone_op.input];
            let (_, new_outputs) = new_circuit.add_clone(new_input, clone_op.outputs.len());

            for (old, new) in clone_op.outputs.iter().zip(new_outputs) {
                value_map.insert(*old, new);
            }
        }
    }

    // Step 4: Rebuild drops.
    for (idx, drop_op) in drops.into_iter().enumerate() {
        let drop_id = DropId::new(idx);
        if reachability.is_operation_reachable(Operation::Drop(drop_id)) {
            if let Some(&new_input) = value_map.get(&drop_op.input) {
                new_circuit.add_drop(new_input);
            }
        }
    }

    // Step 5: Rebuild outputs.
    for (idx, output_op) in outputs.into_iter().enumerate() {
        let output_id = OutputId::new(idx);
        if reachability.is_operation_reachable(Operation::Output(output_id)) {
            let new_input = value_map[&output_op.get_input()];
            new_circuit.add_output(new_input);
        }
    }

    Ok((new_circuit, Vec::new()))
}
