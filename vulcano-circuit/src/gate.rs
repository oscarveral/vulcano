//! Gate trait and helpers
//!
//! This module defines the minimal [`Gate`] trait used by the circuit
//! representation. A gate is a small pure value that describes an
//! operation (for example: an adder, a boolean AND, a constant loader,
//! etc.). The circuit uses implementations of [`Gate`] when building and
//! executing the computation graph.
//!
//! Conventions:
//! - Implementations should make [`Gate::arity`] and [`Gate::name`] cheap
//!   to call.
//! - A gate is expected to produce exactly one output value. That single
//!   output may be consumed multiple times by downstream gates (fan-out is
//!   supported at the circuit level).

/// Trait implemented by a gate used inside a circuit.
///
/// A [`Gate`] is a small descriptor for an operation node. The circuit
/// machinery can query the gate for its [`Gate::arity`] and its [`Gate::name`].
/// Gates are expected to produce a single output value; the circuit model handles
/// fan-out (multiple consumers of that output).
///
/// Keeping this trait minimal keeps gate implementations simple and
/// allows the circuit code to remain generic over the concrete gate
/// type.
pub trait Gate {
    /// Number of inputs the gate consumes.
    ///
    /// For a binary adder this would be `2`, for a unary negation `1`,
    /// and for a constant-producing gate `0`.
    fn arity(&self) -> usize;

    /// A human-readable name for the gate.
    ///
    /// This is used in debugging, logging and textual dumps of the
    /// circuit. Implementations should return a short, descriptive
    /// string slice (often a static string).
    fn name(&self) -> &str;
}
