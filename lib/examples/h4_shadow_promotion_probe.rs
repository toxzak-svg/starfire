use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use rand::prelude::*;
use serde::Serialize;
use star::charge::{
    Charge, ChargeKind, ChargeScope, ConceptPredicate, Direction, EmpiricalInductionConfig,
    OntologyObservation, PromotionCriteria, ResolverOutcome, ShadowControlScore,
    ShadowPromotionConfig, ShadowPromotionMonitor, ShadowPromotionStatus,
};

const SEED: u64 = 0x4834_5348_4144_4f57;
const TRAIN_WINDOWS: usize = 2;
const HOLDOUT_WINDOWS: usize = 1;
const TRANSFER_WINDOWS: usize = 4;
const PER_CLASS: usize = 36;
const MAX_CONCEPTS: usize = 2;
const MIN_PARTITION_SUPPORT: usize = 28;
const MIN_HOLDOUT_SUPPORT: usize = 14;
const MAX_THRESHOLDS_PER_DIMENSION: usize = 96;
const COMPLEXITY_PENALTY: f64 = 0.003;
const MIN_PROMOTION_OBSERVATIONS: u64 = 28;
const MIN_PROMOTION_HOLDOUT_GAIN: f64 = 0.035;
const MIN_PROMOTION_UTILITY_GAIN: f64 = 0.035;
const MIN_TRANSFER_EFFICIENCY_RATIO: f64 = 1.35;
const MIN_TRANSFER_WIN_FRACTION: f64 = 1.0;
const MIN_WORST_WINDOW_RATIO: f64 = 1.20;
const MIN_CONTROL_EFFICIENCY_RATIO: f64 = 1.25;
#[allow(dead_code)] // Frozen resolver vocabulary.
const RESOLVERS: [&str; 3] = ["memory", "reasoning", "causal"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
enum HiddenClass {
    Gap,
    Contradiction,
    Residual,
}

impl HiddenClass {
    fn all() -> [Self; 3] {
        [Self::Gap, Self::Contradiction, Self::Residual]
    }
}

#[derive(Debug, Clone)]
struct LabeledObservation {
    observation: OntologyObservation,
    hidden: HiddenClass,
}

#[derive(Debug, Clone, Serialize)]
struct ControlReport {
    name: String,
    proposal_evaluations: usize,
    routing_evaluations: usize,
    training_efficiency: f64,
    holdout_gain: f64,
    applied: bool,
    future_efficiency: f64,
}

#[derive(Debug, Clone)]
struct ComputedControl {
    score: ShadowControlScore,
    report: ControlReport,
}

#[derive(Debug, Serialize)]
struct ConceptReport {
    id: u64,
    predicate: String,
    resolver: String,
    dominant_future_hidden_class: HiddenClass,
    future_purity: f64,
}

#[derive(Debug, Serialize)]
struct TransferWindowReport {
    index: usize,
    shadow_efficiency: f64,
    baseline_efficiency: f64,
    efficiency_ratio: f64,
}

#[derive(Debug, Serialize)]
struct GateReport {
    promoted_concepts: bool,
    transfer_efficiency: bool,
    transfer_window_wins: bool,
    worst_window: bool,
    matched_budget_controls: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    seed: u64,
    visible_charge_kind: &'static str,
    train_windows: usize,
    holdout_windows: usize,
    future_transfer_windows: usize,
    observations_per_window: usize,
    proposal_budget: usize,
    future_routing_budget: usize,
    concepts: Vec<ConceptReport>,
    transfer_windows: Vec<TransferWindowReport>,
    transfer_efficiency_ratio: f64,
    transfer_window_win_fraction: f64,
    worst_window_ratio: f64,
    controls: Vec<ControlReport>,
    gate_thresholds: BTreeMap<&'static str, f64>,
    gates: GateReport,
    status: String,
    pass: bool,
}

#[derive(Debug, Clone)]
struct FeaturePolicy {
    predicates: Vec<ConceptPredicate>,
    resolvers: Vec<String>,
    parent_resolver: String,
}

impl FeaturePolicy {
    fn route(&self, charge: &Charge) -> &str {
        self.predicates
            .iter()
            .position(|predicate| predicate.matches(charge))
            .map(|index| self.resolvers[index].as_str())
            .unwrap_or(self.parent_resolver.as_str())
    }
}

#[derive(Debug, Clone)]
struct HashPartitionPolicy {
    salt: u64,
    resolvers: Vec<String>,
}

impl HashPartitionPolicy {
    fn group(&self, observation: &OntologyObservation) -> usize {
        let id = if observation.charge.id == 0 {
            observation.charge.magnitude.to_bits() as u64
        } else {
            observation.charge.id
        };
        (mix64(id ^ self.salt) % self.resolvers.len().max(1) as u64) as usize
    }

    fn route(&self, observation: &OntologyObservation) -> &str {
        self.resolvers[self.group(observation)].as_str()
    }
}

fn main() {
    let specs = [
        ("train-surface-a", 0.00, SEED ^ 0x11),
        ("train-surface-b", 0.03, SEED ^ 0x22),
        ("holdout-surface", 0.06, SEED ^ 0x33),
        ("future-family-a", 0.09, SEED ^ 0x44),
        ("future-family-b", 0.12, SEED ^ 0x55),
        ("future-family-c", 0.15, SEED ^ 0x66),
        ("future-family-d", 0.18, SEED ^ 0x77),
    ];
    let windows: Vec<Vec<LabeledObservation>> = specs
        .iter()
        .enumerate()
        .map(|(index, (surface, drift, seed))| {
            generate_window(*seed, index as u64 * 10_000 + 1, surface, *drift)
        })
        .collect();

    let config = frozen_config();
    let mut monitor = ShadowPromotionMonitor::new(config).unwrap();
    for window in &windows {
        let observations = window
            .iter()
            .map(|observation| observation.observation.clone())
            .collect();
        monitor.observe_window(observations).unwrap();
    }

    assert_eq!(
        monitor.status(),
        ShadowPromotionStatus::AwaitingMatchedBudgetControls
    );
    let budget = monitor.required_control_budget().unwrap();
    let training = flatten_plain(&windows[..TRAIN_WINDOWS]);
    let holdout = flatten_plain(&windows[TRAIN_WINDOWS..TRAIN_WINDOWS + HOLDOUT_WINDOWS]);
    let future = plain_windows(&windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..]);
    let concept_count = monitor
        .learned_ontology()
        .unwrap()
        .routes()
        .len()
        .max(1);

    let random = matched_random_partition_control(
        &training,
        &holdout,
        &future,
        budget.proposal_evaluations,
        concept_count + 1,
        SEED ^ 0xa11c_e,
    );
    let permuted = matched_permuted_feature_control(
        &training,
        &holdout,
        &future,
        budget.proposal_evaluations,
        concept_count,
        SEED ^ 0x55aa_55aa,
    );
    let controls = vec![random, permuted];
    let control_scores: Vec<ShadowControlScore> =
        controls.iter().map(|control| control.score.clone()).collect();
    let assessment = monitor.assess_controls(&control_scores).unwrap().clone();

    let future_labeled = &windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..];
    let concepts = monitor
        .learned_ontology()
        .unwrap()
        .routes()
        .iter()
        .map(|route| {
            let members: Vec<&LabeledObservation> = future_labeled
                .iter()
                .flat_map(|window| window.iter())
                .filter(|observation| route.concept.predicate.matches(&observation.observation.charge))
                .collect();
            let (dominant_future_hidden_class, future_purity) = dominant_hidden(&members);
            ConceptReport {
                id: route.concept.id.as_u64(),
                predicate: format!("{:?}", route.concept.predicate),
                resolver: route.resolver.clone(),
                dominant_future_hidden_class,
                future_purity,
            }
        })
        .collect();

    let transfer_windows = monitor
        .transfer_windows()
        .iter()
        .enumerate()
        .map(|(index, metrics)| TransferWindowReport {
            index,
            shadow_efficiency: metrics.shadow_efficiency,
            baseline_efficiency: metrics.baseline_efficiency,
            efficiency_ratio: metrics.efficiency_ratio,
        })
        .collect();

    let mut gate_thresholds = BTreeMap::new();
    gate_thresholds.insert("min_control_efficiency_ratio", MIN_CONTROL_EFFICIENCY_RATIO);
    gate_thresholds.insert("min_transfer_efficiency_ratio", MIN_TRANSFER_EFFICIENCY_RATIO);
    gate_thresholds.insert("min_transfer_win_fraction", MIN_TRANSFER_WIN_FRACTION);
    gate_thresholds.insert("min_worst_window_ratio", MIN_WORST_WINDOW_RATIO);

    let report = Report {
        experiment: "H4 shadow-only online latent-concept promotion transfer",
        seed: SEED,
        visible_charge_kind: "Custom(unresolved)",
        train_windows: TRAIN_WINDOWS,
        holdout_windows: HOLDOUT_WINDOWS,
        future_transfer_windows: TRANSFER_WINDOWS,
        observations_per_window: PER_CLASS * HiddenClass::all().len(),
        proposal_budget: assessment.budget.proposal_evaluations,
        future_routing_budget: assessment.budget.routing_evaluations,
        concepts,
        transfer_windows,
        transfer_efficiency_ratio: assessment.transfer.efficiency_ratio,
        transfer_window_win_fraction: assessment.transfer.window_win_fraction,
        worst_window_ratio: assessment.transfer.worst_window_ratio,
        controls: controls.into_iter().map(|control| control.report).collect(),
        gate_thresholds,
        gates: GateReport {
            promoted_concepts: assessment.criteria.promoted_concepts,
            transfer_efficiency: assessment.criteria.transfer_efficiency,
            transfer_window_wins: assessment.criteria.transfer_window_wins,
            worst_window: assessment.criteria.worst_window,
            matched_budget_controls: assessment.criteria.matched_budget_controls,
        },
        status: format!("{:?}", assessment.status),
        pass: assessment.status == ShadowPromotionStatus::Eligible,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if !report.pass {
        std::process::exit(1);
    }
}

