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
    ontology_feature_charge, prediction_contradiction_charge, Charge, ChargeKind, ConceptPredicate,
    Direction, EmpiricalInductionConfig, FixedResidualProjectionConfig, IdentifiabilityAssessment,
    IdentifiabilityCriteria, ImprovementDirection, OntologyObservation, OutcomeWitness,
    PromotionCriteria, QuanotTrajectoryEmitter, RelativeImprovementJudge, Resolution,
    ShadowControlScore, ShadowPromotionConfig, ShadowPromotionMonitor, ShadowPromotionStatus,
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
    profiled_fixed_observation: OntologyObservation,
    hidden: EventClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerifierProfile {
    SurfaceCoverage,
    TaskProfiled,
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
    class: EventClass,
    profile: VerifierProfile,
    progress: f64,
    solved: bool,
    evidence: Vec<String>,
}

impl TargetVerifierEnvironment {
    fn new(task: &ProbeTask, profile: VerifierProfile) -> Self {
        Self {
            prompt: task.prompt.clone(),
            target: task.target.clone(),
            class: task.class,
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
        self.progress = match self.profile {
            VerifierProfile::SurfaceCoverage => resolution_score(action, &self.target),
            VerifierProfile::TaskProfiled => {
                profiled_resolution_score(action, &self.target, self.class)
            }
        };
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
    excluded_by_h4_memory_predicate: usize,
    retained_non_memory: usize,
    excluded_hidden_distribution: BTreeMap<EventClass, usize>,
    retained_hidden_distribution: BTreeMap<EventClass, usize>,
    assessment: IdentifiabilityAssessment,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
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
    h5a: H5AReport,
    h5b: H5BReport,
    h5b_task_profiled: H5BReport,
}

fn main() -> Result<(), Box<dyn Error>> {
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
                )?;
                let profiled_attempts = judged_component_attempts(
                    &fixed_charge,
                    &task,
                    &state,
                    VerifierProfile::TaskProfiled,
                )?;
                total_judged_cycle_attempts += attempts.len() + profiled_attempts.len();
                let observation = recorder.record(variable_charge, &attempts)?;
                let fixed_observation = recorder.record(fixed_charge.clone(), &attempts)?;
                let profiled_fixed_observation =
                    recorder.record(fixed_charge, &profiled_attempts)?;
                window.push(LabeledObservation {
                    observation,
                    fixed_observation,
                    profiled_fixed_observation,
                    hidden: class,
                });
            }
        }
        windows.push(window);
    }

    let variable_windows = plain_windows(&windows);
    let fixed_windows = fixed_plain_windows(&windows);
    let h4_variable = run_shadow_view(
        "h4_variable",
        "[rms, mean_abs, non_zero_fraction, positive_fraction, max_abs, ...raw residual]",
        &variable_windows,
    )?;
    let h4_variable_permuted = run_shadow_view(
        "h4_variable_permuted",
        "H4 variable representation with independently permuted visible features",
        &permuted_windows(&variable_windows, SEED ^ 0x4835_a),
    )?;
    let h5_fixed = run_shadow_view(
        "h5_fixed",
        "fixed-width residual projection; no raw residual length or masks",
        &fixed_windows,
    )?;
    let h5_fixed_permuted = run_shadow_view(
        "h5_fixed_permuted",
        "fixed-width residual projection with independently permuted visible features",
        &permuted_windows(&fixed_windows, SEED ^ 0x4835_b),
    )?;

    let h5a_gates = H5AGates {
        fixed_retains_h4_utility: h5_fixed.future_efficiency
            >= 0.90 * h4_variable.future_efficiency,
        fixed_beats_fixed_permuted: h5_fixed.future_efficiency
            >= 1.15 * h5_fixed_permuted.future_efficiency,
        fixed_permuted_below_h4_permuted: h5_fixed_permuted.future_efficiency
            <= 0.90 * h4_variable_permuted.future_efficiency,
        fixed_beats_baseline: h5_fixed.future_efficiency >= 1.25 * h5_fixed.baseline_efficiency,
    };
    let interpretation = if h5a_gates.all_pass() {
        "H5-A supports residual-shape leakage as a material H4 confound"
    } else if h5_fixed.future_efficiency < 0.90 * h4_variable.future_efficiency
        && h5_fixed_permuted.future_efficiency < 0.90 * h4_variable_permuted.future_efficiency
    {
        "fixed-width normalization removed utility from both real and permuted views"
    } else if h5_fixed_permuted.future_efficiency > 0.90 * h4_variable_permuted.future_efficiency {
        "fixed-width permuted control retained utility; residual length is not the main explanation"
    } else {
        "mixed H5-A result; inspect fixed representation and outcome matrix before ontology work"
    };

    let (non_memory_future, h5b_report_base) = non_memory_future_windows(&windows);
    let h5b_assessment =
        assess_resolver_identifiability(&non_memory_future, IdentifiabilityCriteria::h5_default());
    let (profiled_non_memory_future, h5b_profiled_report_base) =
        profiled_non_memory_future_windows(&windows);
    let h5b_task_profiled_assessment = assess_resolver_identifiability(
        &profiled_non_memory_future,
        IdentifiabilityCriteria::h5_default(),
    );

    let total_real_emitter_observations = windows.iter().map(Vec::len).sum::<usize>();
    let report = Report {
        experiment: "H5 residual identity and non-memory resolver identifiability diagnostic",
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
        h5a: H5AReport {
            h4_variable,
            h4_variable_permuted,
            h5_fixed,
            h5_fixed_permuted,
            gates: h5a_gates,
            interpretation,
        },
        h5b: H5BReport {
            excluded_by_h4_memory_predicate: h5b_report_base.0,
            retained_non_memory: h5b_report_base.1,
            excluded_hidden_distribution: h5b_report_base.2,
            retained_hidden_distribution: h5b_report_base.3,
            assessment: h5b_assessment,
        },
        h5b_task_profiled: H5BReport {
            excluded_by_h4_memory_predicate: h5b_profiled_report_base.0,
            retained_non_memory: h5b_profiled_report_base.1,
            excluded_hidden_distribution: h5b_profiled_report_base.2,
            retained_hidden_distribution: h5b_profiled_report_base.3,
            assessment: h5b_task_profiled_assessment,
        },
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
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

fn non_memory_future_windows(
    windows: &[Vec<LabeledObservation>],
) -> (
    Vec<Vec<OntologyObservation>>,
    (
        usize,
        usize,
        BTreeMap<EventClass, usize>,
        BTreeMap<EventClass, usize>,
    ),
) {
    let memory_predicate = h4_memory_predicate();
    let mut excluded = 0usize;
    let mut retained = 0usize;
    let mut excluded_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let mut retained_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let future = windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..]
        .iter()
        .map(|window| {
            window
                .iter()
                .filter_map(|observation| {
                    if memory_predicate.matches(&observation.observation.charge) {
                        excluded += 1;
                        *excluded_hidden_distribution
                            .entry(observation.hidden)
                            .or_default() += 1;
                        None
                    } else {
                        retained += 1;
                        *retained_hidden_distribution
                            .entry(observation.hidden)
                            .or_default() += 1;
                        Some(observation.fixed_observation.clone())
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    (
        future,
        (
            excluded,
            retained,
            excluded_hidden_distribution,
            retained_hidden_distribution,
        ),
    )
}

fn profiled_non_memory_future_windows(
    windows: &[Vec<LabeledObservation>],
) -> (
    Vec<Vec<OntologyObservation>>,
    (
        usize,
        usize,
        BTreeMap<EventClass, usize>,
        BTreeMap<EventClass, usize>,
    ),
) {
    let memory_predicate = h4_memory_predicate();
    let mut excluded = 0usize;
    let mut retained = 0usize;
    let mut excluded_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let mut retained_hidden_distribution = BTreeMap::<EventClass, usize>::new();
    let future = windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..]
        .iter()
        .map(|window| {
            window
                .iter()
                .filter_map(|observation| {
                    if memory_predicate.matches(&observation.observation.charge) {
                        excluded += 1;
                        *excluded_hidden_distribution
                            .entry(observation.hidden)
                            .or_default() += 1;
                        None
                    } else {
                        retained += 1;
                        *retained_hidden_distribution
                            .entry(observation.hidden)
                            .or_default() += 1;
                        Some(observation.profiled_fixed_observation.clone())
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    (
        future,
        (
            excluded,
            retained,
            excluded_hidden_distribution,
            retained_hidden_distribution,
        ),
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
    profile: VerifierProfile,
) -> Result<Vec<JudgedResolverAttempt>, Box<dyn Error>> {
    let mut attempts = Vec::with_capacity(CANDIDATES.len());
    for (candidate_index, candidate) in CANDIDATES.iter().enumerate() {
        let output = resolve_component(*candidate, task, state)?;
        let mut environment = TargetVerifierEnvironment::new(task, profile);
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

fn conversation_context(task: &ProbeTask, quanot_state: Option<Vec<f64>>) -> ConversationContext {
    let mut context = ConversationContext::new(task.topic.clone(), 2, quanot_state, Some(0.5));
    context.recent_text = vec![task.prompt.clone()];
    context.discussed_entities = token_set(&task.prompt).into_iter().take(8).collect();
    context
}

fn resolution_score(output: &str, target: &str) -> f64 {
    let output_lower = output.to_ascii_lowercase();
    let target_lower = target.to_ascii_lowercase();
    if output_lower.contains(target_lower.trim_end_matches('.')) {
        return 1.0;
    }

    let output_tokens = token_set(output);
    let target_tokens = token_set(target);
    if output_tokens.is_empty() || target_tokens.is_empty() {
        return 0.0;
    }

    let overlap = output_tokens.intersection(&target_tokens).count() as f64;
    let precision = overlap / output_tokens.len() as f64;
    let recall = overlap / target_tokens.len() as f64;
    let mut score = if precision + recall <= f64::EPSILON {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };

    let target_negative = contains_negation(&target_tokens);
    let output_negative = contains_negation(&output_tokens);
    if target_negative != output_negative {
        score *= 0.2;
    }

    score.clamp(0.0, 1.0)
}

fn profiled_resolution_score(output: &str, target: &str, class: EventClass) -> f64 {
    let surface = resolution_score(output, target);
    let output_tokens = token_set(output);
    let target_tokens = token_set(target);
    if output_tokens.is_empty() || target_tokens.is_empty() {
        return surface;
    }

    match class {
        EventClass::KnowledgeGap => surface,
        EventClass::PredictionContradiction => {
            let negation_match =
                contains_negation(&output_tokens) == contains_negation(&target_tokens);
            let correction_overlap = output_tokens.intersection(&target_tokens).count() as f64
                / target_tokens.len() as f64;
            let contradiction_score = if negation_match {
                correction_overlap
            } else {
                correction_overlap * 0.2
            };
            surface.max(contradiction_score).clamp(0.0, 1.0)
        }
        EventClass::QuanotTrajectory => {
            let mechanism_tokens = [
                "cause",
                "causes",
                "caused",
                "causing",
                "increase",
                "increased",
                "reduce",
                "reduced",
                "higher",
                "slower",
                "heat",
            ];
            let mechanism_signal = mechanism_tokens
                .iter()
                .filter(|token| output_tokens.contains(**token))
                .count() as f64
                / 2.0;
            let effect_overlap = output_tokens.intersection(&target_tokens).count() as f64
                / target_tokens.len() as f64;
            let causal_score =
                (0.70 * effect_overlap + 0.30 * mechanism_signal.min(1.0)).clamp(0.0, 1.0);
            surface.max(causal_score).clamp(0.0, 1.0)
        }
    }
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

fn contains_negation(tokens: &HashSet<String>) -> bool {
    ["not", "never", "no", "without", "false"]
        .iter()
        .any(|negation| tokens.contains(*negation))
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
