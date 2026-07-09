//! Minimal state foundation for Starfire's closed cognitive cycle.
//!
//! This module establishes persistence semantics for repeated independently judged
//! resolution attempts and a counterfactual observation recorder for shadow-only
//! ontology experiments. It still does not mutate the live runtime router.

use thiserror::Error;

use crate::charge::{
    Charge, DischargeJudge, JudgedDischarge, OntologyObservation, OutcomeWitness, Resolution,
    ResolverOutcome,
};

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
}

/// One resolver request plus independently measured outcome evidence.
#[derive(Debug, Clone)]
pub struct JudgedResolverAttempt {
    pub resolver: String,
    pub resolution: Resolution,
    pub witness: OutcomeWitness,
}

impl JudgedResolverAttempt {
    pub fn new(
        resolver: impl Into<String>,
        resolution: Resolution,
        witness: OutcomeWitness,
    ) -> Self {
        Self {
            resolver: resolver.into(),
            resolution,
            witness,
        }
    }
}

#[derive(Debug, Error)]
pub enum CycleObservationError {
    #[error("at least one resolver attempt is required")]
    EmptyAttempts,
    #[error("resolver attempt {index} has an empty resolver name")]
    EmptyResolverName { index: usize },
    #[error("resolver attempt {index} declares zero compute cost")]
    ZeroComputeCost { index: usize },
    #[error("CHARGE could not be admitted to a counterfactual cycle")]
    ChargeRejected,
    #[error("counterfactual cycle failed to apply judgment for resolver attempt {index}")]
    JudgmentApplicationFailed { index: usize },
}

/// Convert independently judged closed-cycle attempts into empirical ontology data.
///
/// Each resolver attempt is replayed from the exact same initial CHARGE snapshot in
/// a fresh `CognitiveCycleState`. This produces a counterfactual outcome matrix for
/// shadow evaluation without allowing an earlier resolver to change the state seen
/// by a later resolver. Only discharge accepted by `DischargeJudge` and actually
/// applied by `CognitiveCycleState` is recorded.
#[derive(Debug, Clone)]
pub struct CycleObservationRecorder<J> {
    judge: J,
}

impl<J> CycleObservationRecorder<J> {
    pub fn new(judge: J) -> Self {
        Self { judge }
    }

    pub fn judge(&self) -> &J {
        &self.judge
    }
}

impl<J: DischargeJudge> CycleObservationRecorder<J> {
    pub fn record(
        &self,
        charge: Charge,
        attempts: &[JudgedResolverAttempt],
    ) -> Result<OntologyObservation, CycleObservationError> {
        if attempts.is_empty() {
            return Err(CycleObservationError::EmptyAttempts);
        }

        let mut observation = OntologyObservation::new(charge.clone());
        for (index, attempt) in attempts.iter().enumerate() {
            if attempt.resolver.trim().is_empty() {
                return Err(CycleObservationError::EmptyResolverName { index });
            }
            if attempt.resolution.compute_cost == 0 {
                return Err(CycleObservationError::ZeroComputeCost { index });
            }

            let judged = self
                .judge
                .evaluate(&charge, &attempt.resolution, &attempt.witness);
            let mut cycle = CognitiveCycleState::new();
            if !cycle.admit_charge(charge.clone()) {
                return Err(CycleObservationError::ChargeRejected);
            }
            cycle
                .apply_judgment(0, &judged)
                .ok_or(CycleObservationError::JudgmentApplicationFailed { index })?;

            observation.record_outcome(ResolverOutcome::new(
                attempt.resolver.clone(),
                cycle.total_accepted_discharge() as f32,
                attempt.resolution.compute_cost,
            ));
        }

        Ok(observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{
        ChargeKind, ChargeScope, ImprovementDirection, RelativeImprovementJudge,
    };

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

    fn attempt(resolver: &str, before: f64, after: f64) -> JudgedResolverAttempt {
        JudgedResolverAttempt::new(
            resolver,
            Resolution {
                discharged: 1.0,
                emitted: vec![],
                permitted_decay: 0.0,
                compute_cost: 1,
            },
            OutcomeWitness::new(
                "objective_progress",
                before,
                after,
                ImprovementDirection::HigherIsBetter,
                vec![format!("{resolver} verifier evidence")],
            ),
        )
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

    #[test]
    fn recorder_uses_independently_accepted_not_requested_discharge() {
        let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
        let observation = recorder
            .record(
                charge(1.0),
                &[attempt("unchanged", 0.0, 0.0), attempt("useful", 0.0, 0.6)],
            )
            .unwrap();

        assert_eq!(observation.outcomes.len(), 2);
        assert_eq!(observation.outcomes[0].discharged, 0.0);
        assert!((observation.outcomes[1].discharged - 0.6).abs() < 1e-6);
        assert_eq!(observation.charge.magnitude, 1.0);
        assert_eq!(observation.charge.persistence, 0);
    }

    #[test]
    fn recorder_replays_each_resolver_from_the_same_charge_snapshot() {
        let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
        let observation = recorder
            .record(
                charge(1.0),
                &[attempt("first", 0.0, 0.5), attempt("second", 0.0, 0.5)],
            )
            .unwrap();

        assert!((observation.outcomes[0].discharged - 0.5).abs() < 1e-6);
        assert!((observation.outcomes[1].discharged - 0.5).abs() < 1e-6);
    }

    #[test]
    fn recorder_rejects_zero_cost_attempts_before_induction() {
        let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
        let mut invalid = attempt("memory", 0.0, 1.0);
        invalid.resolution.compute_cost = 0;

        let error = recorder.record(charge(1.0), &[invalid]).unwrap_err();
        assert!(matches!(error, CycleObservationError::ZeroComputeCost { .. }));
    }
}
