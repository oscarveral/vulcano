mod entry;
#[cfg(test)]
mod tests;

use crate::{
    builder::entry::{Destination, GateEntry, Source},
    error::Error,
    gate::Gate,
    handles::{Input, Node, Output},
};

const INITIAL_GATE_CAPACITY: usize = 16;
const INITIAL_INPUT_CAPACITY: usize = 4;
const INITIAL_OUTPUT_CAPACITY: usize = 4;
const _: () = assert!(
    INITIAL_GATE_CAPACITY > 0,
    "INITIAL_GATE_CAPACITY must be positive"
);
const _: () = assert!(
    INITIAL_INPUT_CAPACITY > 0,
    "INITIAL_INPUT_CAPACITY must be positive"
);
const _: () = assert!(
    INITIAL_OUTPUT_CAPACITY > 0,
    "INITIAL_OUTPUT_CAPACITY must be positive"
);

pub struct Builder<T: Gate> {
    gate_entries: Vec<GateEntry<T>>,
    connected_outputs: Vec<bool>,
    connected_inputs: Vec<bool>,
}

impl<T: Gate> Builder<T> {
    pub fn new() -> Self {
        Self::with_capacity(
            INITIAL_GATE_CAPACITY,
            INITIAL_INPUT_CAPACITY,
            INITIAL_OUTPUT_CAPACITY,
        )
    }

    pub fn with_capacity(gates: usize, inputs: usize, outputs: usize) -> Self {
        Self {
            gate_entries: Vec::with_capacity(gates),
            connected_outputs: Vec::with_capacity(outputs),
            connected_inputs: Vec::with_capacity(inputs),
        }
    }

    pub fn gate_count(&self) -> usize {
        self.gate_entries.len()
    }

    pub fn input_count(&self) -> usize {
        self.connected_inputs.len()
    }

    pub fn output_count(&self) -> usize {
        self.connected_outputs.len()
    }

    pub fn add_gate(&mut self, gate: T) -> Node {
        let handle = self.gate_entries.len();
        self.gate_entries.push(GateEntry::new(gate));
        Node(handle)
    }

    pub fn add_input(&mut self) -> Input {
        let handle = self.connected_inputs.len();
        self.connected_inputs.push(false);
        Input(handle)
    }

    pub fn add_output(&mut self) -> Output {
        let handle = self.connected_outputs.len();
        self.connected_outputs.push(false);
        Output(handle)
    }

