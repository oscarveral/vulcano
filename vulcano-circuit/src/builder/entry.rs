use crate::{
    gate::Gate,
    handles::{Input, Node, Output},
};
pub(super) enum Source {
    Input(Input),
    Gate(Node),
}

pub(super) enum Destination {
    Output(Output),
    Gate(Node),
}

pub(super) struct Entry<T: Gate> {
    pub(super) gate: T,
    pub(super) backward_edges: Vec<Source>,
    pub(super) forward_edges: Vec<Destination>,
}

impl<T: Gate> Entry<T> {
    pub(super) fn new(gate: T) -> Self {
        Self {
            backward_edges: Vec::with_capacity(gate.arity()),
            forward_edges: Vec::new(),
            gate,
        }
    }
}
