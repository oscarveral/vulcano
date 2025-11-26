//! Sub-circuit connectivity analysis.
//!
//! This module identifies disjoint sub-circuits within a circuit. Two operations
//! or inputs belong to the same sub-circuit if they are connected through data
//! dependencies. This includes:
//! - Operations connected through gate-to-gate dependencies
//! - Operations and inputs connected when an operation uses an input
//! - Multiple operations sharing the same input (they belong to the same sub-circuit)
//!
//! This analysis is useful for wire allocation optimization, as values in different
//! sub-circuits cannot interfere with each other.

use std::collections::{HashMap, hash_map::Entry};

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        circuit::Circuit,
    },
    handles::{Input, Operation, Source},
};

/// Analysis that computes sub-circuit connectivity information.
pub struct SubCircuitAnalysis;

/// Sub-circuit connectivity information for a circuit.
#[derive(Debug, Clone)]
pub struct SubCircuitInfo {
    /// Sub-circuit ID for each operation.
    operation_subcircuits: HashMap<Operation, usize>,
    /// Sub-circuit ID for each input.
    input_subcircuits: HashMap<Input, usize>,
    /// Total number of disjoint sub-circuits.
    pub subcircuit_count: usize,
}

impl SubCircuitInfo {
    /// Get the sub-circuit ID for an operation.
    pub fn operation_subcircuit(&self, op: &Operation) -> Result<usize> {
        self.operation_subcircuits
            .get(op)
            .copied()
            .ok_or(Error::SubCircuitOperationNotFound(*op))
    }

    /// Get the sub-circuit ID for an input.
    pub fn input_subcircuit(&self, input: &Input) -> Result<usize> {
        self.input_subcircuits
            .get(input)
            .copied()
            .ok_or(Error::SubCircuitInputNotFound(*input))
    }

    /// Check if two operations belong to the same sub-circuit.
    pub fn same_subcircuit_ops(&self, op1: &Operation, op2: &Operation) -> Result<bool> {
        let id1 = self.operation_subcircuit(op1)?;
        let id2 = self.operation_subcircuit(op2)?;
        Ok(id1 == id2)
    }

    /// Check if an operation and input belong to the same sub-circuit.
    pub fn same_subcircuit_op_input(&self, op: &Operation, input: &Input) -> Result<bool> {
        let id1 = self.operation_subcircuit(op)?;
        let id2 = self.input_subcircuit(input)?;
        Ok(id1 == id2)
    }

    /// Check if two inputs belong to the same sub-circuit.
    pub fn same_subcircuit_inputs(&self, input1: &Input, input2: &Input) -> Result<bool> {
        let id1 = self.input_subcircuit(input1)?;
        let id2 = self.input_subcircuit(input2)?;
        Ok(id1 == id2)
    }
}

impl Analysis for SubCircuitAnalysis {
    type Output = SubCircuitInfo;

    fn run<T: Gate>(circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        // Use Union-Find to group connected components.
        let mut uf = UnionFind::new();

        // Add all operations and inputs to the union-find structure.
        for op in circuit.operations() {
            uf.make_set(Node::Operation(op));
        }

        for input in circuit.inputs() {
            uf.make_set(Node::Input(input));
        }

        // Union operations with their dependencies.
        for op in circuit.operations() {
            let (_, sources) = &circuit.gate_entries[op.id()];

            for source in sources {
                match source {
                    Source::Input(input) => {
                        // Union operation with input.
                        uf.union(Node::Operation(op), Node::Input(*input));
                    }
                    Source::Gate(producer_op) => {
                        // Union operation with producer operation.
                        uf.union(Node::Operation(op), Node::Operation(*producer_op));
                    }
                }
            }
        }

        // Assign sub-circuit IDs based on connected components.
        let mut subcircuit_map: HashMap<Node, usize> = HashMap::new();
        let mut next_id = 0;

        // Process all nodes and assign IDs.
        for op in circuit.operations() {
            let node = Node::Operation(op);
            let root = uf.find(node);

            if let Entry::Vacant(e) = subcircuit_map.entry(root) {
                e.insert(next_id);
                next_id += 1;
            }
        }

        for input in circuit.inputs() {
            let node = Node::Input(input);
            let root = uf.find(node);

            if let Entry::Vacant(e) = subcircuit_map.entry(root) {
                e.insert(next_id);
                next_id += 1;
            }
        }

        // Build final mappings.
        let mut operation_subcircuits = HashMap::new();
        let mut input_subcircuits = HashMap::new();

        for op in circuit.operations() {
            let root = uf.find(Node::Operation(op));
            if let Some(&id) = subcircuit_map.get(&root) {
                operation_subcircuits.insert(op, id);
            }
        }

        for input in circuit.inputs() {
            let root = uf.find(Node::Input(input));
            if let Some(&id) = subcircuit_map.get(&root) {
                input_subcircuits.insert(input, id);
            }
        }

        Ok(SubCircuitInfo {
            operation_subcircuits,
            input_subcircuits,
            subcircuit_count: next_id,
        })
    }
}

