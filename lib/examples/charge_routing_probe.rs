use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;

use star::charge::{
    Charge, ChargeKind, ChargeLedger, ChargeScope, ChargeSignature, Resolution, ResolverStats,
};

const N_SIGNATURES: usize = 4;
const N_RESOLVERS: usize = 6;
const TRAINING_SAMPLES: usize = 24;
const EPISODES_PER_SIGNATURE: usize = 80;
const COMPUTE_BUDGET: u64 = 12;
const STOP_MAGNITUDE: f32 = 0.05;
const MAX_ATTEMPTS: usize = 20;
const NOISE_STD: f64 = 0.07;

#[derive(Debug, Clone)]
struct Environment {
    means: [[f64; N_RESOLVERS]; N_SIGNATURES],
    costs: [u64; N_RESOLVERS],
}

#[derive(Debug, Clone, Copy, Serialize)]
struct StrategyMetrics {
    discharge_per_compute: f64,
    mean_remaining: f64,
    solve_rate: f64,
    mean_attempts: f64,
    max_conservation_error: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Policy {
    Learned,
    Random,
    Fixed,
    Oracle,
    Scrambled,
}

impl Policy {
    fn name(self) -> &'static str {
        match self {
            Self::Learned => "learned",
            Self::Random => "random",
            Self::Fixed => "fixed",
            Self::Oracle => "oracle",
            Self::Scrambled => "scrambled",
        }
    }
}

const POLICIES: [Policy; 5] = [
    Policy::Learned,
    Policy::Random,
    Policy::Fixed,
    Policy::Oracle,
    Policy::Scrambled,
];

#[derive(Debug, Serialize)]
struct Config {
    trials: usize,
    signatures: usize,
    resolvers: usize,
    training_samples_per_pair: usize,
    episodes_per_signature: usize,
    compute_budget: u64,
    stop_magnitude: f32,
    noise_std: f64,
}

#[derive(Debug, Serialize)]
struct Diagnostics {
    efficiency_vs_random: f64,
    efficiency_vs_fixed: f64,
    remaining_ratio_vs_fixed: f64,
    solve_rate_margin_vs_fixed: f64,
    oracle_efficiency_retention: f64,
    signature_information_gain: f64,
    specialization_agreement: f64,
    max_conservation_error: f64,
}

#[derive(Debug, Serialize)]
struct Criteria {
    beats_random_efficiency_1_5x: bool,
    beats_fixed_efficiency_1_2x: bool,
    halves_fixed_remaining_charge: bool,
    improves_fixed_solve_rate_20pp: bool,
    retains_90pct_oracle_efficiency: bool,
    signature_control_matters_1_3x: bool,
    discovers_oracle_specialist_80pct: bool,
    conserves_charge: bool,
}

