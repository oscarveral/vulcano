//! Dead Code Elimination Pass
//!
//! Removes unreachable operations and values from the circuit.
//! Modifies the circuit in-place by removing elements that don't contribute to outputs.

use std::any::TypeId;

use crate::{
    analyzer::{Analyzer, analyses::element_reachability::ElementReachability},
    circuit::{Circuit, operations::Operation},
    error::{Error, Result},
    gate::Gate,
};

/// Eliminate dead code by removing unreachable elements from the circuit.
pub fn dead_code_elimination<G: Gate>(
    mut circuit: Circuit<G>,
    analyzer: &mut Analyzer<G>,
) -> Result<(Circuit<G>, Vec<TypeId>)> {
    let reachability = analyzer.get::<ElementReachability>(&circuit)?;

    // Track if any changes were made.
    let mut made_changes = false;

    // Process each subcircuit.
    for subcircuit in circuit.iter_mut() {
        let subcircuit_id = subcircuit.id();
        let reach = reachability
            .for_subcircuit(subcircuit_id)
            .ok_or(Error::SubcircuitAnalysisMissing(subcircuit_id))?;

        // Check early exit: if all operations are reachable, skip this subcircuit.
        let total_ops = subcircuit.all_operations().count();
        if reach.reachable_operations().len() == total_ops {
            continue;
        }

        made_changes = true;

        // Collect unreachable operations (we need to collect first since we'll mutate).
        let unreachable_gates: Vec<_> = subcircuit
            .all_gates()
            .filter(|(id, _)| !reach.is_operation_reachable(Operation::Gate(*id)))
            .map(|(id, _)| id)
            .collect();

        let unreachable_clones: Vec<_> = subcircuit
            .all_clones()
            .filter(|(id, _)| !reach.is_operation_reachable(Operation::Clone(*id)))
            .map(|(id, _)| id)
            .collect();

        let unreachable_drops: Vec<_> = subcircuit
            .all_drops()
            .filter(|(id, _)| !reach.is_operation_reachable(Operation::Drop(*id)))
            .map(|(id, _)| id)
            .collect();

        let unreachable_inputs: Vec<_> = subcircuit
            .all_inputs()
            .filter(|(id, _)| !reach.is_operation_reachable(Operation::Input(*id)))
            .map(|(id, _)| id)
            .collect();

        let unreachable_outputs: Vec<_> = subcircuit
            .all_outputs()
            .filter(|(id, _)| !reach.is_operation_reachable(Operation::Output(*id)))
            .map(|(id, _)| id)
            .collect();

        let unreachable_values: Vec<_> = subcircuit
            .all_values()
            .filter(|(id, _)| !reach.is_value_reachable(*id))
            .map(|(id, _)| id)
            .collect();

        // Safe because reachability analysis guarantees unreachable elements
        // are not referenced by any reachable elements.
        for id in unreachable_gates {
            subcircuit.remove_gate_unchecked(id);
        }
        for id in unreachable_clones {
            subcircuit.remove_clone_unchecked(id);
        }
        for id in unreachable_drops {
            subcircuit.remove_drop_unchecked(id);
        }
        for id in unreachable_inputs {
            subcircuit.remove_input_unchecked(id);
        }
        for id in unreachable_outputs {
            subcircuit.remove_output_unchecked(id);
        }
        for id in unreachable_values {
            subcircuit.remove_value_unchecked(id);
        }
    }

    // If no changes were made, preserve the reachability analysis.
    if !made_changes {
        return Ok((circuit, Vec::from([TypeId::of::<ElementReachability>()])));
    }

    // All cached analyses are invalidated after mutation.
    Ok((circuit, Vec::with_capacity(0)))
}
