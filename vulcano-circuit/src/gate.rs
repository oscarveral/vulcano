//! Gate trait
//!
//! This module defines the trait for user-defined gates.

use std::hash::Hash;

use crate::{error::Result, handles::Ownership};

/// Trait implemented by a gate used inside a circuit.
///
/// A gate is a descriptor for a computational operation.
/// Typically implemented as an enum of all possible gate types.
pub trait Gate: Eq + Copy + 'static {
    /// Number of inputs the gate consumes.
    fn input_count(&self) -> usize;

    /// Number of outputs the gate produces.
    fn output_count(&self) -> usize;

    /// The type descriptor for operands (e.g., ciphertext, plaintext).
    type Operand: Eq + Copy + Hash + 'static;

    /// Returns the operand type at the given input index.
    fn input_type(&self, idx: usize) -> Result<Self::Operand>;

    /// Returns the operand type at the given output index.
    fn output_type(&self, idx: usize) -> Result<Self::Operand>;

    /// Returns the access mode for the input at the given index.
    fn access_mode(&self, idx: usize) -> Result<Ownership>;

    /// Returns an iterator over all input types.
    fn input_types(&self) -> Result<impl Iterator<Item = Self::Operand>> {
        (0..self.input_count())
            .map(|idx| self.input_type(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }

    /// Returns an iterator over all output types.
    fn output_types(&self) -> Result<impl Iterator<Item = Self::Operand>> {
        (0..self.output_count())
            .map(|idx| self.output_type(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }

    /// Returns an iterator over all access modes.
    fn access_modes(&self) -> Result<impl Iterator<Item = Ownership>> {
        (0..self.input_count())
            .map(|idx| self.access_mode(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }
}
