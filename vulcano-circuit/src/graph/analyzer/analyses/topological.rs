//! Topological order analysis for circuits.
//! This module provides functionality to compute a topological
//! ordering of the gates in a circuit, detecting cycles if present.

use std::collections::{HashMap, VecDeque};

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::{Circuit, Use},
    },
    handles::Operation,
};

/// Analysis that computes a topological ordering of the gates in a circuit.
pub struct TopologicalOrder;

impl Analysis for TopologicalOrder {
    type Output = Vec<usize>;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer) -> Result<Self::Output> {
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

        Ok(topo)
    }
}
