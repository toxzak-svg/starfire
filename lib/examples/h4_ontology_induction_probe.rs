use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use rand::prelude::*;
use serde::Serialize;
use star::charge::{
    Charge, ChargeKind, ChargeScope, ConceptEvidence, ConceptPredicate, ConceptUtility, Direction,
    InducedConcept, OntologyInducer, PromotionCriteria,
};

const TRAIN_PER_CLASS: usize = 48;
const HOLDOUT_PER_CLASS: usize = 48;
const MIN_SUPPORT: usize = 16;
const MAX_CONCEPTS: usize = 2;
const RESOLVERS: [&str; 3] = ["memory", "reasoning", "causal"];
const SEED: u64 = 0x4844_5f4f_4e54_4f4c;

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
    support: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Metrics {
    mean_discharge_efficiency: f64,
    mean_remaining: f64,
    solve_rate: f64,
}

#[derive(Debug, Serialize)]
struct CandidateReport {
    predicate: String,
    training_gain: f64,
    support: usize,
}

#[derive(Debug, Serialize)]
struct ConceptReport {
    id: u64,
    predicate: String,
    training_support: usize,
    holdout_support: usize,
    holdout_gain: f64,
    utility_gain: f64,
    resolver_leader: String,
    dominant_hidden_class: HiddenClass,
    dominant_fraction: f64,
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

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    seed: u64,
    visible_charge_kind: &'static str,
    train_observations: usize,
    holdout_observations: usize,
    top_training_candidates: Vec<CandidateReport>,
    promoted_concepts: Vec<ConceptReport>,
    policy_metrics: BTreeMap<&'static str, Metrics>,
    diagnostics: Diagnostics,
    criteria: Criteria,
    pass: bool,
}

