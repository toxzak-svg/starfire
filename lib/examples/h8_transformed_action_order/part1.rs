#![allow(dead_code)]

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use star::causal::CausalEngine;
use star::charge::{
    directed_normalized_motion, fixed_residual_feature_charge, knowledge_gap_charge,
    ontology_feature_charge, prediction_contradiction_charge, score_resolution, Charge, ChargeKind,
    ConceptPredicate, Direction, DischargeJudge, FixedResidualProjectionConfig,
    ImprovementDirection, OutcomeWitness, QuanotTrajectoryEmitter, RelativeImprovementJudge,
    Resolution, VerifierProfile, VerifierTaskClass,
};
use star::cognitive_cycle::CognitiveCycleState;
use star::environment::{Environment, ObjectiveFeedback, Step};
use star::metacog::{KnowledgeGap, MetaCognition};
use star::persistence::{Memory, MemoryDomain};
use star::prediction::{ConversationContext, Evidence, PredictionCenter, PredictionOutcome};
use star::quanot::Quanot;
use star::reasoning::ReasoningEngine;

const SEED: u64 = 0x4838_4143_5444_4941;
const TRAIN_WINDOWS: usize = 2;
const HOLDOUT_WINDOWS: usize = 1;
const TRANSFER_WINDOWS: usize = 4;
const REPEATS_PER_CLASS: usize = 12;
const CLASS_COUNT: usize = 3;
const OBSERVATIONS_PER_WINDOW: usize = REPEATS_PER_CLASS * CLASS_COUNT;
const SOLVE_SCORE: f64 = 0.70;

const PROPOSAL_BUDGET: usize = 2;
const COMPLEXITY_PENALTY: f64 = 0.01;

const MIN_TRAIN_ELIGIBLE: usize = 32;
const MIN_HOLDOUT_ELIGIBLE: usize = 16;
const MIN_FUTURE_ELIGIBLE: usize = 64;

const MIN_TRAIN_GAIN_AFTER_PENALTY: f64 = 0.05;
const MIN_TRAIN_ORDER_ADVANTAGE: f64 = 0.03;
const MIN_HOLDOUT_GAIN: f64 = 0.05;
const MIN_HOLDOUT_ORDER_ADVANTAGE: f64 = 0.03;
const MIN_HOLDOUT_POSITIVE_FRACTION: f64 = 0.60;
const MAX_HOLDOUT_RIGHT_ABSORPTION: f64 = 0.75;
const MIN_FUTURE_GAIN: f64 = 0.05;
const MIN_FUTURE_ORDER_ADVANTAGE: f64 = 0.03;
const MIN_FUTURE_WINDOW_WINS: usize = 4;
const MIN_WORST_FAMILY_GAIN: f64 = 0.01;
const MAX_FUTURE_RIGHT_ABSORPTION: f64 = 0.75;
const MIN_REWIRED_MARGIN: f64 = 0.03;
const MIN_SCALAR_MARGIN: f64 = 0.03;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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
        contradiction_prompt: "Are Earth's seasons caused mainly by changing distance from the Sun?",
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
    Causal,
}

impl Candidate {
    fn name(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::Causal => "causal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum Word {
    ReasoningThenCausal,
    CausalThenReasoning,
}

impl Word {
    fn all() -> [Self; 2] {
        [Self::ReasoningThenCausal, Self::CausalThenReasoning]
    }

    fn name(self) -> &'static str {
        match self {
            Self::ReasoningThenCausal => "reasoning->causal",
            Self::CausalThenReasoning => "causal->reasoning",
        }
    }

    fn first(self) -> Candidate {
        match self {
            Self::ReasoningThenCausal => Candidate::Reasoning,
            Self::CausalThenReasoning => Candidate::Causal,
        }
    }

    fn second(self) -> Candidate {
        match self {
            Self::ReasoningThenCausal => Candidate::Causal,
            Self::CausalThenReasoning => Candidate::Reasoning,
        }
    }

    fn reverse(self) -> Self {
        match self {
            Self::ReasoningThenCausal => Self::CausalThenReasoning,
            Self::CausalThenReasoning => Self::ReasoningThenCausal,
        }
    }

    fn salt(self) -> u64 {
        match self {
            Self::ReasoningThenCausal => 0x5243,
            Self::CausalThenReasoning => 0x4352,
        }
    }
}

#[derive(Debug, Clone)]
struct AnchorObservation {
    fixed_charge: Charge,
    task: ProbeTask,
    hidden: EventClass,
}

struct ProbeState {
    reasoning_memories: Vec<Memory>,
}

