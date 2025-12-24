//! Gate trait and helpers
//!
//! This module defines the minimal traits needed to define, create
//! and compile computational circuits.

use std::num::NonZeroUsize;

use crate::{error::Result, handles::AccessMode};

/// Trait implemented by a gate used inside a circuit.
///
/// A gate is a small descriptor for an operation node.
/// Intended to be an enum of all possible gate types.
pub(super) trait Gate: Eq + Copy {
    /// Number of inputs the gate consumes.
    /// This can be though as the number of input ports.
    ///
    /// For a binary adder this would be `2`, for a unary negation `1`.
    fn input_count(&self) -> NonZeroUsize;

    /// Number of outputs the gate produces.
    /// This can be though as the number of output ports.
    ///
    /// For a binary adder this would be `1`, for a unary negation `1`.
    /// For thigs like hoisted rotations could be more.
    fn output_count(&self) -> NonZeroUsize;

    /// The type of the operands this gate consumes.
    /// Intended to be an enum of all possible operand types.
    type Operand: Eq + Copy;

    /// Returns the operand type at the given index.
    fn input(&self, idx: usize) -> Result<Self::Operand>;

    /// Returns an iterator over all input operands types.
    fn inputs(&self) -> Result<impl Iterator<Item = Self::Operand>> {
        (0..self.input_count().get())
            .map(|idx| self.input(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }

    /// Returns the operand type at the given index.
    fn output(&self, idx: usize) -> Result<Self::Operand>;

    /// Returns an iterator over all output operands types.
    fn outputs(&self) -> Result<impl Iterator<Item = Self::Operand>> {
        (0..self.output_count().get())
            .map(|idx| self.output(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }

    /// Returns the access mode for the input at the given index.
    fn access_mode(&self, idx: usize) -> Result<AccessMode>;

    /// Returns an iterator over all access modes.
    fn access_modes(&self) -> Result<impl Iterator<Item = AccessMode>> {
        (0..self.input_count().get())
            .map(|idx| self.access_mode(idx))
            .collect::<Result<Vec<_>>>()
            .map(|v| v.into_iter())
    }
}
