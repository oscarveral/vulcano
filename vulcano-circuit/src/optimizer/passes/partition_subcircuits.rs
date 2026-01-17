//! Subcircuit Partitioning Pass
//!
//! Splits subcircuits that contain disjoint (unconnected) parts into separate subcircuits.
//! Keeps the largest connected component in the original subcircuit and moves
//! smaller components to new subcircuits.

use std::any::TypeId;

use crate::{
    analyzer::{Analyzer, analyses::connected_components::ConnectedComponents},
    circuit::{Circuit, operations::Operation},
    error::Result,
    gate::Gate,
};

/// Partition subcircuits with disjoint parts into separate subcircuits.
pub fn partition_subcircuits<G: Gate>(
    mut circuit: Circuit<G>,
    analyzer: &mut Analyzer<G>,
) -> Result<(Circuit<G>, Vec<TypeId>)> {
    let components = analyzer.get::<ConnectedComponents>(&circuit)?;

    // Find subcircuits that need partitioning.
    let subcircuits_to_partition: Vec<_> = components
        .iter()
        .filter(|(_, comp)| comp.is_disjoint())
        .map(|(id, _)| id)
        .collect();

    if subcircuits_to_partition.is_empty() {
        // Nothing to partition, preserve analysis.
        return Ok((circuit, vec![TypeId::of::<ConnectedComponents>()]));
    }

    // Process each subcircuit that needs partitioning.
    for subcircuit_id in subcircuits_to_partition {
        let comp = match components.for_subcircuit(subcircuit_id) {
            Some(c) => c,
            None => continue,
        };

        // Keep the largest component in the original subcircuit.
        let keep_component = comp.largest_component();

        // For each other component, split it into a new subcircuit.
        for component_id in comp.component_ids() {
            if component_id == keep_component {
                continue;
            }

            // Collect operations for this component.
            let ops: Vec<Operation> = comp.operations_in(component_id).collect();

            // Split the component into a new subcircuit.
            circuit.split_subcircuit(subcircuit_id, &ops)?;
        }
    }

    // All analyses are invalidated after partitioning.
    Ok((circuit, Vec::new()))
}
