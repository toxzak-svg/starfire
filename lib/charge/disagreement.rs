//! Disagreement-induced residual coordinates.
//!
//! This module preserves an experimentally gated state between an observation
//! and a promoted concept. Independently judged resolver preferences create
//! pairwise residual axes; validation decides whether an axis may influence a
//! shadow decision. Nothing here mutates the live runtime router.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use thiserror::Error;

use super::induction::OntologyObservation;
use super::types::Charge;

const FEATURE_EPSILON: f64 = 1e-12;
const MAGNITUDE_TOLERANCE: f32 = 1e-5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DisagreementBasisConfig {
    pub tie_epsilon: f64,
    pub min_side_support: usize,
    pub min_validation_support: usize,
    pub min_validation_accuracy: f64,
}

impl Default for DisagreementBasisConfig {
    fn default() -> Self {
        Self {
            tie_epsilon: 1e-12,
            min_side_support: 4,
            min_validation_support: 4,
            min_validation_accuracy: 0.65,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DisagreementBudget {
    pub carrier_slots: usize,
    pub fit_comparisons: usize,
    pub validation_comparisons: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisagreementCarrier {
    pub id: u64,
    pub resolver_a: String,
    pub resolver_b: String,
    pub axis: Vec<f64>,
    pub origin: Vec<f64>,
    pub positive_support: usize,
    pub negative_support: usize,
    pub validation_support: usize,
    pub validation_accuracy: f64,
    pub mean_pair_ceiling: f64,
    pub margin_scale: f64,
    pub eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PairwiseProjection {
    pub carrier_id: u64,
    pub predicted_resolver: String,
    pub margin: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiscriminationProbe {
    pub carrier_id: Option<u64>,
    pub first_resolver: String,
    pub second_resolver: String,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisagreementDecision {
    pub predicted_resolver: String,
    pub probe: Option<DiscriminationProbe>,
    pub projections: Vec<PairwiseProjection>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DisagreementBasisError {
    #[error("training observations are empty")]
    EmptyTrainingSet,
    #[error("validation observations are empty")]
    EmptyValidationSet,
    #[error("at least two measured resolvers are required")]
    TooFewResolvers,
    #[error("invalid disagreement basis configuration: {0}")]
    InvalidConfig(&'static str),
    #[error(
        "{cohort} observation {index} has feature width {actual}; expected {expected}"
    )]
    FeatureWidthMismatch {
        cohort: &'static str,
        index: usize,
        expected: usize,
        actual: usize,
    },
    #[error("{cohort} observation {index} feature {dimension} is non-finite")]
    NonFiniteFeature {
        cohort: &'static str,
        index: usize,
        dimension: usize,
    },
    #[error("{cohort} observation {index} has invalid CHARGE magnitude")]
    InvalidMagnitude {
        cohort: &'static str,
        index: usize,
    },
    #[error("{cohort} observation {index} has an invalid resolver outcome")]
    InvalidOutcome {
        cohort: &'static str,
        index: usize,
    },
    #[error("{cohort} observation {index} is missing resolver {resolver}")]
    MissingResolver {
        cohort: &'static str,
        index: usize,
        resolver: String,
    },
    #[error("decision charge has no residual features")]
    EmptyDecisionFeatures,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisagreementBasis {
    config: DisagreementBasisConfig,
    feature_mean: Vec<f64>,
    feature_scale: Vec<f64>,
    resolvers: Vec<String>,
    resolver_priors: BTreeMap<String, f64>,
    carriers: Vec<DisagreementCarrier>,
    budget: DisagreementBudget,
}

impl DisagreementBasis {
    pub fn fit(
        train: &[OntologyObservation],
        validation: &[OntologyObservation],
        config: DisagreementBasisConfig,
    ) -> Result<Self, DisagreementBasisError> {
        validate_config(config)?;
        if train.is_empty() {
            return Err(DisagreementBasisError::EmptyTrainingSet);
        }
        if validation.is_empty() {
            return Err(DisagreementBasisError::EmptyValidationSet);
        }

        let feature_width = train[0].charge.residual.len();
        if feature_width == 0 {
            return Err(DisagreementBasisError::FeatureWidthMismatch {
                cohort: "training",
                index: 0,
                expected: 1,
                actual: 0,
            });
        }
        validate_basic_observations("training", train, feature_width)?;

        let resolvers = resolver_names(train);
        if resolvers.len() < 2 {
            return Err(DisagreementBasisError::TooFewResolvers);
        }
        validate_resolver_matrix("training", train, &resolvers)?;
        validate_basic_observations("validation", validation, feature_width)?;
        validate_resolver_matrix("validation", validation, &resolvers)?;

        let (feature_mean, feature_scale) = feature_statistics(train, feature_width);
        let standardized_train = standardize_observations(train, &feature_mean, &feature_scale);
        let standardized_validation =
            standardize_observations(validation, &feature_mean, &feature_scale);
        let resolver_priors = resolvers
            .iter()
            .map(|resolver| {
                let mean = train
                    .iter()
                    .map(|observation| resolver_efficiency(observation, resolver).unwrap())
                    .sum::<f64>()
                    / train.len() as f64;
                (resolver.clone(), mean)
            })
            .collect::<BTreeMap<_, _>>();

        let mut carriers = Vec::new();
        let mut fit_comparisons = 0usize;
        let mut validation_comparisons = 0usize;

        for left in 0..resolvers.len() {
            for right in left + 1..resolvers.len() {
                let resolver_a = &resolvers[left];
                let resolver_b = &resolvers[right];
                let mut positive = Vec::<&[f64]>::new();
                let mut negative = Vec::<&[f64]>::new();
                let mut pair_ceiling = 0.0;

                for (observation, features) in train.iter().zip(&standardized_train) {
                    fit_comparisons = fit_comparisons.saturating_add(1);
                    let utility_a = resolver_efficiency(observation, resolver_a).unwrap();
                    let utility_b = resolver_efficiency(observation, resolver_b).unwrap();
                    pair_ceiling += utility_a.max(utility_b);
                    match preference(utility_a, utility_b, config.tie_epsilon) {
                        Preference::A => positive.push(features),
                        Preference::B => negative.push(features),
                        Preference::Tie => {}
                    }
                }

                let positive_support = positive.len();
                let negative_support = negative.len();
                let positive_centroid = mean_vectors(&positive, feature_width);
                let negative_centroid = mean_vectors(&negative, feature_width);
                let mut axis: Vec<f64> = positive_centroid
                    .iter()
                    .zip(&negative_centroid)
                    .map(|(positive, negative)| positive - negative)
                    .collect();
                let axis_norm = vector_norm(&axis);
                let training_support_ok = positive_support >= config.min_side_support
                    && negative_support >= config.min_side_support;
                let non_degenerate = training_support_ok && axis_norm > FEATURE_EPSILON;
                if non_degenerate {
                    for value in &mut axis {
                        *value /= axis_norm;
                    }
                } else {
                    axis.fill(0.0);
                }
                let origin: Vec<f64> = positive_centroid
                    .iter()
                    .zip(&negative_centroid)
                    .map(|(positive, negative)| (positive + negative) * 0.5)
                    .collect();
                let mut margin_scale = if non_degenerate {
                    positive
                        .iter()
                        .chain(negative.iter())
                        .map(|features| dot_offset(features, &origin, &axis).abs())
                        .sum::<f64>()
                        / (positive_support + negative_support).max(1) as f64
                } else {
                    1.0
                };
                if !margin_scale.is_finite() || margin_scale <= FEATURE_EPSILON {
                    margin_scale = 1.0;
                }

                let mut validation_support = 0usize;
                let mut validation_correct = 0usize;
                for (observation, features) in
                    validation.iter().zip(&standardized_validation)
                {
                    validation_comparisons = validation_comparisons.saturating_add(1);
                    let utility_a = resolver_efficiency(observation, resolver_a).unwrap();
                    let utility_b = resolver_efficiency(observation, resolver_b).unwrap();
                    let actual = preference(utility_a, utility_b, config.tie_epsilon);
                    if actual == Preference::Tie || !non_degenerate {
                        continue;
                    }
                    validation_support += 1;
                    let predicted = if dot_offset(features, &origin, &axis) >= 0.0 {
                        Preference::A
                    } else {
                        Preference::B
                    };
                    if predicted == actual {
                        validation_correct += 1;
                    }
                }
                let validation_accuracy = validation_correct as f64
                    / validation_support.max(1) as f64;
                let eligible = non_degenerate
                    && validation_support >= config.min_validation_support
                    && validation_accuracy + FEATURE_EPSILON
                        >= config.min_validation_accuracy;

                carriers.push(DisagreementCarrier {
                    id: carriers.len() as u64 + 1,
                    resolver_a: resolver_a.clone(),
                    resolver_b: resolver_b.clone(),
                    axis,
                    origin,
                    positive_support,
                    negative_support,
                    validation_support,
                    validation_accuracy,
                    mean_pair_ceiling: pair_ceiling / train.len() as f64,
                    margin_scale,
                    eligible,
                });
            }
        }

        let budget = DisagreementBudget {
            carrier_slots: carriers.len(),
            fit_comparisons,
            validation_comparisons,
        };

        Ok(Self {
            config,
            feature_mean,
            feature_scale,
            resolvers,
            resolver_priors,
            carriers,
            budget,
        })
    }

    pub fn config(&self) -> DisagreementBasisConfig {
        self.config
    }

    pub fn feature_width(&self) -> usize {
        self.feature_mean.len()
    }

    pub fn resolvers(&self) -> &[String] {
        &self.resolvers
    }

    pub fn resolver_priors(&self) -> &BTreeMap<String, f64> {
        &self.resolver_priors
    }

    pub fn carriers(&self) -> &[DisagreementCarrier] {
        &self.carriers
    }

    pub fn eligible_carriers(&self) -> impl Iterator<Item = &DisagreementCarrier> {
        self.carriers.iter().filter(|carrier| carrier.eligible)
    }

    pub fn budget(&self) -> DisagreementBudget {
        self.budget
    }

    pub fn validation_pairwise_accuracy(&self) -> f64 {
        let support = self
            .eligible_carriers()
            .map(|carrier| carrier.validation_support)
            .sum::<usize>();
        if support == 0 {
            return 0.0;
        }
        self.eligible_carriers()
            .map(|carrier| carrier.validation_accuracy * carrier.validation_support as f64)
            .sum::<f64>()
            / support as f64
    }

    pub fn pairwise_prediction(
        &self,
        charge: &Charge,
        carrier: &DisagreementCarrier,
    ) -> Result<Option<PairwiseProjection>, DisagreementBasisError> {
        let standardized = self.standardize_charge(charge)?;
        Ok(self.project_standardized(&standardized, carrier))
    }

    pub fn decide(
        &self,
        charge: &Charge,
    ) -> Result<DisagreementDecision, DisagreementBasisError> {
        let standardized = self.standardize_charge(charge)?;
        let mut scores = self.resolver_priors.clone();
        let mut projections = Vec::new();

        for carrier in &self.carriers {
            let Some(projection) = self.project_standardized(&standardized, carrier) else {
                continue;
            };
            let weight = 0.5 + 0.5 * carrier.validation_accuracy.clamp(0.0, 1.0);
            *scores
                .entry(projection.predicted_resolver.clone())
                .or_default() += weight;
            projections.push(projection);
        }

        let mut ranked = self.resolvers.clone();
        ranked.sort_by(|left, right| {
            scores
                .get(right)
                .copied()
                .unwrap_or_default()
                .partial_cmp(&scores.get(left).copied().unwrap_or_default())
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.cmp(right))
        });
        let predicted_resolver = ranked
            .first()
            .cloned()
            .ok_or(DisagreementBasisError::TooFewResolvers)?;

        let probe = ranked.get(1).map(|second| {
            let first = &ranked[0];
            let direct_carrier = self.carriers.iter().find(|carrier| {
                carrier.eligible
                    && ((carrier.resolver_a == *first && carrier.resolver_b == *second)
                        || (carrier.resolver_a == *second && carrier.resolver_b == *first))
            });
            let direct_projection = direct_carrier.and_then(|carrier| {
                projections
                    .iter()
                    .find(|projection| projection.carrier_id == carrier.id)
            });
            DiscriminationProbe {
                carrier_id: direct_carrier.map(|carrier| carrier.id),
                first_resolver: first.clone(),
                second_resolver: second.clone(),
                uncertainty: direct_projection
                    .map(|projection| 1.0 - projection.confidence)
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0),
            }
        });

        Ok(DisagreementDecision {
            predicted_resolver,
            probe,
            projections,
        })
    }

    fn standardize_charge(
        &self,
        charge: &Charge,
    ) -> Result<Vec<f64>, DisagreementBasisError> {
        if charge.residual.is_empty() {
            return Err(DisagreementBasisError::EmptyDecisionFeatures);
        }
        if charge.residual.len() != self.feature_width() {
            return Err(DisagreementBasisError::FeatureWidthMismatch {
                cohort: "decision",
                index: 0,
                expected: self.feature_width(),
                actual: charge.residual.len(),
            });
        }
        let mut standardized = Vec::with_capacity(self.feature_width());
        for (dimension, value) in charge.residual.iter().enumerate() {
            if !value.is_finite() {
                return Err(DisagreementBasisError::NonFiniteFeature {
                    cohort: "decision",
                    index: 0,
                    dimension,
                });
            }
            standardized.push(
                (*value as f64 - self.feature_mean[dimension])
                    / self.feature_scale[dimension],
            );
        }
        Ok(standardized)
    }

    fn project_standardized(
        &self,
        standardized: &[f64],
        carrier: &DisagreementCarrier,
    ) -> Option<PairwiseProjection> {
        if !carrier.eligible {
            return None;
        }
        let margin = dot_offset(standardized, &carrier.origin, &carrier.axis);
        let predicted_resolver = if margin >= 0.0 {
            carrier.resolver_a.clone()
        } else {
            carrier.resolver_b.clone()
        };
        let confidence = (margin.abs() / carrier.margin_scale.max(FEATURE_EPSILON))
            .clamp(0.0, 1.0);
        Some(PairwiseProjection {
            carrier_id: carrier.id,
            predicted_resolver,
            margin,
            confidence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preference {
    A,
    B,
    Tie,
}

fn validate_config(config: DisagreementBasisConfig) -> Result<(), DisagreementBasisError> {
    if !config.tie_epsilon.is_finite() || config.tie_epsilon < 0.0 {
        return Err(DisagreementBasisError::InvalidConfig(
            "tie_epsilon must be finite and non-negative",
        ));
    }
    if config.min_side_support == 0 || config.min_validation_support == 0 {
        return Err(DisagreementBasisError::InvalidConfig(
            "support requirements must be non-zero",
        ));
    }
    if !config.min_validation_accuracy.is_finite()
        || !(0.0..=1.0).contains(&config.min_validation_accuracy)
    {
        return Err(DisagreementBasisError::InvalidConfig(
            "min_validation_accuracy must be in [0, 1]",
        ));
    }
    Ok(())
}

fn validate_basic_observations(
    cohort: &'static str,
    observations: &[OntologyObservation],
    expected_width: usize,
) -> Result<(), DisagreementBasisError> {
    for (index, observation) in observations.iter().enumerate() {
        if observation.charge.residual.len() != expected_width {
            return Err(DisagreementBasisError::FeatureWidthMismatch {
                cohort,
                index,
                expected: expected_width,
                actual: observation.charge.residual.len(),
            });
        }
        for (dimension, value) in observation.charge.residual.iter().enumerate() {
            if !value.is_finite() {
                return Err(DisagreementBasisError::NonFiniteFeature {
                    cohort,
                    index,
                    dimension,
                });
            }
        }
        if !observation.charge.magnitude.is_finite() || observation.charge.magnitude <= 0.0 {
            return Err(DisagreementBasisError::InvalidMagnitude { cohort, index });
        }
        if observation.outcomes.is_empty()
            || observation.outcomes.iter().any(|outcome| {
                outcome.resolver.trim().is_empty()
                    || !outcome.discharged.is_finite()
                    || outcome.discharged < 0.0
                    || outcome.discharged
                        > observation.charge.magnitude + MAGNITUDE_TOLERANCE
                    || outcome.compute_cost == 0
            })
        {
            return Err(DisagreementBasisError::InvalidOutcome { cohort, index });
        }
    }
    Ok(())
}

fn validate_resolver_matrix(
    cohort: &'static str,
    observations: &[OntologyObservation],
    resolvers: &[String],
) -> Result<(), DisagreementBasisError> {
    for (index, observation) in observations.iter().enumerate() {
        for resolver in resolvers {
            if resolver_efficiency(observation, resolver).is_none() {
                return Err(DisagreementBasisError::MissingResolver {
                    cohort,
                    index,
                    resolver: resolver.clone(),
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

fn resolver_efficiency(observation: &OntologyObservation, resolver: &str) -> Option<f64> {
    let matching: Vec<_> = observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
        .collect();
    if matching.is_empty() {
        return None;
    }
    Some(
        matching
            .iter()
            .map(|outcome| {
                (outcome.discharged as f64 / observation.charge.magnitude as f64)
                    / outcome.compute_cost as f64
            })
            .sum::<f64>()
            / matching.len() as f64,
    )
}

fn preference(utility_a: f64, utility_b: f64, tie_epsilon: f64) -> Preference {
    let delta = utility_a - utility_b;
    if delta.abs() <= tie_epsilon {
        Preference::Tie
    } else if delta > 0.0 {
        Preference::A
    } else {
        Preference::B
    }
}

fn feature_statistics(
    observations: &[OntologyObservation],
    width: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut mean = vec![0.0; width];
    for observation in observations {
        for (dimension, value) in observation.charge.residual.iter().enumerate() {
            mean[dimension] += *value as f64;
        }
    }
    for value in &mut mean {
        *value /= observations.len() as f64;
    }

    let mut scale = vec![0.0; width];
    for observation in observations {
        for (dimension, value) in observation.charge.residual.iter().enumerate() {
            let delta = *value as f64 - mean[dimension];
            scale[dimension] += delta * delta;
        }
    }
    for value in &mut scale {
        *value = (*value / observations.len() as f64).sqrt();
        if !value.is_finite() || *value <= FEATURE_EPSILON {
            *value = 1.0;
        }
    }
    (mean, scale)
}

fn standardize_observations(
    observations: &[OntologyObservation],
    mean: &[f64],
    scale: &[f64],
) -> Vec<Vec<f64>> {
    observations
        .iter()
        .map(|observation| {
            observation
                .charge
                .residual
                .iter()
                .enumerate()
                .map(|(dimension, value)| {
                    (*value as f64 - mean[dimension]) / scale[dimension]
                })
                .collect()
        })
        .collect()
}

fn mean_vectors(vectors: &[&[f64]], width: usize) -> Vec<f64> {
    if vectors.is_empty() {
        return vec![0.0; width];
    }
    let mut mean = vec![0.0; width];
    for vector in vectors {
        for (dimension, value) in vector.iter().enumerate() {
            mean[dimension] += *value;
        }
    }
    for value in &mut mean {
        *value /= vectors.len() as f64;
    }
    mean
}

fn vector_norm(vector: &[f64]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn dot_offset(features: &[f64], origin: &[f64], axis: &[f64]) -> f64 {
    features
        .iter()
        .zip(origin)
        .zip(axis)
        .map(|((feature, origin), axis)| (feature - origin) * axis)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charge::{Charge, ChargeKind, ChargeScope, OntologyObservation, ResolverOutcome};

    fn observation(id: u64, x: f32, a: f32, b: f32) -> OntologyObservation {
        let mut charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![x, x * x],
            1.0,
            ChargeScope::Global,
        );
        charge.id = id;
        OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("opaque-a", a, 1))
            .with_outcome(ResolverOutcome::new("opaque-b", b, 1))
    }

    fn separable() -> (Vec<OntologyObservation>, Vec<OntologyObservation>) {
        let train = (0..8)
            .map(|i| {
                if i < 4 {
                    observation(i + 1, -1.0 - i as f32 * 0.05, 0.9, 0.1)
                } else {
                    observation(i + 1, 1.0 + i as f32 * 0.05, 0.1, 0.9)
                }
            })
            .collect();
        let validation = vec![
            observation(101, -1.1, 0.9, 0.1),
            observation(102, -0.9, 0.8, 0.2),
            observation(103, 0.9, 0.2, 0.8),
            observation(104, 1.1, 0.1, 0.9),
        ];
        (train, validation)
    }

    #[test]
    fn disagreement_creates_a_coordinate_not_present_in_the_ontology() {
        let (train, validation) = separable();
        let basis = DisagreementBasis::fit(
            &train,
            &validation,
            DisagreementBasisConfig {
                min_side_support: 2,
                min_validation_support: 4,
                ..DisagreementBasisConfig::default()
            },
        )
        .unwrap();

        assert_eq!(basis.carriers().len(), 1);
        assert!(basis.carriers()[0].eligible);
        assert_eq!(
            basis.decide(&validation[0].charge).unwrap().predicted_resolver,
            "opaque-a"
        );
        assert_eq!(
            basis.decide(&validation[3].charge).unwrap().predicted_resolver,
            "opaque-b"
        );
    }

    #[test]
    fn one_sided_preference_cannot_fabricate_a_carrier() {
        let train = (0..8)
            .map(|i| observation(i + 1, i as f32, 0.9, 0.1))
            .collect::<Vec<_>>();
        let validation = train.clone();
        let basis = DisagreementBasis::fit(&train, &validation, Default::default()).unwrap();

        assert_eq!(basis.carriers().len(), 1);
        assert!(!basis.carriers()[0].eligible);
    }

    #[test]
    fn replay_is_bit_deterministic() {
        let (train, validation) = separable();
        let config = DisagreementBasisConfig {
            min_side_support: 2,
            min_validation_support: 4,
            ..Default::default()
        };
        let first = DisagreementBasis::fit(&train, &validation, config).unwrap();
        let second = DisagreementBasis::fit(&train, &validation, config).unwrap();

        assert_eq!(first, second);
        for observation in &validation {
            assert_eq!(
                first.decide(&observation.charge).unwrap(),
                second.decide(&observation.charge).unwrap()
            );
        }
    }

    #[test]
    fn a_decision_emits_an_executable_two_resolver_question() {
        let (train, validation) = separable();
        let basis = DisagreementBasis::fit(
            &train,
            &validation,
            DisagreementBasisConfig {
                min_side_support: 2,
                min_validation_support: 4,
                ..Default::default()
            },
        )
        .unwrap();

        let probe = basis.decide(&validation[0].charge).unwrap().probe.unwrap();
        assert_ne!(probe.first_resolver, probe.second_resolver);
        assert!((0.0..=1.0).contains(&probe.uncertainty));
    }

    #[test]
    fn feature_width_mismatch_is_rejected() {
        let (train, mut validation) = separable();
        validation[0].charge.residual.push(3.0);

        assert!(matches!(
            DisagreementBasis::fit(&train, &validation, Default::default()),
            Err(DisagreementBasisError::FeatureWidthMismatch { .. })
        ));
    }
}