fn frozen_config() -> ShadowPromotionConfig {
    ShadowPromotionConfig {
        training_windows: TRAIN_WINDOWS,
        holdout_windows: HOLDOUT_WINDOWS,
        transfer_windows: TRANSFER_WINDOWS,
        min_promoted_concepts: 2,
        min_transfer_efficiency_ratio: MIN_TRANSFER_EFFICIENCY_RATIO,
        min_transfer_win_fraction: MIN_TRANSFER_WIN_FRACTION,
        min_worst_window_ratio: MIN_WORST_WINDOW_RATIO,
        min_control_efficiency_ratio: MIN_CONTROL_EFFICIENCY_RATIO,
        induction: EmpiricalInductionConfig {
            max_concepts: MAX_CONCEPTS,
            min_partition_support: MIN_PARTITION_SUPPORT,
            min_holdout_support: MIN_HOLDOUT_SUPPORT,
            max_thresholds_per_dimension: MAX_THRESHOLDS_PER_DIMENSION,
            complexity_penalty: COMPLEXITY_PENALTY,
            promotion: PromotionCriteria {
                min_observations: MIN_PROMOTION_OBSERVATIONS,
                min_holdout_gain: MIN_PROMOTION_HOLDOUT_GAIN,
                min_total_utility_gain: MIN_PROMOTION_UTILITY_GAIN,
            },
        },
    }
}

