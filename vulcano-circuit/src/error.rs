//! Error types used when constructing circuits with the builder API.
//!
//! These errors are returned when callers
//! attempt invalid operations (out-of-bounds handles, exceeding gate
//! arity, self-connections, etc.).

use std::any::TypeId;

use crate::handles::{Input, Operation, Output};

/// Errors that can occur while constructing a circuit.
///
/// Each variant carries the relevant handle so callers can present
/// helpful diagnostics or recover (for example by creating the
/// missing handle and retrying).
#[derive(PartialEq, Eq)]
pub enum Error {
    /// The referenced gate handle doesn't exist.
    NonExistentGate(Operation),
    /// The referenced input handle doesn't exist.
    NonExistentInput(Input),
    /// The referenced output handle doesn't exist.
    NonExistentOutput(Output),
    /// The circuit has no gates.
    EmptyCircuit,
    /// An attempt was made to connect more inputs to a gate than its
    /// declared arity allows.
    InputArityOverLimit(Operation),
    /// An attempt was made to connect fewer inputs to a gate than its
    /// declared arity requires.
    InputArityUnderLimit(Operation),
    /// A gate was connected to more than one output slot.
    OutputArityOverLimit(Operation),
    /// A gate was connected to itself.
    SelfConnection(Operation),
    /// An input slot was not connected to any gate.
    UnusedInput(Input),
    /// An output slot was not connected to any gate.
    UnusedOutput(Output),
    /// The requested output slot is already occupied.
    UsedOutput(Output),
    /// A cycle was detected while ordering the circuit.
    /// Carries a list of offending [`Operation`] handles that may be
    /// involved in the cycle.
    CycleDetected(Vec<Operation>),
    /// An attempt was made to optimize a circuit that has finalized all optimizations.
    AlreadyFinalized,
    /// Internal invariant was violated while manipulating circuit data.
    /// This indicates a bug in the optimizer or an unexpected internal
    /// state; carries a short diagnostic message.
    InvariantViolation(String),
    /// The analysis cache contained a result of unexpected type.
    AnalysisCacheTypeMismatch(TypeId),
    /// The analysis cache is missing an expected entry.
    AnalysisCacheMissingEntry(TypeId),
    /// An operation was not found in liveness information.
    LivenessOperationNotFound(Operation),
    /// An input was not found in liveness information.
    LivenessInputNotFound(Input),
    /// An operation was not found in use count information.
    UseCountOperationNotFound(Operation),
    /// An input was not found in use count information.
    UseCountInputNotFound(Input),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NonExistentGate(h) => write!(f, "Gate {:?} does not exist", h),
            Error::NonExistentInput(h) => write!(f, "Input {:?} does not exist", h),
            Error::NonExistentOutput(h) => write!(f, "Output {:?} does not exist", h),
            Error::EmptyCircuit => write!(f, "Circuit is empty"),
            Error::InputArityOverLimit(gate) => {
                write!(f, "Gate {:?} has too many input connections", gate)
            }
            Error::InputArityUnderLimit(gate) => {
                write!(f, "Gate {:?} has too few input connections", gate)
            }
            Error::OutputArityOverLimit(gate) => {
                write!(f, "Gate {:?} can only have 1 output connection", gate)
            }
            Error::SelfConnection(gate) => write!(f, "Gate {:?} cannot connect to itself", gate),
            Error::UnusedInput(input) => {
                write!(f, "Input {:?} is not connected to any gate", input)
            }
            Error::UnusedOutput(output) => {
                write!(f, "Output {:?} is not connected to any gate", output)
            }
            Error::UsedOutput(output) => write!(f, "Output {:?} is already connected", output),
            Error::CycleDetected(ops) => {
                write!(f, "Cycle detected involving operations: {:?}", ops)
            }
            Error::AlreadyFinalized => write!(f, "Circuit optimizations have been finalized"),
            Error::InvariantViolation(msg) => write!(f, "Invariant violated: {}", msg),
            Error::AnalysisCacheTypeMismatch(type_id) => {
                write!(f, "Analysis cache type mismatch for TypeId {:?}", type_id)
            }
            Error::AnalysisCacheMissingEntry(type_id) => {
                write!(f, "Analysis cache missing entry for TypeId {:?}", type_id)
            }
            Error::LivenessOperationNotFound(op) => {
                write!(f, "Operation {:?} not found in liveness information", op)
            }
            Error::LivenessInputNotFound(input) => {
                write!(f, "Input {:?} not found in liveness information", input)
            }
            Error::UseCountOperationNotFound(op) => {
                write!(f, "Operation {:?} not found in use count information", op)
            }
            Error::UseCountInputNotFound(input) => {
                write!(f, "Input {:?} not found in use count information", input)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NonExistentGate(h) => write!(f, "NonExistentGate({:?})", h),
            Error::NonExistentInput(h) => write!(f, "NonExistentInput({:?})", h),
            Error::NonExistentOutput(h) => write!(f, "NonExistentOutput({:?})", h),
            Error::EmptyCircuit => write!(f, "EmptyCircuit"),
            Error::InputArityOverLimit(gate) => write!(f, "InputArityOverLimit({:?})", gate),
            Error::InputArityUnderLimit(gate) => write!(f, "InputArityUnderLimit({:?})", gate),
            Error::OutputArityOverLimit(gate) => write!(f, "OutputArityOverLimit({:?})", gate),
            Error::SelfConnection(node) => write!(f, "SelfConnection({:?})", node),
            Error::UnusedInput(input) => write!(f, "UnusedInput({:?})", input),
            Error::UnusedOutput(output) => write!(f, "UnusedOutput({:?})", output),
            Error::UsedOutput(output) => write!(f, "UsedOutput({:?})", output),
            Error::CycleDetected(ops) => write!(f, "CycleDetected({:?})", ops),
            Error::AlreadyFinalized => write!(f, "AlreadyFinalized"),
            Error::InvariantViolation(msg) => write!(f, "InvariantViolation({})", msg),
            Error::AnalysisCacheTypeMismatch(type_id) => {
                write!(f, "AnalysisCacheTypeMismatch({:?})", type_id)
            }
            Error::AnalysisCacheMissingEntry(type_id) => {
                write!(f, "AnalysisCacheMissingEntry({:?})", type_id)
            }
            Error::LivenessOperationNotFound(op) => {
                write!(f, "LivenessOperationNotFound({:?})", op)
            }
            Error::LivenessInputNotFound(input) => {
                write!(f, "LivenessInputNotFound({:?})", input)
            }
            Error::UseCountOperationNotFound(op) => {
                write!(f, "UseCountOperationNotFound({:?})", op)
            }
            Error::UseCountInputNotFound(input) => {
                write!(f, "UseCountInputNotFound({:?})", input)
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
