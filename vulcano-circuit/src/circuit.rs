//! Circuit representation.
//!
//! This module provides functionality to build circuits incrementally.

use crate::{
    error::{Error, Result},
    gate::Gate,
    handles::{Destination, GateId, InputId, NodeId, OutputId, PortId, Source},
};

/// An edge in the circuit.
///
/// An edge specifies a connection with other node.
pub(super) struct Edge {
    /// The node id of the other node.
    node: NodeId,
    /// The specific port of the connection on the other node.
    port: PortId,
}

/// A gate node.
pub(super) struct GateInternal<T: Gate> {
    /// The gate that this node represents.
    gate: T,
    /// The sources of this node.
    /// Each input port of the gate should have a source.
    sources: Vec<Option<Edge>>,
    /// The destinations of this node.
    /// Association of output port on this gate with an edge.
    destinations: Vec<(PortId, Edge)>,
}

impl<T: Gate> GateInternal<T> {
    /// Get the gate instance.
    pub(super) fn get_gate(&self) -> &T {
        &self.gate
    }

    /// Get the source nodes of a gate.
    pub(super) fn get_sources(&self) -> impl Iterator<Item = NodeId> {
        self.sources
            .iter()
            .filter_map(|edge| edge.as_ref().map(|edge| edge.node))
    }

    /// Get the source nodes with their output ports.
    /// Returns (this_gate_input_port, source_node, source_output_port).
    pub(super) fn get_sources_with_ports(&self) -> impl Iterator<Item = (usize, NodeId, PortId)> {
        self.sources
            .iter()
            .enumerate()
            .filter_map(|(idx, edge)| edge.as_ref().map(|edge| (idx, edge.node, edge.port)))
    }

    /// Get the destination nodes of a gate.
    pub(super) fn get_destinations(&self) -> impl Iterator<Item = (PortId, NodeId)> {
        self.destinations
            .iter()
            .map(|(port, edge)| (*port, edge.node))
    }
}

/// An input node.
pub(super) struct InputInternal {
    /// The destinations of this node.
    /// Each edge stores (consumer_node, consumer_input_port).
    destinations: Vec<Edge>,
}

impl InputInternal {
    /// Get the destination nodes of an input.
    pub(super) fn get_destinations(&self) -> impl Iterator<Item = NodeId> {
        self.destinations.iter().map(|edge| edge.node)
    }

    /// Get the destination nodes with their input ports.
    pub(super) fn get_destinations_with_ports(&self) -> impl Iterator<Item = (NodeId, PortId)> {
        self.destinations.iter().map(|edge| (edge.node, edge.port))
    }
}

/// An output node.
pub(super) struct OutputInternal {
    /// The source of this node.
    /// The edge stores (producer_node, producer_output_port).
    source: Option<Edge>,
}

impl OutputInternal {
    /// Get the source node of an output.
    pub(super) fn get_source(&self) -> Option<NodeId> {
        self.source.as_ref().map(|edge| edge.node)
    }

    /// Get the source node with its output port.
    pub(super) fn get_source_with_port(&self) -> Option<(NodeId, PortId)> {
        self.source.as_ref().map(|edge| (edge.node, edge.port))
    }
}

/// A node in the circuit.
pub(super) enum Node<T: Gate> {
    /// A gate node.
    Gate { node: GateInternal<T> },
    /// An input node.
    Input { node: InputInternal },
    /// An output node.
    Output { node: OutputInternal },
}

/// A circuit containing a set of nodes with their connections.
pub(super) struct Circuit<T: Gate> {
    /// The nodes of the circuit.
    nodes: Vec<Node<T>>,
    /// The inputs of the circuit.
    /// Each input must have, at least, one edge into a node.
    inputs: Vec<NodeId>,
    /// The outputs of the circuit.
    /// Each output must have an edge out of a node.
    outputs: Vec<NodeId>,
    /// The gates of the circuit.
    /// Indices of the nodes that are gates.
    gates: Vec<NodeId>,
}

