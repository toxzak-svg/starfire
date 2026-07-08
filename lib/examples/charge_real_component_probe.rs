use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use star::causal::CausalEngine;
use star::charge::{
    knowledge_gap_charge, prediction_contradiction_charge, Charge, ChargeLedger,
    ChargeRoutingSignature, QuanotTrajectoryEmitter, Resolution, ResolverStats,
};
use star::metacog::{KnowledgeGap, MetaCognition};
use star::persistence::{Memory, MemoryDomain, Store};
use star::prediction::{
    ConversationContext, Evidence, PredictionCenter, PredictionOutcome,
};
use star::quanot::Quanot;
use star::reasoning::ReasoningEngine;

const SOLVE_SCORE: f64 = 0.70;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EventClass {
    QuanotTrajectory,
    PredictionContradiction,
    KnowledgeGap,
}

impl EventClass {
    fn name(self) -> &'static str {
        match self {
            Self::QuanotTrajectory => "quanot_trajectory",
            Self::PredictionContradiction => "prediction_contradiction",
            Self::KnowledgeGap => "knowledge_gap",
        }
    }

    fn scrambled(self) -> Self {
        match self {
            Self::QuanotTrajectory => Self::KnowledgeGap,
            Self::PredictionContradiction => Self::QuanotTrajectory,
            Self::KnowledgeGap => Self::PredictionContradiction,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ProbeTask {
    class: EventClass,
    topic: &'static str,
    prompt: &'static str,
    target: &'static str,
    lead_a: &'static str,
    lead_b: &'static str,
}

const TRAIN_TASKS: [ProbeTask; 12] = [
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "dns",
        prompt: "What does DNS do?",
        target: "DNS resolves domain names to IP addresses.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "mutex",
        prompt: "What does a mutex protect?",
        target: "A mutex protects shared state from concurrent access.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "photosynthesis",
        prompt: "What gas do plants absorb during photosynthesis?",
        target: "Plants absorb carbon dioxide during photosynthesis.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "heart",
        prompt: "What organ pumps blood through the body?",
        target: "The heart pumps blood through the body.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "copper",
        prompt: "Does copper become ferromagnetic at room temperature?",
        target: "Copper is not ferromagnetic at room temperature.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "sound",
        prompt: "Does sound travel faster in air than in steel?",
        target: "Sound travels faster in steel than in air.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "dns encryption",
        prompt: "Does DNS encrypt web traffic?",
        target: "DNS resolves names while TLS encrypts web traffic.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "moonlight",
        prompt: "Does the Moon produce its own visible light?",
        target: "The Moon reflects sunlight and does not produce its own visible light.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "cache miss",
        prompt: "Why does a cache miss increase latency?",
        target: "A cache miss causes slower memory fetch and increased latency.",
        lead_a: "The processor requests data from a cache.",
        lead_b: "The requested cache line is absent.",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "heavy rain",
        prompt: "Why can heavy rain raise river levels?",
        target: "Heavy rain causes increased runoff and higher river levels.",
        lead_a: "Rain continues over already wet ground.",
        lead_b: "The soil absorbs less additional water.",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "coolant",
        prompt: "Why does insufficient coolant cause thermal throttling?",
        target: "Insufficient coolant causes higher temperature and thermal throttling.",
        lead_a: "A processor is operating under sustained load.",
        lead_b: "Cooling capacity falls below heat production.",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "packet loss",
        prompt: "Why does packet loss increase network latency?",
        target: "Packet loss causes retransmission and increased latency.",
        lead_a: "A sender transmits a sequence of packets.",
        lead_b: "Some packets fail to reach the receiver.",
    },
];

const TEST_TASKS: [ProbeTask; 6] = [
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "cache",
        prompt: "What is a cache used for?",
        target: "A cache stores frequently used data for faster access.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::KnowledgeGap,
        topic: "compiler",
        prompt: "What does a compiler translate?",
        target: "A compiler translates source code into machine code.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "borrow checker",
        prompt: "Does Rust's borrow checker enforce ownership only at runtime?",
        target: "Rust's borrow checker enforces many ownership rules at compile time, not only runtime.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::PredictionContradiction,
        topic: "mutex writes",
        prompt: "Does a mutex permit unlimited concurrent writes to shared state?",
        target: "A mutex serializes access to shared state and does not permit unlimited concurrent writes.",
        lead_a: "",
        lead_b: "",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "pipe pressure",
        prompt: "Why does low pipe pressure reduce flow?",
        target: "Low pipe pressure causes reduced flow.",
        lead_a: "Fluid is moving through a pipe.",
        lead_b: "The pressure difference across the pipe falls.",
    },
    ProbeTask {
        class: EventClass::QuanotTrajectory,
        topic: "combustion",
        prompt: "Why does combustion produce heat?",
        target: "Combustion causes heat release.",
        lead_a: "Fuel and oxygen are present together.",
        lead_b: "A combustion reaction begins.",
    },
];

