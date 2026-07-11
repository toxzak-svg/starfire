//! H-Infant typed residual-pressure routing probe.
//!
//! Diagnostic-only. Structured developmental prediction residuals are placed in
//! `ChargeKind::PredictionResidual`, routed to one of two model-repair operators,
//! and accepted only after an independent held-out witness discharges the charge.
//! No live runtime routing is modified.

use std::collections::{BTreeMap, VecDeque};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use star::charge::{
    Charge, ChargeKind, ChargeScope, DischargeJudge, ImprovementDirection,
    OutcomeWitness, RelativeImprovementJudge, Resolution,
};
use star::cognitive_cycle::{ChargeDisposition, CognitiveCycleState};
use star::developmental::{
    compare_numeric_transition, NamedVector, NumericStateObservation,
    NumericTransitionPrediction, ResidualCalibrationProfile,
    ResidualCalibrationScope,
};

const SEEDS: u64 = 256;
const CALIBRATION_SAMPLES: usize = 1_000;
const EPISODE_STEPS: usize = 180;
const SHIFT_MIN: usize = 60;
const SHIFT_MAX: usize = 120;
const NOISE_SIGMA: f64 = 0.20;
const CALIBRATION_QUANTILE: f64 = 0.99;
const PERSISTENCE_REQUIRED: usize = 4;
const WITNESS_SAMPLES: usize = 32;
const VERIFIER_MAX_ERROR_RATIO: f64 = 0.25;
const STATE_SPACE: &str = "synthetic.affine_delta.v1";
const ACTION_SPACE: &str = "synthetic.action.scalar.v1";

