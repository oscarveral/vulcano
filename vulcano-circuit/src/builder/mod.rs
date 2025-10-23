mod entry;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::{
    Circuit,
    builder::entry::{BuilderEntry, Source},
    circuit::entry::CircuitEntry,
    error::Error,
    gate::Gate,
    handles::{Input, Node, Output, Wire},
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
    gate_entries: Vec<BuilderEntry<T>>,
    connected_outputs: Vec<Option<Node>>,
    connected_inputs: Vec<Option<Node>>,
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
        self.gate_entries.push(BuilderEntry::new(gate));
        Node(handle)
    }

    pub fn add_input(&mut self) -> Input {
        let handle = self.connected_inputs.len();
        self.connected_inputs.push(None);
        Input(handle)
    }

    pub fn add_output(&mut self) -> Output {
        let handle = self.connected_outputs.len();
        self.connected_outputs.push(None);
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
        self.connected_inputs[input.0] = Some(gate);
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
        Ok(())
    }

    pub fn connect_gate_to_output(&mut self, gate: Node, output: Output) -> Result<(), Error> {
        if gate.0 >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(gate));
        }
        if output.0 >= self.connected_outputs.len() {
            return Err(Error::NonExistentOutput(output));
        }
        if self.connected_outputs[output.0].is_some() {
            return Err(Error::OutputAlreadyConnectedToGate(output));
        }
        if self.connected_outputs.contains(&Some(gate)) {
            return Err(Error::GateAlreadyConnectedToOutput(gate));
        }
        self.connected_outputs[output.0] = Some(gate);
        Ok(())
    }

    pub fn build(self) -> Result<Circuit<T>, Error> {
        for (i, &connected) in self.connected_inputs.iter().enumerate() {
            if connected.is_none() {
                return Err(Error::UnusedInput(Input(i)));
            }
        }

        for (i, &connected) in self.connected_outputs.iter().enumerate() {
            if connected.is_none() {
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
        let mut topological_order = Vec::with_capacity(self.gate_entries.len());
        let mut reachable_from_inputs = vec![false; self.gate_entries.len()];

        for (gate_idx, entry) in self.gate_entries.iter().enumerate() {
            if entry
                .backward_edges
                .iter()
                .any(|s| matches!(s, Source::Input(_)))
            {
                reachable_from_inputs[gate_idx] = true;
            }
        }

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
                        topological_order.push(gate_idx);
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

        for &gate_idx in &topological_order {
            if !reachable_from_inputs[gate_idx] {
                continue;
            }
            for (dst_idx, entry) in self.gate_entries.iter().enumerate() {
                if entry
                    .backward_edges
                    .iter()
                    .any(|s| matches!(s, Source::Gate(g) if g.0 == gate_idx))
                {
                    reachable_from_inputs[dst_idx] = true;
                }
            }
        }

        for (gate_idx, &is_reachable) in reachable_from_inputs
            .iter()
            .enumerate()
            .take(self.gate_entries.len())
        {
            if !is_reachable {
                return Err(Error::UnreachableGate(Node(gate_idx)));
            }
        }

        let mut reachable_to_outputs = vec![false; self.gate_entries.len()];

        for &gate_node in self.connected_outputs.iter().flatten() {
            reachable_to_outputs[gate_node.0] = true;
        }

        for &gate_idx in topological_order.iter().rev() {
            if !reachable_to_outputs[gate_idx] {
                continue;
            }
            for source in &self.gate_entries[gate_idx].backward_edges {
                if let Source::Gate(src_gate) = source {
                    reachable_to_outputs[src_gate.0] = true;
                }
            }
        }

        for (gate_idx, &is_reachable) in reachable_to_outputs
            .iter()
            .enumerate()
            .take(self.gate_entries.len())
        {
            if !is_reachable {
                return Err(Error::DeadEndGate(Node(gate_idx)));
            }
        }

        let mut wire_counter: usize = 0;

        let input_wires: Vec<Wire> = (0..self.connected_inputs.len())
            .map(|_| {
                let w = Wire(wire_counter);
                wire_counter += 1;
                w
            })
            .collect();

        let mut gate_output_wires: HashMap<Node, Wire> =
            HashMap::with_capacity(self.gate_entries.len());

        let mut circuit_entries: Vec<CircuitEntry<T>> = Vec::with_capacity(self.gate_entries.len());

        let mut owned_entries: Vec<Option<BuilderEntry<T>>> =
            self.gate_entries.into_iter().map(Some).collect();

        for &gate_idx in &topological_order {
            let entry = owned_entries[gate_idx]
                .take()
                .ok_or(Error::UnexpectedNoneGateEntry(Node(gate_idx)))?;

            let mut gate_input_wires = Vec::with_capacity(entry.backward_edges.len());

            for source in &entry.backward_edges {
                let wire = match source {
                    Source::Input(input) => input_wires[input.0],
                    Source::Gate(src_gate) => *gate_output_wires
                        .get(src_gate)
                        .ok_or(Error::UnmappedGateWire(*src_gate))?,
                };
                gate_input_wires.push(wire);
            }

            let output_wire = Wire(wire_counter);
            gate_output_wires.insert(Node(gate_idx), output_wire);
            wire_counter += 1;

            circuit_entries.push(CircuitEntry {
                gate: entry.gate,
                input_wires: gate_input_wires,
                output_wire,
            });
        }

        let mut output_wires: Vec<Wire> = Vec::with_capacity(self.connected_outputs.len());
        for (i, &gate_node) in self.connected_outputs.iter().enumerate() {
            let gate = gate_node.ok_or(Error::UnexpectedUnusedOutput(Output(i)))?;
            let wire = gate_output_wires
                .get(&gate)
                .ok_or(Error::UnmappedGateWire(gate))?;
            output_wires.push(*wire);
        }

        Ok(Circuit::new(
            circuit_entries,
            input_wires,
            output_wires,
            wire_counter,
        ))
    }
}

impl<T: Gate> Default for Builder<T> {
    fn default() -> Self {
        Self::new()
    }
}
