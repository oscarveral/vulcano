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
    type Output = Vec<Operation>;

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

        // Convert indices to Operation handles.
        Ok(topo.into_iter().map(Operation::new).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        gate::Gate,
        graph::{
            analyzer::{Analyzer, analyses::topological::TopologicalOrder},
            builder::Builder,
            circuit::Circuit,
        },
        handles::{Operation, Source},
    };

    enum TestGate {
        Negate,
        Addition,
        Rotate,
    }

    impl Gate for TestGate {
        fn arity(&self) -> usize {
            match self {
                TestGate::Negate => 1,
                TestGate::Addition => 2,
                TestGate::Rotate => 2,
            }
        }

        fn name(&self) -> &str {
            match self {
                TestGate::Negate => "Negate",
                TestGate::Addition => "Addition",
                TestGate::Rotate => "Rotate",
            }
        }
    }

    #[test]
    fn simple_circuit() {
        // Layout: input -> negate -> output

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let gate = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, gate).unwrap();
        builder.connect_gate_to_output(gate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 1);
        assert_eq!(topo[0], gate);
    }

    #[test]
    fn linear_circuit() {
        // Layout: input -> negate1 -> negate2 -> negate3 -> output

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let negate3 = builder.add_gate(TestGate::Negate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_gate(negate2, negate3).unwrap();
        builder.connect_gate_to_output(negate3, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 3);
        let pos1 = topo.iter().position(|&x| x == negate1).unwrap();
        let pos2 = topo.iter().position(|&x| x == negate2).unwrap();
        let pos3 = topo.iter().position(|&x| x == negate3).unwrap();
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn complex_circuit() {
        // Layout:
        //
        // input1 -> negate1 -> \
        //                        addition -> output
        // input2 -> negate2 -> /

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition = builder.add_gate(TestGate::Addition);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_input_to_gate(input2, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition).unwrap();
        builder.connect_gate_to_gate(negate2, addition).unwrap();
        builder.connect_gate_to_output(addition, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 3);
        let pos1 = topo.iter().position(|&x| x == negate1).unwrap();
        let pos2 = topo.iter().position(|&x| x == negate2).unwrap();
        let pos_add = topo.iter().position(|&x| x == addition).unwrap();
        assert!(pos1 < pos_add);
        assert!(pos2 < pos_add);
    }

    #[test]
    fn diamond_pattern() {
        // Layout:
        //
        // input -> negate1 -> \
        //       \-> negate2 -> / -> addition -> output

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
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 3);
        let pos1 = topo.iter().position(|&x| x == negate1).unwrap();
        let pos2 = topo.iter().position(|&x| x == negate2).unwrap();
        let pos_add = topo.iter().position(|&x| x == addition).unwrap();
        assert!(pos1 < pos_add);
        assert!(pos2 < pos_add);
    }

    #[test]
    fn multiple_outputs() {
        // Layout:
        //
        // input -> negate1 -> output1
        //                  \-> negate2 -> output2

        let mut builder: Builder<TestGate> = Builder::new();
        let input = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let output1 = builder.add_output();
        let output2 = builder.add_output();

        builder.connect_input_to_gate(input, negate1).unwrap();
        builder.connect_gate_to_output(negate1, output1).unwrap();
        builder.connect_gate_to_gate(negate1, negate2).unwrap();
        builder.connect_gate_to_output(negate2, output2).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 2);
        let pos1 = topo.iter().position(|&x| x == negate1).unwrap();
        let pos2 = topo.iter().position(|&x| x == negate2).unwrap();
        assert!(pos1 < pos2);
    }

    #[test]
    fn cycle_detection_indirect() {
        // This test creates a circuit with a cycle by manually building the circuit
        // and bypassing validation.

        let gate1 = TestGate::Negate;
        let gate2 = TestGate::Negate;
        let gate3 = TestGate::Negate;

        let circuit = Circuit {
            gate_entries: vec![
                (gate1, vec![Source::Gate(Operation::new(2))]), // gate1 depends on gate3
                (gate2, vec![Source::Gate(Operation::new(0))]), // gate2 depends on gate1
                (gate3, vec![Source::Gate(Operation::new(1))]), // gate3 depends on gate2 -> cycle!
            ],
            input_count: 0,
            connected_outputs: vec![Operation::new(2)],
        };

        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::CycleDetected(ops) => {
                assert_eq!(ops.len(), 3);
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn cycle_detection_self_loop() {
        // Create a gate that references its own output.

        let gate = TestGate::Negate;

        // gate uses its own output
        let circuit = Circuit {
            gate_entries: vec![(gate, vec![Source::Gate(Operation::new(0))])],
            input_count: 0,
            connected_outputs: vec![Operation::new(0)],
        };

        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::CycleDetected(ops) => {
                assert_eq!(ops.len(), 1);
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn complex_valid_dag() {
        // More complex DAG:
        //     input1 -> negate1 -> \
        //                           addition1 -> rotate -> output
        //     input2 -> negate2 -> /              /
        //     input3 ---------------------------> /

        let mut builder: Builder<TestGate> = Builder::new();
        let input1 = builder.add_input();
        let input2 = builder.add_input();
        let input3 = builder.add_input();
        let negate1 = builder.add_gate(TestGate::Negate);
        let negate2 = builder.add_gate(TestGate::Negate);
        let addition1 = builder.add_gate(TestGate::Addition);
        let rotate = builder.add_gate(TestGate::Rotate);
        let output = builder.add_output();

        builder.connect_input_to_gate(input1, negate1).unwrap();
        builder.connect_input_to_gate(input2, negate2).unwrap();
        builder.connect_gate_to_gate(negate1, addition1).unwrap();
        builder.connect_gate_to_gate(negate2, addition1).unwrap();
        builder.connect_gate_to_gate(addition1, rotate).unwrap();
        builder.connect_input_to_gate(input3, rotate).unwrap();
        builder.connect_gate_to_output(rotate, output).unwrap();

        let circuit = builder.finalize().unwrap();
        let mut analyzer = Analyzer::new();
        let result = analyzer.get::<TopologicalOrder>(&circuit);

        assert!(result.is_ok());
        let topo = result.unwrap();
        assert_eq!(topo.len(), 4);

        let pos_neg1 = topo.iter().position(|&x| x == negate1).unwrap();
        let pos_neg2 = topo.iter().position(|&x| x == negate2).unwrap();
        let pos_add = topo.iter().position(|&x| x == addition1).unwrap();
        let pos_rot = topo.iter().position(|&x| x == rotate).unwrap();

        assert!(pos_neg1 < pos_add);
        assert!(pos_neg2 < pos_add);
        assert!(pos_add < pos_rot);
    }
}
