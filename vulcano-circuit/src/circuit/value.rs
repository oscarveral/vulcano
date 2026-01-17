//! Values in the circuit.
//!
//! Values are defined exactly once, consumed exactly once.
//! Values can be borrowed any number of times before being consumed.

use vulcano_arena::Key;

use crate::{
    circuit::{
        operations::{Consumer, PortId, Producer},
        subcircuit::CircuitId,
    },
    gate::Gate,
};

/// Ownership mode for a use of a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ownership {
    /// Value is borrowed. Remains available after use.
    Borrow,
    /// Value is moved. Consumed, no longer available.
    Move,
}

/// A specific usage of a value.
#[derive(Clone, Copy, Debug)]
pub struct Destination {
    /// Who consumes this value.
    consumer: Consumer,
    /// Which input port on the consumer.
    port: PortId,
    /// Access mode of the value.
    mode: Ownership,
}

impl Destination {
    /// Create a new usage.
    pub fn new(consumer: Consumer, port: PortId, mode: Ownership) -> Self {
        Self {
            consumer,
            port,
            mode,
        }
    }

    /// Get the consumer.
    pub fn get_consumer(&self) -> Consumer {
        self.consumer
    }

    /// Get the port.
    pub fn get_port(&self) -> PortId {
        self.port
    }

    /// Get the mode.
    pub fn get_mode(&self) -> Ownership {
        self.mode
    }
}

/// A product of a producer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Origin {
    /// Who produces this value.
    producer: Producer,
    /// Which output port of the producer.
    port: PortId,
}

impl Origin {
    /// Create a new product.
    pub fn new(producer: Producer, port: PortId) -> Self {
        Self { producer, port }
    }

    /// Get the producer.
    pub fn get_producer(&self) -> Producer {
        self.producer
    }

    /// Get the port.
    pub fn get_port(&self) -> PortId {
        self.port
    }
}

/// Handle identifying a value in the circuit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ValueId {
    /// The circuit this value belongs to.
    circuit: CircuitId,
    /// The key of the value.
    value: Key,
}

impl ValueId {
    /// Create a new value id from a key.
    pub fn new(circuit: CircuitId, key: Key) -> Self {
        Self {
            circuit,
            value: key,
        }
    }

    /// Return the underlying key.
    pub fn key(self) -> Key {
        self.value
    }

    /// Return the circuit this value belongs to.
    pub fn circuit(self) -> CircuitId {
        self.circuit
    }
}

/// A circuit value. Defined exactly once, consumed exactly once.
pub struct Value<G: Gate> {
    /// Who produces this value.
    origin: Origin,
    /// Who uses this value.
    destinations: Vec<Destination>,
    /// Type of the value.
    value_type: G::Operand,
}

impl<G: Gate> Value<G> {
    pub fn new(origin: Origin, value_type: G::Operand) -> Self {
        Self {
            origin,
            destinations: Vec::new(),
            value_type,
        }
    }

    /// Get the producer of this value.
    pub fn get_product(&self) -> &Origin {
        &self.origin
    }

    /// Get all uses of this value.
    pub fn get_destinations(&self) -> &[Destination] {
        &self.destinations
    }

    /// Check if this value has exactly one Move consumer.
    pub fn has_single_move(&self) -> bool {
        self.destinations
            .iter()
            .filter(|u| u.mode == Ownership::Move)
            .count()
            == 1
    }

    /// Get the the consumer, if exactly one exists.
    pub fn get_move_consumer(&self) -> Option<&Destination> {
        let moves: Vec<_> = self
            .destinations
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
    pub fn get_borrow_consumers(&self) -> impl Iterator<Item = &Destination> {
        self.destinations
            .iter()
            .filter(|u| u.mode == Ownership::Borrow)
    }

    /// Get the type of this value.
    pub fn get_type(&self) -> G::Operand {
        self.value_type
    }

    /// Add a destination to this value.
    pub fn add_destination(&mut self, destination: Destination) {
        self.destinations.push(destination);
    }

    /// Remove destinations matching a specific consumer.
    pub fn remove_destinations_for(&mut self, consumer: Consumer) {
        self.destinations.retain(|d| d.consumer != consumer);
    }
}
