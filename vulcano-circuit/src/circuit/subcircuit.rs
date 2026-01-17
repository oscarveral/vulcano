//! Subcircuit definitions.
//!
//! This module defines a subcircuit as a sort of static single
//! assignment form of values produced and consumed or borrowed
//! in order to create or output other values.

use std::collections::HashMap;

use vulcano_arena::{Arena, Key};

use crate::{
    circuit::{
        operations::{
            Consumer, Operation, PortId, Producer,
            clone::{CloneId, CloneOp},
            drop::{DropId, DropOp},
            gate::{GateId, GateOp},
            input::{InputId, InputOp},
            output::{OutputId, OutputOp},
        },
        value::{Destination, Origin, Ownership, Value, ValueId},
    },
    error::{Error, Result},
    gate::Gate,
};

/// Handle identifying a specific subcircuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CircuitId(Key);

impl CircuitId {
    /// Create a new circuit id from a key.
    pub fn new(key: Key) -> Self {
        Self(key)
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.0
    }
}

/// A subcircuit in Linear SSA form.
pub struct Subcircuit<G: Gate> {
    /// Id of this circuit.
    id: CircuitId,
    /// All gates.
    gates: Arena<GateOp<G>>,
    /// All clones.
    clones: Arena<CloneOp>,
    /// All drops.
    drops: Arena<DropOp>,
    /// Circuit inputs.
    inputs: Arena<InputOp>,
    /// Circuit outputs.
    outputs: Arena<OutputOp>,
    /// All values.
    values: Arena<Value<G>>,
}

impl<G: Gate> Subcircuit<G> {
    /// Create a new empty circuit.
    pub fn new(id: CircuitId) -> Self {
        Self {
            id,
            gates: Arena::new(),
            clones: Arena::new(),
            drops: Arena::new(),
            values: Arena::new(),
            inputs: Arena::new(),
            outputs: Arena::new(),
        }
    }

    /// Get the circuit id.
    pub fn id(&self) -> CircuitId {
        self.id
    }

    /// Validate that a value belongs to this circuit and exists.
    fn validate_value(&self, value: ValueId) -> Result<()> {
        if value.circuit() != self.id {
            return Err(Error::CircuitIdMismatch(value, self.id));
        }
        if self.values.get(value.key()).is_none() {
            return Err(Error::ValueNotFound(value));
        }
        Ok(())
    }
    /// Add a destination to a value.
    fn link_destination(&mut self, value: ValueId, destination: Destination) -> Result<()> {
        let value_ref = self
            .values
            .get_mut(value.key())
            .ok_or(Error::ValueNotFound(value))?;
        value_ref.add_destination(destination);
        Ok(())
    }

    /// Add a new input to the circuit.
    pub fn add_input(&mut self, operand: G::Operand) -> Result<(InputId, ValueId)> {
        // Reserve keys and create IDs.
        let input_id = InputId::new(self.id, self.inputs.reserve());
        let value_id = ValueId::new(self.id, self.values.reserve());

        // Create the input and its produced value.
        let input = InputOp::new(value_id);
        let origin = Origin::new(Producer::Input(input_id), PortId::new(0));
        let value = Value::new(origin, operand);

        // Helper to cleanup on failure.
        let cleanup = |s: &mut Self| {
            s.inputs.remove(input_id.key());
            s.values.remove(value_id.key());
        };

        // Fill arenas.
        if self.inputs.fill(input_id.key(), input).is_err() {
            cleanup(self);
            return Err(Error::FailedToCreateInput(input_id));
        }
        if self.values.fill(value_id.key(), value).is_err() {
            cleanup(self);
            return Err(Error::FailedToCreateValue(value_id));
        }

        Ok((input_id, value_id))
    }