fn generate_window(
    seed: u64,
    id_offset: u64,
    surface: &str,
    drift: f32,
) -> Vec<LabeledObservation> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut observations = Vec::with_capacity(PER_CLASS * HiddenClass::all().len());
    let mut next_id = id_offset;

    for hidden in HiddenClass::all() {
        for index in 0..PER_CLASS {
            let mut residual = match hidden {
                HiddenClass::Gap => vec![0.74 + drift * 0.20, 0.34, 0.37],
                HiddenClass::Contradiction => vec![0.36, 0.75 - drift * 0.08, 0.38],
                HiddenClass::Residual => vec![0.38, 0.35, 0.76 + drift * 0.12],
            };
            for value in &mut residual {
                *value = (*value + triangular_noise(&mut rng, 0.18)).clamp(0.0, 1.0);
            }
            if rng.gen_bool(0.12) {
                let other = match hidden {
                    HiddenClass::Gap => 2,
                    HiddenClass::Contradiction => 0,
                    HiddenClass::Residual => 1,
                };
                let favored = match hidden {
                    HiddenClass::Gap => 0,
                    HiddenClass::Contradiction => 1,
                    HiddenClass::Residual => 2,
                };
                residual.swap(favored, other);
            }

            let base = match hidden {
                HiddenClass::Gap => [0.84, 0.23, 0.19],
                HiddenClass::Contradiction => [0.21, 0.85, 0.22],
                HiddenClass::Residual => [0.19, 0.24, 0.86],
            };
            let outcomes = noisy_outcomes(&mut rng, base, drift);
            let mut charge = Charge::new(
                ChargeKind::Custom("unresolved".into()),
                residual,
                1.0,
                ChargeScope::Topic(format!("{surface}-{hidden:?}-{index}")),
            );
            charge.id = next_id;
            charge.persistence = match hidden {
                HiddenClass::Gap => rng.gen_range(4..9),
                HiddenClass::Contradiction => rng.gen_range(1..6),
                HiddenClass::Residual => rng.gen_range(2..7),
            };
            let observation = OntologyObservation::new(charge)
                .with_outcome(ResolverOutcome::new("memory", outcomes[0], 1))
                .with_outcome(ResolverOutcome::new("reasoning", outcomes[1], 1))
                .with_outcome(ResolverOutcome::new("causal", outcomes[2], 1));
            observations.push(LabeledObservation { observation, hidden });
            next_id += 1;
        }
    }

    observations.shuffle(&mut rng);
    observations
}

