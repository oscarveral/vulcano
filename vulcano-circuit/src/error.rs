//! Error types used throughout the crate.
//!
//! These errors are returned when callers attempt invalid operations.

use std::any::TypeId;

use crate::{
    circuit::Operation,
    handles::{CloneId, DropId, GateId, InputId, OutputId, ValueId},
};

/// Errors that can occur in this crate.
#[derive(Debug)]
pub(super) enum Error {
    /// Gate not found.
    GateNotFound(GateId),
    /// Clone not found.
    CloneNotFound(CloneId),
    /// Drop not found.
    DropNotFound(DropId),
    /// Value not found.
    ValueNotFound(ValueId),
    /// Input not found.
    InputNotFound(InputId),
    /// Output not found.
    OutputNotFound(OutputId),
    /// Wrong number of inputs provided to a gate.
    WrongInputCount { expected: usize, got: usize },
    /// Invalid input port index.
    InvalidInputIndex { idx: usize, max: usize },
    /// Invalid output port index.
    InvalidOutputIndex { idx: usize, max: usize },
    /// Type mismatch at gate input.
    TypeMismatch { gate: GateId, port: usize },
    /// Wrong number of types provided to add_inputs.
    WrongInputTypeCount { expected: usize, got: usize },

    /// Tried to convert an invalid operation.
    BadOperationConversion(Operation),

    /// Cycle detected in circuit during topological sort.
    CycleDetected(Vec<Operation>),

    /// Analysis cache missing entry.
    AnalysisCacheInconsistentEntry(TypeId),
    /// Analysis cache type mismatch.
    AnalysisCacheTypeMismatch(TypeId),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GateNotFound(id) => write!(f, "gate not found: {:?}", id),
            Error::CloneNotFound(id) => write!(f, "clone not found: {:?}", id),
            Error::DropNotFound(id) => write!(f, "drop not found: {:?}", id),
            Error::ValueNotFound(id) => write!(f, "value not found: {:?}", id),
            Error::InputNotFound(id) => write!(f, "input not found: {:?}", id),
            Error::OutputNotFound(id) => write!(f, "output not found: {:?}", id),
            Error::WrongInputCount { expected, got } => {
                write!(f, "wrong input count: expected {}, got {}", expected, got)
            }
            Error::InvalidInputIndex { idx, max } => {
                write!(f, "invalid input index: {} (max {})", idx, max)
            }
            Error::InvalidOutputIndex { idx, max } => {
                write!(f, "invalid output index: {} (max {})", idx, max)
            }
            Error::TypeMismatch { gate, port } => {
                write!(f, "type mismatch at gate {:?} port {}", gate, port)
            }
            Error::WrongInputTypeCount { expected, got } => {
                write!(
                    f,
                    "wrong input type count: expected {}, got {}",
                    expected, got
                )
            }
            Error::BadOperationConversion(op) => {
                write!(f, "bad operation conversion: {:?}", op)
            }
            Error::CycleDetected(ops) => {
                write!(f, "cycle detected involving {} operations", ops.len())
            }
            Error::AnalysisCacheInconsistentEntry(id) => {
                write!(f, "analysis cache inconsistent: {:?}", id)
            }
            Error::AnalysisCacheTypeMismatch(id) => {
                write!(f, "analysis cache type mismatch: {:?}", id)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for this crate.
pub(super) type Result<T> = std::result::Result<T, Error>;
