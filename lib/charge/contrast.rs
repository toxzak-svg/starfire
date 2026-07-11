//! Disagreement-induced contrast probes for unresolved CHARGE state.
//!
//! The operation in this module is deliberately narrower than ontology induction.
//! Two historical unresolved states are allowed to interact only when their
//! independently measured resolver-utility distributions disagree. Their residual
//! difference is normalized into a new projection axis and their projected midpoint
//! becomes an executable binary discrimination test.
//!
//! The pair's resolver labels do not become the route. Candidate axes are evaluated
//! on the full training cohort and each side learns its own empirical resolver.
//! Holdout gating is mandatory. This makes resolver disagreement a source of new
//! representation, rather than merely another feature consumed by a static router.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use thiserror::Error;

use super::{Charge, OntologyObservation};

const SCORE_EPSILON: f64 = 1e-12;
const NORM_EPSILON: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastProbeConfig {
    /// Minimum total-variation distance between normalized resolver-utility vectors
    /// before two unresolved states may manufacture a contrast axis.
    pub min_preference_disagreement: f64,
    /// Minimum observations required on each side of a training projection.
    pub min_partition_support: usize,
    /// Minimum observations required on each side of the independent holdout.
    pub min_holdout_support: usize,
    /// Training-only complexity penalty.
    pub complexity_penalty: f64,
    /// Independent holdout gain required before the probe becomes executable.
    pub min_holdout_gain: f64,
}

impl Default for ContrastProbeConfig {
    fn default() -> Self {
        Self {
            min_preference_disagreement: 0.25,
            min_partition_support: 12,
            min_holdout_support: 6,
            complexity_penalty: 0.003,
            min_holdout_gain: 0.04,
        }
    }
}

#[derive(Debug, Error)]
pub enum ContrastProbeError {
    #[error("training observations are empty")]
    EmptyTraining,
    #[error("holdout observations are empty")]
    EmptyHoldout,
    #[error("contrast-probe configuration contains an invalid finite/non-negative value")]
    InvalidConfig,
    #[error("training observations expose no resolver outcomes")]
    NoResolvers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeSide {
    Lower,
    Upper,
}

/// A representation manufactured by the interaction of two unresolved states.
///
/// `axis` and `threshold` define the executable question:
///
/// `dot(axis, future_residual) <= threshold ? lower : upper`
#[derive(Debug, Clone, PartialEq)]
pub struct TensionContrast {
    pub left_index: usize,
    pub right_index: usize,
    pub axis: Vec<f32>,
    pub threshold: f32,
    pub source_preference_disagreement: f64,
}

impl TensionContrast {
    pub fn projection(&self, charge: &Charge) -> Option<f64> {
        dot(&self.axis, &charge.residual)
    }

