//! Ownership Reconciliation Pass
//!
//! Fixes ownership issues in the circuit:
//! - Inserts drops for leaked values (never consumed).
//! - Inserts clones for overconsumed values (moved multiple times).

use std::any::TypeId;

use crate::{
    analyzer::{Analyzer, analyses::ownership_issues::OwnershipIssues},
    circuit::Circuit,
    error::Result,
    gate::Gate,
};

/// Reconcile ownership issues by inserting drops and clones.
pub fn reconcile_ownership<G: Gate>(
    mut circuit: Circuit<G>,
    analyzer: &mut Analyzer<G>,
) -> Result<(Circuit<G>, Vec<TypeId>)> {
    // Get ownership analysis.
    let issues = analyzer.get::<OwnershipIssues>(&circuit)?;

    // Process each subcircuit.
    for subcircuit in circuit.iter_mut() {
        let subcircuit_id = subcircuit.id();
        let subcircuit_issues = match issues.for_subcircuit(subcircuit_id) {
            Some(issues) => issues,
            None => continue, // No issues found for this subcircuit.
        };

        // Skip if ownership is already valid.
        if subcircuit_issues.is_valid() {
            continue;
        }

        // Insert drops for leaked values.
        for value_id in subcircuit_issues.leaked() {
            subcircuit.add_drop(value_id)?;
        }

        // Insert clones for overconsumed values.
        for (value_id, move_count) in subcircuit_issues.overconsumed() {
            // One consumer uses the original, the rest use clone outputs.
            let clone_count = move_count - 1;

            // Get all move usages before inserting clone.
            let move_uses = subcircuit.get_move_destinations(value_id)?;

            // Insert clone that produces (N-1) copies.
            let (_, clone_outputs) = subcircuit.add_clone(value_id, clone_count)?;

            // Rewire all but the first move to use clone outputs instead.
            for (usage, clone_output) in move_uses.iter().skip(1).zip(clone_outputs.iter()) {
                subcircuit.rewire_destination(
                    value_id,
                    *clone_output,
                    usage.get_consumer(),
                    usage.get_port(),
                )?;
            }
        }
    }

    Ok((circuit, Vec::new()))
}
