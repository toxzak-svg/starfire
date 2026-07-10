//! Diagnostic metrics for CHARGE experiment reports.

use serde::Serialize;

use super::induction::OntologyObservation;

const TIE_EPSILON: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct IdentifiabilityCriteria {
    pub min_reasoning_leader_fraction: f64,
    pub min_causal_leader_fraction: f64,
    pub margin_threshold: f64,
    pub min_positive_margin_fraction: f64,
    pub min_negative_margin_fraction: f64,
    pub min_directional_windows: usize,
    pub max_single_future_resolver_fraction: f64,
}

impl IdentifiabilityCriteria {
    pub fn h5_default() -> Self {
        Self {
            min_reasoning_leader_fraction: 0.20,
            min_causal_leader_fraction: 0.20,
            margin_threshold: 0.10,
            min_positive_margin_fraction: 0.15,
            min_negative_margin_fraction: 0.15,
            min_directional_windows: 3,
            max_single_future_resolver_fraction: 0.70,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ResolverLeaderDistribution {
    pub observations: usize,
    pub reasoning_fraction: f64,
    pub causal_fraction: f64,
    pub prediction_fraction: f64,
    pub metacognition_fraction: f64,
    pub tie_fraction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ResolverMarginSummary {
    pub observations: usize,
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
    pub q10: f64,
    pub q25: f64,
    pub q50: f64,
    pub q75: f64,
    pub q90: f64,
    pub fraction_at_least_positive: f64,
    pub fraction_at_most_negative: f64,
    pub ambiguous_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResolverWindowIdentifiability {
    pub index: usize,
    pub leaders: ResolverLeaderDistribution,
    pub margin: ResolverMarginSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct IdentifiabilityGates {
    pub reasoning_leader_floor: bool,
    pub causal_leader_floor: bool,
    pub positive_margin_floor: bool,
    pub negative_margin_floor: bool,
    pub stable_future_directionality: bool,
    pub no_single_future_resolver_dominates: bool,
}

impl IdentifiabilityGates {
    fn passed(self) -> bool {
        self.reasoning_leader_floor
            && self.causal_leader_floor
            && self.positive_margin_floor
            && self.negative_margin_floor
            && self.stable_future_directionality
            && self.no_single_future_resolver_dominates
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdentifiabilitySummary {
    pub leaders: ResolverLeaderDistribution,
    pub margin: ResolverMarginSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdentifiabilityAssessment {
    pub criteria: IdentifiabilityCriteria,
    pub overall: IdentifiabilitySummary,
    pub windows: Vec<ResolverWindowIdentifiability>,
    pub directional_windows: usize,
    pub gates: IdentifiabilityGates,
    pub passed: bool,
}

pub fn assess_resolver_identifiability(
    windows: &[Vec<OntologyObservation>],
    criteria: IdentifiabilityCriteria,
) -> IdentifiabilityAssessment {
    let flattened: Vec<OntologyObservation> = windows
        .iter()
        .flat_map(|window| window.iter().cloned())
        .collect();
    let overall = summarize_identifiability(&flattened, criteria.margin_threshold);
    let window_summaries: Vec<ResolverWindowIdentifiability> = windows
        .iter()
        .enumerate()
        .map(|(index, window)| ResolverWindowIdentifiability {
            index,
            leaders: leader_distribution(window),
            margin: margin_summary(window, criteria.margin_threshold),
        })
        .collect();
    let directional_windows = window_summaries
        .iter()
        .filter(|window| {
            window.margin.fraction_at_least_positive >= criteria.min_positive_margin_fraction
                && window.margin.fraction_at_most_negative >= criteria.min_negative_margin_fraction
        })
        .count();
    let max_leader_fraction = [
        overall.leaders.reasoning_fraction,
        overall.leaders.causal_fraction,
        overall.leaders.prediction_fraction,
        overall.leaders.metacognition_fraction,
    ]
    .into_iter()
    .fold(0.0, f64::max);
    let gates = IdentifiabilityGates {
        reasoning_leader_floor: overall.leaders.reasoning_fraction
            >= criteria.min_reasoning_leader_fraction,
        causal_leader_floor: overall.leaders.causal_fraction >= criteria.min_causal_leader_fraction,
        positive_margin_floor: overall.margin.fraction_at_least_positive
            >= criteria.min_positive_margin_fraction,
        negative_margin_floor: overall.margin.fraction_at_most_negative
            >= criteria.min_negative_margin_fraction,
        stable_future_directionality: directional_windows >= criteria.min_directional_windows,
        no_single_future_resolver_dominates: max_leader_fraction
            <= criteria.max_single_future_resolver_fraction,
    };

    IdentifiabilityAssessment {
        criteria,
        overall,
        windows: window_summaries,
        directional_windows,
        gates,
        passed: gates.passed(),
    }
}

fn summarize_identifiability(
    observations: &[OntologyObservation],
    margin_threshold: f64,
) -> IdentifiabilitySummary {
    IdentifiabilitySummary {
        leaders: leader_distribution(observations),
        margin: margin_summary(observations, margin_threshold),
    }
}

fn leader_distribution(observations: &[OntologyObservation]) -> ResolverLeaderDistribution {
    let mut reasoning = 0usize;
    let mut causal = 0usize;
    let mut prediction = 0usize;
    let mut metacognition = 0usize;
    let mut ties = 0usize;

    for observation in observations {
        let scores = [
            ("reasoning", resolver_score(observation, "reasoning")),
            ("causal", resolver_score(observation, "causal")),
            ("prediction", resolver_score(observation, "prediction")),
            (
                "metacognition",
                resolver_score(observation, "metacognition"),
            ),
        ];
        let best = scores
            .iter()
            .map(|(_, score)| *score)
            .fold(f64::NEG_INFINITY, f64::max);
        let winners: Vec<&str> = scores
            .iter()
            .filter(|(_, score)| (*score - best).abs() <= TIE_EPSILON)
            .map(|(name, _)| *name)
            .collect();
        if winners.len() != 1 {
            ties += 1;
            continue;
        }
        match winners[0] {
            "reasoning" => reasoning += 1,
            "causal" => causal += 1,
            "prediction" => prediction += 1,
            "metacognition" => metacognition += 1,
            _ => {}
        }
    }

    let total = observations.len().max(1) as f64;
    ResolverLeaderDistribution {
        observations: observations.len(),
        reasoning_fraction: reasoning as f64 / total,
        causal_fraction: causal as f64 / total,
        prediction_fraction: prediction as f64 / total,
        metacognition_fraction: metacognition as f64 / total,
        tie_fraction: ties as f64 / total,
    }
}

fn margin_summary(
    observations: &[OntologyObservation],
    margin_threshold: f64,
) -> ResolverMarginSummary {
    let mut margins: Vec<f64> = observations
        .iter()
        .map(|observation| {
            resolver_score(observation, "reasoning") - resolver_score(observation, "causal")
        })
        .collect();
    margins.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let count = margins.len().max(1) as f64;
    let mean = margins.iter().sum::<f64>() / count;
    let stddev = (margins
        .iter()
        .map(|margin| {
            let delta = *margin - mean;
            delta * delta
        })
        .sum::<f64>()
        / count)
        .sqrt();
    let positive = margins
        .iter()
        .filter(|margin| **margin >= margin_threshold)
        .count() as f64;
    let negative = margins
        .iter()
        .filter(|margin| **margin <= -margin_threshold)
        .count() as f64;
    let ambiguous = margins
        .iter()
        .filter(|margin| margin.abs() < margin_threshold)
        .count() as f64;

    ResolverMarginSummary {
        observations: margins.len(),
        mean,
        median: quantile(&margins, 0.50),
        stddev,
        q10: quantile(&margins, 0.10),
        q25: quantile(&margins, 0.25),
        q50: quantile(&margins, 0.50),
        q75: quantile(&margins, 0.75),
        q90: quantile(&margins, 0.90),
        fraction_at_least_positive: positive / count,
        fraction_at_most_negative: negative / count,
        ambiguous_fraction: ambiguous / count,
    }
}

fn resolver_score(observation: &OntologyObservation, resolver: &str) -> f64 {
    let mut total = 0.0;
    let mut attempts = 0usize;
    for outcome in observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
    {
        total += (outcome.discharged as f64 / observation.charge.magnitude as f64).clamp(0.0, 1.0)
            / outcome.compute_cost as f64;
        attempts += 1;
    }
    if attempts == 0 {
        0.0
    } else {
        total / attempts as f64
    }
}

fn quantile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    let position = percentile.clamp(0.0, 1.0) * (sorted_values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = position - lower as f64;
        sorted_values[lower] + (sorted_values[upper] - sorted_values[lower]) * weight
    }
}

#[cfg(test)]
mod tests {
    use crate::charge::{
        assess_resolver_identifiability, Charge, ChargeKind, ChargeScope, IdentifiabilityCriteria,
        OntologyObservation, ResolverOutcome,
    };

    fn observation(id: u64, outcomes: [f32; 4]) -> OntologyObservation {
        let mut charge = Charge::new(
            ChargeKind::Custom("unresolved".into()),
            vec![id as f32 / 100.0],
            1.0,
            ChargeScope::Global,
        );
        charge.id = id;
        OntologyObservation::new(charge)
            .with_outcome(ResolverOutcome::new("reasoning", outcomes[0], 1))
            .with_outcome(ResolverOutcome::new("causal", outcomes[1], 1))
            .with_outcome(ResolverOutcome::new("prediction", outcomes[2], 1))
            .with_outcome(ResolverOutcome::new("metacognition", outcomes[3], 1))
    }

    #[test]
    fn h5b_identifiability_passes_when_reasoning_and_causal_both_have_stable_margins() {
        let mut windows = Vec::new();
        for window in 0..4 {
            let mut observations = Vec::new();
            for index in 0..10 {
                observations.push(observation(1 + window * 100 + index, [0.9, 0.2, 0.1, 0.1]));
            }
            for index in 0..10 {
                observations.push(observation(21 + window * 100 + index, [0.2, 0.9, 0.1, 0.1]));
            }
            for index in 0..4 {
                observations.push(observation(41 + window * 100 + index, [0.4, 0.4, 0.6, 0.1]));
            }
            windows.push(observations);
        }

        let assessment =
            assess_resolver_identifiability(&windows, IdentifiabilityCriteria::h5_default());

        assert!(assessment.passed);
        assert!(assessment.overall.leaders.reasoning_fraction >= 0.20);
        assert!(assessment.overall.leaders.causal_fraction >= 0.20);
        assert!(assessment.overall.margin.fraction_at_least_positive >= 0.15);
        assert!(assessment.overall.margin.fraction_at_most_negative >= 0.15);
    }

    #[test]
    fn h5b_identifiability_rejects_single_resolver_dominance() {
        let windows = (0..4)
            .map(|window| {
                (0..20)
                    .map(|index| observation(1 + window * 100 + index, [0.9, 0.2, 0.1, 0.1]))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let assessment =
            assess_resolver_identifiability(&windows, IdentifiabilityCriteria::h5_default());

        assert!(!assessment.passed);
        assert!(!assessment.gates.causal_leader_floor);
        assert!(!assessment.gates.negative_margin_floor);
        assert!(!assessment.gates.no_single_future_resolver_dominates);
    }
}
