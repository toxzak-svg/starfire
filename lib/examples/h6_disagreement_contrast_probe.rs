#![allow(dead_code)]

use rand::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use star::causal::CausalEngine;
use star::charge::{
    disagreement_pair_schedule, fit_contrast_from_pairs, fixed_residual_feature_charge,
    knowledge_gap_charge, ontology_feature_charge, prediction_contradiction_charge,
    score_resolution, valid_contrast_pairs, Charge, ChargeKind, ConceptPredicate,
    ContrastProbeConfig, ContrastProbeFit, Direction, EmpiricalInductionConfig,
    EmpiricalOntologyInducer, FixedResidualProjectionConfig, OntologyObservation,
    PromotionCriteria, QuanotTrajectoryEmitter, RelativeImprovementJudge, Resolution,
    VerifierProfile, VerifierTaskClass,
};
use star::cognitive_cycle::{CycleObservationRecorder, JudgedResolverAttempt};
use star::environment::{Environment, ObjectiveFeedback, Step};
use star::metacog::{KnowledgeGap, MetaCognition};
use star::persistence::{Memory, MemoryDomain, Store};
use star::prediction::{ConversationContext, Evidence, PredictionCenter, PredictionOutcome};
use star::quanot::Quanot;
use star::reasoning::ReasoningEngine;

const SEED: u64 = 0x4836_434f_4e54_5241;
const TRAIN_WINDOWS: usize = 2;
const HOLDOUT_WINDOWS: usize = 1;
const TRANSFER_WINDOWS: usize = 4;
const REPEATS_PER_CLASS: usize = 12;
const CLASS_COUNT: usize = 3;
const OBSERVATIONS_PER_WINDOW: usize = REPEATS_PER_CLASS * CLASS_COUNT;
const SOLVE_SCORE: f64 = 0.70;

const MIN_FUTURE_BASELINE_RATIO: f64 = 1.20;
const MIN_CONTROL_EFFICIENCY_RATIO: f64 = 1.10;
const MIN_FUTURE_LEADER_ACCURACY: f64 = 0.75;
const MIN_CONTROL_ACCURACY_MARGIN: f64 = 0.15;
const MIN_WINDOW_WIN_FRACTION: f64 = 1.0;

const RESOLVERS: [&str; 5] = [
    "reasoning",
    "memory",
    "causal",
    "prediction",
    "metacognition",
];

const SURFACE_PREFIXES: [&str; REPEATS_PER_CLASS] = [
    "Briefly,",
    "For a systems note,",
    "Using plain language,",
    "As a direct check,",
    "For an incident review,",
    "Without extra context,",
    "In one technical sentence,",
    "For a verifier,",
    "As a causal check,",
    "For a new operator,",
    "In a compact answer,",
    "For an unfamiliar surface form,",
];

