use std::cmp::Ordering;
use std::collections::BTreeSet;

use thiserror::Error;

use super::ontology::{
    ConceptEvidence, ConceptId, ConceptPredicate, ConceptUtility, Direction, InducedConcept,
    OntologyInducer, PromotionCriteria,
};
use super::types::{Charge, Resolution};

const SCORE_EPSILON: f64 = 1e-12;
const MAGNITUDE_TOLERANCE: f32 = 1e-5;

/// One measured resolver outcome for one CHARGE observation.
///
/// The inducer never calls resolvers itself. Callers execute candidate resolvers,
/// preserve the measured outcomes, and feed those observations into `fit`.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolverOutcome {
    pub resolver: String,
    pub discharged: f32,
    pub compute_cost: u64,
}

impl ResolverOutcome {
    pub fn new(resolver: impl Into<String>, discharged: f32, compute_cost: u64) -> Self {
        Self {
            resolver: resolver.into(),
            discharged,
            compute_cost,
        }
    }

    pub fn from_resolution(resolver: impl Into<String>, resolution: &Resolution) -> Self {
        Self::new(resolver, resolution.discharged, resolution.compute_cost)
    }
}

/// A replayable CHARGE state plus empirically measured candidate-resolver outcomes.
///
/// Partial outcome matrices are allowed. A resolver with no measurement for an
/// observation receives zero utility for that observation rather than benefiting
/// from sparse coverage.
#[derive(Debug, Clone)]
pub struct OntologyObservation {
    pub charge: Charge,
    pub outcomes: Vec<ResolverOutcome>,
}

impl OntologyObservation {
    pub fn new(charge: Charge) -> Self {
        Self {
            charge,
            outcomes: Vec::new(),
        }
    }

    pub fn with_outcome(mut self, outcome: ResolverOutcome) -> Self {
        self.outcomes.push(outcome);
        self
    }

    pub fn with_resolution(
        self,
        resolver: impl Into<String>,
        resolution: &Resolution,
    ) -> Self {
        self.with_outcome(ResolverOutcome::from_resolution(resolver, resolution))
    }