impl ProbeState {
    fn new() -> Self {
        let reasoning_memories = FAMILIES
            .iter()
            .map(|family| {
                Memory::new(family.contradiction_target, MemoryDomain::Empirical, 0.9)
                    .with_confidence(0.95)
                    .with_provenance("h8-transformed-action-order-diamond")
            })
            .collect();
        Self { reasoning_memories }
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
            evidence: vec!["episode reset before action-order diamond".into()],
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

#[derive(Debug, Clone)]
struct IntermediateState {
    anchor_id: u64,
    first_output: String,
    signed_objective_motion: f64,
    accepted_fraction: f64,
    unresolved_fraction: f64,
}

struct FirstStepRuntime {
    environment: TargetVerifierEnvironment,
    cycle: CognitiveCycleState,
    intermediate: IntermediateState,
    compute_cost: u64,
}

#[derive(Debug, Clone, Copy)]
enum ExecutionMode {
    Stateful,
    Blind,
    ScalarState,
    Rewired,
}

#[derive(Debug, Clone)]
struct Execution {
    terminal_score: f64,
    compute_cost: u64,
    resolver_calls: usize,
    objective_evaluations: usize,
    second_prompt_bytes: usize,
}

#[derive(Debug, Clone)]
struct PathRow {
    anchor_id: u64,
    stateful_score: f64,
    blind_score: f64,
    scalar_score: f64,
    rewired_score: f64,
    stateful_prompt_bytes: usize,
    scalar_prompt_bytes: usize,
    rewired_prompt_bytes: usize,
}

#[derive(Debug, Clone)]
struct SplitEvaluation {
    retained: usize,
    eligible: usize,
    rows: BTreeMap<Word, Vec<PathRow>>,
    eligibility_resolver_calls: usize,
    eligibility_objective_evaluations: usize,
    composite_resolver_calls: usize,
    composite_objective_evaluations: usize,
}

#[derive(Debug, Clone, Serialize)]
struct WordMetrics {
    word: &'static str,
    eligible: usize,
    mean_stateful_score: f64,
    mean_blind_score: f64,
    composition_gain: f64,
    mean_scalar_score: f64,
    scalar_margin: f64,
    mean_rewired_score: f64,
    rewired_margin: f64,
    positive_fraction: f64,
    right_absorption_rate: f64,
    mean_stateful_prompt_bytes: f64,
    mean_scalar_prompt_bytes: f64,
    mean_rewired_prompt_bytes: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SplitMetricsReport {
    retained: usize,
    eligible: usize,
    candidate_order_advantage: f64,
    words: Vec<WordMetrics>,
}

#[derive(Debug, Clone, Serialize)]
struct FutureWindowReport {
    index: usize,
    family: &'static str,
    eligible: usize,
    candidate_stateful_score: f64,
    candidate_blind_score: f64,
    composition_gain: f64,
    order_advantage: f64,
    win: bool,
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    seed: u64,
    proposal_budget: usize,
    complexity_penalty: f64,
    min_train_eligible: usize,
    min_holdout_eligible: usize,
    min_future_eligible: usize,
    min_train_gain_after_penalty: f64,
    min_train_order_advantage: f64,
    min_holdout_gain: f64,
    min_holdout_order_advantage: f64,
    min_holdout_positive_fraction: f64,
    max_holdout_right_absorption: f64,
    min_future_gain: f64,
    min_future_order_advantage: f64,
    min_future_window_wins: usize,
    min_worst_family_gain: f64,
    max_future_right_absorption: f64,
    min_rewired_margin: f64,
    min_scalar_margin: f64,
}

#[derive(Debug, Serialize)]
struct CohortReport {
    total_real_emitter_observations: usize,
    excluded_by_frozen_h4_memory_predicate: usize,
    retained_non_memory: usize,
    train_non_memory: usize,
    holdout_non_memory: usize,
    future_non_memory: usize,
    excluded_hidden_distribution: BTreeMap<String, usize>,
    retained_hidden_distribution: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct BudgetReport {
    eligibility_resolver_calls: usize,
    eligibility_objective_evaluations: usize,
    composite_resolver_calls: usize,
    composite_objective_evaluations: usize,
    expected_eligibility_resolver_calls: usize,
    expected_eligibility_objective_evaluations: usize,
    expected_composite_resolver_calls: usize,
    expected_composite_objective_evaluations: usize,
    budget_exact: bool,
}

#[derive(Debug, Serialize)]
struct GateReport {
    train_support: bool,
    holdout_support: bool,
    future_support: bool,
    train_gain_after_penalty: bool,
    train_order_advantage: bool,
    holdout_gain: bool,
    holdout_order_advantage: bool,
    holdout_positive_fraction: bool,
    holdout_right_absorption: bool,
    future_gain: bool,
    future_order_advantage: bool,
    all_future_windows_win: bool,
    worst_family_gain: bool,
    future_right_absorption: bool,
    rewired_margin: bool,
    scalar_margin: bool,
    budget_exact: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.train_support
            && self.holdout_support
            && self.future_support
            && self.train_gain_after_penalty
            && self.train_order_advantage
            && self.holdout_gain
            && self.holdout_order_advantage
            && self.holdout_positive_fraction
            && self.holdout_right_absorption
            && self.future_gain
            && self.future_order_advantage
            && self.all_future_windows_win
            && self.worst_family_gain
            && self.future_right_absorption
            && self.rewired_margin
            && self.scalar_margin
            && self.budget_exact
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    observation_source: &'static str,
    visible_charge_kind: &'static str,
    representation_boundary: &'static str,
    operation_under_test: &'static str,
    frozen_contract: FrozenContract,
    cohort: CohortReport,
    candidate_word: &'static str,
    promoted_word: Option<&'static str>,
    training: SplitMetricsReport,
    holdout: SplitMetricsReport,
    future: SplitMetricsReport,
    future_windows: Vec<FutureWindowReport>,
    future_window_wins: usize,
    worst_family_gain: f64,
    budget: BudgetReport,
    gates: GateReport,
    terminal_classification: &'static str,
}
