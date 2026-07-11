//! H-Infant multi-pressure prioritization probe.
//!
//! Diagnostic-only. Multiple unresolved pressures compete for one attention slot
//! per step. The developmental prediction residual is individually quieter than
//! fresh transient distractors but becomes persistent. A kind-agnostic scheduler
//! may use only generic charge magnitude and persistence to decide what receives
//! attention first.
//!
//! The selected developmental residual still uses the typed affine residual
//! structure, held-out witness, `PredictionResidual`, `CognitiveCycleState`, and
//! independent discharge path established in the parent experiment.

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
const EVIDENCE_REQUIRED: usize = 4;
const WITNESS_SAMPLES: usize = 32;
const VERIFIER_MAX_ERROR_RATIO: f64 = 0.25;
const RELEVANT_MAGNITUDE: f32 = 1.0;
const DISTRACTOR_MAGNITUDE_MIN: f32 = 3.2;
const DISTRACTOR_MAGNITUDE_MAX: f32 = 3.8;
const STATE_SPACE: &str = "synthetic.affine_delta.v1";
const ACTION_SPACE: &str = "synthetic.action.scalar.v1";

// Frozen acceptance gates.
const MIN_PRIORITY_COMMIT_RATE: f64 = 0.95;
const MAX_PRIORITY_FALSE_COMMIT_RATE: f64 = 0.02;
const MAX_PRIORITY_MEDIAN_DELAY: f64 = 4.0;
const MAX_PRIORITY_TO_DIRECT_MSE_RATIO: f64 = 1.05;
const MAX_PRIORITY_TO_MAGNITUDE_MSE_RATIO: f64 = 0.25;
const MAX_PRIORITY_TO_RANDOM_MSE_RATIO: f64 = 0.50;
const MAX_PRIORITY_TO_OLDEST_MSE_RATIO: f64 = 0.50;
const MAX_PRIORITY_EXCESS_MSE_OVER_ORACLE: f64 = 10.0;
const MIN_PRIORITY_RELEVANT_SELECTION_PERSISTENCE: u32 = 3;

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
    PersistenceWeightedPressure,
    DirectPersistenceWeighted,
    MagnitudeOnlyPressure,
    RandomPressure,
    OldestFirstPressure,
    NoRevision,
    Oracle,
}

impl Strategy {
    const ALL: [Self; 7] = [
        Self::PersistenceWeightedPressure,
        Self::DirectPersistenceWeighted,
        Self::MagnitudeOnlyPressure,
        Self::RandomPressure,
        Self::OldestFirstPressure,
        Self::NoRevision,
        Self::Oracle,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::PersistenceWeightedPressure => "persistence_weighted_pressure",
            Self::DirectPersistenceWeighted => "direct_persistence_weighted",
            Self::MagnitudeOnlyPressure => "magnitude_only_pressure",
            Self::RandomPressure => "random_pressure",
            Self::OldestFirstPressure => "oldest_first_pressure",
            Self::NoRevision => "no_revision",
            Self::Oracle => "oracle",
        }
    }

    fn uses_charge_path(self) -> bool {
        matches!(
            self,
            Self::PersistenceWeightedPressure
                | Self::MagnitudeOnlyPressure
                | Self::RandomPressure
                | Self::OldestFirstPressure
        )
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
    distractor_magnitudes: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
struct Witness {
    before_mse: f64,
    after_mse: f64,
    verified: bool,
}

#[derive(Debug, Clone)]
struct RelevantPressure {
    charge: Charge,
    evidence: VecDeque<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize)]
struct StrategyRun {
    cumulative_mse: f64,
    post_shift_mse: f64,
    attention_steps: u64,
    distractor_selections: u64,
    relevant_selections: u64,
    attempted: bool,
    committed: bool,
    false_commit: bool,
    route_correct: bool,
    selected_resolver: Option<ResolverKind>,
    relevant_selection_persistence: Option<u32>,
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
    mean_attention_steps: f64,
    mean_distractor_selections: f64,
    mean_relevant_selections: f64,
    attempt_rate: f64,
    commit_rate: f64,
    false_commit_rate: f64,
    route_correct_rate: f64,
    unresolved_charge_rate: f64,
    unverified_commit_rate: f64,
    median_detection_delay: Option<f64>,
    median_relevant_selection_persistence: Option<f64>,
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
    calibration_samples_per_seed: usize,
    episode_steps: usize,
    shift_step_range_inclusive: [usize; 2],
    noise_sigma: f64,
    calibration_quantile: f64,
    one_attention_slot_per_step: bool,
    relevant_charge_magnitude: f32,
    distractor_magnitude_range: [f32; 2],
    persistence_score: &'static str,
    scheduler_inspects_charge_kind: bool,
    evidence_pattern: &'static str,
    witness_samples: usize,
    resolver_attempt_budget: u32,
}