const GAP_MEMORIES: [&str; 6] = [
    "DNS resolves domain names to IP addresses.",
    "A mutex protects shared state from concurrent access.",
    "Plants absorb carbon dioxide during photosynthesis.",
    "The heart pumps blood through the body.",
    "A cache stores frequently used data for faster access.",
    "A compiler translates source code into machine code.",
];

const CONTRADICTION_MEMORIES: [&str; 6] = [
    "Copper is not ferromagnetic at room temperature.",
    "Sound travels faster in steel than in air.",
    "DNS resolves names while TLS encrypts web traffic.",
    "The Moon reflects sunlight and does not produce its own visible light.",
    "Rust's borrow checker enforces many ownership rules at compile time, not only runtime.",
    "A mutex serializes access to shared state and does not permit unlimited concurrent writes.",
];

const CAUSAL_EDGES: [(&str, &str); 6] = [
    ("cache miss", "slower memory fetch and increased latency"),
    ("heavy rain", "increased runoff and higher river levels"),
    ("insufficient coolant", "higher temperature and thermal throttling"),
    ("packet loss", "retransmission and increased latency"),
    ("low pipe pressure", "reduced flow"),
    ("combustion", "heat release"),
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Policy {
    Learned,
    Fixed,
    Scrambled,
    Random,
    Oracle,
}

impl Policy {
    fn name(self) -> &'static str {
        match self {
            Self::Learned => "learned",
            Self::Fixed => "fixed",
            Self::Scrambled => "scrambled",
            Self::Random => "random",
            Self::Oracle => "oracle",
        }
    }
}

const POLICIES: [Policy; 5] = [
    Policy::Learned,
    Policy::Fixed,
    Policy::Scrambled,
    Policy::Random,
    Policy::Oracle,
];

struct ProbeState {
    store: Store,
    reasoning_memories: Vec<Memory>,
    db_path: PathBuf,
}

impl ProbeState {
    fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = std::env::temp_dir().join(format!(
            "starfire-charge-real-probe-{}-{}.db",
            std::process::id(),
            star::now_timestamp()
        ));
        remove_sqlite_files(&db_path);
        let store = Store::open(&db_path)?;

        for fact in GAP_MEMORIES {
            let memory = Memory::new(fact, MemoryDomain::Empirical, 0.9)
                .with_confidence(0.95)
                .with_provenance("charge-real-component-probe");
            store.insert_memory(&memory)?;
        }