    pub fn connect_input_to_gate(&mut self, input: Input, gate: Node) -> Result<(), Error> {
        if input.0 >= self.connected_inputs.len() {
            return Err(Error::NonExistentInput(input));
        }
        if gate.0 >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(gate));
        }
        let gate_arity = self.gate_entries[gate.0].gate.arity();
        if self.gate_entries[gate.0].backward_edges.len() >= gate_arity {
            return Err(Error::TooManyConnections {
                gate,
                arity: gate_arity,
            });
        }
        self.gate_entries[gate.0]
            .backward_edges
            .push(Source::Input(input));
        self.connected_inputs[input.0] = true;
        Ok(())
    }

    pub fn connect_gate_to_gate(&mut self, src_gate: Node, dst_gate: Node) -> Result<(), Error> {
        if src_gate.0 >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(src_gate));
        }
        if dst_gate.0 >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(dst_gate));
        }
        if src_gate == dst_gate {
            return Err(Error::SelfConnection(src_gate));
        }
        let dst_arity = self.gate_entries[dst_gate.0].gate.arity();
        if self.gate_entries[dst_gate.0].backward_edges.len() >= dst_arity {
            return Err(Error::TooManyConnections {
                gate: dst_gate,
                arity: dst_arity,
            });
        }
        self.gate_entries[dst_gate.0]
            .backward_edges
            .push(Source::Gate(src_gate));
        self.gate_entries[src_gate.0]
            .forward_edges
            .push(Destination::Gate(dst_gate));
        Ok(())
    }

    pub fn connect_gate_to_output(&mut self, gate: Node, output: Output) -> Result<(), Error> {
        if gate.0 >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(gate));
        }
        if output.0 >= self.connected_outputs.len() {
            return Err(Error::NonExistentOutput(output));
        }
        if self.connected_outputs[output.0] {
            return Err(Error::OutputAlreadyConnectedToGate(output));
        }
        if self.gate_entries[gate.0]
            .forward_edges
            .iter()
            .any(|d| matches!(d, Destination::Output(_)))
        {
            return Err(Error::GateAlreadyConnectedToOutput(gate));
        }
        self.gate_entries[gate.0]
            .forward_edges
            .push(Destination::Output(output));
        self.connected_outputs[output.0] = true;
        Ok(())
    }

    pub fn build(&self) -> Result<(), Error> {
        for (i, &connected) in self.connected_inputs.iter().enumerate() {
            if !connected {
                return Err(Error::UnusedInput(Input(i)));
            }
        }

        for (i, &connected) in self.connected_outputs.iter().enumerate() {
            if !connected {
                return Err(Error::UnusedOutput(Output(i)));
            }
        }

        for (i, gate) in self.gate_entries.iter().enumerate() {
            if gate.gate.arity() == 0 {
                return Err(Error::ZeroArityGate(Node(i)));
            }
            if gate.backward_edges.len() != gate.gate.arity() {
                return Err(Error::TooLittleConnections {
                    gate: Node(i),
                    arity: gate.gate.arity(),
                });
            }
        }

        #[derive(Clone, Copy, PartialEq)]
        enum VisitState {
            Unvisited,
            Visiting,
            Visited,
        }

        let mut state = vec![VisitState::Unvisited; self.gate_entries.len()];

        for start_idx in 0..self.gate_entries.len() {
            if state[start_idx] != VisitState::Unvisited {
                continue;
            }

            let mut stack = vec![start_idx];

            while let Some(&gate_idx) = stack.last() {
                match state[gate_idx] {
                    VisitState::Visited => {
                        return Err(Error::AnomalyOnCycleCheck(Node(gate_idx)));
                    }
                    VisitState::Visiting => {
                        state[gate_idx] = VisitState::Visited;
                        stack.pop();
                        continue;
                    }
                    VisitState::Unvisited => {}
                }

                state[gate_idx] = VisitState::Visiting;

                for source in &self.gate_entries[gate_idx].backward_edges {
                    if let Source::Gate(src_gate) = source {
                        match state[src_gate.0] {
                            VisitState::Visiting => {
                                return Err(Error::CycleDetected(Node(src_gate.0)));
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

        let mut reachable_from_inputs = vec![false; self.gate_entries.len()];
        let mut queue = Vec::new();

        for (gate_idx, edges) in self.gate_entries.iter().enumerate() {
            if edges
                .backward_edges
                .iter()
                .any(|s| matches!(s, Source::Input(_)))
            {
                reachable_from_inputs[gate_idx] = true;
                queue.push(gate_idx);
            }
        }

        while let Some(gate_idx) = queue.pop() {
            for dest in &self.gate_entries[gate_idx].forward_edges {
                if let Destination::Gate(dst_gate) = dest
                    && !reachable_from_inputs[dst_gate.0]
                {
                    reachable_from_inputs[dst_gate.0] = true;
                    queue.push(dst_gate.0);
                }
            }
        }

        let mut reachable_to_outputs = vec![false; self.gate_entries.len()];
        queue.clear();

        for (gate_idx, edges) in self.gate_entries.iter().enumerate() {
            if edges
                .forward_edges
                .iter()
                .any(|d| matches!(d, Destination::Output(_)))
            {
                reachable_to_outputs[gate_idx] = true;
                queue.push(gate_idx);
            }
        }

        while let Some(gate_idx) = queue.pop() {
            for source in &self.gate_entries[gate_idx].backward_edges {
                if let Source::Gate(src_gate) = source
                    && !reachable_to_outputs[src_gate.0]
                {
                    reachable_to_outputs[src_gate.0] = true;
                    queue.push(src_gate.0);
                }
            }
        }

        for gate_idx in 0..self.gate_entries.len() {
            if !reachable_from_inputs[gate_idx] {
                return Err(Error::UnreachableGate(Node(gate_idx)));
            }
            if !reachable_to_outputs[gate_idx] {
                return Err(Error::DeadEndGate(Node(gate_idx)));
            }
        }

        Ok(())
    }
}

impl<T: Gate> Default for Builder<T> {
    fn default() -> Self {
        Self::new()
    }
}
