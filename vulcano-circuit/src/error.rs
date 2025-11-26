//! Error types used when constructing circuits with the builder API.
//!
//! These errors are returned when callers
//! attempt invalid operations (out-of-bounds handles, exceeding gate
//! arity, self-connections, etc.).

use std::any::TypeId;

use crate::handles::{GateId, InputId, OutputId};

/// Errors that can occur while constructing a circuit.
///
/// Each variant carries the relevant handle so callers can present
/// helpful diagnostics or recover (for example by creating the
/// missing handle and retrying).
#[derive(PartialEq, Eq)]
pub enum Error {
    /// The referenced gate handle doesn't exist.
    NonExistentGate(GateId),
    /// The referenced input handle doesn't exist.
    NonExistentInput(InputId),
    /// The referenced output handle doesn't exist.
    NonExistentOutput(OutputId),
    /// The circuit has no gates.
    EmptyCircuit,
    /// An attempt was made to connect more inputs to a gate than its
    /// declared arity allows.
    InputArityOverLimit(GateId),
    /// An attempt was made to connect fewer inputs to a gate than its
    /// declared arity requires.
    InputArityUnderLimit(GateId),
    /// A gate was connected to more than one output slot.
    OutputArityOverLimit(GateId),
    /// A gate was connected to itself.
    SelfConnection(GateId),
    /// An input slot was not connected to any gate.
    UnusedInput(InputId),
    /// An output slot was not connected to any gate.
    UnusedOutput(OutputId),
    /// The requested output slot is already occupied.
    UsedOutput(OutputId),
    /// A cycle was detected while ordering the circuit.
    /// Carries a list of offending [`GateId`] handles that may be
    /// involved in the cycle.
    CycleDetected(Vec<GateId>),
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
    /// A gate was not found in liveness information.
    LivenessGateNotFound(GateId),
    /// An input was not found in liveness information.
    LivenessInputNotFound(InputId),
    /// A gate was not found in use count information.
    UseCountGateNotFound(GateId),
    /// An input was not found in use count information.
    UseCountInputNotFound(InputId),
    /// A gate was not found in sub-circuit information.
    SubCircuitGateNotFound(GateId),
    /// An input was not found in sub-circuit information.
    SubCircuitInputNotFound(InputId),
    /// A value was not assigned a wire color during wire allocation.
    WireAllocationValueNotColored,
    /// Failed to find an available wire color during allocation.
    WireAllocationNoColorAvailable,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NonExistentGate(h) => write!(f, "Gate {:?} does not exist", h),
            Error::NonExistentInput(h) => write!(f, "InputId {:?} does not exist", h),
            Error::NonExistentOutput(h) => write!(f, "OutputId {:?} does not exist", h),
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
                write!(f, "InputId {:?} is not connected to any gate", input)
            }
            Error::UnusedOutput(output) => {
                write!(f, "OutputId {:?} is not connected to any gate", output)
            }
            Error::UsedOutput(output) => write!(f, "OutputId {:?} is already connected", output),
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
            Error::LivenessGateNotFound(op) => {
                write!(f, "GateId {:?} not found in liveness information", op)
            }
            Error::LivenessInputNotFound(input) => {
                write!(f, "InputId {:?} not found in liveness information", input)
            }
            Error::UseCountGateNotFound(op) => {
                write!(f, "GateId {:?} not found in use count information", op)
            }
            Error::UseCountInputNotFound(input) => {
                write!(f, "InputId {:?} not found in use count information", input)
            }
            Error::SubCircuitGateNotFound(op) => {
                write!(f, "GateId {:?} not found in sub-circuit information", op)
            }
            Error::SubCircuitInputNotFound(input) => {
                write!(
                    f,
                    "InputId {:?} not found in sub-circuit information",
                    input
                )
            }
            Error::WireAllocationValueNotColored => {
                write!(
                    f,
                    "A value was not assigned a wire color during wire allocation"
                )
            }
            Error::WireAllocationNoColorAvailable => {
                write!(
                    f,
                    "Failed to find an available wire color during allocation"
                )
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
            Error::LivenessGateNotFound(op) => {
                write!(f, "LivenessOperationNotFound({:?})", op)
            }
            Error::LivenessInputNotFound(input) => {
                write!(f, "LivenessInputNotFound({:?})", input)
            }
            Error::UseCountGateNotFound(op) => {
                write!(f, "UseCountOperationNotFound({:?})", op)
            }
            Error::UseCountInputNotFound(input) => {
                write!(f, "UseCountInputNotFound({:?})", input)
            }
            Error::SubCircuitGateNotFound(op) => {
                write!(f, "SubCircuitOperationNotFound({:?})", op)
            }
            Error::SubCircuitInputNotFound(input) => {
                write!(f, "SubCircuitInputNotFound({:?})", input)
            }
            Error::WireAllocationValueNotColored => {
                write!(f, "WireAllocationValueNotColored")
            }
            Error::WireAllocationNoColorAvailable => {
                write!(f, "WireAllocationNoColorAvailable")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
