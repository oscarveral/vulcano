//! Dead Code Elimination Pass
//!
//! Removes unreachable operations and values from the circuit.
//! Modifies the circuit in-place by removing elements that don't contribute to outputs.

use std::any::TypeId;

use crate::{
    analyzer::{Analyzer, analyses::element_reachability::ElementReachability},
    circuit::{Circuit, Operation},
    error::Result,
    gate::Gate,
    handles::ValueId,
};

/// Eliminate dead code by removing unreachable elements from the circuit.
pub fn dead_code_elimination<G: Gate>(
    mut circuit: Circuit<G>,
    analyzer: &mut Analyzer<G>,
) -> Result<(Circuit<G>, Vec<TypeId>)> {
    let reachability = analyzer.get::<ElementReachability>(&circuit)?;

    // If everything is reachable, nothing to do.
    let total_ops = circuit.all_operations().count();
    if reachability.reachable_operations().len() == total_ops {
        return Ok((circuit, Vec::from([TypeId::of::<ElementReachability>()])));
    }

    // Collect unreachable operations (we need to collect first since we'll mutate).
    let unreachable_gates: Vec<_> = circuit
        .all_gates()
        .filter(|(id, _)| !reachability.is_operation_reachable(Operation::Gate(*id)))
        .map(|(id, _)| id)
        .collect();

    let unreachable_clones: Vec<_> = circuit
        .all_clones()
        .filter(|(id, _)| !reachability.is_operation_reachable(Operation::Clone(*id)))
        .map(|(id, _)| id)
        .collect();

    let unreachable_drops: Vec<_> = circuit
        .all_drops()
        .filter(|(id, _)| !reachability.is_operation_reachable(Operation::Drop(*id)))
        .map(|(id, _)| id)
        .collect();

    let unreachable_inputs: Vec<_> = circuit
        .all_inputs()
        .filter(|(id, _)| !reachability.is_operation_reachable(Operation::Input(*id)))
        .map(|(id, _)| id)
        .collect();

    let unreachable_outputs: Vec<_> = circuit
        .all_outputs()
        .filter(|(id, _)| !reachability.is_operation_reachable(Operation::Output(*id)))
        .map(|(id, _)| id)
        .collect();

    let unreachable_values: Vec<ValueId> = circuit
        .all_values()
        .filter(|(id, _)| !reachability.is_value_reachable(*id))
        .map(|(id, _)| id)
        .collect();

    // Safe because reachability analysis guarantees unreachable elements
    // are not referenced by any reachable elements.
    for id in unreachable_gates {
        circuit.remove_gate_unchecked(id);
    }
    for id in unreachable_clones {
        circuit.remove_clone_unchecked(id);
    }
    for id in unreachable_drops {
        circuit.remove_drop_unchecked(id);
    }
    for id in unreachable_inputs {
        circuit.remove_input_unchecked(id);
    }
    for id in unreachable_outputs {
        circuit.remove_output_unchecked(id);
    }
    for id in unreachable_values {
        circuit.remove_value_unchecked(id);
    }

    // All cached analyses are invalidated after mutation.
    Ok((circuit, Vec::with_capacity(0)))
}
