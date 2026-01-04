//! Liveness Analysis
//!
//! Computes live ranges for each value in the circuit.
//! A live range spans from when a value is produced (birth) to when
//! it is last consumed (death).

use std::collections::HashMap;

use crate::{
    analyzer::{
        analyses::scheduled_order::ScheduledOrder,
        {Analysis, Analyzer},
    },
    circuit::{Circuit, Operation},
    error::{Error, Result},
    gate::Gate,
    handles::ValueId,
};

/// The live range of a value in the schedule.
#[derive(Clone, Copy, Debug)]
pub struct LiveRange {
    /// Schedule position where value becomes live (birth).
    pub birth: usize,
    /// Schedule position where value is no longer needed (death).
    pub death: usize,
}

impl LiveRange {
    /// Check if this range overlaps with another.
    pub fn overlaps(&self, other: &LiveRange) -> bool {
        self.birth < other.death && other.birth < self.death
    }

    /// Length of the live range.
    pub fn len(&self) -> usize {
        self.death.saturating_sub(self.birth)
    }

    /// Check if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.birth >= self.death
    }
}

/// Result of liveness analysis.
pub struct ValueLiveness {
    /// Map from ValueId to its live range.
    live_ranges: HashMap<ValueId, LiveRange>,
}

impl ValueLiveness {
    /// Get the live range for a value.
    pub fn get(&self, value: ValueId) -> Option<&LiveRange> {
        self.live_ranges.get(&value)
    }

    /// Get all live ranges.
    pub fn all_ranges(&self) -> &HashMap<ValueId, LiveRange> {
        &self.live_ranges
    }

    /// Iterate over all values and their live ranges.
    pub fn iter(&self) -> impl Iterator<Item = (ValueId, &LiveRange)> {
        self.live_ranges.iter().map(|(&v, r)| (v, r))
    }

    /// Get values that are live at a specific schedule position.
    pub fn live_at(&self, position: usize) -> impl Iterator<Item = ValueId> {
        self.live_ranges
            .iter()
            .filter(move |(_, r)| r.birth <= position && position < r.death)
            .map(|(&v, _)| v)
    }
}

impl<G: Gate> Analysis<G> for ValueLiveness {
    type Output = Self;

    fn run(circuit: &Circuit<G>, analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        // Depend on scheduled order analysis.
        let schedule = analyzer.get::<ScheduledOrder>(circuit)?;

        let mut live_ranges: HashMap<ValueId, LiveRange> = HashMap::new();

        // For each value, compute birth and death positions.
        for (value_id, value) in circuit.all_values() {
            let producer = value.get_producer();

            // Birth: when the producer runs.
            let producer_op: Operation = producer.into();
            let birth = schedule
                .position(producer_op)
                .ok_or(Error::OperationNotScheduled(producer_op))?;

            // Death: after the last consumer runs.
            let mut max_consumer_pos: Option<usize> = None;
            for usage in value.get_uses() {
                let consumer_op: Operation = usage.get_consumer().into();
                let pos = schedule
                    .position(consumer_op)
                    .ok_or(Error::OperationNotScheduled(consumer_op))?;
                max_consumer_pos = Some(max_consumer_pos.map_or(pos, |m| m.max(pos)));
            }
            let death = max_consumer_pos.map_or(birth + 1, |p| p + 1);

            live_ranges.insert(value_id, LiveRange { birth, death });
        }

        Ok(ValueLiveness { live_ranges })
    }
}
