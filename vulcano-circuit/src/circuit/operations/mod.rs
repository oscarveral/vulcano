//! Operations.
//!
//! This module defines all available operations that can be added to a circuit.
//! Also defines many types used to identify operations, ports and to add semantic
//! meaning to each operation.

use crate::{
    circuit::operations::{
        clone::CloneId, drop::DropId, gate::GateId, input::InputId, output::OutputId,
    },
    error::{Error, Result},
};

pub mod clone;
pub mod drop;
pub mod gate;
pub mod input;
pub mod output;

/// Handle identifying a port. Operations may have multiple input and output ports.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PortId(usize);

impl PortId {
    /// Create a new port id from a numeric index.
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Return the numeric index.
    pub fn index(self) -> usize {
        self.0
    }
}

/// Consumer operations on a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consumer {
    /// Used by a gate.
    Gate(GateId),
    /// Used by a clone.
    Clone(CloneId),
    /// Used by a drop.
    Drop(DropId),
    /// Circuit output.
    Output(OutputId),
}

impl TryFrom<Operation> for Consumer {
    type Error = Error;

    fn try_from(value: Operation) -> Result<Self> {
        match value {
            Operation::Gate(id) => Ok(Consumer::Gate(id)),
            Operation::Clone(id) => Ok(Consumer::Clone(id)),
            Operation::Drop(id) => Ok(Consumer::Drop(id)),
            Operation::Output(id) => Ok(Consumer::Output(id)),
            _ => Err(Error::InvalidOperationConversion(value)),
        }
    }
}

/// What produces a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Producer {
    /// External circuit input.
    Input(InputId),
    /// Produced by a gate.
    Gate(GateId),
    /// Produced by a clone.
    Clone(CloneId),
}

impl TryFrom<Operation> for Producer {
    type Error = Error;

    fn try_from(value: Operation) -> Result<Self> {
        match value {
            Operation::Input(id) => Ok(Producer::Input(id)),
            Operation::Gate(id) => Ok(Producer::Gate(id)),
            Operation::Clone(id) => Ok(Producer::Clone(id)),
            _ => Err(Error::InvalidOperationConversion(value)),
        }
    }
}

/// An enum representing all possible operations in a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    /// Circuit input.
    Input(InputId),
    /// A gate computation.
    Gate(GateId),
    /// A clone operation.
    Clone(CloneId),
    /// A drop operation.
    Drop(DropId),
    /// A circuit output.
    Output(OutputId),
}

impl From<Consumer> for Operation {
    fn from(consumer: Consumer) -> Self {
        match consumer {
            Consumer::Gate(id) => Operation::Gate(id),
            Consumer::Clone(id) => Operation::Clone(id),
            Consumer::Drop(id) => Operation::Drop(id),
            Consumer::Output(id) => Operation::Output(id),
        }
    }
}

impl From<Producer> for Operation {
    fn from(producer: Producer) -> Self {
        match producer {
            Producer::Input(id) => Operation::Input(id),
            Producer::Gate(id) => Operation::Gate(id),
            Producer::Clone(id) => Operation::Clone(id),
        }
    }
}
