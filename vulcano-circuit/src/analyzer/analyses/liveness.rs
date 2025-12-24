//! Liveness analysis
//!
//! This module contains the liveness analysis algorithm used to analyze the circuit.
//! Computes the liveness of each value produced in the circuit.

use crate::{
    analyzer::{Analysis, Analyzer, analyses::topological::TopologicalOrder},
    circuit::{Circuit, Node},
    error::{Error, Result},
    gate::Gate,
    handles::{NodeId, PortId},
};

/// Liveness of a value.
///
/// A value is live if it is used by a gate or output.
struct Liveness {
    /// Producer node ID.
    producer: NodeId,
    /// Output port ID.
    port: Option<PortId>,
    /// Birth index in the topological order.
    birth: usize,
    /// Death index in the topological order.
    death: usize,
}

/// Liveness analysis.
pub(super) struct LivenessAnalysis {
    /// Vector of liveness of all values in the circuit.
    liveness: Vec<Liveness>,
}

impl LivenessAnalysis {
    /// Returns an iterator over all liveness information.
    pub(super) fn liveness(&self) -> impl Iterator<Item = &Liveness> {
        self.liveness.iter()
    }

    /// Returns the liveness information for a specific value.
    pub(super) fn get_liveness(&self, producer: NodeId, port: Option<PortId>) -> Option<&Liveness> {
        self.liveness
            .iter()
            .find(|l| l.producer == producer && l.port == port)
    }
}

impl Analysis for LivenessAnalysis {
    type Output = Self;

    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        // Step 1. Get topological order of all nodes.
        let topological_order = analyzer.get::<TopologicalOrder>(circuit)?;
        let order = topological_order.get_order();

        // Step 2. Create a mapping from node ID to index in the topological order.
        let mut node_to_topo = Vec::with_capacity(order.len());
        for (index, _) in order.iter().enumerate() {
            node_to_topo.push(index);
        }

        // Step 3. Compute liveness for each value.
        let mut liveness = Vec::with_capacity(circuit.gate_count() + circuit.input_count());
        for node_id in order {
            let node = circuit.get_node(*node_id)?;
            match node {
                Node::Input { node: input } => {
                    let birth = node_to_topo[node_id.id()];
                    let mut death = node_to_topo[node_id.id()];

                    for consumer in input.get_destinations() {
                        let candidate_death = node_to_topo[consumer.id()];
                        if candidate_death >= death {
                            death = candidate_death;
                        } else {
                            return Err(Error::InconsistentOrder);
                        }
                    }

                    liveness.push(Liveness {
                        producer: *node_id,
                        port: None,
                        birth,
                        death,
                    });
                }
                Node::Gate { node: gate_node } => {
                    let birth = node_to_topo[node_id.id()];

                    // Need one liveness entry per output port.
                    let gate_outputs = gate_node.get_gate().output_count().get();
                    let mut outputs_livenesses = Vec::with_capacity(gate_outputs);
                    for i in 0..gate_outputs {
                        outputs_livenesses.push(Liveness {
                            producer: *node_id,
                            port: Some(PortId::new(i)),
                            birth,
                            death: birth,
                        });
                    }

                    for (port, consumer) in gate_node.get_destinations() {
                        let candidate_death = node_to_topo[consumer.id()];
                        if candidate_death >= outputs_livenesses[port.id()].death {
                            outputs_livenesses[port.id()].death = candidate_death;
                        } else {
                            return Err(Error::InconsistentOrder);
                        }
                    }

                    liveness.extend(outputs_livenesses);
                }
                _ => {
                    // Outputs nodes do not produce any value.
                }
            }
        }

        Ok(LivenessAnalysis { liveness })
    }
}