        let reasoning_memories = CONTRADICTION_MEMORIES
            .iter()
            .map(|fact| {
                Memory::new(fact, MemoryDomain::Empirical, 0.9)
                    .with_confidence(0.95)
                    .with_provenance("charge-real-component-probe")
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
struct ScoredEvent {
    class: EventClass,
    charge: Charge,
    scores: BTreeMap<Candidate, f64>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct StrategyMetrics {
    mean_discharge: f64,
    mean_remaining: f64,
    solve_rate: f64,
    oracle_selection_agreement: f64,
    max_conservation_error: f64,
}

#[derive(Debug, Serialize)]
struct Diagnostics {
    distinct_signature_leaders: usize,
    learned_discharge_vs_fixed: f64,
    learned_discharge_vs_scrambled: f64,
    learned_remaining_ratio_vs_fixed: f64,
    learned_oracle_selection_agreement: f64,
    max_conservation_error: f64,
}

#[derive(Debug, Serialize)]
struct Criteria {
    all_real_emitters_produced_charge: bool,
    three_distinct_signature_leaders: bool,
    learned_beats_fixed_discharge_1_25x: bool,
    learned_beats_scrambled_discharge_1_25x: bool,
    learned_remaining_le_75pct_fixed: bool,
    learned_matches_oracle_selection_66pct: bool,
    conserves_charge: bool,
}

impl Criteria {
    fn passed(&self) -> bool {
        self.all_real_emitters_produced_charge
            && self.three_distinct_signature_leaders
            && self.learned_beats_fixed_discharge_1_25x
            && self.learned_beats_scrambled_discharge_1_25x
            && self.learned_remaining_le_75pct_fixed
            && self.learned_matches_oracle_selection_66pct
            && self.conserves_charge
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    train_events: usize,
    held_out_events: usize,
    routing_identity: &'static str,
    signature_leaders: BTreeMap<String, String>,
    strategies: BTreeMap<String, StrategyMetrics>,
    diagnostics: Diagnostics,
    criteria: Criteria,
    passed: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let state = ProbeState::new()?;
    let mut profiles = vec![ResolverStats::default(); CANDIDATES.len()];
    let mut global_discharge = vec![0.0f64; CANDIDATES.len()];
    let mut route_by_class = BTreeMap::new();
    let mut emitted_train = 0usize;

    for task in TRAIN_TASKS {
        let charge = emit_real_charge(&task)?;
        emitted_train += usize::from(charge.magnitude > 0.0);
        let route = ChargeRoutingSignature::from_charge(&charge);
        route_by_class.insert(task.class, route);

        for (index, candidate) in CANDIDATES.iter().enumerate() {
            let output = resolve_component(*candidate, &task, &state)?;
            let score = resolution_score(&output, task.target);
            let discharged = charge.magnitude * score as f32;
            let resolution = Resolution {
                discharged,
                emitted: Vec::new(),
                permitted_decay: 0.0,
                compute_cost: 1,
            };
            profiles[index].observe(&charge, &resolution);
            global_discharge[index] += discharged as f64;
        }
    }

    let fixed_index = max_index(&global_discharge).ok_or("no fixed resolver")?;
    let mut signature_leaders = BTreeMap::new();
    let mut distinct_leaders = BTreeSet::new();
    for (class, route) in &route_by_class {
        let leader = profile_best(&profiles, route).ok_or("missing routing profile")?;
        distinct_leaders.insert(leader);
        signature_leaders.insert(
            class.name().to_string(),
            CANDIDATES[leader].name().to_string(),
        );
    }

    let mut held_out = Vec::new();
    let mut emitted_test = 0usize;
    for task in TEST_TASKS {
        let charge = emit_real_charge(&task)?;
        emitted_test += usize::from(charge.magnitude > 0.0);
        let mut scores = BTreeMap::new();
        for candidate in CANDIDATES {
            let output = resolve_component(candidate, &task, &state)?;
            scores.insert(candidate, resolution_score(&output, task.target));
        }
        held_out.push(ScoredEvent {
            class: task.class,
            charge,
            scores,
        });
    }

    let mut strategies = BTreeMap::new();
    for policy in POLICIES {
        strategies.insert(
            policy.name().to_string(),
            evaluate_policy(
                policy,
                &held_out,
                &profiles,
                fixed_index,
                &route_by_class,
            )?,
        );
    }

    let learned = strategies["learned"];
    let fixed = strategies["fixed"];
    let scrambled = strategies["scrambled"];
    let max_conservation_error = strategies
        .values()
        .map(|metrics| metrics.max_conservation_error)
        .fold(0.0, f64::max);

    let diagnostics = Diagnostics {
        distinct_signature_leaders: distinct_leaders.len(),
        learned_discharge_vs_fixed: learned.mean_discharge / fixed.mean_discharge.max(f64::EPSILON),
        learned_discharge_vs_scrambled: learned.mean_discharge
            / scrambled.mean_discharge.max(f64::EPSILON),
        learned_remaining_ratio_vs_fixed: learned.mean_remaining
            / fixed.mean_remaining.max(f64::EPSILON),
        learned_oracle_selection_agreement: learned.oracle_selection_agreement,
        max_conservation_error,
    };

    let criteria = Criteria {
        all_real_emitters_produced_charge: emitted_train == TRAIN_TASKS.len()
            && emitted_test == TEST_TASKS.len(),
        three_distinct_signature_leaders: diagnostics.distinct_signature_leaders == 3,
        learned_beats_fixed_discharge_1_25x: diagnostics.learned_discharge_vs_fixed >= 1.25,
        learned_beats_scrambled_discharge_1_25x: diagnostics.learned_discharge_vs_scrambled >= 1.25,
        learned_remaining_le_75pct_fixed: diagnostics.learned_remaining_ratio_vs_fixed <= 0.75,
        learned_matches_oracle_selection_66pct: diagnostics.learned_oracle_selection_agreement
            >= 2.0 / 3.0,
        conserves_charge: diagnostics.max_conservation_error <= 1e-5,
    };
    let passed = criteria.passed();

    let report = Report {
        experiment: "charge-real-component-specialization-v1",
        train_events: TRAIN_TASKS.len(),
        held_out_events: TEST_TASKS.len(),
        routing_identity: "charge kind + coarse scope class; exact scope retained for provenance",
        signature_leaders,
        strategies,
        diagnostics,
        criteria,
        passed,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.passed {
        std::process::exit(1);
    }
    Ok(())
}

fn emit_real_charge(task: &ProbeTask) -> Result<Charge, Box<dyn Error>> {
    match task.class {
        EventClass::KnowledgeGap => {
            let mut metacog = MetaCognition::new();
            metacog.note_gap(KnowledgeGap::new(task.topic, 0.85));
            let gap = metacog.top_gap().ok_or("metacognition did not retain gap")?;
            knowledge_gap_charge(gap).ok_or_else(|| "gap emitter returned no charge".into())
        }
        EventClass::PredictionContradiction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(task.prompt);
            let mut center = PredictionCenter::new();
            let context = conversation_context(task, Some(state.reservoir_state));
            let predictions = center.generate(&context);
            let prediction = predictions.first().ok_or("prediction center emitted no prediction")?;
            let charge = prediction_contradiction_charge(
                prediction,
                PredictionOutcome::Refuted,
                task.target,
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
            let first = quanot.process(task.lead_a);
            let second = quanot.process(task.lead_b);
            let third = quanot.process(task.prompt);
            let _ = emitter.observe(&first);
            let _ = emitter.observe(&second);
            emitter
                .observe(&third)
                .ok_or_else(|| "Quanot trajectory emitter returned no charge".into())
        }
    }
}

fn resolve_component(
    candidate: Candidate,
    task: &ProbeTask,
    state: &ProbeState,
) -> Result<String, Box<dyn Error>> {
    match candidate {
        Candidate::Reasoning => {
            let mut engine = ReasoningEngine::new();
            let result = engine.reason(task.prompt, &state.reasoning_memories);
            let mut output = result.answer.unwrap_or_default();
            if !result.reasoning_chain.is_empty() {
                output.push(' ');
                output.push_str(&result.reasoning_chain.join(" "));
            }
            Ok(output)
        }
        Candidate::Memory => {
            let memories = state.store.search_memories(task.topic, 3, None)?;
            Ok(memories
                .iter()
                .map(|memory| memory.content.as_str())
                .collect::<Vec<_>>()
                .join(" "))
        }
        Candidate::Causal => Ok(resolve_causal(task)),
        Candidate::Prediction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(task.prompt);
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
            metacog.note_curiosity(task.topic, task.prompt);
            Ok(metacog
                .curiosity_question(task.topic)
                .map(|intent| intent.format())
                .unwrap_or_default())
        }
    }
}

fn resolve_causal(task: &ProbeTask) -> String {
    let mut engine = CausalEngine::new();
    for (cause, effect) in CAUSAL_EDGES {
        engine.add_edge(cause, effect, 0.9, Some(1));
    }

    let prompt_tokens = token_set(task.prompt);
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
    let mut context = ConversationContext::new(task.topic.to_string(), 2, quanot_state, Some(0.5));
    context.recent_text = vec![task.prompt.to_string()];
    context.discussed_entities = token_set(task.prompt).into_iter().take(8).collect();
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

fn token_set(text: &str) -> HashSet<String> {
    const STOPWORDS: [&str; 22] = [
        "a", "an", "and", "are", "at", "be", "by", "do", "does", "for", "from", "in",
        "is", "it", "of", "on", "only", "the", "to", "used", "what", "why",
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

fn profile_best(
    profiles: &[ResolverStats],
    signature: &ChargeRoutingSignature,
) -> Option<usize> {
    (0..profiles.len()).max_by(|left, right| {
        let left_score = profiles[*left]
            .routing_efficiency_for(signature)
            .unwrap_or(0.0);
        let right_score = profiles[*right]
            .routing_efficiency_for(signature)
            .unwrap_or(0.0);
        left_score
            .total_cmp(&right_score)
            .then_with(|| left.cmp(right))
    })
}

fn evaluate_policy(
    policy: Policy,
    events: &[ScoredEvent],
    profiles: &[ResolverStats],
    fixed_index: usize,
    route_by_class: &BTreeMap<EventClass, ChargeRoutingSignature>,
) -> Result<StrategyMetrics, Box<dyn Error>> {
    let mut total_discharge = 0.0f64;
    let mut total_remaining = 0.0f64;
    let mut solved = 0usize;
    let mut oracle_agreement = 0usize;
    let mut max_conservation_error = 0.0f64;

    for (event_index, event) in events.iter().enumerate() {
        let oracle_index = max_score_candidate(&event.scores).ok_or("event had no resolver scores")?;
        let route = route_by_class
            .get(&event.class)
            .ok_or("missing route for event class")?;
        let selected_index = match policy {
            Policy::Learned => profile_best(profiles, route).ok_or("no learned resolver")?,
            Policy::Fixed => fixed_index,
            Policy::Scrambled => {
                let scrambled_route = route_by_class
                    .get(&event.class.scrambled())
                    .ok_or("missing scrambled route")?;
                profile_best(profiles, scrambled_route).ok_or("no scrambled resolver")?
            }
            Policy::Random => event_index % CANDIDATES.len(),
            Policy::Oracle => oracle_index,
        };

        let candidate = CANDIDATES[selected_index];
        let score = *event.scores.get(&candidate).unwrap_or(&0.0);
        let mut ledger = ChargeLedger::default();
        let issued = ledger.issue(event.charge.clone())?;
        let discharged = issued.magnitude * score as f32;
        let (receipt, _) = ledger.record_resolution(
            issued.id,
            Resolution {
                discharged,
                emitted: Vec::new(),
                permitted_decay: 0.0,
                compute_cost: 1,
            },
        )?;

        total_discharge += discharged as f64;
        total_remaining += receipt.remaining as f64;
        solved += usize::from(score >= SOLVE_SCORE);
        oracle_agreement += usize::from(selected_index == oracle_index);
        let conservation_error =
            (receipt.incoming - receipt.discharged - receipt.emitted - receipt.permitted_decay
                - receipt.remaining)
                .abs() as f64;
        max_conservation_error = max_conservation_error.max(conservation_error);
    }

    let count = events.len() as f64;
    Ok(StrategyMetrics {
        mean_discharge: total_discharge / count,
        mean_remaining: total_remaining / count,
        solve_rate: solved as f64 / count,
        oracle_selection_agreement: oracle_agreement as f64 / count,
        max_conservation_error,
    })
}

fn max_score_candidate(scores: &BTreeMap<Candidate, f64>) -> Option<usize> {
    CANDIDATES
        .iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            scores
                .get(left)
                .copied()
                .unwrap_or(0.0)
                .total_cmp(&scores.get(right).copied().unwrap_or(0.0))
                .then_with(|| left_index.cmp(right_index))
        })
        .map(|(index, _)| index)
}

fn max_index(values: &[f64]) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            left.total_cmp(right)
                .then_with(|| left_index.cmp(right_index))
        })
        .map(|(index, _)| index)
}

fn remove_sqlite_files(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
