//! Scheduled Order Analysis
//!
//! Computes an optimal execution order that minimizes register pressure.
//! Uses priority-based Kahn's algorithm with these rules:
//! - Outputs and Drops: High priority.
//! - Inputs and Clones: Low priority.
//! - Gates: Indiferent.

use std::collections::{BinaryHeap, HashMap};

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{
        Circuit,
        operations::Operation,
        subcircuit::{CircuitId, Subcircuit},
    },
    error::{Error, Result},
    gate::Gate,
};

/// Scheduling priority for operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority (Inputs, Clones).
    Low = 0,
    /// Normal priority (Gates).
    Normal = 1,
    /// High priority (Outputs, Drops).
    High = 2,
}

impl Priority {
    /// Get the scheduling priority for an operation.
    pub fn for_operation(op: &Operation) -> Self {
        match op {
            Operation::Output(_) | Operation::Drop(_) => Priority::High,
            Operation::Gate(_) => Priority::Normal,
            Operation::Input(_) | Operation::Clone(_) => Priority::Low,
        }
    }
}

/// Priority wrapper for operations in the scheduling queue.
#[derive(Eq, PartialEq)]
struct PrioritizedOp {
    /// Priority of the operation.
    priority: Priority,
    /// Sequence number for deterministic ordering.
    sequence: usize,
    /// Operation to schedule.
    op: Operation,
}

impl Ord for PrioritizedOp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then lower sequence for determinism.
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for PrioritizedOp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Per-subcircuit scheduled order result.
pub struct SubcircuitSchedule {
    /// Operations in optimized execution order.
    order: Vec<Operation>,
    /// Map from operation to its position in the schedule.
    position: HashMap<Operation, usize>,
}

impl SubcircuitSchedule {
    /// Get the operations in scheduled order.
    pub fn operations(&self) -> &[Operation] {
        &self.order
    }

    /// Get the position of an operation in the schedule.
    pub fn position(&self, op: Operation) -> Option<usize> {
        self.position.get(&op).copied()
    }

    /// Iterate over operations in scheduled order.
    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.order.iter()
    }
}

/// Result of scheduled order analysis.
pub struct ScheduledOrder {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitSchedule>,
}

impl ScheduledOrder {
    /// Get the scheduled order for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitSchedule> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitSchedule)> {
        self.results.iter().map(|(&id, schedule)| (id, schedule))
    }
}

impl<G: Gate> Analysis<G> for ScheduledOrder {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let schedule = compute_scheduled_order(subcircuit)?;
            results.insert(subcircuit.id(), schedule);
        }

        Ok(ScheduledOrder { results })
    }
}

/// Compute scheduled order for a single subcircuit using priority-based Kahn's algorithm.
fn compute_scheduled_order<G: Gate>(subcircuit: &Subcircuit<G>) -> Result<SubcircuitSchedule> {
    // Step 1. Initialize in-degree for each operation.
    let mut in_degree: HashMap<Operation, usize> = HashMap::new();

    for op in subcircuit.all_operations() {
        in_degree.insert(op, 0);
    }

    // Step 2. Build edges: each consumer depends on producer.
    for (_, value) in subcircuit.all_values() {
        for usage in value.get_destinations() {
            let consumer_op: Operation = usage.get_consumer().into();
            *in_degree.entry(consumer_op).or_insert(0) += 1;
        }
    }

    // Step 3. Priority-based Kahn's algorithm.
    let mut heap: BinaryHeap<PrioritizedOp> = BinaryHeap::new();
    let mut order: Vec<Operation> = Vec::new();
    let mut position: HashMap<Operation, usize> = HashMap::new();
    let mut sequence = 0usize;

    // Substep A. Start with operations that have no dependencies.
    for (&op, &deg) in &in_degree {
        if deg == 0 {
            heap.push(PrioritizedOp {
                priority: Priority::for_operation(&op),
                sequence,
                op,
            });
            sequence += 1;
        }
    }

    // Substep B. Process operations by priority.
    while let Some(PrioritizedOp { op, .. }) = heap.pop() {
        let pos = order.len();
        position.insert(op, pos);
        order.push(op);

        // Substep C. Reduce in-degree of consumers.
        for value_id in subcircuit.produced_values(op)? {
            let value = subcircuit.value(value_id)?;
            for usage in value.get_destinations() {
                let consumer_op: Operation = usage.get_consumer().into();
                if let Some(deg) = in_degree.get_mut(&consumer_op) {
                    *deg -= 1;
                    if *deg == 0 {
                        heap.push(PrioritizedOp {
                            priority: Priority::for_operation(&consumer_op),
                            sequence,
                            op: consumer_op,
                        });
                        sequence += 1;
                    }
                }
            }
        }
    }

    // Step 4. Check for cycles.
    if order.len() != in_degree.len() {
        let cycle_ops: Vec<Operation> = in_degree
            .into_iter()
            .filter(|(_, deg)| *deg > 0)
            .map(|(op, _)| op)
            .collect();
        return Err(Error::CycleDetected(cycle_ops));
    }

    Ok(SubcircuitSchedule { order, position })
}
