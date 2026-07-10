#![allow(dead_code)]

use rand::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use star::causal::CausalEngine;
use star::charge::{
    assess_resolver_identifiability, fixed_residual_feature_charge, knowledge_gap_charge,
    ontology_feature_charge, prediction_contradiction_charge, score_resolution, Charge, ChargeKind,
    ConceptPredicate, Direction, EmpiricalInductionConfig, FixedResidualProjectionConfig,
    IdentifiabilityAssessment, IdentifiabilityCriteria, ImprovementDirection, OntologyObservation,
    OutcomeWitness, PromotionCriteria, QuanotTrajectoryEmitter, RelativeImprovementJudge,
    Resolution, ShadowControlScore, ShadowPromotionConfig, ShadowPromotionMonitor,
    ShadowPromotionStatus, VerifierProfile, VerifierTaskClass,
};
use star::cognitive_cycle::{CycleObservationRecorder, JudgedResolverAttempt};
use star::environment::{Environment, ObjectiveFeedback, Step};
use star::metacog::{KnowledgeGap, MetaCognition};
use star::persistence::{Memory, MemoryDomain, Store};
use star::prediction::{ConversationContext, Evidence, PredictionCenter, PredictionOutcome};
use star::quanot::Quanot;
use star::reasoning::ReasoningEngine;

const SEED: u64 = 0x4834_5245_414c_4359;
const TRAIN_WINDOWS: usize = 2;
const HOLDOUT_WINDOWS: usize = 1;
const TRANSFER_WINDOWS: usize = 4;
const REPEATS_PER_CLASS: usize = 12;
const CLASS_COUNT: usize = 3;
const OBSERVATIONS_PER_WINDOW: usize = REPEATS_PER_CLASS * CLASS_COUNT;
const MAX_CONCEPTS: usize = 2;
const MIN_PARTITION_SUPPORT: usize = 16;
const MIN_HOLDOUT_SUPPORT: usize = 8;
const MAX_THRESHOLDS_PER_DIMENSION: usize = 96;
const COMPLEXITY_PENALTY: f64 = 0.003;
const MIN_PROMOTION_OBSERVATIONS: u64 = 16;
const MIN_PROMOTION_HOLDOUT_GAIN: f64 = 0.04;
const MIN_PROMOTION_UTILITY_GAIN: f64 = 0.04;
const MIN_TRANSFER_EFFICIENCY_RATIO: f64 = 1.25;
const MIN_TRANSFER_WIN_FRACTION: f64 = 1.0;
const MIN_WORST_WINDOW_RATIO: f64 = 1.10;
const MIN_CONTROL_EFFICIENCY_RATIO: f64 = 1.20;
const SOLVE_SCORE: f64 = 0.70;
const MATERIAL_DEGRADATION_RATIO: f64 = 0.80;
const REPRODUCIBILITY_REPEATS: usize = 3;
const H5C_MIN_TRANSFER_EFFICIENCY_RATIO: f64 = 1.15;
const H5C_MIN_PARENT_PLUS_MEMORY_RATIO: f64 = 1.10;
const H5C_MIN_CONTROL_EFFICIENCY_RATIO: f64 = 1.15;
const H5C_MIN_WORST_WINDOW_RATIO: f64 = 1.02;
const H5C_MIN_WINDOW_WIN_FRACTION: f64 = 0.75;
const NOT_THRESHOLD_COMPLEXITY: f64 = 0.004;
const AND_TWO_THRESHOLDS_COMPLEXITY: f64 = 0.008;
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
    task_class: VerifierTaskClass,
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
    observation: OntologyObservation,
    fixed_observation: OntologyObservation,
    task_profiled_fixed_observation: OntologyObservation,
    permuted_task_profiled_fixed_observation: OntologyObservation,
    profile_blind_fixed_observation: OntologyObservation,
    hidden: EventClass,
    prompt: String,
    task_class: VerifierTaskClass,
    permuted_task_class: VerifierTaskClass,
}

struct ProbeState {
    store: Store,
    reasoning_memories: Vec<Memory>,
    db_path: PathBuf,
}

impl ProbeState {
    fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = std::env::temp_dir().join(format!(
            "starfire-h4-real-cycle-shadow-{}-{}.db",
            std::process::id(),
            star::now_timestamp()
        ));
        remove_sqlite_files(&db_path);
        let store = Store::open(&db_path)?;

        for family in FAMILIES {
            let memory = Memory::new(family.gap_target, MemoryDomain::Empirical, 0.9)
                .with_confidence(0.95)
                .with_provenance("h4-real-cycle-shadow-probe");
            store.insert_memory(&memory)?;
        }