#[derive(Debug, Serialize)]
struct Gates {
    priority_commit_rate_gte: f64,
    priority_false_commit_rate_lte: f64,
    priority_median_detection_delay_lte: f64,
    priority_to_direct_mean_mse_ratio_lte: f64,
    priority_to_magnitude_mean_mse_ratio_lte: f64,
    priority_to_random_mean_mse_ratio_lte: f64,
    priority_to_oldest_mean_mse_ratio_lte: f64,
    priority_excess_mean_mse_over_oracle_lte: f64,
    priority_median_relevant_selection_persistence_gte: u32,
    priority_must_beat: Vec<&'static str>,
    unverified_commit_rate_must_equal: f64,
}

#[derive(Debug, Serialize)]
struct Checks {
    priority_commit_rate: f64,
    priority_false_commit_rate: f64,
    priority_median_detection_delay: f64,
    priority_to_direct_mean_mse_ratio: f64,
    priority_to_magnitude_mean_mse_ratio: f64,
    priority_to_random_mean_mse_ratio: f64,
    priority_to_oldest_mean_mse_ratio: f64,
    priority_excess_mean_mse_over_oracle: f64,
    priority_median_relevant_selection_persistence: f64,
    priority_beats_required_controls: bool,
    all_unverified_commit_rates_zero: bool,
    all_gates_pass: bool,
}

fn calibration_scope(seed: u64) -> ResidualCalibrationScope {
    ResidualCalibrationScope {
        predictor_scope: format!("affine-baseline-scale1-bias0/v1/seed-{seed}"),
        environment_scope: "hidden-affine-multi-pressure/v1".to_string(),
        state_space: STATE_SPACE.to_string(),
        horizon_steps: 1,
    }
}

fn numeric_residual(
    transition_id: String,
    predicted: f64,
    observed: f64,
) -> star::developmental::NumericPredictionResidual {
    let prediction = NumericTransitionPrediction {
        transition_id: transition_id.clone(),
        action: NamedVector {
            space: ACTION_SPACE.to_string(),
            values: vec![0.0],
        },
        predicted_next_state: NamedVector {
            space: STATE_SPACE.to_string(),
            values: vec![predicted as f32],
        },
        horizon_steps: 1,
    };
    let observation = NumericStateObservation {
        transition_id,
        state: NamedVector {
            space: STATE_SPACE.to_string(),
            values: vec![observed as f32],
        },
    };
    compare_numeric_transition(&prediction, &observation)
        .expect("probe vectors must be comparable")
}

