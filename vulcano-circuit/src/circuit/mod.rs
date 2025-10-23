pub(crate) mod entry;
#[cfg(test)]
mod tests;

use std::fmt::Write;

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

    pub fn to_ssa(&self) -> String {
        let mut output = String::new();

        writeln!(
            &mut output,
            "; Circuit with {} inputs, {} gates, {} outputs",
            self.input_count(),
            self.gate_count(),
            self.output_count()
        )
        .unwrap();

        for (i, wire) in self.input_wires.iter().enumerate() {
            writeln!(&mut output, "%w{} = input @i{}", wire.0, i).unwrap();
        }

        if !self.input_wires.is_empty() && !self.gates.is_empty() {
            writeln!(&mut output).unwrap();
        }

        for entry in &self.gates {
            write!(
                &mut output,
                "%w{} = {}(",
                entry.output_wire.0,
                entry.gate.name()
            )
            .unwrap();

            for (i, input_wire) in entry.input_wires.iter().enumerate() {
                if i > 0 {
                    write!(&mut output, ", ").unwrap();
                }
                write!(&mut output, "%w{}", input_wire.0).unwrap();
            }

            writeln!(&mut output, ")").unwrap();
        }

        if !self.gates.is_empty() && !self.output_wires.is_empty() {
            writeln!(&mut output).unwrap();
        }

        for (i, wire) in self.output_wires.iter().enumerate() {
            writeln!(&mut output, "output @o{} = %w{}", i, wire.0).unwrap();
        }

        output
    }
}