/// Node in the union-find structure (either an operation or input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Operation(Operation),
    Input(Input),
}

/// Union-Find (Disjoint Set) data structure for tracking connectivity.
struct UnionFind {
    parent: HashMap<Node, Node>,
    rank: HashMap<Node, usize>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn make_set(&mut self, node: Node) {
        self.parent.insert(node, node);
        self.rank.insert(node, 0);
    }

    fn find(&mut self, node: Node) -> Node {
        let parent = *self.parent.get(&node).unwrap_or(&node);

        if parent != node {
            // Path compression.
            let root = self.find(parent);
            self.parent.insert(node, root);
            root
        } else {
            node
        }
    }

    fn union(&mut self, node1: Node, node2: Node) {
        let root1 = self.find(node1);
        let root2 = self.find(node2);

        if root1 == root2 {
            return;
        }

        // Union by rank.
        let rank1 = *self.rank.get(&root1).unwrap_or(&0);
        let rank2 = *self.rank.get(&root2).unwrap_or(&0);

        if rank1 < rank2 {
            self.parent.insert(root1, root2);
        } else if rank1 > rank2 {
            self.parent.insert(root2, root1);
        } else {
            self.parent.insert(root2, root1);
            self.rank.insert(root1, rank1 + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::subcircuit::SubCircuitAnalysis},
            builder::Builder,
        },
    };

    enum TestGate {
        Negate,
        Addition,
    }

    impl Gate for TestGate {
        fn arity(&self) -> usize {
            match self {
                TestGate::Negate => 1,
                TestGate::Addition => 2,
            }
        }

        fn name(&self) -> &str {
            match self {
                TestGate::Negate => "Negate",
                TestGate::Addition => "Addition",
            }
        }
    }

    #[test]
    fn single_connected_circuit() {
        // Circuit: input -> negate -> output
        // All should be in the same sub-circuit.

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let info = analyzer.get::<SubCircuitAnalysis>(&circuit).unwrap();

        assert_eq!(info.subcircuit_count, 1);
        assert_eq!(info.input_subcircuit(&input).unwrap(), 0);
        assert_eq!(info.operation_subcircuit(&gate).unwrap(), 0);
    }

    #[test]
    fn two_disjoint_circuits() {
        // Circuit:
        // input1 -> negate1 -> output1
        // input2 -> negate2 -> output2
        // Two completely independent circuits.

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        let output2 = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_gate_to_output(negate1, output1).unwrap();
        builder.connect_input_to_gate(input2, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let info = analyzer.get::<SubCircuitAnalysis>(&circuit).unwrap();

        assert_eq!(info.subcircuit_count, 2);

        let subcircuit1 = info.operation_subcircuit(&negate1).unwrap();
        let subcircuit2 = info.operation_subcircuit(&negate2).unwrap();

        assert_ne!(subcircuit1, subcircuit2);
        assert_eq!(info.input_subcircuit(&input1).unwrap(), subcircuit1);
        assert_eq!(info.input_subcircuit(&input2).unwrap(), subcircuit2);
    }

    #[test]
    fn shared_input_same_subcircuit() {
        // Circuit:
        // input -> negate1 -> output1
        //       -> negate2 -> output2
        // Both operations share the same input, so they're in the same sub-circuit.

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        let output2 = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_output(negate1, output1).unwrap();
        builder.connect_input_to_gate(input, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let info = analyzer.get::<SubCircuitAnalysis>(&circuit).unwrap();

        assert_eq!(info.subcircuit_count, 1);
        assert!(info.same_subcircuit_ops(&negate1, &negate2).unwrap());
        assert!(info.same_subcircuit_op_input(&negate1, &input).unwrap());
        assert!(info.same_subcircuit_op_input(&negate2, &input).unwrap());
    }

    #[test]
    fn diamond_pattern() {
        // Circuit:
        // input -> negate1 -> \
        //       -> negate2 -> / -> addition -> output
        // All connected, should be one sub-circuit.

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_input_to_gate(input, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition).unwrap();
        builder.connect_gate_to_gate(negate2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let info = analyzer.get::<SubCircuitAnalysis>(&circuit).unwrap();

        assert_eq!(info.subcircuit_count, 1);
        assert!(info.same_subcircuit_ops(&negate1, &negate2).unwrap());
        assert!(info.same_subcircuit_ops(&negate1, &addition).unwrap());
    }
}