    /// Add a new output to the circuit.
    pub fn add_output(&mut self, value: ValueId) -> Result<OutputId> {
        // Validate the input value exists.
        self.validate_value(value)?;

        // Reserve key and create output.
        let output_id = OutputId::new(self.id, self.outputs.reserve());
        let output = OutputOp::new(value);

        // Fill arena.
        if self.outputs.fill(output_id.key(), output).is_err() {
            self.outputs.remove(output_id.key());
            return Err(Error::FailedToCreateOutput(output_id));
        }

        // Link the value to this consumer.
        let destination =
            Destination::new(Consumer::Output(output_id), PortId::new(0), Ownership::Move);
        if let Err(err) = self.link_destination(value, destination) {
            self.outputs.remove(output_id.key());
            return Err(err);
        }

        Ok(output_id)
    }

    /// Add a new drop to the circuit.
    pub fn add_drop(&mut self, value: ValueId) -> Result<DropId> {
        // Validate the input value exists.
        self.validate_value(value)?;

        // Reserve key and create drop.
        let drop_id = DropId::new(self.id, self.drops.reserve());
        let drop_op = DropOp::new(value);

        // Fill arena.
        if self.drops.fill(drop_id.key(), drop_op).is_err() {
            self.drops.remove(drop_id.key());
            return Err(Error::FailedToCreateDrop(drop_id));
        }

        // Link the value to this consumer.
        let destination =
            Destination::new(Consumer::Drop(drop_id), PortId::new(0), Ownership::Move);
        if let Err(err) = self.link_destination(value, destination) {
            self.drops.remove(drop_id.key());
            return Err(err);
        }

        Ok(drop_id)
    }

    /// Add a new clone operation to the circuit.
    pub fn add_clone(
        &mut self,
        value: ValueId,
        quantity: usize,
    ) -> Result<(CloneId, Vec<ValueId>)> {
        // Validate quantity.
        if quantity == 0 {
            return Err(Error::InvalidCloneQuantity);
        }

        // Validate the input value exists and get its type.
        let value_type = self.value(value)?.get_type();

        // Reserve keys and create IDs.
        let clone_id = CloneId::new(self.id, self.clones.reserve());
        let output_ids: Vec<ValueId> = (0..quantity)
            .map(|_| ValueId::new(self.id, self.values.reserve()))
            .collect();

        // Helper to cleanup on failure.
        let cleanup = |s: &mut Self, destination_linked: bool| {
            // Remove the destination from the input value if it was linked.
            if destination_linked && let Some(v) = s.values.get_mut(value.key()) {
                v.remove_destinations_for(Consumer::Clone(clone_id));
            }
            // Remove the clone and output values.
            s.clones.remove(clone_id.key());
            for output_id in &output_ids {
                s.values.remove(output_id.key());
            }
        };

        // Create the clone operation.
        let clone_op = CloneOp::new(value, output_ids.clone());

        // Create output values.
        let producer = Producer::Clone(clone_id);
        let output_values: Vec<Value<G>> = (0..quantity)
            .map(|i| {
                let origin = Origin::new(producer, PortId::new(i));
                Value::new(origin, value_type)
            })
            .collect();

        // Fill clone arena.
        if self.clones.fill(clone_id.key(), clone_op).is_err() {
            cleanup(self, false);
            return Err(Error::FailedToCreateClone(clone_id));
        }

        // Fill value arenas.
        for (output_id, output_value) in output_ids.iter().zip(output_values) {
            if self.values.fill(output_id.key(), output_value).is_err() {
                cleanup(self, false);
                return Err(Error::FailedToCreateValue(*output_id));
            }
        }

        // Link the input value to this consumer (borrow).
        let destination =
            Destination::new(Consumer::Clone(clone_id), PortId::new(0), Ownership::Borrow);
        if let Err(err) = self.link_destination(value, destination) {
            cleanup(self, false);
            return Err(err);
        }

        Ok((clone_id, output_ids))
    }