const SURFACE_SUFFIXES: [&str; REPEATS_PER_CLASS] = [
    "Answer directly.",
    "State the mechanism.",
    "Give the core fact.",
    "Avoid analogy.",
    "Use the decisive detail.",
    "Name the relevant process.",
    "Keep the answer concrete.",
    "Focus on the objective claim.",
    "Resolve the uncertainty.",
    "State what actually happens.",
    "Give the shortest correct explanation.",
    "Treat the wording as unseen.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
enum EventClass {
    KnowledgeGap,
    PredictionContradiction,
    QuanotTrajectory,
}

impl EventClass {
    fn all() -> [Self; CLASS_COUNT] {
        [
            Self::KnowledgeGap,
            Self::PredictionContradiction,
            Self::QuanotTrajectory,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::KnowledgeGap => "knowledge_gap",
            Self::PredictionContradiction => "prediction_contradiction",
            Self::QuanotTrajectory => "quanot_trajectory",
        }
    }
}

fn verifier_task_class(class: EventClass) -> VerifierTaskClass {
    match class {
        EventClass::KnowledgeGap => VerifierTaskClass::KnowledgeGap,
        EventClass::PredictionContradiction => VerifierTaskClass::PredictionContradiction,
        EventClass::QuanotTrajectory => VerifierTaskClass::CausalMechanism,
    }
}

#[derive(Debug, Clone, Copy)]
struct TaskFamily {
    name: &'static str,
    gap_topic: &'static str,
    gap_prompt: &'static str,
    gap_target: &'static str,
    contradiction_topic: &'static str,
    contradiction_prompt: &'static str,
    contradiction_target: &'static str,
    trajectory_topic: &'static str,
    trajectory_prompt: &'static str,
    trajectory_target: &'static str,
    lead_a: &'static str,
    lead_b: &'static str,
    trajectory_effect: &'static str,
}

const FAMILIES: [TaskFamily; TRAIN_WINDOWS + HOLDOUT_WINDOWS + TRANSFER_WINDOWS] = [
    TaskFamily {
        name: "dns_copper_cache_miss",
        gap_topic: "dns",
        gap_prompt: "What does DNS do?",
        gap_target: "DNS resolves domain names to IP addresses.",
        contradiction_topic: "copper",
        contradiction_prompt: "Does copper become ferromagnetic at room temperature?",
        contradiction_target: "Copper is not ferromagnetic at room temperature.",
        trajectory_topic: "cache miss",
        trajectory_prompt: "Why does a cache miss increase latency?",
        trajectory_target: "A cache miss causes slower memory fetch and increased latency.",
        lead_a: "The processor requests data from a cache.",
        lead_b: "The requested cache line is absent.",
        trajectory_effect: "slower memory fetch and increased latency",
    },
    TaskFamily {
        name: "mutex_sound_heavy_rain",
        gap_topic: "mutex",
        gap_prompt: "What does a mutex protect?",
        gap_target: "A mutex protects shared state from concurrent access.",
        contradiction_topic: "sound",
        contradiction_prompt: "Does sound travel faster in air than in steel?",
        contradiction_target: "Sound travels faster in steel than in air.",
        trajectory_topic: "heavy rain",
        trajectory_prompt: "Why can heavy rain raise river levels?",
        trajectory_target: "Heavy rain causes increased runoff and higher river levels.",
        lead_a: "Rain continues over already wet ground.",
        lead_b: "The soil absorbs less additional water.",
        trajectory_effect: "increased runoff and higher river levels",
    },
    TaskFamily {
        name: "photosynthesis_moonlight_coolant",
        gap_topic: "photosynthesis",
        gap_prompt: "What gas do plants absorb during photosynthesis?",
        gap_target: "Plants absorb carbon dioxide during photosynthesis.",
        contradiction_topic: "moonlight",
        contradiction_prompt: "Does the Moon produce its own visible light?",
        contradiction_target:
            "The Moon reflects sunlight and does not produce its own visible light.",
        trajectory_topic: "insufficient coolant",
        trajectory_prompt: "Why does insufficient coolant cause thermal throttling?",
        trajectory_target: "Insufficient coolant causes higher temperature and thermal throttling.",
        lead_a: "A processor is operating under sustained load.",
        lead_b: "Cooling capacity falls below heat production.",
        trajectory_effect: "higher temperature and thermal throttling",
    },
    TaskFamily {
        name: "compiler_borrow_checker_packet_loss",
        gap_topic: "compiler",
        gap_prompt: "What does a compiler translate?",
        gap_target: "A compiler translates source code into machine code.",
        contradiction_topic: "borrow checker",
        contradiction_prompt: "Does Rust's borrow checker enforce ownership only at runtime?",
        contradiction_target:
            "Rust's borrow checker enforces many ownership rules at compile time, not only runtime.",
        trajectory_topic: "packet loss",
        trajectory_prompt: "Why does packet loss increase network latency?",
        trajectory_target: "Packet loss causes retransmission and increased latency.",
        lead_a: "A sender transmits a sequence of packets.",
        lead_b: "Some packets fail to reach the receiver.",
        trajectory_effect: "retransmission and increased latency",
    },
    TaskFamily {
        name: "mitochondria_glass_pipe_pressure",
        gap_topic: "mitochondria",
        gap_prompt: "Which organelle produces most cellular ATP?",
        gap_target: "Mitochondria produce most cellular ATP.",
        contradiction_topic: "glass",
        contradiction_prompt: "Is ordinary glass a slowly flowing liquid at room temperature?",
        contradiction_target:
            "Ordinary glass is an amorphous solid, not a slowly flowing liquid at room temperature.",
        trajectory_topic: "low pipe pressure",
        trajectory_prompt: "Why does low pipe pressure reduce flow?",
        trajectory_target: "Low pipe pressure causes reduced flow.",
        lead_a: "Fluid is moving through a pipe.",
        lead_b: "The pressure difference across the pipe falls.",
        trajectory_effect: "reduced flow",
    },
    TaskFamily {
        name: "http404_seasons_combustion",
        gap_topic: "404",
        gap_prompt: "What does HTTP status 404 indicate?",
        gap_target: "HTTP status 404 indicates that a resource was not found.",
        contradiction_topic: "seasons",
        contradiction_prompt:
            "Are Earth's seasons caused mainly by changing distance from the Sun?",
        contradiction_target:
            "Earth's seasons are caused mainly by axial tilt, not changing distance from the Sun.",
        trajectory_topic: "combustion",
        trajectory_prompt: "Why does combustion produce heat?",
        trajectory_target: "Combustion causes heat release.",
        lead_a: "Fuel and oxygen are present together.",
        lead_b: "A combustion reaction begins.",
        trajectory_effect: "heat release",
    },
    TaskFamily {
        name: "index_bats_friction",
        gap_topic: "index",
        gap_prompt: "What is a database index used for?",
        gap_target: "A database index speeds up data lookup.",
        contradiction_topic: "bats",
        contradiction_prompt: "Are bats completely blind?",
        contradiction_target: "Bats are not completely blind and many species can see.",
        trajectory_topic: "friction",
        trajectory_prompt: "Why can friction heat two surfaces?",
        trajectory_target: "Friction causes mechanical energy to become heat.",
        lead_a: "Two surfaces move against each other.",
        lead_b: "Microscopic contact resists their relative motion.",
        trajectory_effect: "mechanical energy to become heat",
    },
];

#[derive(Debug, Clone)]
struct ProbeTask {
    class: EventClass,
    topic: String,
    prompt: String,
    target: String,
    lead_a: String,
    lead_b: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Candidate {
    Reasoning,
    Memory,
    Causal,
    Prediction,
    MetaCognition,
}

impl Candidate {
    fn name(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::Memory => "memory",
            Self::Causal => "causal",
            Self::Prediction => "prediction",
            Self::MetaCognition => "metacognition",
        }
    }
}

const CANDIDATES: [Candidate; 5] = [
    Candidate::Reasoning,
    Candidate::Memory,
    Candidate::Causal,
    Candidate::Prediction,
    Candidate::MetaCognition,
];

#[derive(Debug, Clone)]
struct LabeledObservation {
    variable_charge: Charge,
    fixed_observation: OntologyObservation,
    hidden: EventClass,
}

struct ProbeState {
    store: Store,
    reasoning_memories: Vec<Memory>,
    db_path: PathBuf,
}

impl ProbeState {
    fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = std::env::temp_dir().join(format!(
            "starfire-h6-disagreement-contrast-{}-{}.db",
            std::process::id(),
            star::now_timestamp()
        ));
        remove_sqlite_files(&db_path);
        let store = Store::open(&db_path)?;

        for family in FAMILIES {
            let memory = Memory::new(family.gap_target, MemoryDomain::Empirical, 0.9)
                .with_confidence(0.95)
                .with_provenance("h6-disagreement-contrast-probe");
            store.insert_memory(&memory)?;
        }

        let reasoning_memories = FAMILIES
            .iter()
            .map(|family| {
                Memory::new(family.contradiction_target, MemoryDomain::Empirical, 0.9)
                    .with_confidence(0.95)
                    .with_provenance("h6-disagreement-contrast-probe")
            })
            .collect();

        Ok(Self {
            store,
            reasoning_memories,
            db_path,
        })
    }
}

