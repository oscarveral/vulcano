use crate::handles::{Input, Node, Output};
use std::{error::Error as StdErr, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NonExistentGate(Node),
    NonExistentInput(Input),
    NonExistentOutput(Output),
    TooManyConnections { gate: Node, arity: usize },
    TooLittleConnections { gate: Node, arity: usize },
    SelfConnection(Node),
    OutputAlreadyConnectedToGate(Output),
    GateAlreadyConnectedToOutput(Node),
    UnusedInput(Input),
    UnusedOutput(Output),
    CycleDetected(Node),
    UnreachableGate(Node),
    DeadEndGate(Node),
    ZeroArityGate(Node),
    AnomalyOnCycleCheck(Node),
    UnmappedGateWire(Node),
    UnexpectedNoneGateEntry(Node),
    UnexpectedUnusedOutput(Output),
}

impl StdErr for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NonExistentGate(h) => write!(f, "Gate {:?} does not exist", h),
            Error::NonExistentInput(h) => write!(f, "Input {:?} does not exist", h),
            Error::NonExistentOutput(h) => write!(f, "Output {:?} does not exist", h),
            Error::TooManyConnections { gate, arity } => {
                write!(f, "Gate {:?} already has {} connections (max)", gate, arity)
            }
            Error::TooLittleConnections { gate, arity } => {
                write!(
                    f,
                    "Gate {:?} has insufficient connections (expected {})",
                    gate, arity
                )
            }
            Error::SelfConnection(h) => write!(f, "Gate {:?} cannot connect to itself", h),
            Error::OutputAlreadyConnectedToGate(h) => {
                write!(f, "Output {:?} is already connected", h)
            }
            Error::GateAlreadyConnectedToOutput(h) => {
                write!(f, "Gate {:?} is already connected to an output", h)
            }
            Error::UnusedInput(h) => write!(f, "Input {:?} is unused", h),
            Error::UnusedOutput(h) => write!(f, "Output {:?} is unused", h),
            Error::CycleDetected(h) => write!(f, "Cycle detected at gate {:?}", h),
            Error::UnreachableGate(h) => {
                write!(f, "Gate {:?} is not reachable from any input", h)
            }
            Error::DeadEndGate(h) => {
                write!(f, "Gate {:?} does not lead to any output", h)
            }
            Error::ZeroArityGate(h) => {
                write!(f, "Gate {:?} has zero arity, which is not allowed", h)
            }
            Error::AnomalyOnCycleCheck(h) => {
                write!(f, "Anomaly detected during cycle check at gate {:?}", h)
            }
            Error::UnmappedGateWire(h) => {
                write!(f, "Gate {:?} has an unmapped wire", h)
            }
            Error::UnexpectedNoneGateEntry(h) => {
                write!(f, "Unexpected None gate entry for gate {:?}", h)
            }
            Error::UnexpectedUnusedOutput(h) => {
                write!(f, "Unexpected unused output {:?}", h)
            }
        }
    }
}
