//! Linear SSA Circuit Representation
//!
//! This module provides the circuit data structure in Linear SSA form.
//! Values are defined exactly once and consumed exactly once.
//! Values can be borrowed any number of times before being consumed.

use crate::{
    error::{Error, Result},
    gate::Gate,
    handles::{CloneId, DropId, GateId, InputId, OutputId, Ownership, PortId, ValueId},
};

/// A gate operation: user-defined computation.
pub(super) struct GateOperation<G: Gate> {
    /// The gate descriptor.
    pub gate: G,
    /// Input values.
    pub inputs: Vec<ValueId>,
    /// Output values.
    pub outputs: Vec<ValueId>,
}

impl<G: Gate> GateOperation<G> {
    /// Get the gate descriptor.
    pub(super) fn get_gate(&self) -> &G {
        &self.gate
    }

    /// Get the input values.
    pub(super) fn get_inputs(&self) -> &[ValueId] {
        &self.inputs
    }

    /// Get the output values.
    pub(super) fn get_outputs(&self) -> &[ValueId] {
        &self.outputs
    }
}

/// Clone operation: borrow one value, produce N copies.
pub(super) struct CloneOperation {
    /// The input value.
    pub input: ValueId,
    /// The output values.
    pub outputs: Vec<ValueId>,
}

impl CloneOperation {
    /// Get the input value.
    pub(super) fn get_input(&self) -> ValueId {
        self.input
    }

    /// Get the output values.
    pub(super) fn get_outputs(&self) -> &[ValueId] {
        &self.outputs
    }

    /// Get the number of output copies.
    pub(super) fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

/// Drop operation: consume a value, produce nothing.
pub(super) struct DropOperation {
    /// The input value.
    pub input: ValueId,
}

impl DropOperation {
    /// Get the input value.
    pub(super) fn get_input(&self) -> ValueId {
        self.input
    }
}

/// Input operation: external circuit input, produces one value.
pub(super) struct InputOperation {
    /// The output value.
    output: ValueId,
}

impl InputOperation {
    /// Get the output value.
    pub(super) fn get_output(&self) -> ValueId {
        self.output
    }
}

/// Output operation: circuit output, consumes one value.
pub(super) struct OutputOperation {
    /// The input value.
    input: ValueId,
}

impl OutputOperation {
    /// Get the input value.
    pub(super) fn get_input(&self) -> ValueId {
        self.input
    }
}

/// A specific usage of a value.
#[derive(Clone, Copy, Debug)]
pub(super) struct Usage {
    /// Who consumes this value.
    pub consumer: Consumer,
    /// Which input port on the consumer.
    pub port: PortId,
    /// Access mode of the value.
    pub mode: Ownership,
}

/// What consumes a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Consumer {
    /// Used by a gate.
    Gate(GateId),
    /// Used by a clone.
    Clone(CloneId),
    /// Used by a drop.
    Drop(DropId),
    /// Circuit output.
    Output(OutputId),
}

impl TryFrom<Operation> for Consumer {
    type Error = Error;

    fn try_from(value: Operation) -> Result<Self> {
        match value {
            Operation::Gate(id) => Ok(Consumer::Gate(id)),
            Operation::Clone(id) => Ok(Consumer::Clone(id)),
            Operation::Drop(id) => Ok(Consumer::Drop(id)),
            Operation::Output(id) => Ok(Consumer::Output(id)),
            _ => Err(Error::BadOperationConversion(value)),
        }
    }
}

/// An SSA value: defined exactly once, consumed exactly once.
pub(super) struct Value<G: Gate> {
    /// Who produces this value.
    pub producer: Producer,
    /// Which output port of the producer.
    pub port: PortId,
    /// All uses of this value.
    pub uses: Vec<Usage>,
    /// Type of the value.
    pub value_type: G::Operand,
}

impl<G: Gate> Value<G> {
    /// Get the producer of this value.
    pub(super) fn get_producer(&self) -> Producer {
        self.producer
    }

    /// Get the output port of the producer.
    pub(super) fn get_port(&self) -> PortId {
        self.port
    }

    /// Get all uses of this value.
    pub(super) fn get_uses(&self) -> &[Usage] {
        &self.uses
    }

    /// Check if this value has exactly one Move consumer.
    pub(super) fn has_single_move(&self) -> bool {
        self.uses
            .iter()
            .filter(|u| u.mode == Ownership::Move)
            .count()
            == 1
    }

