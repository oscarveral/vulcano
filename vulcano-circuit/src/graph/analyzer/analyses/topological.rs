//! Topological order analysis for circuits.
//! This module provides functionality to compute a topological
//! ordering of the gates in a circuit, detecting cycles if present.

use std::collections::VecDeque;

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::Circuit,
    },
    handles::{Operation, Source},
};

/// Analysis that computes a topological ordering of the gates in a circuit.
pub struct TopologicalOrder;

impl Analysis for TopologicalOrder {
    type Output = Vec<usize>;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let n = circuit.gate_entries.len();

        // Build adjacency list (edge: src -> dst) and indegree counts by
        // directly traversing Source dependencies.
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut indeg: Vec<usize> = vec![0; n];

        for (dst, (_, sources)) in circuit.gate_entries.iter().enumerate() {
            for source in sources.iter() {
                match source {
                    Source::Input(_) => {
                        // External inputs contribute no dependency edges
                    }
                    Source::Gate(op) => {
                        let src = op.id();
                        adj[src].push(dst);
                        indeg[dst] += 1;
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
