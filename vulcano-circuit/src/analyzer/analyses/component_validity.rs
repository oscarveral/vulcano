//! Useful circuit analyses.
//!
//! This module provides a circuit analysis that computes what subgraphs are
//! invalid computational workflows. Valid subgraphs are those that appropiately consume some inputs,
//! does some gate computation over them, and emits some outputs.

use crate::{
    analyzer::{Analysis, Analyzer, analyses::connected_components::ConnectedComponents},
    circuit::Circuit,
    error::Result,
    gate::Gate,
};

/// Validity of the components of the circuit.
struct ComponentValidity {
    /// The invalid components of the circuit.
    invalid: Vec<usize>,
    /// Number of total subgraphs.
    count: usize,
}

impl ComponentValidity {
    /// Returns the invalid components of the circuit.
    pub(super) fn get_invalid(&self) -> impl Iterator<Item = usize> {
        self.invalid.iter().copied()
    }

    /// Returns the valid components of the circuit.
    pub(super) fn get_valid(&self) -> impl Iterator<Item = usize> {
        (0..self.count).filter(|&comp| !self.invalid.contains(&comp))
    }
}

impl Analysis for ComponentValidity {
    type Output = Self;

    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        let connected = analyzer.get::<ConnectedComponents>(circuit)?;

        let invalid = (0..connected.get_count())
            .filter(|&comp| {
                let has_input = circuit
                    .get_input_ids()
                    .any(|id| connected.get_component(id) == comp);
                let has_gate = circuit
                    .get_gate_ids()
                    .any(|id| connected.get_component(id) == comp);
                let has_output = circuit
                    .get_output_ids()
                    .any(|id| connected.get_component(id) == comp);
                !(has_input && has_gate && has_output)
            })
            .collect();

        Ok(ComponentValidity {
            invalid,
            count: connected.get_count(),
        })
    }
}
