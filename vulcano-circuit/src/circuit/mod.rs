//! A circuit representation.
//!
//! A circuit is a collection of subcircuits.

use vulcano_arena::Arena;

use crate::{
    circuit::subcircuit::{CircuitId, Subcircuit},
    error::{Error, Result},
    gate::Gate,
};

pub mod operations;
pub mod subcircuit;
pub mod value;

/// A circuit representation.
pub struct Circuit<G: Gate> {
    /// Individual disjoint subcircuits.
    subcircuits: Arena<Subcircuit<G>>,
}

impl<G: Gate> Circuit<G> {
    /// Create a new empty circuit.
    pub fn new() -> Self {
        let subcircuits = Arena::new();
        Self { subcircuits }
    }

    /// Create a new empty circuit with a given capacity of subcircuits.
    pub fn with_capacity(capacity: usize) -> Self {
        let subcircuits = Arena::with_capacity(capacity);
        Self { subcircuits }
    }

    /// Iterate over all subcircuits.
    pub fn iter(&self) -> impl Iterator<Item = &Subcircuit<G>> {
        self.subcircuits.iter().map(|(_, subcircuit)| subcircuit)
    }

    /// Iterate over all subcircuits mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Subcircuit<G>> {
        self.subcircuits
            .iter_mut()
            .map(|(_, subcircuit)| subcircuit)
    }

    /// Get a subcircuit by id.
    pub fn get(&self, id: CircuitId) -> Option<&Subcircuit<G>> {
        self.subcircuits.get(id.key())
    }

    /// Get a subcircuit mutably by id.
    pub fn get_mut(&mut self, id: CircuitId) -> Option<&mut Subcircuit<G>> {
        self.subcircuits.get_mut(id.key())
    }

    /// Add a new subcircuit to the circuit.
    pub fn add_subcircuit(&mut self) -> Result<CircuitId> {
        // Reserve a key and create a subcircuit id.
        let subcircuit_key = self.subcircuits.reserve();
        let subcircuit_id = CircuitId::new(subcircuit_key);

        // Create a subcircuit and fill it into the arena.
        let subcircuit = Subcircuit::new(subcircuit_id);
        self.subcircuits
            .fill(subcircuit_id.key(), subcircuit)
            .map_err(|_| {
                // If we failed to fill the arena, remove the reserved key.
                self.subcircuits.remove(subcircuit_id.key());
                Error::FailedToCreateCircuit(subcircuit_id)
            })?;
        Ok(subcircuit_id)
    }

    /// Remove a subcircuit from the circuit.
    pub fn remove_subcircuit(&mut self, id: CircuitId) -> Option<Subcircuit<G>> {
        self.subcircuits.remove(id.key())
    }
}

impl<G: Gate> Default for Circuit<G> {
    fn default() -> Self {
        Self::new()
    }
}