fn triangular_noise(rng: &mut StdRng, amplitude: f32) -> f32 {
    (rng.gen_range(-amplitude..amplitude) + rng.gen_range(-amplitude..amplitude)) * 0.5
}

fn noisy_outcomes(rng: &mut StdRng, base: [f32; 3], drift: f32) -> [f32; 3] {
    let mut output = base;
    for (index, value) in output.iter_mut().enumerate() {
        let drift_penalty = if *value > 0.5 { drift * 0.10 } else { 0.0 };
        *value = (*value - drift_penalty + triangular_noise(rng, 0.07 + index as f32 * 0.005))
            .clamp(0.0, 1.0);
    }
    output
}

fn flatten_plain(windows: &[Vec<LabeledObservation>]) -> Vec<OntologyObservation> {
    windows
        .iter()
        .flat_map(|window| window.iter())
        .map(|observation| observation.observation.clone())
        .collect()
}

fn plain_windows(windows: &[Vec<LabeledObservation>]) -> Vec<Vec<OntologyObservation>> {
    windows
        .iter()
        .map(|window| {
            window
                .iter()
                .map(|observation| observation.observation.clone())
                .collect()
        })
        .collect()
}

fn matched_random_partition_control(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    future: &[Vec<OntologyObservation>],
    proposal_budget: usize,
    groups: usize,
    seed: u64,
) -> ComputedControl {
    let resolvers = resolver_names(train);
    let baseline = best_resolver(train, &(0..train.len()).collect::<Vec<_>>(), &resolvers);
    let baseline_holdout = mean_named_efficiency(holdout, |_| baseline.as_str());
    let mut rng = StdRng::seed_from_u64(seed);
    let mut best_policy = None;
    let mut best_training = f64::NEG_INFINITY;

    for _ in 0..proposal_budget {
        let salt = rng.gen::<u64>();
        let mut group_indices = vec![Vec::<usize>::new(); groups.max(1)];
        for (index, observation) in train.iter().enumerate() {
            let id = if observation.charge.id == 0 {
                index as u64 + 1
            } else {
                observation.charge.id
            };
            let group = (mix64(id ^ salt) % group_indices.len() as u64) as usize;
            group_indices[group].push(index);
        }
        if group_indices
            .iter()
            .any(|indices| indices.len() < MIN_PARTITION_SUPPORT)
        {
            continue;
        }
        let policy = HashPartitionPolicy {
            salt,
            resolvers: group_indices
                .iter()
                .map(|indices| best_resolver(train, indices, &resolvers))
                .collect(),
        };
        let training_efficiency = mean_named_efficiency(train, |observation| policy.route(observation));
        if training_efficiency > best_training {
            best_training = training_efficiency;
            best_policy = Some(policy);
        }
    }

    let mut applied = false;
    let mut holdout_gain = 0.0;
    let future_efficiency = if let Some(policy) = best_policy {
        let holdout_groups_valid = (0..policy.resolvers.len()).all(|group| {
            holdout
                .iter()
                .filter(|observation| policy.group(observation) == group)
                .count()
                >= MIN_HOLDOUT_SUPPORT
        });
        let holdout_efficiency = mean_named_efficiency(holdout, |observation| policy.route(observation));
        holdout_gain = holdout_efficiency - baseline_holdout;
        applied = holdout_groups_valid && holdout_gain >= MIN_PROMOTION_HOLDOUT_GAIN;
        if applied {
            mean_future_efficiency(future, |observation| policy.route(observation).to_string())
        } else {
            mean_future_efficiency(future, |_| baseline.clone())
        }
    } else {
        best_training = mean_named_efficiency(train, |_| baseline.as_str());
        mean_future_efficiency(future, |_| baseline.clone())
    };

    let routing_evaluations = future.iter().map(Vec::len).sum();
    ComputedControl {
        score: ShadowControlScore::new(
            "matched_random_partition_search",
            proposal_budget,
            routing_evaluations,
            future_efficiency,
        ),
        report: ControlReport {
            name: "matched_random_partition_search".into(),
            proposal_evaluations: proposal_budget,
            routing_evaluations,
            training_efficiency: best_training,
            holdout_gain,
            applied,
            future_efficiency,
        },
    }
}

