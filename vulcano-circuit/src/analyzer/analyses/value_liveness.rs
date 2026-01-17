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
    circuit::{
        Circuit,
        operations::Operation,
        subcircuit::{CircuitId, Subcircuit},
        value::ValueId,
    },
    error::{Error, Result},
    gate::Gate,
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

/// Per-subcircuit liveness result.
pub struct SubcircuitLiveness {
    /// Map from ValueId to its live range.
    live_ranges: HashMap<ValueId, LiveRange>,
}

impl SubcircuitLiveness {
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
    pub fn live_at(&self, position: usize) -> impl Iterator<Item = ValueId> + '_ {
        self.live_ranges
            .iter()
            .filter(move |(_, r)| r.birth <= position && position < r.death)
            .map(|(&v, _)| v)
    }
}

/// Result of liveness analysis.
pub struct ValueLiveness {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitLiveness>,
}

impl ValueLiveness {
    /// Get the liveness for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitLiveness> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitLiveness)> {
        self.results.iter().map(|(&id, liveness)| (id, liveness))
    }
}

impl<G: Gate> Analysis<G> for ValueLiveness {
    type Output = Self;

    fn run(circuit: &Circuit<G>, analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        // Depend on scheduled order analysis.
        let schedule = analyzer.get::<ScheduledOrder>(circuit)?;

        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let subcircuit_id = subcircuit.id();
            let subcircuit_schedule = schedule
                .for_subcircuit(subcircuit_id)
                .ok_or(Error::SubcircuitAnalysisMissing(subcircuit_id))?;
            let liveness = compute_liveness(subcircuit, subcircuit_schedule)?;
            results.insert(subcircuit_id, liveness);
        }

        Ok(ValueLiveness { results })
    }
}

/// Compute liveness for a single subcircuit.
fn compute_liveness<G: Gate>(
    subcircuit: &Subcircuit<G>,
    schedule: &crate::analyzer::analyses::scheduled_order::SubcircuitSchedule,
) -> Result<SubcircuitLiveness> {
    let mut live_ranges: HashMap<ValueId, LiveRange> = HashMap::new();

    // For each value, compute birth and death positions.
    for (value_id, value) in subcircuit.all_values() {
        let producer = value.get_product().get_producer();

        // Birth: when the producer runs.
        let producer_op: Operation = producer.into();
        let birth = schedule
            .position(producer_op)
            .ok_or(Error::OperationNotScheduled(producer_op))?;

        // Death: after the last consumer runs.
        let mut max_consumer_pos: Option<usize> = None;
        for usage in value.get_destinations() {
            let consumer_op: Operation = usage.get_consumer().into();
            let pos = schedule
                .position(consumer_op)
                .ok_or(Error::OperationNotScheduled(consumer_op))?;
            max_consumer_pos = Some(max_consumer_pos.map_or(pos, |m| m.max(pos)));
        }
        let death = max_consumer_pos.map_or(birth + 1, |p| p + 1);

        live_ranges.insert(value_id, LiveRange { birth, death });
    }

    Ok(SubcircuitLiveness { live_ranges })
}