    /// Get the the consumer, if exactly one exists.
    pub(super) fn get_move_consumer(&self) -> Option<&Usage> {
        let moves: Vec<_> = self
            .uses
            .iter()
            .filter(|u| u.mode == Ownership::Move)
            .collect();
        if moves.len() == 1 {
            Some(moves[0])
        } else {
            None
        }
    }

    /// Get all borrow consumers.
    pub(super) fn get_borrow_consumers(&self) -> impl Iterator<Item = &Usage> {
        self.uses.iter().filter(|u| u.mode == Ownership::Borrow)
    }

    /// Get the type of this value.
    pub(super) fn get_type(&self) -> G::Operand {
        self.value_type
    }
}

/// What produces a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Producer {
    /// External circuit input.
    Input(InputId),
    /// Produced by a gate.
    Gate(GateId),
    /// Produced by a clone.
    Clone(CloneId),
}

impl TryFrom<Operation> for Producer {
    type Error = Error;

    fn try_from(value: Operation) -> Result<Self> {
        match value {
            Operation::Input(id) => Ok(Producer::Input(id)),
            Operation::Gate(id) => Ok(Producer::Gate(id)),
            Operation::Clone(id) => Ok(Producer::Clone(id)),
            _ => Err(Error::BadOperationConversion(value)),
        }
    }
}

/// A schedulable operation in the circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Operation {
    /// Circuit input.
    Input(InputId),
    /// A gate computation.
    Gate(GateId),
    /// A clone operation.
    Clone(CloneId),
    /// A drop operation.
    Drop(DropId),
    /// A circuit output.
    Output(OutputId),
}

impl From<Consumer> for Operation {
    fn from(consumer: Consumer) -> Self {
        match consumer {
            Consumer::Gate(id) => Operation::Gate(id),
            Consumer::Clone(id) => Operation::Clone(id),
            Consumer::Drop(id) => Operation::Drop(id),
            Consumer::Output(id) => Operation::Output(id),
        }
    }
}

impl From<Producer> for Operation {
    fn from(producer: Producer) -> Self {
        match producer {
            Producer::Input(id) => Operation::Input(id),
            Producer::Gate(id) => Operation::Gate(id),
            Producer::Clone(id) => Operation::Clone(id),
        }
    }
}

/// A circuit in Linear SSA form.
pub(super) struct Circuit<G: Gate> {
    /// All gates, indexed by GateId.
    gates: Vec<GateOperation<G>>,
    /// All clones, indexed by CloneId.
    clones: Vec<CloneOperation>,
    /// All drops, indexed by DropId.
    drops: Vec<DropOperation>,
    /// Circuit inputs, indexed by InputId.
    inputs: Vec<InputOperation>,
    /// Circuit outputs, indexed by OutputId.
    outputs: Vec<OutputOperation>,
    /// All values, indexed by ValueId.
    values: Vec<Value<G>>,
}