fn matched_permuted_feature_control(
    train: &[OntologyObservation],
    holdout: &[OntologyObservation],
    future: &[Vec<OntologyObservation>],
    proposal_budget: usize,
    concept_count: usize,
    seed: u64,
) -> ComputedControl {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut train = train.to_vec();
    let mut holdout = holdout.to_vec();
    let mut future = future.to_vec();
    permute_visible_features(&mut train, &mut rng);
    permute_visible_features(&mut holdout, &mut rng);
    for window in &mut future {
        permute_visible_features(window, &mut rng);
    }

    let pool = candidate_pool(&train);
    let baseline = build_feature_policy(&train, &[]);
    let baseline_holdout = feature_policy_efficiency(&holdout, &baseline);
    let mut best_policy = baseline.clone();
    let mut best_training = feature_policy_efficiency(&train, &baseline);

    for _ in 0..proposal_budget {
        if pool.len() < concept_count || concept_count == 0 {
            break;
        }
        let mut selected = BTreeSet::new();
        while selected.len() < concept_count {
            selected.insert(rng.gen_range(0..pool.len()));
        }
        let predicates: Vec<ConceptPredicate> = selected
            .into_iter()
            .map(|index| pool[index].clone())
            .collect();
        if !valid_feature_policy_support(&train, &predicates, MIN_PARTITION_SUPPORT) {
            continue;
        }
        let policy = build_feature_policy(&train, &predicates);
        let training_efficiency = feature_policy_efficiency(&train, &policy);
        if training_efficiency > best_training {
            best_training = training_efficiency;
            best_policy = policy;
        }
    }

    let holdout_efficiency = feature_policy_efficiency(&holdout, &best_policy);
    let holdout_gain = holdout_efficiency - baseline_holdout;
    let applied = !best_policy.predicates.is_empty()
        && valid_feature_policy_support(&holdout, &best_policy.predicates, MIN_HOLDOUT_SUPPORT)
        && holdout_gain >= MIN_PROMOTION_HOLDOUT_GAIN;
    let final_policy = if applied { &best_policy } else { &baseline };
    let future_efficiency = mean_future_efficiency(&future, |observation| {
        final_policy.route(&observation.charge).to_string()
    });
    let routing_evaluations = future.iter().map(Vec::len).sum();

    ComputedControl {
        score: ShadowControlScore::new(
            "matched_permuted_feature_search",
            proposal_budget,
            routing_evaluations,
            future_efficiency,
        ),
        report: ControlReport {
            name: "matched_permuted_feature_search".into(),
            proposal_evaluations: proposal_budget,
            routing_evaluations,
            training_efficiency: best_training,
            holdout_gain,
            applied,
            future_efficiency,
        },
    }
}