impl Drop for ProbeState {
    fn drop(&mut self) {
        remove_sqlite_files(&self.db_path);
    }
}

#[derive(Debug, Clone)]
struct TargetVerifierEnvironment {
    prompt: String,
    target: String,
    class: EventClass,
    progress: f64,
    solved: bool,
    evidence: Vec<String>,
}

impl TargetVerifierEnvironment {
    fn new(task: &ProbeTask) -> Self {
        Self {
            prompt: task.prompt.clone(),
            target: task.target.clone(),
            class: task.class,
            progress: 0.0,
            solved: false,
            evidence: vec!["episode reset before resolver action".into()],
        }
    }
}

impl Environment for TargetVerifierEnvironment {
    type Action = String;
    type Observation = String;

    fn reset(&mut self, seed: u64) -> Self::Observation {
        self.progress = 0.0;
        self.solved = false;
        self.evidence = vec![format!("deterministic verifier episode seed={seed}")];
        self.prompt.clone()
    }

    fn available_actions(&self) -> Vec<Self::Action> {
        Vec::new()
    }

    fn act(&mut self, action: &Self::Action) -> Step<Self::Observation> {
        self.progress = score_resolution(
            action,
            &self.target,
            verifier_task_class(self.class),
            VerifierProfile::TaskProfiled,
        );
        self.solved = self.progress >= SOLVE_SCORE;
        self.evidence = vec![format!(
            "target verifier profile=TaskProfiled measured coverage={:.6} solved={}",
            self.progress, self.solved
        )];
        Step::new(action.clone(), 1, true)
    }

    fn objective_feedback(&self) -> ObjectiveFeedback {
        ObjectiveFeedback::new(self.progress, self.solved, self.evidence.clone())
    }
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    seed: u64,
    pair_interactions: usize,
    min_preference_disagreement: f64,
    min_partition_support: usize,
    min_holdout_support: usize,
    complexity_penalty: f64,
    min_holdout_gain: f64,
    min_future_baseline_ratio: f64,
    min_control_efficiency_ratio: f64,
    min_future_leader_accuracy: f64,
    min_control_accuracy_margin: f64,
    min_window_win_fraction: f64,
}

#[derive(Debug, Serialize)]
struct CohortReport {
    total_real_emitter_observations: usize,
    total_judged_cycle_attempts: usize,
    excluded_by_frozen_h4_memory_predicate: usize,
    retained_non_memory: usize,
    train_non_memory: usize,
    holdout_non_memory: usize,
    future_non_memory: usize,
    excluded_hidden_distribution: BTreeMap<EventClass, usize>,
    retained_hidden_distribution: BTreeMap<EventClass, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct WindowReport {
    index: usize,
    family: &'static str,
    observations: usize,
    efficiency: f64,
    baseline_efficiency: f64,
    baseline_ratio: f64,
    leader_accuracy: f64,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    applied: bool,
    proposal_evaluations: usize,
    unique_pair_evaluations: Option<usize>,
    efficiency: f64,
    baseline_efficiency: f64,
    baseline_ratio: f64,
    leader_accuracy: f64,
    window_win_fraction: f64,
    worst_window_ratio: f64,
    windows: Vec<WindowReport>,
}

#[derive(Debug, Serialize)]
struct ProbeReport {
    source_pair_count: usize,
    mean_source_preference_disagreement: f64,
    dominant_eigenvalue_fraction: f64,
    axis_width: usize,
    threshold: f32,
    lower_resolver: String,
    upper_resolver: String,
    lower_training_support: usize,
    upper_training_support: usize,
    lower_holdout_support: usize,
    upper_holdout_support: usize,
    training_efficiency: f64,
    holdout_efficiency: f64,
    holdout_gain: f64,
    future_side_hidden_distribution: BTreeMap<String, BTreeMap<EventClass, usize>>,
}

#[derive(Debug, Serialize)]
struct FitDiagnostics {
    baseline_training_efficiency: f64,
    baseline_holdout_efficiency: f64,
    best_training_efficiency: f64,
    best_training_gain_after_penalty: f64,
}

#[derive(Debug, Serialize)]
struct StaticOntologyReport {
    promoted_concepts: usize,
    candidates_considered: usize,
    routes: Vec<String>,
    metrics: PathMetrics,
}

#[derive(Debug, Serialize)]
struct GateReport {
    real_probe_applied: bool,
    future_baseline_ratio: bool,
    all_future_windows_win: bool,
    future_leader_accuracy: bool,
    beats_random_pair_control: bool,
    beats_shuffled_outcome_control: bool,
    beats_permuted_residual_control: bool,
    control_accuracy_margin: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.real_probe_applied
            && self.future_baseline_ratio
            && self.all_future_windows_win
            && self.future_leader_accuracy
            && self.beats_random_pair_control
            && self.beats_shuffled_outcome_control
            && self.beats_permuted_residual_control
            && self.control_accuracy_margin
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    observation_source: &'static str,
    visible_charge_kind: &'static str,
    representation_boundary: &'static str,
    primitive_operation: &'static str,
    frozen_contract: FrozenContract,
    cohort: CohortReport,
    parent_baseline_resolver: String,
    real_fit_diagnostics: FitDiagnostics,
    real_primitive: PathMetrics,
    real_probe: Option<ProbeReport>,
    matched_random_pair_fit_diagnostics: FitDiagnostics,
    matched_random_pair_control: PathMetrics,
    shuffled_outcome_pair_fit_diagnostics: FitDiagnostics,
    shuffled_outcome_pair_control: PathMetrics,
    independently_permuted_residual_fit_diagnostics: FitDiagnostics,
    independently_permuted_residual_control: PathMetrics,
    best_existing_static_h5_path: StaticOntologyReport,
    gates: GateReport,
    primary_conclusion: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = frozen_contrast_config();
    let state = ProbeState::new()?;
    let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
    let fixed_config = FixedResidualProjectionConfig::default();
    let mut next_id = 1u64;
    let mut emitter_counts = BTreeMap::<String, usize>::new();
    let mut total_judged_cycle_attempts = 0usize;

