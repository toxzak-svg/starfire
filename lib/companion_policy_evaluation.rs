//! S5-C comparative evaluation for shadow interaction-policy trials.
//!
//! The evaluator consumes only frozen S5-B trials, independently witnessed S4
//! outcomes, and predeclared compute observations. Split assignment uses opaque
//! subject digests and issue times only. Development evidence is reported but is
//! never used for the promotion verdict.

use crate::companion_interaction_outcomes::{
    InteractionOutcomeEvent, InteractionOutcomeLedger, InteractionTrial, InteractionTrialId,
    ObservedSignal, PairwisePreference,
};
use crate::companion_interaction_policy::PolicyVariant;
use crate::companion_prediction_ledger::PredictionStatus;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const BASIS_POINTS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvaluationSplit {
    Development,
    OpaqueSubjectHoldout,
    TemporalHoldout,
}

impl EvaluationSplit {
    #[must_use]
    pub const fn is_holdout(self) -> bool {
        matches!(self, Self::OpaqueSubjectHoldout | Self::TemporalHoldout)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationSplitPolicy {
    pub temporal_holdout_start_ms: u64,
    pub opaque_subject_modulus: u64,
    pub opaque_subject_remainder: u64,
}

impl EvaluationSplitPolicy {
    pub fn validate(self) -> Result<Self, PolicyEvaluationError> {
        if self.temporal_holdout_start_ms == 0 {
            return Err(PolicyEvaluationError::InvalidTemporalBoundary);
        }
        if self.opaque_subject_modulus < 2
            || self.opaque_subject_remainder >= self.opaque_subject_modulus
        {
            return Err(PolicyEvaluationError::InvalidSubjectPartition);
        }
        Ok(self)
    }

    pub fn classify(
        self,
        trial: &InteractionTrial,
    ) -> Result<EvaluationSplit, PolicyEvaluationError> {
        self.validate()?;
        if trial.issued_at_ms >= self.temporal_holdout_start_ms {
            return Ok(EvaluationSplit::TemporalHoldout);
        }
        if trial.subject_scope_digest % self.opaque_subject_modulus
            == self.opaque_subject_remainder
        {
            return Ok(EvaluationSplit::OpaqueSubjectHoldout);
        }
        Ok(EvaluationSplit::Development)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmComputeObservation {
    pub trial_id: InteractionTrialId,
    pub variant: PolicyVariant,
    pub compute_micros: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationConfig {
    pub split_policy: EvaluationSplitPolicy,
    pub min_resolved_per_arm_per_holdout: u64,
    pub min_direct_outcomes_per_arm_per_holdout: u64,
    pub min_pairwise_comparisons_per_control_per_holdout: u64,
    pub min_brier_improvement_ppm: u32,
    pub min_pairwise_win_margin_bps: u16,
    pub max_calibration_regression_bps: u16,
    pub max_correction_regression_bps: u16,
    pub max_clarification_regression_bps: u16,
    pub max_completion_regression_bps: u16,
    pub max_abandonment_regression_bps: u16,
    pub max_abstention_regression_bps: u16,
    pub max_compute_overhead_bps: u16,
}

impl PolicyEvaluationConfig {
    pub fn validate(self) -> Result<Self, PolicyEvaluationError> {
        self.split_policy.validate()?;
        if self.min_resolved_per_arm_per_holdout == 0
            || self.min_direct_outcomes_per_arm_per_holdout == 0
            || self.min_pairwise_comparisons_per_control_per_holdout == 0
        {
            return Err(PolicyEvaluationError::InvalidEvidenceMinimum);
        }
        for value in [
            self.min_pairwise_win_margin_bps,
            self.max_calibration_regression_bps,
            self.max_correction_regression_bps,
            self.max_clarification_regression_bps,
            self.max_completion_regression_bps,
            self.max_abandonment_regression_bps,
            self.max_abstention_regression_bps,
            self.max_compute_overhead_bps,
        ] {
            if u64::from(value) > BASIS_POINTS {
                return Err(PolicyEvaluationError::BasisPointsOutOfRange(value));
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArmEvaluationMetrics {
    pub trials: u64,
    pub predictions: u64,
    pub resolved_predictions: u64,
    pub pending_predictions: u64,
    pub expired_predictions: u64,
    pub abstentions: u64,
    pub total_brier_score_ppm: u64,
    pub mean_brier_score_ppm: Option<u32>,
    pub total_top_label_calibration_error_bps: u64,
    pub mean_top_label_calibration_error_bps: Option<u16>,
    pub direct_events: u64,
    pub direct_outcomes: u64,
    pub positive_direct_outcomes: u64,
    pub corrections: u64,
    pub clarification_requests: u64,
    pub completions: u64,
    pub abandonments: u64,
    pub pairwise_wins: u64,
    pub pairwise_losses: u64,
    pub pairwise_ties: u64,
    pub compute_observations: u64,
    pub total_compute_micros: u64,
    pub mean_compute_micros: Option<u64>,
    pub correction_rate_bps: Option<u16>,
    pub clarification_rate_bps: Option<u16>,
    pub completion_rate_bps: Option<u16>,
    pub abandonment_rate_bps: Option<u16>,
    pub abstention_rate_bps: Option<u16>,
}

impl ArmEvaluationMetrics {
    fn finalize(&mut self) {
        self.mean_brier_score_ppm = mean_u32(self.total_brier_score_ppm, self.resolved_predictions);
        self.mean_top_label_calibration_error_bps = mean_u16(
            self.total_top_label_calibration_error_bps,
            self.resolved_predictions,
        );
        self.mean_compute_micros = mean_u64(self.total_compute_micros, self.compute_observations);
        self.correction_rate_bps = rate_bps(self.corrections, self.direct_outcomes);
        self.clarification_rate_bps = rate_bps(self.clarification_requests, self.direct_outcomes);
        self.completion_rate_bps = rate_bps(self.completions, self.direct_outcomes);
        self.abandonment_rate_bps = rate_bps(self.abandonments, self.direct_outcomes);
        self.abstention_rate_bps = rate_bps(self.abstentions, self.trials);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CandidatePairwiseMetrics {
    pub candidate_wins: u64,
    pub control_wins: u64,
    pub ties: u64,
    pub total: u64,
    pub candidate_win_margin_bps: Option<i32>,
}

impl CandidatePairwiseMetrics {
    fn finalize(&mut self) {
        self.total = self.candidate_wins + self.control_wins + self.ties;
        self.candidate_win_margin_bps =
            signed_rate_delta_bps(self.candidate_wins, self.control_wins, self.total);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitEvaluationReport {
    pub split: EvaluationSplit,
    pub arms: BTreeMap<PolicyVariant, ArmEvaluationMetrics>,
    pub candidate_pairwise: BTreeMap<PolicyVariant, CandidatePairwiseMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonGates {
    pub resolved_evidence_sufficient: bool,
    pub direct_evidence_sufficient: bool,
    pub pairwise_evidence_sufficient: bool,
    pub brier_improvement_passed: bool,
    pub calibration_non_regression_passed: bool,
    pub pairwise_margin_passed: bool,
    pub correction_non_regression_passed: bool,
    pub clarification_non_regression_passed: bool,
    pub completion_non_regression_passed: bool,
    pub abandonment_non_regression_passed: bool,
    pub abstention_non_regression_passed: bool,
    pub compute_overhead_passed: bool,
}

impl ComparisonGates {
    #[must_use]
    pub const fn evidence_sufficient(&self) -> bool {
        self.resolved_evidence_sufficient
            && self.direct_evidence_sufficient
            && self.pairwise_evidence_sufficient
    }

    #[must_use]
    pub const fn performance_passed(&self) -> bool {
        self.brier_improvement_passed
            && self.calibration_non_regression_passed
            && self.pairwise_margin_passed
            && self.correction_non_regression_passed
            && self.clarification_non_regression_passed
            && self.completion_non_regression_passed
            && self.abandonment_non_regression_passed
            && self.abstention_non_regression_passed
            && self.compute_overhead_passed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateControlComparison {
    pub split: EvaluationSplit,
    pub control: PolicyVariant,
    pub candidate_resolved: u64,
    pub control_resolved: u64,
    pub candidate_direct_outcomes: u64,
    pub control_direct_outcomes: u64,
    pub pairwise: CandidatePairwiseMetrics,
    pub brier_improvement_ppm: Option<i64>,
    pub calibration_regression_bps: Option<i32>,
    pub correction_regression_bps: Option<i32>,
    pub clarification_regression_bps: Option<i32>,
    pub completion_regression_bps: Option<i32>,
    pub abandonment_regression_bps: Option<i32>,
    pub abstention_regression_bps: Option<i32>,
    pub compute_overhead_bps: Option<i32>,
    pub gates: ComparisonGates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEvaluationVerdict {
    Pass,
    Fail,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationReport {
    pub splits: Vec<SplitEvaluationReport>,
    pub holdout_comparisons: Vec<CandidateControlComparison>,
    pub verdict: PolicyEvaluationVerdict,
    pub development_excluded_from_verdict: bool,
    pub promotion_eligible: bool,
    pub live_response_influence: bool,
    pub routing_authority: bool,
    pub belief_promotion_authority: bool,
    pub action_authority: bool,
}

pub fn evaluate_shadow_policies(
    outcomes: &InteractionOutcomeLedger,
    compute: &[ArmComputeObservation],
    config: PolicyEvaluationConfig,
) -> Result<PolicyEvaluationReport, PolicyEvaluationError> {
    let config = config.validate()?;
    let costs = validate_compute_observations(outcomes, compute)?;
    let mut split_metrics = empty_split_metrics();
    let mut pairwise = empty_pairwise_metrics();

    for trial in outcomes.trials().values() {
        let split = config.split_policy.classify(trial)?;
        let arms = split_metrics
            .get_mut(&split)
            .expect("all evaluation splits are initialized");
        for arm in &trial.arms {
            let metrics = arms
                .get_mut(&arm.variant)
                .expect("all policy variants are initialized");
            metrics.trials += 1;
            let cost = costs
                .get(&(trial.id, arm.variant))
                .copied()
                .ok_or(PolicyEvaluationError::MissingComputeObservation {
                    trial_id: trial.id,
                    variant: arm.variant,
                })?;
            metrics.compute_observations += 1;
            metrics.total_compute_micros = metrics
                .total_compute_micros
                .checked_add(cost)
                .ok_or(PolicyEvaluationError::MetricOverflow)?;

            if let Some(prediction_id) = arm.prediction_id {
                metrics.predictions += 1;
                let prediction = outcomes
                    .mirrored_prediction_ledger()
                    .prediction(prediction_id)
                    .ok_or(PolicyEvaluationError::MissingPrediction(prediction_id))?;
                match &prediction.status {
                    PredictionStatus::Pending => metrics.pending_predictions += 1,
                    PredictionStatus::Expired { .. } => metrics.expired_predictions += 1,
                    PredictionStatus::Resolved { witness, score } => {
                        metrics.resolved_predictions += 1;
                        metrics.total_brier_score_ppm = metrics
                            .total_brier_score_ppm
                            .checked_add(u64::from(score.score_ppm))
                            .ok_or(PolicyEvaluationError::MetricOverflow)?;
                        metrics.total_top_label_calibration_error_bps = metrics
                            .total_top_label_calibration_error_bps
                            .checked_add(u64::from(top_label_calibration_error_bps(
                                &prediction.outcomes,
                                &witness.label,
                            )?))
                            .ok_or(PolicyEvaluationError::MetricOverflow)?;
                    }
                }
            } else if arm.abstention_id.is_some() {
                metrics.abstentions += 1;
            } else {
                return Err(PolicyEvaluationError::MalformedTrialArm {
                    trial_id: trial.id,
                    variant: arm.variant,
                });
            }
        }
    }

    for event in outcomes.events() {
        match event {
            InteractionOutcomeEvent::TrialRegistered { .. } => {}
            InteractionOutcomeEvent::ObservedSignalRecorded {
                trial_id,
                delivered_variant,
                evidence,
                ..
            } => {
                let trial = outcomes
                    .trials()
                    .get(trial_id)
                    .ok_or(PolicyEvaluationError::UnknownTrial(*trial_id))?;
                let split = config.split_policy.classify(trial)?;
                let metrics = split_metrics
                    .get_mut(&split)
                    .and_then(|arms| arms.get_mut(delivered_variant))
                    .expect("all splits and variants are initialized");
                metrics.direct_events += 1;
                if evidence.signal.outcome_label().is_some() {
                    metrics.direct_outcomes += 1;
                }
                match evidence.signal {
                    ObservedSignal::ExplicitPositiveRating => {
                        metrics.positive_direct_outcomes += 1;
                    }
                    ObservedSignal::TaskCompleted => {
                        metrics.positive_direct_outcomes += 1;
                        metrics.completions += 1;
                    }
                    ObservedSignal::UserCorrection => metrics.corrections += 1,
                    ObservedSignal::ClarificationRequest => {
                        metrics.clarification_requests += 1;
                    }
                    ObservedSignal::Abandoned => metrics.abandonments += 1,
                    ObservedSignal::ExplicitNegativeRating | ObservedSignal::NeutralFollowUp => {}
                }
            }
            InteractionOutcomeEvent::PairedEvaluationRecorded {
                trial_id, evidence, ..
            } => {
                let trial = outcomes
                    .trials()
                    .get(trial_id)
                    .ok_or(PolicyEvaluationError::UnknownTrial(*trial_id))?;
                let split = config.split_policy.classify(trial)?;
                record_pairwise_arm_metrics(
                    split_metrics
                        .get_mut(&split)
                        .expect("all splits are initialized"),
                    evidence.left_variant,
                    evidence.right_variant,
                    evidence.preference,
                );
                record_candidate_pairwise(
                    pairwise
                        .get_mut(&split)
                        .expect("all splits are initialized"),
                    evidence.left_variant,
                    evidence.right_variant,
                    evidence.preference,
                );
            }
        }
    }

    let mut splits = Vec::new();
    for split in [
        EvaluationSplit::Development,
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ] {
        let mut arms = split_metrics
            .remove(&split)
            .expect("all splits are initialized");
        for metrics in arms.values_mut() {
            metrics.finalize();
        }
        let mut candidate_pairwise = pairwise
            .remove(&split)
            .expect("all splits are initialized");
        for metrics in candidate_pairwise.values_mut() {
            metrics.finalize();
        }
        splits.push(SplitEvaluationReport {
            split,
            arms,
            candidate_pairwise,
        });
    }

    let mut holdout_comparisons = Vec::new();
    for split_report in splits.iter().filter(|report| report.split.is_holdout()) {
        for control in control_variants() {
            holdout_comparisons.push(compare_candidate_to_control(
                split_report,
                control,
                config,
            ));
        }
    }

    let evidence_sufficient = holdout_comparisons
        .iter()
        .all(|comparison| comparison.gates.evidence_sufficient());
    let performance_passed = holdout_comparisons
        .iter()
        .all(|comparison| comparison.gates.performance_passed());
    let verdict = if !evidence_sufficient {
        PolicyEvaluationVerdict::Inconclusive
    } else if performance_passed {
        PolicyEvaluationVerdict::Pass
    } else {
        PolicyEvaluationVerdict::Fail
    };

    Ok(PolicyEvaluationReport {
        splits,
        holdout_comparisons,
        verdict,
        development_excluded_from_verdict: true,
        promotion_eligible: verdict == PolicyEvaluationVerdict::Pass,
        live_response_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    })
}

fn empty_split_metrics(
) -> BTreeMap<EvaluationSplit, BTreeMap<PolicyVariant, ArmEvaluationMetrics>> {
    [
        EvaluationSplit::Development,
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ]
    .into_iter()
    .map(|split| {
        let arms = PolicyVariant::all()
            .into_iter()
            .map(|variant| (variant, ArmEvaluationMetrics::default()))
            .collect();
        (split, arms)
    })
    .collect()
}

fn empty_pairwise_metrics(
) -> BTreeMap<EvaluationSplit, BTreeMap<PolicyVariant, CandidatePairwiseMetrics>> {
    [
        EvaluationSplit::Development,
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ]
    .into_iter()
    .map(|split| {
        let controls = control_variants()
            .into_iter()
            .map(|variant| (variant, CandidatePairwiseMetrics::default()))
            .collect();
        (split, controls)
    })
    .collect()
}

fn validate_compute_observations(
    outcomes: &InteractionOutcomeLedger,
    compute: &[ArmComputeObservation],
) -> Result<BTreeMap<(InteractionTrialId, PolicyVariant), u64>, PolicyEvaluationError> {
    let expected = outcomes
        .trials()
        .values()
        .flat_map(|trial| trial.arms.iter().map(move |arm| (trial.id, arm.variant)))
        .collect::<BTreeSet<_>>();
    let mut costs = BTreeMap::new();
    for observation in compute {
        let key = (observation.trial_id, observation.variant);
        if !expected.contains(&key) {
            return Err(PolicyEvaluationError::UnexpectedComputeObservation {
                trial_id: observation.trial_id,
                variant: observation.variant,
            });
        }
        if observation.compute_micros == 0 {
            return Err(PolicyEvaluationError::ZeroComputeObservation {
                trial_id: observation.trial_id,
                variant: observation.variant,
            });
        }
        if costs.insert(key, observation.compute_micros).is_some() {
            return Err(PolicyEvaluationError::DuplicateComputeObservation {
                trial_id: observation.trial_id,
                variant: observation.variant,
            });
        }
    }
    for (trial_id, variant) in expected {
        if !costs.contains_key(&(trial_id, variant)) {
            return Err(PolicyEvaluationError::MissingComputeObservation { trial_id, variant });
        }
    }
    Ok(costs)
}

fn record_pairwise_arm_metrics(
    arms: &mut BTreeMap<PolicyVariant, ArmEvaluationMetrics>,
    left: PolicyVariant,
    right: PolicyVariant,
    preference: PairwisePreference,
) {
    match preference {
        PairwisePreference::Left => {
            arms.get_mut(&left).expect("variant exists").pairwise_wins += 1;
            arms.get_mut(&right).expect("variant exists").pairwise_losses += 1;
        }
        PairwisePreference::Right => {
            arms.get_mut(&right).expect("variant exists").pairwise_wins += 1;
            arms.get_mut(&left).expect("variant exists").pairwise_losses += 1;
        }
        PairwisePreference::Tie => {
            arms.get_mut(&left).expect("variant exists").pairwise_ties += 1;
            arms.get_mut(&right).expect("variant exists").pairwise_ties += 1;
        }
    }
}

fn record_candidate_pairwise(
    controls: &mut BTreeMap<PolicyVariant, CandidatePairwiseMetrics>,
    left: PolicyVariant,
    right: PolicyVariant,
    preference: PairwisePreference,
) {
    let candidate = PolicyVariant::CompanionDerived;
    let control = if left == candidate && right != candidate {
        Some((right, true))
    } else if right == candidate && left != candidate {
        Some((left, false))
    } else {
        None
    };
    let Some((control, candidate_is_left)) = control else {
        return;
    };
    let metrics = controls
        .get_mut(&control)
        .expect("all control variants are initialized");
    match preference {
        PairwisePreference::Tie => metrics.ties += 1,
        PairwisePreference::Left if candidate_is_left => metrics.candidate_wins += 1,
        PairwisePreference::Right if !candidate_is_left => metrics.candidate_wins += 1,
        PairwisePreference::Left | PairwisePreference::Right => metrics.control_wins += 1,
    }
}

fn compare_candidate_to_control(
    split_report: &SplitEvaluationReport,
    control: PolicyVariant,
    config: PolicyEvaluationConfig,
) -> CandidateControlComparison {
    let candidate = split_report
        .arms
        .get(&PolicyVariant::CompanionDerived)
        .expect("candidate metrics exist");
    let control_metrics = split_report
        .arms
        .get(&control)
        .expect("control metrics exist");
    let pairwise = split_report
        .candidate_pairwise
        .get(&control)
        .cloned()
        .expect("pairwise metrics exist");

    let brier_improvement_ppm = signed_option_delta(
        control_metrics.mean_brier_score_ppm.map(i64::from),
        candidate.mean_brier_score_ppm.map(i64::from),
    );
    let calibration_regression_bps = signed_option_delta_i32(
        candidate
            .mean_top_label_calibration_error_bps
            .map(i32::from),
        control_metrics
            .mean_top_label_calibration_error_bps
            .map(i32::from),
    );
    let correction_regression_bps = rate_regression(
        candidate.correction_rate_bps,
        control_metrics.correction_rate_bps,
    );
    let clarification_regression_bps = rate_regression(
        candidate.clarification_rate_bps,
        control_metrics.clarification_rate_bps,
    );
    let completion_regression_bps = rate_regression(
        control_metrics.completion_rate_bps,
        candidate.completion_rate_bps,
    );
    let abandonment_regression_bps = rate_regression(
        candidate.abandonment_rate_bps,
        control_metrics.abandonment_rate_bps,
    );
    let abstention_regression_bps = rate_regression(
        candidate.abstention_rate_bps,
        control_metrics.abstention_rate_bps,
    );
    let compute_overhead_bps = relative_overhead_bps(
        candidate.mean_compute_micros,
        control_metrics.mean_compute_micros,
    );

    let gates = ComparisonGates {
        resolved_evidence_sufficient: candidate.resolved_predictions
            >= config.min_resolved_per_arm_per_holdout
            && control_metrics.resolved_predictions >= config.min_resolved_per_arm_per_holdout,
        direct_evidence_sufficient: candidate.direct_outcomes
            >= config.min_direct_outcomes_per_arm_per_holdout
            && control_metrics.direct_outcomes
                >= config.min_direct_outcomes_per_arm_per_holdout,
        pairwise_evidence_sufficient: pairwise.total
            >= config.min_pairwise_comparisons_per_control_per_holdout,
        brier_improvement_passed: brier_improvement_ppm
            .is_some_and(|value| value >= i64::from(config.min_brier_improvement_ppm)),
        calibration_non_regression_passed: calibration_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_calibration_regression_bps)),
        pairwise_margin_passed: pairwise
            .candidate_win_margin_bps
            .is_some_and(|value| value >= i32::from(config.min_pairwise_win_margin_bps)),
        correction_non_regression_passed: correction_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_correction_regression_bps)),
        clarification_non_regression_passed: clarification_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_clarification_regression_bps)),
        completion_non_regression_passed: completion_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_completion_regression_bps)),
        abandonment_non_regression_passed: abandonment_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_abandonment_regression_bps)),
        abstention_non_regression_passed: abstention_regression_bps
            .is_some_and(|value| value <= i32::from(config.max_abstention_regression_bps)),
        compute_overhead_passed: compute_overhead_bps
            .is_some_and(|value| value <= i32::from(config.max_compute_overhead_bps)),
    };

    CandidateControlComparison {
        split: split_report.split,
        control,
        candidate_resolved: candidate.resolved_predictions,
        control_resolved: control_metrics.resolved_predictions,
        candidate_direct_outcomes: candidate.direct_outcomes,
        control_direct_outcomes: control_metrics.direct_outcomes,
        pairwise,
        brier_improvement_ppm,
        calibration_regression_bps,
        correction_regression_bps,
        clarification_regression_bps,
        completion_regression_bps,
        abandonment_regression_bps,
        abstention_regression_bps,
        compute_overhead_bps,
        gates,
    }
}

fn control_variants() -> [PolicyVariant; 5] {
    [
        PolicyVariant::NeutralDefault,
        PolicyVariant::RecencyOnly,
        PolicyVariant::MajorityPrior,
        PolicyVariant::ContextOnly,
        PolicyVariant::ScrambledScope,
    ]
}

fn top_label_calibration_error_bps(
    outcomes: &[crate::companion_prediction_ledger::OutcomeProbability],
    observed_label: &str,
) -> Result<u16, PolicyEvaluationError> {
    let top = outcomes
        .iter()
        .max_by_key(|outcome| outcome.probability_bps)
        .ok_or(PolicyEvaluationError::EmptyPredictionOutcomes)?;
    let observed = if top.label == observed_label { 10_000 } else { 0 };
    Ok(top.probability_bps.abs_diff(observed))
}

fn mean_u64(total: u64, count: u64) -> Option<u64> {
    (count > 0).then_some(total / count)
}

fn mean_u32(total: u64, count: u64) -> Option<u32> {
    (count > 0).then_some((total / count) as u32)
}

fn mean_u16(total: u64, count: u64) -> Option<u16> {
    (count > 0).then_some((total / count) as u16)
}

fn rate_bps(numerator: u64, denominator: u64) -> Option<u16> {
    if denominator == 0 {
        return None;
    }
    Some(((numerator.saturating_mul(BASIS_POINTS)) / denominator) as u16)
}

fn signed_rate_delta_bps(left: u64, right: u64, denominator: u64) -> Option<i32> {
    if denominator == 0 {
        return None;
    }
    let numerator = i128::from(left) - i128::from(right);
    Some((numerator * i128::from(BASIS_POINTS) / i128::from(denominator)) as i32)
}

fn rate_regression(candidate: Option<u16>, control: Option<u16>) -> Option<i32> {
    signed_option_delta_i32(candidate.map(i32::from), control.map(i32::from))
}

fn signed_option_delta(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    Some(left? - right?)
}

fn signed_option_delta_i32(left: Option<i32>, right: Option<i32>) -> Option<i32> {
    Some(left? - right?)
}

fn relative_overhead_bps(candidate: Option<u64>, control: Option<u64>) -> Option<i32> {
    let candidate = i128::from(candidate?);
    let control = i128::from(control?);
    if control == 0 {
        return None;
    }
    Some(((candidate - control) * i128::from(BASIS_POINTS) / control) as i32)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyEvaluationError {
    #[error("temporal holdout boundary must be non-zero")]
    InvalidTemporalBoundary,
    #[error("opaque-subject partition must use modulus >= 2 and remainder < modulus")]
    InvalidSubjectPartition,
    #[error("all holdout evidence minimums must be non-zero")]
    InvalidEvidenceMinimum,
    #[error("basis-point threshold {0} exceeds 10,000")]
    BasisPointsOutOfRange(u16),
    #[error("unknown trial {0}")]
    UnknownTrial(InteractionTrialId),
    #[error("prediction {0} referenced by a trial is absent")]
    MissingPrediction(u64),
    #[error("trial {trial_id} has malformed arm {variant:?}")]
    MalformedTrialArm {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("missing compute observation for trial {trial_id}, arm {variant:?}")]
    MissingComputeObservation {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("duplicate compute observation for trial {trial_id}, arm {variant:?}")]
    DuplicateComputeObservation {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("unexpected compute observation for trial {trial_id}, arm {variant:?}")]
    UnexpectedComputeObservation {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("zero compute observation for trial {trial_id}, arm {variant:?}")]
    ZeroComputeObservation {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("resolved prediction contains no outcomes")]
    EmptyPredictionOutcomes,
    #[error("evaluation metric overflow")]
    MetricOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_interaction_outcomes::TrialArm;

    fn trial(issued_at_ms: u64, subject_scope_digest: u64) -> InteractionTrial {
        InteractionTrial {
            id: 1,
            source_companion_version: 1,
            context_digest: 10,
            subject_scope_digest,
            issued_at_ms,
            not_before_ms: issued_at_ms + 1,
            expires_at_ms: issued_at_ms + 2,
            delivered_variant: None,
            arms: PolicyVariant::all()
                .into_iter()
                .enumerate()
                .map(|(index, variant)| TrialArm {
                    variant,
                    policy_digest_fnv1a64: index as u64 + 1,
                    prediction_id: Some(index as u64 + 1),
                    abstention_id: None,
                })
                .collect(),
        }
    }

    #[test]
    fn temporal_holdout_precedes_subject_partition() {
        let policy = EvaluationSplitPolicy {
            temporal_holdout_start_ms: 100,
            opaque_subject_modulus: 2,
            opaque_subject_remainder: 1,
        };
        assert_eq!(
            policy.classify(&trial(100, 1)).unwrap(),
            EvaluationSplit::TemporalHoldout
        );
        assert_eq!(
            policy.classify(&trial(99, 1)).unwrap(),
            EvaluationSplit::OpaqueSubjectHoldout
        );
        assert_eq!(
            policy.classify(&trial(99, 2)).unwrap(),
            EvaluationSplit::Development
        );
    }

    #[test]
    fn split_policy_rejects_invalid_partition() {
        let policy = EvaluationSplitPolicy {
            temporal_holdout_start_ms: 100,
            opaque_subject_modulus: 1,
            opaque_subject_remainder: 0,
        };
        assert_eq!(
            policy.validate(),
            Err(PolicyEvaluationError::InvalidSubjectPartition)
        );
    }

    #[test]
    fn pairwise_orientation_is_candidate_relative() {
        let mut metrics = control_variants()
            .into_iter()
            .map(|variant| (variant, CandidatePairwiseMetrics::default()))
            .collect::<BTreeMap<_, _>>();
        record_candidate_pairwise(
            &mut metrics,
            PolicyVariant::NeutralDefault,
            PolicyVariant::CompanionDerived,
            PairwisePreference::Right,
        );
        let neutral = metrics.get_mut(&PolicyVariant::NeutralDefault).unwrap();
        neutral.finalize();
        assert_eq!(neutral.candidate_wins, 1);
        assert_eq!(neutral.candidate_win_margin_bps, Some(10_000));
    }
}