fn candidate_pool(observations: &[OntologyObservation]) -> Vec<ConceptPredicate> {
    let dimensions = observations
        .iter()
        .map(|observation| observation.charge.residual.len())
        .max()
        .unwrap_or(0);
    let mut predicates = Vec::new();
    for dimension in 0..dimensions {
        let mut values: Vec<f32> = observations
            .iter()
            .filter_map(|observation| observation.charge.residual.get(dimension).copied())
            .filter(|value| value.is_finite())
            .collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        values.dedup_by(|left, right| (*left - *right).abs() < f32::EPSILON);
        for threshold in bounded_midpoints(&values, MAX_THRESHOLDS_PER_DIMENSION) {
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
    predicates
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
        let index = (((slot * total) + (take / 2)) / take).min(total - 1);
        if seen.insert(index) {
            midpoints.push(values[index] + (values[index + 1] - values[index]) * 0.5);
        }
    }
    midpoints
}

fn build_feature_policy(
    observations: &[OntologyObservation],
    predicates: &[ConceptPredicate],
) -> FeaturePolicy {
    let resolvers = resolver_names(observations);
    let mut learned_resolvers = Vec::with_capacity(predicates.len());
    for (position, predicate) in predicates.iter().enumerate() {
        let indices = effective_membership(observations, &predicates[..position], predicate);
        learned_resolvers.push(best_resolver(observations, &indices, &resolvers));
    }
    let parent = parent_indices(observations, predicates);
    let parent_resolver = best_resolver(observations, &parent, &resolvers);
    FeaturePolicy {
        predicates: predicates.to_vec(),
        resolvers: learned_resolvers,
        parent_resolver,
    }
}

fn valid_feature_policy_support(
    observations: &[OntologyObservation],
    predicates: &[ConceptPredicate],
    min_support: usize,
) -> bool {
    for (position, predicate) in predicates.iter().enumerate() {
        if effective_membership(observations, &predicates[..position], predicate).len() < min_support {
            return false;
        }
    }
    parent_indices(observations, predicates).len() >= min_support
}

fn effective_membership(
    observations: &[OntologyObservation],
    active: &[ConceptPredicate],
    predicate: &ConceptPredicate,
) -> Vec<usize> {
    observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| {
            !active.iter().any(|active| active.matches(&observation.charge))
                && predicate.matches(&observation.charge)
        })
        .map(|(index, _)| index)
        .collect()
}

