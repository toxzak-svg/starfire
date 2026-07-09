//! Independent discharge judging for CHARGE.
//!
//! A resolver may request discharge in a [`Resolution`], but it does not get to
//! declare that the unresolved state actually improved. A [`DischargeJudge`]
//! compares independently measured before/after outcome evidence and caps the
//! accepted discharge accordingly.

use super::{Charge, Resolution};

/// Direction in which an independently measured metric represents improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImprovementDirection {
    /// Larger values are better, e.g. objective progress.
    HigherIsBetter,
    /// Smaller values are better, e.g. prediction or replay error.
    LowerIsBetter,
}

/// Independently measured before/after evidence for one resolution attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct OutcomeWitness {
    pub metric: String,
    pub before: f64,
    pub after: f64,
    pub direction: ImprovementDirection,
    pub evidence: Vec<String>,
}

impl OutcomeWitness {
    pub fn new(
        metric: impl Into<String>,
        before: f64,
        after: f64,
        direction: ImprovementDirection,
        evidence: Vec<String>,
    ) -> Self {
        Self {
            metric: metric.into(),
            before,
            after,
            direction,
            evidence,
        }
    }

    /// Relative measurable improvement in [0, 1].
    ///
    /// The denominator is at least 1.0 so a metric moving from 0.0 to 1.0 can
    /// represent full normalized progress without exploding near zero.
    pub fn relative_improvement(&self) -> f64 {
        if !self.before.is_finite() || !self.after.is_finite() {
            return 0.0;
        }

        let raw = match self.direction {
            ImprovementDirection::HigherIsBetter => self.after - self.before,
            ImprovementDirection::LowerIsBetter => self.before - self.after,
        };

        if raw <= 0.0 {
            return 0.0;
        }

        let scale = self.before.abs().max(self.after.abs()).max(1.0);
        (raw / scale).clamp(0.0, 1.0)
    }
}

/// Accepted result after independent outcome judging.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgedDischarge {
    /// Resolver-requested discharge after non-negative clamping.
    pub requested: f32,
    /// Independently accepted discharge.
    pub accepted: f32,
    /// Fractional measured improvement in [0, 1].
    pub measured_improvement: f64,
    /// Metric used to judge the resolution.
    pub metric: String,
    /// Evidence carried by the outcome witness.
    pub evidence: Vec<String>,
}

/// Independent policy for accepting or rejecting requested CHARGE discharge.
pub trait DischargeJudge {
    fn evaluate(
        &self,
        charge: &Charge,
        resolution: &Resolution,
        witness: &OutcomeWitness,
    ) -> JudgedDischarge;
}

/// Deterministic judge that caps accepted discharge by relative measured outcome
/// improvement.
///
/// This is intentionally conservative and simple. It is a foundation contract,
/// not the final semantics for every CHARGE kind.
#[derive(Debug, Clone, Copy, Default)]
pub struct RelativeImprovementJudge;

impl DischargeJudge for RelativeImprovementJudge {
    fn evaluate(
        &self,
        charge: &Charge,
        resolution: &Resolution,
        witness: &OutcomeWitness,
    ) -> JudgedDischarge {
        let requested = resolution.discharged.max(0.0).min(charge.magnitude.max(0.0));
        let measured_improvement = witness.relative_improvement();
        let evidence_cap = (charge.magnitude.max(0.0) as f64 * measured_improvement) as f32;
        let accepted = requested.min(evidence_cap);

        JudgedDischarge {
            requested,
            accepted,
            measured_improvement,
            metric: witness.metric.clone(),
            evidence: witness.evidence.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

    fn charge(magnitude: f32) -> Charge {
        Charge::new(
            ChargeKind::PredictionResidual,
            vec![1.0],
            magnitude,
            ChargeScope::Global,
        )
    }

    #[test]
    fn resolver_cannot_create_discharge_without_measured_improvement() {
        let charge = charge(1.0);
        let resolution = Resolution {
            discharged: 1.0,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: 1,
        };
        let witness = OutcomeWitness::new(
            "prediction_error",
            0.8,
            0.8,
            ImprovementDirection::LowerIsBetter,
            vec!["held-out error unchanged".into()],
        );

        let judged = RelativeImprovementJudge.evaluate(&charge, &resolution, &witness);
        assert_eq!(judged.requested, 1.0);
        assert_eq!(judged.accepted, 0.0);
    }

    #[test]
    fn accepted_discharge_is_capped_by_measured_improvement() {
        let charge = charge(1.0);
        let resolution = Resolution {
            discharged: 1.0,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: 1,
        };
        let witness = OutcomeWitness::new(
            "prediction_error",
            1.0,
            0.4,
            ImprovementDirection::LowerIsBetter,
            vec!["held-out transition error fell".into()],
        );

        let judged = RelativeImprovementJudge.evaluate(&charge, &resolution, &witness);
        assert!((judged.measured_improvement - 0.6).abs() < 1e-9);
        assert!((judged.accepted - 0.6).abs() < 1e-6);
    }

    #[test]
    fn judge_never_accepts_more_than_resolver_requested() {
        let charge = charge(1.0);
        let resolution = Resolution {
            discharged: 0.2,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: 1,
        };
        let witness = OutcomeWitness::new(
            "objective_progress",
            0.0,
            1.0,
            ImprovementDirection::HigherIsBetter,
            vec!["objective verifier reached terminal target".into()],
        );

        let judged = RelativeImprovementJudge.evaluate(&charge, &resolution, &witness);
        assert_eq!(judged.accepted, 0.2);
    }

    #[test]
    fn worsening_or_non_finite_metrics_accept_no_discharge() {
        for witness in [
            OutcomeWitness::new(
                "prediction_error",
                0.2,
                0.8,
                ImprovementDirection::LowerIsBetter,
                vec![],
            ),
            OutcomeWitness::new(
                "prediction_error",
                f64::NAN,
                0.1,
                ImprovementDirection::LowerIsBetter,
                vec![],
            ),
        ] {
            let judged = RelativeImprovementJudge.evaluate(
                &charge(1.0),
                &Resolution {
                    discharged: 1.0,
                    emitted: vec![],
                    permitted_decay: 0.0,
                    compute_cost: 1,
                },
                &witness,
            );
            assert_eq!(judged.accepted, 0.0);
        }
    }
}
