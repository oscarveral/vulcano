//! Reorder gates in a circuit in topological order.
//! This is done by performing a topological sort on the circuit's
//! gate entries, ensuring that all dependencies are resolved before
//! a gate is executed.

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        circuit::{Circuit, Use},
        optimizer::OptimizationState,
    },
    handles::{Operation, Wire},
};

use std::collections::{HashMap, VecDeque};

/// Reorder [`Circuit::gate_entries`] in topological order (producers
/// before consumers). Returns [`Error::CycleDetected`] if a cycle is
/// detected.
pub(super) fn topological_sort<T: Gate>(
    circuit: &mut Circuit<T>,
    state: &mut OptimizationState,
) -> Result<()> {
    if state.finalized {
        // Cannot reorder finalized circuit.
        return Err(Error::AlreadyFinalized);
    }
    if state.ordered {
        // Already ordered; no-op.
        return Ok(());
    }

    let n = circuit.gate_entries.len();

    // Map each wire id that is an output of a gate to that gate index.
    let mut wire_to_gate: HashMap<usize, usize> = HashMap::with_capacity(n);
    for (i, entry) in circuit.gate_entries.iter().enumerate() {
        let wire_id = entry.2.id();
        wire_to_gate.insert(wire_id, i);
    }

    // Build adjacency list (edge: src -> dst) and indegree counts.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indeg: Vec<usize> = vec![0; n];
    for (dst, indegree) in indeg.iter_mut().enumerate() {
        let uses = &circuit.gate_entries[dst].1;
        for u in uses.iter() {
            match u {
                Use::Read(w) | Use::Consume(w) => {
                    if let Some(&src) = wire_to_gate.get(&w.id()) {
                        adj[src].push(dst);
                        *indegree += 1;
                    }
                }
            }
        }
    }

    // For determinism, sort neighbor lists by index.
    for neighbors in &mut adj {
        neighbors.sort_unstable();
    }

    // Kahn's algorithm.
    let mut q: VecDeque<usize> = VecDeque::new();
    for (i, indegree) in indeg.iter().enumerate() {
        if *indegree == 0 {
            q.push_back(i);
        }
    }

    let mut topo: Vec<usize> = Vec::with_capacity(n);
    while let Some(u) = q.pop_front() {
        topo.push(u);
        for &v in &adj[u] {
            indeg[v] -= 1;
            if indeg[v] == 0 {
                q.push_back(v);
            }
        }
    }

    if topo.len() != n {
        // Collect nodes involved in the cycle (those with indegree > 0).
        let mut cycle_ops: Vec<Operation> = Vec::new();
        for (i, indegree) in indeg.iter().enumerate() {
            if *indegree > 0 {
                cycle_ops.push(Operation::new(i));
            }
        }
        return Err(Error::CycleDetected(cycle_ops));
    }

    // Reorder gate_entries according to topo.
    let old = std::mem::take(&mut circuit.gate_entries);
    let mut slots: Vec<Option<_>> = old.into_iter().map(Some).collect();
    let mut new_entries: Vec<(T, Vec<Use>, Wire)> = Vec::with_capacity(n);
    for &i in &topo {
        if let Some(entry) = slots[i].take() {
            new_entries.push(entry);
        } else {
            return Err(Error::InvariantViolation(format!(
                "Expected gate entry at index {} while reordering!",
                i
            )));
        }
    }

    circuit.gate_entries = new_entries;
    state.ordered = true;

    Ok(())
}