    /// Add a new gate to the circuit.
    pub fn add_gate(&mut self, inputs: &[ValueId], gate: G) -> Result<(GateId, Vec<ValueId>)> {
        // Validate all input values exist.
        for &input in inputs {
            self.validate_value(input)?;
        }

        // Reserve keys and create IDs.
        let gate_id = GateId::new(self.id, self.gates.reserve());
        let output_count = gate.output_count();
        let output_ids: Vec<ValueId> = (0..output_count)
            .map(|_| ValueId::new(self.id, self.values.reserve()))
            .collect();

        // Helper to cleanup on failure.
        let cleanup = |s: &mut Self, linked_inputs: &[ValueId]| {
            // Remove destinations we already linked.
            let consumer = Consumer::Gate(gate_id);
            for input in linked_inputs {
                if let Some(v) = s.values.get_mut(input.key()) {
                    v.remove_destinations_for(consumer);
                }
            }
            // Remove the gate and output values.
            s.gates.remove(gate_id.key());
            for output_id in &output_ids {
                s.values.remove(output_id.key());
            }
        };

        // Create the gate operation.
        let gate_op = match GateOp::new(gate, inputs.to_vec(), output_ids.clone()) {
            Ok(op) => op,
            Err(err) => {
                cleanup(self, &[]);
                return Err(err);
            }
        };

        // Create output values.
        let producer = Producer::Gate(gate_id);
        let output_values: Vec<Value<G>> = match (0..output_count)
            .map(|i| {
                let origin = Origin::new(producer, PortId::new(i));
                let operand = gate.output_type(i)?;
                Ok(Value::new(origin, operand))
            })
            .collect::<Result<Vec<_>>>()
        {
            Ok(values) => values,
            Err(err) => {
                cleanup(self, &[]);
                return Err(err);
            }
        };

        // Fill gate arena.
        if self.gates.fill(gate_id.key(), gate_op).is_err() {
            cleanup(self, &[]);
            return Err(Error::FailedToCreateGate(gate_id));
        }

        // Fill value arenas.
        for (output_id, value) in output_ids.iter().zip(output_values) {
            if self.values.fill(output_id.key(), value).is_err() {
                cleanup(self, &[]);
                return Err(Error::FailedToCreateValue(*output_id));
            }
        }

        // Link each input value to this consumer.
        let access_modes = match gate.access_modes() {
            Ok(modes) => modes,
            Err(err) => {
                cleanup(self, &[]);
                return Err(err);
            }
        };
        let mut linked_inputs: Vec<ValueId> = Vec::new();
        for (port_idx, (&input, mode)) in inputs.iter().zip(access_modes).enumerate() {
            let destination =
                Destination::new(Consumer::Gate(gate_id), PortId::new(port_idx), mode);
            if let Err(err) = self.link_destination(input, destination) {
                cleanup(self, &linked_inputs);
                return Err(err);
            }
            linked_inputs.push(input);
        }

        Ok((gate_id, output_ids))
    }

    /// Iterate over all values in the circuit.
    pub fn all_values(&self) -> impl Iterator<Item = (ValueId, &Value<G>)> {
        self.values
            .iter()
            .map(|(id, value)| (ValueId::new(self.id, id), value))
    }

