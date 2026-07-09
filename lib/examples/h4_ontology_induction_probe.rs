use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use rand::prelude::*;
use serde::Serialize;
use star::charge::{
    Charge, ChargeKind, ChargeScope, ConceptEvidence, ConceptPredicate, ConceptUtility, Direction,
    OntologyInducer, PromotionCriteria,
};

const TRAIN_PER_CLASS: usize = 48;
const HOLDOUT_PER_CLASS: usize = 48;
const RESOLVERS: [&str; 3] = ["memory", "reasoning", "causal"];
const MIN_SUPPORT: usize = 16;
const TOP_K: usize = 12;
const COMPUTE_BUDGET: u64 = 1;
const STOP_MAGNITUDE: f64 = 1e-9;

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
struct Observation {
    id: u64,
    charge: Charge,
    outcomes: [f64; 3],
    hidden: HiddenClass,
}

#[derive(Debug, Clone)]
struct Candidate {
    predicate: ConceptPredicate,
    training_gain: f64,
    complexity: f64,
    support: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Metrics {
    mean_discharge_efficiency: f64,
    mean_remaining: f64,
    solve_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ConceptReport {
    id: u64,
    predicate: String,
    support: usize,
    holdout_gain: f64,
    utility_gain: f64,
    dominant_hidden_class: HiddenClass,
    dominant_fraction: f64,
    resolver_leader: String,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    seed: u64,
    train_observations: usize,
    holdout_observations: usize,
    promoted_concepts: Vec<ConceptReport>,
    policy_metrics: BTreeMap<&'static str, Metrics>,
    diagnostics: Diagnostics,
    criteria: Criteria,
    pass: bool,
}

#[derive(Debug, Serialize)]
struct Diagnostics {
    induced_vs_undifferentiated_efficiency: f64,
    remaining_ratio_vs_undifferentiated: f64,
    solve_rate_margin: f64,
    oracle_efficiency_retention: f64,
    induced_vs_random_efficiency: f64,
    induced_vs_permuted_efficiency: f64,
}

#[derive(Debug, Serialize)]
struct Criteria {
    at_least_two_promoted: bool,
    efficiency_vs_undifferentiated: bool,
    remaining_vs_undifferentiated: bool,
    solve_rate_margin: bool,
    oracle_retention: bool,
    beats_random_partition: bool,
    beats_permuted_features: bool,
    promoted_have_positive_holdout_gain: bool,
}

fn main() {
    let seed = 0x48445f4f4e544f4c;
    let train = generate_observations(seed, TRAIN_PER_CLASS, 0);
    let holdout = generate_observations(seed ^ 0x9e3779b97f4a7c15, HOLDOUT_PER_CLASS, 10_000);

    let proposals = propose_thresholds(&train);
    let mut inducer = OntologyInducer::new(PromotionCriteria {
        min_observations: MIN_SUPPORT as u64,
        min_holdout_gain: 0.02,
        min_total_utility_gain: 0.02,
    });

    let mut promoted = Vec::new();
    for candidate in proposals.iter().take(TOP_K) {
        let holdout_gain = split_gain(&holdout, &candidate.predicate);
        let utility = ConceptUtility {
            routing_gain: holdout_gain,
            discharge_gain: holdout_gain,
            ..ConceptUtility::default()
        };
        let evidence = ConceptEvidence {
            observations: candidate.support as u64,
            positive_instances: membership_ids(&holdout, &candidate.predicate, true),
            negative_instances: membership_ids(&holdout, &candidate.predicate, false),
            holdout_gain,
        };
        let concept = inducer.propose(None, candidate.predicate.clone(), evidence, utility);
        if inducer.promote(concept.clone()) {
            promoted.push(concept);
        }
    }

    promoted.sort_by(|left, right| {
        right
            .evidence
            .holdout_gain
            .partial_cmp(&left.evidence.holdout_gain)
            .unwrap_or(Ordering::Equal)
    });
    promoted.truncate(2);

    let induced_predicates: Vec<ConceptPredicate> = promoted
        .iter()
        .map(|concept| concept.predicate.clone())
        .collect();

    let undifferentiated = evaluate_parent_policy(&train, &holdout);
    let induced = evaluate_induced_policy(&train, &holdout, &induced_predicates);
    let oracle = evaluate_hidden_oracle(&train, &holdout);
    let random = evaluate_random_partition(&train, &holdout, induced_predicates.len(), seed ^ 0xa11ce);
    let permuted = evaluate_permuted_search(&train, &holdout, seed ^ 0x55aa55aa);

    let efficiency_ratio = ratio(induced.mean_discharge_efficiency, undifferentiated.mean_discharge_efficiency);
    let remaining_ratio = ratio(induced.mean_remaining, undifferentiated.mean_remaining);
    let solve_margin = induced.solve_rate - undifferentiated.solve_rate;
    let oracle_retention = ratio(induced.mean_discharge_efficiency, oracle.mean_discharge_efficiency);
    let random_ratio = ratio(induced.mean_discharge_efficiency, random.mean_discharge_efficiency);
    let permuted_ratio = ratio(induced.mean_discharge_efficiency, permuted.mean_discharge_efficiency);

    let criteria = Criteria {
        at_least_two_promoted: promoted.len() >= 2,
        efficiency_vs_undifferentiated: efficiency_ratio >= 1.25,
        remaining_vs_undifferentiated: remaining_ratio <= 0.75,
        solve_rate_margin: solve_margin >= 0.20,
        oracle_retention: oracle_retention >= 0.80,
        beats_random_partition: random_ratio >= 1.25,
        beats_permuted_features: permuted_ratio >= 1.25,
        promoted_have_positive_holdout_gain: promoted.iter().all(|concept| {
            concept.evidence.holdout_gain > 0.0
                && concept.evidence.observations >= MIN_SUPPORT as u64
        }),
    };

    let pass = criteria.at_least_two_promoted
        && criteria.efficiency_vs_undifferentiated
        && criteria.remaining_vs_undifferentiated
        && criteria.solve_rate_margin
        && criteria.oracle_retention
        && criteria.beats_random_partition
        && criteria.beats_permuted_features
        && criteria.promoted_have_positive_holdout_gain;

    let promoted_concepts = promoted
        .iter()
        .map(|concept| {
            let members: Vec<&Observation> = holdout
                .iter()
                .filter(|observation| concept.predicate.matches(&observation.charge))
                .collect();
            let (dominant_hidden_class, dominant_fraction) = dominant_hidden(&members);
            let resolver_leader = leader_for_subset(&train, &concept.predicate);
            ConceptReport {
                id: concept.id.0,
                predicate: format!("{:?}", concept.predicate),
                support: members.len(),
                holdout_gain: concept.evidence.holdout_gain,
                utility_gain: concept.utility.total_gain(),
                dominant_hidden_class,
                dominant_fraction,
                resolver_leader: RESOLVERS[resolver_leader].to_string(),
            }
        })
        .collect();

    let mut policy_metrics = BTreeMap::new();
    policy_metrics.insert("induced", induced);
    policy_metrics.insert("oracle_hidden_classes", oracle);
    policy_metrics.insert("permuted_feature_search", permuted);
    policy_metrics.insert("random_partition", random);
    policy_metrics.insert("undifferentiated", undifferentiated);

    let report = Report {
        experiment: "H4 latent distinction induction",
        seed,
        train_observations: train.len(),
        holdout_observations: holdout.len(),
        promoted_concepts,
        policy_metrics,
        diagnostics: Diagnostics {
            induced_vs_undifferentiated_efficiency: efficiency_ratio,
            remaining_ratio_vs_undifferentiated: remaining_ratio,
            solve_rate_margin: solve_margin,
            oracle_efficiency_retention: oracle_retention,
            induced_vs_random_efficiency: random_ratio,
            induced_vs_permuted_efficiency: permuted_ratio,
        },
        criteria,
        pass,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if !pass {
        std::process::exit(1);
    }
}

fn generate_observations(seed: u64, per_class: usize, id_offset: u64) -> Vec<Observation> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut observations = Vec::with_capacity(per_class * 3);
    let mut next_id = id_offset;

    for hidden in HiddenClass::all() {
        for _ in 0..per_class {
            let jitter = || rng.gen_range(-0.045f32..0.045f32);
            let (residual, outcomes, persistence) = match hidden {
                HiddenClass::Gap => (
                    vec![0.88 + jitter(), 0.14 + jitter(), 0.18 + jitter()],
                    noisy_outcomes(&mut rng, [0.88, 0.24, 0.18]),
                    rng.gen_range(4..8),
                ),
                HiddenClass::Contradiction => (
                    vec![0.16 + jitter(), 0.87 + jitter(), 0.20 + jitter()],
                    noisy_outcomes(&mut rng, [0.22, 0.90, 0.20]),
                    rng.gen_range(1..5),
                ),
                HiddenClass::Residual => (
                    vec![0.18 + jitter(), 0.18 + jitter(), 0.89 + jitter()],
                    noisy_outcomes(&mut rng, [0.18, 0.23, 0.91]),
                    rng.gen_range(2..6),
                ),
            };

            let mut charge = Charge::new(
                ChargeKind::Custom("unresolved".into()),
                residual,
                1.0,
                ChargeScope::Topic(format!("opaque-{next_id}")),
            );
            charge.id = next_id + 1;
            charge.persistence = persistence;
            observations.push(Observation {
                id: charge.id,
                charge,
                outcomes,
                hidden,
            });
            next_id += 1;
        }
    }

    observations.shuffle(&mut rng);
    observations
}

fn noisy_outcomes(rng: &mut StdRng, base: [f64; 3]) -> [f64; 3] {
    let mut output = base;
    for value in &mut output {
        *value = (*value + rng.gen_range(-0.035..0.035)).clamp(0.0, 1.0);
    }
    output
}

fn propose_thresholds(observations: &[Observation]) -> Vec<Candidate> {
    let dimensions = observations
        .first()
        .map(|observation| observation.charge.residual.len())
        .unwrap_or(0);
    let mut candidates = Vec::new();

    for dimension in 0..dimensions {
        let mut values: Vec<f32> = observations
            .iter()
            .filter_map(|observation| observation.charge.residual.get(dimension).copied())
            .collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        values.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        for pair in values.windows(2) {
            let threshold = (pair[0] + pair[1]) * 0.5;
            for direction in [Direction::AtLeast, Direction::AtMost] {
                let predicate = ConceptPredicate::ResidualThreshold {
                    dimension,
                    threshold,
                    direction,
                };
                let support = observations
                    .iter()
                    .filter(|observation| predicate.matches(&observation.charge))
                    .count();
                if support < MIN_SUPPORT || observations.len() - support < MIN_SUPPORT {
                    continue;
                }
                let complexity = 0.002;
                let gain = split_gain(observations, &predicate) - complexity;
                candidates.push(Candidate {
                    predicate,
                    training_gain: gain,
                    complexity,
                    support,
                });
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .training_gain
            .partial_cmp(&left.training_gain)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.complexity.partial_cmp(&right.complexity).unwrap_or(Ordering::Equal))
    });

    let mut unique = Vec::new();
    let mut seen_memberships: Vec<Vec<u64>> = Vec::new();
    for candidate in candidates {
        let ids = membership_ids(observations, &candidate.predicate, true);
        if seen_memberships.iter().any(|seen| *seen == ids) {
            continue;
        }
        seen_memberships.push(ids);
        unique.push(candidate);
    }
    unique
}

fn split_gain(observations: &[Observation], predicate: &ConceptPredicate) -> f64 {
    let parent = best_mean_outcome(observations.iter());
    let left: Vec<&Observation> = observations
        .iter()
        .filter(|observation| predicate.matches(&observation.charge))
        .collect();
    let right: Vec<&Observation> = observations
        .iter()
        .filter(|observation| !predicate.matches(&observation.charge))
        .collect();
    if left.len() < MIN_SUPPORT || right.len() < MIN_SUPPORT {
        return f64::NEG_INFINITY;
    }

    let weighted = (left.len() as f64 * best_mean_outcome(left.iter().copied())
        + right.len() as f64 * best_mean_outcome(right.iter().copied()))
        / observations.len() as f64;
    weighted - parent
}

fn best_mean_outcome<'a>(observations: impl Iterator<Item = &'a Observation>) -> f64 {
    let collected: Vec<&Observation> = observations.collect();
    if collected.is_empty() {
        return 0.0;
    }
    (0..RESOLVERS.len())
        .map(|resolver| {
            collected
                .iter()
                .map(|observation| observation.outcomes[resolver])
                .sum::<f64>()
                / collected.len() as f64
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

fn leader_for_subset(observations: &[Observation], predicate: &ConceptPredicate) -> usize {
    let members: Vec<&Observation> = observations
        .iter()
        .filter(|observation| predicate.matches(&observation.charge))
        .collect();
    best_resolver(members.iter().copied())
}

fn best_resolver<'a>(observations: impl Iterator<Item = &'a Observation>) -> usize {
    let collected: Vec<&Observation> = observations.collect();
    (0..RESOLVERS.len())
        .max_by(|left, right| {
            let left_mean = collected
                .iter()
                .map(|observation| observation.outcomes[*left])
                .sum::<f64>()
                / collected.len().max(1) as f64;
            let right_mean = collected
                .iter()
                .map(|observation| observation.outcomes[*right])
                .sum::<f64>()
                / collected.len().max(1) as f64;
            left_mean.partial_cmp(&right_mean).unwrap_or(Ordering::Equal)
        })
        .unwrap_or(0)
}

fn evaluate_parent_policy(train: &[Observation], holdout: &[Observation]) -> Metrics {
    let resolver = best_resolver(train.iter());
    evaluate_with_selector(holdout, |_| resolver)
}

fn evaluate_induced_policy(
    train: &[Observation],
    holdout: &[Observation],
    predicates: &[ConceptPredicate],
) -> Metrics {
    let parent = best_resolver(train.iter());
    let leaders: Vec<usize> = predicates
        .iter()
        .map(|predicate| leader_for_subset(train, predicate))
        .collect();
    evaluate_with_selector(holdout, |observation| {
        predicates
            .iter()
            .position(|predicate| predicate.matches(&observation.charge))
            .map(|index| leaders[index])
            .unwrap_or(parent)
    })
}

fn evaluate_hidden_oracle(train: &[Observation], holdout: &[Observation]) -> Metrics {
    let mut leaders = HashMap::new();
    for hidden in HiddenClass::all() {
        let subset: Vec<&Observation> = train.iter().filter(|observation| observation.hidden == hidden).collect();
        leaders.insert(hidden, best_resolver(subset.iter().copied()));
    }
    evaluate_with_selector(holdout, |observation| leaders[&observation.hidden])
}

fn evaluate_random_partition(
    train: &[Observation],
    holdout: &[Observation],
    concept_count: usize,
    seed: u64,
) -> Metrics {
    if concept_count == 0 {
        return evaluate_parent_policy(train, holdout);
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let train_assignment: HashMap<u64, usize> = train
        .iter()
        .map(|observation| (observation.id, rng.gen_range(0..=concept_count)))
        .collect();
    let holdout_assignment: HashMap<u64, usize> = holdout
        .iter()
        .map(|observation| (observation.id, rng.gen_range(0..=concept_count)))
        .collect();
    let leaders: Vec<usize> = (0..=concept_count)
        .map(|group| {
            let subset: Vec<&Observation> = train
                .iter()
                .filter(|observation| train_assignment[&observation.id] == group)
                .collect();
            best_resolver(subset.iter().copied())
        })
        .collect();
    evaluate_with_selector(holdout, |observation| leaders[holdout_assignment[&observation.id]])
}

fn evaluate_permuted_search(train: &[Observation], holdout: &[Observation], seed: u64) -> Metrics {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut train_permuted = train.to_vec();
    let mut holdout_permuted = holdout.to_vec();
    permute_visible_features(&mut train_permuted, &mut rng);
    permute_visible_features(&mut holdout_permuted, &mut rng);
    let predicates: Vec<ConceptPredicate> = propose_thresholds(&train_permuted)
        .into_iter()
        .take(2)
        .map(|candidate| candidate.predicate)
        .collect();
    evaluate_induced_policy(&train_permuted, &holdout_permuted, &predicates)
}

fn permute_visible_features(observations: &mut [Observation], rng: &mut StdRng) {
    if observations.is_empty() {
        return;
    }
    let dimensions = observations[0].charge.residual.len();
    for dimension in 0..dimensions {
        let mut values: Vec<f32> = observations
            .iter()
            .map(|observation| observation.charge.residual[dimension])
            .collect();
        values.shuffle(rng);
        for (observation, value) in observations.iter_mut().zip(values) {
            observation.charge.residual[dimension] = value;
        }
    }
    let mut persistence: Vec<u32> = observations.iter().map(|observation| observation.charge.persistence).collect();
    persistence.shuffle(rng);
    for (observation, value) in observations.iter_mut().zip(persistence) {
        observation.charge.persistence = value;
    }
}

fn evaluate_with_selector(
    observations: &[Observation],
    selector: impl Fn(&Observation) -> usize,
) -> Metrics {
    let mut total_discharge = 0.0;
    let mut total_remaining = 0.0;
    let mut solved = 0usize;
    for observation in observations {
        let resolver = selector(observation);
        let discharge = observation.outcomes[resolver].clamp(0.0, 1.0);
        let remaining = (1.0 - discharge).max(0.0);
        total_discharge += discharge;
        total_remaining += remaining;
        if remaining <= 0.25 + STOP_MAGNITUDE {
            solved += 1;
        }
    }
    let count = observations.len().max(1) as f64;
    Metrics {
        mean_discharge_efficiency: total_discharge / (count * COMPUTE_BUDGET as f64),
        mean_remaining: total_remaining / count,
        solve_rate: solved as f64 / count,
    }
}

fn membership_ids(
    observations: &[Observation],
    predicate: &ConceptPredicate,
    matching: bool,
) -> Vec<u64> {
    let mut ids: Vec<u64> = observations
        .iter()
        .filter(|observation| predicate.matches(&observation.charge) == matching)
        .map(|observation| observation.id)
        .collect();
    ids.sort_unstable();
    ids
}

fn dominant_hidden(observations: &[&Observation]) -> (HiddenClass, f64) {
    let mut counts = HashMap::new();
    for observation in observations {
        *counts.entry(observation.hidden).or_insert(0usize) += 1;
    }
    let (hidden, count) = HiddenClass::all()
        .into_iter()
        .map(|hidden| (hidden, counts.get(&hidden).copied().unwrap_or(0)))
        .max_by_key(|(_, count)| *count)
        .unwrap();
    (hidden, count as f64 / observations.len().max(1) as f64)
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() <= f64::EPSILON {
        if numerator.abs() <= f64::EPSILON { 1.0 } else { f64::INFINITY }
    } else {
        numerator / denominator
    }
}
