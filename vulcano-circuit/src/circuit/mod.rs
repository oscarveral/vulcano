pub(crate) mod entry;

use crate::{circuit::entry::CircuitEntry, gate::Gate, handles::Wire};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit<T: Gate> {
    gates: Vec<CircuitEntry<T>>,
    input_wires: Vec<Wire>,
    output_wires: Vec<Wire>,
    wire_count: usize,
}

impl<T: Gate> Circuit<T> {
    pub(crate) fn new(
        gates: Vec<CircuitEntry<T>>,
        input_wires: Vec<Wire>,
        output_wires: Vec<Wire>,
        wire_count: usize,
    ) -> Self {
        Self {
            gates,
            input_wires,
            output_wires,
            wire_count,
        }
    }

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
}
