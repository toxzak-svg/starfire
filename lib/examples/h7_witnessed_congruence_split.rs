#![allow(dead_code)]

use rand::prelude::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use star::causal::CausalEngine;
use star::charge::{
    directed_normalized_motion, fit_witnessed_congruence_split, fixed_residual_feature_charge,
    knowledge_gap_charge, ontology_feature_charge, prediction_contradiction_charge,
    quantize_terminal_witness, score_resolution, Charge, ChargeKind, ConceptPredicate,
    CongruenceSplitConfig, CongruenceSplitFit, ContinuationObservation, Direction, DischargeJudge,
    FixedResidualProjectionConfig, ImprovementDirection, OntologyObservation, OutcomeWitness,
    QuanotTrajectoryEmitter, RelativeImprovementJudge, Resolution, ResolverOutcome, ResolverWord,
    TerminalDisposition, TerminalWitness, VerifierProfile, VerifierTaskClass,
    WitnessedCongruenceObserver,
};
use star::cognitive_cycle::{CognitiveCycleState, CycleObservationRecorder, JudgedResolverAttempt};
use star::environment::{Environment, ObjectiveFeedback, Step};
use star::metacog::{KnowledgeGap, MetaCognition};
use star::persistence::{Memory, MemoryDomain, Store};
use star::prediction::{ConversationContext, Evidence, PredictionCenter, PredictionOutcome};
use star::quanot::Quanot;
use star::reasoning::ReasoningEngine;

const SEED: u64 = 0x4837_5743_5350_4c54;
const TRAIN_WINDOWS: usize = 2;
const HOLDOUT_WINDOWS: usize = 1;
const TRANSFER_WINDOWS: usize = 4;
const REPEATS_PER_CLASS: usize = 12;
const CLASS_COUNT: usize = 3;
const OBSERVATIONS_PER_WINDOW: usize = REPEATS_PER_CLASS * CLASS_COUNT;
const SOLVE_SCORE: f64 = 0.70;

const MIN_FUTURE_DEFECT_GAIN: f64 = 0.10;
const MAX_FUTURE_DEFECT_RATIO: f64 = 0.80;
const MIN_FUTURE_WINDOW_WINS: usize = 4;
const MIN_CONTROL_DEFECT_MARGIN: f64 = 0.05;

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
        contradiction_target: "The Moon reflects sunlight and does not produce its own visible light.",
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
        contradiction_target: "Rust's borrow checker enforces many ownership rules at compile time, not only runtime.",
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
        contradiction_target: "Ordinary glass is an amorphous solid, not a slowly flowing liquid at room temperature.",
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
        contradiction_prompt: "Are Earth's seasons caused mainly by changing distance from the Sun?",
        contradiction_target: "Earth's seasons are caused mainly by axial tilt, not changing distance from the Sun.",
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
struct OneStepMeasurement {
    terminal: TerminalWitness,
    accepted_fraction: f64,
    compute_cost: u64,
}

#[derive(Debug, Clone)]
struct AnchorObservation {
    variable_charge: Charge,
    fixed_charge: Charge,
    fixed_observation: OntologyObservation,
    hidden: EventClass,
    task: ProbeTask,
    one_step: Vec<OneStepMeasurement>,
    continuations: Vec<ContinuationObservation>,
}

struct ProbeState {
    store: Store,
    reasoning_memories: Vec<Memory>,
    db_path: PathBuf,
}