    /// Iterate over all operations in the circuit.
    pub fn all_operations(&self) -> impl Iterator<Item = Operation> + '_ {
        let inputs = self
            .inputs
            .iter()
            .map(|(key, _)| Operation::Input(InputId::new(self.id, key)));
        let gates = self
            .gates
            .iter()
            .map(|(key, _)| Operation::Gate(GateId::new(self.id, key)));
        let clones = self
            .clones
            .iter()
            .map(|(key, _)| Operation::Clone(CloneId::new(self.id, key)));
        let drops = self
            .drops
            .iter()
            .map(|(key, _)| Operation::Drop(DropId::new(self.id, key)));
        let outputs = self
            .outputs
            .iter()
            .map(|(key, _)| Operation::Output(OutputId::new(self.id, key)));
        inputs
            .chain(gates)
            .chain(clones)
            .chain(drops)
            .chain(outputs)
    }

    /// Iterate over all outputs in the circuit.
    pub fn all_outputs(&self) -> impl Iterator<Item = (OutputId, &OutputOp)> {
        self.outputs
            .iter()
            .map(|(key, op)| (OutputId::new(self.id, key), op))
    }

    /// Iterate over all inputs in the circuit.
    pub fn all_inputs(&self) -> impl Iterator<Item = (InputId, &InputOp)> {
        self.inputs
            .iter()
            .map(|(key, op)| (InputId::new(self.id, key), op))
    }

    /// Iterate over all gates in the circuit.
    pub fn all_gates(&self) -> impl Iterator<Item = (GateId, &GateOp<G>)> {
        self.gates
            .iter()
            .map(|(key, op)| (GateId::new(self.id, key), op))
    }

    /// Iterate over all clones in the circuit.
    pub fn all_clones(&self) -> impl Iterator<Item = (CloneId, &CloneOp)> {
        self.clones
            .iter()
            .map(|(key, op)| (CloneId::new(self.id, key), op))
    }

    /// Iterate over all drops in the circuit.
    pub fn all_drops(&self) -> impl Iterator<Item = (DropId, &DropOp)> {
        self.drops
            .iter()
            .map(|(key, op)| (DropId::new(self.id, key), op))
    }

    /// Get an immutable reference to a value, validating circuit ownership.
    pub fn value(&self, value: ValueId) -> Result<&Value<G>> {
        self.validate_value(value)?;
        self.values
            .get(value.key())
            .ok_or(Error::ValueNotFound(value))
    }

    /// Get a gate operation by id.
    pub fn gate_op(&self, id: GateId) -> Result<&GateOp<G>> {
        if id.circuit() != self.id {
            return Err(Error::GateNotFound(id));
        }
        self.gates.get(id.key()).ok_or(Error::GateNotFound(id))
    }

    /// Get a clone operation by id.
    pub fn clone_op(&self, id: CloneId) -> Result<&CloneOp> {
        if id.circuit() != self.id {
            return Err(Error::CloneNotFound(id));
        }
        self.clones.get(id.key()).ok_or(Error::CloneNotFound(id))
    }

    /// Get a drop operation by id.
    pub fn drop_op(&self, id: DropId) -> Result<&DropOp> {
        if id.circuit() != self.id {
            return Err(Error::DropNotFound(id));
        }
        self.drops.get(id.key()).ok_or(Error::DropNotFound(id))
    }

    /// Get an input operation by id.
    pub fn input_op(&self, id: InputId) -> Result<&InputOp> {
        if id.circuit() != self.id {
            return Err(Error::InputNotFound(id));
        }
        self.inputs.get(id.key()).ok_or(Error::InputNotFound(id))
    }

    /// Get an output operation by id.
    pub fn output_op(&self, id: OutputId) -> Result<&OutputOp> {
        if id.circuit() != self.id {
            return Err(Error::OutputNotFound(id));
        }
        self.outputs.get(id.key()).ok_or(Error::OutputNotFound(id))
    }

    /// Get the values produced by an operation.
    pub fn produced_values(&self, op: Operation) -> Result<Vec<ValueId>> {
        match op {
            Operation::Input(id) => {
                let input = self.input_op(id)?;
                Ok(vec![input.get_output()])
            }
            Operation::Gate(id) => {
                let gate = self.gate_op(id)?;
                Ok(gate.get_outputs().to_vec())
            }
            Operation::Clone(id) => {
                let clone = self.clone_op(id)?;
                Ok(clone.get_outputs().to_vec())
            }
            // Drops and outputs don't produce values.
            Operation::Drop(_) | Operation::Output(_) => Ok(vec![]),
        }
    }

    /// Get all move destinations for a value.
    pub fn get_move_destinations(&self, value: ValueId) -> Result<Vec<Destination>> {
        let val = self.value(value)?;
        Ok(val
            .get_destinations()
            .iter()
            .filter(|d| d.get_mode() == Ownership::Move)
            .cloned()
            .collect())
    }

    /// Rewire a destination from one value to another.
    ///
    /// Finds where `old_value` is consumed by the given consumer/port and changes
    /// it to consume `new_value` instead.
    pub fn rewire_destination(
        &mut self,
        old_value: ValueId,
        new_value: ValueId,
        consumer: Consumer,
        port: PortId,
    ) -> Result<()> {
        // Validate both values exist.
        self.validate_value(old_value)?;
        self.validate_value(new_value)?;

        // Find and remove the destination from old_value.
        let old_val = self
            .values
            .get_mut(old_value.key())
            .ok_or(Error::ValueNotFound(old_value))?;

        let dest_idx = old_val
            .get_destinations()
            .iter()
            .position(|d| d.get_consumer() == consumer && d.get_port() == port)
            .ok_or(Error::DestinationNotFound(old_value, consumer, port))?;

        let destination = old_val.get_destinations()[dest_idx];
        old_val.remove_destination_at(dest_idx);

        // Add the destination to new_value.
        let new_val = self
            .values
            .get_mut(new_value.key())
            .ok_or(Error::ValueNotFound(new_value))?;
        new_val.add_destination(destination);

        // Update the consumer's input to point to new_value.
        match consumer {
            Consumer::Gate(gate_id) => {
                let gate = self
                    .gates
                    .get_mut(gate_id.key())
                    .ok_or(Error::GateNotFound(gate_id))?;
                gate.set_input(port.index(), new_value)?;
            }
            Consumer::Drop(drop_id) => {
                let drop_op = self
                    .drops
                    .get_mut(drop_id.key())
                    .ok_or(Error::DropNotFound(drop_id))?;
                drop_op.set_input(new_value);
            }
            Consumer::Output(output_id) => {
                let output = self
                    .outputs
                    .get_mut(output_id.key())
                    .ok_or(Error::OutputNotFound(output_id))?;
                output.set_input(new_value);
            }
            Consumer::Clone(clone_id) => {
                let clone_op = self
                    .clones
                    .get_mut(clone_id.key())
                    .ok_or(Error::CloneNotFound(clone_id))?;
                clone_op.set_input(new_value);
            }
        }

        Ok(())
    }

    /// Remove a gate without checking references.
    pub fn remove_gate_unchecked(&mut self, id: GateId) {
        self.gates.remove(id.key());
    }

    /// Remove a clone without checking references.
    pub fn remove_clone_unchecked(&mut self, id: CloneId) {
        self.clones.remove(id.key());
    }

    /// Remove a drop without checking references.
    pub fn remove_drop_unchecked(&mut self, id: DropId) {
        self.drops.remove(id.key());
    }

    /// Remove an input without checking references.
    pub fn remove_input_unchecked(&mut self, id: InputId) {
        self.inputs.remove(id.key());
    }

    /// Remove an output without checking references.
    pub fn remove_output_unchecked(&mut self, id: OutputId) {
        self.outputs.remove(id.key());
    }

    /// Remove a value without checking references.
    pub fn remove_value_unchecked(&mut self, id: ValueId) {
        self.values.remove(id.key());
    }

    /// Check if the subcircuit is empty.
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
            && self.clones.is_empty()
            && self.drops.is_empty()
            && self.inputs.is_empty()
            && self.outputs.is_empty()
            && self.values.is_empty()
    }

    /// Split off a subset of operations into a new subcircuit.
    ///
    /// Extracts the specified operations and their associated values from this
    /// subcircuit and creates a new subcircuit containing them. The new subcircuit
    /// is assigned the provided ID.
    ///
    /// Operations must form a complete connected component.
    pub fn split(&mut self, new_id: CircuitId, ops_to_extract: &[Operation]) -> Result<Self> {
        // Create the new subcircuit.
        let mut new_subcircuit = Subcircuit::new(new_id);

        // Mapping from old value keys to new ValueIds.
        let mut value_map: HashMap<Key, ValueId> = HashMap::new();

        // Phase 1: Extract inputs (they produce values with no dependencies).
        for &op in ops_to_extract {
            if let Operation::Input(input_id) = op
                && let Some(input_op) = self.inputs.remove(input_id.key())
            {
                let old_value_id = input_op.get_output();
                if let Some(old_value) = self.values.remove(old_value_id.key()) {
                    let operand = old_value.get_type();
                    let (_, new_value_id) = new_subcircuit.add_input(operand)?;
                    value_map.insert(old_value_id.key(), new_value_id);
                }
            }
        }

        // Phase 2: Extract gates (topological order not needed if component is valid).
        // We may need multiple passes for gates that depend on other gates.
        let mut remaining_gates: Vec<GateId> = ops_to_extract
            .iter()
            .filter_map(|op| {
                if let Operation::Gate(id) = op {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        let mut progress = true;
        while progress && !remaining_gates.is_empty() {
            progress = false;
            remaining_gates.retain(|&gate_id| {
                let gate_op = match self.gates.get(gate_id.key()) {
                    Some(g) => g,
                    None => return false, // Already processed.
                };

                // Check if all inputs are mapped.
                let all_inputs_ready = gate_op
                    .get_inputs()
                    .iter()
                    .all(|v| value_map.contains_key(&v.key()));

                if !all_inputs_ready {
                    return true; // Keep for next iteration.
                }

                // Extract the gate.
                let gate_op = self.gates.remove(gate_id.key()).unwrap();
                let gate = *gate_op.get_gate();

                // Map inputs.
                let new_inputs: Vec<ValueId> = gate_op
                    .get_inputs()
                    .iter()
                    .map(|v| value_map[&v.key()])
                    .collect();

                // Collect output types and remove old values.
                let output_info: Vec<(Key, G::Operand)> = gate_op
                    .get_outputs()
                    .iter()
                    .filter_map(|v| {
                        let old_value = self.values.remove(v.key())?;
                        Some((v.key(), old_value.get_type()))
                    })
                    .collect();

                // Add gate to new subcircuit.
                if let Ok((_, new_outputs)) = new_subcircuit.add_gate(&new_inputs, gate) {
                    for ((old_key, _), new_value_id) in output_info.iter().zip(new_outputs.iter()) {
                        value_map.insert(*old_key, *new_value_id);
                    }
                }

                progress = true;
                false // Processed, remove from remaining.
            });
        }

        // Phase 3: Extract clones.
        for &op in ops_to_extract {
            if let Operation::Clone(clone_id) = op
                && let Some(clone_op) = self.clones.remove(clone_id.key())
            {
                let input_key = clone_op.get_input().key();
                if let Some(&new_input) = value_map.get(&input_key) {
                    let quantity = clone_op.output_count();

                    // Collect old output keys.
                    let old_output_keys: Vec<Key> =
                        clone_op.get_outputs().iter().map(|v| v.key()).collect();

                    // Remove old values.
                    for key in &old_output_keys {
                        self.values.remove(*key);
                    }

                    // Add clone to new subcircuit.
                    if let Ok((_, new_outputs)) = new_subcircuit.add_clone(new_input, quantity) {
                        for (old_key, new_value_id) in
                            old_output_keys.iter().zip(new_outputs.iter())
                        {
                            value_map.insert(*old_key, *new_value_id);
                        }
                    }
                }
            }
        }

        // Phase 4: Extract drops.
        for &op in ops_to_extract {
            if let Operation::Drop(drop_id) = op
                && let Some(drop_op) = self.drops.remove(drop_id.key())
            {
                let input_key = drop_op.get_input().key();
                if let Some(&new_input) = value_map.get(&input_key) {
                    let _ = new_subcircuit.add_drop(new_input);
                }
            }
        }

        // Phase 5: Extract outputs.
        for &op in ops_to_extract {
            if let Operation::Output(output_id) = op
                && let Some(output_op) = self.outputs.remove(output_id.key())
            {
                let input_key = output_op.get_input().key();
                if let Some(&new_input) = value_map.get(&input_key) {
                    let _ = new_subcircuit.add_output(new_input);
                }
            }
        }

        Ok(new_subcircuit)
    }
}
