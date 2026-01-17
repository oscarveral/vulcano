//! Register Allocation Analysis
//!
//! Assigns virtual registers to values using linear scan algorithm.
//! Each operand type gets its own register file.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::{
    analyzer::{
        analyses::value_liveness::ValueLiveness,
        {Analysis, Analyzer},
    },
    circuit::{
        Circuit,
        subcircuit::{CircuitId, Subcircuit},
        value::ValueId,
    },
    error::{Error, Result},
    gate::Gate,
};

/// Per-subcircuit register allocation result.
pub struct SubcircuitAllocation<G: Gate> {
    /// Map from ValueId to its register number.
    assignments: HashMap<ValueId, usize>,
    /// Peak register usage per operand type.
    peak_usage: HashMap<G::Operand, usize>,
}

impl<G: Gate> SubcircuitAllocation<G> {
    /// Get the register number for a value.
    pub fn get(&self, value: ValueId) -> Option<usize> {
        self.assignments.get(&value).copied()
    }

    /// Get all register assignments.
    pub fn all_assignments(&self) -> &HashMap<ValueId, usize> {
        &self.assignments
    }

    /// Iterate over all assignments.
    pub fn iter(&self) -> impl Iterator<Item = (ValueId, usize)> + '_ {
        self.assignments.iter().map(|(&v, &r)| (v, r))
    }

    /// Get peak register usage for a specific operand type.
    pub fn peak_for_type(&self, ty: G::Operand) -> usize {
        self.peak_usage.get(&ty).copied().unwrap_or(0)
    }

    /// Get all peak register usages by type.
    pub fn peak_usage(&self) -> &HashMap<G::Operand, usize> {
        &self.peak_usage
    }
}

/// Result of register allocation analysis.
pub struct RegisterAllocation<G: Gate> {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitAllocation<G>>,
}

impl<G: Gate> RegisterAllocation<G> {
    /// Get the register allocation for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitAllocation<G>> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitAllocation<G>)> {
        self.results.iter().map(|(&id, alloc)| (id, alloc))
    }
}

impl<G: Gate> Analysis<G> for RegisterAllocation<G> {
    type Output = Self;

    fn run(circuit: &Circuit<G>, analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let liveness = analyzer.get::<ValueLiveness>(circuit)?;

        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let subcircuit_id = subcircuit.id();
            let subcircuit_liveness = liveness
                .for_subcircuit(subcircuit_id)
                .ok_or(Error::SubcircuitAnalysisMissing(subcircuit_id))?;
            let allocation = compute_register_allocation(subcircuit, subcircuit_liveness)?;
            results.insert(subcircuit_id, allocation);
        }

        Ok(RegisterAllocation { results })
    }
}

/// Compute register allocation for a single subcircuit using linear scan.
fn compute_register_allocation<G: Gate>(
    subcircuit: &Subcircuit<G>,
    liveness: &crate::analyzer::analyses::value_liveness::SubcircuitLiveness,
) -> Result<SubcircuitAllocation<G>> {
    // Group values by operand type.
    let mut by_type: HashMap<G::Operand, Vec<(ValueId, usize, usize)>> = HashMap::new();
    for (value_id, value) in subcircuit.all_values() {
        let ty = value.get_type();
        let range = liveness
            .get(value_id)
            .ok_or(Error::ValueNotFound(value_id))?;
        by_type
            .entry(ty)
            .or_default()
            .push((value_id, range.birth, range.death));
    }

    let mut assignments = HashMap::new();
    let mut peak_usage = HashMap::new();

    // Linear scan per operand type.
    for (ty, mut values) in by_type {
        // Sort by birth position.
        values.sort_by_key(|(_, birth, _)| *birth);

        // Active: (death, register), min-heap by death.
        let mut active: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();
        // Free registers, min-heap by register number.
        let mut free_regs: BinaryHeap<Reverse<usize>> = BinaryHeap::new();
        let mut next_reg = 0usize;

        for (value_id, birth, death) in values {
            // Expire intervals that died before this birth.
            while let Some(&Reverse((d, reg))) = active.peek() {
                if d <= birth {
                    active.pop();
                    free_regs.push(Reverse(reg));
                } else {
                    break;
                }
            }

            // Assign register: reuse if available, else allocate new.
            let reg = free_regs.pop().map(|Reverse(r)| r).unwrap_or_else(|| {
                let r = next_reg;
                next_reg += 1;
                r
            });

            assignments.insert(value_id, reg);
            active.push(Reverse((death, reg)));
        }

        peak_usage.insert(ty, next_reg);
    }

    Ok(SubcircuitAllocation {
        assignments,
        peak_usage,
    })
}