    let mut windows = Vec::<Vec<LabeledObservation>>::new();
    for family in FAMILIES {
        let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
        for class in EventClass::all() {
            for repeat in 0..REPEATS_PER_CLASS {
                let task = surface_task(family, class, repeat);
                let mut charge = emit_real_charge(&task)?;
                *emitter_counts.entry(class.name().to_string()).or_default() += 1;
                charge.kind = ChargeKind::Custom("unresolved".into());
                charge.id = next_id;
                next_id = next_id.saturating_add(1);

                let variable_charge = ontology_feature_charge(&charge);
                let fixed_charge = fixed_residual_feature_charge(&charge, fixed_config);
                let attempts = judged_component_attempts(&fixed_charge, &task, &state)?;
                total_judged_cycle_attempts += attempts.len();
                let fixed_observation = recorder.record(fixed_charge, &attempts)?;
                window.push(LabeledObservation {
                    variable_charge,
                    fixed_observation,
                    hidden: class,
                });
            }
        }
        windows.push(window);
    }

    let (
        non_memory_windows,
        excluded,
        retained,
        excluded_hidden_distribution,
        retained_hidden_distribution,
    ) = non_memory_windows(&windows);

    let train = flatten_windows(&non_memory_windows[..TRAIN_WINDOWS]);
    let holdout = flatten_windows(
        &non_memory_windows[TRAIN_WINDOWS..TRAIN_WINDOWS + HOLDOUT_WINDOWS],
    );
    let future_labeled = &non_memory_windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..];
    let future = plain_windows(future_labeled);

    let mut real_pairs =
        disagreement_pair_schedule(&train, config.min_preference_disagreement);
    real_pairs.truncate(config.max_pair_interactions);
    if real_pairs.is_empty() {
        return Err("real disagreement schedule produced zero interactions".into());
    }
    let real_fit = fit_contrast_from_pairs(&train, &holdout, &real_pairs, config)?;
    let parent_baseline_resolver = real_fit.baseline_resolver.clone();
    let real_metrics = evaluate_path(
        &future,
        &parent_baseline_resolver,
        real_fit.applied(),
        real_fit.proposal_evaluations,
        Some(unique_pair_count(&real_pairs)),
        |observation| real_fit.route(&observation.charge).to_string(),
    );

    let random_pairs = random_pair_schedule(&train, real_pairs.len(), SEED ^ 0x5241_4e44)?;
    let random_fit = fit_contrast_from_pairs(&train, &holdout, &random_pairs, config)?;
    let random_metrics = evaluate_path(
        &future,
        &parent_baseline_resolver,
        random_fit.applied(),
        random_fit.proposal_evaluations,
        Some(unique_pair_count(&random_pairs)),
        |observation| random_fit.route(&observation.charge).to_string(),
    );

    let shuffled_pairs = shuffled_outcome_pair_schedule(
        &train,
        real_pairs.len(),
        config.min_preference_disagreement,
        SEED ^ 0x4f55_5443,
    )?;
    let shuffled_fit = fit_contrast_from_pairs(&train, &holdout, &shuffled_pairs, config)?;
    let shuffled_metrics = evaluate_path(
        &future,
        &parent_baseline_resolver,
        shuffled_fit.applied(),
        shuffled_fit.proposal_evaluations,
        Some(unique_pair_count(&shuffled_pairs)),
        |observation| shuffled_fit.route(&observation.charge).to_string(),
    );

    let mut permuted_train = train.clone();
    let mut permuted_holdout = holdout.clone();
    let mut permuted_future = future.clone();
    let mut residual_rng = StdRng::seed_from_u64(SEED ^ 0x5245_5349);
    permute_visible_features(&mut permuted_train, &mut residual_rng);
    permute_visible_features(&mut permuted_holdout, &mut residual_rng);
    for window in &mut permuted_future {
        permute_visible_features(window, &mut residual_rng);
    }
    let permuted_fit =
        fit_contrast_from_pairs(&permuted_train, &permuted_holdout, &real_pairs, config)?;
    let permuted_metrics = evaluate_path(
        &permuted_future,
        &permuted_fit.baseline_resolver,
        permuted_fit.applied(),
        permuted_fit.proposal_evaluations,
        Some(unique_pair_count(&real_pairs)),
        |observation| permuted_fit.route(&observation.charge).to_string(),
    );