impl<G: Gate> Circuit<G> {
    /// Create a new empty circuit.
    pub(super) fn new() -> Self {
        Self {
            gates: Vec::new(),
            clones: Vec::new(),
            drops: Vec::new(),
            values: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Create a new value from a producer and port.
    fn create_value(&mut self, producer: Producer, port: PortId, ty: G::Operand) -> ValueId {
        let id = ValueId::new(self.values.len());
        self.values.push(Value {
            producer,
            port,
            uses: Vec::new(),
            value_type: ty,
        });
        id
    }

    /// Record the use of a value.
    fn record_use(&mut self, value: ValueId, consumer: Consumer, port: PortId, mode: Ownership) {
        self.values[value.index()].uses.push(Usage {
            consumer,
            port,
            mode,
        });
    }

    /// Get all move usages of a value.
    pub(super) fn get_move_uses(&self, value: ValueId) -> Vec<Usage> {
        self.values[value.index()]
            .uses
            .iter()
            .filter(|u| u.mode == Ownership::Move)
            .copied()
            .collect()
    }

    /// Rewire a use from one value to another.
    /// Finds the usage matching (consumer, port) on old_value and moves it to new_value.
    pub(super) fn rewire_use(
        &mut self,
        old_value: ValueId,
        new_value: ValueId,
        consumer: Consumer,
        port: PortId,
    ) {
        // Remove usage from old value.
        let old_uses = &mut self.values[old_value.index()].uses;
        if let Some(pos) = old_uses
            .iter()
            .position(|u| u.consumer == consumer && u.port == port)
        {
            let usage = old_uses.remove(pos);
            // Add usage to new value.
            self.values[new_value.index()].uses.push(usage);
        }
    }

    /// Create a circuit input.
    pub(super) fn add_input(&mut self, value_type: G::Operand) -> (InputId, ValueId) {
        let input_id = InputId::new(self.inputs.len());
        let value_id = self.create_value(Producer::Input(input_id), PortId::new(0), value_type);
        self.inputs.push(InputOperation { output: value_id });
        (input_id, value_id)
    }

    /// Mark a value as a circuit output.
    pub(super) fn add_output(&mut self, value: ValueId) -> OutputId {
        let output_id = OutputId::new(self.outputs.len());
        self.record_use(
            value,
            Consumer::Output(output_id),
            PortId::new(0),
            Ownership::Move,
        );
        self.outputs.push(OutputOperation { input: value });
        output_id
    }

    /// Add a gate.
    pub(super) fn add_gate(
        &mut self,
        gate: G,
        inputs: Vec<ValueId>,
    ) -> Result<(GateId, Vec<ValueId>)> {
        let expected = gate.input_count();
        if inputs.len() != expected {
            return Err(Error::WrongInputCount {
                expected,
                got: inputs.len(),
            });
        }

        // Pre-compute output types (may fail).
        let output_count = gate.output_count();
        let mut output_types = Vec::with_capacity(output_count);
        for p in 0..output_count {
            output_types.push(gate.output_type(p)?);
        }

        // Pre-compute access modes and validate input types.
        let mut access_modes = Vec::with_capacity(inputs.len());
        let gate_id = GateId::new(self.gates.len());
        for (idx, &v) in inputs.iter().enumerate() {
            let expected_ty = gate.input_type(idx)?;
            let actual_ty = self.values[v.index()].value_type;
            if expected_ty != actual_ty {
                return Err(Error::TypeMismatch {
                    gate: gate_id,
                    port: idx,
                });
            }
            access_modes.push(gate.access_mode(idx)?);
        }

        // Create output values.
        let mut outputs = Vec::with_capacity(output_count);
        for (p, ty) in output_types.into_iter().enumerate() {
            let value_id = self.create_value(Producer::Gate(gate_id), PortId::new(p), ty);
            outputs.push(value_id);
        }

        // Record input uses.
        for (idx, (&v, mode)) in inputs.iter().zip(access_modes).enumerate() {
            let port = PortId::new(idx);
            self.record_use(v, Consumer::Gate(gate_id), port, mode);
        }

        self.gates.push(GateOperation {
            gate,
            inputs,
            outputs: outputs.clone(),
        });

        Ok((gate_id, outputs))
    }

    /// Clone a value into N copies.
    pub(super) fn add_clone(&mut self, input: ValueId, count: usize) -> (CloneId, Vec<ValueId>) {
        let clone_id = CloneId::new(self.clones.len());

        // Clone preserves the input's type.
        let ty = self.values[input.index()].value_type;

        // Create outputs.
        let outputs: Vec<_> = (0..count)
            .map(|p| self.create_value(Producer::Clone(clone_id), PortId::new(p), ty))
            .collect();

        // Clone borrows the input.
        self.record_use(
            input,
            Consumer::Clone(clone_id),
            PortId::new(0),
            Ownership::Borrow,
        );

        self.clones.push(CloneOperation {
            input,
            outputs: outputs.clone(),
        });

        (clone_id, outputs)
    }

    /// Drop a value.
    pub(super) fn add_drop(&mut self, input: ValueId) -> DropId {
        let drop_id = DropId::new(self.drops.len());

        // Drop moves the input.
        self.record_use(
            input,
            Consumer::Drop(drop_id),
            PortId::new(0),
            Ownership::Move,
        );

        self.drops.push(DropOperation { input });

        drop_id
    }

    /// Get a gate by id.
    pub(super) fn gate_op(&self, id: GateId) -> Result<&GateOperation<G>> {
        self.gates.get(id.index()).ok_or(Error::GateNotFound(id))
    }

    /// Get a clone by id.
    pub(super) fn clone_op(&self, id: CloneId) -> Result<&CloneOperation> {
        self.clones.get(id.index()).ok_or(Error::CloneNotFound(id))
    }

    /// Get a drop by id.
    pub(super) fn drop_op(&self, id: DropId) -> Result<&DropOperation> {
        self.drops.get(id.index()).ok_or(Error::DropNotFound(id))
    }

    /// Get a input by id.
    pub(super) fn input_op(&self, id: InputId) -> Result<&InputOperation> {
        self.inputs.get(id.index()).ok_or(Error::InputNotFound(id))
    }

    /// Get a output by id.
    pub(super) fn output_op(&self, id: OutputId) -> Result<&OutputOperation> {
        self.outputs
            .get(id.index())
            .ok_or(Error::OutputNotFound(id))
    }

    /// Get a value by id.
    pub(super) fn value(&self, id: ValueId) -> Result<&Value<G>> {
        self.values.get(id.index()).ok_or(Error::ValueNotFound(id))
    }

    /// Number of gates.
    pub(super) fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Number of clones.
    pub(super) fn clone_count(&self) -> usize {
        self.clones.len()
    }

    /// Number of drops.
    pub(super) fn drop_count(&self) -> usize {
        self.drops.len()
    }

    /// Number of circuit inputs.
    pub(super) fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Number of circuit outputs.
    pub(super) fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Number of values.
    pub(super) fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Iterate over all gates.
    pub(super) fn all_gates(&self) -> impl Iterator<Item = (GateId, &GateOperation<G>)> {
        self.gates
            .iter()
            .enumerate()
            .map(|(i, g)| (GateId::new(i), g))
    }

    /// Iterate over all clones.
    pub(super) fn all_clones(&self) -> impl Iterator<Item = (CloneId, &CloneOperation)> {
        self.clones
            .iter()
            .enumerate()
            .map(|(i, c)| (CloneId::new(i), c))
    }

    /// Iterate over all drops.
    pub(super) fn all_drops(&self) -> impl Iterator<Item = (DropId, &DropOperation)> {
        self.drops
            .iter()
            .enumerate()
            .map(|(i, d)| (DropId::new(i), d))
    }

    /// Iterate over all circuit inputs.
    pub(super) fn all_inputs(&self) -> impl Iterator<Item = (InputId, &InputOperation)> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, op)| (InputId::new(i), op))
    }

    /// Iterate over all circuit outputs.
    pub(super) fn all_outputs(&self) -> impl Iterator<Item = (OutputId, &OutputOperation)> {
        self.outputs
            .iter()
            .enumerate()
            .map(|(i, op)| (OutputId::new(i), op))
    }

    /// Iterate over all values.
    pub(super) fn all_values(&self) -> impl Iterator<Item = (ValueId, &Value<G>)> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, v)| (ValueId::new(i), v))
    }

    /// Iterate over all operations in the circuit.
    pub(super) fn all_operations(&self) -> impl Iterator<Item = Operation> + '_ {
        self.all_inputs()
            .map(|(id, _)| Operation::Input(id))
            .chain(self.all_gates().map(|(id, _)| Operation::Gate(id)))
            .chain(self.all_clones().map(|(id, _)| Operation::Clone(id)))
            .chain(self.all_drops().map(|(id, _)| Operation::Drop(id)))
            .chain(self.all_outputs().map(|(id, _)| Operation::Output(id)))
    }

    /// Iterate over values produced by an operation.
    pub(super) fn produced_values(&self, op: Operation) -> impl Iterator<Item = ValueId> {
        let (input_val, gate_vals, clone_vals): (Option<ValueId>, &[ValueId], &[ValueId]) = match op
        {
            Operation::Input(id) => {
                let val = self.inputs.get(id.index()).map(|i| i.output);
                (val, &[], &[])
            }
            Operation::Gate(id) => {
                let vals = self
                    .gates
                    .get(id.index())
                    .map(|g| g.outputs.as_slice())
                    .unwrap_or(&[]);
                (None, vals, &[])
            }
            Operation::Clone(id) => {
                let vals = self
                    .clones
                    .get(id.index())
                    .map(|c| c.outputs.as_slice())
                    .unwrap_or(&[]);
                (None, &[], vals)
            }
            Operation::Drop(_) | Operation::Output(_) => (None, &[], &[]),
        };
        input_val
            .into_iter()
            .chain(gate_vals.iter().copied())
            .chain(clone_vals.iter().copied())
    }
}

impl<G: Gate> Default for Circuit<G> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the tuple of all circuit components.
pub(super) type CircuitParts<G> = (
    Vec<GateOperation<G>>,
    Vec<CloneOperation>,
    Vec<DropOperation>,
    Vec<InputOperation>,
    Vec<OutputOperation>,
    Vec<Value<G>>,
);

impl<G: Gate> Circuit<G> {
    /// Consume the circuit and return ownership of all components.
    pub(super) fn into_parts(self) -> CircuitParts<G> {
        (
            self.gates,
            self.clones,
            self.drops,
            self.inputs,
            self.outputs,
            self.values,
        )
    }
}
