use crate::{gate::Gate, handles::Wire};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CircuitEntry<T: Gate> {
    pub(crate) gate: T,
    pub(crate) input_wires: Vec<Wire>,
    pub(crate) output_wire: Wire,
}
