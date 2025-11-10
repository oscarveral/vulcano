use std::any::TypeId;

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analyzer, analyses::reachability::Reachability},
        circuit::Circuit,
    },
};

/// Optimization pass that eliminates dead gates from the circuit.
///
/// Dead gates refer to gates that are not reachable from the circuit inputs
/// and cannot reach the circuit outputs. This pass relies on the
/// [`Reachability`] analysis to identify such gates.
pub fn dead_gate_elimination<T: Gate>(
    mut circuit: Circuit<T>,
    analyzer: &mut Analyzer,
) -> Result<(Circuit<T>, Vec<TypeId>)> {
    // Get the set of reachable gates from the Reachability analysis.
    let reachable_gates = analyzer.get::<Reachability, T>(&circuit)?;

    // Retain only the gates that are reachable.
    circuit.gate_entries = circuit
        .gate_entries
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| reachable_gates.contains(idx))
        .map(|(_, entry)| entry)
        .collect();

    // Invalidate all analyses since the circuit has changed.
    Ok((circuit, Vec::new()))
}
