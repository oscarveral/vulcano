//! Builder for constructing computation circuits.
//!
//! The [`Builder`] offers a lightweight, incremental API to create gates
//! and wire them together. It performs cheap, local validation and
//! returns structured errors from [`crate::error::Error`] when callers
//! attempt invalid operations (out-of-bounds handles, exceeding gate
//! arity, self-connections, or trying to reuse an output slot).

#[cfg(test)]
mod tests;

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::circuit::Circuit,
    handles::{Input, Operation, Output, Source},
};

/// Incremental builder for a circuit.
///
/// The builder stores gates and their backward edges (the sources for
/// each gate's inputs). It exposes helpers to add gates/inputs/outputs
/// and to connect them. All helpers return [`crate::error::Error`]
/// variants on invalid operations.
pub struct Builder<T: Gate> {
    /// Per-gate storage: (gate, list of backward sources).
    gate_entries: Vec<(T, Vec<Source>)>,
    /// Per-output storage: (output, source).
    connected_outputs: Vec<Option<Operation>>,
    /// Per-input storage: (input, connected).
    connected_inputs: Vec<bool>,
}

impl<T: Gate> Builder<T> {
    /// Create a new, empty [`Builder`].
    pub fn new() -> Self {
        Self {
            gate_entries: Vec::new(),
            connected_outputs: Vec::new(),
            connected_inputs: Vec::new(),
        }
    }

    /// Return the number of gates currently registered in the builder.
    ///
    /// This is the number of [`Operation`] handles that have been
    /// issued by [`Builder::add_gate`] so far.        
    pub fn gate_count(&self) -> usize {
        self.gate_entries.len()
    }

    /// Return the number of input slots in the circuit under
    /// construction.
    ///
    /// Each input is represented by an [`crate::handles::Input`] handle
    /// created with [`Builder::add_input`].
    pub fn input_count(&self) -> usize {
        self.connected_inputs.len()
    }

    /// Return the number of output slots in the circuit under
    /// construction.
    ///
    /// Each output is represented by an [`crate::handles::Output`]
    /// handle created with [`Builder::add_output`].
    pub fn output_count(&self) -> usize {
        self.connected_outputs.len()
    }

    /// Add a new gate instance to the builder and return its
    /// [`crate::handles::Operation`] handle.
    ///
    /// The builder takes ownership of `gate`. A fresh per-gate
    /// input-source list is created and the returned handle may be
    /// used in subsequent connect calls.
    pub fn add_gate(&mut self, gate: T) -> Operation {
        let handle = self.gate_entries.len();
        self.gate_entries.push((gate, Vec::new()));
        Operation::new(handle)
    }

    /// Add a new external input slot and return its [`Input`] handle.
    ///
    /// The slot starts unconnected (represented as `false`) until
    /// [`Builder::connect_input_to_gate`] wires it into a gate.
    pub fn add_input(&mut self) -> Input {
        let handle = self.connected_inputs.len();
        self.connected_inputs.push(false);
        Input::new(handle)
    }

    /// Add a new external output slot and return its [`Output`] handle.
    ///
    /// The slot starts unconnected (represented as [`None`]) until
    /// [`Builder::connect_gate_to_output`] assigns a gate output to it.
    pub fn add_output(&mut self) -> Output {
        let handle = self.connected_outputs.len();
        self.connected_outputs.push(None);
        Output::new(handle)
    }

