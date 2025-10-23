use crate::{
    gate::Gate,
    handles::{Input, Node},
};

pub(super) enum Source {
    Input(Input),
    Gate(Node),
}

pub(super) struct BuilderEntry<T: Gate> {
    pub(super) gate: T,
    pub(super) backward_edges: Vec<Source>,
}

impl<T: Gate> BuilderEntry<T> {
    pub(super) fn new(gate: T) -> Self {
        Self {
            backward_edges: Vec::with_capacity(gate.arity()),
            gate,
        }
    }
}
