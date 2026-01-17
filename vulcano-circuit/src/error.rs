//! Error types used throughout the crate.
//!
//! These errors are returned when callers attempt invalid operations.

use std::any::TypeId;

use crate::circuit::{
    operations::{
        Consumer, Operation, PortId, clone::CloneId, drop::DropId, gate::GateId, input::InputId,
        output::OutputId,
    },
    subcircuit::CircuitId,
    value::ValueId,
};

/// Errors that can occur in this crate.
#[derive(Debug)]
pub enum Error {
    /// Failed to create a circuit.
    FailedToCreateCircuit(CircuitId),
    /// Failed to create a value.
    FailedToCreateValue(ValueId),
    /// Failed to create an input.
    FailedToCreateInput(InputId),
    /// Failed to create an output.
    FailedToCreateOutput(OutputId),
    /// Failed to create a clone.
    FailedToCreateClone(CloneId),
    /// Failed to create a drop.
    FailedToCreateDrop(DropId),
    /// Failed to create a gate.
    FailedToCreateGate(GateId),
    /// Non existing value.
    ValueNotFound(ValueId),
    /// Circuit id mismatch.
    CircuitIdMismatch(ValueId, CircuitId),
    /// Invalid input count.
    InvalidInputCount(usize, usize),
    /// Invalid output count.
    InvalidOutputCount(usize, usize),
    /// Invalid clone quantity.
    InvalidCloneQuantity,
    /// Tried to convert an invalid operation.
    InvalidOperationConversion(Operation),
    /// Analysis cache type mismatch.
    AnalysisCacheTypeMismatch(TypeId),
    /// Gate not found.
    GateNotFound(GateId),
    /// Clone not found.
    CloneNotFound(CloneId),
    /// Drop not found.
    DropNotFound(DropId),
    /// Input not found.
    InputNotFound(InputId),
    /// Output not found.
    OutputNotFound(OutputId),
    /// Cycle detected during analysis.
    CycleDetected(Vec<Operation>),
    /// Operation not found in schedule.
    OperationNotScheduled(Operation),
    /// Subcircuit analysis result missing.
    SubcircuitAnalysisMissing(CircuitId),
    /// Destination not found when trying to rewire.
    DestinationNotFound(ValueId, Consumer, PortId),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FailedToCreateCircuit(id) => {
                write!(f, "failed to create circuit: {:?}", id)
            }
            Error::FailedToCreateValue(id) => {
                write!(f, "failed to create value: {:?}", id)
            }
            Error::FailedToCreateInput(id) => {
                write!(f, "failed to create input: {:?}", id)
            }
            Error::FailedToCreateOutput(id) => {
                write!(f, "failed to create output: {:?}", id)
            }
            Error::FailedToCreateClone(id) => {
                write!(f, "failed to create clone: {:?}", id)
            }
            Error::FailedToCreateDrop(id) => {
                write!(f, "failed to create drop: {:?}", id)
            }
            Error::FailedToCreateGate(id) => {
                write!(f, "failed to create gate: {:?}", id)
            }
            Error::ValueNotFound(id) => {
                write!(f, "value not found: {:?}", id)
            }
            Error::CircuitIdMismatch(id, circuit) => {
                write!(f, "circuit id mismatch: {:?} != {:?}", id, circuit)
            }
            Error::InvalidInputCount(gate_inputs, inputs) => {
                write!(f, "invalid input count: {:?} != {:?}", gate_inputs, inputs)
            }
            Error::InvalidOutputCount(gate_outputs, outputs) => {
                write!(
                    f,
                    "invalid output count: {:?} != {:?}",
                    gate_outputs, outputs
                )
            }
            Error::InvalidCloneQuantity => {
                write!(f, "invalid clone quantity: must be greater than 0")
            }
            Error::InvalidOperationConversion(op) => {
                write!(f, "invalid operation conversion: {:?}", op)
            }
            Error::AnalysisCacheTypeMismatch(id) => {
                write!(f, "analysis cache type mismatch: {:?}", id)
            }
            Error::GateNotFound(id) => {
                write!(f, "gate not found: {:?}", id)
            }
            Error::CloneNotFound(id) => {
                write!(f, "clone not found: {:?}", id)
            }
            Error::DropNotFound(id) => {
                write!(f, "drop not found: {:?}", id)
            }
            Error::InputNotFound(id) => {
                write!(f, "input not found: {:?}", id)
            }
            Error::OutputNotFound(id) => {
                write!(f, "output not found: {:?}", id)
            }
            Error::CycleDetected(ops) => {
                write!(f, "cycle detected involving operations: {:?}", ops)
            }
            Error::OperationNotScheduled(op) => {
                write!(f, "operation not scheduled: {:?}", op)
            }
            Error::SubcircuitAnalysisMissing(id) => {
                write!(f, "subcircuit analysis result missing: {:?}", id)
            }
            Error::DestinationNotFound(value, consumer, port) => {
                write!(
                    f,
                    "destination not found for value {:?} at consumer {:?} port {:?}",
                    value, consumer, port
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
