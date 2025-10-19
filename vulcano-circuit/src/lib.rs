#[cfg(test)]
mod tests;

use smallvec::SmallVec;
use std::{error::Error, fmt::Display, num::NonZeroUsize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateHandle(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputHandle(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputHandle(usize);

const EDGE_THRESHOLD: usize = 4;
const INITIAL_CAPACITY: usize = 16;
const _: () = assert!(EDGE_THRESHOLD > 0, "EDGE_THRESHOLD must be positive");
const _: () = assert!(INITIAL_CAPACITY > 0, "INITIAL_CAPACITY must be positive");

pub trait Gate {
    fn arity(&self) -> NonZeroUsize;
}

enum Source {
    Input(InputHandle),
    Gate(GateHandle),
}

enum Destination {
    Output(OutputHandle),
    Gate(GateHandle),
}

type BackwardEdge = SmallVec<Source, EDGE_THRESHOLD>;
type ForwardEdge = SmallVec<Destination, EDGE_THRESHOLD>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitError {
    NonExistentGate(GateHandle),
    NonExistentInput(InputHandle),
    NonExistentOutput(OutputHandle),
    TooManyConnections { gate: GateHandle, arity: usize },
    SelfConnection(GateHandle),
    OutputAlreadyConnectedToGate(OutputHandle),
    GateAlreadyConnectedToOutput(GateHandle),
    UnusedInput(InputHandle),
    UnusedOutput(OutputHandle),
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
            CircuitError::SelfConnection(h) => write!(f, "Gate {:?} cannot connect to itself", h),
            CircuitError::OutputAlreadyConnectedToGate(h) => {
                write!(f, "Output {:?} is already connected", h)
            }
            CircuitError::GateAlreadyConnectedToOutput(h) => {
                write!(f, "Gate {:?} is already connected to an output", h)
            }
            CircuitError::UnusedInput(h) => write!(f, "Input {:?} is unused", h),
            CircuitError::UnusedOutput(h) => write!(f, "Output {:?} is unused", h),
        }
    }
}

pub struct Circuit<T: Gate> {
    gates: Vec<T>,
    backward_edges: Vec<BackwardEdge>,
    forward_edges: Vec<ForwardEdge>,
    connected_outputs: Vec<bool>,
    connected_inputs: Vec<bool>,
    input_count: usize,
    output_count: usize,
}

impl<T: Gate> Circuit<T> {
    pub fn new() -> Self {
        Self::with_capacity(INITIAL_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            gates: Vec::with_capacity(capacity),
            backward_edges: Vec::with_capacity(capacity),
            forward_edges: Vec::with_capacity(capacity),
            connected_outputs: Vec::with_capacity(0),
            connected_inputs: Vec::with_capacity(0),
            input_count: 0,
            output_count: 0,
        }
    }

    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    pub fn input_count(&self) -> usize {
        self.input_count
    }

    pub fn output_count(&self) -> usize {
        self.output_count
    }

    pub fn add_gate(&mut self, gate: T) -> GateHandle {
        let handle = self.gates.len();
        self.gates.push(gate);
        self.backward_edges.push(BackwardEdge::new());
        self.forward_edges.push(ForwardEdge::new());
        GateHandle(handle)
    }

    pub fn add_input(&mut self) -> InputHandle {
        let handle = self.input_count;
        self.input_count += 1;
        self.connected_inputs.push(false);
        InputHandle(handle)
    }

    pub fn add_output(&mut self) -> OutputHandle {
        let handle = self.output_count;
        self.output_count += 1;
        self.connected_outputs.push(false);
        OutputHandle(handle)
    }

    pub fn connect_input_to_gate(
        &mut self,
        input: InputHandle,
        gate: GateHandle,
    ) -> Result<(), CircuitError> {
        if input.0 >= self.input_count {
            return Err(CircuitError::NonExistentInput(input));
        }
        if gate.0 >= self.gates.len() {
            return Err(CircuitError::NonExistentGate(gate));
        }
        let gate_arity = self.gates[gate.0].arity().get();
        if self.backward_edges[gate.0].len() >= gate_arity {
            return Err(CircuitError::TooManyConnections {
                gate,
                arity: gate_arity,
            });
        }
        self.backward_edges[gate.0].push(Source::Input(input));
        self.connected_inputs[input.0] = true;
        Ok(())
    }

    pub fn connect_gate_to_gate(
        &mut self,
        src_gate: GateHandle,
        dst_gate: GateHandle,
    ) -> Result<(), CircuitError> {
        if src_gate.0 >= self.gates.len() {
            return Err(CircuitError::NonExistentGate(src_gate));
        }
        if dst_gate.0 >= self.gates.len() {
            return Err(CircuitError::NonExistentGate(dst_gate));
        }
        if src_gate == dst_gate {
            return Err(CircuitError::SelfConnection(src_gate));
        }
        let dst_arity = self.gates[dst_gate.0].arity().get();
        if self.backward_edges[dst_gate.0].len() >= dst_arity {
            return Err(CircuitError::TooManyConnections {
                gate: dst_gate,
                arity: dst_arity,
            });
        }
        self.backward_edges[dst_gate.0].push(Source::Gate(src_gate));
        self.forward_edges[src_gate.0].push(Destination::Gate(dst_gate));
        Ok(())
    }

    pub fn connect_gate_to_output(
        &mut self,
        gate: GateHandle,
        output: OutputHandle,
    ) -> Result<(), CircuitError> {
        if gate.0 >= self.gates.len() {
            return Err(CircuitError::NonExistentGate(gate));
        }
        if output.0 >= self.output_count {
            return Err(CircuitError::NonExistentOutput(output));
        }
        if self.connected_outputs[output.0] {
            return Err(CircuitError::OutputAlreadyConnectedToGate(output));
        }
        if self.forward_edges[gate.0]
            .iter()
            .any(|d| matches!(d, Destination::Output(_)))
        {
            return Err(CircuitError::GateAlreadyConnectedToOutput(gate));
        }
        self.forward_edges[gate.0].push(Destination::Output(output));
        self.connected_outputs[output.0] = true;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), CircuitError> {
        for (i, &connected) in self.connected_inputs.iter().enumerate() {
            if !connected {
                return Err(CircuitError::UnusedInput(InputHandle(i)));
            }
        }
        for (i, &connected) in self.connected_outputs.iter().enumerate() {
            if !connected {
                return Err(CircuitError::UnusedOutput(OutputHandle(i)));
            }
        }

        Ok(())
    }
}

impl<T: Gate> Default for Circuit<T> {
    fn default() -> Self {
        Self::new()
    }
}
