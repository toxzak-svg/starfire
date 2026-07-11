//! Disagreement-induced contrast state for unresolved CHARGE.
//!
//! Starfire's static ontology path can inspect one residual at a time. This
//! module tests a different operation: repeated unresolved states whose
//! independently judged resolver preferences disagree are allowed to interact.
//!
//! The interaction is second-order. Each admitted pair contributes the outer
//! product of its normalized residual displacement to one persistent accumulator:
//!
//! `M <- M + normalize(x_j - x_i) normalize(x_j - x_i)^T`
//!
//! The dominant mode of `M` becomes a new projection axis. The projected mean
//! midpoint of the source pairs becomes its binary discrimination threshold.
//! Side resolvers are then learned from the full training cohort and must survive
//! an independent holdout gate.
//!
//! This is diagnostic-only. It does not promote a concept or modify live routing.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use thiserror::Error;

use super::{Charge, OntologyObservation};

const SCORE_EPSILON: f64 = 1e-12;
const NORM_EPSILON: f64 = 1e-12;
const POWER_ITERATIONS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastProbeConfig {
    /// Minimum total-variation distance between normalized resolver-utility
    /// vectors before two unresolved states may enter the interaction schedule.
    pub min_preference_disagreement: f64,
    /// Maximum pair interactions accumulated into the disagreement mode.
    pub max_pair_interactions: usize,
    /// Minimum observations required on each side of the training projection.
    pub min_partition_support: usize,
    /// Minimum observations required on each side of the independent holdout.
    pub min_holdout_support: usize,
    /// Training-only complexity penalty.
    pub complexity_penalty: f64,
    /// Independent holdout gain required before the mode becomes executable.
    pub min_holdout_gain: f64,
}