    pub fn side(&self, charge: &Charge) -> Option<ProbeSide> {
        self.projection(charge).map(|projection| {
            if projection <= self.threshold as f64 {
                ProbeSide::Lower
            } else {
                ProbeSide::Upper
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LearnedContrastProbe {
    pub contrast: TensionContrast,
    pub lower_resolver: String,
    pub upper_resolver: String,
    pub lower_training_support: usize,
    pub upper_training_support: usize,
    pub lower_holdout_support: usize,
    pub upper_holdout_support: usize,
    pub training_efficiency: f64,
    pub holdout_efficiency: f64,
    pub holdout_gain: f64,
}

impl LearnedContrastProbe {
    pub fn route(&self, charge: &Charge) -> &str {
        match self.contrast.side(charge) {
            Some(ProbeSide::Lower) => self.lower_resolver.as_str(),
            Some(ProbeSide::Upper) => self.upper_resolver.as_str(),
            None => self.lower_resolver.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContrastProbeFit {
    pub baseline_resolver: String,
    pub baseline_training_efficiency: f64,
    pub baseline_holdout_efficiency: f64,
    pub best_training_efficiency: f64,
    pub best_training_gain_after_penalty: f64,
    pub proposal_evaluations: usize,
    pub probe: Option<LearnedContrastProbe>,
}

impl ContrastProbeFit {
    pub fn route(&self, charge: &Charge) -> &str {
        match &self.probe {
            Some(probe) => probe.route(charge),
            None => self.baseline_resolver.as_str(),
        }
    }

    pub fn applied(&self) -> bool {
        self.probe.is_some()
    }
}

/// Produce the pair schedule used by the real primitive.
///
/// Pairs are admitted only when:
/// - both residuals can produce a non-degenerate contrast axis,
/// - their best resolvers differ, and
/// - normalized resolver utility differs by the configured amount.
///
/// No concept label, charge kind, scope name, or task class is inspected.
pub fn disagreement_pair_schedule(
    observations: &[OntologyObservation],
    min_preference_disagreement: f64,
) -> Vec<(usize, usize)> {
    let resolvers = resolver_names(observations);
    let mut pairs = Vec::new();

    for left in 0..observations.len() {
        for right in (left + 1)..observations.len() {
            if contrast_axis_from_pair(observations, left, right).is_none() {
                continue;
            }
            let Some((disagreement, left_leader, right_leader)) =
                preference_disagreement(&observations[left], &observations[right], &resolvers)
            else {
                continue;
            };
            if left_leader != right_leader
                && disagreement + SCORE_EPSILON >= min_preference_disagreement
            {
                pairs.push((left, right));
            }
        }
    }

    pairs
}

/// Return every pair that can manufacture a finite, non-degenerate residual axis.
///
/// This is useful for exact-budget controls that must receive the same number of
/// pair interactions while ignoring resolver disagreement.
pub fn valid_contrast_pairs(observations: &[OntologyObservation]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for left in 0..observations.len() {
        for right in (left + 1)..observations.len() {
            if contrast_axis_from_pair(observations, left, right).is_some() {
                pairs.push((left, right));
            }
        }
    }
    pairs
}

/// Fit the real disagreement-induced primitive.
pub fn fit_disagreement_contrast(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    config: ContrastProbeConfig,
) -> Result<ContrastProbeFit, ContrastProbeError> {
    validate(train, holdout, config)?;
    let pairs = disagreement_pair_schedule(train, config.min_preference_disagreement);
    fit_contrast_from_pairs(train, holdout, &pairs, config)
}

/// Fit contrast probes from an explicit pair schedule.
///
/// This is intentionally public for matched-budget falsification controls. The
/// fitting and holdout gate are identical to the real primitive; only the source
/// of pair interactions changes.
pub fn fit_contrast_from_pairs(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    pair_schedule: &[(usize, usize)],
    config: ContrastProbeConfig,
) -> Result<ContrastProbeFit, ContrastProbeError> {
    validate(train, holdout, config)?;

    let resolvers = resolver_names(train);
    if resolvers.is_empty() {
        return Err(ContrastProbeError::NoResolvers);
    }
    let all_train: Vec<usize> = (0..train.len()).collect();
    let baseline_resolver = best_resolver(train, &all_train, &resolvers);
    let baseline_training_efficiency =
        mean_efficiency(train, |_| baseline_resolver.as_str());
    let baseline_holdout_efficiency =
        mean_efficiency(holdout, |_| baseline_resolver.as_str());

    let mut best: Option<ProbeCandidate> = None;
    for &(left, right) in pair_schedule {
        let Some(contrast) = contrast_axis_from_pair(train, left, right) else {
            continue;
        };
        let (lower_indices, upper_indices) = partition_indices(train, &contrast);
        if lower_indices.len() < config.min_partition_support
            || upper_indices.len() < config.min_partition_support
        {
            continue;
        }

        let lower_resolver = best_resolver(train, &lower_indices, &resolvers);
        let upper_resolver = best_resolver(train, &upper_indices, &resolvers);
        if lower_resolver == upper_resolver {
            continue;
        }

        let training_efficiency = mean_efficiency(train, |observation| {
            route_contrast(
                &contrast,
                observation,
                lower_resolver.as_str(),
                upper_resolver.as_str(),
            )
        });
        let training_gain_after_penalty = training_efficiency
            - baseline_training_efficiency
            - config.complexity_penalty;

        let candidate = ProbeCandidate {
            contrast,
            lower_resolver,
            upper_resolver,
            lower_training_support: lower_indices.len(),
            upper_training_support: upper_indices.len(),
            training_efficiency,
            training_gain_after_penalty,
        };
        if best
            .as_ref()
            .is_none_or(|current| candidate_is_better(&candidate, current))
        {
            best = Some(candidate);
        }
    }

    let best_training_efficiency = best
        .as_ref()
        .map(|candidate| candidate.training_efficiency)
        .unwrap_or(baseline_training_efficiency);
    let best_training_gain_after_penalty = best
        .as_ref()
        .map(|candidate| candidate.training_gain_after_penalty)
        .unwrap_or(0.0);

    let probe = best.and_then(|candidate| {
        if candidate.training_gain_after_penalty <= SCORE_EPSILON {
            return None;
        }
        let (lower_holdout, upper_holdout) = partition_indices(holdout, &candidate.contrast);
        if lower_holdout.len() < config.min_holdout_support
            || upper_holdout.len() < config.min_holdout_support
        {
            return None;
        }

        let holdout_efficiency = mean_efficiency(holdout, |observation| {
            route_contrast(
                &candidate.contrast,
                observation,
                candidate.lower_resolver.as_str(),
                candidate.upper_resolver.as_str(),
            )
        });
        let holdout_gain = holdout_efficiency - baseline_holdout_efficiency;
        if holdout_gain + SCORE_EPSILON < config.min_holdout_gain {
            return None;
        }

        Some(LearnedContrastProbe {
            contrast: candidate.contrast,
            lower_resolver: candidate.lower_resolver,
            upper_resolver: candidate.upper_resolver,
            lower_training_support: candidate.lower_training_support,
            upper_training_support: candidate.upper_training_support,
            lower_holdout_support: lower_holdout.len(),
            upper_holdout_support: upper_holdout.len(),
            training_efficiency: candidate.training_efficiency,
            holdout_efficiency,
            holdout_gain,
        })
    });

    Ok(ContrastProbeFit {
        baseline_resolver,
        baseline_training_efficiency,
        baseline_holdout_efficiency,
        best_training_efficiency,
        best_training_gain_after_penalty,
        proposal_evaluations: pair_schedule.len(),
        probe,
    })
}

#[derive(Debug, Clone)]
struct ProbeCandidate {
    contrast: TensionContrast,
    lower_resolver: String,
    upper_resolver: String,
    lower_training_support: usize,
    upper_training_support: usize,
    training_efficiency: f64,
    training_gain_after_penalty: f64,
}

fn validate(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    config: ContrastProbeConfig,
) -> Result<(), ContrastProbeError> {
    if train.is_empty() {
        return Err(ContrastProbeError::EmptyTraining);
    }
    if holdout.is_empty() {
        return Err(ContrastProbeError::EmptyHoldout);
    }
    if !config.min_preference_disagreement.is_finite()
        || config.min_preference_disagreement < 0.0
        || config.min_preference_disagreement > 1.0
        || !config.complexity_penalty.is_finite()
        || config.complexity_penalty < 0.0
        || !config.min_holdout_gain.is_finite()
        || config.min_holdout_gain < 0.0
    {
        return Err(ContrastProbeError::InvalidConfig);
    }
    Ok(())
}

fn contrast_axis_from_pair(
    observations: &[OntologyObservation],
    left_index: usize,
    right_index: usize,
) -> Option<TensionContrast> {
    let left = observations.get(left_index)?;
    let right = observations.get(right_index)?;
    let left_residual = &left.charge.residual;
    let right_residual = &right.charge.residual;
    if left_residual.is_empty() || left_residual.len() != right_residual.len() {
        return None;
    }
    if left_residual
        .iter()
        .chain(right_residual.iter())
        .any(|value| !value.is_finite())
    {
        return None;
    }

    let delta: Vec<f64> = right_residual
        .iter()
        .zip(left_residual.iter())
        .map(|(right, left)| f64::from(*right) - f64::from(*left))
        .collect();
    let norm = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm <= NORM_EPSILON {
        return None;
    }
    let axis: Vec<f32> = delta.iter().map(|value| (value / norm) as f32).collect();
    let left_projection = dot(&axis, left_residual)?;
    let right_projection = dot(&axis, right_residual)?;
    let threshold = ((left_projection + right_projection) * 0.5) as f32;

    let resolvers = resolver_names(observations);
    let source_preference_disagreement =
        preference_disagreement(left, right, &resolvers)
            .map(|(distance, _, _)| distance)
            .unwrap_or(0.0);

    Some(TensionContrast {
        left_index,
        right_index,
        axis,
        threshold,
        source_preference_disagreement,
    })
}

fn partition_indices(
    observations: &[OntologyObservation],
    contrast: &TensionContrast,
) -> (Vec<usize>, Vec<usize>) {
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (index, observation) in observations.iter().enumerate() {
        match contrast.side(&observation.charge) {
            Some(ProbeSide::Lower) => lower.push(index),
            Some(ProbeSide::Upper) => upper.push(index),
            None => {}
        }
    }
    (lower, upper)
}

fn route_contrast<'a>(
    contrast: &TensionContrast,
    observation: &OntologyObservation,
    lower_resolver: &'a str,
    upper_resolver: &'a str,
) -> &'a str {
    match contrast.side(&observation.charge) {
        Some(ProbeSide::Upper) => upper_resolver,
        Some(ProbeSide::Lower) | None => lower_resolver,
    }
}

fn candidate_is_better(left: &ProbeCandidate, right: &ProbeCandidate) -> bool {
    left.training_gain_after_penalty
        .partial_cmp(&right.training_gain_after_penalty)
        .unwrap_or(Ordering::Equal)
        .then_with(|| {
            left.contrast
                .source_preference_disagreement
                .partial_cmp(&right.contrast.source_preference_disagreement)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| right.contrast.left_index.cmp(&left.contrast.left_index))
        .then_with(|| right.contrast.right_index.cmp(&left.contrast.right_index))
        == Ordering::Greater
}

fn preference_disagreement(
    left: &OntologyObservation,
    right: &OntologyObservation,
    resolvers: &[String],
) -> Option<(f64, String, String)> {
    if resolvers.is_empty() {
        return None;
    }
    let left_distribution = normalized_preferences(left, resolvers)?;
    let right_distribution = normalized_preferences(right, resolvers)?;
    let disagreement = left_distribution
        .iter()
        .zip(right_distribution.iter())
        .map(|(left, right)| (left - right).abs())
        .sum::<f64>()
        * 0.5;
    let left_leader = best_resolver_for_observation(left, resolvers);
    let right_leader = best_resolver_for_observation(right, resolvers);
    Some((disagreement.clamp(0.0, 1.0), left_leader, right_leader))
}

fn normalized_preferences(
    observation: &OntologyObservation,
    resolvers: &[String],
) -> Option<Vec<f64>> {
    let values: Vec<f64> = resolvers
        .iter()
        .map(|resolver| resolver_efficiency(observation, resolver))
        .collect();
    let total = values.iter().sum::<f64>();
    if total <= SCORE_EPSILON {
        return None;
    }
    Some(values.into_iter().map(|value| value / total).collect())
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

fn resolver_efficiency(observation: &OntologyObservation, resolver: &str) -> f64 {
    if !observation.charge.magnitude.is_finite() || observation.charge.magnitude <= 0.0 {
        return 0.0;
    }
    let mut total = 0.0;
    let mut attempts = 0usize;
    for outcome in observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
    {
        if outcome.compute_cost == 0 || !outcome.discharged.is_finite() {
            continue;
        }
        let discharge_fraction =
            (f64::from(outcome.discharged) / f64::from(observation.charge.magnitude))
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

fn best_resolver_for_observation(
    observation: &OntologyObservation,
    resolvers: &[String],
) -> String {
    resolvers
        .iter()
        .max_by(|left, right| {
            resolver_efficiency(observation, left)
                .partial_cmp(&resolver_efficiency(observation, right))
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .cloned()
        .unwrap_or_default()
}

fn best_resolver(
    observations: &[OntologyObservation],
    indices: &[usize],
    resolvers: &[String],
) -> String {
    resolvers
        .iter()
        .max_by(|left, right| {
            mean_resolver_efficiency(observations, indices, left)
                .partial_cmp(&mean_resolver_efficiency(observations, indices, right))
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .cloned()
        .unwrap_or_default()
}

fn mean_resolver_efficiency(
    observations: &[OntologyObservation],
    indices: &[usize],
    resolver: &str,
) -> f64 {
    if indices.is_empty() {
        return 0.0;
    }
    indices
        .iter()
        .map(|index| resolver_efficiency(&observations[*index], resolver))
        .sum::<f64>()
        / indices.len() as f64
}

fn mean_efficiency<'a, F>(observations: &'a [OntologyObservation], route: F) -> f64
where
    F: Fn(&'a OntologyObservation) -> &'a str,
{
    if observations.is_empty() {
        return 0.0;
    }
    observations
        .iter()
        .map(|observation| resolver_efficiency(observation, route(observation)))
        .sum::<f64>()
        / observations.len() as f64
}

fn dot(left: &[f32], right: &[f32]) -> Option<f64> {
    if left.len() != right.len() || left.is_empty() {
        return None;
    }
    Some(
        left.iter()
            .zip(right.iter())
            .map(|(left, right)| f64::from(*left) * f64::from(*right))
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope, ResolverOutcome};

    fn observation(
        id: u64,
        residual: [f32; 2],
        left: f32,
        right: f32,
    ) -> OntologyObservation {
        let mut charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            residual.to_vec(),
            1.0,
            ChargeScope::Global,
        );
        charge.id = id;
        OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("left", left, 1))
            .with_outcome(ResolverOutcome::new("right", right, 1))
    }

    #[test]
    fn disagreement_manufactures_an_oblique_projection_axis() {
        let observations = vec![
            observation(1, [-1.0, -1.0], 0.9, 0.1),
            observation(2, [1.0, 1.0], 0.1, 0.9),
        ];
        let pairs = disagreement_pair_schedule(&observations, 0.25);
        assert_eq!(pairs, vec![(0, 1)]);

        let contrast = contrast_axis_from_pair(&observations, 0, 1).unwrap();
        assert!((contrast.axis[0].abs() - contrast.axis[1].abs()).abs() < 1e-6);
        assert_eq!(
            contrast.side(&observations[0].charge),
            Some(ProbeSide::Lower)
        );
        assert_eq!(
            contrast.side(&observations[1].charge),
            Some(ProbeSide::Upper)
        );
    }

    #[test]
    fn same_resolver_preference_cannot_create_a_disagreement_pair() {
        let observations = vec![
            observation(1, [-1.0, 0.0], 0.9, 0.1),
            observation(2, [1.0, 0.0], 0.8, 0.2),
        ];
        assert!(disagreement_pair_schedule(&observations, 0.10).is_empty());
        assert_eq!(valid_contrast_pairs(&observations), vec![(0, 1)]);
    }

    #[test]
    fn learned_probe_changes_future_routing_after_holdout_gate() {
        let train = vec![
            observation(1, [-1.2, -1.1], 0.9, 0.1),
            observation(2, [-1.0, -1.2], 0.8, 0.1),
            observation(3, [-0.8, -0.9], 0.9, 0.2),
            observation(4, [0.8, 0.9], 0.1, 0.9),
            observation(5, [1.0, 1.2], 0.2, 0.8),
            observation(6, [1.2, 1.1], 0.1, 0.9),
        ];
        let holdout = vec![
            observation(7, [-0.9, -1.0], 0.9, 0.1),
            observation(8, [-1.1, -0.8], 0.8, 0.2),
            observation(9, [0.9, 1.0], 0.1, 0.9),
            observation(10, [1.1, 0.8], 0.2, 0.8),
        ];
        let config = ContrastProbeConfig {
            min_preference_disagreement: 0.25,
            min_partition_support: 2,
            min_holdout_support: 1,
            complexity_penalty: 0.0,
            min_holdout_gain: 0.1,
        };
        let fit = fit_disagreement_contrast(&train, &holdout, config).unwrap();
        assert!(fit.applied());
        let probe = fit.probe.unwrap();
        assert_eq!(probe.route(&holdout[0].charge), "left");
        assert_eq!(probe.route(&holdout[3].charge), "right");
        assert!(probe.holdout_gain >= 0.1);
    }
}