impl Criteria {
    fn passed(&self) -> bool {
        self.beats_random_efficiency_1_5x
            && self.beats_fixed_efficiency_1_2x
            && self.halves_fixed_remaining_charge
            && self.improves_fixed_solve_rate_20pp
            && self.retains_90pct_oracle_efficiency
            && self.signature_control_matters_1_3x
            && self.discovers_oracle_specialist_80pct
            && self.conserves_charge
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    config: Config,
    strategies: BTreeMap<String, StrategyMetrics>,
    diagnostics: Diagnostics,
    criteria: Criteria,
    passed: bool,
}

fn signatures() -> [ChargeSignature; N_SIGNATURES] {
    [
        ChargeSignature {
            kind: ChargeKind::Contradiction,
            scope: ChargeScope::Component("benchmark:contradiction".into()),
        },
        ChargeSignature {
            kind: ChargeKind::EpistemicGap,
            scope: ChargeScope::Component("benchmark:epistemic".into()),
        },
        ChargeSignature {
            kind: ChargeKind::CausalAmbiguity,
            scope: ChargeScope::Component("benchmark:causal".into()),
        },
        ChargeSignature {
            kind: ChargeKind::TemporalMismatch,
            scope: ChargeScope::Component("benchmark:temporal".into()),
        },
    ]
}

fn charge(signature: &ChargeSignature) -> Charge {
    Charge::new(
        signature.kind.clone(),
        vec![1.0],
        1.0,
        signature.scope.clone(),
    )
}

fn mix_seed(parts: &[u64]) -> u64 {
    let mut value = 0x9E37_79B9_7F4A_7C15u64;
    for part in parts {
        value ^= part
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(value << 6)
            .wrapping_add(value >> 2);
    }
    value
}

fn bounded_fraction(mean: f64, seed_parts: &[u64]) -> f64 {
    let mut rng = StdRng::seed_from_u64(mix_seed(seed_parts));
    let noise = Normal::new(0.0, NOISE_STD).unwrap().sample(&mut rng);
    (mean + noise).clamp(0.02, 0.95)
}

fn build_environment(seed: u64) -> Environment {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut costs = [0u64; N_RESOLVERS];
    for cost in &mut costs {
        *cost = rng.gen_range(1..=5);
    }

    let mut resolver_indices: Vec<usize> = (0..N_RESOLVERS).collect();
    resolver_indices.shuffle(&mut rng);
    let specialists = &resolver_indices[..N_SIGNATURES];
    let mut means = [[0.0f64; N_RESOLVERS]; N_SIGNATURES];

    for signature_index in 0..N_SIGNATURES {
        for resolver_index in 0..N_RESOLVERS {
            means[signature_index][resolver_index] = rng.gen_range(0.08..0.28);
        }

        let specialist = specialists[signature_index];
        means[signature_index][specialist] = rng.gen_range(0.68..0.90);
        let second_candidates: Vec<usize> = (0..N_RESOLVERS)
            .filter(|resolver_index| *resolver_index != specialist)
            .collect();
        let second_best = *second_candidates.choose(&mut rng).unwrap();
        means[signature_index][second_best] =
            means[signature_index][second_best].max(rng.gen_range(0.35..0.52));
    }

    Environment { means, costs }
}

fn profile_best(
    profiles: &[ResolverStats],
    signature: &ChargeSignature,
    candidates: &[usize],
) -> Option<usize> {
    candidates.iter().copied().max_by(|left, right| {
        let left_score = profiles[*left].efficiency_for(signature).unwrap_or(0.0);
        let right_score = profiles[*right].efficiency_for(signature).unwrap_or(0.0);
        left_score
            .total_cmp(&right_score)
            .then_with(|| left.cmp(right))
    })
}

fn oracle_best(
    environment: &Environment,
    signature_index: usize,
    candidates: &[usize],
) -> Option<usize> {
    candidates.iter().copied().max_by(|left, right| {
        let left_score =
            environment.means[signature_index][*left] / environment.costs[*left] as f64;
        let right_score =
            environment.means[signature_index][*right] / environment.costs[*right] as f64;
        left_score
            .total_cmp(&right_score)
            .then_with(|| left.cmp(right))
    })
}

fn train_profiles(
    environment: &Environment,
    signatures: &[ChargeSignature; N_SIGNATURES],
    seed: u64,
) -> (Vec<ResolverStats>, usize) {
    let mut profiles = vec![ResolverStats::default(); N_RESOLVERS];
    let mut global_discharged = [0.0f64; N_RESOLVERS];
    let mut global_compute = [0u64; N_RESOLVERS];

    for signature_index in 0..N_SIGNATURES {
        for resolver_index in 0..N_RESOLVERS {
            for sample in 0..TRAINING_SAMPLES {
                let fraction = bounded_fraction(
                    environment.means[signature_index][resolver_index],
                    &[
                        seed,
                        1,
                        signature_index as u64,
                        resolver_index as u64,
                        sample as u64,
                    ],
                );
                let resolution = Resolution {
                    discharged: fraction as f32,
                    emitted: Vec::new(),
                    permitted_decay: 0.0,
                    compute_cost: environment.costs[resolver_index],
                };
                profiles[resolver_index].observe(&charge(&signatures[signature_index]), &resolution);
                global_discharged[resolver_index] += fraction;
                global_compute[resolver_index] += environment.costs[resolver_index];
            }
        }
    }

    let fixed_resolver = (0..N_RESOLVERS)
        .max_by(|left, right| {
            let left_score = global_discharged[*left] / global_compute[*left] as f64;
            let right_score = global_discharged[*right] / global_compute[*right] as f64;
            left_score
                .total_cmp(&right_score)
                .then_with(|| left.cmp(right))
        })
        .unwrap();
    (profiles, fixed_resolver)
}

#[allow(clippy::too_many_arguments)]
fn select_resolver(
    policy: Policy,
    environment: &Environment,
    profiles: &[ResolverStats],
    fixed_resolver: usize,
    signatures: &[ChargeSignature; N_SIGNATURES],
    signature_index: usize,
    candidates: &[usize],
    seed: u64,
    episode: usize,
    attempt: usize,
) -> Option<usize> {
    match policy {
        Policy::Learned => profile_best(profiles, &signatures[signature_index], candidates),
        Policy::Oracle => oracle_best(environment, signature_index, candidates),
        Policy::Fixed => candidates.contains(&fixed_resolver).then_some(fixed_resolver),
        Policy::Scrambled => {
            let wrong_index = (signature_index + 1) % N_SIGNATURES;
            profile_best(profiles, &signatures[wrong_index], candidates)
        }
        Policy::Random => {
            let mut rng = StdRng::seed_from_u64(mix_seed(&[
                seed,
                3,
                signature_index as u64,
                episode as u64,
                attempt as u64,
            ]));
            candidates.choose(&mut rng).copied()
        }
    }
}

fn evaluate_policy(
    policy: Policy,
    environment: &Environment,
    profiles: &[ResolverStats],
    fixed_resolver: usize,
    signatures: &[ChargeSignature; N_SIGNATURES],
    seed: u64,
) -> Result<StrategyMetrics, Box<dyn Error>> {
    let mut ledger = ChargeLedger::default();
    let mut total_discharged = 0.0f64;
    let mut total_compute = 0u64;
    let mut total_attempts = 0usize;
    let mut remaining_values = Vec::new();
    let mut solved = 0usize;
    let mut max_conservation_error = 0.0f64;

    for signature_index in 0..N_SIGNATURES {
        for episode in 0..EPISODES_PER_SIGNATURE {
            let parent = ledger.issue(charge(&signatures[signature_index]))?;
            let mut spent = 0u64;
            let mut attempt = 0usize;
            let mut episode_discharged = 0.0f64;

            while ledger.remaining(parent.id).unwrap() > STOP_MAGNITUDE {
                if attempt >= MAX_ATTEMPTS {
                    break;
                }
                let remaining_budget = COMPUTE_BUDGET.saturating_sub(spent);
                let candidates: Vec<usize> = (0..N_RESOLVERS)
                    .filter(|resolver_index| environment.costs[*resolver_index] <= remaining_budget)
                    .collect();
                let resolver_index = match select_resolver(
                    policy,
                    environment,
                    profiles,
                    fixed_resolver,
                    signatures,
                    signature_index,
                    &candidates,
                    seed,
                    episode,
                    attempt,
                ) {
                    Some(index) => index,
                    None => break,
                };

                let incoming = ledger.remaining(parent.id).unwrap();
                let fraction = bounded_fraction(
                    environment.means[signature_index][resolver_index],
                    &[
                        seed,
                        2,
                        signature_index as u64,
                        episode as u64,
                        attempt as u64,
                        resolver_index as u64,
                    ],
                );
                let discharged = incoming * fraction as f32;
                let cost = environment.costs[resolver_index];
                let (receipt, _) = ledger.record_resolution(
                    parent.id,
                    Resolution {
                        discharged,
                        emitted: Vec::new(),
                        permitted_decay: 0.0,
                        compute_cost: cost,
                    },
                )?;

                spent += cost;
                attempt += 1;
                total_attempts += 1;
                total_compute += cost;
                total_discharged += discharged as f64;
                episode_discharged += discharged as f64;
                if receipt.remaining > receipt.incoming + 1e-5 {
                    return Err("remaining charge increased after a resolver attempt".into());
                }
            }

            let remaining = ledger.remaining(parent.id).unwrap();
            remaining_values.push(remaining as f64);
            if remaining <= STOP_MAGNITUDE {
                solved += 1;
            }
            let conservation_error = (1.0 - episode_discharged - remaining as f64).abs();
            max_conservation_error = max_conservation_error.max(conservation_error);
        }
    }

    let episodes = (N_SIGNATURES * EPISODES_PER_SIGNATURE) as f64;
    Ok(StrategyMetrics {
        discharge_per_compute: total_discharged / total_compute as f64,
        mean_remaining: remaining_values.iter().sum::<f64>() / remaining_values.len() as f64,
        solve_rate: solved as f64 / episodes,
        mean_attempts: total_attempts as f64 / episodes,
        max_conservation_error,
    })
}

fn mean_metrics(values: &[StrategyMetrics]) -> StrategyMetrics {
    let count = values.len() as f64;
    StrategyMetrics {
        discharge_per_compute: values
            .iter()
            .map(|value| value.discharge_per_compute)
            .sum::<f64>()
            / count,
        mean_remaining: values.iter().map(|value| value.mean_remaining).sum::<f64>() / count,
        solve_rate: values.iter().map(|value| value.solve_rate).sum::<f64>() / count,
        mean_attempts: values.iter().map(|value| value.mean_attempts).sum::<f64>() / count,
        max_conservation_error: values
            .iter()
            .map(|value| value.max_conservation_error)
            .fold(0.0, f64::max),
    }
}

fn run_benchmark(trials: usize) -> Result<Report, Box<dyn Error>> {
    let signatures = signatures();
    let mut by_policy: BTreeMap<Policy, Vec<StrategyMetrics>> = POLICIES
        .into_iter()
        .map(|policy| (policy, Vec::new()))
        .collect();
    let mut specialization_agreement = Vec::new();

    for trial in 0..trials {
        let environment = build_environment(1_000 + trial as u64);
        let (profiles, fixed_resolver) =
            train_profiles(&environment, &signatures, 5_000 + trial as u64);

        let all_candidates: Vec<usize> = (0..N_RESOLVERS).collect();
        let correct = (0..N_SIGNATURES)
            .filter(|signature_index| {
                profile_best(
                    &profiles,
                    &signatures[*signature_index],
                    &all_candidates,
                ) == oracle_best(&environment, *signature_index, &all_candidates)
            })
            .count();
        specialization_agreement.push(correct as f64 / N_SIGNATURES as f64);

        for policy in POLICIES {
            let metrics = evaluate_policy(
                policy,
                &environment,
                &profiles,
                fixed_resolver,
                &signatures,
                9_000 + trial as u64,
            )?;
            by_policy.get_mut(&policy).unwrap().push(metrics);
        }
    }

    let mut strategies = BTreeMap::new();
    for policy in POLICIES {
        strategies.insert(
            policy.name().to_string(),
            mean_metrics(by_policy.get(&policy).unwrap()),
        );
    }

    let learned = strategies["learned"];
    let random = strategies["random"];
    let fixed = strategies["fixed"];
    let oracle = strategies["oracle"];
    let scrambled = strategies["scrambled"];
    let specialization_agreement =
        specialization_agreement.iter().sum::<f64>() / specialization_agreement.len() as f64;

    let diagnostics = Diagnostics {
        efficiency_vs_random: learned.discharge_per_compute / random.discharge_per_compute,
        efficiency_vs_fixed: learned.discharge_per_compute / fixed.discharge_per_compute,
        remaining_ratio_vs_fixed: learned.mean_remaining / fixed.mean_remaining,
        solve_rate_margin_vs_fixed: learned.solve_rate - fixed.solve_rate,
        oracle_efficiency_retention: learned.discharge_per_compute / oracle.discharge_per_compute,
        signature_information_gain: learned.discharge_per_compute / scrambled.discharge_per_compute,
        specialization_agreement,
        max_conservation_error: strategies
            .values()
            .map(|metrics| metrics.max_conservation_error)
            .fold(0.0, f64::max),
    };

    let criteria = Criteria {
        beats_random_efficiency_1_5x: diagnostics.efficiency_vs_random >= 1.5,
        beats_fixed_efficiency_1_2x: diagnostics.efficiency_vs_fixed >= 1.2,
        halves_fixed_remaining_charge: diagnostics.remaining_ratio_vs_fixed <= 0.5,
        improves_fixed_solve_rate_20pp: diagnostics.solve_rate_margin_vs_fixed >= 0.2,
        retains_90pct_oracle_efficiency: diagnostics.oracle_efficiency_retention >= 0.9,
        signature_control_matters_1_3x: diagnostics.signature_information_gain >= 1.3,
        discovers_oracle_specialist_80pct: diagnostics.specialization_agreement >= 0.8,
        conserves_charge: diagnostics.max_conservation_error <= 1e-5,
    };
    let passed = criteria.passed();

    Ok(Report {
        experiment: "charge-routing-falsification-rust-v1",
        config: Config {
            trials,
            signatures: N_SIGNATURES,
            resolvers: N_RESOLVERS,
            training_samples_per_pair: TRAINING_SAMPLES,
            episodes_per_signature: EPISODES_PER_SIGNATURE,
            compute_budget: COMPUTE_BUDGET,
            stop_magnitude: STOP_MAGNITUDE,
            noise_std: NOISE_STD,
        },
        strategies,
        diagnostics,
        criteria,
        passed,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let report = run_benchmark(32)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.passed {
        std::process::exit(1);
    }
    Ok(())
}