    pub fn record_outcome(&mut self, outcome: ResolverOutcome) {
        self.outcomes.push(outcome);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpiricalInductionConfig {
    /// Maximum number of promoted root-level distinctions.
    pub max_concepts: usize,
    /// Minimum effective support on both sides of a training split.
    pub min_partition_support: usize,
    /// Minimum effective support on both sides of the independent holdout split.
    pub min_holdout_support: usize,
    /// Bound on residual midpoint thresholds examined per dimension and generation.
    pub max_thresholds_per_dimension: usize,
    /// Training-only penalty subtracted from candidate marginal utility.
    pub complexity_penalty: f64,
    /// Independent empirical promotion contract.
    pub promotion: PromotionCriteria,
}

impl Default for EmpiricalInductionConfig {
    fn default() -> Self {
        Self {
            max_concepts: 8,
            min_partition_support: 16,
            min_holdout_support: 8,
            max_thresholds_per_dimension: 64,
            complexity_penalty: 0.002,
            promotion: PromotionCriteria::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum OntologyInductionError {
    #[error("training observations are empty")]
    EmptyTrainingSet,
    #[error("holdout observations are empty")]
    EmptyHoldoutSet,
    #[error("no resolver outcomes exist in the training observations")]
    NoTrainingResolvers,
    #[error("{cohort} observation {index} has non-finite or non-positive CHARGE magnitude")]
    InvalidChargeMagnitude {
        cohort: &'static str,
        index: usize,
    },
    #[error("{cohort} observation {index} has no resolver outcomes")]
    MissingResolverOutcomes {
        cohort: &'static str,
        index: usize,
    },
    #[error("{cohort} observation {index} outcome {outcome} has an empty resolver name")]
    EmptyResolverName {
        cohort: &'static str,
        index: usize,
        outcome: usize,
    },
    #[error("{cohort} observation {index} outcome {outcome} has invalid discharge")]
    InvalidDischarge {
        cohort: &'static str,
        index: usize,
        outcome: usize,
    },
    #[error("{cohort} observation {index} outcome {outcome} declares zero compute cost")]
    ZeroComputeCost {
        cohort: &'static str,
        index: usize,
        outcome: usize,
    },
    #[error("max_thresholds_per_dimension must be greater than zero")]
    ZeroThresholdBudget,
    #[error("complexity_penalty must be finite and non-negative")]
    InvalidComplexityPenalty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConceptRoute {
    pub concept: InducedConcept,
    pub resolver: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyRouteDecision {
    pub concept: Option<ConceptId>,
    pub resolver: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OntologyPolicyMetrics {
    pub mean_discharge_efficiency: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OntologyInductionSummary {
    pub training_observations: usize,
    pub holdout_observations: usize,
    pub candidates_considered: usize,
    pub promoted_concepts: usize,
    pub baseline_training: OntologyPolicyMetrics,
    pub baseline_holdout: OntologyPolicyMetrics,
    pub induced_training: OntologyPolicyMetrics,
    pub induced_holdout: OntologyPolicyMetrics,
}

/// An executable ontology learned from empirical CHARGE-resolution history.
///
/// Routes are checked in promotion order. A charge that matches no promoted
/// concept falls back to the empirically strongest resolver for the remaining
/// parent cohort.
#[derive(Debug, Clone)]
pub struct LearnedOntology {
    routes: Vec<ConceptRoute>,
    parent_resolver: String,
    summary: OntologyInductionSummary,
}

impl LearnedOntology {
    pub fn route(&self, charge: &Charge) -> OntologyRouteDecision {
        for route in &self.routes {
            if route.concept.predicate.matches(charge) {
                return OntologyRouteDecision {
                    concept: Some(route.concept.id),
                    resolver: route.resolver.clone(),
                };
            }
        }

        OntologyRouteDecision {
            concept: None,
            resolver: self.parent_resolver.clone(),
        }
    }

    pub fn routes(&self) -> &[ConceptRoute] {
        &self.routes
    }

    pub fn parent_resolver(&self) -> &str {
        &self.parent_resolver
    }

    pub fn summary(&self) -> &OntologyInductionSummary {
        &self.summary
    }
}

/// Deterministic greedy ontology induction over CHARGE-native predicates.
///
/// Candidate generation and ranking use the training cohort only. Exactly the
/// strongest training candidate is evaluated against holdout at each generation;
/// the implementation does not scan holdout to choose among competing concepts.
/// A holdout rejection stops growth for that fit.
#[derive(Debug, Clone)]
pub struct EmpiricalOntologyInducer {
    config: EmpiricalInductionConfig,
}

impl EmpiricalOntologyInducer {
    pub fn new(config: EmpiricalInductionConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> EmpiricalInductionConfig {
        self.config
    }

    pub fn fit(
        &self,
        train: &[OntologyObservation],
        holdout: &[OntologyObservation],
    ) -> Result<LearnedOntology, OntologyInductionError> {
        validate_config(self.config)?;
        validate_observations("training", train, true)?;
        validate_observations("holdout", holdout, false)?;

        let resolvers = resolver_names(train);
        if resolvers.is_empty() {
            return Err(OntologyInductionError::NoTrainingResolvers);
        }

        let baseline_policy = build_policy(train, &[], &resolvers);
        let baseline_training = evaluate_policy(train, &baseline_policy);
        let baseline_holdout = evaluate_policy(holdout, &baseline_policy);

        let mut registry = OntologyInducer::new(self.config.promotion);
        let mut active_predicates = Vec::<ConceptPredicate>::new();
        let mut active_routes = Vec::<ConceptRoute>::new();
        let mut candidates_considered = 0usize;

        while active_predicates.len() < self.config.max_concepts {
            let current_policy = build_policy(train, &active_predicates, &resolvers);
            let current_training = evaluate_policy(train, &current_policy);
            let current_holdout = evaluate_policy(holdout, &current_policy);

            let mut candidates = generate_candidates(
                train,
                &active_predicates,
                self.config.min_partition_support,
                self.config.max_thresholds_per_dimension,
            );
            candidates_considered = candidates_considered.saturating_add(candidates.len());

            for candidate in &mut candidates {
                let mut trial_predicates = active_predicates.clone();
                trial_predicates.push(candidate.predicate.clone());
                let trial_policy = build_policy(train, &trial_predicates, &resolvers);
                let trial_training = evaluate_policy(train, &trial_policy);
                candidate.training_gain = trial_training.mean_discharge_efficiency
                    - current_training.mean_discharge_efficiency
                    - self.config.complexity_penalty;
            }

            candidates.sort_by(candidate_order);
            let Some(candidate) = candidates.into_iter().next() else {
                break;
            };
            if candidate.training_gain <= SCORE_EPSILON {
                break;
            }

            let holdout_membership = effective_membership_indices(
                holdout,
                &active_predicates,
                &candidate.predicate,
            );
            let holdout_parent = parent_indices(holdout, &active_predicates);
            let holdout_support = holdout_membership.len();
            let holdout_complement = holdout_parent.len().saturating_sub(holdout_support);
            if holdout_support < self.config.min_holdout_support
                || holdout_complement < self.config.min_holdout_support
            {
                break;
            }

            let mut trial_predicates = active_predicates.clone();
            trial_predicates.push(candidate.predicate.clone());
            let trial_policy = build_policy(train, &trial_predicates, &resolvers);
            let trial_holdout = evaluate_policy(holdout, &trial_policy);
            let holdout_gain = trial_holdout.mean_discharge_efficiency
                - current_holdout.mean_discharge_efficiency;

            let holdout_membership_set: BTreeSet<usize> =
                holdout_membership.iter().copied().collect();
            let holdout_negative: Vec<usize> = holdout_parent
                .iter()
                .copied()
                .filter(|index| !holdout_membership_set.contains(index))
                .collect();

            let concept = registry.propose(
                None,
                candidate.predicate.clone(),
                ConceptEvidence {
                    observations: holdout_parent.len() as u64,
                    positive_instances: observation_ids(holdout, &holdout_membership),
                    negative_instances: observation_ids(holdout, &holdout_negative),
                    holdout_gain,
                },
                ConceptUtility {
                    routing_gain: holdout_gain,
                    ..ConceptUtility::default()
                },
            );

            if !registry.promote(concept.clone()) {
                break;
            }

            let resolver = trial_policy
                .routes
                .last()
                .map(|route| route.resolver.clone())
                .unwrap_or_else(|| current_policy.parent_resolver.clone());
            active_predicates.push(candidate.predicate);
            active_routes.push(ConceptRoute { concept, resolver });
            registry.advance_generation();
        }

        let final_policy = build_policy(train, &active_predicates, &resolvers);
        let induced_training = evaluate_policy(train, &final_policy);
        let induced_holdout = evaluate_policy(holdout, &final_policy);

        Ok(LearnedOntology {
            routes: active_routes,
            parent_resolver: final_policy.parent_resolver,
            summary: OntologyInductionSummary {
                training_observations: train.len(),
                holdout_observations: holdout.len(),
                candidates_considered,
                promoted_concepts: active_predicates.len(),
                baseline_training,
                baseline_holdout,
                induced_training,
                induced_holdout,
            },
        })
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    predicate: ConceptPredicate,
    support: usize,
    training_gain: f64,
}

#[derive(Debug, Clone)]
struct PolicyRoute {
    predicate: ConceptPredicate,
    resolver: String,
}

#[derive(Debug, Clone)]
struct LearnedPolicy {
    routes: Vec<PolicyRoute>,
    parent_resolver: String,
}

fn validate_config(config: EmpiricalInductionConfig) -> Result<(), OntologyInductionError> {
    if config.max_thresholds_per_dimension == 0 {
        return Err(OntologyInductionError::ZeroThresholdBudget);
    }
    if !config.complexity_penalty.is_finite() || config.complexity_penalty < 0.0 {
        return Err(OntologyInductionError::InvalidComplexityPenalty);
    }
    Ok(())
}

fn validate_observations(
    cohort: &'static str,
    observations: &[OntologyObservation],
    training: bool,
) -> Result<(), OntologyInductionError> {
    if observations.is_empty() {
        return if training {
            Err(OntologyInductionError::EmptyTrainingSet)
        } else {
            Err(OntologyInductionError::EmptyHoldoutSet)
        };
    }

    for (index, observation) in observations.iter().enumerate() {
        if !observation.charge.magnitude.is_finite() || observation.charge.magnitude <= 0.0 {
            return Err(OntologyInductionError::InvalidChargeMagnitude { cohort, index });
        }
        if observation.outcomes.is_empty() {
            return Err(OntologyInductionError::MissingResolverOutcomes { cohort, index });
        }
        for (outcome, measured) in observation.outcomes.iter().enumerate() {
            if measured.resolver.trim().is_empty() {
                return Err(OntologyInductionError::EmptyResolverName {
                    cohort,
                    index,
                    outcome,
                });
            }
            if !measured.discharged.is_finite()
                || measured.discharged < 0.0
                || measured.discharged > observation.charge.magnitude + MAGNITUDE_TOLERANCE
            {
                return Err(OntologyInductionError::InvalidDischarge {
                    cohort,
                    index,
                    outcome,
                });
            }
            if measured.compute_cost == 0 {
                return Err(OntologyInductionError::ZeroComputeCost {
                    cohort,
                    index,
                    outcome,
                });
            }
        }
    }

    Ok(())
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

fn best_resolver(
    observations: &[OntologyObservation],
    indices: &[usize],
    resolvers: &[String],
) -> String {
    resolvers
        .iter()
        .max_by(|left, right| {
            mean_resolver_score(observations, indices, left)
                .partial_cmp(&mean_resolver_score(observations, indices, right))
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .cloned()
        .unwrap_or_default()
}

fn mean_resolver_score(
    observations: &[OntologyObservation],
    indices: &[usize],
    resolver: &str,
) -> f64 {
    if indices.is_empty() {
        return 0.0;
    }
    indices
        .iter()
        .map(|index| resolver_score(&observations[*index], resolver))
        .sum::<f64>()
        / indices.len() as f64
}

fn build_policy(
    train: &[OntologyObservation],
    predicates: &[ConceptPredicate],
    resolvers: &[String],
) -> LearnedPolicy {
    let mut routes = Vec::with_capacity(predicates.len());
    for (position, predicate) in predicates.iter().enumerate() {
        let indices = effective_membership_indices(train, &predicates[..position], predicate);
        routes.push(PolicyRoute {
            predicate: predicate.clone(),
            resolver: best_resolver(train, &indices, resolvers),
        });
    }

    let parent = parent_indices(train, predicates);
    let parent_resolver = if parent.is_empty() {
        let all: Vec<usize> = (0..train.len()).collect();
        best_resolver(train, &all, resolvers)
    } else {
        best_resolver(train, &parent, resolvers)
    };

    LearnedPolicy {
        routes,
        parent_resolver,
    }
}

fn policy_resolver<'a>(policy: &'a LearnedPolicy, charge: &Charge) -> &'a str {
    policy
        .routes
        .iter()
        .find(|route| route.predicate.matches(charge))
        .map(|route| route.resolver.as_str())
        .unwrap_or(policy.parent_resolver.as_str())
}

fn evaluate_policy(
    observations: &[OntologyObservation],
    policy: &LearnedPolicy,
) -> OntologyPolicyMetrics {
    let total = observations
        .iter()
        .map(|observation| {
            resolver_score(
                observation,
                policy_resolver(policy, &observation.charge),
            )
        })
        .sum::<f64>();

    OntologyPolicyMetrics {
        mean_discharge_efficiency: total / observations.len().max(1) as f64,
    }
}

fn generate_candidates(
    observations: &[OntologyObservation],
    active: &[ConceptPredicate],
    min_support: usize,
    max_thresholds_per_dimension: usize,
) -> Vec<Candidate> {
    let parent = parent_indices(observations, active);
    if parent.len() < min_support.saturating_mul(2) {
        return Vec::new();
    }

    let dimensions = parent
        .iter()
        .map(|index| observations[*index].charge.residual.len())
        .max()
        .unwrap_or(0);
    let mut predicates = Vec::new();

    for dimension in 0..dimensions {
        let mut values: Vec<f32> = parent
            .iter()
            .filter_map(|index| observations[*index].charge.residual.get(dimension).copied())
            .filter(|value| value.is_finite())
            .collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        values.dedup_by(|left, right| (*left - *right).abs() < f32::EPSILON);
        for threshold in bounded_midpoints(&values, max_thresholds_per_dimension) {
            predicates.push(ConceptPredicate::ResidualThreshold {
                dimension,
                threshold,
                direction: Direction::AtLeast,
            });
            predicates.push(ConceptPredicate::ResidualThreshold {
                dimension,
                threshold,
                direction: Direction::AtMost,
            });
        }
    }

    let mut persistence: Vec<u32> = parent
        .iter()
        .map(|index| observations[*index].charge.persistence)
        .collect();
    persistence.sort_unstable();
    persistence.dedup();
    for threshold in persistence.into_iter().skip(1) {
        predicates.push(ConceptPredicate::PersistenceRange {
            min: threshold,
            max: None,
        });
        predicates.push(ConceptPredicate::PersistenceRange {
            min: 0,
            max: Some(threshold.saturating_sub(1)),
        });
    }

    let trace_resolvers: BTreeSet<String> = parent
        .iter()
        .flat_map(|index| observations[*index].charge.trace.resolvers.iter().cloned())
        .collect();
    for resolver in trace_resolvers {
        predicates.push(ConceptPredicate::TraceContains { resolver });
    }

    let mut candidates = Vec::new();
    let mut seen_memberships = Vec::<Vec<usize>>::new();
    for predicate in predicates {
        let membership = effective_membership_indices(observations, active, &predicate);
        let support = membership.len();
        let complement = parent.len().saturating_sub(support);
        if support < min_support || complement < min_support {
            continue;
        }
        if seen_memberships.iter().any(|seen| seen == &membership) {
            continue;
        }
        seen_memberships.push(membership.clone());
        candidates.push(Candidate {
            predicate,
            support,
            training_gain: f64::NEG_INFINITY,
        });
    }

    candidates
}

fn bounded_midpoints(values: &[f32], limit: usize) -> Vec<f32> {
    if values.len() < 2 || limit == 0 {
        return Vec::new();
    }
    let total = values.len() - 1;
    let take = total.min(limit);
    if take == total {
        return values
            .windows(2)
            .map(|pair| pair[0] + (pair[1] - pair[0]) * 0.5)
            .collect();
    }

    let mut midpoints = Vec::with_capacity(take);
    let mut seen = BTreeSet::new();
    for slot in 0..take {
        let index = ((slot * total) + (take / 2)) / take;
        let index = index.min(total - 1);
        if seen.insert(index) {
            midpoints.push(values[index] + (values[index + 1] - values[index]) * 0.5);
        }
    }
    midpoints
}

fn candidate_order(left: &Candidate, right: &Candidate) -> Ordering {
    right
        .training_gain
        .partial_cmp(&left.training_gain)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.support.cmp(&right.support))
        .then_with(|| predicate_key(&left.predicate).cmp(&predicate_key(&right.predicate)))
}

fn predicate_key(predicate: &ConceptPredicate) -> String {
    format!("{predicate:?}")
}

fn parent_indices(
    observations: &[OntologyObservation],
    active: &[ConceptPredicate],
) -> Vec<usize> {
    observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| {
            !active
                .iter()
                .any(|predicate| predicate.matches(&observation.charge))
        })
        .map(|(index, _)| index)
        .collect()
}

fn effective_membership_indices(
    observations: &[OntologyObservation],
    active: &[ConceptPredicate],
    candidate: &ConceptPredicate,
) -> Vec<usize> {
    observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| {
            !active
                .iter()
                .any(|predicate| predicate.matches(&observation.charge))
                && candidate.matches(&observation.charge)
        })
        .map(|(index, _)| index)
        .collect()
}

fn observation_ids(observations: &[OntologyObservation], indices: &[usize]) -> Vec<u64> {
    indices
        .iter()
        .map(|index| {
            let id = observations[*index].charge.id;
            if id == 0 {
                (*index as u64).saturating_add(1)
            } else {
                id
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{ChargeKind, ChargeScope};

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

    fn separable_dataset(offset: u64, per_class: usize) -> Vec<OntologyObservation> {
        let mut observations = Vec::new();
        let mut id = offset;
        for index in 0..per_class {
            let jitter = index as f32 * 0.001;
            observations.push(observation(
                id,
                vec![0.90 - jitter, 0.10 + jitter, 0.15],
                [0.92, 0.10, 0.08],
            ));
            id += 1;
        }
        for index in 0..per_class {
            let jitter = index as f32 * 0.001;
            observations.push(observation(
                id,
                vec![0.15, 0.90 - jitter, 0.10 + jitter],
                [0.09, 0.93, 0.09],
            ));
            id += 1;
        }
        for index in 0..per_class {
            let jitter = index as f32 * 0.001;
            observations.push(observation(
                id,
                vec![0.10 + jitter, 0.15, 0.90 - jitter],
                [0.08, 0.10, 0.94],
            ));
            id += 1;
        }
        observations
    }

    #[test]
    fn empirical_inducer_builds_executable_routes_from_unresolved_history() {
        let train = separable_dataset(1, 24);
        let holdout = separable_dataset(10_000, 24);
        let inducer = EmpiricalOntologyInducer::new(EmpiricalInductionConfig {
            max_concepts: 2,
            min_partition_support: 12,
            min_holdout_support: 12,
            max_thresholds_per_dimension: 64,
            complexity_penalty: 0.001,
            promotion: PromotionCriteria {
                min_observations: 12,
                min_holdout_gain: 0.05,
                min_total_utility_gain: 0.05,
            },
        });

        let ontology = inducer.fit(&train, &holdout).unwrap();

        assert_eq!(ontology.routes().len(), 2);
        assert!(
            ontology.summary().induced_holdout.mean_discharge_efficiency
                > ontology.summary().baseline_holdout.mean_discharge_efficiency * 2.0
        );
        assert_eq!(ontology.route(&holdout[0].charge).resolver, "memory");
        assert_eq!(ontology.route(&holdout[24].charge).resolver, "reasoning");
        assert_eq!(ontology.route(&holdout[48].charge).resolver, "causal");
    }

    #[test]
    fn holdout_rejection_stops_growth_at_training_winner() {
        let train = separable_dataset(1, 20);
        let mut holdout = separable_dataset(10_000, 20);
        let config = EmpiricalInductionConfig {
            max_concepts: 2,
            min_partition_support: 10,
            min_holdout_support: 10,
            max_thresholds_per_dimension: 32,
            complexity_penalty: 0.001,
            promotion: PromotionCriteria {
                min_observations: 10,
                min_holdout_gain: 0.02,
                min_total_utility_gain: 0.02,
            },
        };

        let resolvers = resolver_names(&train);
        let current_policy = build_policy(&train, &[], &resolvers);
        let current_training = evaluate_policy(&train, &current_policy);
        let mut candidates = generate_candidates(
            &train,
            &[],
            config.min_partition_support,
            config.max_thresholds_per_dimension,
        );
        for candidate in &mut candidates {
            let trial_policy = build_policy(
                &train,
                &[candidate.predicate.clone()],
                &resolvers,
            );
            let trial_training = evaluate_policy(&train, &trial_policy);
            candidate.training_gain = trial_training.mean_discharge_efficiency
                - current_training.mean_discharge_efficiency
                - config.complexity_penalty;
        }
        candidates.sort_by(candidate_order);
        let winner = candidates.first().expect("training must produce a candidate");
        let winner_policy = build_policy(
            &train,
            &[winner.predicate.clone()],
            &resolvers,
        );

        for observation in &mut holdout {
            let baseline = policy_resolver(&current_policy, &observation.charge).to_string();
            let winner_route = policy_resolver(&winner_policy, &observation.charge).to_string();
            for outcome in &mut observation.outcomes {
                outcome.discharged = if outcome.resolver == baseline { 0.9 } else { 0.0 };
            }
            if winner_route != baseline {
                let winner_outcome = observation
                    .outcomes
                    .iter_mut()
                    .find(|outcome| outcome.resolver == winner_route)
                    .unwrap();
                winner_outcome.discharged = 0.0;
            }
        }

        let ontology = EmpiricalOntologyInducer::new(config)
            .fit(&train, &holdout)
            .unwrap();

        assert!(ontology.routes().is_empty());
        assert_eq!(ontology.summary().promoted_concepts, 0);
    }

    #[test]
    fn sparse_resolver_coverage_is_penalized_instead_of_ignored() {
        let mut train = Vec::new();
        let mut holdout = Vec::new();
        for id in 1..=24 {
            let charge = Charge::new(
                ChargeKind::Custom("unresolved".into()),
                vec![id as f32 / 24.0],
                1.0,
                ChargeScope::Global,
            );
            let mut observation = OntologyObservation::new(charge)
                .with_outcome(ResolverOutcome::new("reliable", 0.6, 1));
            if id == 1 {
                observation.record_outcome(ResolverOutcome::new("sparse", 1.0, 1));
            }
            train.push(observation.clone());
            holdout.push(observation);
        }

        let ontology = EmpiricalOntologyInducer::new(EmpiricalInductionConfig {
            max_concepts: 0,
            ..EmpiricalInductionConfig::default()
        })
        .fit(&train, &holdout)
        .unwrap();

        assert_eq!(ontology.parent_resolver(), "reliable");
    }

    #[test]
    fn invalid_empirical_outcomes_are_rejected() {
        let charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![0.5],
            1.0,
            ChargeScope::Global,
        );
        let invalid = vec![OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("memory", 1.5, 1))];

        let error = EmpiricalOntologyInducer::new(EmpiricalInductionConfig::default())
            .fit(&invalid, &invalid)
            .unwrap_err();

        assert!(matches!(error, OntologyInductionError::InvalidDischarge { .. }));
    }

    #[test]
    fn observation_can_record_existing_resolution_objects() {
        let charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![0.5],
            1.0,
            ChargeScope::Global,
        );
        let resolution = Resolution {
            discharged: 0.4,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: 3,
        };
        let observation = OntologyObservation::new(charge)
            .with_resolution("reasoning", &resolution);

        assert_eq!(observation.outcomes.len(), 1);
        assert_eq!(observation.outcomes[0].resolver, "reasoning");
        assert_eq!(observation.outcomes[0].discharged, 0.4);
        assert_eq!(observation.outcomes[0].compute_cost, 3);
    }
}
