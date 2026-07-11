//! H-Infant residual revision usefulness probe.
//!
//! Diagnostic-only experiment. This does **not** emit CHARGE and does not touch
//! live runtime routing. It tests whether persistent calibrated residual
//! exceedance is a useful trigger for one bounded model revision in a hidden
//! regime-shift environment.
//!
//! All strategies receive the same episode, the same two-model bank, and the
//! same maximum budget of one revision. Only the trigger policy differs.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use star::developmental::{
    compare_numeric_transition, NamedVector, NumericStateObservation,
    NumericTransitionPrediction, ResidualCalibrationProfile,
    ResidualCalibrationScope,
};

const SEEDS: u64 = 128;
const CALIBRATION_SAMPLES: usize = 1_000;
const EPISODE_STEPS: usize = 200;
const SHIFT_MIN: usize = 70;
const SHIFT_MAX: usize = 130;
const NOISE_SIGMA: f64 = 0.05;
const CALIBRATION_QUANTILE: f64 = 0.99;
const PERSISTENCE_REQUIRED: u32 = 3;

// Frozen acceptance gates.
const MAX_PERSISTENT_FALSE_REVISION_RATE: f64 = 0.05;
const MAX_PERSISTENT_MEDIAN_DELAY: f64 = 3.0;
const MAX_PERSISTENT_TO_NO_REVISION_MSE_RATIO: f64 = 0.10;
const MIN_FALSE_REVISION_ADVANTAGE_OVER_SINGLE: f64 = 0.30;
const MAX_PERSISTENT_EXCESS_MSE_OVER_ORACLE: f64 = 10.0;

const STATE_SPACE: &str = "synthetic.delta.v1";
const ACTION_SPACE: &str = "synthetic.action.scalar.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Strategy {
    PersistentCalibrated,
    SingleExceedance,
    Scheduled,
    Random,
    NoRevision,
    OracleAfterFirstShiftedObservation,
}

impl Strategy {
    const ALL: [Strategy; 6] = [
        Strategy::PersistentCalibrated,
        Strategy::SingleExceedance,
        Strategy::Scheduled,
        Strategy::Random,
        Strategy::NoRevision,
        Strategy::OracleAfterFirstShiftedObservation,
    ];