impl<T: Gate> Circuit<T> {
    /// Create a new empty circuit.
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            gates: Vec::new(),
        }
    }

    /// Add a new gate to the circuit.
    fn add_gate(&mut self, gate: T) -> GateId {
        let external_id = GateId::new(self.gates.len());
        let internal_id = NodeId::new(self.nodes.len());
        let input_count = gate.input_count().get();
        self.gates.push(internal_id);
        self.nodes.push(Node::Gate {
            node: GateInternal {
                gate,
                sources: (0..input_count).map(|_| None).collect(),
                destinations: Vec::new(),
            },
        });
        external_id
    }

    /// Add multiple gates to the circuit.
    fn add_gates(
        &mut self,
        gates: impl IntoIterator<Item = T>,
    ) -> impl IntoIterator<Item = GateId> {
        gates
            .into_iter()
            .map(|gate| self.add_gate(gate))
            .collect::<Vec<_>>()
    }

    /// Add a new input to the circuit.
    fn add_input(&mut self) -> InputId {
        let external_id = InputId::new(self.inputs.len());
        let internal_id = NodeId::new(self.nodes.len());
        self.inputs.push(internal_id);
        self.nodes.push(Node::Input {
            node: InputInternal {
                destinations: Vec::new(),
            },
        });
        external_id
    }

    /// Add multiple inputs to the circuit.
    fn add_inputs(&mut self, count: usize) -> impl IntoIterator<Item = InputId> {
        (0..count).map(|_| self.add_input()).collect::<Vec<_>>()
    }

    /// Add a new output to the circuit.
    fn add_output(&mut self) -> OutputId {
        let external_id = OutputId::new(self.outputs.len());
        let internal_id = NodeId::new(self.nodes.len());
        self.outputs.push(internal_id);
        self.nodes.push(Node::Output {
            node: OutputInternal { source: None },
        });
        external_id
    }

    /// Add multiple outputs to the circuit.
    fn add_outputs(&mut self, count: usize) -> impl IntoIterator<Item = OutputId> {
        (0..count).map(|_| self.add_output()).collect::<Vec<_>>()
    }

    /// Add a new wire to the circuit.
    fn add_wire(
        mut self,
        source: impl Into<Source>,
        destination: impl Into<Destination>,
    ) -> Result<Self> {
        let source = source.into();
        let destination = destination.into();

        // Validate source node.
        let (src_internal_index, src_port) = match source {
            Source::Gate { gate, port } => {
                let src_external_index = gate.id();
                if src_external_index >= self.gates.len() {
                    return Err(Error::GateNotFound(gate));
                }
                let src_internal_index = self.gates[src_external_index];
                if src_internal_index.id() >= self.nodes.len() {
                    return Err(Error::UnmappedGate(gate));
                }
                let src_node = &self.nodes[src_internal_index.id()];
                match src_node {
                    Node::Gate {
                        node: gate_instance,
                    } => {
                        let output_count = gate_instance.gate.output_count().get();
                        if output_count <= port.id() {
                            return Err(Error::OutputPortNotFound(gate, port));
                        }
                    }
                    _ => return Err(Error::MissmatchedGate(gate)),
                }
                (src_internal_index, port)
            }
            Source::Input(input) => {
                let src_external_index = input.id();
                if src_external_index >= self.inputs.len() {
                    return Err(Error::InputNotFound(input));
                }
                let src_internal_index = self.inputs[src_external_index];
                if src_internal_index.id() >= self.nodes.len() {
                    return Err(Error::UnmappedInput(input));
                }
                match &self.nodes[src_internal_index.id()] {
                    Node::Input { .. } => {}
                    _ => return Err(Error::MissmatchedInput(input)),
                }
                (src_internal_index, PortId::new(0))
            }
        };

        // Validate destination node.
        let (dst_internal_index, dst_port) = match destination {
            Destination::Gate { gate, port } => {
                let dst_external_index = gate.id();
                if dst_external_index >= self.gates.len() {
                    return Err(Error::GateNotFound(gate));
                }
                let dst_internal_index = self.gates[dst_external_index];
                if dst_internal_index.id() >= self.nodes.len() {
                    return Err(Error::UnmappedGate(gate));
                }
                if src_internal_index == dst_internal_index {
                    return Err(Error::SelfConnection(gate));
                }

                let dst_node = &self.nodes[dst_internal_index.id()];
                match dst_node {
                    Node::Gate {
                        node: gate_instance,
                    } => {
                        let input_count = gate_instance.gate.input_count().get();
                        if input_count <= port.id() || gate_instance.sources.len() <= port.id() {
                            return Err(Error::InputPortNotFound(gate, port));
                        }
                        if gate_instance.sources[port.id()].is_some() {
                            return Err(Error::InputAlreadyConnected(gate, port));
                        }
                    }
                    _ => return Err(Error::MissmatchedGate(gate)),
                }
                (dst_internal_index, port)
            }
            Destination::Output(output) => {
                let dst_external_index = output.id();
                if dst_external_index >= self.outputs.len() {
                    return Err(Error::OutputNotFound(output));
                }
                let dst_internal_index = self.outputs[dst_external_index];
                if dst_internal_index.id() >= self.nodes.len() {
                    return Err(Error::UnmappedOutput(output));
                }
                let dst_node = &self.nodes[dst_internal_index.id()];
                match dst_node {
                    Node::Output {
                        node: output_instance,
                    } => {
                        if output_instance.source.is_some() {
                            return Err(Error::OutputAlreadyConnected(output));
                        }
                    }
                    _ => return Err(Error::MissmatchedOutput(output)),
                }
                (dst_internal_index, PortId::new(0))
            }
        };

        // Update source node.
        let src_node = &mut self.nodes[src_internal_index.id()];
        match src_node {
            Node::Gate {
                node: gate_instance,
            } => {
                gate_instance.destinations.push((
                    src_port,
                    Edge {
                        node: dst_internal_index,
                        port: dst_port,
                    },
                ));
            }
            Node::Input {
                node: input_instance,
            } => {
                input_instance.destinations.push(Edge {
                    node: dst_internal_index,
                    port: dst_port,
                });
            }
            _ => return Err(Error::NodeMissmatched(src_internal_index)),
        }

        // Update destination node.
        let dst_node = &mut self.nodes[dst_internal_index.id()];
        match dst_node {
            Node::Gate {
                node: gate_instance,
            } => {
                gate_instance.sources[dst_port.id()] = Some(Edge {
                    node: src_internal_index,
                    port: src_port,
                });
            }
            Node::Output {
                node: output_instance,
            } => {
                output_instance.source = Some(Edge {
                    node: src_internal_index,
                    port: src_port,
                });
            }
            _ => {
                return Err(Error::NodeMissmatched(dst_internal_index));
            }
        }

        Ok(self)
    }

    /// Returns the total number of nodes.
    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of gates in the circuit.
    pub(super) fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Returns the number of inputs in the circuit.
    pub(super) fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Returns the number of outputs in the circuit.
    pub(super) fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Returns true if the node is a gate.
    pub(super) fn is_gate(&self, node: NodeId) -> Result<bool> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        Ok(matches!(&self.nodes[node.id()], Node::Gate { .. }))
    }

    /// Returns true if the node is an input.
    pub(super) fn is_input(&self, node: NodeId) -> Result<bool> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        Ok(matches!(&self.nodes[node.id()], Node::Input { .. }))
    }

    /// Returns true if the node is an output.
    pub(super) fn is_output(&self, node: NodeId) -> Result<bool> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        Ok(matches!(&self.nodes[node.id()], Node::Output { .. }))
    }

    /// Get all [`NodeId`] of the circuit gates.
    pub(super) fn get_gate_ids(&self) -> impl Iterator<Item = NodeId> {
        self.gates.iter().copied()
    }

    /// Get all [`NodeId`] of the circuit inputs.
    pub(super) fn get_input_ids(&self) -> impl Iterator<Item = NodeId> {
        self.inputs.iter().copied()
    }

    /// Get all [`NodeId`] of the circuit outputs.
    pub(super) fn get_output_ids(&self) -> impl Iterator<Item = NodeId> {
        self.outputs.iter().copied()
    }

    /// Get all nodes of the circuit.
    pub(super) fn get_node_ids(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.iter().enumerate().map(|(i, _)| NodeId::new(i))
    }

    /// Get the node at the given [`NodeId`].
    pub(super) fn get_node(&self, node: NodeId) -> Result<&Node<T>> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        Ok(&self.nodes[node.id()])
    }

    /// Get the gate at the given [`NodeId`].
    pub(super) fn get_gate(&self, node: NodeId) -> Result<&GateInternal<T>> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        match &self.nodes[node.id()] {
            Node::Gate {
                node: internal_repr,
            } => Ok(internal_repr),
            _ => Err(Error::NodeMissmatched(node)),
        }
    }

    /// Get the input at the given [`NodeId`].
    pub(super) fn get_input(&self, node: NodeId) -> Result<&InputInternal> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        match &self.nodes[node.id()] {
            Node::Input {
                node: internal_repr,
            } => Ok(internal_repr),
            _ => Err(Error::NodeMissmatched(node)),
        }
    }

    /// Get the output at the given [`NodeId`].
    pub(super) fn get_output(&self, node: NodeId) -> Result<&OutputInternal> {
        if node.id() >= self.nodes.len() {
            return Err(Error::NodeNotFound(node));
        }
        match &self.nodes[node.id()] {
            Node::Output {
                node: internal_repr,
            } => Ok(internal_repr),
            _ => Err(Error::NodeMissmatched(node)),
        }
    }
}