impl Default for ContrastProbeConfig {
    fn default() -> Self {
        Self {
            min_preference_disagreement: 0.25,
            max_pair_interactions: 64,
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

/// A new executable residual direction manufactured by repeated disagreement.
///
/// `axis` and `threshold` define:
///
/// `dot(axis, future_residual) <= threshold ? lower : upper`
#[derive(Debug, Clone, PartialEq)]
pub struct TensionContrast {
    pub axis: Vec<f32>,
    pub threshold: f32,
    pub source_pair_count: usize,
    pub mean_source_preference_disagreement: f64,
    /// Share of total displacement second moment captured by the learned mode.
    pub dominant_eigenvalue_fraction: f64,
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

/// Produce the interaction schedule used by the real primitive.
///
/// A pair is admitted only when its normalized resolver-utility distributions
/// disagree by the configured amount and the two observations have different
/// resolver leaders. No charge kind, scope, emitter identity, task class, target
/// text, or developer-authored concept label is inspected.
pub fn disagreement_pair_schedule(
    observations: &[OntologyObservation],
    min_preference_disagreement: f64,
) -> Vec<(usize, usize)> {
    let resolvers = resolver_names(observations);
    let mut pairs = Vec::<(f64, usize, usize)>::new();

    for left in 0..observations.len() {
        for right in (left + 1)..observations.len() {
            if normalized_displacement(
                &observations[left].charge.residual,
                &observations[right].charge.residual,
            )
            .is_none()
            {
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
                pairs.push((disagreement, left, right));
            }
        }
    }

    pairs.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    pairs
        .into_iter()
        .map(|(_, left, right)| (left, right))
        .collect()
}

/// Return every pair that can contribute a finite non-degenerate residual
/// displacement. Exact-budget controls use this schedule vocabulary.
pub fn valid_contrast_pairs(observations: &[OntologyObservation]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for left in 0..observations.len() {
        for right in (left + 1)..observations.len() {
            if normalized_displacement(
                &observations[left].charge.residual,
                &observations[right].charge.residual,
            )
            .is_some()
            {
                pairs.push((left, right));
            }
        }
    }
    pairs
}

/// Fit the real disagreement-conditioned mode.
pub fn fit_disagreement_contrast(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    config: ContrastProbeConfig,
) -> Result<ContrastProbeFit, ContrastProbeError> {
    validate(train, holdout, config)?;
    let mut pairs = disagreement_pair_schedule(train, config.min_preference_disagreement);
    pairs.truncate(config.max_pair_interactions);
    fit_contrast_from_pairs(train, holdout, &pairs, config)
}

/// Accumulate an explicit pair schedule into one persistent disagreement mode.
///
/// This is public so matched-budget controls can use the identical state
/// transition and holdout gate while changing only the source of pair structure.
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

    let Some(contrast) = disagreement_mode_from_pairs(train, pair_schedule) else {
        return Ok(ContrastProbeFit {
            baseline_resolver,
            baseline_training_efficiency,
            baseline_holdout_efficiency,
            best_training_efficiency: baseline_training_efficiency,
            best_training_gain_after_penalty: 0.0,
            proposal_evaluations: pair_schedule.len(),
            probe: None,
        });
    };

    let (lower_indices, upper_indices) = partition_indices(train, &contrast);
    if lower_indices.len() < config.min_partition_support
        || upper_indices.len() < config.min_partition_support
    {
        return Ok(ContrastProbeFit {
            baseline_resolver,
            baseline_training_efficiency,
            baseline_holdout_efficiency,
            best_training_efficiency: baseline_training_efficiency,
            best_training_gain_after_penalty: 0.0,
            proposal_evaluations: pair_schedule.len(),
            probe: None,
        });
    }

    let lower_resolver = best_resolver(train, &lower_indices, &resolvers);
    let upper_resolver = best_resolver(train, &upper_indices, &resolvers);
    if lower_resolver == upper_resolver {
        return Ok(ContrastProbeFit {
            baseline_resolver,
            baseline_training_efficiency,
            baseline_holdout_efficiency,
            best_training_efficiency: baseline_training_efficiency,
            best_training_gain_after_penalty: 0.0,
            proposal_evaluations: pair_schedule.len(),
            probe: None,
        });
    }

    let training_efficiency = mean_efficiency(train, |observation| {
        route_contrast(
            &contrast,
            observation,
            lower_resolver.as_str(),
            upper_resolver.as_str(),
        )
    });
    let training_gain_after_penalty =
        training_efficiency - baseline_training_efficiency - config.complexity_penalty;

    if training_gain_after_penalty <= SCORE_EPSILON {
        return Ok(ContrastProbeFit {
            baseline_resolver,
            baseline_training_efficiency,
            baseline_holdout_efficiency,
            best_training_efficiency: training_efficiency,
            best_training_gain_after_penalty: training_gain_after_penalty,
            proposal_evaluations: pair_schedule.len(),
            probe: None,
        });
    }

    let (lower_holdout, upper_holdout) = partition_indices(holdout, &contrast);
    if lower_holdout.len() < config.min_holdout_support
        || upper_holdout.len() < config.min_holdout_support
    {
        return Ok(ContrastProbeFit {
            baseline_resolver,
            baseline_training_efficiency,
            baseline_holdout_efficiency,
            best_training_efficiency: training_efficiency,
            best_training_gain_after_penalty: training_gain_after_penalty,
            proposal_evaluations: pair_schedule.len(),
            probe: None,
        });
    }

    let holdout_efficiency = mean_efficiency(holdout, |observation| {
        route_contrast(
            &contrast,
            observation,
            lower_resolver.as_str(),
            upper_resolver.as_str(),
        )
    });
    let holdout_gain = holdout_efficiency - baseline_holdout_efficiency;

    let probe = if holdout_gain + SCORE_EPSILON >= config.min_holdout_gain {
        Some(LearnedContrastProbe {
            contrast,
            lower_resolver,
            upper_resolver,
            lower_training_support: lower_indices.len(),
            upper_training_support: upper_indices.len(),
            lower_holdout_support: lower_holdout.len(),
            upper_holdout_support: upper_holdout.len(),
            training_efficiency,
            holdout_efficiency,
            holdout_gain,
        })
    } else {
        None
    };

    Ok(ContrastProbeFit {
        baseline_resolver,
        baseline_training_efficiency,
        baseline_holdout_efficiency,
        best_training_efficiency: training_efficiency,
        best_training_gain_after_penalty: training_gain_after_penalty,
        proposal_evaluations: pair_schedule.len(),
        probe,
    })
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
        || config.max_pair_interactions == 0
        || !config.complexity_penalty.is_finite()
        || config.complexity_penalty < 0.0
        || !config.min_holdout_gain.is_finite()
        || config.min_holdout_gain < 0.0
    {
        return Err(ContrastProbeError::InvalidConfig);
    }
    Ok(())
}

/// Accumulate displacement outer products and extract their dominant mode.
///
/// The outer product removes arbitrary sign from each pair displacement, so
/// repeated geometrically compatible disagreements reinforce the same state
/// without a developer choosing an orientation or residual coordinate.
fn disagreement_mode_from_pairs(
    observations: &[OntologyObservation],
    pair_schedule: &[(usize, usize)],
) -> Option<TensionContrast> {
    let dimension = pair_schedule.iter().find_map(|(left, right)| {
        let left = observations.get(*left)?;
        let right = observations.get(*right)?;
        normalized_displacement(&left.charge.residual, &right.charge.residual)
            .map(|axis| axis.len())
    })?;
    if dimension == 0 {
        return None;
    }

    let mut second_moment = vec![0.0f64; dimension * dimension];
    let mut source_pairs = Vec::<(usize, usize)>::new();

    for &(left_index, right_index) in pair_schedule {
        let left = observations.get(left_index)?;
        let right = observations.get(right_index)?;
        let Some(displacement) =
            normalized_displacement(&left.charge.residual, &right.charge.residual)
        else {
            continue;
        };
        if displacement.len() != dimension {
            continue;
        }

        for row in 0..dimension {
            for column in 0..dimension {
                second_moment[row * dimension + column] +=
                    displacement[row] * displacement[column];
            }
        }
        source_pairs.push((left_index, right_index));
    }

    if source_pairs.len() < 2 {
        return None;
    }

    let trace = (0..dimension)
        .map(|index| second_moment[index * dimension + index])
        .sum::<f64>();
    if trace <= NORM_EPSILON {
        return None;
    }

    let seed_dimension = (0..dimension).max_by(|left, right| {
        second_moment[*left * dimension + *left]
            .partial_cmp(&second_moment[*right * dimension + *right])
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.cmp(left))
    })?;
    let mut axis = vec![0.0f64; dimension];
    axis[seed_dimension] = 1.0;

    for _ in 0..POWER_ITERATIONS {
        let mut next = vec![0.0f64; dimension];
        for row in 0..dimension {
            next[row] = (0..dimension)
                .map(|column| second_moment[row * dimension + column] * axis[column])
                .sum();
        }
        let norm = l2_norm(&next);
        if norm <= NORM_EPSILON {
            return None;
        }
        for value in &mut next {
            *value /= norm;
        }
        axis = next;
    }

    let orientation_dimension = (0..dimension).max_by(|left, right| {
        axis[*left]
            .abs()
            .partial_cmp(&axis[*right].abs())
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.cmp(left))
    })?;
    if axis[orientation_dimension] < 0.0 {
        for value in &mut axis {
            *value = -*value;
        }
    }

    let eigenvalue = axis
        .iter()
        .enumerate()
        .map(|(row, value)| {
            *value
                * (0..dimension)
                    .map(|column| {
                        second_moment[row * dimension + column] * axis[column]
                    })
                    .sum::<f64>()
        })
        .sum::<f64>();
    let dominant_eigenvalue_fraction = (eigenvalue / trace).clamp(0.0, 1.0);

    let axis_f32: Vec<f32> = axis.iter().map(|value| *value as f32).collect();
    let threshold = source_pairs
        .iter()
        .filter_map(|(left, right)| {
            let left_projection =
                dot(&axis_f32, &observations[*left].charge.residual)?;
            let right_projection =
                dot(&axis_f32, &observations[*right].charge.residual)?;
            Some((left_projection + right_projection) * 0.5)
        })
        .sum::<f64>()
        / source_pairs.len() as f64;

    let resolvers = resolver_names(observations);
    let preference_disagreements: Vec<f64> = source_pairs
        .iter()
        .filter_map(|(left, right)| {
            preference_disagreement(
                &observations[*left],
                &observations[*right],
                &resolvers,
            )
            .map(|(distance, _, _)| distance)
        })
        .collect();
    let mean_source_preference_disagreement = if preference_disagreements.is_empty() {
        0.0
    } else {
        preference_disagreements.iter().sum::<f64>()
            / preference_disagreements.len() as f64
    };

    Some(TensionContrast {
        axis: axis_f32,
        threshold: threshold as f32,
        source_pair_count: source_pairs.len(),
        mean_source_preference_disagreement,
        dominant_eigenvalue_fraction,
    })
}

fn normalized_displacement(left: &[f32], right: &[f32]) -> Option<Vec<f64>> {
    if left.is_empty() || left.len() != right.len() {
        return None;
    }
    if left
        .iter()
        .chain(right.iter())
        .any(|value| !value.is_finite())
    {
        return None;
    }

    let mut delta: Vec<f64> = right
        .iter()
        .zip(left.iter())
        .map(|(right, left)| f64::from(*right) - f64::from(*left))
        .collect();
    let norm = l2_norm(&delta);
    if norm <= NORM_EPSILON {
        return None;
    }
    for value in &mut delta {
        *value /= norm;
    }
    Some(delta)
}

fn l2_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
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
    fn repeated_disagreement_accretes_a_dominant_oblique_mode() {
        let observations = vec![
            observation(1, [-1.2, -1.0], 0.9, 0.1),
            observation(2, [-0.8, -1.1], 0.8, 0.1),
            observation(3, [1.0, 0.9], 0.1, 0.9),
            observation(4, [1.1, 1.2], 0.1, 0.8),
        ];
        let pairs = disagreement_pair_schedule(&observations, 0.25);
        let contrast = disagreement_mode_from_pairs(&observations, &pairs).unwrap();

        assert_eq!(contrast.source_pair_count, 4);
        assert!((contrast.axis[0].abs() - contrast.axis[1].abs()).abs() < 0.2);
        assert!(contrast.dominant_eigenvalue_fraction > 0.95);
        assert_eq!(
            contrast.side(&observations[0].charge),
            Some(ProbeSide::Lower)
        );
        assert_eq!(
            contrast.side(&observations[3].charge),
            Some(ProbeSide::Upper)
        );
    }

    #[test]
    fn same_resolver_preference_cannot_enter_the_disagreement_schedule() {
        let observations = vec![
            observation(1, [-1.0, 0.0], 0.9, 0.1),
            observation(2, [1.0, 0.0], 0.8, 0.2),
        ];
        assert!(disagreement_pair_schedule(&observations, 0.10).is_empty());
        assert_eq!(valid_contrast_pairs(&observations), vec![(0, 1)]);
    }

    #[test]
    fn learned_mode_changes_future_routing_after_holdout_gate() {
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
            max_pair_interactions: 16,
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
        assert!(probe.contrast.source_pair_count >= 2);
    }
}
