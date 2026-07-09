//! Minimal state foundation for Starfire's closed cognitive cycle.
//!
//! This module deliberately does not select operators, mutate the world model, or
//! wire into `Runtime::chat()`. It establishes the persistence semantics required
//! for repeated independently judged resolution attempts.

use crate::charge::{Charge, JudgedDischarge};

const RESOLVED_EPSILON: f32 = 1e-6;

/// Result of applying one independently judged resolution attempt to a pending
/// charge.
#[derive(Debug, Clone, PartialEq)]
pub enum ChargeDisposition {
    Resolved,
    Persisted {
        remaining_magnitude: f32,
        persistence: u32,
    },
}

/// State carried across iterations of the future observe/reconcile/route/act loop.
#[derive(Debug, Default)]
pub struct CognitiveCycleState {
    pending: Vec<Charge>,
    attempts: u64,
    total_accepted_discharge: f64,
}

impl CognitiveCycleState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit non-zero unresolved state to the cycle.
    pub fn admit_charge(&mut self, charge: Charge) -> bool {
        if !charge.magnitude.is_finite() || charge.magnitude <= RESOLVED_EPSILON {
            return false;
        }
        self.pending.push(charge);
        true
    }

    pub fn pending(&self) -> &[Charge] {
        &self.pending
    }

    pub fn attempts(&self) -> u64 {
        self.attempts
    }

    pub fn total_accepted_discharge(&self) -> f64 {
        self.total_accepted_discharge
    }

    /// Apply independently accepted discharge to one pending charge.
    ///
    /// Unresolved charge survives, ages by one persistence step, and remains in
    /// the cycle. Fully resolved charge is removed.
    pub fn apply_judgment(
        &mut self,
        pending_index: usize,
        judged: &JudgedDischarge,
    ) -> Option<ChargeDisposition> {
        self.attempts = self.attempts.saturating_add(1);

        let (resolved, remaining_magnitude, persistence, accepted) = {
            let charge = self.pending.get_mut(pending_index)?;

            let accepted = if judged.accepted.is_finite() {
                judged.accepted.max(0.0).min(charge.magnitude.max(0.0))
            } else {
                0.0
            };

            charge.magnitude = (charge.magnitude - accepted).max(0.0);

            if charge.magnitude <= RESOLVED_EPSILON {
                (true, charge.magnitude, charge.persistence, accepted)
            } else {
                charge.persistence = charge.persistence.saturating_add(1);
                (false, charge.magnitude, charge.persistence, accepted)
            }
        };

        self.total_accepted_discharge += accepted as f64;

        if resolved {
            self.pending.remove(pending_index);
            return Some(ChargeDisposition::Resolved);
        }

        Some(ChargeDisposition::Persisted {
            remaining_magnitude,
            persistence,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

    fn charge(magnitude: f32) -> Charge {
        Charge::new(
            ChargeKind::EpistemicGap,
            vec![1.0],
            magnitude,
            ChargeScope::Topic("hidden-rule".into()),
        )
    }

    fn judged(accepted: f32) -> JudgedDischarge {
        JudgedDischarge {
            requested: accepted,
            accepted,
            measured_improvement: accepted as f64,
            metric: "test".into(),
            evidence: vec![],
        }
    }

    #[test]
    fn unresolved_charge_persists_and_ages() {
        let mut cycle = CognitiveCycleState::new();
        assert!(cycle.admit_charge(charge(1.0)));

        let disposition = cycle.apply_judgment(0, &judged(0.25)).unwrap();
        assert_eq!(
            disposition,
            ChargeDisposition::Persisted {
                remaining_magnitude: 0.75,
                persistence: 1,
            }
        );
        assert_eq!(cycle.pending().len(), 1);
        assert_eq!(cycle.pending()[0].persistence, 1);
        assert!((cycle.pending()[0].magnitude - 0.75).abs() < 1e-6);
    }

    #[test]
    fn zero_accepted_discharge_cannot_suppress_charge() {
        let mut cycle = CognitiveCycleState::new();
        cycle.admit_charge(charge(0.8));

        let disposition = cycle.apply_judgment(0, &judged(0.0)).unwrap();
        assert_eq!(
            disposition,
            ChargeDisposition::Persisted {
                remaining_magnitude: 0.8,
                persistence: 1,
            }
        );
        assert_eq!(cycle.pending()[0].magnitude, 0.8);
    }

    #[test]
    fn fully_discharged_charge_leaves_pending_set() {
        let mut cycle = CognitiveCycleState::new();
        cycle.admit_charge(charge(0.5));

        assert_eq!(
            cycle.apply_judgment(0, &judged(0.5)),
            Some(ChargeDisposition::Resolved)
        );
        assert!(cycle.pending().is_empty());
        assert_eq!(cycle.attempts(), 1);
        assert!((cycle.total_accepted_discharge() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn invalid_or_zero_charge_is_not_admitted() {
        let mut cycle = CognitiveCycleState::new();
        assert!(!cycle.admit_charge(charge(0.0)));
        assert!(!cycle.admit_charge(charge(f32::NAN)));
        assert!(cycle.pending().is_empty());
    }
}