impl ProbeState {
    fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = std::env::temp_dir().join(format!(
            "starfire-h7-witnessed-congruence-{}-{}.db",
            std::process::id(),
            star::now_timestamp()
        ));
        remove_sqlite_files(&db_path);
        let store = Store::open(&db_path)?;
        for family in FAMILIES {
            let memory = Memory::new(family.gap_target, MemoryDomain::Empirical, 0.9)
                .with_confidence(0.95)
                .with_provenance("h7-witnessed-congruence-split");
            store.insert_memory(&memory)?;
        }
        let reasoning_memories = FAMILIES
            .iter()
            .map(|family| {
                Memory::new(family.contradiction_target, MemoryDomain::Empirical, 0.9)
                    .with_confidence(0.95)
                    .with_provenance("h7-witnessed-congruence-split")
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
            evidence: vec!["episode reset before resolver word".into()],
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
    max_context_length: usize,
    allow_self_repeat: bool,
    proposal_budget: usize,
    max_promoted_splits: usize,
    max_signature_classes: usize,
    witness_deadzone: f64,
    witness_strong_boundary: f64,
    min_train_signature_support: usize,
    min_holdout_signature_support: usize,
    complexity_penalty: f64,
    min_train_defect_gain_after_penalty: f64,
    min_holdout_defect_gain: f64,
    max_holdout_defect_ratio: f64,
    min_future_defect_gain: f64,
    max_future_defect_ratio: f64,
    min_future_window_wins: usize,
    min_control_defect_margin: f64,
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
    sequential_continuation_executions: usize,
    actual_sequential_compute_cost: u64,
}

#[derive(Debug, Clone, Serialize)]
struct DefectMetrics {
    applied: bool,
    parent_defect: f64,
    split_defect: f64,
    defect_gain: f64,
    defect_ratio: f64,
    window_wins: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    candidate_exists: bool,
    promotion_holdout_passed: bool,
    future_defect_gain: bool,
    future_defect_ratio: bool,
    all_future_windows_win: bool,
    basis_exact_match: bool,
    beats_outcome_destroyed_control: bool,
    beats_ledger_closure_control: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.candidate_exists
            && self.promotion_holdout_passed
            && self.future_defect_gain
            && self.future_defect_ratio
            && self.all_future_windows_win
            && self.basis_exact_match
            && self.beats_outcome_destroyed_control
            && self.beats_ledger_closure_control
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
    initial_one_step_classes: usize,
    candidate_word: Option<String>,
    promoted_word: Option<String>,
    training_parent_defect: f64,
    training_split_defect: f64,
    training_gain_after_penalty: f64,
    holdout_parent_defect: f64,
    holdout_split_defect: f64,
    holdout_defect_gain: f64,
    holdout_defect_ratio: f64,
    future: DefectMetrics,
    right_absorption_rate: f64,
    basis_exact_match: bool,
    outcome_destroyed_control: DefectMetrics,
    ledger_closure_control: DefectMetrics,
    gates: GateReport,
    primary_conclusion: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = frozen_config();
    let state = ProbeState::new()?;
    let judge = RelativeImprovementJudge;
    let recorder = CycleObservationRecorder::new(judge);
    let fixed_config = FixedResidualProjectionConfig::default();
    let h4_predicate = h4_memory_predicate();
    let mut next_id = 1_u64;
    let mut total_judged_cycle_attempts = 0_usize;
    let mut excluded = 0_usize;
    let mut retained = 0_usize;
    let mut sequential_continuation_executions = 0_usize;
    let mut actual_sequential_compute_cost = 0_u64;

    let mut windows = Vec::<Vec<AnchorObservation>>::new();
    for family in FAMILIES {
        let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
        for class in EventClass::all() {
            for repeat in 0..REPEATS_PER_CLASS {
                let task = surface_task(family, class, repeat);
                let mut charge = emit_real_charge(&task)?;
                charge.kind = ChargeKind::Custom("unresolved".into());
                charge.id = next_id;
                next_id = next_id.saturating_add(1);

                let variable_charge = ontology_feature_charge(&charge);
                let fixed_charge = fixed_residual_feature_charge(&charge, fixed_config);
                let attempts = judged_component_attempts(&fixed_charge, &task, &state)?;
                total_judged_cycle_attempts += attempts.len();
                let one_step = one_step_measurements(&fixed_charge, &attempts, &judge, config)?;
                let fixed_observation = recorder.record(fixed_charge.clone(), &attempts)?;

                if h4_predicate.matches(&variable_charge) {
                    excluded += 1;
                    continue;
                }
                retained += 1;

                let continuations = execute_all_words(
                    &fixed_charge,
                    &task,
                    &state,
                    &judge,
                    config,
                )?;
                sequential_continuation_executions += continuations.len();
                actual_sequential_compute_cost = actual_sequential_compute_cost.saturating_add(
                    continuations.iter().map(|observation| observation.compute_cost).sum(),
                );
                window.push(AnchorObservation {
                    variable_charge,
                    fixed_charge,
                    fixed_observation,
                    hidden: class,
                    task,
                    one_step,
                    continuations,
                });
            }
        }
        windows.push(window);
    }

    let train = flatten_anchors(&windows[..TRAIN_WINDOWS]);
    let holdout = flatten_anchors(&windows[TRAIN_WINDOWS..TRAIN_WINDOWS + HOLDOUT_WINDOWS]);
    let future_windows = &windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..];
    let future = flatten_anchors(future_windows);

    let train_table = continuation_table(&train);
    let holdout_table = continuation_table(&holdout);
    let future_table = continuation_table(&future);
    let train_one_step = one_step_signatures(&train);
    let holdout_one_step = one_step_signatures(&holdout);
    let future_one_step = one_step_signatures(&future);

    let fit = fit_witnessed_congruence_split(
        &train_table,
        &train_one_step,
        &holdout_table,
        &holdout_one_step,
        config,
    )?;

    let future_metrics = evaluate_fit(
        &fit,
        &train_one_step,
        &future_one_step,
        &future_table,
        future_windows,
        config.max_signature_classes,
    );

    let mut transformed = train.clone();
    transformed.extend(holdout.clone());
    for anchor in &mut transformed {
        anchor.fixed_charge.residual = householder_basis_intervention(&anchor.fixed_charge.residual);
    }
    let basis_fit = fit_witnessed_congruence_split(
        &train_table,
        &train_one_step,
        &holdout_table,
        &holdout_one_step,
        config,
    )?;
    let basis_exact_match = fit == basis_fit;

    let mut destroyed_train = train_table.clone();
    let mut destroyed_holdout = holdout_table.clone();
    let mut destroyed_future = future_table.clone();
    destroy_continuation_identity(&mut destroyed_train, SEED ^ 0x4f55_5443);
    destroy_continuation_identity(&mut destroyed_holdout, SEED ^ 0x484f_4c44);
    destroy_continuation_identity(&mut destroyed_future, SEED ^ 0x4655_5452);
    let destroyed_fit = fit_witnessed_congruence_split(
        &destroyed_train,
        &train_one_step,
        &destroyed_holdout,
        &holdout_one_step,
        config,
    )?;
    let destroyed_metrics = evaluate_fit(
        &destroyed_fit,
        &train_one_step,
        &future_one_step,
        &destroyed_future,
        future_windows,
        config.max_signature_classes,
    );

    let ledger_train = ledger_closure_table(&train, config)?;
    let ledger_holdout = ledger_closure_table(&holdout, config)?;
    let ledger_future = ledger_closure_table(&future, config)?;
    let ledger_fit = fit_witnessed_congruence_split(
        &ledger_train,
        &train_one_step,
        &ledger_holdout,
        &holdout_one_step,
        config,
    )?;
    let ledger_metrics = evaluate_fit(
        &ledger_fit,
        &train_one_step,
        &future_one_step,
        &ledger_future,
        future_windows,
        config.max_signature_classes,
    );

    let right_absorption_rate = right_absorption_rate(&future, config);
    let initial_one_step_classes = train_one_step
        .values()
        .cloned()
        .collect::<BTreeSet<_>>()
        .len();

    let gates = GateReport {
        candidate_exists: fit.candidate_word.is_some(),
        promotion_holdout_passed: fit.applied(),
        future_defect_gain: future_metrics.defect_gain + 1e-12 >= MIN_FUTURE_DEFECT_GAIN,
        future_defect_ratio: future_metrics.defect_ratio <= MAX_FUTURE_DEFECT_RATIO + 1e-12,
        all_future_windows_win: future_metrics.window_wins >= MIN_FUTURE_WINDOW_WINS,
        basis_exact_match,
        beats_outcome_destroyed_control: future_metrics.defect_gain + 1e-12
            >= destroyed_metrics.defect_gain + MIN_CONTROL_DEFECT_MARGIN,
        beats_ledger_closure_control: future_metrics.defect_gain + 1e-12
            >= ledger_metrics.defect_gain + MIN_CONTROL_DEFECT_MARGIN,
    };

    let primary_conclusion = if gates.all_pass() {
        "WITNESSED CONGRUENCE SPLIT TRANSFERRED"
    } else if fit.applied() && future_metrics.defect_gain > 0.0 {
        "CONTINUATION STRUCTURE DETECTED, NOT TRANSFERRED"
    } else if fit.candidate_word.is_none() {
        "NO NONCONGRUENCE BEYOND ONE-STEP STATE"
    } else if destroyed_metrics.defect_gain + MIN_CONTROL_DEFECT_MARGIN
        > future_metrics.defect_gain
        || ledger_metrics.defect_gain + MIN_CONTROL_DEFECT_MARGIN > future_metrics.defect_gain
    {
        "CONTROL EXPLAINS EFFECT"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "H7 witnessed congruence split",
        observation_source: "real Starfire subsystem CHARGE -> resolver word executed without intermediate reset -> Environment objective feedback -> OutcomeWitness -> RelativeImprovementJudge -> same CognitiveCycleState",
        visible_charge_kind: "Custom(unresolved)",
        representation_boundary: "complete one-step terminal witness signature after frozen H4 memory-cohort exclusion; WCS fitting API receives no residual, kind, scope, task class, target, evidence text, or resolver-leader label",
        primitive_operation: "a state-threaded resolver continuation w refines the current behavioral quotient by equivalence <- equivalence intersect kernel(F_w) when its terminal witness predicts independent continuation behavior",
        frozen_contract: FrozenContract {
            seed: SEED,
            max_context_length: 2,
            allow_self_repeat: false,
            proposal_budget: config.proposal_budget,
            max_promoted_splits: 1,
            max_signature_classes: config.max_signature_classes,
            witness_deadzone: config.witness_deadzone,
            witness_strong_boundary: config.witness_strong_boundary,
            min_train_signature_support: config.min_train_signature_support,
            min_holdout_signature_support: config.min_holdout_signature_support,
            complexity_penalty: config.complexity_penalty,
            min_train_defect_gain_after_penalty: config.min_train_defect_gain_after_penalty,
            min_holdout_defect_gain: config.min_holdout_defect_gain,
            max_holdout_defect_ratio: config.max_holdout_defect_ratio,
            min_future_defect_gain: MIN_FUTURE_DEFECT_GAIN,
            max_future_defect_ratio: MAX_FUTURE_DEFECT_RATIO,
            min_future_window_wins: MIN_FUTURE_WINDOW_WINS,
            min_control_defect_margin: MIN_CONTROL_DEFECT_MARGIN,
        },
        cohort: CohortReport {
            total_real_emitter_observations: (TRAIN_WINDOWS + HOLDOUT_WINDOWS + TRANSFER_WINDOWS)
                * OBSERVATIONS_PER_WINDOW,
            total_judged_cycle_attempts,
            excluded_by_frozen_h4_memory_predicate: excluded,
            retained_non_memory: retained,
            train_non_memory: train.len(),
            holdout_non_memory: holdout.len(),
            future_non_memory: future.len(),
            sequential_continuation_executions,
            actual_sequential_compute_cost,
        },
        initial_one_step_classes,
        candidate_word: fit.candidate_word.map(word_name),
        promoted_word: fit.observer.as_ref().map(|observer| word_name(observer.word)),
        training_parent_defect: fit.training_parent_defect,
        training_split_defect: fit.training_split_defect,
        training_gain_after_penalty: fit.training_gain_after_penalty,
        holdout_parent_defect: fit.holdout_parent_defect,
        holdout_split_defect: fit.holdout_split_defect,
        holdout_defect_gain: fit.holdout_defect_gain,
        holdout_defect_ratio: fit.holdout_defect_ratio,
        future: future_metrics,
        right_absorption_rate,
        basis_exact_match,
        outcome_destroyed_control: destroyed_metrics,
        ledger_closure_control: ledger_metrics,
        gates,
        primary_conclusion,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn frozen_config() -> CongruenceSplitConfig {
    CongruenceSplitConfig {
        witness_deadzone: 0.05,
        witness_strong_boundary: 0.25,
        proposal_budget: 20,
        max_signature_classes: 16,
        min_train_signature_support: 8,
        min_holdout_signature_support: 4,
        complexity_penalty: 0.02,
        min_train_defect_gain_after_penalty: 0.15,
        min_holdout_defect_gain: 0.10,
        max_holdout_defect_ratio: 0.75,
    }
}

fn h4_memory_predicate() -> ConceptPredicate {
    ConceptPredicate::ResidualThreshold {
        dimension: 2,
        threshold: 0.171875,
        direction: Direction::AtMost,
    }
}

fn all_words() -> Vec<ResolverWord> {
    let mut words = Vec::with_capacity(CANDIDATES.len() * (CANDIDATES.len() - 1));
    for first in 0..CANDIDATES.len() {
        for second in 0..CANDIDATES.len() {
            if first != second {
                words.push(ResolverWord {
                    first: first as u8,
                    second: second as u8,
                });
            }
        }
    }
    words
}

fn one_step_measurements(
    charge: &Charge,
    attempts: &[JudgedResolverAttempt],
    judge: &RelativeImprovementJudge,
    config: CongruenceSplitConfig,
) -> Result<Vec<OneStepMeasurement>, Box<dyn Error>> {
    let mut measurements = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let judged = judge.evaluate(charge, &attempt.resolution, &attempt.witness);
        let mut cycle = CognitiveCycleState::new();
        if !cycle.admit_charge(charge.clone()) {
            return Err("one-step charge rejected".into());
        }
        cycle
            .apply_judgment(0, &judged)
            .ok_or("one-step judgment application failed")?;
        let motion = directed_normalized_motion(&attempt.witness)
            .ok_or("non-finite one-step witness")?;
        let disposition = if cycle.pending().is_empty() {
            TerminalDisposition::Resolved
        } else {
            TerminalDisposition::Persisted
        };
        let terminal = quantize_terminal_witness(motion, disposition, config)
            .ok_or("one-step witness quantization failed")?;
        measurements.push(OneStepMeasurement {
            terminal,
            accepted_fraction: if charge.magnitude > 0.0 {
                judged.accepted as f64 / charge.magnitude as f64
            } else {
                0.0
            },
            compute_cost: attempt.resolution.compute_cost,
        });
    }
    Ok(measurements)
}

fn execute_all_words(
    charge: &Charge,
    task: &ProbeTask,
    state: &ProbeState,
    judge: &RelativeImprovementJudge,
    config: CongruenceSplitConfig,
) -> Result<Vec<ContinuationObservation>, Box<dyn Error>> {
    all_words()
        .into_iter()
        .map(|word| execute_word(charge, task, word, state, judge, config))
        .collect()
}

fn execute_word(
    initial_charge: &Charge,
    task: &ProbeTask,
    word: ResolverWord,
    state: &ProbeState,
    judge: &RelativeImprovementJudge,
    config: CongruenceSplitConfig,
) -> Result<ContinuationObservation, Box<dyn Error>> {
    let mut environment = TargetVerifierEnvironment::new(task);
    let _ = environment.reset(initial_charge.id ^ word_seed(word));
    let initial_feedback = environment.objective_feedback();
    let mut cycle = CognitiveCycleState::new();
    if !cycle.admit_charge(initial_charge.clone()) {
        return Err("sequential charge rejected".into());
    }
    let mut total_cost = 0_u64;

    for resolver_id in [word.first, word.second] {
        if cycle.pending().is_empty() {
            break;
        }
        let current_charge = cycle.pending()[0].clone();
        let candidate = candidate_from_id(resolver_id)?;
        let output = resolve_component(candidate, task, state)?;
        let before = environment.objective_feedback();
        let step = environment.act(&output);
        let after = environment.objective_feedback();
        let compute_cost = step.action_cost.max(1);
        total_cost = total_cost.saturating_add(compute_cost);
        let resolution = Resolution {
            discharged: current_charge.magnitude,
            emitted: vec![],
            permitted_decay: 0.0,
            compute_cost,
        };
        let witness = OutcomeWitness::new(
            "objective_progress",
            before.progress,
            after.progress,
            ImprovementDirection::HigherIsBetter,
            after.evidence,
        );
        let judged = judge.evaluate(&current_charge, &resolution, &witness);
        cycle
            .apply_judgment(0, &judged)
            .ok_or("sequential judgment application failed")?;
    }

    let final_feedback = environment.objective_feedback();
    let terminal_witness = OutcomeWitness::new(
        "objective_progress",
        initial_feedback.progress,
        final_feedback.progress,
        ImprovementDirection::HigherIsBetter,
        final_feedback.evidence,
    );
    let motion = directed_normalized_motion(&terminal_witness)
        .ok_or("non-finite terminal continuation witness")?;
    let disposition = if cycle.pending().is_empty() {
        TerminalDisposition::Resolved
    } else {
        TerminalDisposition::Persisted
    };
    let terminal = quantize_terminal_witness(motion, disposition, config)
        .ok_or("terminal continuation witness quantization failed")?;

    Ok(ContinuationObservation {
        anchor_id: initial_charge.id,
        word,
        terminal,
        compute_cost: total_cost.max(1),
    })
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
        let witness = OutcomeWitness::new(
            "objective_progress",
            before.progress,
            after.progress,
            ImprovementDirection::HigherIsBetter,
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
            let gap = metacog.top_gap().ok_or("metacognition did not retain gap")?;
            knowledge_gap_charge(gap).ok_or_else(|| "gap emitter returned no charge".into())
        }
        EventClass::PredictionContradiction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(&task.prompt);
            let mut center = PredictionCenter::new();
            let context = conversation_context(task, Some(state.reservoir_state));
            let predictions = center.generate(&context);
            let prediction = predictions.first().ok_or("prediction center emitted no prediction")?;
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

fn conversation_context(task: &ProbeTask, quanot_state: Option<Vec<f64>>) -> ConversationContext {
    let mut context = ConversationContext::new(task.topic.clone(), 2, quanot_state, Some(0.5));
    context.recent_text = vec![task.prompt.clone()];
    context.discussed_entities = token_set(&task.prompt).into_iter().take(8).collect();
    context
}

fn token_set(text: &str) -> BTreeSet<String> {
    const STOPWORDS: [&str; 25] = [
        "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "for", "from", "in", "is",
        "it", "of", "on", "only", "the", "to", "used", "what", "which", "why", "with",
    ];
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '\'')
        .map(|token| token.trim_matches('\'').to_ascii_lowercase())
        .filter(|token| token.len() > 1 && !STOPWORDS.contains(&token.as_str()))
        .collect()
}

fn continuation_table(anchors: &[AnchorObservation]) -> Vec<ContinuationObservation> {
    anchors
        .iter()
        .flat_map(|anchor| anchor.continuations.iter().copied())
        .collect()
}

fn one_step_signatures(anchors: &[AnchorObservation]) -> BTreeMap<u64, Vec<TerminalWitness>> {
    anchors
        .iter()
        .map(|anchor| {
            (
                anchor.fixed_charge.id,
                anchor.one_step.iter().map(|measurement| measurement.terminal).collect(),
            )
        })
        .collect()
}

fn flatten_anchors(windows: &[Vec<AnchorObservation>]) -> Vec<AnchorObservation> {
    windows.iter().flat_map(|window| window.iter().cloned()).collect()
}

fn evaluate_fit(
    fit: &CongruenceSplitFit,
    train_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    eval_one_step: &BTreeMap<u64, Vec<TerminalWitness>>,
    eval_table: &[ContinuationObservation],
    eval_windows: &[Vec<AnchorObservation>],
    max_signature_classes: usize,
) -> DefectMetrics {
    let parent_map = frozen_parent_signature_map(train_one_step, max_signature_classes);
    let parent_classes = apply_parent_map(eval_one_step, &parent_map);
    let Some(observer) = fit.observer.as_ref() else {
        let parent_defect = star::charge::congruence_defect(
            &parent_classes,
            eval_table,
            &all_words(),
        )
        .unwrap_or(0.0);
        return DefectMetrics {
            applied: false,
            parent_defect,
            split_defect: parent_defect,
            defect_gain: 0.0,
            defect_ratio: 1.0,
            window_wins: 0,
        };
    };

    let audit_words: Vec<_> = all_words()
        .into_iter()
        .filter(|word| *word != observer.word && *word != observer.word.reverse())
        .collect();
    let split_classes = apply_observer(observer, eval_one_step, eval_table);
    let parent_defect = star::charge::congruence_defect(
        &parent_classes,
        eval_table,
        &audit_words,
    )
    .unwrap_or(0.0);
    let split_defect = star::charge::congruence_defect(
        &split_classes,
        eval_table,
        &audit_words,
    )
    .unwrap_or(parent_defect);
    let mut window_wins = 0_usize;
    for window in eval_windows {
        let signatures = one_step_signatures(window);
        let table = continuation_table(window);
        let window_parent = apply_parent_map(&signatures, &parent_map);
        let window_split = apply_observer(observer, &signatures, &table);
        let parent = star::charge::congruence_defect(&window_parent, &table, &audit_words)
            .unwrap_or(0.0);
        let split = star::charge::congruence_defect(&window_split, &table, &audit_words)
            .unwrap_or(parent);
        if split + 1e-12 < parent {
            window_wins += 1;
        }
    }
    DefectMetrics {
        applied: true,
        parent_defect,
        split_defect,
        defect_gain: parent_defect - split_defect,
        defect_ratio: ratio(split_defect, parent_defect),
        window_wins,
    }
}

fn frozen_parent_signature_map(
    signatures: &BTreeMap<u64, Vec<TerminalWitness>>,
    max_classes: usize,
) -> BTreeMap<Vec<TerminalWitness>, u16> {
    let mut counts = BTreeMap::<Vec<TerminalWitness>, usize>::new();
    for signature in signatures.values() {
        *counts.entry(signature.clone()).or_default() += 1;
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked.truncate(max_classes.max(1));
    ranked
        .into_iter()
        .enumerate()
        .map(|(index, (signature, _))| (signature, index as u16))
        .collect()
}

fn apply_parent_map(
    signatures: &BTreeMap<u64, Vec<TerminalWitness>>,
    map: &BTreeMap<Vec<TerminalWitness>, u16>,
) -> BTreeMap<u64, u16> {
    signatures
        .iter()
        .map(|(anchor_id, signature)| (*anchor_id, map.get(signature).copied().unwrap_or(0)))
        .collect()
}

fn apply_observer(
    observer: &WitnessedCongruenceObserver,
    signatures: &BTreeMap<u64, Vec<TerminalWitness>>,
    table: &[ContinuationObservation],
) -> BTreeMap<u64, u16> {
    let lookup: BTreeMap<_, _> = table
        .iter()
        .filter(|observation| observation.word == observer.word)
        .map(|observation| (observation.anchor_id, observation.terminal))
        .collect();
    signatures
        .iter()
        .map(|(anchor_id, signature)| {
            (
                *anchor_id,
                observer.class(signature, lookup.get(anchor_id).copied()),
            )
        })
        .collect()
}

fn destroy_continuation_identity(observations: &mut [ContinuationObservation], seed: u64) {
    let mut rng = StdRng::seed_from_u64(seed);
    for word in all_words() {
        let indices: Vec<_> = observations
            .iter()
            .enumerate()
            .filter(|(_, observation)| observation.word == word)
            .map(|(index, _)| index)
            .collect();
        let mut bundles: Vec<_> = indices
            .iter()
            .map(|index| (observations[*index].terminal, observations[*index].compute_cost))
            .collect();
        bundles.shuffle(&mut rng);
        for (index, (terminal, compute_cost)) in indices.into_iter().zip(bundles) {
            observations[index].terminal = terminal;
            observations[index].compute_cost = compute_cost;
        }
    }
}

fn ledger_closure_table(
    anchors: &[AnchorObservation],
    config: CongruenceSplitConfig,
) -> Result<Vec<ContinuationObservation>, Box<dyn Error>> {
    let mut table = Vec::with_capacity(anchors.len() * all_words().len());
    for anchor in anchors {
        for word in all_words() {
            let first = &anchor.one_step[word.first as usize];
            let second = &anchor.one_step[word.second as usize];
            let remaining = (1.0 - first.accepted_fraction.clamp(0.0, 1.0))
                * (1.0 - second.accepted_fraction.clamp(0.0, 1.0));
            let predicted_motion = 1.0 - remaining;
            let disposition = if remaining <= 1e-6 {
                TerminalDisposition::Resolved
            } else {
                TerminalDisposition::Persisted
            };
            let terminal = quantize_terminal_witness(predicted_motion, disposition, config)
                .ok_or("ledger closure quantization failed")?;
            table.push(ContinuationObservation {
                anchor_id: anchor.fixed_charge.id,
                word,
                terminal,
                compute_cost: first.compute_cost.saturating_add(second.compute_cost).max(1),
            });
        }
    }
    Ok(table)
}

fn right_absorption_rate(anchors: &[AnchorObservation], config: CongruenceSplitConfig) -> f64 {
    let signatures = one_step_signatures(anchors);
    let mut absorbed = 0_usize;
    let mut total = 0_usize;
    for anchor in anchors {
        let Some(one_step) = signatures.get(&anchor.fixed_charge.id) else {
            continue;
        };
        for observation in &anchor.continuations {
            total += 1;
            if observation.terminal == one_step[observation.word.second as usize] {
                absorbed += 1;
            }
        }
    }
    let _ = config;
    absorbed as f64 / total.max(1) as f64
}

fn householder_basis_intervention(residual: &[f32]) -> Vec<f32> {
    if residual.is_empty() {
        return Vec::new();
    }
    let norm = (1..=residual.len())
        .map(|index| (index as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    let v: Vec<f64> = (1..=residual.len())
        .map(|index| index as f64 / norm)
        .collect();
    let dot = residual
        .iter()
        .zip(v.iter())
        .map(|(value, basis)| *value as f64 * basis)
        .sum::<f64>();
    residual
        .iter()
        .zip(v)
        .map(|(value, basis)| (*value as f64 - 2.0 * basis * dot) as f32)
        .collect()
}

fn candidate_from_id(id: u8) -> Result<Candidate, Box<dyn Error>> {
    CANDIDATES
        .get(id as usize)
        .copied()
        .ok_or_else(|| format!("unknown resolver id {id}").into())
}

fn word_seed(word: ResolverWord) -> u64 {
    ((word.first as u64) << 32) | word.second as u64
}

fn word_name(word: ResolverWord) -> String {
    format!(
        "{}->{}",
        RESOLVERS[word.first as usize], RESOLVERS[word.second as usize]
    )
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

fn remove_sqlite_files(path: &Path) {
    for candidate in [
        path.to_path_buf(),
        PathBuf::from(format!("{}-shm", path.display())),
        PathBuf::from(format!("{}-wal", path.display())),
    ] {
        let _ = std::fs::remove_file(candidate);
    }
}