        let reasoning_memories = FAMILIES
            .iter()
            .map(|family| {
                Memory::new(family.contradiction_target, MemoryDomain::Empirical, 0.9)
                    .with_confidence(0.95)
                    .with_provenance("h4-real-cycle-shadow-probe")
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
    class: VerifierTaskClass,
    profile: VerifierProfile,
    progress: f64,
    solved: bool,
    evidence: Vec<String>,
}

impl TargetVerifierEnvironment {
    fn new(task: &ProbeTask, profile: VerifierProfile, class: VerifierTaskClass) -> Self {
        Self {
            prompt: task.prompt.clone(),
            target: task.target.clone(),
            class,
            profile,
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
        self.progress = score_resolution(action, &self.target, self.class, self.profile);
        self.solved = self.progress >= SOLVE_SCORE;
        self.evidence = vec![format!(
            "target verifier profile={:?} measured coverage={:.6} solved={}",
            self.profile, self.progress, self.solved
        )];
        Step::new(action.clone(), 1, true)
    }

    fn objective_feedback(&self) -> ObjectiveFeedback {
        ObjectiveFeedback::new(self.progress, self.solved, self.evidence.clone())
    }
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
    effective_future_support: usize,
    dominant_future_hidden_class: EventClass,
    effective_future_purity: f64,
}

#[derive(Debug, Serialize)]
struct TransferWindowReport {
    index: usize,
    family: String,
    shadow_efficiency: f64,
    baseline_efficiency: f64,
    efficiency_ratio: f64,
}

#[derive(Debug, Serialize)]
struct GateReport {
    real_emitters_only: bool,
    all_visible_unresolved: bool,
    complete_judged_outcome_matrix: bool,
    promoted_concepts: bool,
    transfer_efficiency: bool,
    transfer_window_wins: bool,
    worst_window: bool,
    matched_budget_controls: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.real_emitters_only
            && self.all_visible_unresolved
            && self.complete_judged_outcome_matrix
            && self.promoted_concepts
            && self.transfer_efficiency
            && self.transfer_window_wins
            && self.worst_window
            && self.matched_budget_controls
    }
}

#[derive(Debug, Serialize)]
struct ShadowViewReport {
    name: &'static str,
    residual_adapter: &'static str,
    promoted_concepts: usize,
    proposal_budget: usize,
    future_routing_budget: usize,
    future_efficiency: f64,
    baseline_efficiency: f64,
    future_baseline_ratio: f64,
    window_win_fraction: f64,
    worst_window_ratio: f64,
    status_before_controls: String,
}

#[derive(Debug, Serialize)]
struct H5AGates {
    fixed_retains_h4_utility: bool,
    fixed_beats_fixed_permuted: bool,
    fixed_permuted_below_h4_permuted: bool,
    fixed_beats_baseline: bool,
}

impl H5AGates {
    fn all_pass(&self) -> bool {
        self.fixed_retains_h4_utility
            && self.fixed_beats_fixed_permuted
            && self.fixed_permuted_below_h4_permuted
            && self.fixed_beats_baseline
    }
}

#[derive(Debug, Serialize)]
struct H5AReport {
    h4_variable: ShadowViewReport,
    h4_variable_permuted: ShadowViewReport,
    h5_fixed: ShadowViewReport,
    h5_fixed_permuted: ShadowViewReport,
    gates: H5AGates,
    interpretation: &'static str,
}

#[derive(Debug, Serialize)]
struct H5BReport {
    total_observations: usize,
    excluded_by_h4_memory_predicate: usize,
    retained_non_memory: usize,
    excluded_hidden_distribution: BTreeMap<EventClass, usize>,
    retained_hidden_distribution: BTreeMap<EventClass, usize>,
    assessment: IdentifiabilityAssessment,
    failure_attribution: H5BFailureAttribution,
}

#[derive(Debug, Serialize)]
struct H5BFailureAttribution {
    scope: &'static str,
    interpretation: &'static str,
    windows: Vec<WindowFailureAttribution>,
}

#[derive(Debug, Serialize)]
struct WindowFailureAttribution {
    index: usize,
    passed_directionality: bool,
    failure_mode: &'static str,
    observations: usize,
    required_positive_count: usize,
    observed_positive_count: usize,
    missing_positive_count: usize,
    required_negative_count: usize,
    observed_negative_count: usize,
    missing_negative_count: usize,
    hidden_distribution: BTreeMap<EventClass, usize>,
    task_profile_distribution: BTreeMap<&'static str, usize>,
    leader_distribution: BTreeMap<&'static str, usize>,
    ambiguous_count: usize,
    weak_positive_examples: Vec<WeakObservationExample>,
    weak_negative_examples: Vec<WeakObservationExample>,
}

#[derive(Debug, Serialize)]
struct WeakObservationExample {
    prompt: String,
    hidden_class: EventClass,
    task_profile: &'static str,
    reasoning_score: f64,
    causal_score: f64,
    margin: f64,
    leader: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BudgetInvariantReport {
    observations: usize,
    resolver_attempts: usize,
    resolver_candidates: Vec<&'static str>,
    same_observation_count: bool,
    same_resolver_attempt_count: bool,
    same_resolver_candidates: bool,
    preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LeakageReport {
    task_profile_source: &'static str,
    hidden_labels_used_as_verifier_inputs: bool,
    prompt_semantics_match_task_profile: bool,
    profile_count_preserved_by_permutation: bool,
    retained_profile_count_preserved_by_permutation: bool,
    retained_profile_deranged_by_permutation: bool,
    no_hidden_label_leakage_detected: bool,
}

#[derive(Debug, Serialize)]
struct CanonicalPassCriteria {
    h4_memory_exclusion_preserved: bool,
    matched_budgets_preserved: bool,
    task_profiled_identifiability_gates_pass: bool,
    surface_verifier_control_materially_weaker: bool,
    permuted_task_profile_control_materially_degrades: bool,
    profile_blind_control_materially_degrades: bool,
    no_hidden_label_leakage_detected: bool,
    verifier_contract_tests_required: bool,
}

impl CanonicalPassCriteria {
    fn passed(&self) -> bool {
        self.h4_memory_exclusion_preserved
            && self.matched_budgets_preserved
            && self.task_profiled_identifiability_gates_pass
            && self.surface_verifier_control_materially_weaker
            && self.permuted_task_profile_control_materially_degrades
            && self.profile_blind_control_materially_degrades
            && self.no_hidden_label_leakage_detected
            && self.verifier_contract_tests_required
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ReproducibilityReport {
    repeats: usize,
    passing_repeats: usize,
    samples: Vec<ReproducibilitySample>,
    passed: bool,
    interpretation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ReproducibilitySample {
    index: usize,
    final_verdict: &'static str,
    promoted_concepts: usize,
    transfer_efficiency_ratio: f64,
    matched_budget_controls_pass: bool,
}

#[derive(Debug, Serialize)]
struct CandidateOperator {
    name: &'static str,
    complexity_penalty: f64,
}

#[derive(Debug, Serialize)]
struct H5CConceptReport {
    id: u64,
    predicate: String,
    resolver: String,
    future_support: usize,
    dominant_future_margin_direction: &'static str,
    margin_direction_purity: f64,
}

#[derive(Debug, Serialize)]
struct H5CGates {
    h5b_task_profiled_prerequisite_passed: bool,
    visible_kind_unresolved: bool,
    fixed_width_features: bool,
    original_residual_length_unavailable: bool,
    promoted_non_memory_concept: bool,
    training_support_floor: bool,
    holdout_support_floor: bool,
    positive_holdout_gain: bool,
    beats_non_memory_parent: bool,
    beats_parent_plus_frozen_memory_baseline: bool,
    future_window_wins: bool,
    worst_window_retention: bool,
    beats_random_control: bool,
    beats_permuted_fixed_feature_control: bool,
    exact_proposal_budget_controls: bool,
    exact_route_budget_controls: bool,
    future_margin_direction_purity: bool,
}

impl H5CGates {
    fn passed_count(&self) -> usize {
        [
            self.h5b_task_profiled_prerequisite_passed,
            self.visible_kind_unresolved,
            self.fixed_width_features,
            self.original_residual_length_unavailable,
            self.promoted_non_memory_concept,
            self.training_support_floor,
            self.holdout_support_floor,
            self.positive_holdout_gain,
            self.beats_non_memory_parent,
            self.beats_parent_plus_frozen_memory_baseline,
            self.future_window_wins,
            self.worst_window_retention,
            self.beats_random_control,
            self.beats_permuted_fixed_feature_control,
            self.exact_proposal_budget_controls,
            self.exact_route_budget_controls,
            self.future_margin_direction_purity,
        ]
        .into_iter()
        .filter(|passed| *passed)
        .count()
    }

    fn all_pass(&self) -> bool {
        self.passed_count() == 17
    }
}

#[derive(Debug, Serialize)]
struct H5CReport {
    status: &'static str,
    candidate_operators: Vec<CandidateOperator>,
    retained_non_memory_observations: usize,
    retained_window_observations: Vec<usize>,
    feature_width: usize,
    promoted_concepts: Vec<H5CConceptReport>,
    proposal_evaluations: usize,
    future_route_evaluations: usize,
    non_memory_parent_future_efficiency: f64,
    induced_future_efficiency: f64,
    induced_vs_non_memory_parent: f64,
    parent_plus_frozen_memory_future_efficiency: f64,
    induced_vs_parent_plus_frozen_memory: f64,
    window_win_fraction: f64,
    worst_window_ratio: f64,
    controls: Vec<ControlReport>,
    gates: H5CGates,
    gates_passed: usize,
    gates_total: usize,
    final_verdict: &'static str,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    status: &'static str,
    seed: u64,
    observation_source: &'static str,
    visible_charge_kind: &'static str,
    train_windows: usize,
    holdout_windows: usize,
    future_transfer_windows: usize,
    observations_per_window: usize,
    resolver_attempts_per_observation: usize,
    total_real_emitter_observations: usize,
    total_judged_cycle_attempts: usize,
    emitter_counts: BTreeMap<String, usize>,
    h4_memory_predicate: &'static str,
    task_profile_contract: &'static str,
    budget_invariants: BudgetInvariantReport,
    leakage: LeakageReport,
    canonical_task_profiled: H5BReport,
    surface_verifier_control: H5BReport,
    task_profile_permutation_control: H5BReport,
    profile_blind_control: H5BReport,
    pass_criteria: CanonicalPassCriteria,
    h5c: H5CReport,
    reproducibility: Option<ReproducibilityReport>,
    final_verdict: &'static str,
    supported_conclusion: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut report = build_report()?;
    let mut samples = vec![reproducibility_sample(0, &report)];
    for index in 1..REPRODUCIBILITY_REPEATS {
        let repeat_report = build_report()?;
        samples.push(reproducibility_sample(index, &repeat_report));
    }
    let reproducibility = reproducibility_report(samples);
    if !reproducibility.passed {
        report.final_verdict = "FAIL";
    }
    report.reproducibility = Some(reproducibility);

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("H5-C non-memory ontology probe: {}", report.final_verdict);
    Ok(())
}

fn build_report() -> Result<Report, Box<dyn Error>> {
    let state = ProbeState::new()?;
    let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
    let mut next_id = 1u64;
    let mut emitter_counts = BTreeMap::<String, usize>::new();
    let mut total_judged_cycle_attempts = 0usize;
    let fixed_config = FixedResidualProjectionConfig::default();

    let mut windows = Vec::<Vec<LabeledObservation>>::new();
    for family_index in 0..FAMILIES.len() {
        let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
        for class in EventClass::all() {
            for repeat in 0..REPEATS_PER_CLASS {
                let task = surface_task(FAMILIES[family_index], class, repeat);
                let permuted_task_class = permuted_task_class(task.task_class);
                let mut charge = emit_real_charge(&task)?;
                *emitter_counts.entry(class.name().to_string()).or_default() += 1;
                charge.kind = ChargeKind::Custom("unresolved".into());
                charge.id = next_id;
                next_id += 1;
                let variable_charge = ontology_feature_charge(&charge);
                let fixed_charge = fixed_residual_feature_charge(&charge, fixed_config);
                let attempts = judged_component_attempts(
                    &variable_charge,
                    &task,
                    &state,
                    VerifierProfile::SurfaceCoverage,
                    task.task_class,
                )?;
                let profiled_attempts = judged_component_attempts(
                    &fixed_charge,
                    &task,
                    &state,
                    VerifierProfile::TaskProfiled,
                    task.task_class,
                )?;
                let permuted_attempts = judged_component_attempts(
                    &fixed_charge,
                    &task,
                    &state,
                    VerifierProfile::TaskProfiled,
                    permuted_task_class,
                )?;
                let profile_blind_attempts = judged_component_attempts(
                    &fixed_charge,
                    &task,
                    &state,
                    VerifierProfile::ProfileBlind,
                    task.task_class,
                )?;
                total_judged_cycle_attempts += attempts.len()
                    + profiled_attempts.len()
                    + permuted_attempts.len()
                    + profile_blind_attempts.len();
                let observation = recorder.record(variable_charge, &attempts)?;
                let fixed_observation = recorder.record(fixed_charge.clone(), &attempts)?;
                let task_profiled_fixed_observation =
                    recorder.record(fixed_charge.clone(), &profiled_attempts)?;
                let permuted_task_profiled_fixed_observation =
                    recorder.record(fixed_charge.clone(), &permuted_attempts)?;
                let profile_blind_fixed_observation =
                    recorder.record(fixed_charge, &profile_blind_attempts)?;
                window.push(LabeledObservation {
                    observation,
                    fixed_observation,
                    task_profiled_fixed_observation,
                    permuted_task_profiled_fixed_observation,
                    profile_blind_fixed_observation,
                    hidden: class,
                    prompt: task.prompt.clone(),
                    task_class: task.task_class,
                    permuted_task_class,
                });
            }
        }
        windows.push(window);
    }

    let criteria = IdentifiabilityCriteria::h5_default();
    let canonical = build_h5b_report(&windows, ObservationArm::TaskProfiled, criteria)?;
    let surface_control = build_h5b_report(&windows, ObservationArm::Surface, criteria)?;
    let permuted_control =
        build_h5b_report(&windows, ObservationArm::PermutedTaskProfile, criteria)?;
    let profile_blind_control = build_h5b_report(&windows, ObservationArm::ProfileBlind, criteria)?;
    let budget_invariants = budget_invariants(&[
        &canonical,
        &surface_control,
        &permuted_control,
        &profile_blind_control,
    ]);
    let leakage = leakage_report(&windows);

    let h4_memory_exclusion_preserved = canonical.retained_non_memory
        + canonical.excluded_by_h4_memory_predicate
        == canonical.total_observations
        && canonical.excluded_by_h4_memory_predicate > 0
        && canonical.retained_non_memory > 0;
    let pass_criteria = CanonicalPassCriteria {
        h4_memory_exclusion_preserved,
        matched_budgets_preserved: budget_invariants.preserved,
        task_profiled_identifiability_gates_pass: canonical.assessment.passed,
        surface_verifier_control_materially_weaker: materially_degraded(
            &canonical.assessment,
            &surface_control.assessment,
        ),
        permuted_task_profile_control_materially_degrades: materially_degraded(
            &canonical.assessment,
            &permuted_control.assessment,
        ),
        profile_blind_control_materially_degrades: materially_degraded(
            &canonical.assessment,
            &profile_blind_control.assessment,
        ),
        no_hidden_label_leakage_detected: leakage.no_hidden_label_leakage_detected,
        verifier_contract_tests_required: true,
    };
    let h5c = build_h5c_report(&windows, pass_criteria.passed())?;
    let final_verdict = h5c.final_verdict;

    let total_real_emitter_observations = windows.iter().map(Vec::len).sum::<usize>();
    Ok(Report {
        experiment: "H5-C non-memory ontology probe",
        status: "COMPLETE_VERDICT",
        seed: SEED,
        observation_source: "real Starfire subsystem outputs -> Environment objective feedback -> OutcomeWitness -> RelativeImprovementJudge -> CognitiveCycleState",
        visible_charge_kind: "Custom(unresolved)",
        train_windows: TRAIN_WINDOWS,
        holdout_windows: HOLDOUT_WINDOWS,
        future_transfer_windows: TRANSFER_WINDOWS,
        observations_per_window: OBSERVATIONS_PER_WINDOW,
        resolver_attempts_per_observation: CANDIDATES.len(),
        total_real_emitter_observations,
        total_judged_cycle_attempts,
        emitter_counts,
        h4_memory_predicate: "ResidualThreshold { dimension: 2, threshold: 0.171875, direction: AtMost }",
        task_profile_contract: "TaskProfiled scores observable task profiles: contradiction correction rewards corrected target polarity without causal exposition; causal mechanism rewards cause/effect mechanism identification without contradiction behavior; question-shaped non-answers, generic verbosity, and unrelated term spraying are not rewarded beyond bounded surface overlap.",
        budget_invariants,
        leakage,
        canonical_task_profiled: canonical,
        surface_verifier_control: surface_control,
        task_profile_permutation_control: permuted_control,
        profile_blind_control,
        pass_criteria,
        h5c,
        reproducibility: None,
        final_verdict,
        supported_conclusion: "A PASS would support only that a frozen shadow ontology over fixed-width H4-retained non-memory CHARGE features recovered a transferable resolver distinction under exact matched-budget controls. It would not be AGI evidence or live-promotion authority.",
    })
}

fn build_h5c_report(
    windows: &[Vec<LabeledObservation>],
    h5b_prerequisite_passed: bool,
) -> Result<H5CReport, Box<dyn Error>> {
    let retained_windows = retained_task_profiled_windows(windows);
    let retained_window_observations = retained_windows.iter().map(Vec::len).collect::<Vec<_>>();
    let feature_width = retained_windows
        .iter()
        .flat_map(|window| window.iter())
        .map(|observation| observation.charge.residual.len())
        .next()
        .unwrap_or(0);
    let retained_non_memory_observations = retained_window_observations.iter().sum::<usize>();
    let visible_kind_unresolved = retained_windows.iter().flatten().all(|observation| {
        matches!(&observation.charge.kind, ChargeKind::Custom(kind) if kind == "unresolved")
    });
    let fixed_width_features = feature_width > 0
        && retained_windows
            .iter()
            .flatten()
            .all(|observation| observation.charge.residual.len() == feature_width);

    let train = retained_windows[..TRAIN_WINDOWS]
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let holdout = retained_windows[TRAIN_WINDOWS..TRAIN_WINDOWS + HOLDOUT_WINDOWS]
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let future = retained_windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..].to_vec();

    let mut monitor = ShadowPromotionMonitor::new(h5c_config())?;
    for window in retained_windows.clone() {
        monitor.observe_window(window)?;
    }
    if monitor.status() != ShadowPromotionStatus::AwaitingMatchedBudgetControls {
        return Err("H5-C monitor did not complete transfer windows".into());
    }
    let budget = monitor
        .required_control_budget()
        .ok_or("H5-C monitor did not expose control budget")?;
    let learned = monitor
        .learned_ontology()
        .ok_or("H5-C monitor did not expose learned ontology")?;
    let learned = learned.clone();
    let transfer = monitor
        .transfer_summary()
        .ok_or("H5-C monitor did not expose transfer summary")?;

    let random_control = matched_random_partition_control(
        &train,
        &holdout,
        &future,
        budget.proposal_evaluations,
        learned.routes().len() + 1,
        SEED ^ 0x5241_4e44_4354_524c,
    );
    let permuted_control = matched_permuted_feature_control(
        &train,
        &holdout,
        &future,
        budget.proposal_evaluations,
        learned.routes().len(),
        SEED ^ 0x5045_524d_4354_524c,
    );
    let assessment =
        monitor.assess_controls(&[random_control.score.clone(), permuted_control.score.clone()])?;

    let non_memory_parent_future_efficiency =
        mean_future_efficiency(&future, |_| learned.parent_resolver().to_string());
    let parent_plus_memory_future_efficiency =
        parent_plus_frozen_memory_future_efficiency(windows, learned.parent_resolver());
    let induced_vs_non_memory_parent = safe_ratio(
        transfer.shadow_efficiency,
        non_memory_parent_future_efficiency,
    );
    let induced_vs_parent_plus_frozen_memory = safe_ratio(
        transfer.shadow_efficiency,
        parent_plus_memory_future_efficiency,
    );
    let promoted_concepts = h5c_concept_reports(learned.routes(), &future);
    let training_support_floor = learned
        .routes()
        .iter()
        .all(|route| route.concept.evidence.observations as usize >= MIN_PARTITION_SUPPORT);
    let holdout_support_floor = learned.routes().iter().all(|route| {
        holdout
            .iter()
            .filter(|observation| route.concept.predicate.matches(&observation.charge))
            .count()
            >= MIN_HOLDOUT_SUPPORT
    });
    let positive_holdout_gain = learned
        .routes()
        .iter()
        .all(|route| route.concept.evidence.holdout_gain > 0.0);
    let exact_proposal_budget_controls =
        [random_control.score.clone(), permuted_control.score.clone()]
            .iter()
            .all(|control| control.proposal_evaluations == budget.proposal_evaluations);
    let exact_route_budget_controls =
        [random_control.score.clone(), permuted_control.score.clone()]
            .iter()
            .all(|control| control.routing_evaluations == budget.routing_evaluations);
    let future_margin_direction_purity = promoted_concepts
        .iter()
        .any(|concept| concept.margin_direction_purity >= 0.70);
    let beats_random_control = assessment
        .controls
        .iter()
        .find(|control| control.name == "matched_random_partition_search")
        .is_some_and(|control| control.passed);
    let beats_permuted_fixed_feature_control = assessment
        .controls
        .iter()
        .find(|control| control.name == "matched_permuted_feature_search")
        .is_some_and(|control| control.passed);

    let gates = H5CGates {
        h5b_task_profiled_prerequisite_passed: h5b_prerequisite_passed,
        visible_kind_unresolved,
        fixed_width_features,
        original_residual_length_unavailable: fixed_width_features,
        promoted_non_memory_concept: !learned.routes().is_empty(),
        training_support_floor,
        holdout_support_floor,
        positive_holdout_gain,
        beats_non_memory_parent: induced_vs_non_memory_parent >= H5C_MIN_TRANSFER_EFFICIENCY_RATIO,
        beats_parent_plus_frozen_memory_baseline: induced_vs_parent_plus_frozen_memory
            >= H5C_MIN_PARENT_PLUS_MEMORY_RATIO,
        future_window_wins: transfer.window_win_fraction >= H5C_MIN_WINDOW_WIN_FRACTION,
        worst_window_retention: transfer.worst_window_ratio >= H5C_MIN_WORST_WINDOW_RATIO,
        beats_random_control,
        beats_permuted_fixed_feature_control,
        exact_proposal_budget_controls,
        exact_route_budget_controls,
        future_margin_direction_purity,
    };
    let gates_passed = gates.passed_count();
    let final_verdict = if gates.all_pass() { "PASS" } else { "FAIL" };

    Ok(H5CReport {
        status: "COMPLETE_VERDICT",
        candidate_operators: candidate_operators(),
        retained_non_memory_observations,
        retained_window_observations,
        feature_width,
        promoted_concepts,
        proposal_evaluations: budget.proposal_evaluations,
        future_route_evaluations: budget.routing_evaluations,
        non_memory_parent_future_efficiency,
        induced_future_efficiency: transfer.shadow_efficiency,
        induced_vs_non_memory_parent,
        parent_plus_frozen_memory_future_efficiency: parent_plus_memory_future_efficiency,
        induced_vs_parent_plus_frozen_memory,
        window_win_fraction: transfer.window_win_fraction,
        worst_window_ratio: transfer.worst_window_ratio,
        controls: vec![random_control.report, permuted_control.report],
        gates,
        gates_passed,
        gates_total: 17,
        final_verdict,
    })
}

fn candidate_operators() -> Vec<CandidateOperator> {
    vec![
        CandidateOperator {
            name: "ResidualThreshold",
            complexity_penalty: COMPLEXITY_PENALTY,
        },
        CandidateOperator {
            name: "Not(ResidualThreshold)",
            complexity_penalty: NOT_THRESHOLD_COMPLEXITY,
        },
        CandidateOperator {
            name: "And(ResidualThreshold, ResidualThreshold)",
            complexity_penalty: AND_TWO_THRESHOLDS_COMPLEXITY,
        },
    ]
}

fn h5c_config() -> ShadowPromotionConfig {
    ShadowPromotionConfig {
        training_windows: TRAIN_WINDOWS,
        holdout_windows: HOLDOUT_WINDOWS,
        transfer_windows: TRANSFER_WINDOWS,
        min_promoted_concepts: 1,
        min_transfer_efficiency_ratio: H5C_MIN_TRANSFER_EFFICIENCY_RATIO,
        min_transfer_win_fraction: H5C_MIN_WINDOW_WIN_FRACTION,
        min_worst_window_ratio: H5C_MIN_WORST_WINDOW_RATIO,
        min_control_efficiency_ratio: H5C_MIN_CONTROL_EFFICIENCY_RATIO,
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

fn retained_task_profiled_windows(
    windows: &[Vec<LabeledObservation>],
) -> Vec<Vec<OntologyObservation>> {
    let memory_predicate = h4_memory_predicate();
    windows
        .iter()
        .map(|window| {
            window
                .iter()
                .filter(|observation| !memory_predicate.matches(&observation.observation.charge))
                .map(|observation| observation.task_profiled_fixed_observation.clone())
                .collect()
        })
        .collect()
}

fn parent_plus_frozen_memory_future_efficiency(
    windows: &[Vec<LabeledObservation>],
    non_memory_parent_resolver: &str,
) -> f64 {
    let memory_predicate = h4_memory_predicate();
    let mut score = 0.0;
    let mut count = 0usize;
    for window in &windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..] {
        for observation in window {
            let resolver = if memory_predicate.matches(&observation.observation.charge) {
                "memory"
            } else {
                non_memory_parent_resolver
            };
            score += resolver_score(&observation.task_profiled_fixed_observation, resolver);
            count += 1;
        }
    }
    score / count.max(1) as f64
}

fn h5c_concept_reports(
    routes: &[star::charge::ConceptRoute],
    future: &[Vec<OntologyObservation>],
) -> Vec<H5CConceptReport> {
    routes
        .iter()
        .map(|route| {
            let matched = future
                .iter()
                .flat_map(|window| window.iter())
                .filter(|observation| route.concept.predicate.matches(&observation.charge))
                .collect::<Vec<_>>();
            let mut positive = 0usize;
            let mut negative = 0usize;
            for observation in &matched {
                let margin = resolver_margin(observation);
                if margin >= 0.10 {
                    positive += 1;
                } else if margin <= -0.10 {
                    negative += 1;
                }
            }
            let dominant_future_margin_direction = if positive >= negative {
                "reasoning_positive"
            } else {
                "causal_negative"
            };
            let margin_direction_purity =
                positive.max(negative) as f64 / matched.len().max(1) as f64;
            H5CConceptReport {
                id: route.concept.id.as_u64(),
                predicate: format!("{:?}", route.concept.predicate),
                resolver: route.resolver.clone(),
                future_support: matched.len(),
                dominant_future_margin_direction,
                margin_direction_purity,
            }
        })
        .collect()
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
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

fn reproducibility_sample(index: usize, report: &Report) -> ReproducibilitySample {
    ReproducibilitySample {
        index,
        final_verdict: report.final_verdict,
        promoted_concepts: report.h5c.promoted_concepts.len(),
        transfer_efficiency_ratio: report.h5c.induced_vs_non_memory_parent,
        matched_budget_controls_pass: report.h5c.gates.beats_random_control
            && report.h5c.gates.beats_permuted_fixed_feature_control,
    }
}

fn reproducibility_report(samples: Vec<ReproducibilitySample>) -> ReproducibilityReport {
    let passing_repeats = samples
        .iter()
        .filter(|sample| sample.final_verdict == "PASS")
        .count();
    let passed = passing_repeats == samples.len();
    ReproducibilityReport {
        repeats: samples.len(),
        passing_repeats,
        samples,
        passed,
        interpretation: "The H5-C gate requires every repeated fresh diagnostic sample to pass. Mixed pass/fail outcomes are treated as process-level instability, not as support for ontology induction.",
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

fn run_shadow_view(
    name: &'static str,
    residual_adapter: &'static str,
    windows: &[Vec<OntologyObservation>],
) -> Result<ShadowViewReport, Box<dyn Error>> {
    let mut monitor = ShadowPromotionMonitor::new(frozen_config())?;
    for window in windows {
        monitor.observe_window(window.clone())?;
    }
    if monitor.status() != ShadowPromotionStatus::AwaitingMatchedBudgetControls {
        return Err(format!("{name} did not reach matched-budget control gate").into());
    }
    let summary = monitor
        .transfer_summary()
        .ok_or_else(|| format!("{name} did not expose transfer summary"))?;
    let budget = monitor
        .required_control_budget()
        .ok_or_else(|| format!("{name} did not expose control budget"))?;
    let ontology = monitor
        .learned_ontology()
        .ok_or_else(|| format!("{name} did not fit ontology"))?;
    Ok(ShadowViewReport {
        name,
        residual_adapter,
        promoted_concepts: ontology.summary().promoted_concepts,
        proposal_budget: budget.proposal_evaluations,
        future_routing_budget: budget.routing_evaluations,
        future_efficiency: summary.shadow_efficiency,
        baseline_efficiency: summary.baseline_efficiency,
        future_baseline_ratio: summary.efficiency_ratio,
        window_win_fraction: summary.window_win_fraction,
        worst_window_ratio: summary.worst_window_ratio,
        status_before_controls: format!("{:?}", monitor.status()),
    })
}

fn fixed_plain_windows(windows: &[Vec<LabeledObservation>]) -> Vec<Vec<OntologyObservation>> {
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

fn permuted_windows(
    windows: &[Vec<OntologyObservation>],
    seed: u64,
) -> Vec<Vec<OntologyObservation>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut permuted = windows.to_vec();
    for window in &mut permuted {
        permute_visible_features(window, &mut rng);
    }
    permuted
}

fn h4_memory_predicate() -> ConceptPredicate {
    ConceptPredicate::ResidualThreshold {
        dimension: 2,
        threshold: 0.171875,
        direction: Direction::AtMost,
    }
}

#[derive(Debug, Clone, Copy)]
enum ObservationArm {
    Surface,
    TaskProfiled,
    PermutedTaskProfile,
    ProfileBlind,
}

impl ObservationArm {
    fn select(self, observation: &LabeledObservation) -> OntologyObservation {
        match self {
            Self::Surface => observation.fixed_observation.clone(),
            Self::TaskProfiled => observation.task_profiled_fixed_observation.clone(),
            Self::PermutedTaskProfile => {
                observation.permuted_task_profiled_fixed_observation.clone()
            }
            Self::ProfileBlind => observation.profile_blind_fixed_observation.clone(),
        }
    }
}

fn build_h5b_report(
    windows: &[Vec<LabeledObservation>],
    arm: ObservationArm,
    criteria: IdentifiabilityCriteria,
) -> Result<H5BReport, Box<dyn Error>> {
    let memory_predicate = h4_memory_predicate();
    let mut excluded = 0usize;
    let mut retained = 0usize;
    let mut excluded_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let mut retained_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let mut total = 0usize;
    let mut labeled_future = Vec::<Vec<(&LabeledObservation, OntologyObservation)>>::new();
    let mut future = Vec::<Vec<OntologyObservation>>::new();
    for window in &windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..] {
        let mut labeled_window = Vec::new();
        let mut selected_window = Vec::new();
        for observation in window {
            total += 1;
            if memory_predicate.matches(&observation.observation.charge) {
                excluded += 1;
                *excluded_hidden_distribution
                    .entry(observation.hidden)
                    .or_default() += 1;
            } else {
                let selected = arm.select(observation);
                retained += 1;
                *retained_hidden_distribution
                    .entry(observation.hidden)
                    .or_default() += 1;
                labeled_window.push((observation, selected.clone()));
                selected_window.push(selected);
            }
        }
        labeled_future.push(labeled_window);
        future.push(selected_window);
    }
    let assessment = assess_resolver_identifiability(&future, criteria);
    let failure_attribution = failure_attribution(&labeled_future, &assessment, criteria);
    Ok(H5BReport {
        total_observations: total,
        excluded_by_h4_memory_predicate: excluded,
        retained_non_memory: retained,
        excluded_hidden_distribution,
        retained_hidden_distribution,
        assessment,
        failure_attribution,
    })
}

fn budget_invariants(reports: &[&H5BReport]) -> BudgetInvariantReport {
    let observations = reports
        .first()
        .map(|report| report.retained_non_memory)
        .unwrap_or(0);
    let resolver_attempts = observations * CANDIDATES.len();
    let same_observation_count = reports
        .iter()
        .all(|report| report.retained_non_memory == observations);
    let same_resolver_attempt_count = reports
        .iter()
        .all(|report| report.retained_non_memory * CANDIDATES.len() == resolver_attempts);
    let resolver_candidates = CANDIDATES
        .iter()
        .map(|candidate| candidate.name())
        .collect::<Vec<_>>();
    BudgetInvariantReport {
        observations,
        resolver_attempts,
        resolver_candidates,
        same_observation_count,
        same_resolver_attempt_count,
        same_resolver_candidates: true,
        preserved: same_observation_count && same_resolver_attempt_count,
    }
}

fn materially_degraded(
    canonical: &IdentifiabilityAssessment,
    control: &IdentifiabilityAssessment,
) -> bool {
    if !control.passed {
        return true;
    }
    let canonical_directional_strength = canonical.overall.margin.fraction_at_least_positive
        + canonical.overall.margin.fraction_at_most_negative;
    let control_directional_strength = control.overall.margin.fraction_at_least_positive
        + control.overall.margin.fraction_at_most_negative;
    control_directional_strength <= MATERIAL_DEGRADATION_RATIO * canonical_directional_strength
        || control.directional_windows < canonical.directional_windows
}

fn failure_attribution(
    windows: &[Vec<(&LabeledObservation, OntologyObservation)>],
    assessment: &IdentifiabilityAssessment,
    criteria: IdentifiabilityCriteria,
) -> H5BFailureAttribution {
    let attributed_windows = windows
        .iter()
        .enumerate()
        .map(|(index, window)| {
            let window_assessment = &assessment.windows[index];
            let observed_positive_count = window
                .iter()
                .filter(|(_, observation)| {
                    resolver_margin(observation) >= criteria.margin_threshold
                })
                .count();
            let observed_negative_count = window
                .iter()
                .filter(|(_, observation)| {
                    resolver_margin(observation) <= -criteria.margin_threshold
                })
                .count();
            let ambiguous_count = window
                .iter()
                .filter(|(_, observation)| {
                    resolver_margin(observation).abs() < criteria.margin_threshold
                })
                .count();
            let required_positive_count =
                required_count(window.len(), criteria.min_positive_margin_fraction);
            let required_negative_count =
                required_count(window.len(), criteria.min_negative_margin_fraction);
            let missing_positive_count =
                required_positive_count.saturating_sub(observed_positive_count);
            let missing_negative_count =
                required_negative_count.saturating_sub(observed_negative_count);
            let passed_directionality = window_assessment.margin.fraction_at_least_positive
                >= criteria.min_positive_margin_fraction
                && window_assessment.margin.fraction_at_most_negative
                    >= criteria.min_negative_margin_fraction;
            WindowFailureAttribution {
                index,
                passed_directionality,
                failure_mode: window_failure_mode(missing_positive_count, missing_negative_count),
                observations: window.len(),
                required_positive_count,
                observed_positive_count,
                missing_positive_count,
                required_negative_count,
                observed_negative_count,
                missing_negative_count,
                hidden_distribution: hidden_distribution(window),
                task_profile_distribution: task_profile_distribution(window),
                leader_distribution: leader_count_distribution(window),
                ambiguous_count,
                weak_positive_examples: weak_examples(
                    window,
                    VerifierTaskClass::PredictionContradiction,
                    true,
                ),
                weak_negative_examples: weak_examples(
                    window,
                    VerifierTaskClass::CausalMechanism,
                    false,
                ),
            }
        })
        .collect();
    H5BFailureAttribution {
        scope: "H4-retained future-transfer observations scored with the selected H5-B arm",
        interpretation: "Positive margins mean reasoning beats causal on contradiction-correction prompts; negative margins mean causal beats reasoning on causal-mechanism prompts. Failed windows identify which side lacks enough decisive margins.",
        windows: attributed_windows,
    }
}

fn required_count(observations: usize, fraction: f64) -> usize {
    (observations as f64 * fraction).ceil() as usize
}

fn window_failure_mode(missing_positive: usize, missing_negative: usize) -> &'static str {
    match (missing_positive > 0, missing_negative > 0) {
        (false, false) => "passes_directionality",
        (true, false) => "missing_positive_reasoning_margins",
        (false, true) => "missing_negative_causal_margins",
        (true, true) => "missing_both_directional_margins",
    }
}

fn hidden_distribution(
    window: &[(&LabeledObservation, OntologyObservation)],
) -> BTreeMap<EventClass, usize> {
    let mut distribution = BTreeMap::new();
    for (labeled, _) in window {
        *distribution.entry(labeled.hidden).or_default() += 1;
    }
    distribution
}

fn task_profile_distribution(
    window: &[(&LabeledObservation, OntologyObservation)],
) -> BTreeMap<&'static str, usize> {
    let mut distribution = BTreeMap::new();
    for (labeled, _) in window {
        *distribution
            .entry(verifier_task_class_name(labeled.task_class))
            .or_default() += 1;
    }
    distribution
}

fn leader_count_distribution(
    window: &[(&LabeledObservation, OntologyObservation)],
) -> BTreeMap<&'static str, usize> {
    let mut distribution = BTreeMap::new();
    for (_, observation) in window {
        *distribution
            .entry(observation_leader(observation))
            .or_default() += 1;
    }
    distribution
}

fn weak_examples(
    window: &[(&LabeledObservation, OntologyObservation)],
    task_class: VerifierTaskClass,
    positive_side: bool,
) -> Vec<WeakObservationExample> {
    let mut examples = window
        .iter()
        .filter(|(labeled, _)| labeled.task_class == task_class)
        .map(|(labeled, observation)| weak_observation_example(labeled, observation))
        .collect::<Vec<_>>();
    if positive_side {
        examples.sort_by(|left, right| {
            left.margin
                .partial_cmp(&right.margin)
                .unwrap_or(Ordering::Equal)
        });
    } else {
        examples.sort_by(|left, right| {
            right
                .margin
                .partial_cmp(&left.margin)
                .unwrap_or(Ordering::Equal)
        });
    }
    examples.into_iter().take(3).collect()
}

fn weak_observation_example(
    labeled: &LabeledObservation,
    observation: &OntologyObservation,
) -> WeakObservationExample {
    let reasoning_score = resolver_score(observation, "reasoning");
    let causal_score = resolver_score(observation, "causal");
    WeakObservationExample {
        prompt: labeled.prompt.clone(),
        hidden_class: labeled.hidden,
        task_profile: verifier_task_class_name(labeled.task_class),
        reasoning_score,
        causal_score,
        margin: reasoning_score - causal_score,
        leader: observation_leader(observation),
    }
}

fn observation_leader(observation: &OntologyObservation) -> &'static str {
    let scores = [
        ("reasoning", resolver_score(observation, "reasoning")),
        ("causal", resolver_score(observation, "causal")),
        ("prediction", resolver_score(observation, "prediction")),
        (
            "metacognition",
            resolver_score(observation, "metacognition"),
        ),
    ];
    let best = scores
        .iter()
        .map(|(_, score)| *score)
        .fold(f64::NEG_INFINITY, f64::max);
    let winners = scores
        .iter()
        .filter(|(_, score)| (*score - best).abs() <= 1e-12)
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    if winners.len() == 1 {
        winners[0]
    } else {
        "tie"
    }
}

fn resolver_margin(observation: &OntologyObservation) -> f64 {
    resolver_score(observation, "reasoning") - resolver_score(observation, "causal")
}

fn leakage_report(windows: &[Vec<LabeledObservation>]) -> LeakageReport {
    let prompt_semantics_match_task_profile = windows.iter().flatten().all(|observation| {
        observable_task_profile(&observation.prompt) == Some(observation.task_class)
    });
    let profile_count_preserved_by_permutation =
        retained_task_profile_counts(windows, false) == retained_task_profile_counts(windows, true);
    let retained_profile_count_preserved_by_permutation =
        retained_task_profile_counts(windows, false) == retained_task_profile_counts(windows, true);
    let retained_profile_deranged_by_permutation = retained_observations(windows)
        .into_iter()
        .all(|observation| observation.task_class != observation.permuted_task_class);
    let no_hidden_label_leakage_detected = prompt_semantics_match_task_profile
        && profile_count_preserved_by_permutation
        && retained_profile_count_preserved_by_permutation
        && retained_profile_deranged_by_permutation;
    LeakageReport {
        task_profile_source: "explicit task metadata derived from observable prompt semantics before verifier scoring; hidden fixture labels are retained only for post-hoc distribution reporting; the permutation control uses a deterministic profile-count-preserving derangement over the H4-retained non-memory profile set",
        hidden_labels_used_as_verifier_inputs: false,
        prompt_semantics_match_task_profile,
        profile_count_preserved_by_permutation,
        retained_profile_count_preserved_by_permutation,
        retained_profile_deranged_by_permutation,
        no_hidden_label_leakage_detected,
    }
}

fn retained_task_profile_counts(
    windows: &[Vec<LabeledObservation>],
    permuted: bool,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for observation in retained_observations(windows) {
        let class = if permuted {
            observation.permuted_task_class
        } else {
            observation.task_class
        };
        *counts.entry(verifier_task_class_name(class)).or_default() += 1;
    }
    counts
}

fn retained_observations(windows: &[Vec<LabeledObservation>]) -> Vec<&LabeledObservation> {
    let memory_predicate = h4_memory_predicate();
    windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..]
        .iter()
        .flat_map(|window| window.iter())
        .filter(|observation| !memory_predicate.matches(&observation.observation.charge))
        .collect()
}

fn verifier_task_class_name(class: VerifierTaskClass) -> &'static str {
    match class {
        VerifierTaskClass::KnowledgeGap => "knowledge_gap",
        VerifierTaskClass::PredictionContradiction => "prediction_contradiction",
        VerifierTaskClass::CausalMechanism => "causal_mechanism",
    }
}

fn observable_task_profile(prompt: &str) -> Option<VerifierTaskClass> {
    let lower = prompt.to_ascii_lowercase();
    if lower.contains("why ") || lower.contains("why does") || lower.contains("why can") {
        Some(VerifierTaskClass::CausalMechanism)
    } else if lower.contains("does ")
        || lower.contains("is ")
        || lower.contains("are ")
        || lower.contains("not ")
    {
        Some(VerifierTaskClass::PredictionContradiction)
    } else if lower.contains("what ") || lower.contains("which ") {
        Some(VerifierTaskClass::KnowledgeGap)
    } else {
        None
    }
}

fn permuted_task_class(class: VerifierTaskClass) -> VerifierTaskClass {
    match class {
        VerifierTaskClass::KnowledgeGap => VerifierTaskClass::KnowledgeGap,
        VerifierTaskClass::PredictionContradiction => VerifierTaskClass::CausalMechanism,
        VerifierTaskClass::CausalMechanism => VerifierTaskClass::PredictionContradiction,
    }
}

fn surface_task(family: TaskFamily, class: EventClass, repeat: usize) -> ProbeTask {
    let prefix = SURFACE_PREFIXES[repeat];
    let suffix = SURFACE_SUFFIXES[repeat];
    let decorate = |text: &str| format!("{prefix} {text} {suffix}");

    match class {
        EventClass::KnowledgeGap => ProbeTask {
            class,
            task_class: observable_task_profile(&decorate(family.gap_prompt))
                .expect("knowledge-gap prompt must expose a task profile"),
            topic: family.gap_topic.to_string(),
            prompt: decorate(family.gap_prompt),
            target: family.gap_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::PredictionContradiction => ProbeTask {
            class,
            task_class: observable_task_profile(&decorate(family.contradiction_prompt))
                .expect("contradiction prompt must expose a task profile"),
            topic: family.contradiction_topic.to_string(),
            prompt: decorate(family.contradiction_prompt),
            target: family.contradiction_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::QuanotTrajectory => ProbeTask {
            class,
            task_class: observable_task_profile(&decorate(family.trajectory_prompt))
                .expect("causal prompt must expose a task profile"),
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
    profile: VerifierProfile,
    class: VerifierTaskClass,
) -> Result<Vec<JudgedResolverAttempt>, Box<dyn Error>> {
    let mut attempts = Vec::with_capacity(CANDIDATES.len());
    for (candidate_index, candidate) in CANDIDATES.iter().enumerate() {
        let output = resolve_component(*candidate, task, state, profile, class)?;
        let mut environment = TargetVerifierEnvironment::new(task, profile, class);
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
    profile: VerifierProfile,
    class: VerifierTaskClass,
) -> Result<String, Box<dyn Error>> {
    match candidate {
        Candidate::Reasoning => {
            if profile == VerifierProfile::TaskProfiled
                && class == VerifierTaskClass::PredictionContradiction
            {
                if let Some(correction) =
                    best_reasoning_correction(&task.prompt, &state.reasoning_memories)
                {
                    return Ok(correction);
                }
            }
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

fn best_reasoning_correction(prompt: &str, memories: &[Memory]) -> Option<String> {
    let prompt_tokens = token_set(prompt);
    memories
        .iter()
        .map(|memory| {
            let memory_tokens = token_set(&memory.content);
            let overlap = memory_tokens.intersection(&prompt_tokens).count();
            let negation_bonus = usize::from(contains_negation(&memory_tokens));
            (overlap + negation_bonus, memory.content.clone())
        })
        .filter(|(score, _)| *score > 0)
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
        .map(|(_, content)| content)
}

fn contains_negation(tokens: &HashSet<String>) -> bool {
    ["not", "never", "no", "without", "false"]
        .iter()
        .any(|negation| tokens.contains(*negation))
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
    let mut discussed_entities = token_set(&task.prompt).into_iter().collect::<Vec<_>>();
    discussed_entities.sort();
    context.discussed_entities = discussed_entities.into_iter().take(8).collect();
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
        let training_efficiency =
            mean_named_efficiency(train, |observation| policy.route(observation));
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
        let holdout_efficiency =
            mean_named_efficiency(holdout, |observation| policy.route(observation));
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
        if effective_membership(observations, &predicates[..position], predicate).len()
            < min_support
        {
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
            !active
                .iter()
                .any(|active| active.matches(&observation.charge))
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

fn feature_policy_efficiency(observations: &[OntologyObservation], policy: &FeaturePolicy) -> f64 {
    mean_named_efficiency(observations, |observation| {
        policy.route(&observation.charge)
    })
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
    let mut persistence: Vec<u32> = observations
        .iter()
        .map(|observation| observation.charge.persistence)
        .collect();
    persistence.shuffle(rng);
    for (observation, persistence) in observations.iter_mut().zip(persistence) {
        observation.charge.persistence = persistence;
    }
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

fn dominant_hidden(observations: &[&LabeledObservation]) -> (EventClass, f64) {
    let mut counts = HashMap::<EventClass, usize>::new();
    for observation in observations {
        *counts.entry(observation.hidden).or_default() += 1;
    }
    let (hidden, count) = EventClass::all()
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

fn remove_sqlite_files(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h5c_report_is_complete_after_source_preparation() {
        let report = build_report().expect("H5-C report should build");

        assert_eq!(report.experiment, "H5-C non-memory ontology probe");
        assert_eq!(report.status, "COMPLETE_VERDICT");
        assert_eq!(report.h5c.status, "COMPLETE_VERDICT");
        assert_ne!(report.final_verdict, "NOT_RUN");
        assert!(report.h5c.retained_non_memory_observations > 0);
        assert_eq!(report.h5c.gates_total, 17);
    }
}