fn main() {
    let train = generate_observations(SEED, TRAIN_PER_CLASS, 0);
    let holdout = generate_observations(SEED ^ 0x9e37_79b9_7f4a_7c15, HOLDOUT_PER_CLASS, 10_000);

    let candidates = propose_thresholds(&train);
    let top_training_candidates = candidates
        .iter()
        .take(12)
        .map(|candidate| CandidateReport {
            predicate: format!("{:?}", candidate.predicate),
            training_gain: candidate.training_gain,
            support: candidate.support,
        })
        .collect();

    let mut inducer = OntologyInducer::new(PromotionCriteria {
        min_observations: MIN_SUPPORT as u64,
        min_holdout_gain: 0.02,
        min_total_utility_gain: 0.04,
    });
    let promoted = induce_greedily(&train, &holdout, &candidates, &mut inducer);
    let predicates: Vec<ConceptPredicate> = promoted
        .iter()
        .map(|concept| concept.predicate.clone())
        .collect();

    let undifferentiated = evaluate_parent(&train, &holdout);
    let induced = evaluate_predicates(&train, &holdout, &predicates);
    let oracle = evaluate_hidden_oracle(&train, &holdout);
    let random = evaluate_random_partition(
        &train,
        &holdout,
        predicates.len() + 1,
        SEED ^ 0xa11c_e55a,
    );
    let permuted = evaluate_permuted_control(&train, &holdout, SEED ^ 0x55aa_55aa);

    let efficiency_ratio = ratio(
        induced.mean_discharge_efficiency,
        undifferentiated.mean_discharge_efficiency,
    );
    let remaining_ratio = ratio(induced.mean_remaining, undifferentiated.mean_remaining);
    let solve_margin = induced.solve_rate - undifferentiated.solve_rate;
    let oracle_retention = ratio(
        induced.mean_discharge_efficiency,
        oracle.mean_discharge_efficiency,
    );
    let random_ratio = ratio(induced.mean_discharge_efficiency, random.mean_discharge_efficiency);
    let permuted_ratio = ratio(
        induced.mean_discharge_efficiency,
        permuted.mean_discharge_efficiency,
    );

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
        .map(|concept| concept_report(concept, &train, &holdout))
        .collect();

    let mut policy_metrics = BTreeMap::new();
    policy_metrics.insert("induced", induced);
    policy_metrics.insert("oracle_hidden_classes", oracle);
    policy_metrics.insert("permuted_feature_search", permuted);
    policy_metrics.insert("random_partition", random);
    policy_metrics.insert("undifferentiated", undifferentiated);

    let report = Report {
        experiment: "H4 latent distinction induction",
        seed: SEED,
        visible_charge_kind: "Custom(unresolved)",
        train_observations: train.len(),
        holdout_observations: holdout.len(),
        top_training_candidates,
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

fn induce_greedily(
    train: &[Observation],
    holdout: &[Observation],
    candidates: &[Candidate],
    inducer: &mut OntologyInducer,
) -> Vec<InducedConcept> {
    let mut promoted = Vec::new();
    let mut active = Vec::<ConceptPredicate>::new();

    while active.len() < MAX_CONCEPTS {
        let train_before = evaluate_predicates(train, train, &active).mean_discharge_efficiency;
        let holdout_before = evaluate_predicates(train, holdout, &active).mean_discharge_efficiency;
        let mut ranked = Vec::new();

        for candidate in candidates {
            if active.iter().any(|predicate| predicate == &candidate.predicate) {
                continue;
            }
            let mut trial = active.clone();
            trial.push(candidate.predicate.clone());
            let train_after = evaluate_predicates(train, train, &trial).mean_discharge_efficiency;
            ranked.push((train_after - train_before, candidate));
        }

        ranked.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    right
                        .1
                        .training_gain
                        .partial_cmp(&left.1.training_gain)
                        .unwrap_or(Ordering::Equal)
                })
        });

        let mut accepted = None;
        for (_, candidate) in ranked.into_iter().take(24) {
            let mut trial = active.clone();
            trial.push(candidate.predicate.clone());
            let holdout_after = evaluate_predicates(train, holdout, &trial).mean_discharge_efficiency;
            let holdout_gain = holdout_after - holdout_before;
            let training_support = matching_count(train, &candidate.predicate);
            let concept = inducer.propose(
                None,
                candidate.predicate.clone(),
                ConceptEvidence {
                    observations: training_support as u64,
                    positive_instances: membership_ids(holdout, &candidate.predicate, true),
                    negative_instances: membership_ids(holdout, &candidate.predicate, false),
                    holdout_gain,
                },
                ConceptUtility {
                    routing_gain: holdout_gain,
                    discharge_gain: holdout_gain,
                    ..ConceptUtility::default()
                },
            );
            if inducer.promote(concept.clone()) {
                accepted = Some(concept);
                break;
            }
        }

        let Some(concept) = accepted else {
            break;
        };
        active.push(concept.predicate.clone());
        promoted.push(concept);
        inducer.advance_generation();
    }

    promoted
}

fn generate_observations(seed: u64, per_class: usize, id_offset: u64) -> Vec<Observation> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut observations = Vec::with_capacity(per_class * HiddenClass::all().len());
    let mut next_id = id_offset;

    for hidden in HiddenClass::all() {
        for _ in 0..per_class {
            let (centers, base_outcomes, persistence) = match hidden {
                HiddenClass::Gap => ([0.88, 0.14, 0.18], [0.92, 0.10, 0.08], rng.gen_range(4..8)),
                HiddenClass::Contradiction => {
                    ([0.16, 0.87, 0.20], [0.09, 0.93, 0.09], rng.gen_range(1..5))
                }
                HiddenClass::Residual => {
                    ([0.18, 0.18, 0.89], [0.08, 0.10, 0.94], rng.gen_range(2..6))
                }
            };
            let residual = centers
                .into_iter()
                .map(|center| (center + rng.gen_range(-0.045f32..0.045f32)).clamp(0.0, 1.0))
                .collect();
            let outcomes = noisy_outcomes(&mut rng, base_outcomes);
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
        *value = (*value + rng.gen_range(-0.025f64..0.025f64)).clamp(0.0, 1.0);
    }
    output
}