    fn name(self) -> &'static str {
        match self {
            Strategy::PersistentCalibrated => "persistent_calibrated",
            Strategy::SingleExceedance => "single_exceedance",
            Strategy::Scheduled => "scheduled",
            Strategy::Random => "random",
            Strategy::NoRevision => "no_revision",
            Strategy::OracleAfterFirstShiftedObservation => {
                "oracle_after_first_shifted_observation"
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Episode {
    shift_step: usize,
    random_revision_step: usize,
    actions: Vec<f64>,
    noises: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct StrategyRun {
    cumulative_mse: f64,
    post_shift_mse: f64,
    revision_step: Option<usize>,
    false_revision: bool,
    detection_delay: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct SeedRun {
    seed: u64,
    threshold_mse: f64,
    shift_step: usize,
    strategies: std::collections::BTreeMap<String, StrategyRun>,
}

#[derive(Debug, Clone, Serialize)]
struct StrategyAggregate {
    mean_cumulative_mse: f64,
    median_cumulative_mse: f64,
    mean_post_shift_mse: f64,
    false_revision_rate: f64,
    no_revision_rate: f64,
    median_detection_delay: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ProbeOutput {
    experiment: &'static str,
    status: &'static str,
    protocol: Protocol,
    predeclared_gates: Gates,
    aggregate: std::collections::BTreeMap<String, StrategyAggregate>,
    derived_checks: DerivedChecks,
    runs: Vec<SeedRun>,
    interpretation_constraints: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct Protocol {
    seeds: u64,
    calibration_samples_per_seed: usize,
    episode_steps: usize,
    shift_step_range_inclusive: [usize; 2],
    noise_sigma: f64,
    calibration_quantile: f64,
    persistence_required: u32,
    revision_budget_per_strategy: u32,
    model_bank: [&'static str; 2],
}

#[derive(Debug, Serialize)]
struct Gates {
    persistent_false_revision_rate_lte: f64,
    persistent_median_detection_delay_lte: f64,
    persistent_to_no_revision_mean_mse_ratio_lte: f64,
    single_minus_persistent_false_revision_rate_gte: f64,
    persistent_excess_mean_mse_over_oracle_lte: f64,
    persistent_mean_mse_must_beat_non_oracle_controls: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct DerivedChecks {
    persistent_false_revision_rate: f64,
    persistent_median_detection_delay: f64,
    persistent_to_no_revision_mean_mse_ratio: f64,
    single_minus_persistent_false_revision_rate: f64,
    persistent_excess_mean_mse_over_oracle: f64,
    persistent_beats_all_non_oracle_controls: bool,
    all_gates_pass: bool,
}

fn calibration_scope(seed: u64) -> ResidualCalibrationScope {
    ResidualCalibrationScope {
        predictor_scope: format!("regime-a-model/seed-{seed}/v1"),
        environment_scope: "hidden-regime-shift/v1".to_string(),
        state_space: STATE_SPACE.to_string(),
        horizon_steps: 1,
    }
}

fn residual(
    transition_id: String,
    predicted_delta: f64,
    observed_delta: f64,
) -> star::developmental::NumericPredictionResidual {
    let prediction = NumericTransitionPrediction {
        transition_id: transition_id.clone(),
        action: NamedVector {
            space: ACTION_SPACE.to_string(),
            values: vec![0.0],
        },
        predicted_next_state: NamedVector {
            space: STATE_SPACE.to_string(),
            values: vec![predicted_delta as f32],
        },
        horizon_steps: 1,
    };
    let observation = NumericStateObservation {
        transition_id,
        state: NamedVector {
            space: STATE_SPACE.to_string(),
            values: vec![observed_delta as f32],
        },
    };

    compare_numeric_transition(&prediction, &observation)
        .expect("probe residual vectors must be comparable")
}

fn fit_profile(seed: u64) -> ResidualCalibrationProfile {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xA11C_E5E5_DA7A_0001);
    let noise = Normal::new(0.0, NOISE_SIGMA).expect("valid normal distribution");
    let mut residuals = Vec::with_capacity(CALIBRATION_SAMPLES);

    for index in 0..CALIBRATION_SAMPLES {
        let action = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let predicted = action;
        let observed = action + noise.sample(&mut rng);
        residuals.push(residual(
            format!("calibration-{seed}-{index}"),
            predicted,
            observed,
        ));
    }

    ResidualCalibrationProfile::fit_higher_quantile(
        calibration_scope(seed),
        &residuals,
        CALIBRATION_QUANTILE,
    )
    .expect("calibration profile must fit")
}

fn generate_episode(seed: u64) -> Episode {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xE915_0DE5_5EED_0002);
    let noise = Normal::new(0.0, NOISE_SIGMA).expect("valid normal distribution");
    let shift_step = rng.gen_range(SHIFT_MIN..=SHIFT_MAX);
    let random_revision_step = rng.gen_range(0..EPISODE_STEPS);

    let mut actions = Vec::with_capacity(EPISODE_STEPS);
    let mut noises = Vec::with_capacity(EPISODE_STEPS);
    for _ in 0..EPISODE_STEPS {
        actions.push(if rng.gen_bool(0.5) { 1.0 } else { -1.0 });
        noises.push(noise.sample(&mut rng));
    }

    Episode {
        shift_step,
        random_revision_step,
        actions,
        noises,
    }
}

fn run_strategy(
    seed: u64,
    strategy: Strategy,
    profile: &ResidualCalibrationProfile,
    episode: &Episode,
) -> StrategyRun {
    let scope = calibration_scope(seed);
    let mut active_model_scale = 1.0_f64; // model A
    let revised_model_scale = -1.0_f64; // model B
    let mut revision_step = None;
    let mut consecutive_exceedances = 0_u32;
    let mut cumulative_mse = 0.0_f64;
    let mut post_shift_mse = 0.0_f64;

    for step in 0..EPISODE_STEPS {
        let action = episode.actions[step];
        let true_scale = if step < episode.shift_step { 1.0 } else { -1.0 };
        let observed_delta = true_scale * action + episode.noises[step];
        let predicted_delta = active_model_scale * action;
        let current_residual = residual(
            format!("episode-{seed}-{step}-{}", strategy.name()),
            predicted_delta,
            observed_delta,
        );

        cumulative_mse += current_residual.mean_squared_error;
        if step >= episode.shift_step {
            post_shift_mse += current_residual.mean_squared_error;
        }

        if revision_step.is_some() {
            continue;
        }

        let assessment = profile
            .assess(&scope, &current_residual)
            .expect("episode residual must match calibration scope");

        let trigger = match strategy {
            Strategy::PersistentCalibrated => {
                if assessment.exceeded {
                    consecutive_exceedances = consecutive_exceedances.saturating_add(1);
                } else {
                    consecutive_exceedances = 0;
                }
                consecutive_exceedances >= PERSISTENCE_REQUIRED
            }
            Strategy::SingleExceedance => assessment.exceeded,
            Strategy::Scheduled => step == 100,
            Strategy::Random => step == episode.random_revision_step,
            Strategy::NoRevision => false,
            Strategy::OracleAfterFirstShiftedObservation => {
                step == episode.shift_step
            }
        };

        if trigger {
            revision_step = Some(step);
            active_model_scale = revised_model_scale;
        }
    }

    let false_revision = revision_step
        .map(|step| step < episode.shift_step)
        .unwrap_or(false);
    let detection_delay = revision_step.and_then(|step| {
        if step >= episode.shift_step {
            Some(step - episode.shift_step)
        } else {
            None
        }
    });

    StrategyRun {
        cumulative_mse,
        post_shift_mse,
        revision_step,
        false_revision,
        detection_delay,
    }
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| {
        left.partial_cmp(right)
            .expect("finite probe metrics must be comparable")
    });
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
}

fn aggregate(
    strategy: Strategy,
    runs: &[SeedRun],
) -> StrategyAggregate {
    let values: Vec<&StrategyRun> = runs
        .iter()
        .map(|run| {
            run.strategies
                .get(strategy.name())
                .expect("all runs must contain all strategies")
        })
        .collect();

    let count = values.len() as f64;
    let cumulative: Vec<f64> = values.iter().map(|run| run.cumulative_mse).collect();
    let detection_delays: Vec<f64> = values
        .iter()
        .filter_map(|run| run.detection_delay.map(|delay| delay as f64))
        .collect();

    StrategyAggregate {
        mean_cumulative_mse: cumulative.iter().sum::<f64>() / count,
        median_cumulative_mse: median(cumulative).expect("non-empty strategy runs"),
        mean_post_shift_mse: values
            .iter()
            .map(|run| run.post_shift_mse)
            .sum::<f64>()
            / count,
        false_revision_rate: values
            .iter()
            .filter(|run| run.false_revision)
            .count() as f64
            / count,
        no_revision_rate: values
            .iter()
            .filter(|run| run.revision_step.is_none())
            .count() as f64
            / count,
        median_detection_delay: median(detection_delays),
    }
}

fn main() {
    let mut runs = Vec::with_capacity(SEEDS as usize);

    for seed in 0..SEEDS {
        let profile = fit_profile(seed);
        let episode = generate_episode(seed);
        let mut strategies = std::collections::BTreeMap::new();

        for strategy in Strategy::ALL {
            strategies.insert(
                strategy.name().to_string(),
                run_strategy(seed, strategy, &profile, &episode),
            );
        }

        runs.push(SeedRun {
            seed,
            threshold_mse: profile.threshold,
            shift_step: episode.shift_step,
            strategies,
        });
    }

    let mut aggregate_by_strategy = std::collections::BTreeMap::new();
    for strategy in Strategy::ALL {
        aggregate_by_strategy.insert(
            strategy.name().to_string(),
            aggregate(strategy, &runs),
        );
    }

    let persistent = &aggregate_by_strategy["persistent_calibrated"];
    let single = &aggregate_by_strategy["single_exceedance"];
    let scheduled = &aggregate_by_strategy["scheduled"];
    let random = &aggregate_by_strategy["random"];
    let no_revision = &aggregate_by_strategy["no_revision"];
    let oracle = &aggregate_by_strategy["oracle_after_first_shifted_observation"];

    let persistent_median_delay = persistent
        .median_detection_delay
        .unwrap_or(f64::INFINITY);
    let persistent_to_no_revision =
        persistent.mean_cumulative_mse / no_revision.mean_cumulative_mse;
    let false_revision_advantage =
        single.false_revision_rate - persistent.false_revision_rate;
    let excess_over_oracle =
        persistent.mean_cumulative_mse - oracle.mean_cumulative_mse;
    let beats_all_non_oracle_controls = [single, scheduled, random, no_revision]
        .iter()
        .all(|control| {
            persistent.mean_cumulative_mse < control.mean_cumulative_mse
        });

    let all_gates_pass = persistent.false_revision_rate
        <= MAX_PERSISTENT_FALSE_REVISION_RATE
        && persistent.no_revision_rate == 0.0
        && persistent_median_delay <= MAX_PERSISTENT_MEDIAN_DELAY
        && persistent_to_no_revision
            <= MAX_PERSISTENT_TO_NO_REVISION_MSE_RATIO
        && false_revision_advantage
            >= MIN_FALSE_REVISION_ADVANTAGE_OVER_SINGLE
        && excess_over_oracle <= MAX_PERSISTENT_EXCESS_MSE_OVER_ORACLE
        && beats_all_non_oracle_controls;

    let output = ProbeOutput {
        experiment: "h_infant_residual_revision_probe",
        status: if all_gates_pass { "pass" } else { "fail" },
        protocol: Protocol {
            seeds: SEEDS,
            calibration_samples_per_seed: CALIBRATION_SAMPLES,
            episode_steps: EPISODE_STEPS,
            shift_step_range_inclusive: [SHIFT_MIN, SHIFT_MAX],
            noise_sigma: NOISE_SIGMA,
            calibration_quantile: CALIBRATION_QUANTILE,
            persistence_required: PERSISTENCE_REQUIRED,
            revision_budget_per_strategy: 1,
            model_bank: ["regime_a_scale_+1", "regime_b_scale_-1"],
        },
        predeclared_gates: Gates {
            persistent_false_revision_rate_lte:
                MAX_PERSISTENT_FALSE_REVISION_RATE,
            persistent_median_detection_delay_lte:
                MAX_PERSISTENT_MEDIAN_DELAY,
            persistent_to_no_revision_mean_mse_ratio_lte:
                MAX_PERSISTENT_TO_NO_REVISION_MSE_RATIO,
            single_minus_persistent_false_revision_rate_gte:
                MIN_FALSE_REVISION_ADVANTAGE_OVER_SINGLE,
            persistent_excess_mean_mse_over_oracle_lte:
                MAX_PERSISTENT_EXCESS_MSE_OVER_ORACLE,
            persistent_mean_mse_must_beat_non_oracle_controls: vec![
                "single_exceedance",
                "scheduled",
                "random",
                "no_revision",
            ],
        },
        aggregate: aggregate_by_strategy,
        derived_checks: DerivedChecks {
            persistent_false_revision_rate: persistent.false_revision_rate,
            persistent_median_detection_delay: persistent_median_delay,
            persistent_to_no_revision_mean_mse_ratio:
                persistent_to_no_revision,
            single_minus_persistent_false_revision_rate:
                false_revision_advantage,
            persistent_excess_mean_mse_over_oracle: excess_over_oracle,
            persistent_beats_all_non_oracle_controls:
                beats_all_non_oracle_controls,
            all_gates_pass,
        },
        runs,
        interpretation_constraints: vec![
            "This is a deterministic diagnostic environment, not live runtime integration.",
            "The model bank is supplied to every strategy; only the one-revision trigger differs.",
            "Persistent calibrated exceedance is tested as a model-revision trigger, not as CHARGE.",
            "Passing does not justify a universal threshold or autonomous ontology promotion.",
            "A later experiment must compare a typed pressure candidate against matched non-pressure routing in a closed cognitive cycle.",
        ],
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("probe output must serialize")
    );

    if !all_gates_pass {
        std::process::exit(1);
    }
}