    let static_ontology =
        EmpiricalOntologyInducer::new(frozen_static_h5_config()).fit(&train, &holdout)?;
    let static_metrics = evaluate_path(
        &future,
        &parent_baseline_resolver,
        static_ontology.summary().promoted_concepts > 0,
        static_ontology.summary().candidates_considered,
        None,
        |observation| static_ontology.route(&observation.charge).resolver,
    );
    let static_report = StaticOntologyReport {
        promoted_concepts: static_ontology.summary().promoted_concepts,
        candidates_considered: static_ontology.summary().candidates_considered,
        routes: static_ontology
            .routes()
            .iter()
            .map(|route| format!("{:?} -> {}", route.concept.predicate, route.resolver))
            .collect(),
        metrics: static_metrics,
    };

    let max_control_accuracy = random_metrics
        .leader_accuracy
        .max(shuffled_metrics.leader_accuracy)
        .max(permuted_metrics.leader_accuracy);
    let gates = GateReport {
        real_probe_applied: real_fit.applied(),
        future_baseline_ratio: real_metrics.baseline_ratio + 1e-12
            >= MIN_FUTURE_BASELINE_RATIO,
        all_future_windows_win: real_metrics.window_win_fraction + 1e-12
            >= MIN_WINDOW_WIN_FRACTION,
        future_leader_accuracy: real_metrics.leader_accuracy + 1e-12
            >= MIN_FUTURE_LEADER_ACCURACY,
        beats_random_pair_control: real_metrics.efficiency + 1e-12
            >= MIN_CONTROL_EFFICIENCY_RATIO * random_metrics.efficiency,
        beats_shuffled_outcome_control: real_metrics.efficiency + 1e-12
            >= MIN_CONTROL_EFFICIENCY_RATIO * shuffled_metrics.efficiency,
        beats_permuted_residual_control: real_metrics.efficiency + 1e-12
            >= MIN_CONTROL_EFFICIENCY_RATIO * permuted_metrics.efficiency,
        control_accuracy_margin: real_metrics.leader_accuracy + 1e-12
            >= max_control_accuracy + MIN_CONTROL_ACCURACY_MARGIN,
    };

    let primary_conclusion = if gates.all_pass() {
        "CAUSAL REASONING EFFECT DETECTED"
    } else if real_fit.applied()
        && real_metrics.baseline_ratio >= 1.05
        && real_metrics.window_win_fraction >= 0.75
    {
        "TRANSFER DETECTED, CAUSALITY UNCLEAR"
    } else if real_fit.applied() {
        "MECHANISM DETECTED, NOT TRANSFERRED"
    } else {
        "REJECTED"
    };

    let real_probe = real_fit.probe.as_ref().map(|probe| ProbeReport {
        source_pair_count: probe.contrast.source_pair_count,
        mean_source_preference_disagreement: probe
            .contrast
            .mean_source_preference_disagreement,
        dominant_eigenvalue_fraction: probe.contrast.dominant_eigenvalue_fraction,
        axis_width: probe.contrast.axis.len(),
        threshold: probe.contrast.threshold,
        lower_resolver: probe.lower_resolver.clone(),
        upper_resolver: probe.upper_resolver.clone(),
        lower_training_support: probe.lower_training_support,
        upper_training_support: probe.upper_training_support,
        lower_holdout_support: probe.lower_holdout_support,
        upper_holdout_support: probe.upper_holdout_support,
        training_efficiency: probe.training_efficiency,
        holdout_efficiency: probe.holdout_efficiency,
        holdout_gain: probe.holdout_gain,
        future_side_hidden_distribution: future_side_hidden_distribution(
            future_labeled,
            &probe.contrast,
        ),
    });