fn propose_thresholds(observations: &[Observation]) -> Vec<Candidate> {
    let dimensions = observations
        .first()
        .map(|observation| observation.charge.residual.len())
        .unwrap_or(0);
    let parent = evaluate_parent(observations, observations).mean_discharge_efficiency;
    let mut candidates = Vec::new();

    for dimension in 0..dimensions {
        let mut values: Vec<f32> = observations
            .iter()
            .filter_map(|observation| observation.charge.residual.get(dimension).copied())
            .collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        values.dedup_by(|left, right| (*left - *right).abs() < 1e-6);

        for pair in values.windows(2) {
            let threshold = (pair[0] + pair[1]) * 0.5;
            for direction in [Direction::AtLeast, Direction::AtMost] {
                let predicate = ConceptPredicate::ResidualThreshold {
                    dimension,
                    threshold,
                    direction,
                };
                let support = matching_count(observations, &predicate);
                if support < MIN_SUPPORT || observations.len() - support < MIN_SUPPORT {
                    continue;
                }
                let gain = evaluate_predicates(observations, observations, &[predicate.clone()])
                    .mean_discharge_efficiency
                    - parent
                    - 0.002;
                candidates.push(Candidate {
                    predicate,
                    training_gain: gain,
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
    });

    let mut unique = Vec::new();
    let mut memberships = Vec::<Vec<u64>>::new();
    for candidate in candidates {
        let ids = membership_ids(observations, &candidate.predicate, true);
        if memberships.iter().any(|seen| seen == &ids) {
            continue;
        }
        memberships.push(ids);
        unique.push(candidate);
    }
    unique
}

fn evaluate_parent(train: &[Observation], observations: &[Observation]) -> Metrics {
    let resolver = best_resolver(train.iter());
    evaluate_with_selector(observations, |_| resolver)
}

fn evaluate_predicates(
    train: &[Observation],
    observations: &[Observation],
    predicates: &[ConceptPredicate],
) -> Metrics {
    let parent = best_resolver(train.iter());
    let leaders: Vec<usize> = predicates
        .iter()
        .map(|predicate| {
            let members: Vec<&Observation> = train
                .iter()
                .filter(|observation| predicate.matches(&observation.charge))
                .collect();
            if members.is_empty() {
                parent
            } else {
                best_resolver(members.into_iter())
            }
        })
        .collect();

    evaluate_with_selector(observations, |observation| {
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
        let members: Vec<&Observation> = train
            .iter()
            .filter(|observation| observation.hidden == hidden)
            .collect();
        leaders.insert(hidden, best_resolver(members.into_iter()));
    }
    evaluate_with_selector(holdout, |observation| leaders[&observation.hidden])
}

fn evaluate_random_partition(
    train: &[Observation],
    holdout: &[Observation],
    groups: usize,
    seed: u64,
) -> Metrics {
    let groups = groups.max(1);
    let mut rng = StdRng::seed_from_u64(seed);
    let train_groups: HashMap<u64, usize> = train
        .iter()
        .map(|observation| (observation.id, rng.gen_range(0..groups)))
        .collect();
    let holdout_groups: HashMap<u64, usize> = holdout
        .iter()
        .map(|observation| (observation.id, rng.gen_range(0..groups)))
        .collect();
    let parent = best_resolver(train.iter());
    let leaders: Vec<usize> = (0..groups)
        .map(|group| {
            let members: Vec<&Observation> = train
                .iter()
                .filter(|observation| train_groups[&observation.id] == group)
                .collect();
            if members.is_empty() {
                parent
            } else {
                best_resolver(members.into_iter())
            }
        })
        .collect();

    evaluate_with_selector(holdout, |observation| leaders[holdout_groups[&observation.id]])
}

fn evaluate_permuted_control(train: &[Observation], holdout: &[Observation], seed: u64) -> Metrics {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut permuted_train = train.to_vec();
    let mut permuted_holdout = holdout.to_vec();
    permute_visible_features(&mut permuted_train, &mut rng);
    permute_visible_features(&mut permuted_holdout, &mut rng);
    let candidates = propose_thresholds(&permuted_train);
    let mut inducer = OntologyInducer::new(PromotionCriteria {
        min_observations: MIN_SUPPORT as u64,
        min_holdout_gain: 0.02,
        min_total_utility_gain: 0.04,
    });
    let promoted = induce_greedily(
        &permuted_train,
        &permuted_holdout,
        &candidates,
        &mut inducer,
    );
    let predicates: Vec<ConceptPredicate> = promoted
        .iter()
        .map(|concept| concept.predicate.clone())
        .collect();
    evaluate_predicates(&permuted_train, &permuted_holdout, &predicates)
}

fn permute_visible_features(observations: &mut [Observation], rng: &mut StdRng) {
    let Some(first) = observations.first() else {
        return;
    };
    let dimensions = first.charge.residual.len();
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
    let mut persistence: Vec<u32> = observations
        .iter()
        .map(|observation| observation.charge.persistence)
        .collect();
    persistence.shuffle(rng);
    for (observation, value) in observations.iter_mut().zip(persistence) {
        observation.charge.persistence = value;
    }
}

fn best_resolver<'a>(observations: impl Iterator<Item = &'a Observation>) -> usize {
    let observations: Vec<&Observation> = observations.collect();
    (0..RESOLVERS.len())
        .max_by(|left, right| {
            mean_outcome(&observations, *left)
                .partial_cmp(&mean_outcome(&observations, *right))
                .unwrap_or(Ordering::Equal)
        })
        .unwrap_or(0)
}

fn mean_outcome(observations: &[&Observation], resolver: usize) -> f64 {
    if observations.is_empty() {
        return 0.0;
    }
    observations
        .iter()
        .map(|observation| observation.outcomes[resolver])
        .sum::<f64>()
        / observations.len() as f64
}

fn evaluate_with_selector(
    observations: &[Observation],
    selector: impl Fn(&Observation) -> usize,
) -> Metrics {
    let mut total_discharge = 0.0;
    let mut total_remaining = 0.0;
    let mut solved = 0usize;

    for observation in observations {
        let discharge = observation.outcomes[selector(observation)].clamp(0.0, 1.0);
        let remaining = (1.0 - discharge).max(0.0);
        total_discharge += discharge;
        total_remaining += remaining;
        if remaining <= 0.25 {
            solved += 1;
        }
    }

    let count = observations.len().max(1) as f64;
    Metrics {
        mean_discharge_efficiency: total_discharge / count,
        mean_remaining: total_remaining / count,
        solve_rate: solved as f64 / count,
    }
}

fn matching_count(observations: &[Observation], predicate: &ConceptPredicate) -> usize {
    observations
        .iter()
        .filter(|observation| predicate.matches(&observation.charge))
        .count()
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

fn concept_report(
    concept: &InducedConcept,
    train: &[Observation],
    holdout: &[Observation],
) -> ConceptReport {
    let training_support = matching_count(train, &concept.predicate);
    let members: Vec<&Observation> = holdout
        .iter()
        .filter(|observation| concept.predicate.matches(&observation.charge))
        .collect();
    let leader = if training_support == 0 {
        best_resolver(train.iter())
    } else {
        let training_members: Vec<&Observation> = train
            .iter()
            .filter(|observation| concept.predicate.matches(&observation.charge))
            .collect();
        best_resolver(training_members.into_iter())
    };
    let (dominant_hidden_class, dominant_fraction) = dominant_hidden(&members);

    ConceptReport {
        id: concept.id.as_u64(),
        predicate: format!("{:?}", concept.predicate),
        training_support,
        holdout_support: members.len(),
        holdout_gain: concept.evidence.holdout_gain,
        utility_gain: concept.utility.total_gain(),
        resolver_leader: RESOLVERS[leader].to_string(),
        dominant_hidden_class,
        dominant_fraction,
    }
}

fn dominant_hidden(observations: &[&Observation]) -> (HiddenClass, f64) {
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
