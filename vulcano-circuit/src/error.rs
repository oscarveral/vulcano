//! Error types used when constructing circuits with the builder API.
//!
//! These errors are returned when callers
//! attempt invalid operations (out-of-bounds handles, exceeding gate
//! arity, self-connections, etc.).

use std::any::TypeId;

use crate::handles::{GateId, InputId, NodeId, OutputId, PortId};

/// Errors that can occur on this crate.
pub(super) enum Error {
    /// The [`GateId`] provided was not found on the circuit.
    GateNotFound(GateId),
    /// The [`GateId`] provided was present on the circuit but was not mapped to an existing node.
    UnmappedGate(GateId),
    /// The [`GateId`] provided was present on the circuit but was not mapped to a gate node.
    MissmatchedGate(GateId),
    /// The output port provided was not found on the gate corresponding to the [`GateId`].
    OutputPortNotFound(GateId, PortId),
    /// The input port provided was not found on the gate corresponding to the [`GateId`].
    InputPortNotFound(GateId, PortId),
    /// The input port provided was already connected to another node.
    InputAlreadyConnected(GateId, PortId),
    /// The [`InputId`] provided was not found on the circuit.
    InputNotFound(InputId),
    /// The [`InputId`] provided was present on the circuit but was not mapped to an existing node.
    UnmappedInput(InputId),
    /// The [`InputId`] provided was present on the circuit but was not mapped to an input node.
    MissmatchedInput(InputId),
    /// The [`OutputId`] provided was not found on the circuit.
    OutputNotFound(OutputId),
    /// The [`OutputId`] provided was present on the circuit but was not mapped to an existing node.
    UnmappedOutput(OutputId),
    /// The [`OutputId`] provided was present on the circuit but was not mapped to an output node.
    MissmatchedOutput(OutputId),
    /// The [`OutputId`] provided was already connected to a source node.
    OutputAlreadyConnected(OutputId),
    /// Tried to connect a node to itself.
    SelfConnection(GateId),
    /// Analysis cache missing entry for the requested analysis.
    AnalysisCacheInconsistentEntry(TypeId),
    /// Analysis cache type mismatch for the requested analysis.
    AnalysisCacheTypeMismatch(TypeId),
    /// The [`NodeId`] provided was not found on the circuit.
    NodeNotFound(NodeId),
    /// The [`NodeId`] provided was present on the circuit but was mapped to a node of a different type.
    NodeMissmatched(NodeId),
    /// A cycle was detected in the circuit during topological sorting.
    CycleDetected,
    /// Failed to compute connected components.
    ConnectedComponentFail,
    /// The circuit is empty.
    EmptyCircuit,
    /// Inconsistent order detected.
    InconsistentOrder,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

pub(super) type Result<T> = std::result::Result<T, Error>;