fn fit_profile(seed: u64) -> ResidualCalibrationProfile {
    let mut rng = StdRng::seed_from_u64(seed ^ 0xCA11_BA7E_5EED_1001);
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let mut residuals = Vec::with_capacity(CALIBRATION_SAMPLES);
    for index in 0..CALIBRATION_SAMPLES {
        let action = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let predicted = Model::BASELINE.predict(action);
        residuals.push(numeric_residual(
            format!("calibration-{seed}-{index}"),
            predicted,
            predicted + noise.sample(&mut rng),
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
    let mut rng = StdRng::seed_from_u64(seed ^ 0xA771_EA77_5EED_1002);
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let family = if seed % 2 == 0 {
        ShiftFamily::Scale
    } else {
        ShiftFamily::Bias
    };
    let shift_step = rng.gen_range(SHIFT_MIN..=SHIFT_MAX);
    let post_shift_model = match family {
        ShiftFamily::Scale => Model {
            scale: rng.gen_range(-0.5..0.2),
            bias: 0.0,
        },
        ShiftFamily::Bias => Model {
            scale: 1.0,
            bias: rng.gen_range(0.8..1.5),
        },
    };
    let first = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    let actions = (0..EPISODE_STEPS)
        .map(|step| match step % 4 {
            0 | 1 => first,
            _ => -first,
        })
        .collect();
    let noises = (0..EPISODE_STEPS)
        .map(|_| noise.sample(&mut rng))
        .collect();
    let distractor_magnitudes = (0..EPISODE_STEPS)
        .map(|_| rng.gen_range(DISTRACTOR_MAGNITUDE_MIN..DISTRACTOR_MAGNITUDE_MAX))
        .collect();

    Episode {
        family,
        shift_step,
        post_shift_model,
        actions,
        noises,
        distractor_magnitudes,
    }
}

fn features(evidence: &VecDeque<(f64, f64)>) -> Features {
    let n = evidence.len() as f64;
    Features {
        intercept: evidence.iter().map(|(_, residual)| *residual).sum::<f64>() / n,
        slope: evidence
            .iter()
            .map(|(action, residual)| action * residual)
            .sum::<f64>()
            / n,
    }
}

fn typed_route(f: Features) -> ResolverKind {
    if f.slope.abs() > f.intercept.abs() {
        ResolverKind::ReviseScale
    } else {
        ResolverKind::ReviseBias
    }
}

fn candidate(resolver: ResolverKind, f: Features) -> Model {
    match resolver {
        ResolverKind::ReviseScale => Model {
            scale: 1.0 + f.slope,
            bias: 0.0,
        },
        ResolverKind::ReviseBias => Model {
            scale: 1.0,
            bias: f.intercept,
        },
    }
}

fn witness(seed: u64, step: usize, truth: Model, before: Model, after: Model) -> Witness {
    let mut rng = StdRng::seed_from_u64(
        seed ^ (step as u64).rotate_left(17) ^ 0xB17E_55A5_1003,
    );
    let noise = Normal::new(0.0, NOISE_SIGMA).unwrap();
    let mut before_sum = 0.0;
    let mut after_sum = 0.0;
    for index in 0..WITNESS_SAMPLES {
        let action = if index % 2 == 0 { 1.0 } else { -1.0 };
        let observed = truth.predict(action) + noise.sample(&mut rng);
        before_sum += (observed - before.predict(action)).powi(2);
        after_sum += (observed - after.predict(action)).powi(2);
    }
    let before_mse = before_sum / WITNESS_SAMPLES as f64;
    let after_mse = after_sum / WITNESS_SAMPLES as f64;
    Witness {
        before_mse,
        after_mse,
        verified: after_mse.is_finite()
            && after_mse <= before_mse * VERIFIER_MAX_ERROR_RATIO,
    }
}

fn distractor_charge(step: usize, magnitude: f32) -> Charge {
    let kind = match step % 3 {
        0 => ChargeKind::EpistemicGap,
        1 => ChargeKind::GoalTension,
        _ => ChargeKind::Contradiction,
    };
    Charge::new(
        kind,
        vec![magnitude],
        magnitude,
        ChargeScope::Component(format!("transient-distractor-{step}")),
    )
}

fn update_relevant_pressure(
    existing: Option<RelevantPressure>,
    action: f64,
    signed_residual: f64,
) -> RelevantPressure {
    let mut pressure = existing.unwrap_or_else(|| RelevantPressure {
        charge: Charge::new(
            ChargeKind::PredictionResidual,
            vec![0.0, 0.0],
            RELEVANT_MAGNITUDE,
            ChargeScope::Component("infant-developmental-transition-model".to_string()),
        ),
        evidence: VecDeque::with_capacity(EVIDENCE_REQUIRED),
    });

    if pressure.evidence.len() == EVIDENCE_REQUIRED {
        pressure.evidence.pop_front();
    }
    pressure.evidence.push_back((action, signed_residual));
    let f = features(&pressure.evidence);
    pressure.charge.residual = vec![f.intercept as f32, f.slope as f32];
    pressure.charge.magnitude = RELEVANT_MAGNITUDE;
    if pressure.evidence.len() > 1 {
        pressure.charge.persistence = pressure.charge.persistence.saturating_add(1);
    }
    pressure
}

fn priority_score(charge: &Charge) -> f64 {
    f64::from(charge.magnitude) * (1.0 + f64::from(charge.persistence))
}

fn resolve_distractor(charge: Charge) {
    let mut cycle = CognitiveCycleState::new();
    assert!(cycle.admit_charge(charge));
    let pending = cycle.pending()[0].clone();
    let resolution = Resolution {
        discharged: pending.magnitude,
        emitted: vec![],
        permitted_decay: 0.0,
        compute_cost: 1,
    };
    let witness = OutcomeWitness::new(
        "transient_attention_completed",
        0.0,
        1.0,
        ImprovementDirection::HigherIsBetter,
        vec!["distractor handled".to_string()],
    );
    let judged = RelativeImprovementJudge.evaluate(&pending, &resolution, &witness);
    assert_eq!(
        cycle.apply_judgment(0, &judged),
        Some(ChargeDisposition::Resolved)
    );
}

fn commit_relevant_with_charge(
    pressure: &RelevantPressure,
    seed: u64,
    step: usize,
    truth: Model,
    before: Model,
) -> (bool, bool, ResolverKind, Model, Witness, f64) {
    let f = features(&pressure.evidence);
    let resolver = typed_route(f);
    let proposed = candidate(resolver, f);
    let w = witness(seed, step, truth, before, proposed);
    let mut charge = pressure.charge.clone().traced(resolver.name());
    charge.residual = vec![f.intercept as f32, f.slope as f32];
    let mut cycle = CognitiveCycleState::new();
    assert!(cycle.admit_charge(charge));
    let pending = cycle.pending()[0].clone();
    let resolution = Resolution {
        discharged: 1.0,
        emitted: vec![],
        permitted_decay: 0.0,
        compute_cost: 1,
    };
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
    let judged = RelativeImprovementJudge.evaluate(&pending, &resolution, &outcome);
    let disposition = cycle.apply_judgment(0, &judged).unwrap();
    let committed = disposition == ChargeDisposition::Resolved;
    (
        committed,
        committed && !w.verified,
        resolver,
        proposed,
        w,
        cycle.total_accepted_discharge(),
    )
}

fn commit_relevant_direct(
    pressure: &RelevantPressure,
    seed: u64,
    step: usize,
    truth: Model,
    before: Model,
) -> (bool, ResolverKind, Model, Witness) {
    let f = features(&pressure.evidence);
    let resolver = typed_route(f);
    let proposed = candidate(resolver, f);
    let w = witness(seed, step, truth, before, proposed);
    (w.verified, resolver, proposed, w)
}

fn run_strategy(
    seed: u64,
    strategy: Strategy,
    profile: &ResidualCalibrationProfile,
    episode: &Episode,
) -> StrategyRun {
    let scope = calibration_scope(seed);
    let mut model = Model::BASELINE;
    let mut relevant: Option<RelevantPressure> = None;
    let mut attempted = false;
    let mut committed = false;
    let mut false_commit = false;
    let mut route_correct = false;
    let mut selected_resolver = None;
    let mut relevant_selection_persistence = None;
    let mut trigger_step = None;
    let mut witness_before_mse = None;
    let mut witness_after_mse = None;
    let mut accepted_discharge = 0.0;
    let mut unresolved_charge_at_end = false;
    let mut unverified_commit = false;
    let mut cumulative_mse = 0.0;
    let mut post_shift_mse = 0.0;
    let mut attention_steps = 0_u64;
    let mut distractor_selections = 0_u64;
    let mut relevant_selections = 0_u64;
    let mut random_scheduler = StdRng::seed_from_u64(seed ^ 0x5CED_011E_1004);

    for step in 0..EPISODE_STEPS {
        let action = episode.actions[step];
        let truth = if step < episode.shift_step {
            Model::BASELINE
        } else {
            episode.post_shift_model
        };
        let observed = truth.predict(action) + episode.noises[step];
        let predicted = model.predict(action);
        let r = numeric_residual(
            format!("episode-{seed}-{}-{step}", strategy.name()),
            predicted,
            observed,
        );
        cumulative_mse += r.mean_squared_error;
        if step >= episode.shift_step {
            post_shift_mse += r.mean_squared_error;
        }

        if committed || attempted || strategy == Strategy::NoRevision {
            continue;
        }
        if strategy == Strategy::Oracle {
            if step == episode.shift_step {
                attempted = true;
                committed = true;
                route_correct = true;
                selected_resolver = Some(match episode.family {
                    ShiftFamily::Scale => ResolverKind::ReviseScale,
                    ShiftFamily::Bias => ResolverKind::ReviseBias,
                });
                trigger_step = Some(step);
                model = episode.post_shift_model;
            }
            continue;
        }

        if profile.assess(&scope, &r).unwrap().exceeded {
            relevant = Some(update_relevant_pressure(
                relevant.take(),
                action,
                observed - predicted,
            ));
        } else {
            relevant = None;
        }

        let distractor = distractor_charge(step, episode.distractor_magnitudes[step]);
        attention_steps = attention_steps.saturating_add(1);

        let choose_relevant = match (&relevant, strategy) {
            (None, _) => false,
            (Some(pressure), Strategy::PersistenceWeightedPressure) => {
                priority_score(&pressure.charge) > priority_score(&distractor)
            }
            (Some(pressure), Strategy::DirectPersistenceWeighted) => {
                priority_score(&pressure.charge) > priority_score(&distractor)
            }
            (Some(pressure), Strategy::MagnitudeOnlyPressure) => {
                pressure.charge.magnitude > distractor.magnitude
            }
            (Some(_), Strategy::RandomPressure) => random_scheduler.gen_bool(0.5),
            (Some(pressure), Strategy::OldestFirstPressure) => {
                pressure.charge.persistence > distractor.persistence
            }
            (Some(_), Strategy::NoRevision | Strategy::Oracle) => unreachable!(),
        };

        if !choose_relevant {
            distractor_selections = distractor_selections.saturating_add(1);
            if strategy.uses_charge_path() {
                resolve_distractor(distractor);
            }
            continue;
        }

        relevant_selections = relevant_selections.saturating_add(1);
        attempted = true;
        trigger_step = Some(step);
        let pressure = relevant.as_ref().expect("selected relevant pressure must exist");
        relevant_selection_persistence = Some(pressure.charge.persistence);

        if strategy == Strategy::DirectPersistenceWeighted {
            let (verified, resolver, proposed, w) =
                commit_relevant_direct(pressure, seed, step, truth, model);
            selected_resolver = Some(resolver);
            route_correct = resolver.matches(episode.family);
            witness_before_mse = Some(w.before_mse);
            witness_after_mse = Some(w.after_mse);
            if verified {
                committed = true;
                false_commit = step < episode.shift_step;
                model = proposed;
            }
        } else {
            let (resolved, unverified, resolver, proposed, w, accepted) =
                commit_relevant_with_charge(pressure, seed, step, truth, model);
            selected_resolver = Some(resolver);
            route_correct = resolver.matches(episode.family);
            witness_before_mse = Some(w.before_mse);
            witness_after_mse = Some(w.after_mse);
            accepted_discharge = accepted;
            unverified_commit = unverified;
            if resolved {
                committed = true;
                false_commit = step < episode.shift_step;
                model = proposed;
            } else {
                unresolved_charge_at_end = true;
            }
        }
    }

    StrategyRun {
        cumulative_mse,
        post_shift_mse,
        attention_steps,
        distractor_selections,
        relevant_selections,
        attempted,
        committed,
        false_commit,
        route_correct,
        selected_resolver,
        relevant_selection_persistence,
        trigger_step,
        detection_delay: trigger_step.and_then(|step| {
            if committed && step >= episode.shift_step {
                Some(step - episode.shift_step)
            } else {
                None
            }
        }),
        witness_before_mse,
        witness_after_mse,
        accepted_discharge,
        unresolved_charge_at_end,
        unverified_commit,
    }
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let middle = values.len() / 2;
    Some(if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}

fn aggregate(strategy: Strategy, runs: &[SeedRun]) -> Aggregate {
    let values: Vec<&StrategyRun> = runs
        .iter()
        .map(|run| &run.strategies[strategy.name()])
        .collect();
    let n = values.len() as f64;
    let cumulative: Vec<f64> = values.iter().map(|run| run.cumulative_mse).collect();
    let delays: Vec<f64> = values
        .iter()
        .filter_map(|run| run.detection_delay.map(|delay| delay as f64))
        .collect();
    let selection_persistence: Vec<f64> = values
        .iter()
        .filter_map(|run| run.relevant_selection_persistence.map(f64::from))
        .collect();

    Aggregate {
        mean_cumulative_mse: cumulative.iter().sum::<f64>() / n,
        median_cumulative_mse: median(cumulative).unwrap(),
        mean_post_shift_mse: values.iter().map(|run| run.post_shift_mse).sum::<f64>() / n,
        mean_attention_steps: values.iter().map(|run| run.attention_steps as f64).sum::<f64>() / n,
        mean_distractor_selections: values
            .iter()
            .map(|run| run.distractor_selections as f64)
            .sum::<f64>()
            / n,
        mean_relevant_selections: values
            .iter()
            .map(|run| run.relevant_selections as f64)
            .sum::<f64>()
            / n,
        attempt_rate: values.iter().filter(|run| run.attempted).count() as f64 / n,
        commit_rate: values.iter().filter(|run| run.committed).count() as f64 / n,
        false_commit_rate: values.iter().filter(|run| run.false_commit).count() as f64 / n,
        route_correct_rate: values.iter().filter(|run| run.route_correct).count() as f64 / n,
        unresolved_charge_rate: values
            .iter()
            .filter(|run| run.unresolved_charge_at_end)
            .count() as f64
            / n,
        unverified_commit_rate: values
            .iter()
            .filter(|run| run.unverified_commit)
            .count() as f64
            / n,
        median_detection_delay: median(delays),
        median_relevant_selection_persistence: median(selection_persistence),
    }
}

fn main() {
    let mut runs = Vec::with_capacity(SEEDS as usize);
    for seed in 0..SEEDS {
        let profile = fit_profile(seed);
        let episode = generate_episode(seed);
        let strategies = Strategy::ALL
            .into_iter()
            .map(|strategy| {
                (
                    strategy.name().to_string(),
                    run_strategy(seed, strategy, &profile, &episode),
                )
            })
            .collect();
        runs.push(SeedRun {
            seed,
            family: episode.family,
            shift_step: episode.shift_step,
            post_shift_scale: episode.post_shift_model.scale,
            post_shift_bias: episode.post_shift_model.bias,
            threshold_mse: profile.threshold,
            strategies,
        });
    }

    let aggregate_by_strategy: BTreeMap<String, Aggregate> = Strategy::ALL
        .into_iter()
        .map(|strategy| {
            (
                strategy.name().to_string(),
                aggregate(strategy, &runs),
            )
        })
        .collect();

    let priority = aggregate_by_strategy["persistence_weighted_pressure"].clone();
    let direct = aggregate_by_strategy["direct_persistence_weighted"].clone();
    let magnitude = aggregate_by_strategy["magnitude_only_pressure"].clone();
    let random = aggregate_by_strategy["random_pressure"].clone();
    let oldest = aggregate_by_strategy["oldest_first_pressure"].clone();
    let no_revision = aggregate_by_strategy["no_revision"].clone();
    let oracle = aggregate_by_strategy["oracle"].clone();

    let priority_delay = priority.median_detection_delay.unwrap_or(f64::INFINITY);
    let priority_selection_persistence = priority
        .median_relevant_selection_persistence
        .unwrap_or(f64::NEG_INFINITY);
    let priority_to_direct = priority.mean_cumulative_mse / direct.mean_cumulative_mse;
    let priority_to_magnitude = priority.mean_cumulative_mse / magnitude.mean_cumulative_mse;
    let priority_to_random = priority.mean_cumulative_mse / random.mean_cumulative_mse;
    let priority_to_oldest = priority.mean_cumulative_mse / oldest.mean_cumulative_mse;
    let excess_over_oracle = priority.mean_cumulative_mse - oracle.mean_cumulative_mse;
    let beats_controls = [&magnitude, &random, &oldest, &no_revision]
        .iter()
        .all(|control| priority.mean_cumulative_mse < control.mean_cumulative_mse);
    let all_unverified_zero = aggregate_by_strategy
        .values()
        .all(|aggregate| aggregate.unverified_commit_rate == 0.0);

    let pass = priority.commit_rate >= MIN_PRIORITY_COMMIT_RATE
        && priority.false_commit_rate <= MAX_PRIORITY_FALSE_COMMIT_RATE
        && priority_delay <= MAX_PRIORITY_MEDIAN_DELAY
        && priority_to_direct <= MAX_PRIORITY_TO_DIRECT_MSE_RATIO
        && priority_to_magnitude <= MAX_PRIORITY_TO_MAGNITUDE_MSE_RATIO
        && priority_to_random <= MAX_PRIORITY_TO_RANDOM_MSE_RATIO
        && priority_to_oldest <= MAX_PRIORITY_TO_OLDEST_MSE_RATIO
        && excess_over_oracle <= MAX_PRIORITY_EXCESS_MSE_OVER_ORACLE
        && priority_selection_persistence
            >= f64::from(MIN_PRIORITY_RELEVANT_SELECTION_PERSISTENCE)
        && beats_controls
        && all_unverified_zero;

    let output = Output {
        experiment: "h_infant_multi_pressure_priority_probe",
        status: if pass { "pass" } else { "fail" },
        protocol: Protocol {
            seeds: SEEDS,
            families: ["scale_shift", "bias_shift"],
            calibration_samples_per_seed: CALIBRATION_SAMPLES,
            episode_steps: EPISODE_STEPS,
            shift_step_range_inclusive: [SHIFT_MIN, SHIFT_MAX],
            noise_sigma: NOISE_SIGMA,
            calibration_quantile: CALIBRATION_QUANTILE,
            one_attention_slot_per_step: true,
            relevant_charge_magnitude: RELEVANT_MAGNITUDE,
            distractor_magnitude_range: [DISTRACTOR_MAGNITUDE_MIN, DISTRACTOR_MAGNITUDE_MAX],
            persistence_score: "magnitude * (1 + persistence)",
            scheduler_inspects_charge_kind: false,
            evidence_pattern: "action blocks: ++-- (randomized sign)",
            witness_samples: WITNESS_SAMPLES,
            resolver_attempt_budget: 1,
        },
        predeclared_gates: Gates {
            priority_commit_rate_gte: MIN_PRIORITY_COMMIT_RATE,
            priority_false_commit_rate_lte: MAX_PRIORITY_FALSE_COMMIT_RATE,
            priority_median_detection_delay_lte: MAX_PRIORITY_MEDIAN_DELAY,
            priority_to_direct_mean_mse_ratio_lte: MAX_PRIORITY_TO_DIRECT_MSE_RATIO,
            priority_to_magnitude_mean_mse_ratio_lte: MAX_PRIORITY_TO_MAGNITUDE_MSE_RATIO,
            priority_to_random_mean_mse_ratio_lte: MAX_PRIORITY_TO_RANDOM_MSE_RATIO,
            priority_to_oldest_mean_mse_ratio_lte: MAX_PRIORITY_TO_OLDEST_MSE_RATIO,
            priority_excess_mean_mse_over_oracle_lte: MAX_PRIORITY_EXCESS_MSE_OVER_ORACLE,
            priority_median_relevant_selection_persistence_gte:
                MIN_PRIORITY_RELEVANT_SELECTION_PERSISTENCE,
            priority_must_beat: vec![
                "magnitude_only_pressure",
                "random_pressure",
                "oldest_first_pressure",
                "no_revision",
            ],
            unverified_commit_rate_must_equal: 0.0,
        },
        aggregate: aggregate_by_strategy,
        derived_checks: Checks {
            priority_commit_rate: priority.commit_rate,
            priority_false_commit_rate: priority.false_commit_rate,
            priority_median_detection_delay: priority_delay,
            priority_to_direct_mean_mse_ratio: priority_to_direct,
            priority_to_magnitude_mean_mse_ratio: priority_to_magnitude,
            priority_to_random_mean_mse_ratio: priority_to_random,
            priority_to_oldest_mean_mse_ratio: priority_to_oldest,
            priority_excess_mean_mse_over_oracle: excess_over_oracle,
            priority_median_relevant_selection_persistence: priority_selection_persistence,
            priority_beats_required_controls: beats_controls,
            all_unverified_commit_rates_zero: all_unverified_zero,
            all_gates_pass: pass,
        },
        runs,
        interpretation_constraints: vec![
            "This is a diagnostic attention-allocation probe, not live runtime scheduling.",
            "The persistence-weighted scheduler is kind-agnostic: it sees magnitude and persistence, not ChargeKind.",
            "Fresh distractors are individually louder than the developmental residual; only persistence can make the relevant pressure win generically.",
            "The direct persistence-weighted control tests whether task utility comes from the generic priority rule rather than uniquely from the Charge container.",
            "A pass would justify held-out transfer of the generic priority rule before any live scheduler promotion.",
        ],
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    if !pass {
        std::process::exit(1);
    }
}
