#[cfg(test)]
mod tests;

use crate::{CircuitError, Gate, GateHandle, InputHandle, OutputHandle};
use smallvec::SmallVec;

const EDGE_THRESHOLD: usize = 4;
const INITIAL_CAPACITY: usize = 16;
const _: () = assert!(EDGE_THRESHOLD > 0, "EDGE_THRESHOLD must be positive");
const _: () = assert!(INITIAL_CAPACITY > 0, "INITIAL_CAPACITY must be positive");

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
        let gate_arity = self.gates[gate.0].arity();
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
        let dst_arity = self.gates[dst_gate.0].arity();
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

        for (i, gate) in self.gates.iter().enumerate() {
            if gate.arity() == 0 {
                return Err(CircuitError::ZeroArityGate(GateHandle(i)));
            }
            if self.backward_edges[i].len() != gate.arity() {
                return Err(CircuitError::TooLittleConnections {
                    gate: GateHandle(i),
                    arity: gate.arity(),
                });
            }
        }

        #[derive(Clone, Copy, PartialEq)]
        enum VisitState {
            Unvisited,
            Visiting,
            Visited,
        }

        let mut state = vec![VisitState::Unvisited; self.gates.len()];

        for start_idx in 0..self.gates.len() {
            if state[start_idx] != VisitState::Unvisited {
                continue;
            }

            let mut stack = vec![start_idx];

            while let Some(&gate_idx) = stack.last() {
                match state[gate_idx] {
                    VisitState::Visited => {
                        stack.pop();
                        continue;
                    }
                    VisitState::Visiting => {
                        state[gate_idx] = VisitState::Visited;
                        stack.pop();
                        continue;
                    }
                    VisitState::Unvisited => {}
                }

                state[gate_idx] = VisitState::Visiting;

                for source in &self.backward_edges[gate_idx] {
                    if let Source::Gate(src_gate) = source {
                        match state[src_gate.0] {
                            VisitState::Visiting => {
                                return Err(CircuitError::CycleDetected(GateHandle(src_gate.0)));
                            }
                            VisitState::Unvisited => {
                                stack.push(src_gate.0);
                            }
                            VisitState::Visited => {}
                        }
                    }
                }
            }
        }

        let mut reachable_from_inputs = vec![false; self.gates.len()];
        let mut queue = Vec::new();

        for (gate_idx, edges) in self.backward_edges.iter().enumerate() {
            if edges.iter().any(|s| matches!(s, Source::Input(_))) {
                reachable_from_inputs[gate_idx] = true;
                queue.push(gate_idx);
            }
        }

        while let Some(gate_idx) = queue.pop() {
            for dest in &self.forward_edges[gate_idx] {
                if let Destination::Gate(dst_gate) = dest
                    && !reachable_from_inputs[dst_gate.0]
                {
                    reachable_from_inputs[dst_gate.0] = true;
                    queue.push(dst_gate.0);
                }
            }
        }

        let mut reachable_to_outputs = vec![false; self.gates.len()];
        queue.clear();

        for (gate_idx, edges) in self.forward_edges.iter().enumerate() {
            if edges.iter().any(|d| matches!(d, Destination::Output(_))) {
                reachable_to_outputs[gate_idx] = true;
                queue.push(gate_idx);
            }
        }

        while let Some(gate_idx) = queue.pop() {
            for source in &self.backward_edges[gate_idx] {
                if let Source::Gate(src_gate) = source
                    && !reachable_to_outputs[src_gate.0]
                {
                    reachable_to_outputs[src_gate.0] = true;
                    queue.push(src_gate.0);
                }
            }
        }

        for gate_idx in 0..self.gates.len() {
            if !reachable_from_inputs[gate_idx] {
                return Err(CircuitError::UnreachableGate(GateHandle(gate_idx)));
            }
            if !reachable_to_outputs[gate_idx] {
                return Err(CircuitError::DeadEndGate(GateHandle(gate_idx)));
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
