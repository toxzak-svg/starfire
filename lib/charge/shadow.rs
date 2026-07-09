use std::cmp::Ordering;
use std::collections::BTreeSet;

use thiserror::Error;

use super::induction::{
    EmpiricalInductionConfig, EmpiricalOntologyInducer, LearnedOntology, OntologyInductionError,
    OntologyObservation,
};
use super::ontology::ConceptId;

const SCORE_EPSILON: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowPromotionConfig {
    pub training_windows: usize,
    pub holdout_windows: usize,
    pub transfer_windows: usize,
    pub min_promoted_concepts: usize,
    pub min_transfer_efficiency_ratio: f64,
    pub min_transfer_win_fraction: f64,
    pub min_worst_window_ratio: f64,
    pub min_control_efficiency_ratio: f64,
    pub induction: EmpiricalInductionConfig,
}

impl Default for ShadowPromotionConfig {
    fn default() -> Self {
        Self {
            training_windows: 3,
            holdout_windows: 1,
            transfer_windows: 4,
            min_promoted_concepts: 2,
            min_transfer_efficiency_ratio: 1.25,
            min_transfer_win_fraction: 0.75,
            min_worst_window_ratio: 0.90,
            min_control_efficiency_ratio: 1.25,
            induction: EmpiricalInductionConfig::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ShadowPromotionError {
    #[error("shadow promotion window must contain at least one observation")]
    EmptyWindow,
    #[error("training_windows, holdout_windows, and transfer_windows must all be greater than zero")]
    ZeroWindowRequirement,
    #[error("shadow promotion ratio and fraction gates must be finite and non-negative")]
    InvalidGate,
    #[error("shadow trial is already terminal")]
    TrialAlreadyTerminal,
    #[error("shadow transfer has not completed; controls cannot be assessed")]
    TransferIncomplete,
    #[error("at least one matched-budget control is required")]
    MissingControls,
    #[error(
        "control {name} used proposal budget {actual}; expected exactly {expected} proposal evaluations"
    )]
    ProposalBudgetMismatch {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "control {name} used routing budget {actual}; expected exactly {expected} future routing evaluations"
    )]
    RoutingBudgetMismatch {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("control {name} reported a non-finite or negative future efficiency")]
    InvalidControlEfficiency { name: String },
    #[error(transparent)]
    Induction(#[from] OntologyInductionError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowPromotionStatus {
    CollectingTraining,
    CollectingHoldout,
    CollectingTransfer,
    AwaitingMatchedBudgetControls,
    Eligible,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowWindowMetrics {
    pub observations: usize,
    pub shadow_efficiency: f64,
    pub baseline_efficiency: f64,
    pub efficiency_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowTransferSummary {
    pub windows: usize,
    pub observations: usize,
    pub shadow_efficiency: f64,
    pub baseline_efficiency: f64,
    pub efficiency_ratio: f64,
    pub window_win_fraction: f64,
    pub worst_window_ratio: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowControlScore {
    pub name: String,
    pub proposal_evaluations: usize,
    pub routing_evaluations: usize,
    pub mean_future_efficiency: f64,
}

impl ShadowControlScore {
    pub fn new(
        name: impl Into<String>,
        proposal_evaluations: usize,
        routing_evaluations: usize,
        mean_future_efficiency: f64,
    ) -> Self {
        Self {
            name: name.into(),
            proposal_evaluations,
            routing_evaluations,
            mean_future_efficiency,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowControlComparison {
    pub name: String,
    pub efficiency_ratio: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowBudget {
    pub proposal_evaluations: usize,
    pub routing_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPromotionCriteria {
    pub promoted_concepts: bool,
    pub transfer_efficiency: bool,
    pub transfer_window_wins: bool,
    pub worst_window: bool,
    pub matched_budget_controls: bool,
}

impl ShadowPromotionCriteria {
    pub fn all_pass(&self) -> bool {
        self.promoted_concepts
            && self.transfer_efficiency
            && self.transfer_window_wins
            && self.worst_window
            && self.matched_budget_controls
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowPromotionAssessment {
    pub status: ShadowPromotionStatus,
    pub concept_ids: Vec<ConceptId>,
    pub transfer: ShadowTransferSummary,
    pub budget: ShadowBudget,
    pub controls: Vec<ShadowControlComparison>,
    pub criteria: ShadowPromotionCriteria,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShadowUpdate {
    pub status: ShadowPromotionStatus,
    pub training_windows: usize,
    pub holdout_windows: usize,
    pub transfer_windows: usize,
    pub latest_transfer: Option<ShadowWindowMetrics>,
}

/// Online shadow evaluator for automatically induced concepts.
///
/// This monitor may fit and evaluate candidate ontologies, but it deliberately
/// exposes no API that mutates the live CHARGE router. A candidate can become
/// `Eligible` only after future-window transfer and exact matched-budget controls.
#[derive(Debug, Clone)]
pub struct ShadowPromotionMonitor {
    config: ShadowPromotionConfig,
    training: Vec<Vec<OntologyObservation>>,
    holdout: Vec<Vec<OntologyObservation>>,
    candidate: Option<LearnedOntology>,
    baseline_resolver: Option<String>,
    transfer: Vec<ShadowWindowMetrics>,
    terminal: Option<ShadowPromotionAssessment>,
}

impl ShadowPromotionMonitor {
    pub fn new(config: ShadowPromotionConfig) -> Result<Self, ShadowPromotionError> {
        validate_config(config)?;
        Ok(Self {
            config,
            training: Vec::new(),
            holdout: Vec::new(),
            candidate: None,
            baseline_resolver: None,
            transfer: Vec::new(),
            terminal: None,
        })
    }

    pub fn config(&self) -> ShadowPromotionConfig {
        self.config
    }

    pub fn status(&self) -> ShadowPromotionStatus {
        if let Some(assessment) = &self.terminal {
            return assessment.status;
        }
        if self.training.len() < self.config.training_windows {
            return ShadowPromotionStatus::CollectingTraining;
        }
        if self.holdout.len() < self.config.holdout_windows {
            return ShadowPromotionStatus::CollectingHoldout;
        }
        if self.transfer.len() < self.config.transfer_windows {
            return ShadowPromotionStatus::CollectingTransfer;
        }
        ShadowPromotionStatus::AwaitingMatchedBudgetControls
    }

    pub fn observe_window(
        &mut self,
        window: Vec<OntologyObservation>,
    ) -> Result<ShadowUpdate, ShadowPromotionError> {
        if window.is_empty() {
            return Err(ShadowPromotionError::EmptyWindow);
        }
        if self.terminal.is_some() {
            return Err(ShadowPromotionError::TrialAlreadyTerminal);
        }

        let mut latest_transfer = None;
        if self.training.len() < self.config.training_windows {
            self.training.push(window);
        } else if self.holdout.len() < self.config.holdout_windows {
            self.holdout.push(window);
            if self.holdout.len() == self.config.holdout_windows {
                self.fit_candidate()?;
            }
        } else if self.transfer.len() < self.config.transfer_windows {
            let metrics = self.evaluate_future_window(&window);
            self.transfer.push(metrics);
            latest_transfer = Some(metrics);
        } else {
            return Err(ShadowPromotionError::TrialAlreadyTerminal);
        }

        Ok(ShadowUpdate {
            status: self.status(),
            training_windows: self.training.len(),
            holdout_windows: self.holdout.len(),
            transfer_windows: self.transfer.len(),
            latest_transfer,
        })
    }

    pub fn learned_ontology(&self) -> Option<&LearnedOntology> {
        self.candidate.as_ref()
    }

    pub fn transfer_windows(&self) -> &[ShadowWindowMetrics] {
        &self.transfer
    }

    pub fn transfer_summary(&self) -> Option<ShadowTransferSummary> {
        if self.transfer.len() < self.config.transfer_windows {
            return None;
        }
        Some(summarize_transfer(&self.transfer))
    }

    pub fn required_control_budget(&self) -> Option<ShadowBudget> {
        let candidate = self.candidate.as_ref()?;
        let transfer = self.transfer_summary()?;
        Some(ShadowBudget {
            proposal_evaluations: candidate.summary().candidates_considered,
            routing_evaluations: transfer.observations,
        })
    }

    pub fn assess_controls(
        &mut self,
        controls: &[ShadowControlScore],
    ) -> Result<&ShadowPromotionAssessment, ShadowPromotionError> {
        if self.terminal.is_some() {
            return Ok(self.terminal.as_ref().unwrap());
        }
        let transfer = self
            .transfer_summary()
            .ok_or(ShadowPromotionError::TransferIncomplete)?;
        if controls.is_empty() {
            return Err(ShadowPromotionError::MissingControls);
        }
        let budget = self.required_control_budget().unwrap();
        let candidate = self.candidate.as_ref().unwrap();

        let mut comparisons = Vec::with_capacity(controls.len());
        for control in controls {
            if control.proposal_evaluations != budget.proposal_evaluations {
                return Err(ShadowPromotionError::ProposalBudgetMismatch {
                    name: control.name.clone(),
                    expected: budget.proposal_evaluations,
                    actual: control.proposal_evaluations,
                });
            }
            if control.routing_evaluations != budget.routing_evaluations {
                return Err(ShadowPromotionError::RoutingBudgetMismatch {
                    name: control.name.clone(),
                    expected: budget.routing_evaluations,
                    actual: control.routing_evaluations,
                });
            }
            if !control.mean_future_efficiency.is_finite()
                || control.mean_future_efficiency < 0.0
            {
                return Err(ShadowPromotionError::InvalidControlEfficiency {
                    name: control.name.clone(),
                });
            }
            let efficiency_ratio = ratio(
                transfer.shadow_efficiency,
                control.mean_future_efficiency,
            );
            comparisons.push(ShadowControlComparison {
                name: control.name.clone(),
                efficiency_ratio,
                passed: efficiency_ratio + SCORE_EPSILON
                    >= self.config.min_control_efficiency_ratio,
            });
        }

        let criteria = ShadowPromotionCriteria {
            promoted_concepts: candidate.summary().promoted_concepts
                >= self.config.min_promoted_concepts,
            transfer_efficiency: transfer.efficiency_ratio + SCORE_EPSILON
                >= self.config.min_transfer_efficiency_ratio,
            transfer_window_wins: transfer.window_win_fraction + SCORE_EPSILON
                >= self.config.min_transfer_win_fraction,
            worst_window: transfer.worst_window_ratio + SCORE_EPSILON
                >= self.config.min_worst_window_ratio,
            matched_budget_controls: comparisons.iter().all(|control| control.passed),
        };
        let status = if criteria.all_pass() {
            ShadowPromotionStatus::Eligible
        } else {
            ShadowPromotionStatus::Rejected
        };
        let concept_ids = candidate
            .routes()
            .iter()
            .map(|route| route.concept.id)
            .collect();

        self.terminal = Some(ShadowPromotionAssessment {
            status,
            concept_ids,
            transfer,
            budget,
            controls: comparisons,
            criteria,
        });
        Ok(self.terminal.as_ref().unwrap())
    }

    pub fn assessment(&self) -> Option<&ShadowPromotionAssessment> {
        self.terminal.as_ref()
    }

    fn fit_candidate(&mut self) -> Result<(), ShadowPromotionError> {
        let training = flatten_windows(&self.training);
        let holdout = flatten_windows(&self.holdout);
        let baseline_resolver = best_global_resolver(&training);
        let candidate = EmpiricalOntologyInducer::new(self.config.induction)
            .fit(&training, &holdout)?;
        self.baseline_resolver = Some(baseline_resolver);
        self.candidate = Some(candidate);
        Ok(())
    }

    fn evaluate_future_window(&self, window: &[OntologyObservation]) -> ShadowWindowMetrics {
        let candidate = self
            .candidate
            .as_ref()
            .expect("candidate is fit before transfer windows");
        let baseline = self
            .baseline_resolver
            .as_deref()
            .expect("baseline resolver is fit with the candidate");
        let shadow_efficiency = mean_efficiency(window, |observation| {
            candidate.route(&observation.charge).resolver
        });
        let baseline_efficiency = mean_efficiency(window, |_| baseline.to_string());

        ShadowWindowMetrics {
            observations: window.len(),
            shadow_efficiency,
            baseline_efficiency,
            efficiency_ratio: ratio(shadow_efficiency, baseline_efficiency),
        }
    }
}

fn validate_config(config: ShadowPromotionConfig) -> Result<(), ShadowPromotionError> {
    if config.training_windows == 0
        || config.holdout_windows == 0
        || config.transfer_windows == 0
    {
        return Err(ShadowPromotionError::ZeroWindowRequirement);
    }
    for gate in [
        config.min_transfer_efficiency_ratio,
        config.min_transfer_win_fraction,
        config.min_worst_window_ratio,
        config.min_control_efficiency_ratio,
    ] {
        if !gate.is_finite() || gate < 0.0 {
            return Err(ShadowPromotionError::InvalidGate);
        }
    }
    Ok(())
}

fn flatten_windows(windows: &[Vec<OntologyObservation>]) -> Vec<OntologyObservation> {
    windows
        .iter()
        .flat_map(|window| window.iter().cloned())
        .collect()
}

fn resolver_names(observations: &[OntologyObservation]) -> Vec<String> {
    observations
        .iter()
        .flat_map(|observation| observation.outcomes.iter())
        .map(|outcome| outcome.resolver.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn best_global_resolver(observations: &[OntologyObservation]) -> String {
    let resolvers = resolver_names(observations);
    resolvers
        .iter()
        .max_by(|left, right| {
            mean_efficiency(observations, |_| (*left).clone())
                .partial_cmp(&mean_efficiency(observations, |_| (*right).clone()))
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .cloned()
        .unwrap_or_default()
}

fn mean_efficiency(
    observations: &[OntologyObservation],
    resolver: impl Fn(&OntologyObservation) -> String,
) -> f64 {
    if observations.is_empty() {
        return 0.0;
    }
    observations
        .iter()
        .map(|observation| resolver_score(observation, &resolver(observation)))
        .sum::<f64>()
        / observations.len() as f64
}

fn resolver_score(observation: &OntologyObservation, resolver: &str) -> f64 {
    let mut total = 0.0;
    let mut attempts = 0usize;
    for outcome in observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
    {
        let discharge_fraction = (outcome.discharged as f64 / observation.charge.magnitude as f64)
            .clamp(0.0, 1.0);
        total += discharge_fraction / outcome.compute_cost as f64;
        attempts += 1;
    }
    if attempts == 0 {
        0.0
    } else {
        total / attempts as f64
    }
}

fn summarize_transfer(windows: &[ShadowWindowMetrics]) -> ShadowTransferSummary {
    let observations = windows.iter().map(|window| window.observations).sum::<usize>();
    let weighted_shadow = windows
        .iter()
        .map(|window| window.shadow_efficiency * window.observations as f64)
        .sum::<f64>();
    let weighted_baseline = windows
        .iter()
        .map(|window| window.baseline_efficiency * window.observations as f64)
        .sum::<f64>();
    let wins = windows
        .iter()
        .filter(|window| window.shadow_efficiency > window.baseline_efficiency + SCORE_EPSILON)
        .count();
    let worst_window_ratio = windows
        .iter()
        .map(|window| window.efficiency_ratio)
        .fold(f64::INFINITY, f64::min);
    let shadow_efficiency = weighted_shadow / observations.max(1) as f64;
    let baseline_efficiency = weighted_baseline / observations.max(1) as f64;

    ShadowTransferSummary {
        windows: windows.len(),
        observations,
        shadow_efficiency,
        baseline_efficiency,
        efficiency_ratio: ratio(shadow_efficiency, baseline_efficiency),
        window_win_fraction: wins as f64 / windows.len().max(1) as f64,
        worst_window_ratio,
    }
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        if numerator.abs() <= f64::EPSILON {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{
        Charge, ChargeKind, ChargeScope, PromotionCriteria, ResolverOutcome,
    };

    fn config() -> ShadowPromotionConfig {
        ShadowPromotionConfig {
            training_windows: 1,
            holdout_windows: 1,
            transfer_windows: 2,
            min_promoted_concepts: 2,
            min_transfer_efficiency_ratio: 1.5,
            min_transfer_win_fraction: 1.0,
            min_worst_window_ratio: 1.5,
            min_control_efficiency_ratio: 1.25,
            induction: EmpiricalInductionConfig {
                max_concepts: 2,
                min_partition_support: 8,
                min_holdout_support: 8,
                max_thresholds_per_dimension: 64,
                complexity_penalty: 0.001,
                promotion: PromotionCriteria {
                    min_observations: 8,
                    min_holdout_gain: 0.05,
                    min_total_utility_gain: 0.05,
                },
            },
        }
    }

    fn window(offset: u64, per_class: usize, future_regression: bool) -> Vec<OntologyObservation> {
        let mut observations = Vec::new();
        let mut id = offset;
        for index in 0..per_class {
            let jitter = index as f32 * 0.002;
            observations.push(observation(
                id,
                vec![0.88 - jitter, 0.14 + jitter, 0.18],
                if future_regression {
                    [0.10, 0.92, 0.08]
                } else {
                    [0.92, 0.10, 0.08]
                },
            ));
            id += 1;
        }
        for index in 0..per_class {
            let jitter = index as f32 * 0.002;
            observations.push(observation(
                id,
                vec![0.16, 0.88 - jitter, 0.20 + jitter],
                [0.09, 0.93, 0.09],
            ));
            id += 1;
        }
        for index in 0..per_class {
            let jitter = index as f32 * 0.002;
            observations.push(observation(
                id,
                vec![0.18 + jitter, 0.18, 0.89 - jitter],
                [0.08, 0.10, 0.94],
            ));
            id += 1;
        }
        observations
    }

    fn observation(id: u64, residual: Vec<f32>, outcomes: [f32; 3]) -> OntologyObservation {
        let mut charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            residual,
            1.0,
            ChargeScope::Global,
        );
        charge.id = id;
        OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("memory", outcomes[0], 1))
            .with_outcome(ResolverOutcome::new("reasoning", outcomes[1], 1))
            .with_outcome(ResolverOutcome::new("causal", outcomes[2], 1))
    }

    #[test]
    fn automatic_shadow_fit_never_becomes_eligible_before_future_controls() {
        let mut monitor = ShadowPromotionMonitor::new(config()).unwrap();
        monitor.observe_window(window(1, 16, false)).unwrap();
        monitor.observe_window(window(1_000, 16, false)).unwrap();
        monitor.observe_window(window(2_000, 16, false)).unwrap();
        let update = monitor.observe_window(window(3_000, 16, false)).unwrap();

        assert_eq!(
            update.status,
            ShadowPromotionStatus::AwaitingMatchedBudgetControls
        );
        assert!(monitor.assessment().is_none());
        assert_eq!(monitor.learned_ontology().unwrap().routes().len(), 2);
    }

    #[test]
    fn matched_budget_controls_can_make_a_transferring_shadow_candidate_eligible() {
        let mut monitor = ShadowPromotionMonitor::new(config()).unwrap();
        for (offset, regression) in [(1, false), (1_000, false), (2_000, false), (3_000, false)] {
            monitor.observe_window(window(offset, 16, regression)).unwrap();
        }
        let budget = monitor.required_control_budget().unwrap();
        let controls = vec![
            ShadowControlScore::new(
                "random",
                budget.proposal_evaluations,
                budget.routing_evaluations,
                0.36,
            ),
            ShadowControlScore::new(
                "permuted",
                budget.proposal_evaluations,
                budget.routing_evaluations,
                0.37,
            ),
        ];

        let assessment = monitor.assess_controls(&controls).unwrap();

        assert_eq!(assessment.status, ShadowPromotionStatus::Eligible);
        assert!(assessment.criteria.all_pass());
        assert_eq!(assessment.concept_ids.len(), 2);
    }

    #[test]
    fn exact_control_budget_mismatch_is_rejected() {
        let mut monitor = ShadowPromotionMonitor::new(config()).unwrap();
        for offset in [1, 1_000, 2_000, 3_000] {
            monitor.observe_window(window(offset, 16, false)).unwrap();
        }
        let budget = monitor.required_control_budget().unwrap();
        let error = monitor
            .assess_controls(&[ShadowControlScore::new(
                "cheap-random",
                budget.proposal_evaluations.saturating_sub(1),
                budget.routing_evaluations,
                0.1,
            )])
            .unwrap_err();

        assert!(matches!(
            error,
            ShadowPromotionError::ProposalBudgetMismatch { .. }
        ));
    }

    #[test]
    fn future_window_regression_rejects_shadow_promotion() {
        let mut monitor = ShadowPromotionMonitor::new(config()).unwrap();
        monitor.observe_window(window(1, 16, false)).unwrap();
        monitor.observe_window(window(1_000, 16, false)).unwrap();
        monitor.observe_window(window(2_000, 16, true)).unwrap();
        monitor.observe_window(window(3_000, 16, true)).unwrap();
        let budget = monitor.required_control_budget().unwrap();
        let controls = vec![ShadowControlScore::new(
            "matched",
            budget.proposal_evaluations,
            budget.routing_evaluations,
            0.30,
        )];

        let assessment = monitor.assess_controls(&controls).unwrap();

        assert_eq!(assessment.status, ShadowPromotionStatus::Rejected);
        assert!(!assessment.criteria.transfer_window_wins);
    }
}
