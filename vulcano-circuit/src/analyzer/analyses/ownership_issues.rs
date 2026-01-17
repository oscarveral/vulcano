//! Ownership Analysis
//!
//! Analyzes ownership status of values in the circuit.
//! Values consumed (moved) more than once are overconsumed.
//! Values never consumed (moved) are leaked.

use std::collections::HashMap;

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::{
        Circuit,
        subcircuit::{CircuitId, Subcircuit},
        value::{Ownership, ValueId},
    },
    error::Result,
    gate::Gate,
};

/// Ownership issue.
#[derive(Clone, Debug)]
pub enum OwnershipIssue {
    /// Value is moved multiple times.
    Overconsumed { value: ValueId, move_count: usize },
    /// Value is never moved.
    Leaked { value: ValueId },
}

/// Per-subcircuit ownership issues result.
pub struct SubcircuitOwnership {
    /// All non-standard ownership statuses.
    issues: Vec<OwnershipIssue>,
}

impl SubcircuitOwnership {
    /// Get all ownership issues.
    pub fn issues(&self) -> &[OwnershipIssue] {
        &self.issues
    }

    /// Check if ownership is valid (no issues).
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// Get overconsumed values.
    pub fn overconsumed(&self) -> impl Iterator<Item = (ValueId, usize)> + '_ {
        self.issues.iter().filter_map(|s| match s {
            OwnershipIssue::Overconsumed { value, move_count } => Some((*value, *move_count)),
            _ => None,
        })
    }

    /// Get leaked values.
    pub fn leaked(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.issues.iter().filter_map(|s| match s {
            OwnershipIssue::Leaked { value } => Some(*value),
            _ => None,
        })
    }
}

/// Result of ownership analysis.
pub struct OwnershipIssues {
    /// Per-subcircuit results.
    results: HashMap<CircuitId, SubcircuitOwnership>,
}

impl OwnershipIssues {
    /// Get the ownership issues for a specific subcircuit.
    pub fn for_subcircuit(&self, id: CircuitId) -> Option<&SubcircuitOwnership> {
        self.results.get(&id)
    }

    /// Iterate over all subcircuit results.
    pub fn iter(&self) -> impl Iterator<Item = (CircuitId, &SubcircuitOwnership)> {
        self.results.iter().map(|(&id, issues)| (id, issues))
    }

    /// Check if all subcircuits have valid ownership.
    pub fn is_all_valid(&self) -> bool {
        self.results.values().all(|s| s.is_valid())
    }
}

impl<G: Gate> Analysis<G> for OwnershipIssues {
    type Output = Self;

    fn run(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut results = HashMap::new();

        for subcircuit in circuit.iter() {
            let issues = compute_ownership_issues(subcircuit);
            results.insert(subcircuit.id(), issues);
        }

        Ok(OwnershipIssues { results })
    }
}

/// Compute ownership issues for a single subcircuit.
fn compute_ownership_issues<G: Gate>(subcircuit: &Subcircuit<G>) -> SubcircuitOwnership {
    let mut issues = Vec::new();

    for (value_id, value) in subcircuit.all_values() {
        // Count how many times this value is moved.
        let move_count = value
            .get_destinations()
            .iter()
            .filter(|u| u.get_mode() == Ownership::Move)
            .count();

        match move_count {
            0 => {
                // Never consumed.
                issues.push(OwnershipIssue::Leaked { value: value_id });
            }
            1 => {
                // Exactly one move.
            }
            n => {
                // Multiple moves.
                issues.push(OwnershipIssue::Overconsumed {
                    value: value_id,
                    move_count: n,
                });
            }
        }
    }

    SubcircuitOwnership { issues }
}
