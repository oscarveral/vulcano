use crate::{
    gate::Gate,
    handles::{Input, Node},
};

pub(super) enum Source {
    Input(Input),
    Gate(Node),
}

pub(super) enum Destination {
    Output,
    Gate(Node),
}

pub(super) struct BuilderEntry<T: Gate> {
    pub(super) gate: T,
    pub(super) backward_edges: Vec<Source>,
    pub(super) forward_edges: Vec<Destination>,
}

impl<T: Gate> BuilderEntry<T> {
    pub(super) fn new(gate: T) -> Self {
        Self {
            backward_edges: Vec::with_capacity(gate.arity()),
            forward_edges: Vec::new(),
            gate,
        }
    }
}