// Frozen acceptance gates.
const MIN_TYPED_RESOLUTION_RATE: f64 = 0.95;
const MAX_TYPED_FALSE_COMMIT_RATE: f64 = 0.02;
const MAX_TYPED_MEDIAN_DELAY: f64 = 4.0;
const MAX_TYPED_TO_STRUCTURED_DIRECT_MSE_RATIO: f64 = 1.05;
const MAX_TYPED_TO_UNTYPED_MSE_RATIO: f64 = 0.25;
const MAX_TYPED_TO_RANDOM_MSE_RATIO: f64 = 0.30;
const MIN_TYPED_ROUTE_ADVANTAGE_OVER_UNTYPED: f64 = 0.45;
const MAX_TYPED_EXCESS_MSE_OVER_ORACLE: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ShiftFamily {
    Scale,
    Bias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ResolverKind {
    ReviseScale,
    ReviseBias,
}

impl ResolverKind {
    fn name(self) -> &'static str {
        match self {
            Self::ReviseScale => "revise_scale",
            Self::ReviseBias => "revise_bias",
        }
    }

    fn matches(self, family: ShiftFamily) -> bool {
        matches!(
            (self, family),
            (Self::ReviseScale, ShiftFamily::Scale)
                | (Self::ReviseBias, ShiftFamily::Bias)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Strategy {
    TypedPressure,
    StructuredDirectNoPressure,
    UntypedMatchedTrigger,
    RandomMatchedTrigger,
    ScrambledPressure,
    NoRevision,
    Oracle,
}

impl Strategy {
    const ALL: [Self; 7] = [
        Self::TypedPressure,
        Self::StructuredDirectNoPressure,
        Self::UntypedMatchedTrigger,
        Self::RandomMatchedTrigger,
        Self::ScrambledPressure,
        Self::NoRevision,
        Self::Oracle,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::TypedPressure => "typed_pressure",
            Self::StructuredDirectNoPressure => "structured_direct_no_pressure",
            Self::UntypedMatchedTrigger => "untyped_matched_trigger",
            Self::RandomMatchedTrigger => "random_matched_trigger",
            Self::ScrambledPressure => "scrambled_pressure",
            Self::NoRevision => "no_revision",
            Self::Oracle => "oracle",
        }
    }

    fn uses_charge(self) -> bool {
        matches!(self, Self::TypedPressure | Self::ScrambledPressure)
    }
}

#[derive(Debug, Clone, Copy)]
struct Model {
    scale: f64,
    bias: f64,
}

impl Model {
    const BASELINE: Self = Self { scale: 1.0, bias: 0.0 };

    fn predict(self, action: f64) -> f64 {
        self.scale * action + self.bias
    }
}

#[derive(Debug, Clone, Copy)]
struct Features {
    intercept: f64,
    slope: f64,
}

#[derive(Debug, Clone)]
struct Episode {
    family: ShiftFamily,
    shift_step: usize,
    post_shift_model: Model,
    actions: Vec<f64>,
    noises: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct Witness {
    before_mse: f64,
    after_mse: f64,
    verified: bool,
}

#[derive(Debug, Clone, Serialize)]
struct StrategyRun {
    cumulative_mse: f64,
    post_shift_mse: f64,
    attempted: bool,
    committed: bool,
    false_commit: bool,
    route_correct: bool,
    selected_resolver: Option<ResolverKind>,
    trigger_step: Option<usize>,
    detection_delay: Option<usize>,
    witness_before_mse: Option<f64>,
    witness_after_mse: Option<f64>,
    accepted_discharge: f64,
    unresolved_charge_at_end: bool,
    unverified_commit: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SeedRun {
    seed: u64,
    family: ShiftFamily,
    shift_step: usize,
    post_shift_scale: f64,
    post_shift_bias: f64,
    threshold_mse: f64,
    strategies: BTreeMap<String, StrategyRun>,
}

#[derive(Debug, Clone, Serialize)]
struct Aggregate {
    mean_cumulative_mse: f64,
    median_cumulative_mse: f64,
    mean_post_shift_mse: f64,
    attempt_rate: f64,
    commit_rate: f64,
    false_commit_rate: f64,
    route_correct_rate: f64,
    unresolved_charge_rate: f64,
    mean_accepted_discharge: f64,
    unverified_commit_rate: f64,
    median_detection_delay: Option<f64>,
}

#[derive(Debug, Serialize)]
struct Output {
    experiment: &'static str,
    status: &'static str,
    protocol: Protocol,
    predeclared_gates: Gates,
    aggregate: BTreeMap<String, Aggregate>,
    derived_checks: Checks,
    runs: Vec<SeedRun>,
    interpretation_constraints: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct Protocol {
    seeds: u64,
    families: [&'static str; 2],
    scale_shift_range: [f64; 2],
    bias_shift_range: [f64; 2],
    calibration_samples_per_seed: usize,
    episode_steps: usize,
    shift_step_range_inclusive: [usize; 2],
    noise_sigma: f64,
    calibration_quantile: f64,
    persistence_required: usize,
    witness_samples: usize,
    verifier_max_after_to_before_mse_ratio: f64,
    resolver_attempt_budget: u32,
}

#[derive(Debug, Serialize)]
struct Gates {
    typed_resolution_rate_gte: f64,
    typed_false_commit_rate_lte: f64,
    typed_median_detection_delay_lte: f64,
    typed_to_structured_direct_mean_mse_ratio_lte: f64,
    typed_to_untyped_mean_mse_ratio_lte: f64,
    typed_to_random_mean_mse_ratio_lte: f64,
    typed_route_advantage_over_untyped_gte: f64,
    typed_excess_mean_mse_over_oracle_lte: f64,
    typed_must_beat: Vec<&'static str>,
    unverified_commit_rate_must_equal: f64,
}

#[derive(Debug, Serialize)]
struct Checks {
    typed_resolution_rate: f64,
    typed_false_commit_rate: f64,
    typed_median_detection_delay: f64,
    typed_to_structured_direct_mean_mse_ratio: f64,
    typed_to_untyped_mean_mse_ratio: f64,
    typed_to_random_mean_mse_ratio: f64,
    typed_route_advantage_over_untyped: f64,
    typed_excess_mean_mse_over_oracle: f64,
    typed_beats_required_controls: bool,
    all_unverified_commit_rates_zero: bool,
    all_gates_pass: bool,
}

fn scope(seed: u64) -> ResidualCalibrationScope {
    ResidualCalibrationScope {
        predictor_scope: format!("affine-baseline-scale1-bias0/v1/seed-{seed}"),
        environment_scope: "hidden-affine-regime-shift/v1".to_string(),
        state_space: STATE_SPACE.to_string(),
        horizon_steps: 1,
    }
}

fn residual(id: String, predicted: f64, observed: f64) -> star::developmental::NumericPredictionResidual {
    let prediction = NumericTransitionPrediction {
        transition_id: id.clone(),
        action: NamedVector { space: ACTION_SPACE.to_string(), values: vec![0.0] },
        predicted_next_state: NamedVector { space: STATE_SPACE.to_string(), values: vec![predicted as f32] },
        horizon_steps: 1,
    };
    let observation = NumericStateObservation {
        transition_id: id,
        state: NamedVector { space: STATE_SPACE.to_string(), values: vec![observed as f32] },
    };
    compare_numeric_transition(&prediction, &observation).expect("comparable probe vectors")
}

fn fit_profile(seed: u64) -> ResidualCalibrationProfile {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xCA11_BA7E_5EED_0001);
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let mut residuals = Vec::with_capacity(CALIBRATION_SAMPLES);
    for index in 0..CALIBRATION_SAMPLES {
        let action = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let predicted = Model::BASELINE.predict(action);
        residuals.push(residual(
            format!("calibration-{seed}-{index}"),
            predicted,
            predicted + noise.sample(&mut rng),
        ));
    }
    ResidualCalibrationProfile::fit_higher_quantile(scope(seed), &residuals, CALIBRATION_QUANTILE)
        .expect("profile must fit")
}

fn episode(seed: u64) -> Episode {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xAFF1_0E00_5EED_0002);
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let family = if seed % 2 == 0 { ShiftFamily::Scale } else { ShiftFamily::Bias };
    let shift_step = rng.gen_range(SHIFT_MIN..=SHIFT_MAX);
    let post_shift_model = match family {
        ShiftFamily::Scale => Model { scale: rng.gen_range(-0.5..0.2), bias: 0.0 },
        ShiftFamily::Bias => Model { scale: 1.0, bias: rng.gen_range(0.8..1.5) },
    };
    let first = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    let actions = (0..EPISODE_STEPS)
        .map(|step| if step % 2 == 0 { first } else { -first })
        .collect();
    let noises = (0..EPISODE_STEPS).map(|_| noise.sample(&mut rng)).collect();
    Episode { family, shift_step, post_shift_model, actions, noises }
}

fn features(buffer: &VecDeque<(f64, f64)>) -> Features {
    let n = buffer.len() as f64;
    Features {
        intercept: buffer.iter().map(|(_, r)| *r).sum::<f64>() / n,
        slope: buffer.iter().map(|(a, r)| a * r).sum::<f64>() / n,
    }
}

fn typed_route(intercept: f64, slope: f64) -> ResolverKind {
    if slope.abs() > intercept.abs() { ResolverKind::ReviseScale } else { ResolverKind::ReviseBias }
}

fn candidate(resolver: ResolverKind, f: Features) -> Model {
    match resolver {
        ResolverKind::ReviseScale => Model { scale: 1.0 + f.slope, bias: 0.0 },
        ResolverKind::ReviseBias => Model { scale: 1.0, bias: f.intercept },
    }
}

fn witness(seed: u64, step: usize, truth: Model, before: Model, after: Model) -> Witness {
    let mut rng = StdRng::seed_from_u64(seed ^ (step as u64).rotate_left(17) ^ 0xB17E_55A5_0003);
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let mut before_sum = 0.0;
    let mut after_sum = 0.0;
    for i in 0..WITNESS_SAMPLES {
        let action = if i % 2 == 0 { 1.0 } else { -1.0 };
        let observed = truth.predict(action) + noise.sample(&mut rng);
        before_sum += (observed - before.predict(action)).powi(2);
        after_sum += (observed - after.predict(action)).powi(2);
    }
    let before_mse = before_sum / WITNESS_SAMPLES as f64;
    let after_mse = after_sum / WITNESS_SAMPLES as f64;
    Witness {
        before_mse,
        after_mse,
        verified: after_mse.is_finite() && after_mse <= before_mse * VERIFIER_MAX_ERROR_RATIO,
    }
}

fn make_charge(strategy: Strategy, f: Features, resolver: ResolverKind) -> Charge {
    let residual = match strategy {
        Strategy::TypedPressure => vec![f.intercept as f32, f.slope as f32],
        Strategy::ScrambledPressure => vec![f.slope as f32, f.intercept as f32],
        _ => vec![],
    };
    let mut charge = Charge::new(
        ChargeKind::PredictionResidual,
        residual,
        1.0,
        ChargeScope::Component("infant-developmental-transition-model".to_string()),
    )
    .traced(resolver.name());
    charge.persistence = PERSISTENCE_REQUIRED.saturating_sub(1) as u32;
    charge
}

fn run_strategy(seed: u64, strategy: Strategy, profile: &ResidualCalibrationProfile, ep: &Episode) -> StrategyRun {
    let scope = scope(seed);
    let mut model = Model::BASELINE;
    let mut buffer = VecDeque::with_capacity(PERSISTENCE_REQUIRED);
    let mut attempted = false;
    let mut committed = false;
    let mut false_commit = false;
    let mut route_correct = false;
    let mut selected_resolver = None;
    let mut trigger_step = None;
    let mut witness_before_mse = None;
    let mut witness_after_mse = None;
    let mut accepted_discharge = 0.0;
    let mut unresolved_charge_at_end = false;
    let mut unverified_commit = false;
    let mut cumulative_mse = 0.0;
    let mut post_shift_mse = 0.0;
    let mut random_router = StdRng::seed_from_u64(seed ^ 0xA0B7_EA11_0004);

    for step in 0..EPISODE_STEPS {
        let action = ep.actions[step];
        let truth = if step < ep.shift_step { Model::BASELINE } else { ep.post_shift_model };
        let observed = truth.predict(action) + ep.noises[step];
        let predicted = model.predict(action);
        let r = residual(format!("episode-{seed}-{}-{step}", strategy.name()), predicted, observed);
        cumulative_mse += r.mean_squared_error;
        if step >= ep.shift_step { post_shift_mse += r.mean_squared_error; }

        if committed || attempted || strategy == Strategy::NoRevision { continue; }
        if strategy == Strategy::Oracle {
            if step == ep.shift_step {
                attempted = true;
                committed = true;
                route_correct = true;
                trigger_step = Some(step);
                selected_resolver = Some(match ep.family {
                    ShiftFamily::Scale => ResolverKind::ReviseScale,
                    ShiftFamily::Bias => ResolverKind::ReviseBias,
                });
                model = ep.post_shift_model;
            }
            continue;
        }

        if profile.assess(&scope, &r).unwrap().exceeded {
            if buffer.len() == PERSISTENCE_REQUIRED { buffer.pop_front(); }
            buffer.push_back((action, observed - predicted));
        } else {
            buffer.clear();
        }
        if buffer.len() < PERSISTENCE_REQUIRED { continue; }

        attempted = true;
        trigger_step = Some(step);
        let f = features(&buffer);
        let resolver = match strategy {
            Strategy::TypedPressure | Strategy::StructuredDirectNoPressure => typed_route(f.intercept, f.slope),
            Strategy::UntypedMatchedTrigger => ResolverKind::ReviseScale,
            Strategy::RandomMatchedTrigger => if random_router.gen_bool(0.5) { ResolverKind::ReviseScale } else { ResolverKind::ReviseBias },
            Strategy::ScrambledPressure => typed_route(f.slope, f.intercept),
            Strategy::NoRevision | Strategy::Oracle => unreachable!(),
        };
        selected_resolver = Some(resolver);
        route_correct = resolver.matches(ep.family);
        let proposed = candidate(resolver, f);
        let w = witness(seed, step, truth, model, proposed);
        witness_before_mse = Some(w.before_mse);
        witness_after_mse = Some(w.after_mse);

        if strategy.uses_charge() {
            let mut cycle = CognitiveCycleState::new();
            assert!(cycle.admit_charge(make_charge(strategy, f, resolver)));
            let resolution = Resolution { discharged: 1.0, emitted: vec![], permitted_decay: 0.0, compute_cost: 1 };
            let outcome = OutcomeWitness::new(
                "held_out_revision_verified",
                0.0,
                if w.verified { 1.0 } else { 0.0 },
                ImprovementDirection::HigherIsBetter,
                vec![
                    format!("before_mse={}", w.before_mse),
                    format!("after_mse={}", w.after_mse),
                    format!("resolver={}", resolver.name()),
                ],
            );
            let pending = cycle.pending()[0].clone();
            let judged = RelativeImprovementJudge.evaluate(&pending, &resolution, &outcome);
            let disposition = cycle.apply_judgment(0, &judged).unwrap();
            accepted_discharge = cycle.total_accepted_discharge();
            if disposition == ChargeDisposition::Resolved {
                if !w.verified { unverified_commit = true; }
                model = proposed;
                committed = true;
                false_commit = step < ep.shift_step;
            } else {
                unresolved_charge_at_end = !cycle.pending().is_empty();
            }
        } else if w.verified {
            model = proposed;
            committed = true;
            false_commit = step < ep.shift_step;
        }
    }

    StrategyRun {
        cumulative_mse,
        post_shift_mse,
        attempted,
        committed,
        false_commit,
        route_correct,
        selected_resolver,
        trigger_step,
        detection_delay: trigger_step.and_then(|step| if committed && step >= ep.shift_step { Some(step - ep.shift_step) } else { None }),
        witness_before_mse,
        witness_after_mse,
        accepted_discharge,
        unresolved_charge_at_end,
        unverified_commit,
    }
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() { return None; }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = values.len() / 2;
    Some(if values.len() % 2 == 0 { (values[m - 1] + values[m]) / 2.0 } else { values[m] })
}

fn aggregate(strategy: Strategy, runs: &[SeedRun]) -> Aggregate {
    let values: Vec<&StrategyRun> = runs.iter().map(|run| &run.strategies[strategy.name()]).collect();
    let n = values.len() as f64;
    let cumulative: Vec<f64> = values.iter().map(|run| run.cumulative_mse).collect();
    let delays: Vec<f64> = values.iter().filter_map(|run| run.detection_delay.map(|d| d as f64)).collect();
    Aggregate {
        mean_cumulative_mse: cumulative.iter().sum::<f64>() / n,
        median_cumulative_mse: median(cumulative).unwrap(),
        mean_post_shift_mse: values.iter().map(|run| run.post_shift_mse).sum::<f64>() / n,
        attempt_rate: values.iter().filter(|run| run.attempted).count() as f64 / n,
        commit_rate: values.iter().filter(|run| run.committed).count() as f64 / n,
        false_commit_rate: values.iter().filter(|run| run.false_commit).count() as f64 / n,
        route_correct_rate: values.iter().filter(|run| run.route_correct).count() as f64 / n,
        unresolved_charge_rate: values.iter().filter(|run| run.unresolved_charge_at_end).count() as f64 / n,
        mean_accepted_discharge: values.iter().map(|run| run.accepted_discharge).sum::<f64>() / n,
        unverified_commit_rate: values.iter().filter(|run| run.unverified_commit).count() as f64 / n,
        median_detection_delay: median(delays),
    }
}

fn main() {
    let mut runs = Vec::with_capacity(SEEDS as usize);
    for seed in 0..SEEDS {
        let profile = fit_profile(seed);
        let ep = episode(seed);
        let strategies = Strategy::ALL
            .into_iter()
            .map(|strategy| (strategy.name().to_string(), run_strategy(seed, strategy, &profile, &ep)))
            .collect();
        runs.push(SeedRun {
            seed,
            family: ep.family,
            shift_step: ep.shift_step,
            post_shift_scale: ep.post_shift_model.scale,
            post_shift_bias: ep.post_shift_model.bias,
            threshold_mse: profile.threshold,
            strategies,
        });
    }

    let aggregate_by_strategy: BTreeMap<String, Aggregate> = Strategy::ALL
        .into_iter()
        .map(|strategy| (strategy.name().to_string(), aggregate(strategy, &runs)))
        .collect();
    let typed = aggregate_by_strategy["typed_pressure"].clone();
    let direct = aggregate_by_strategy["structured_direct_no_pressure"].clone();
    let untyped = aggregate_by_strategy["untyped_matched_trigger"].clone();
    let random = aggregate_by_strategy["random_matched_trigger"].clone();
    let scrambled = aggregate_by_strategy["scrambled_pressure"].clone();
    let no_revision = aggregate_by_strategy["no_revision"].clone();
    let oracle = aggregate_by_strategy["oracle"].clone();

    let typed_delay = typed.median_detection_delay.unwrap_or(f64::INFINITY);
    let typed_to_direct = typed.mean_cumulative_mse / direct.mean_cumulative_mse;
    let typed_to_untyped = typed.mean_cumulative_mse / untyped.mean_cumulative_mse;
    let typed_to_random = typed.mean_cumulative_mse / random.mean_cumulative_mse;
    let route_advantage = typed.route_correct_rate - untyped.route_correct_rate;
    let excess_over_oracle = typed.mean_cumulative_mse - oracle.mean_cumulative_mse;
    let beats_controls = [&untyped, &random, &scrambled, &no_revision]
        .iter()
        .all(|control| typed.mean_cumulative_mse < control.mean_cumulative_mse);
    let all_unverified_zero = aggregate_by_strategy.values().all(|a| a.unverified_commit_rate == 0.0);
    let pass = typed.commit_rate >= MIN_TYPED_RESOLUTION_RATE
        && typed.false_commit_rate <= MAX_TYPED_FALSE_COMMIT_RATE
        && typed_delay <= MAX_TYPED_MEDIAN_DELAY
        && typed_to_direct <= MAX_TYPED_TO_STRUCTURED_DIRECT_MSE_RATIO
        && typed_to_untyped <= MAX_TYPED_TO_UNTYPED_MSE_RATIO
        && typed_to_random <= MAX_TYPED_TO_RANDOM_MSE_RATIO
        && route_advantage >= MIN_TYPED_ROUTE_ADVANTAGE_OVER_UNTYPED
        && excess_over_oracle <= MAX_TYPED_EXCESS_MSE_OVER_ORACLE
        && beats_controls
        && all_unverified_zero;

    let output = Output {
        experiment: "h_infant_typed_pressure_routing_probe",
        status: if pass { "pass" } else { "fail" },
        protocol: Protocol {
            seeds: SEEDS,
            families: ["scale_shift", "bias_shift"],
            scale_shift_range: [-0.5, 0.2],
            bias_shift_range: [0.8, 1.5],
            calibration_samples_per_seed: CALIBRATION_SAMPLES,
            episode_steps: EPISODE_STEPS,
            shift_step_range_inclusive: [SHIFT_MIN, SHIFT_MAX],
            noise_sigma: NOISE_SIGMA,
            calibration_quantile: CALIBRATION_QUANTILE,
            persistence_required: PERSISTENCE_REQUIRED,
            witness_samples: WITNESS_SAMPLES,
            verifier_max_after_to_before_mse_ratio: VERIFIER_MAX_ERROR_RATIO,
            resolver_attempt_budget: 1,
        },
        predeclared_gates: Gates {
            typed_resolution_rate_gte: MIN_TYPED_RESOLUTION_RATE,
            typed_false_commit_rate_lte: MAX_TYPED_FALSE_COMMIT_RATE,
            typed_median_detection_delay_lte: MAX_TYPED_MEDIAN_DELAY,
            typed_to_structured_direct_mean_mse_ratio_lte: MAX_TYPED_TO_STRUCTURED_DIRECT_MSE_RATIO,
            typed_to_untyped_mean_mse_ratio_lte: MAX_TYPED_TO_UNTYPED_MSE_RATIO,
            typed_to_random_mean_mse_ratio_lte: MAX_TYPED_TO_RANDOM_MSE_RATIO,
            typed_route_advantage_over_untyped_gte: MIN_TYPED_ROUTE_ADVANTAGE_OVER_UNTYPED,
            typed_excess_mean_mse_over_oracle_lte: MAX_TYPED_EXCESS_MSE_OVER_ORACLE,
            typed_must_beat: vec!["untyped_matched_trigger", "random_matched_trigger", "scrambled_pressure", "no_revision"],
            unverified_commit_rate_must_equal: 0.0,
        },
        aggregate: aggregate_by_strategy,
        derived_checks: Checks {
            typed_resolution_rate: typed.commit_rate,
            typed_false_commit_rate: typed.false_commit_rate,
            typed_median_detection_delay: typed_delay,
            typed_to_structured_direct_mean_mse_ratio: typed_to_direct,
            typed_to_untyped_mean_mse_ratio: typed_to_untyped,
            typed_to_random_mean_mse_ratio: typed_to_random,
            typed_route_advantage_over_untyped: route_advantage,
            typed_excess_mean_mse_over_oracle: excess_over_oracle,
            typed_beats_required_controls: beats_controls,
            all_unverified_commit_rates_zero: all_unverified_zero,
            all_gates_pass: pass,
        },
        runs,
        interpretation_constraints: vec![
            "This is a diagnostic closed-cycle probe, not live runtime routing.",
            "Typed pressure and matched controls share calibration, trigger timing, raw evidence, held-out witness budget, and one resolver-attempt budget.",
            "The residual coordinates are interpretable affine features: intercept and action-correlated slope.",
            "The structured direct no-pressure control tests whether task utility comes from residual structure rather than the Charge container itself.",
            "A pass would justify only a later multi-pressure prioritization or held-out transfer test before any live routing promotion.",
        ],
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    if !pass { std::process::exit(1); }
}
