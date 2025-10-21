use crate::{GateHandle, InputHandle, OutputHandle};
use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitError {
    NonExistentGate(GateHandle),
    NonExistentInput(InputHandle),
    NonExistentOutput(OutputHandle),
    TooManyConnections { gate: GateHandle, arity: usize },
    TooLittleConnections { gate: GateHandle, arity: usize },
    SelfConnection(GateHandle),
    OutputAlreadyConnectedToGate(OutputHandle),
    GateAlreadyConnectedToOutput(GateHandle),
    UnusedInput(InputHandle),
    UnusedOutput(OutputHandle),
    CycleDetected(GateHandle),
    UnreachableGate(GateHandle),
    DeadEndGate(GateHandle),
    ZeroArityGate(GateHandle),
}

impl Error for CircuitError {}

impl Display for CircuitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitError::NonExistentGate(h) => write!(f, "Gate {:?} does not exist", h),
            CircuitError::NonExistentInput(h) => write!(f, "Input {:?} does not exist", h),
            CircuitError::NonExistentOutput(h) => write!(f, "Output {:?} does not exist", h),
            CircuitError::TooManyConnections { gate, arity } => {
                write!(f, "Gate {:?} already has {} connections (max)", gate, arity)
            }
            CircuitError::TooLittleConnections { gate, arity } => {
                write!(
                    f,
                    "Gate {:?} has insufficient connections (expected {})",
                    gate, arity
                )
            }
            CircuitError::SelfConnection(h) => write!(f, "Gate {:?} cannot connect to itself", h),
            CircuitError::OutputAlreadyConnectedToGate(h) => {
                write!(f, "Output {:?} is already connected", h)
            }
            CircuitError::GateAlreadyConnectedToOutput(h) => {
                write!(f, "Gate {:?} is already connected to an output", h)
            }
            CircuitError::UnusedInput(h) => write!(f, "Input {:?} is unused", h),
            CircuitError::UnusedOutput(h) => write!(f, "Output {:?} is unused", h),
            CircuitError::CycleDetected(h) => write!(f, "Cycle detected at gate {:?}", h),
            CircuitError::UnreachableGate(h) => {
                write!(f, "Gate {:?} is not reachable from any input", h)
            }
            CircuitError::DeadEndGate(h) => {
                write!(f, "Gate {:?} does not lead to any output", h)
            }
            CircuitError::ZeroArityGate(h) => {
                write!(f, "Gate {:?} has zero arity, which is not allowed", h)
            }
        }
    }
}
