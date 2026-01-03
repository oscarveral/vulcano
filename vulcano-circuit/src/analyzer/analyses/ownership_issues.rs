//! Ownership Analysis
//!
//! Analyzes ownership status of values in the circuit.
//! Values consumed (moved) more than once are overconsumed.
//! Values never consumed (moved) are leaked.

use crate::{
    analyzer::{Analysis, Analyzer},
    circuit::Circuit,
    error::Result,
    gate::Gate,
    handles::{Ownership, ValueId},
};

/// Ownership issue.
#[derive(Clone, Debug)]
pub enum OwnershipIssue {
    /// Value is moved multiple times.
    Overconsumed { value: ValueId, move_count: usize },
    /// Value is never moved.
    Leaked { value: ValueId },
}

/// Result of ownership analysis.
pub struct OwnershipIssues {
    /// All non-standard ownership statuses.
    issues: Vec<OwnershipIssue>,
}

impl OwnershipIssues {
    /// Get all ownership issues.
    pub fn issues(&self) -> &[OwnershipIssue] {
        &self.issues
    }

    /// Check if ownership is valid (no issues).
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// Get overconsumed values.
    pub fn overconsumed(&self) -> impl Iterator<Item = (ValueId, usize)> {
        self.issues.iter().filter_map(|s| match s {
            OwnershipIssue::Overconsumed { value, move_count } => Some((*value, *move_count)),
            _ => None,
        })
    }

    /// Get leaked values.
    pub fn leaked(&self) -> impl Iterator<Item = ValueId> {
        self.issues.iter().filter_map(|s| match s {
            OwnershipIssue::Leaked { value } => Some(*value),
            _ => None,
        })
    }
}

impl Analysis for OwnershipIssues {
    type Output = Self;

    fn run<G: Gate>(circuit: &Circuit<G>, _analyzer: &mut Analyzer<G>) -> Result<Self::Output> {
        let mut issues = Vec::new();

        for (value_id, value) in circuit.all_values() {
            // Count how many times this value is moved.
            let move_count = value
                .get_uses()
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

        Ok(OwnershipIssues { issues })
    }
}