fn parent_indices(
    observations: &[OntologyObservation],
    predicates: &[ConceptPredicate],
) -> Vec<usize> {
    observations
        .iter()
        .enumerate()
        .filter(|(_, observation)| {
            !predicates
                .iter()
                .any(|predicate| predicate.matches(&observation.charge))
        })
        .map(|(index, _)| index)
        .collect()
}

fn feature_policy_efficiency(
    observations: &[OntologyObservation],
    policy: &FeaturePolicy,
) -> f64 {
    mean_named_efficiency(observations, |observation| policy.route(&observation.charge))
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

fn best_resolver(
    observations: &[OntologyObservation],
    indices: &[usize],
    resolvers: &[String],
) -> String {
    let indices: Vec<usize> = if indices.is_empty() {
        (0..observations.len()).collect()
    } else {
        indices.to_vec()
    };
    resolvers
        .iter()
        .max_by(|left, right| {
            mean_resolver_score(observations, &indices, left)
                .partial_cmp(&mean_resolver_score(observations, &indices, right))
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
    indices
        .iter()
        .map(|index| resolver_score(&observations[*index], resolver))
        .sum::<f64>()
        / indices.len().max(1) as f64
}

fn mean_named_efficiency<'a>(
    observations: &'a [OntologyObservation],
    resolver: impl Fn(&'a OntologyObservation) -> &'a str,
) -> f64 {
    observations
        .iter()
        .map(|observation| resolver_score(observation, resolver(observation)))
        .sum::<f64>()
        / observations.len().max(1) as f64
}

fn mean_future_efficiency(
    windows: &[Vec<OntologyObservation>],
    resolver: impl Fn(&OntologyObservation) -> String,
) -> f64 {
    let observations = windows.iter().map(Vec::len).sum::<usize>();
    windows
        .iter()
        .flat_map(|window| window.iter())
        .map(|observation| resolver_score(observation, &resolver(observation)))
        .sum::<f64>()
        / observations.max(1) as f64
}

fn resolver_score(observation: &OntologyObservation, resolver: &str) -> f64 {
    let mut total = 0.0;
    let mut attempts = 0usize;
    for outcome in observation
        .outcomes
        .iter()
        .filter(|outcome| outcome.resolver == resolver)
    {
        total += (outcome.discharged as f64 / observation.charge.magnitude as f64)
            .clamp(0.0, 1.0)
            / outcome.compute_cost as f64;
        attempts += 1;
    }
    if attempts == 0 {
        0.0
    } else {
        total / attempts as f64
    }
}

fn permute_visible_features(observations: &mut [OntologyObservation], rng: &mut StdRng) {
    let dimensions = observations
        .iter()
        .map(|observation| observation.charge.residual.len())
        .max()
        .unwrap_or(0);
    for dimension in 0..dimensions {
        let mut positions = Vec::new();
        let mut values = Vec::new();
        for (index, observation) in observations.iter().enumerate() {
            if let Some(value) = observation.charge.residual.get(dimension).copied() {
                positions.push(index);
                values.push(value);
            }
        }
        values.shuffle(rng);
        for (index, value) in positions.into_iter().zip(values) {
            observations[index].charge.residual[dimension] = value;
        }
    }
    let mut persistence: Vec<u32> = observations
        .iter()
        .map(|observation| observation.charge.persistence)
        .collect();
    persistence.shuffle(rng);
    for (observation, persistence) in observations.iter_mut().zip(persistence) {
        observation.charge.persistence = persistence;
    }
}

fn dominant_hidden(observations: &[&LabeledObservation]) -> (HiddenClass, f64) {
    let mut counts = HashMap::<HiddenClass, usize>::new();
    for observation in observations {
        *counts.entry(observation.hidden).or_default() += 1;
    }
    let (hidden, count) = HiddenClass::all()
        .into_iter()
        .map(|hidden| (hidden, counts.get(&hidden).copied().unwrap_or(0)))
        .max_by_key(|(_, count)| *count)
        .unwrap();
    (hidden, count as f64 / observations.len().max(1) as f64)
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