    /// Connect an external `input` slot to a gate's next available
    /// input position.
    ///
    /// Validation performed:
    /// - `input` must be an existing input slot, otherwise
    ///   [`Error::NonExistentInput`] is returned.
    /// - `gate` must refer to an existing gate, otherwise
    ///   [`Error::NonExistentGate`] is returned.
    /// - The gate must have remaining input arity; otherwise
    ///   [`Error::InputArityOverLimit`] is returned.
    ///
    /// On success, the function records the backward edge in the
    /// gate's source list and marks the input slot as connected.
    pub fn connect_input_to_gate(&mut self, input: Input, gate: Operation) -> Result<()> {
        let input_idx = input.id();
        if input_idx >= self.connected_inputs.len() {
            return Err(Error::NonExistentInput(input));
        }
        let gate_idx = gate.id();
        if gate_idx >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(gate));
        }
        let gate_arity = self.gate_entries[gate_idx].0.arity();
        let edges = &mut self.gate_entries[gate_idx].1;
        if edges.len() >= gate_arity {
            return Err(Error::InputArityOverLimit(gate));
        }
        edges.push(Source::Input(input));
        self.connected_inputs[input_idx] = true;
        Ok(())
    }

    /// Connect the output of `src_gate` to the next available input of
    /// `dst_gate`.
    ///
    /// Validation performed:
    /// - Both handles must exist, otherwise [`Error::NonExistentGate`]
    ///   is returned for the offending handle.
    /// - Connecting a gate to itself is forbidden and yields
    ///   [`Error::SelfConnection`].
    /// - The destination gate must have remaining input arity; otherwise
    ///   [`Error::InputArityOverLimit`] is returned.
    ///
    /// On success, the source gate is appended to the destination's
    /// backward-edge list.
    pub fn connect_gate_to_gate(&mut self, src_gate: Operation, dst_gate: Operation) -> Result<()> {
        let src_idx = src_gate.id();
        let dst_idx = dst_gate.id();

        if src_idx >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(src_gate));
        }
        if dst_idx >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(dst_gate));
        }
        if src_gate == dst_gate {
            return Err(Error::SelfConnection(src_gate));
        }

        let dst_arity = self.gate_entries[dst_idx].0.arity();
        let back = &mut self.gate_entries[dst_idx].1;
        if back.len() >= dst_arity {
            return Err(Error::InputArityOverLimit(dst_gate));
        }

        back.push(Source::Gate(src_gate));
        Ok(())
    }

    /// Connect the output of `gate` to an external `output` slot.
    ///
    /// Validation performed:
    /// - `gate` must exist, otherwise [`Error::NonExistentGate`].
    /// - `output` must exist, otherwise [`Error::NonExistentOutput`].
    /// - The output slot must be unused, otherwise [`Error::UsedOutput`]
    ///   is returned.
    /// - A gate may only be attached to a single output slot; attempting
    ///   to attach it again yields [`Error::OutputArityOverLimit`].
    ///
    /// On success the output slot is marked as connected to `gate`.
    pub fn connect_gate_to_output(&mut self, gate: Operation, output: Output) -> Result<()> {
        let gate_idx = gate.id();
        if gate_idx >= self.gate_entries.len() {
            return Err(Error::NonExistentGate(gate));
        }
        let out_idx = output.id();
        if out_idx >= self.connected_outputs.len() {
            return Err(Error::NonExistentOutput(output));
        }
        if self.connected_outputs[out_idx].is_some() {
            return Err(Error::UsedOutput(output));
        }
        if self
            .connected_outputs
            .iter()
            .any(|opt| opt.as_ref().is_some_and(|g| *g == gate))
        {
            return Err(Error::OutputArityOverLimit(gate));
        }

        self.connected_outputs[out_idx] = Some(gate);
        Ok(())
    }

    /// Validate the current builder state for completeness and
    /// correctness.
    ///
    /// On success, returns `Ok(())`. Otherwise, returns the first
    /// encountered error from [`crate::error::Error`]. Errors checked:
    /// - Unused inputs. [`Error::UnusedInput`]
    /// - Unused outputs. [`Error::UnusedOutput`]
    /// - Gates with too many inputs. [`Error::InputArityOverLimit`]
    /// - Gates with too few inputs. [`Error::InputArityUnderLimit`]
    fn validate(&self) -> Result<()> {
        if self.gate_entries.is_empty() {
            return Err(Error::EmptyCircuit);
        }
        for (i, connected) in self.connected_inputs.iter().enumerate() {
            if !connected {
                return Err(Error::UnusedInput(Input::new(i)));
            }
        }
        for (i, output_opt) in self.connected_outputs.iter().enumerate() {
            if output_opt.is_none() {
                return Err(Error::UnusedOutput(Output::new(i)));
            }
        }
        for (i, (gate, sources)) in self.gate_entries.iter().enumerate() {
            let arity = gate.arity();
            if sources.len() > arity {
                // This should not happen if connect methods are used.
                return Err(Error::InputArityOverLimit(Operation::new(i)));
            }
            if sources.len() < arity {
                return Err(Error::InputArityUnderLimit(Operation::new(i)));
            }
        }
        Ok(())
    }

    /// Finalize the builder and produce a concrete [`Circuit`].
    ///
    /// This consumes the builder and performs the following steps:
    /// 1. Validate local invariants.
    /// 2. Collect gate entries with their `Source` dependencies directly.
    ///
    /// Guarantees and notes:
    /// - The per-gate input ordering is preserved: each gate's
    ///   `Vec<Source>` is used as the canonical input order.
    ///
    /// Returns a [`Circuit`] on success or an appropriate
    /// [`Error`] if validation fails.
    pub fn finalize(self) -> Result<Circuit<T>> {
        self.validate()?;

        let gate_entries = self.gate_entries;
        let input_count = self.connected_inputs.len();

        let mut connected_outputs: Vec<Operation> =
            Vec::with_capacity(self.connected_outputs.len());
        for (i, opt) in self.connected_outputs.into_iter().enumerate() {
            match opt {
                Some(op) => connected_outputs.push(op),
                None => return Err(Error::UnusedOutput(Output::new(i))),
            }
        }

        Ok(Circuit::new(gate_entries, input_count, connected_outputs))
    }
}

impl<T: Gate> Default for Builder<T> {
    fn default() -> Self {
        Self::new()
    }
}