    let report = Report {
        experiment: "H6 disagreement-induced tension mode probe",
        observation_source: "real Starfire subsystem outputs -> task-profiled Environment feedback -> OutcomeWitness -> RelativeImprovementJudge -> CognitiveCycleState",
        visible_charge_kind: "Custom(unresolved)",
        representation_boundary: "H5 fixed-width mask-blind residual projection after frozen H4 memory-cohort exclusion",
        primitive_operation: "resolver-disagreement-selected residual pairs accumulate normalized displacement outer products into one second-moment state M; the dominant mode of M becomes a new projection axis and the mean projected pair midpoint becomes its threshold; side resolvers are relearned on the full training cohort and independently holdout-gated",
        frozen_contract: FrozenContract {
            seed: SEED,
            pair_interactions: config.max_pair_interactions,
            min_preference_disagreement: config.min_preference_disagreement,
            min_partition_support: config.min_partition_support,
            min_holdout_support: config.min_holdout_support,
            complexity_penalty: config.complexity_penalty,
            min_holdout_gain: config.min_holdout_gain,
            min_future_baseline_ratio: MIN_FUTURE_BASELINE_RATIO,
            min_control_efficiency_ratio: MIN_CONTROL_EFFICIENCY_RATIO,
            min_future_leader_accuracy: MIN_FUTURE_LEADER_ACCURACY,
            min_control_accuracy_margin: MIN_CONTROL_ACCURACY_MARGIN,
            min_window_win_fraction: MIN_WINDOW_WIN_FRACTION,
        },
        cohort: CohortReport {
            total_real_emitter_observations: windows.iter().map(Vec::len).sum(),
            total_judged_cycle_attempts,
            excluded_by_frozen_h4_memory_predicate: excluded,
            retained_non_memory: retained,
            train_non_memory: train.len(),
            holdout_non_memory: holdout.len(),
            future_non_memory: future.iter().map(Vec::len).sum(),
            excluded_hidden_distribution,
            retained_hidden_distribution,
        },
        parent_baseline_resolver,
        real_fit_diagnostics: fit_diagnostics(&real_fit),
        real_primitive: real_metrics,
        real_probe,
        matched_random_pair_fit_diagnostics: fit_diagnostics(&random_fit),
        matched_random_pair_control: random_metrics,
        shuffled_outcome_pair_fit_diagnostics: fit_diagnostics(&shuffled_fit),
        shuffled_outcome_pair_control: shuffled_metrics,
        independently_permuted_residual_fit_diagnostics: fit_diagnostics(&permuted_fit),
        independently_permuted_residual_control: permuted_metrics,
        best_existing_static_h5_path: static_report,
        gates,
        primary_conclusion,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn fit_diagnostics(fit: &ContrastProbeFit) -> FitDiagnostics {
    FitDiagnostics {
        baseline_training_efficiency: fit.baseline_training_efficiency,
        baseline_holdout_efficiency: fit.baseline_holdout_efficiency,
        best_training_efficiency: fit.best_training_efficiency,
        best_training_gain_after_penalty: fit.best_training_gain_after_penalty,
    }
}

fn frozen_contrast_config() -> ContrastProbeConfig {
    ContrastProbeConfig {
        min_preference_disagreement: 0.25,
        max_pair_interactions: 64,
        min_partition_support: 12,
        min_holdout_support: 6,
        complexity_penalty: 0.003,
        min_holdout_gain: 0.04,
    }
}

fn frozen_static_h5_config() -> EmpiricalInductionConfig {
    EmpiricalInductionConfig {
        max_concepts: 2,
        min_partition_support: 16,
        min_holdout_support: 8,
        max_thresholds_per_dimension: 96,
        complexity_penalty: 0.003,
        promotion: PromotionCriteria {
            min_observations: 16,
            min_holdout_gain: 0.04,
            min_total_utility_gain: 0.04,
        },
    }
}

fn h4_memory_predicate() -> ConceptPredicate {
    ConceptPredicate::ResidualThreshold {
        dimension: 2,
        threshold: 0.171875,
        direction: Direction::AtMost,
    }
}

fn non_memory_windows(
    windows: &[Vec<LabeledObservation>],
) -> (
    Vec<Vec<LabeledObservation>>,
    usize,
    usize,
    BTreeMap<EventClass, usize>,
    BTreeMap<EventClass, usize>,
) {
    let predicate = h4_memory_predicate();
    let mut excluded = 0usize;
    let mut retained = 0usize;
    let mut excluded_hidden = BTreeMap::new();
    let mut retained_hidden = BTreeMap::new();
    let filtered = windows
        .iter()
        .map(|window| {
            window
                .iter()
                .filter_map(|observation| {
                    if predicate.matches(&observation.variable_charge) {
                        excluded += 1;
                        *excluded_hidden.entry(observation.hidden).or_default() += 1;
                        None
                    } else {
                        retained += 1;
                        *retained_hidden.entry(observation.hidden).or_default() += 1;
                        Some(observation.clone())
                    }
                })
                .collect()
        })
        .collect();
    (
        filtered,
        excluded,
        retained,
        excluded_hidden,
        retained_hidden,
    )
}

fn surface_task(family: TaskFamily, class: EventClass, repeat: usize) -> ProbeTask {
    let prefix = SURFACE_PREFIXES[repeat];
    let suffix = SURFACE_SUFFIXES[repeat];
    let decorate = |text: &str| format!("{prefix} {text} {suffix}");

    match class {
        EventClass::KnowledgeGap => ProbeTask {
            class,
            topic: family.gap_topic.to_string(),
            prompt: decorate(family.gap_prompt),
            target: family.gap_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::PredictionContradiction => ProbeTask {
            class,
            topic: family.contradiction_topic.to_string(),
            prompt: decorate(family.contradiction_prompt),
            target: family.contradiction_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::QuanotTrajectory => ProbeTask {
            class,
            topic: family.trajectory_topic.to_string(),
            prompt: decorate(family.trajectory_prompt),
            target: family.trajectory_target.to_string(),
            lead_a: format!("{prefix} {}", family.lead_a),
            lead_b: format!("{} {suffix}", family.lead_b),
        },
    }
}

fn emit_real_charge(task: &ProbeTask) -> Result<Charge, Box<dyn Error>> {
    match task.class {
        EventClass::KnowledgeGap => {
            let mut metacog = MetaCognition::new();
            metacog.note_gap(KnowledgeGap::new(&task.topic, 0.85));
            let gap = metacog
                .top_gap()
                .ok_or("metacognition did not retain gap")?;
            knowledge_gap_charge(gap).ok_or_else(|| "gap emitter returned no charge".into())
        }
        EventClass::PredictionContradiction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(&task.prompt);
            let mut center = PredictionCenter::new();
            let context = conversation_context(task, Some(state.reservoir_state));
            let predictions = center.generate(&context);
            let prediction = predictions
                .first()
                .ok_or("prediction center emitted no prediction")?;
            let charge = prediction_contradiction_charge(
                prediction,
                PredictionOutcome::Refuted,
                &task.target,
            )
            .ok_or("contradiction emitter returned no charge")?;
            center.update_with_evidence(&Evidence {
                outcome: PredictionOutcome::Refuted,
                prediction_id: prediction.id,
            });
            Ok(charge)
        }
        EventClass::QuanotTrajectory => {
            let mut quanot = Quanot::new(32, 64);
            let mut emitter = QuanotTrajectoryEmitter::new();
            let first = quanot.process(&task.lead_a);
            let second = quanot.process(&task.lead_b);
            let third = quanot.process(&task.prompt);
            let _ = emitter.observe(&first);
            let _ = emitter.observe(&second);
            emitter
                .observe(&third)
                .ok_or_else(|| "Quanot trajectory emitter returned no charge".into())
        }
    }
}

fn judged_component_attempts(
    charge: &Charge,
    task: &ProbeTask,
    state: &ProbeState,
) -> Result<Vec<JudgedResolverAttempt>, Box<dyn Error>> {
    let mut attempts = Vec::with_capacity(CANDIDATES.len());
    for (candidate_index, candidate) in CANDIDATES.iter().enumerate() {
        let output = resolve_component(*candidate, task, state)?;
        let mut environment = TargetVerifierEnvironment::new(task);
        let _ = environment.reset(charge.id ^ candidate_index as u64);
        let before = environment.objective_feedback();
        let step = environment.act(&output);
        let after = environment.objective_feedback();
        let resolution = Resolution {
            discharged: charge.magnitude,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost: step.action_cost.max(1),
        };
        let witness = star::charge::OutcomeWitness::new(
            "objective_progress",
            before.progress,
            after.progress,
            star::charge::ImprovementDirection::HigherIsBetter,
            after.evidence,
        );
        attempts.push(JudgedResolverAttempt::new(
            candidate.name(),
            resolution,
            witness,
        ));
    }
    Ok(attempts)
}

fn resolve_component(
    candidate: Candidate,
    task: &ProbeTask,
    state: &ProbeState,
) -> Result<String, Box<dyn Error>> {
    match candidate {
        Candidate::Reasoning => {
            let mut engine = ReasoningEngine::new();
            let result = engine.reason(&task.prompt, &state.reasoning_memories);
            let mut output = result.answer.unwrap_or_default();
            if !result.reasoning_chain.is_empty() {
                output.push(' ');
                output.push_str(&result.reasoning_chain.join(" "));
            }
            Ok(output)
        }
        Candidate::Memory => {
            let memories = state.store.search_memories(&task.topic, 3, None)?;
            Ok(memories
                .iter()
                .map(|memory| memory.content.as_str())
                .collect::<Vec<_>>()
                .join(" "))
        }
        Candidate::Causal => Ok(resolve_causal(task)),
        Candidate::Prediction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(&task.prompt);
            let mut center = PredictionCenter::new();
            let context = conversation_context(task, Some(state.reservoir_state));
            let predictions = center.generate(&context);
            Ok(predictions
                .iter()
                .take(3)
                .map(|prediction| prediction.description.as_str())
                .collect::<Vec<_>>()
                .join(" "))
        }
        Candidate::MetaCognition => {
            let mut metacog = MetaCognition::new();
            metacog.note_curiosity(&task.topic, &task.prompt);
            Ok(metacog
                .curiosity_question(&task.topic)
                .map(|intent| intent.format())
                .unwrap_or_default())
        }
    }
}

fn resolve_causal(task: &ProbeTask) -> String {
    let mut engine = CausalEngine::new();
    for family in FAMILIES {
        engine.add_edge(
            family.trajectory_topic,
            family.trajectory_effect,
            0.9,
            Some(1),
        );
    }

    let prompt_tokens = token_set(&task.prompt);
    let mut ranked: Vec<(usize, String)> = engine
        .edges()
        .values()
        .map(|edge| {
            let text = format!("{} causes {}.", edge.cause, edge.effect);
            let overlap = token_set(&text).intersection(&prompt_tokens).count();
            (overlap, text)
        })
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    ranked
        .into_iter()
        .filter(|(overlap, _)| *overlap > 0)
        .take(2)
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn conversation_context(task: &ProbeTask, quanot_state: Option<Vec<f64>>) -> ConversationContext {
    let mut context = ConversationContext::new(task.topic.clone(), 2, quanot_state, Some(0.5));
    context.recent_text = vec![task.prompt.clone()];
    context.discussed_entities = token_set(&task.prompt).into_iter().take(8).collect();
    context
}

fn token_set(text: &str) -> HashSet<String> {
    const STOPWORDS: [&str; 25] = [
        "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "for", "from", "in", "is",
        "it", "of", "on", "only", "the", "to", "used", "what", "which", "why", "with",
    ];

    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '\'')
        .map(|token| token.trim_matches('\'').to_ascii_lowercase())
        .filter(|token| token.len() > 1 && !STOPWORDS.contains(&token.as_str()))
        .collect()
}

fn random_pair_schedule(
    train: &[OntologyObservation],
    budget: usize,
    seed: u64,
) -> Result<Vec<(usize, usize)>, Box<dyn Error>> {
    let mut pairs = valid_contrast_pairs(train);
    if pairs.len() < budget {
        return Err(format!(
            "random-pair control has only {} valid pairs for budget {budget}",
            pairs.len()
        )
        .into());
    }
    let mut rng = StdRng::seed_from_u64(seed);
    pairs.shuffle(&mut rng);
    pairs.truncate(budget);
    Ok(pairs)
}

fn shuffled_outcome_pair_schedule(
    train: &[OntologyObservation],
    budget: usize,
    min_disagreement: f64,
    seed: u64,
) -> Result<Vec<(usize, usize)>, Box<dyn Error>> {
    let mut source = train.to_vec();
    let mut outcomes: Vec<_> = source
        .iter()
        .map(|observation| observation.outcomes.clone())
        .collect();
    let mut rng = StdRng::seed_from_u64(seed);
    outcomes.shuffle(&mut rng);
    for (observation, outcomes) in source.iter_mut().zip(outcomes) {
        observation.outcomes = outcomes;
    }

    let mut pairs = disagreement_pair_schedule(&source, min_disagreement);
    if pairs.len() < budget {
        return Err(format!(
            "shuffled-outcome control has only {} disagreement pairs for budget {budget}",
            pairs.len()
        )
        .into());
    }
    pairs.truncate(budget);
    Ok(pairs)
}

fn permute_visible_features(observations: &mut [OntologyObservation], rng: &mut StdRng) {
    let dimensions = observations
        .iter()
        .map(|observation| observation.charge.residual.len())
        .max()
        .unwrap_or(0);
    for dimension in 0..dimensions {
        let indices: Vec<usize> = observations
            .iter()
            .enumerate()
            .filter(|(_, observation)| observation.charge.residual.len() > dimension)
            .map(|(index, _)| index)
            .collect();
        let mut values: Vec<f32> = indices
            .iter()
            .map(|index| observations[*index].charge.residual[dimension])
            .collect();
        values.shuffle(rng);
        for (index, value) in indices.into_iter().zip(values) {
            observations[index].charge.residual[dimension] = value;
        }
    }
}

fn evaluate_path<F>(
    future: &[Vec<OntologyObservation>],
    baseline_resolver: &str,
    applied: bool,
    proposal_evaluations: usize,
    unique_pair_evaluations: Option<usize>,
    route: F,
) -> PathMetrics
where
    F: Fn(&OntologyObservation) -> String,
{
    let mut windows = Vec::new();
    let mut routed_total = 0.0;
    let mut baseline_total = 0.0;
    let mut correct = 0usize;
    let mut observations = 0usize;

    for (index, window) in future.iter().enumerate() {
        let efficiency = mean_named_efficiency(window, |observation| route(observation));
        let baseline_efficiency =
            mean_named_efficiency(window, |_| baseline_resolver.to_string());
        let leader_accuracy = leader_accuracy(window, &route);
        routed_total += efficiency * window.len() as f64;
        baseline_total += baseline_efficiency * window.len() as f64;
        correct += (leader_accuracy * window.len() as f64).round() as usize;
        observations += window.len();
        windows.push(WindowReport {
            index,
            family: FAMILIES[TRAIN_WINDOWS + HOLDOUT_WINDOWS + index].name,
            observations: window.len(),
            efficiency,
            baseline_efficiency,
            baseline_ratio: ratio(efficiency, baseline_efficiency),
            leader_accuracy,
        });
    }

    let efficiency = routed_total / observations.max(1) as f64;
    let baseline_efficiency = baseline_total / observations.max(1) as f64;
    let window_win_fraction = windows
        .iter()
        .filter(|window| window.efficiency > window.baseline_efficiency + 1e-12)
        .count() as f64
        / windows.len().max(1) as f64;
    let worst_window_ratio = windows
        .iter()
        .map(|window| window.baseline_ratio)
        .fold(f64::INFINITY, f64::min);

    PathMetrics {
        applied,
        proposal_evaluations,
        unique_pair_evaluations,
        efficiency,
        baseline_efficiency,
        baseline_ratio: ratio(efficiency, baseline_efficiency),
        leader_accuracy: correct as f64 / observations.max(1) as f64,
        window_win_fraction,
        worst_window_ratio,
        windows,
    }
}

fn leader_accuracy<F>(observations: &[OntologyObservation], route: &F) -> f64
where
    F: Fn(&OntologyObservation) -> String,
{
    observations
        .iter()
        .filter(|observation| route(observation) == best_observation_resolver(observation))
        .count() as f64
        / observations.len().max(1) as f64
}

fn best_observation_resolver(observation: &OntologyObservation) -> String {
    RESOLVERS
        .iter()
        .max_by(|left, right| {
            resolver_score(observation, left)
                .partial_cmp(&resolver_score(observation, right))
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.cmp(left))
        })
        .copied()
        .unwrap_or_default()
        .to_string()
}

fn mean_named_efficiency<F>(observations: &[OntologyObservation], route: F) -> f64
where
    F: Fn(&OntologyObservation) -> String,
{
    observations
        .iter()
        .map(|observation| resolver_score(observation, &route(observation)))
        .sum::<f64>()
        / observations.len().max(1) as f64
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

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 1e-12 {
        if numerator <= 1e-12 {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        numerator / denominator
    }
}

fn unique_pair_count(pairs: &[(usize, usize)]) -> usize {
    pairs.iter().copied().collect::<BTreeSet<_>>().len()
}

fn future_side_hidden_distribution(
    future: &[Vec<LabeledObservation>],
    contrast: &star::charge::TensionContrast,
) -> BTreeMap<String, BTreeMap<EventClass, usize>> {
    let mut report = BTreeMap::<String, BTreeMap<EventClass, usize>>::new();
    for observation in future.iter().flat_map(|window| window.iter()) {
        let side = match contrast.side(&observation.fixed_observation.charge) {
            Some(star::charge::ProbeSide::Lower) => "lower",
            Some(star::charge::ProbeSide::Upper) => "upper",
            None => "unprojectable",
        };
        *report
            .entry(side.to_string())
            .or_default()
            .entry(observation.hidden)
            .or_default() += 1;
    }
    report
}

fn flatten_windows(windows: &[Vec<LabeledObservation>]) -> Vec<OntologyObservation> {
    windows
        .iter()
        .flat_map(|window| window.iter())
        .map(|observation| observation.fixed_observation.clone())
        .collect()
}

fn plain_windows(windows: &[Vec<LabeledObservation>]) -> Vec<Vec<OntologyObservation>> {
    windows
        .iter()
        .map(|window| {
            window
                .iter()
                .map(|observation| observation.fixed_observation.clone())
                .collect()
        })
        .collect()
}

fn remove_sqlite_files(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
