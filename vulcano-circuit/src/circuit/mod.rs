use crate::{gate::Gate, handles::Wire};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEntry<T: Gate> {
    gate: T,
    input_wires: Vec<Wire>,
    output_wire: Wire,
}

impl<T: Gate> GateEntry<T> {
    pub fn gate(&self) -> &T {
        &self.gate
    }

    pub fn input_wires(&self) -> &[Wire] {
        &self.input_wires
    }

    pub fn output_wire(&self) -> Wire {
        self.output_wire
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit<T: Gate> {
    gates: Vec<GateEntry<T>>,
    input_wires: Vec<Wire>,
    output_wires: Vec<Wire>,
    wire_count: usize,
}

impl<T: Gate> Circuit<T> {
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    pub fn input_count(&self) -> usize {
        self.input_wires.len()
    }

    pub fn output_count(&self) -> usize {
        self.output_wires.len()
    }

    pub fn wire_count(&self) -> usize {
        self.wire_count
    }

    pub fn gates(&self) -> &[GateEntry<T>] {
        &self.gates
    }

    pub fn input_wires(&self) -> &[Wire] {
        &self.input_wires
    }

    pub fn output_wires(&self) -> &[Wire] {
        &self.output_wires
    }
}
