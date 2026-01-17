//! Connected Components Analysis
//!
//! Identifies disjoint subgraphs within each subcircuit.
//! Two operations are in the same component if they're connected
//! through value dependencies (producer-consumer relationships).

use std::collections::{HashMap, hash_map::Entry};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{
        Circuit,
        operations::Operation,
        subcircuit::{CircuitId, Subcircuit},
    },
    error::Result,
    gate::Gate,
};

/// Union-Find data structure for connected components.
struct UnionFind {
    parent: HashMap<Operation, Operation>,
    rank: HashMap<Operation, usize>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn make_set(&mut self, op: Operation) {
        if let Entry::Vacant(e) = self.parent.entry(op) {
            e.insert(op);
            self.rank.insert(op, 0);
        }
    }

    fn find(&mut self, op: Operation) -> Operation {
        let parent = *self.parent.get(&op).unwrap_or(&op);
        if parent != op {
            let root = self.find(parent);
            self.parent.insert(op, root);
            root
        } else {
            op
        }
    }

    fn union(&mut self, a: Operation, b: Operation) {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return;
        }

        let rank_a = *self.rank.get(&root_a).unwrap_or(&0);
        let rank_b = *self.rank.get(&root_b).unwrap_or(&0);

        if rank_a < rank_b {
            self.parent.insert(root_a, root_b);
        } else if rank_a > rank_b {
            self.parent.insert(root_b, root_a);
        } else {
            self.parent.insert(root_b, root_a);
            self.rank.insert(root_a, rank_a + 1);
        }
    }
}

/// Per-subcircuit connected components result.
pub struct SubcircuitComponents {
    /// Maps each operation to its component ID.
    component_id: HashMap<Operation, usize>,
    /// Number of components.
    component_count: usize,
    /// Size of each component (number of operations).
    component_sizes: Vec<usize>,
}

impl SubcircuitComponents {
    /// Get the component ID for an operation.
    pub fn component_of(&self, op: Operation) -> Option<usize> {
        self.component_id.get(&op).copied()
    }

    /// Get the number of connected components.
    pub fn count(&self) -> usize {
        self.component_count
    }

    /// Check if subcircuit has disjoint parts.
    pub fn is_disjoint(&self) -> bool {
        self.component_count > 1
    }

    /// Get the size of a component.
    pub fn component_size(&self, component: usize) -> usize {
        self.component_sizes.get(component).copied().unwrap_or(0)
    }

    /// Get the ID of the largest component.
    pub fn largest_component(&self) -> usize {
        self.component_sizes
            .iter()
            .enumerate()
            .max_by_key(|(_, size)| *size)
            .map(|(id, _)| id)
            .unwrap_or(0)
    }

    /// Get all operations in a specific component.
    pub fn operations_in(&self, component: usize) -> impl Iterator<Item = Operation> + '_ {
        self.component_id
            .iter()
            .filter(move |(_, c)| **c == component)
            .map(|(&op, _)| op)
    }

    /// Iterate over all component IDs (0..count).
    pub fn component_ids(&self) -> impl Iterator<Item = usize> {
        0..self.component_count
    }
}

/// Result of connected components analysis.
pub struct ConnectedComponents {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitComponents>,
}

impl ConnectedComponents {
    /// Get the components for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitComponents> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitComponents)> {
        self.results.iter().map(|(&id, comp)| (id, comp))
    }

    /// Check if any subcircuit has disjoint parts.
    pub fn has_disjoint_subcircuits(&self) -> bool {
        self.results.values().any(|c| c.is_disjoint())
    }
}

impl<G: Gate> Analysis<G> for ConnectedComponents {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let components = compute_connected_components(subcircuit);
            results.insert(subcircuit.id(), components);
        }

        Ok(ConnectedComponents { results })
    }
}

/// Compute connected components for a single subcircuit.
fn compute_connected_components<G: Gate>(subcircuit: &Subcircuit<G>) -> SubcircuitComponents {
    let mut uf = UnionFind::new();

    // Initialize all operations as their own set.
    for op in subcircuit.all_operations() {
        uf.make_set(op);
    }

    // Union producer with all consumers for each value.
    for (_, value) in subcircuit.all_values() {
        let producer: Operation = value.get_product().get_producer().into();

        for destination in value.get_destinations() {
            let consumer: Operation = destination.get_consumer().into();
            uf.union(producer, consumer);
        }
    }

    // Flatten and assign component IDs.
    let mut root_to_component: HashMap<Operation, usize> = HashMap::new();
    let mut component_id: HashMap<Operation, usize> = HashMap::new();
    let mut component_sizes: Vec<usize> = Vec::new();
    let mut next_component = 0;

    for op in subcircuit.all_operations() {
        let root = uf.find(op);
        let comp = *root_to_component.entry(root).or_insert_with(|| {
            let id = next_component;
            next_component += 1;
            component_sizes.push(0);
            id
        });
        component_id.insert(op, comp);
        component_sizes[comp] += 1;
    }

    SubcircuitComponents {
        component_id,
        component_count: next_component,
        component_sizes,
    }
}
