//! Connected components analysis
//!
//! This analysis determines the connected components of the circuit, in order to detect
//! disconnected subgraphs.

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::Circuit,
    error::{Error, Result},
    gate::Gate,
    handles::NodeId,
};

/// Connected components of the circuit.
pub(super) struct ConnectedComponents {
    /// Each node is assigned to a connected component.
    components: Vec<usize>,
    /// Number of disconnected subgraphs.
    count: usize,
}

impl ConnectedComponents {
    /// Returns the number of disconnected subgraphs.
    pub(super) fn get_count(&self) -> usize {
        self.count
    }

    /// Returns the connected component of a node.
    pub(super) fn get_component(&self, node: NodeId) -> usize {
        self.components[node.id()]
    }
}

impl Analysis for ConnectedComponents {
    type Output = Self;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let node_count = circuit.node_count();

        // Step 1. Initialize component vectors.
        let mut parent: Vec<usize> = (0..node_count).collect();

        // Step 2. Find algorithm: find the root of a node (with path halving).
        fn find(parent: &mut [usize], x: usize) -> usize {
            // Find root.
            let mut root = x;
            while parent[root] != root {
                root = parent[root];
            }

            // Make the path flat.
            let mut current = x;
            while current != root {
                let next = parent[current];
                parent[current] = root;
                current = next;
            }

            root
        }

        // Step 3. Union algorithm: merge components containing nodes x and y.
        fn union(parent: &mut [usize], x: usize, y: usize) {
            let x_root = find(parent, x);
            let y_root = find(parent, y);
            if x_root != y_root {
                parent[y_root] = x_root;
            }
        }

        // Step 4. Union all edges.
        // Gates: union with sources and destinations.
        for gate_id in circuit.get_gate_ids() {
            let gate = circuit.get_gate(gate_id)?;
            for source in gate.get_sources() {
                union(&mut parent, source.id(), gate_id.id());
            }
            for (_, dest) in gate.get_destinations() {
                union(&mut parent, dest.id(), gate_id.id());
            }
        }

        // Clones: union with source and destinations.
        for clone_id in circuit.get_clone_ids() {
            let clone_node = circuit.get_clone(clone_id)?;
            if let Some(source) = clone_node.get_source() {
                union(&mut parent, source.id(), clone_id.id());
            }
            for (_, dest) in clone_node.get_destinations() {
                union(&mut parent, dest.id(), clone_id.id());
            }
        }

        // Drops: union with source.
        for drop_id in circuit.get_drop_ids() {
            let drop_node = circuit.get_drop(drop_id)?;
            if let Some(source) = drop_node.get_source() {
                union(&mut parent, source.id(), drop_id.id());
            }
        }

        // Step 5. Normalize components.
        for i in 0..node_count {
            parent[i] = find(&mut parent, i);
        }

        // Step 6. Remap into a 0 start indexed component identifiers.
        let mut root_to_id: Vec<Option<usize>> = Vec::with_capacity(node_count);
        root_to_id.resize(node_count, None);
        let mut next_id = 0;
        for i in 0..node_count {
            let root = parent[i];
            if root_to_id[root].is_none() {
                root_to_id[root] = Some(next_id);
                next_id += 1;
            }
            parent[i] = root_to_id[root].ok_or(Error::ConnectedComponentFail)?;
        }

        Ok(ConnectedComponents {
            components: parent,
            count: next_id,
        })
    }
}
