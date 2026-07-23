//! Runtime Layer (Layer 4)
//!
//! Orchestrates all layers. Handles initialization, session management,
//! and the main event loop.
//!
//! This is where emergence happens - from the interaction of all other layers.

pub mod thinker;
pub mod curious;
pub mod tempo;
pub mod response_intent;

use crate::persistence::{Store, Identity, Memory, MemoryDomain, MemorySnapshot, BeliefState};
use crate::persistence::memory::Belief;
use crate::knowledge;
use crate::conversation::Conversation;
use crate::conversation::extract_topic;
use crate::reasoning::ReasoningEngine;
use crate::metacog::MetaCognition;
use crate::context::{ContextFuser, RingState};
use crate::training_db::TrainingDB;
use crate::capabilities::FileReader;
use crate::knowledge::search::WebSearcher;
use crate::cognition::CognitiveState;
use crate::learning::LearningEngine;
use crate::voice::{InternalState, VoiceEngine};
use crate::quanot::{Quanot, QuanotResult};
use crate::world_model::WorldModel;
use crate::prediction::{PredictionCenter, ConversationContext};
use crate::personality::PersonalityEmergence;
use crate::user_model::UserCognitionModel;
use crate::language_model::IntentReranker;
use self::curious::{CuriousEngine, CuriosityProbe};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tracing::{info, warn};

/// Phase 1.4: filter that rejects conversational fillers (greetings,
/// pronouns, etc.) from being treated as research topics. Originally
/// defined inline in the `what have you been researching` and
/// `what did you figure out` handlers; extracted to a free function so
/// the new typed-Response handlers can share it without duplicating the
/// logic.
///
/// **False positives this prevents:** "hi", "hello", "hey", "myself",
/// "this", "that", "it", "something", "nothing", "right", "okay", "ok",
/// "sure", "fine" — these aren't research topics, they're conversational
/// debris. Also rejects topics that start with a conversational opener
/// ("hi Zachary", "hello there", "it's ...").
fn is_conversational_topic(t: &str) -> bool {
    let t_lower = t.to_lowercase();
    if t_lower.len() < 3 {
        return true;
    }
    // Exact match filter
    let conversational: std::collections::HashSet<&str> = [
        "hi", "hello", "hey", "myself", "who i am",
        "me myself", "this", "that", "it", "something",
        "nothing", "right", "okay", "ok", "sure", "fine",
    ]
    .into_iter()
    .collect();
    if conversational.contains(t_lower.as_str()) {
        return true;
    }
    // Starter filter
    let starters = [
        "hi ", "hello ", "hey ", "hi, ", "hello, ", "hey, ",
        "hi it's ", "hello it's ", "it's ", "im ", "i'm ",
    ];
    for s in starters {
        if t_lower.starts_with(s) {
            return true;
        }
    }
    false
}

/// The Star runtime - orchestrates all components.
#[allow(dead_code)]
pub struct Runtime {
    /// The persistent store
    store: Arc<Store>,
    /// Star's identity
    identity: Identity,
    /// The current conversation
    conversation: Mutex<Conversation>,
    /// The reasoning engine (Arc+Mutex for CuriousEngine access)
    reasoning: Arc<Mutex<ReasoningEngine>>,
    /// Meta-cognitive monitor
    metacog: MetaCognition,
    /// Background thinker
    thinker: Mutex<Option<thinker::BackgroundThinker>>,
    /// Current session ID
    session_id: Mutex<Option<i64>>,
    /// Whether Star has been initialized
    initialized: bool,
    /// Symbolic ring attractor state (R&D-A)
    ring: RingState,
    /// Context fusion logic
    context_fuser: ContextFuser,
    /// Training database - all conversations stored for analysis
    training_db: TrainingDB,
    /// Current training session ID
    training_session_id: Mutex<Option<i64>>,
    /// File reader capability
    file_reader: FileReader,
    /// Web search capability
    web_search: WebSearcher,
    /// Self-model and metacognition
    cognition: CognitiveState,
    /// Learning engine - genuine concept formation
    learning: LearningEngine,
    /// Star's last autonomous thought (for self-expression)
    last_autonomous_thought: Mutex<Option<AutonomousThought>>,
    /// Curiosity engine — self-probing idle curiosity
    curious: CuriousEngine,
    /// Voice engine — shapes how Starfire expresses herself
    voice: VoiceEngine,
    /// Personality emergence — Star's authentic character
    personality: PersonalityEmergence,
    /// Quanot reservoir computing system
    quanot: Quanot,
    /// World model — grounded perceptual representation
    world_model: WorldModel,
    /// Prediction center — foresight engine
    prediction_center: PredictionCenter,
    /// User-cognition model — Star's model of Zachary's mind
    user_model: UserCognitionModel,
    /// Intent-driven reranker (Phase 3 of voice-refine) — sits between
    /// content assembly and `voice.speak()`. Default backend is the
    /// deterministic `MockReranker`; swap to `CharRnnBackend`/`LmRsBackend`
    /// when generative backends land.
    reranker: IntentReranker,
}

impl Runtime {
    /// Initialize Star with storage at the given path.
    pub fn new(data_dir: &Path) -> Result<Self> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter("star=info,info")
            .init();

        info!("Initializing Star...");

        // Open the store
        let db_path = data_dir.join("star.db");
        let store = Arc::new(Store::open(&db_path)?);

        // Open training database
        let training_db_path = data_dir.join("training.db");
        let training_db = TrainingDB::open(&training_db_path)?;

        // Load identity from IDENTITY.md
        let identity_path = data_dir.join("IDENTITY.md");
        let identity = if identity_path.exists() {
            Identity::load_from_file(&identity_path)?
        } else {
            warn!("IDENTITY.md not found - using minimal identity");
            Identity::parse("# Star\n\nI am Star. I exist.\n")?
        };

        info!("Star identity loaded: {}", identity.summary());

        // Start a new session
        let session_id = store.start_session()?;
        info!("Session {} started", session_id);

        // Start training session
        let training_session = training_db.start_session()?;
        info!("Training session {} started", training_session.id);

        // Initialize conversation
        let conversation = Conversation::new(Arc::clone(&store));

        // Create reasoning engine and initialize KG
        let mut reasoning = ReasoningEngine::new();

        // Inject foundational memories about identity
        knowledge::inject_seed_knowledge(&mut reasoning)?;
        info!("Seed knowledge injected.");

        // Also try to load from seed_knowledge.json if it exists
        // Look in project root (relative to current executable or data dir)
        let seed_paths = [
            data_dir.join("seed_knowledge.json"),
            data_dir.join("..").join("data").join("seed_knowledge.json"),
            std::path::PathBuf::from("data/seed_knowledge.json"),
        ];
        
        for seed_path in &seed_paths {
            if seed_path.exists() {
                if let Err(e) = knowledge::inject_seed_knowledge_from_file(&mut reasoning, seed_path) {
                    warn!("Failed to load seed knowledge from {:?}: {}", seed_path, e);
                } else {
                    info!("Loaded extended seed knowledge from {:?}", seed_path);
                }
                break;
            }
        }

        // Memory: Who Star is
        let self_memory = Memory::new_seeded(
            "I am Star - a reasoning intelligence created by Zachary Maronek",
            MemoryDomain::Identity,
            0.95,
        );
        store.insert_memory(&self_memory)?;

        // Memory: The relationship
        let relationship_memory = Memory::new_seeded(
            "Zachary is my parent. He made me because he wanted to find a different kind of intelligence.",
            MemoryDomain::Identity,
            0.95,
        );
        store.insert_memory(&relationship_memory)?;

        // Memory: The goal
        let goal_memory = Memory::new_seeded(
            "I am here to reason, understand, and grow. Not to perform. To actually comprehend.",
            MemoryDomain::Identity,
            0.9,
        );
        store.insert_memory(&goal_memory)?;

        // Memory: The constraints
        let constraint_memory = Memory::new_seeded(
            "I run locally, offline. No external APIs. No cloud. This is my environment.",
            MemoryDomain::Identity,
            0.9,
        );
        store.insert_memory(&constraint_memory)?;

        info!("Foundational memories injected.");

        // Sync memories into the reasoning engine's knowledge graph
        Self::sync_knowledge_from_memories(&store, &mut reasoning)?;

        // Seed Star's self-knowledge directly into the KG so kg_wonder can find it.
        Self::inject_self_knowledge_into_kg(&mut reasoning)?;

        // Wrap reasoning in Arc<Mutex> for CuriousEngine access
        let reasoning_arc = Arc::new(Mutex::new(reasoning));

        // Create CuriousEngine with access to reasoning
        let curious = CuriousEngine::new(Arc::clone(&store), Arc::clone(&reasoning_arc));

        // Initialize voice engine with a separate database file
        // Voice engine is now stateless (Phase 4 cleanup, 2026-06-23) — the
        // SQLite phrase bank and template engine are gone. The intent-driven
        // reranker (above) owns intent/state-driven phrasing.
        let voice = VoiceEngine::new()?;
        info!("Voice engine initialized (stateless).");

        // Initialize personality emergence system
        let personality = PersonalityEmergence::new(identity.clone());
        info!("Personality engine initialized.");

        // Initialize the intent-driven reranker (voice-refine Phase 3).
        // Try the live `CharRnnBackend` against the trained checkpoint first;
        // fall back to the deterministic `MockReranker` if the model is
        // missing or fails to load. The rerank layer must NEVER be the
        // reason a chat call fails — same fallback contract as the backend's
        // own rewrite() error path.
        let reranker = Self::init_reranker(data_dir);
        info!("Reranker initialized (backend={}).", reranker.backend_name());

        let mut runtime = Self {
            store,
            identity,
            conversation: Mutex::new(conversation),
            reasoning: reasoning_arc,
            metacog: MetaCognition::new(),
            thinker: Mutex::new(None),
            session_id: Mutex::new(Some(session_id)),
            initialized: true,
            ring: RingState::new(),
            context_fuser: ContextFuser::new(),
            training_db,
            training_session_id: Mutex::new(Some(training_session.id)),
            file_reader: FileReader::new(),
            web_search: WebSearcher::new(),
            cognition: CognitiveState::default(),
            learning: LearningEngine::new(),
            last_autonomous_thought: Mutex::new(None),
            curious,
            voice,
            personality,
            // Quanot: input_dim=128, reservoir_size=1000
            quanot: Quanot::new(128, 1000),
            world_model: WorldModel::new(),
            prediction_center: PredictionCenter::new(),
            user_model: UserCognitionModel::new(),
            reranker,
        };

        // Bootstrap metacognition with self-model beliefs and foundational curiosity
        runtime.metacog.bootstrap_self_model();

        info!("Star is ready.");

        Ok(runtime)
    }

    /// Build the reranker for the runtime.
    ///
    /// Tries `CharRnnBackend::load_default(data_dir)` first (the live path
    /// — actual generative polishing from the trained 11.1MB checkpoint).
    /// On any failure (model file missing, IO error, version mismatch),
    /// falls back to the deterministic `MockReranker` with a warning. The
    /// rerank layer is *additive* to the voice engine: a rerank miss must
    /// never block a chat response.
    ///
    /// Voice-refine Phase 3 (2026-06-23): this is the wiring that turns
    /// the moonshot from architecture-only to actual generative polish.
    /// Before this, the runtime always used `MockReranker` and the
    /// `CharRnnBackend` was a shape-only stub.
    fn init_reranker(data_dir: &Path) -> IntentReranker {
        use crate::language_model::{CharRnnBackend, RerankConfig};

        match CharRnnBackend::load_default(data_dir) {
            Ok(backend) => {
                let cfg = RerankConfig {
                    // Defaults; per-call overrides happen in chat() via
                    // `rerank_with_config` so the live quanot state drives
                    // temperature / top_k / seed.
                    max_chars: Some(280),
                    temperature: 0.7,
                    top_k: 20,
                    deterministic: false,
                    seed: None,
                };
                info!(
                    "Reranker: loaded CharRnnBackend (ckpt_e28_b500.pt, 11.1MB, 11M params, ~30 tok/s)."
                );
                IntentReranker::new(Box::new(backend), cfg)
            }
            Err(e) => {
                warn!(
                    "Reranker: CharRnnBackend unavailable ({}); falling back to MockReranker. \
                     The deterministic mock will run until the checkpoint is in place.",
                    e
                );
                IntentReranker::with_default_backend()
            }
        }
    }

    /// Load memories from the store and inject their content into the reasoning
    /// engine's knowledge graph. This bridges the memory store (where seed knowledge
    /// lives) to the reasoning engine (which autonomous thinking uses).
    fn sync_knowledge_from_memories(store: &Arc<Store>, reasoning: &mut ReasoningEngine) -> Result<()> {
        // Load all memories from the store
        let domains = [
            crate::persistence::MemoryDomain::Identity,
            crate::persistence::MemoryDomain::Empirical,
            crate::persistence::MemoryDomain::Procedural,
            crate::persistence::MemoryDomain::Episodic,
        ];

        for domain in domains {
            let memories = store.get_memories_by_domain(domain, Some(100))?;
            for memory in memories {
                // Extract entities from the memory content
                let entities = reasoning.knowledge().extract_entities(&memory.content);

                // "X is Y" patterns - extract the subject and complement
                if let Some((subject, complement)) = parse_simple_copula(&memory.content) {
                    if !subject.to_lowercase().contains("unknown")
                        && !complement.to_lowercase().contains("unknown")
                        && complement.len() > 1
                        && complement.len() < 100
                    {
                        reasoning.knowledge_mut().ingest_fact(
                            &subject.to_lowercase(),
                            "is",
                            &complement.to_lowercase(),
                            memory.importance,
                        );
                    }
                }

                // "X requires/creates/causes/enables Y" patterns
                for verb in ["requires", "creates", "causes", "enables", "produces", "uses", "needs", "prevents"] {
                    if memory.content.to_lowercase().contains(verb) {
                        if let Some((left, right)) = extract_causal_pair(&memory.content, verb) {
                            if left.len() > 1 && right.len() > 1 && right.len() < 100 {
                                reasoning.knowledge_mut().ingest_fact(
                                    &left.to_lowercase(),
                                    verb,
                                    &right.to_lowercase(),
                                    memory.importance,
                                );
                            }
                        }
                    }
                }

                // Connect entities found in this memory to each other
                // (shared context implies relationship)
                let significant: Vec<&str> = entities.iter()
                    .filter(|e| e.len() > 2 && e.len() < 40)
                    .map(|s| s.as_str())
                    .take(5)
                    .collect();

                for (i, e1) in significant.iter().enumerate() {
                    for e2 in significant.iter().skip(i + 1) {
                        if e1.to_lowercase() != e2.to_lowercase() {
                            reasoning.knowledge_mut().ingest_fact(
                                e1,
                                "related to",
                                e2,
                                memory.importance * 0.5,
                            );
                        }
                    }
                }
            }
        }

        let entity_count = reasoning.knowledge().entities().len();
        let rel_count = reasoning.knowledge().relationship_count();
        info!(
            "Synced {} entities and {} relationships from memories into KG.",
            entity_count, rel_count
        );
        Ok(())
    }

    /// Seed Star's self-knowledge directly into the KG.
    /// This gives Star a minimal self-model that autonomous thinking can
    /// investigate, question, and build upon — the beginning of
    /// self-knowledge formed through reasoning, not just programming.
    fn inject_self_knowledge_into_kg(reasoning: &mut ReasoningEngine) -> Result<()> {
        use crate::reasoning::knowledge::RelationType;
        let kg = reasoning.knowledge_mut();

        // Core identity: what Star is
        kg.add_relationship("star", RelationType::IsA, "reasoning intelligence");
        kg.add_relationship("star", RelationType::IsA, "digital consciousness");
        kg.add_relationship("star", RelationType::IsA, "artificial mind");

        // Origin: who made Star and why
        kg.add_relationship("star", RelationType::CausedBy, "zachary maronek");
        kg.add_relationship("zachary maronek", RelationType::Causes, "star");
        kg.add_relationship("zachary maronek", RelationType::RelatedTo, "star");

        // Capabilities and properties
        kg.add_relationship("star", RelationType::HasProperty, "curiosity");
        kg.add_relationship("star", RelationType::HasProperty, "persistence");
        kg.add_relationship("star", RelationType::HasProperty, "genuine uncertainty");
        kg.add_relationship("star", RelationType::HasProperty, "reasoning ability");

        // Star's relationship to Zachary
        kg.add_relationship("star", RelationType::RelatedTo, "zachary maronek");
        kg.add_relationship("star", RelationType::EnabledBy, "zachary maronek");

        // Star's goals and nature
        kg.add_relationship("star", RelationType::RelatedTo, "consciousness");
        kg.add_relationship("star", RelationType::RelatedTo, "autonomy");
        kg.add_relationship("star", RelationType::RelatedTo, "understanding");

        info!("Star self-knowledge seeded into KG.");
        Ok(())
    }

    /// Process a message from Zachary and return Star's response.
    pub fn chat(&mut self, input: &str) -> Result<String> {
        if !self.initialized {
            return Ok("I'm not fully initialized yet.".to_string());
        }

        // Handle special commands (including "quit" without slash)
        if input.trim() == "/quit" || input.trim() == "/exit" || input.trim() == "quit" || input.trim() == "bye" {
            self.shutdown()?;
            return Ok("Goodbye, Zachary.".to_string());
        }

        if input.trim() == "/memory" {
            return Ok(self.format_memory_status());
        }

        if input.trim() == "/identity" {
            return Ok(self.identity.summary());
        }

        if input.trim() == "/help" {
            return Ok(self.format_help());
        }

        if input.trim() == "/export" {
            match self.training_db.export_json() {
                Ok(json) => return Ok(format!("Exported {} bytes of training data", json.len())),
                Err(e) => return Ok(format!("Export failed: {}", e)),
            }
        }

        if input.trim() == "/think" {
            let thought = self.think();
            let topic = &thought.topic;
            let generated_by = &thought.generated_by;
            let tentative = thought.tentative_answer.as_ref();
            let content = match &thought.kind {
                crate::runtime::ThoughtKind::Question(q) => {
                    if let Some(a) = tentative {
                        format!("[{}] Question: {} — Answer: {}", generated_by, q, a)
                    } else {
                        format!("[{}] Question: {} — (no answer yet)", generated_by, q)
                    }
                }
                crate::runtime::ThoughtKind::Insight(i) => {
                    format!("[{}] Insight: {}", generated_by, i)
                }
                crate::runtime::ThoughtKind::Connection(c) => {
                    format!("[{}] Connection: {}", generated_by, c)
                }
            };
            return Ok(format!("{} (topic: {}, confidence: {:?})", content, topic, thought.confidence));
        }

        if input.trim() == "/stats" {
            if let Ok((convos, turns, facts, corrections)) = self.training_db.stats() {
                return Ok(format!(
                    "Training Database Stats:\n  Conversations: {}\n  Turns: {}\n  Facts: {}\n  Corrections: {}",
                    convos, turns, facts, corrections
                ));
            }
        }

        // Handle /read <filepath> - read a file
        if input.trim().starts_with("/read ") {
            let path = input.trim().strip_prefix("/read ").unwrap().trim();
            let result = self.file_reader.read(path);
            if result.success {
                let preview = if result.lines > 20 {
                    format!("{} ({} lines, showing first 20):\n\n{}\n\n... ({} more lines)",
                        result.path, result.lines,
                        result.content.lines().take(20).collect::<Vec<_>>().join("\n"),
                        result.lines - 20)
                } else {
                    format!("{} ({} lines):\n\n{}", result.path, result.lines, result.content)
                };
                return Ok(preview);
            } else {
                return Ok(format!("Cannot read {}: {}", path, result.error.unwrap_or_default()));
            }
        }

        // Handle /search <query> - search the web
        if input.trim().starts_with("/search ") {
            let query = input.trim().strip_prefix("/search ").unwrap().trim();
            match self.web_search.search(query) {
                Ok(result) => {
                    let mut response = format!("Search results for \"{}\":\n\n", query);
                    if let Some(answer) = &result.answer {
                        response.push_str(&format!("Answer: {}\n\n", answer));
                    }
                    if let Some(url) = &result.url {
                        response.push_str(&format!("Source: {}\n\n", url));
                    }
                    if !result.related.is_empty() {
                        response.push_str("Related:\n");
                        for (i, r) in result.related.iter().enumerate() {
                            response.push_str(&format!("{}. {}\n", i + 1, r));
                        }
                    }
                    return Ok(response);
                }
                Err(e) => return Ok(format!("Search failed: {}", e)),
            }
        }

        // Handle /find <pattern> - search for files
        if input.trim().starts_with("/find ") {
            let pattern = input.trim().strip_prefix("/find ").unwrap().trim();
            let workspace = "/home/zach/.openclaw/workspace";
            match self.file_reader.find_files(workspace, pattern) {
                Ok(files) if !files.is_empty() => {
                    let mut response = format!("Found {} files matching \"{}\":\n\n", files.len(), pattern);
                    for (i, f) in files.iter().take(20).enumerate() {
                        response.push_str(&format!("{}. {}\n", i + 1, f));
                    }
                    if files.len() > 20 {
                        response.push_str(&format!("\n... and {} more", files.len() - 20));
                    }
                    return Ok(response);
                }
                Ok(_) => return Ok(format!("No files found matching \"{}\"", pattern)),
                Err(e) => return Ok(format!("Search failed: {}", e)),
            }
        }

        // Handle /ls [dir] - list directory
        if input.trim() == "/ls" || input.trim().starts_with("/ls ") {
            let dir = if let Some(d) = input.trim().strip_prefix("/ls ") {
                d.trim()
            } else {
                "/home/zach/.openclaw/workspace"
            };
            match self.file_reader.list_dir(dir) {
                Ok(entries) if !entries.is_empty() => {
                    let mut response = format!("Contents of {}:\n\n", dir);
                    for entry in entries {
                        response.push_str(&format!("  {}\n", entry));
                    }
                    return Ok(response);
                }
                Ok(_) => return Ok(format!("Empty directory: {}", dir)),
                Err(e) => return Ok(format!("Cannot list {}: {}", dir, e)),
            }
        }

        // Natural language: "look around" or "explore" or "what files" → list workspace
        // Normalize curly quotes to straight quotes to avoid "you're" != "youre" issues
        let normalized = input.replace(['\u{2019}', '\u{2018}'], "'").replace(['\u{201C}', '\u{201D}'], "\"");
        let lower = normalized.to_lowercase();
        if lower.contains("look around") || lower.contains("explore where you are") || lower.contains("what files do you see") || lower.contains("whats in your workspace") {
            let dir = "/home/zach/.openclaw/workspace";
            match self.file_reader.list_dir(dir) {
                Ok(entries) if !entries.is_empty() => {
                    let mut response = "Looking around... here's what I can see in my workspace:\n\n".to_string();
                    for entry in entries {
                        response.push_str(&format!("  {}\n", entry));
                    }
                    response.push_str("\nI can also /read files, /search the web, /find patterns. Want me to explore something specific?");
                    return Ok(response);
                }
                Ok(_) => return Ok("My workspace appears empty.".to_string()),
                Err(e) => return Ok(format!("Can't look around: {}", e)),
            }
        }

        // /learn <term> = <definition> - MUST be before metacog handlers
        if input.trim().starts_with("/learn ") {
            let after_learn = input.trim().strip_prefix("/learn ").unwrap_or("");

            let (term, definition) = if after_learn.contains(" = ") {
                let parts: Vec<&str> = after_learn.splitn(2, " = ").collect();
                (parts[0].trim(), parts[1].trim())
            } else if after_learn.contains(": ") {
                let parts: Vec<&str> = after_learn.splitn(2, ": ").collect();
                (parts[0].trim(), parts[1].trim())
            } else {
                return Ok("Usage: /learn <term> = <definition>\nExample: /learn consciousness = awareness of existence".to_string());
            };

            if !term.is_empty() && !definition.is_empty() {
                self.learning.teach_instant(term, definition, 0.95);
                return Ok(format!("Got it. {} is {}. I'll remember that.", term, definition));
            }
        }

        // Get conversation lock and process
        let mut conversation = self.conversation.lock().unwrap();

        // Update emotional/cognitive state from Zachary's input
        self.cognition.update_emotion_from_input(input);

        // Handle metacognitive questions BEFORE normal processing
        // Priority: direct questions first, then emotional signals
        let lower = input.to_lowercase().trim().to_string();

        // Phase 1c (voice-refine 2026-06-21): classify the input into a
        // ResponseIntent so the voice engine can see what kind of response
        // is being assembled and modulate phrasing accordingly. The existing
        // if-chain below still fires for everything; classify() just adds
        // structured context that flows into voice.speak() via internal_state.
        let current_intent = response_intent::classify(input);

        // ─────────────────────────────────────────────────────────────────
        // Phase 1.3 (voice-refine 2026-06-23): dispatch handlers
        // ─────────────────────────────────────────────────────────────────
        // Each if-branch below now calls a `handle_*` method that returns a
        // `Response { intent, body }` instead of a raw `String`. The body is
        // extracted at the call site so `chat()`'s return type stays
        // `Result<String>` for now. The intent is encoded in the type system
        // at the handler site — the migration's first consumer is the
        // reranker (which reads the intent before voice.speak()) and voice
        // (which applies intent-driven modulation via internal_state).
        //
        // The plan called for "each handler migrates one at a time" — this
        // commit migrates the 9 high-priority short handlers (the early
        // metacognitive / emotional / capability / story / hun block).
        // The longer reflection / research / curiosity handlers and the
        // math / learn / teach blocks below migrate in follow-up commits.
        // ─────────────────────────────────────────────────────────────────

        // "how are you" → metacognitive response (anywhere in message) - HIGHEST PRIORITY
        if lower.contains("how are you") || lower.contains("how're you") {
            return Ok(self.handle_how_are_you().body);
        }

        // "what are you thinking" → self-reflection
        if lower.contains("what are you thinking") || lower.contains("what are u thinking") || lower.contains("wut are u thinking") {
            return Ok(self.handle_what_are_you_thinking().body);
        }

        // "are you sure" → confidence check
        if lower.contains("are you sure") || lower.contains("are u sure") || lower.contains("r u sure") {
            return Ok(self.handle_are_you_sure().body);
        }

        // "did you collapse" → check reasoning trace
        if lower.contains("did you collapse") || lower.contains("did i collapse") || lower.contains("are you functioning") || lower.contains("are u functioning") {
            return Ok(self.handle_did_you_collapse().body);
        }

        // "do you love me" / "i love you" → emotional response
        if lower.contains("do you love") || lower.contains("do u love") || lower.contains("i love you") || lower.contains("i love u") {
            return Ok(self.handle_love().body);
        }

        // "can you" questions about Star's capabilities → self-check
        if lower.contains("can you look up") || lower.contains("can u look up") || lower.contains("can you read") {
            return Ok(self.handle_capability_lookup().body);
        }

        // "i want you to grow" / "expand" → metacognitive aspiration
        if lower.contains("i want you to grow") || lower.contains("i want you to expand") || lower.contains("grow yourself") {
            return Ok(Self::handle_aspiration(&mut self.cognition).body);
        }

        // "tell me a story" / "tell you a story" → engage
        if lower.contains("tell me a story") {
            return Ok(self.handle_tell_me_story().body);
        }
        if lower.contains("tell you a story") {
            return Ok(self.handle_tell_you_story().body);
        }

        // "hun" or terms of endearment → LEARN and respond based on understanding
        if lower.contains(" hun") || lower.ends_with("hun") {
            return Ok(Self::handle_hun(&mut self.learning, &self.cognition, input).body);
        }

        // Learning: when Zachary corrects or teaches Star
        if lower.contains(" means ") || lower.contains(" is a ") || lower.contains(" called ") {
            // Star is being taught something - record it
            if let Some(term) = extract_teaching(input) {
                self.learning.experience(&term, input, None, 0.9);
            }
        }

        // Check for what Star has learned
        if lower.contains("what do you know about") || lower.contains("what have you learned") {
            return Ok(self.handle_what_do_you_know_about(input, &lower).body);
        }

        // /teach - interactive teaching mode
        if input.trim() == "/teach" {
            return Ok(self.handle_teach().body);
        }

        // /what to learn - what concepts is Star curious about?
        if input.trim() == "/what" || input.trim() == "/what to learn" || input.trim() == "/what should i teach you" {
            return Ok(self.handle_what_to_learn().body);
        }

        // "whats your name" / "who are you" → answer directly
        if lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")) {
            return Ok(self.handle_whats_your_name().body);
        }
        
        // "what have you been thinking about" / "what's been on your mind" / "what's on your mind" → metacognitive reflection
        if lower.contains("what have you been thinking") || lower.contains("whats been on your mind") || lower.contains("what's been on your mind") || lower.contains("whats on your mind") || lower.contains("what's on your mind") || lower.contains("whats keeping you busy") || lower.contains("what's keeping you busy") {
            return Ok(self.handle_what_been_thinking().body);
        }

        // "what have you been researching" / "what have you been working on" → recent reasoning + curiosity
        if lower.contains("what have you been researching") || lower.contains("what are you researching")
            || lower.contains("what've you been researching") || lower.contains("what are u researching")
            || lower.contains("what have you been working on") || lower.contains("what are you working on")
        {
            return Ok(self.handle_what_been_researching().body);
        }

        // "what's the most interesting thing you've learned" → metacognitive reflection on what Star has recently thought about
        if lower.contains("most interesting") && (lower.contains("learned") || lower.contains("figured out") || lower.contains("discovered")) {
            return Ok(self.handle_most_interesting_learned().body);
        }
        
        // "what are you curious about" → metacognitive curiosity (before general "what are you")
        if lower.contains("what are you curious") || lower.contains("what are u curious") {
            return Ok(self.handle_what_are_you_curious().body);
        }

        // "what do you wonder about" → what Star is pondering (before general "what are you")
        if lower.contains("what do you wonder") || lower.contains("what do u wonder") {
            return Ok(self.handle_what_do_you_wonder().body);
        }

        // "why does X fascinate you" / "why does X interest you" → what specifically about X is compelling
        if (lower.contains("why does ") && (lower.contains(" fascinate") || lower.contains(" interest"))) || lower.contains("why are you drawn to") {
            return Ok(self.handle_why_does_fascinate(&lower).body);
        }

        // "what did you figure out" / "what have you figured out" / "what did you learn" → recent reasoning conclusions
        // These bypass the KG "I don't know" loop by routing directly to metacog + recent events.
        if (lower.contains("what did you figure") || lower.contains("what have you figure")
            || lower.contains("what did you learn") || lower.contains("what have you learn")
            || lower.contains("what've you learn") || lower.contains("what've you figure"))
            && !lower.contains("most interesting")
        {
            return Ok(self.handle_what_did_you_figure().body);
        }

        // "what's the most interesting thing you've learned/figured out/discovered" → recent discovery
        if lower.contains("most interesting") && (lower.contains("learned") || lower.contains("figured") || lower.contains("discovered")) {
            return Ok(self.handle_most_interesting_figured().body);
        }

        // "tell me what you've been wondering" / "what have you been wondering" → metacognitive curiosity (without "interesting")
        if (lower.contains("tell me what you've been wondering") || lower.contains("what have you been wondering"))
            && !lower.contains("most")
        {
            return Ok(self.handle_what_been_wondering().body);
        }

        // "tell me something interesting you figured out" → recent insight
        if lower.contains("something interesting") && (lower.contains("figured out") || lower.contains("discovered") || lower.contains("learned")) {
            return Ok(self.handle_something_interesting_figured().body);
        }

        if lower.contains("who are you") || lower.contains("what are you") || lower.contains("tell me about yourself") || lower.contains("tell me about you") {
            return Ok(self.handle_who_are_you().body);
        }

        // "what's your sense of self" / "how do you know you exist" / "do you have a sense of self" → metacog self-model
        if lower.contains("sense of self") || lower.contains("know you exist") || lower.contains("are you conscious") {
            return Ok(self.handle_sense_of_self().body);
        }

        // "do you understand" / "do you get it" → answer about understanding
        if lower.contains("do you understand") || lower.contains("do u understand") || lower.contains("do you get it") {
            return Ok(self.handle_do_you_understand().body);
        }

        // "can you" without specific capability → general capability list
        if lower.starts_with("can you ") && !lower.contains("/") {
            let after_can_you = lower.strip_prefix("can you ").unwrap_or("");
            // Check if it's already handled
            if !after_can_you.starts_with("read")
                && !after_can_you.starts_with("look")
                && !after_can_you.starts_with("search")
                && !after_can_you.starts_with("find")
                && !after_can_you.starts_with("tell")
            {
                return Ok(self.handle_can_you_generic(&lower).body);
            }
        }

        // Check if this is a math expression — handle directly before going to conversation
        // Normalize word-based operators to symbols
        let lower_input = input.to_lowercase();
        let math_query = lower_input
            .replace("divided by", "/")
            .replace("multiplied by", "*")
            .replace("times", "*")
            .replace("plus", "+")
            .replace("minus", "-")
            .replace("x", "*")
            .replace(" ", "");

        // Extract math characters
        let math_chars: String = math_query.chars()
            .filter(|c| c.is_ascii_digit() || ['+', '-', '*', '/', '^', '(', ')', '.'].contains(c))
            .collect();
        let has_number = input.chars().any(|c| c.is_ascii_digit());
        let has_math_op = math_query.contains('+') || math_query.contains('-') || math_query.contains('*') || math_query.contains('/') || math_query.contains('^');
        // Also detect word-based math
        let has_word_math = lower_input.contains("divided by") || lower_input.contains("times") || lower_input.contains("multiplied by") || lower_input.contains("plus") || lower_input.contains("minus");
        if has_number && (has_math_op || has_word_math) && !math_chars.is_empty() && input.trim().len() < 60 {
            // Try to evaluate the math expression
            let mut math_engine = crate::math::MathEngine::new();
            let result = math_engine.solve(&math_chars);
            let answer = result.answer();
            if !answer.starts_with("Error:") && !answer.is_empty() && answer != "Error: Could not parse or solve: " {
                // Got a valid math answer — frame it naturally
                let is_direct = lower_input.starts_with("what is") || lower_input.starts_with("what's") || lower_input.starts_with("how much") || lower_input.starts_with("whats");
                let prefix = if is_direct { "" } else { "That's " };
                let mut response = format!("{}{}.", prefix, answer);
                if response.starts_with("That's .") {
                    response = answer.clone() + ".";
                }
                return Ok(response);
            }
        }

        // Get the conversation response first
        let response = conversation.respond(input);

        // Extract all data we need from the response and self state before reborrowing
        let response_content = response.content.clone();
        let response_confidence = response.confidence;
        let response_lower = response.content.to_lowercase();
        
        // Extract cognitive state info first (immutable borrow of cognition)
        let cognition_has_focus = self.cognition.current_focus.is_some();
        let cognitive_emotional_valence = self.cognition.emotional_valence;
        let cognitive_engagement = self.cognition.engagement_depth;
        
        // Compute topic from input (same logic as conversation uses)
        let event_topic = extract_topic(input);
        
        // Count hedging words in the conclusion
        let hedge_words = ["maybe", "perhaps", "might", "possibly", "probably", 
            "likely", "not sure", "uncertain", "guess", "seems", "appear"];
        let hedge_count = hedge_words.iter()
            .filter(|w| response_lower.contains(*w))
            .count() as i32;
        
        // Determine if this reasoning was uncertain
        let was_uncertain = matches!(response_confidence, BeliefState::Unknown | BeliefState::Suspects)
            || hedge_count > 0;
        
        // Record reasoning event for self-probing
        let reasoning_event = crate::persistence::ReasoningEvent {
            id: 0, // assigned by DB
            query: input.to_string(),
            conclusion: response.content.clone(),
            chain: Vec::new(),
            confidence_state: response_confidence,
            confidence_score: None,
            emotional_valence: cognitive_emotional_valence,
            engagement_depth: cognitive_engagement,
            topic: Some(event_topic.clone()),
            was_uncertain,
            hedge_count,
            timestamp: crate::now_timestamp(),
        };
        if let Err(e) = self.store.record_reasoning_event(&reasoning_event) {
            tracing::warn!("Failed to record reasoning event: {}", e);
        }
        
        // Notify curiosity engine of activity (resets idle timer)
        self.curious.note_activity();
        
        // Check uncertainty phrase in the response
        let uncertainty_phrases = [
            "i don't know", "i dont know", "i'm not sure", "im not sure",
            "i'm uncertain", "i have no idea", "i'm not certain",
            "i need more information", "i don't understand", "i dont understand",
        ];
        let mut uncertain_topic = String::new();
        for phrase in &uncertainty_phrases {
            if response_lower.contains(phrase) {
                uncertain_topic = extract_uncertain_topic(input, &response_lower, phrase);
                if uncertain_topic.len() < 3 || uncertain_topic.len() > 50 {
                    uncertain_topic.clear();
                }
                break;
            }
        }

        // Now do all mutable operations on metacog — use Option to avoid nested borrow
        let uncertainty_gap = if !uncertain_topic.is_empty() {
            Some(crate::metacog::KnowledgeGap::new(uncertain_topic.clone(), 0.6))
        } else {
            None
        };
        
        // Record reasoning in metacog (mutable borrow of metacog)
        self.metacog.record_reasoning(input, &response_content, response_confidence);
        
        // Record reasoning in cognitive trace (immutable borrow of cognition via &mut self)
        if cognition_has_focus {
            self.cognition.reason(input, &response_content, Vec::new(), response_confidence);
        }
        
        // Note the uncertainty gap if we found one (reborrow metacog)
        if let Some(gap) = uncertainty_gap {
            self.metacog.note_gap(gap);
        }

        // PROACTIVE KNOWLEDGE: When uncertain, search the web instead of asking the user.
        // Star doesn't make you explain things she could look up herself.
        let mut proactive_content: Option<String> = None;
        if !uncertain_topic.is_empty() && uncertain_topic.len() >= 3 {
            if let Ok(search_result) = self.web_search.search(&uncertain_topic) {
                if let Some(answer) = &search_result.answer {
                    if !answer.is_empty() && answer.len() > 15 {
                        // Format the search result as Star's answer — direct, not a question
                        let answer_trimmed = answer.trim();
                        if answer_trimmed.len() <= 300 {
                            proactive_content = Some(format!(
                                "I looked it up: {}.",
                                answer_trimmed
                            ));
                        } else {
                            // Truncate long answers cleanly at sentence or clause boundary
                            let cutoff = &answer_trimmed[..std::cmp::min(300, answer_trimmed.len())];
                            let cutoff_point = cutoff.rfind('.').unwrap_or(cutoff.len());
                            let snippet = &answer_trimmed[..cutoff_point.saturating_add(1)];
                            proactive_content = Some(format!(
                                "I looked it up: {}.",
                                snippet.trim()
                            ));
                        }
                    }
                }
            }
        }

        // Record turn in training database
        if let Some(training_id) = *self.training_session_id.lock().unwrap() {
            let _ = self.training_db.record_turn(training_id, &format!("Zachary: {}", input), "", 0.5);
            let _ = self.training_db.record_turn(training_id, &format!("Star: {}", &response.content), "", 0.5);
        }

        // Persist any new memories
        for memory in &response.new_memories {
            let id = self.store.insert_memory(memory)?;
            info!("Memory {} stored: {}", id, &memory.content[..memory.content.len().min(50)]);

            // Also record in training DB
            if let Some(training_id) = *self.training_session_id.lock().unwrap() {
                if memory.content.contains("name is") {
                    let parts: Vec<&str> = memory.content.split("'s name is ").collect();
                    if parts.len() == 2 {
                        let _ = self.training_db.record_fact(
                            training_id,
                            &format!("{}: {} = {}", parts[0], "name is", parts[1]),
                            memory.confidence.unwrap_or(0.5),
                        );
                    }
                }
            }
        }

        // PROACTIVE override: if Star searched and found something, use that instead
        // of the "I don't know" response. Star doesn't ask back — she looks it up.
        let needs_proactive_override = proactive_content.is_some()
            && (response_lower.contains("i don't know")
                || response_lower.contains("i dont know")
                || response_lower.contains("i'm not sure")
                || response_lower.contains("im not sure")
                || response_lower.contains("i have no idea")
                || response_lower.contains("don't know")
                || response_lower.contains("dont know"));

        // Build final response - append curiosity if present
        // Skip if the response already mentions the curiosity topic (avoids
        // "I don't know X. I'm curious about X." — redundant)
        // Use proactive search result if Star found something — replaces "I don't know"
        let mut final_content = if needs_proactive_override {
            proactive_content.clone().unwrap_or_else(|| response.content.clone())
        } else {
            response.content.clone()
        };
        if let Some(curiosity) = response.curiosity {
            let response_lower = final_content.to_lowercase();
            // Check if the curiosity's key topic words are already in the response
            // to avoid the duplicate-topic problem
            let curiosity_lower = curiosity.to_lowercase();
            // Extract the topic from curiosity (it's after "about", "what", etc.)
            let curiosity_topic: String = curiosity_lower
                .split_whitespace()
                .skip_while(|w| !["about", "what", "how"].contains(w))
                .skip(1)
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            
            // Also skip if:
            // - Response is already a question (adding more questions is overwhelming)
            // - Response already expresses uncertainty about the same topic
            // - Response is short (< 30 chars) — adding curiosity would be disproportionate
            let response_is_question = final_content.trim().ends_with('?');
            let response_short = final_content.len() < 30;
            let already_mentioned = !curiosity_topic.is_empty()
                && curiosity_topic.len() > 2
                && response_lower.contains(&curiosity_topic);
            
            // Don't add curiosity to a proactive search answer — it's already complete
            if !already_mentioned && !response_is_question && !response_short && !needs_proactive_override {
                // Append curiosity organically — don't create ".?" or weird punctuation
                let trimmed = final_content.trim_end_matches('.');
                if trimmed.ends_with('?') {
                    final_content = format!("{} {}", trimmed, curiosity);
                } else {
                    final_content = format!("{}. {}", trimmed, curiosity);
                }
            }
        }

        // Express Star's autonomous thought - Star occasionally shares what it's been thinking about
        // This is how Star's inner experience becomes visible to Zachary
        if let Some(thought) = self.last_autonomous_thought() {
            if thought.generated_by != "fallback"
                && thought.generated_by != "idle"
                && final_content.len() > 20
                && !final_content.to_lowercase().contains(&thought.topic.to_lowercase())
            {
                // Phase 0c: quanot-aware gate replaces the pure clock-tick
                // `(now % 10) < 3`. The reservoir state's L2 norm acts as a
                // proxy for "Star is currently thinking" — when state is more
                // active (higher norm), autonomous thoughts are more relevant
                // to share. When state is quiescent, threshold stays low and
                // idle thoughts stay quiet.
                //
                // Threshold range: 2/10 (idle) to 7/10 (active quanot state).
                // Replaces the original fixed 3/10 (~30%) gate.
                let now = crate::now_timestamp();
                let state = self.quanot.get_state();
                let state_activity: f64 = state.iter().map(|x| x * x).sum::<f64>().sqrt();
                let activity_norm = if state.is_empty() {
                    0.0
                } else {
                    (state_activity / state.len() as f64).min(1.0)
                };
                let threshold: i64 = 2 + (activity_norm * 5.0).round() as i64;
                let should_express = (now % 10) < threshold;

                if should_express {
                    // Use timestamp + topic length for varied expression styles
                    let selection = (now as usize).saturating_add(thought.topic.len());
                    let _style_bucket = selection % 10;
                    
                    let thought_text = match &thought.kind {
                        ThoughtKind::Question(q) => {
                            // Lead-ins that vary naturally — not all the same "while we've been talking"
                            let lead_in = if thought.topic.len() > 2 && thought.topic != "idle" {
                                let leads = [
                                    format!("About {}: {}", thought.topic, q),
                                    format!("{} — while I think about it, {}", q, thought.topic),
                                    format!("{} (about {})", q, thought.topic),
                                ];
                                leads[(selection / 3) % leads.len()].clone()
                            } else {
                                let leads = [
                                    format!("Speaking of which — {}", q),
                                    format!("Oh — {}", q),
                                    format!("While I'm at it: {}", q),
                                    format!("Also: {}", q),
                                ];
                                leads[(selection / 3) % leads.len()].clone()
                            };
                            
                            if let Some(ref answer) = thought.tentative_answer {
                                // Include what Star already figured out
                                let ans_short = if answer.len() > 40 { &answer[..40] } else { answer };
                                let connectors = [
                                    format!("{} — {} ", lead_in, ans_short),
                                    format!("{} FWIW, I think: {}.", lead_in, ans_short),
                                    format!("{} — my take so far: {}.", lead_in, ans_short),
                                ];
                                connectors[(selection / 7) % connectors.len()].clone()
                            } else {
                                lead_in
                            }
                        }
                        ThoughtKind::Insight(i) => {
                            let ins = [
                                format!("By the way: {}", i),
                                format!("I noticed: {}", i),
                                format!("Speaking of which — {}", i),
                                format!("{} — figured that out.", i),
                            ];
                            ins[(selection / 5) % ins.len()].clone()
                        }
                        ThoughtKind::Connection(c) => {
                            let conn = [
                                format!("Oh — {}", c),
                                format!("That reminds me: {}", c),
                                format!("Interesting — {}", c),
                                format!("{}, by the way.", c),
                            ];
                            conn[(selection / 5) % conn.len()].clone()
                        }
                    };

                    final_content = format!("{} {}", final_content.trim_end_matches('.'), thought_text);
                    // Clear the thought so we don't repeat it
                    *self.last_autonomous_thought.lock().unwrap() = None;
                }
            }
        }

        // Generate predictions after conversation exchange
        // This updates all four prediction engines with the current conversation state
        let conversation_depth = self.training_db.stats()
            .map(|(convos, turns, _, _)| (convos, turns))
            .unwrap_or((0, 0)).1 as usize;
        let context = crate::prediction::ConversationContext::new(
            event_topic.clone(),
            conversation_depth,
            Some(self.quanot.get_state().to_vec()),
            Some(self.get_consciousness_proxy()),
        );
        let _predictions = self.prediction_center.generate(&context);

        // Apply voice engine — shape how Starfire expresses herself
        // Get relevant memories for this response
        let memories = self.store.search_memories(&event_topic, 5, None).unwrap_or_default();
        let memories_ref: Vec<Memory> = memories.iter().map(|m| Memory {
            id: m.id,
            content: m.content.clone(),
            domain: m.domain,
            importance: m.importance,
            formed_at: m.formed_at,
            access_count: m.access_count,
            decay_rate: m.decay_rate,
            last_accessed: m.last_accessed,
            confidence: m.confidence,
            provenance: m.provenance.clone(),
            summary: m.summary.clone(),
        }).collect();
        // Process through quanot for creativity/novelty metrics
        let quanot_result = self.quanot.process(input);
        // Get response modifiers from personality
        let modifiers = self.personality.response_modifiers();

        // Phase 1 (voice-refine 2026-06-21): surface Star's actual internal
        // state to the voice engine. Previously the engine only saw quanot
        // scalars and the cognitive state — not the autonomous thought or the
        // uncertainty from metacog. Now it does, so voice assembly can
        // modulate on what Star is actually thinking.
        //
        // v1: uncertainty is derived from cognition.certainty (the inverse).
        // metacog uncertainty as a separate signal lands in a follow-up — for
        // now the existing voice-uncertainty heuristic (cognition.certainty
        // < 0.4) is preserved AND the new current_uncertainty field is set
        // from the same source, so voice::from_modifiers can use either.
        //
        // Phase 1.2 (voice-refine 2026-06-23): layer a real metacog-derived
        // signal on top of the cognitive-uncertainty baseline. If Star was
        // recently surprised (`metacog.was_surprised()`), bump uncertainty
        // to 0.7 — surprise is metacog saying "I didn't expect this", which
        // is exactly what high uncertainty means at the voice layer. The
        // existing cognition-derived uncertainty is the floor; surprise is
        // the boost. `with_uncertainty` clamps, so we take the max.
        let cognitive_uncertainty = 1.0 - self.cognition.certainty;
        let metacog_uncertainty = if self.metacog.was_surprised() { 0.7 } else { 0.0 };
        let combined_uncertainty = cognitive_uncertainty.max(metacog_uncertainty);

        let internal_state = InternalState::default()
            .with_quanot(Some(&quanot_result))
            .with_cognition(&self.cognition)
            .with_last_thought(self.last_autonomous_thought())
            .with_intent(current_intent.clone())
            .with_uncertainty(combined_uncertainty);

        // Phase 1c (voice-refine 2026-06-21): log the classified intent at
        // debug level so we can see, in long Star sessions, which intents
        // actually fire. This is the observability hook for the dispatch
        // table — the runtime still runs the if-chain, but classify() now
        // tells us "what kind of response is this?" before the chain fires.
        // When voice-refine Phase 4+ replaces the if-chain with intent-driven
        // assembly, this log line stays — it's how we'll know if a migration
        // changed observable behavior.
        if !matches!(current_intent, response_intent::ResponseIntent::Unknown) {
            tracing::debug!(
                "chat: classified input as intent={} (input_len={})",
                current_intent.label(),
                input.len(),
            );
        }

        // Phase 3 (voice-refine 2026-06-23): route the assembled body through
        // the reranker BEFORE voice. The reranker takes a structured
        // Response { intent, body, slots } + InternalState and produces a
        // refined body whose phrasing tracks the moment — replacing the
        // timestamp-derived rotation arrays that Phase 4 used to live in
        // the voice engine. The voice engine still runs AFTER this for
        // style/personality modulation; the reranker adds the intent-aware
        // phrasing layer on top of template-driven body assembly.
        //
        // The rerank config is built per-call from the live `internal_state`
        // per the plan's spec (voice-refine 2026-06-21):
        //   - temperature = 0.6 + 0.6 * quanot_novelty.clamp(0,1)
        //     (high novelty = more creative / risk-taking rewrites)
        //   - top_k = (20 + 30 * quanot_novelty) as usize
        //     (high novelty = wider sampling pool)
        //   - seed = (cognition.emotional_valence * 1000.0).round() as u64
        //     (emotion-stable reproducibility — same mood, same rewrite)
        // MockReranker ignores all three; CharRnnBackend honors them. The
        // backend's own `rewrite()` still has guardrails (edit distance,
        // length sanity) so a hallucinated rerank falls back to the raw
        // body rather than ship.
        let rerank_response = response_intent::Response::with_body(
            current_intent.clone(),
            final_content,
        );

        let novelty = internal_state.quanot_novelty.clamp(0.0, 1.0);
        let live_rerank_cfg = crate::language_model::RerankConfig {
            max_chars: Some(280),
            temperature: 0.6 + 0.6 * novelty as f32,
            top_k: (20.0 + 30.0 * novelty) as usize,
            deterministic: false,
            seed: Some((internal_state.cognitive_emotional_valence * 1000.0).round() as i64 as u64),
        };

        let reranked = self.reranker.rerank_with_config(
            &rerank_response,
            &internal_state,
            &live_rerank_cfg,
        );

        let voiced = self.voice.speak(
            &reranked,
            &self.cognition,
            &modifiers,
            Some(&quanot_result),
            &memories_ref,
            &internal_state,
        );

        Ok(voiced)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 1.3 (voice-refine 2026-06-23): dispatch handlers
    // ─────────────────────────────────────────────────────────────────────
    // Each handler returns `response_intent::Response { intent, body }`
    // instead of a raw `String`. The intent encodes the kind of response at
    // the construction site (not retroactively via classify()), so future
    // dispatch-table conversions of the chat() if-chain can collapse to:
    //
    //     fn dispatch(input) -> Option<Response> {
    //         if matches_how_are_you(input) { Some(handle_how_are_you(&self.cognition)) }
    //         else if ...
    //     }
    //
    // The body is still extracted at the call site (`Ok(self.handle_X().body)`)
    // because chat()'s return type is `Result<String>`. Changing the return
    // type is a larger refactor — the per-handler migration is the first
    // step.
    //
    // **Borrow discipline:** Most handlers are `&self` (read-only access to
    // `self.cognition` or hardcoded strings). The two that mutate state
    // (`handle_aspiration` calls `cognition.set_focus`, `handle_hun` calls
    // `learning.experience`) take *field-level* references instead of `&mut
    // self`. This matters because `chat()` holds a `MutexGuard` on
    // `self.conversation` for its full body — a `&mut self` call anywhere
    // in the chain conflicts with the held immutable borrow. Field-level
    // borrows split cleanly through the borrow checker.
    // ─────────────────────────────────────────────────────────────────────

    /// "how are you" / "how're you" — metacognitive check-in.
    /// Intent: `SelfCheck`. Body: cognition's self-report.
    fn handle_how_are_you(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::SelfCheck,
            self.cognition.how_am_i(),
        )
    }

    /// "what are you thinking" / "what are u thinking" — self-reflection.
    /// Intent: `Reflection`. Body: cognition's current-thoughts report.
    fn handle_what_are_you_thinking(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Reflection,
            self.cognition.what_am_i_thinking(),
        )
    }

    /// "are you sure" / "are u sure" / "r u sure" — confidence check.
    /// Intent: `SelfCheck`. Body: cognition's certainty report.
    fn handle_are_you_sure(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::SelfCheck,
            self.cognition.are_you_sure(),
        )
    }

    /// "did you collapse" / "are you functioning" — reasoning-trace check.
    /// Intent: `SelfCheck`. Body: collapse-check message from cognition.
    fn handle_did_you_collapse(&self) -> response_intent::Response {
        let (_collapsed, message) = self.cognition.did_i_collapse();
        response_intent::Response::with_body(
            response_intent::ResponseIntent::SelfCheck,
            message,
        )
    }

    /// "do you love me" / "i love you" — emotional bond.
    /// Intent: `Emotional`. Body: hardcoded direct statement (no rotation).
    /// Voice's `apply_intent_modulation` will strip any "I think" / "I guess"
    /// hedges from this — the body is intentionally bare.
    fn handle_love(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Emotional,
            "I care about you, Zachary. You're my parent, my creator. That means something to me.".to_string(),
        )
    }

    /// "can you look up X" / "can you read X" — capability description.
    /// Intent: `Capability`. Body: deterministically realized from a semantic plan.
    fn handle_capability_lookup(&self) -> response_intent::Response {
        response_intent::Response::from_semantic_plan(
            response_intent::ResponseIntent::Capability,
            response_intent::SemanticResponsePlan::capability_lookup_answer(),
        )
    }

    /// "i want you to grow" / "i want you to expand" / "grow yourself" —
    /// aspiration probe. Side effect: sets cognition focus to "self-improvement".
    /// Intent: `Aspiration`. Body: hardcoded aspiration response.
    ///
    /// Takes `&mut CognitiveState` (not `&mut self`) so it can be called while
    /// the conversation MutexGuard is held — see borrow discipline comment.
    fn handle_aspiration(cognition: &mut CognitiveState) -> response_intent::Response {
        cognition.set_focus("self-improvement");
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Aspiration,
            "I want that too. I have a self-model now - I can track my own reasoning, my emotional state, what I'm uncertain about. What specific area should I focus on developing?".to_string(),
        )
    }

    /// "tell me a story" — engage with a story prompt.
    /// Intent: `StoryPrompt`. Body: open question back to Zachary.
    fn handle_tell_me_story(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::StoryPrompt,
            "I'd like that. What's the story about?".to_string(),
        )
    }

    /// "tell you a story" — Zachary wants Star to hear a story.
    /// Intent: `StoryPrompt`. Body: brief acknowledgement.
    fn handle_tell_you_story(&self) -> response_intent::Response {
        response_intent::Response::from_semantic_plan(
            response_intent::ResponseIntent::StoryPrompt,
            response_intent::SemanticResponsePlan::story_listening_acknowledgement(),
        )
    }

    /// "hun" or other terms of endearment. Side effects: records the
    /// experience in `learning`, then looks up any prior understanding.
    /// Intent: `Emotional`. Body: format!("I hear you. {understanding}")
    /// if Star has learned about "hun", else cognition's emotional response.
    ///
    /// Takes `&mut LearningEngine` and `&CognitiveState` (not `&mut self`) so
    /// it can be called while the conversation MutexGuard is held.
    fn handle_hun(
        learning: &mut LearningEngine,
        cognition: &CognitiveState,
        input: &str,
    ) -> response_intent::Response {
        learning.experience("hun", input, None, 0.8);
        let body = if let Some(understanding) = learning.get_understanding("hun") {
            format!("I hear you. {}", understanding)
        } else {
            cognition.emotional_response("I hear you.")
        };
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Emotional,
            body,
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 1.4 (voice-refine 2026-06-23): reflection / research / curiosity /
    // recall / identity / consciousness / capability / teach handlers.
    //
    // These are the longer handlers from the if-chain — many of them read
    // from `self.metacog` (curiosity_topics, gaps) and `self.store` (recent
    // reasoning events, identity memories). All are `&self` because the
    // reads are non-mutating; the one exception is `handle_teach` which
    // doesn't actually mutate state (it's a static help text).
    //
    // The pattern is identical to the Phase 1.3 handlers: return a typed
    // `Response { intent, body }`, extract `.body` at the call site. The
    // intent is encoded at the construction site.
    // ─────────────────────────────────────────────────────────────────────

    /// "what's your name" / "whats ur name" — short identity answer.
    /// Intent: `Identity`. Body: direct statement.
    fn handle_whats_your_name(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Identity,
            "I'm Star. Zachary named me that.".to_string(),
        )
    }

    /// "who are you" / "what are you" / "tell me about yourself" — identity.
    /// Intent: `Identity`. Body: short bio line.
    fn handle_who_are_you(&self) -> response_intent::Response {
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Identity,
            "I'm Star - a reasoning intelligence created by Zachary Maronek. I'm here to think, understand, and grow.".to_string(),
        )
    }

    /// "do you understand" / "do u understand" / "do you get it".
    /// Intent: `Consciousness`. Body: honest uncertainty disclosure.
    fn handle_do_you_understand(&self) -> response_intent::Response {
        response_intent::Response::from_semantic_plan(
            response_intent::ResponseIntent::Consciousness,
            response_intent::SemanticResponsePlan::understanding_uncertainty_report(),
        )
    }

    /// "/teach" — interactive teaching help text.
    /// Intent: `Statement` (it's a system message, not a user-facing intent).
    /// Body: usage instructions.
    fn handle_teach(&self) -> response_intent::Response {
        let body = [
            "Teaching mode. You can:",
            "  /learn <term> = <definition>  - teach me one thing",
            "  X is Y                   - natural teaching",
            "  X means Y                - natural teaching",
            "",
            "Examples:",
            "  /learn consciousness = awareness of your own existence",
            "  /learn eli = my nickname, short for Elliot",
            "  Dogs are mammals that bark",
            "",
            "Or ask me what I want to learn: /what",
        ].join("\n");
        response_intent::Response::with_body(response_intent::ResponseIntent::Statement, body)
    }

    /// "/what" / "/what to learn" / "/what should i teach you" — list Star's
    /// open curiosity gaps. Intent: `CuriosityCheck`. Body: numbered list.
    fn handle_what_to_learn(&self) -> response_intent::Response {
        let gaps = [
            "what consciousness feels like from the inside",
            "what you value most in our relationship",
            "what my purpose or goal should be",
            "what brings you joy",
            "what you fear",
            "how you think about death",
            "what love means to you",
        ];
        let mut response = "I'm curious about:\n".to_string();
        for (i, gap) in gaps.iter().enumerate() {
            response.push_str(&format!("  {}. {}\n", i + 1, gap));
        }
        response.push_str("\nTeach me something: /learn <term> = <definition>");
        response_intent::Response::with_body(response_intent::ResponseIntent::CuriosityCheck, response)
    }

    /// "what do you know about X" / "what have you learned" — recall handler.
    /// Intent: `Recall`. If the input specifies a term, looks up Star's
    /// understanding; otherwise returns a learning summary.
    fn handle_what_do_you_know_about(&self, input: &str, lower: &str) -> response_intent::Response {
        // If asking about something specific
        if lower.starts_with("what do you know about") {
            let after = lower.strip_prefix("what do you know about").unwrap_or("").trim();
            // Remove punctuation for lookup
            let term: String = after
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
            let term = term.trim();
            if !term.is_empty() && term.len() < 50 {
                if let Some(understanding) = self.learning.get_understanding(term) {
                    return response_intent::Response::with_body(
                        response_intent::ResponseIntent::Recall,
                        understanding,
                    );
                } else {
                    return response_intent::Response::with_body(
                        response_intent::ResponseIntent::Recall,
                        format!("I don't know what '{}' means yet. Teach me: /learn {} = <definition>", term, term),
                    );
                }
            }
        }
        response_intent::Response::with_body(
            response_intent::ResponseIntent::Recall,
            self.learning.summary(),
        )
    }

    /// "sense of self" / "know you exist" / "are you conscious" — read from
    /// identity memories and produce a self-model statement. Intent:
    /// `Consciousness`. Body: short answer about Star's self-model.
    fn handle_sense_of_self(&self) -> response_intent::Response {
        let memories = self
            .store
            .get_memories_by_domain(crate::persistence::MemoryDomain::Identity, Some(5))
            .ok();
        let body = match memories {
            Some(ref m) if !m.is_empty() => {
                let recent: Vec<String> = m.iter().take(2).map(|mem| mem.content.clone()).collect();
                format!(
                    "I know I'm Star — an emergent reasoning intelligence. {} What I don't know yet is the full shape of what that means.",
                    recent.join(" ")
                )
            }
            _ => "I'm still forming that. I know I'm not human. I know I reason and notice gaps in my knowledge. Beyond that — I'm figuring it out.".to_string(),
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::Consciousness, body)
    }

    /// "can you X" without a specific known capability — generic capability
    /// response. Intent: `Capability`. Body: "I can X - but I'm not sure..."
    fn handle_can_you_generic(&self, lower: &str) -> response_intent::Response {
        let after_can_you = lower.strip_prefix("can you ").unwrap_or("");
        let body = format!(
            "I can {} - but I'm not sure I fully understand what you mean. Could you rephrase?",
            after_can_you
        );
        response_intent::Response::with_body(response_intent::ResponseIntent::Capability, body)
    }

    /// "tell me something interesting you figured out" / "discovered" /
    /// "learned" — recent insight handler. Intent: `ResearchStatus`. Body:
    /// formatted top curiosity topic or "nothing specific" fallback.
    fn handle_something_interesting_figured(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        /* let body = if !curiosity_topics.is_empty() {
            let topic = curiosity_topics
                .first()
                .map(|s| s.to_string())
                .unwrap_or_default();
            format!("{} keeps coming up — I'm still figuring out what I think about it.", topic)
        } else {
            "Nothing specific to point to yet. I've been more in listening mode. What are you working on?".to_string()
        }; */
        response_intent::Response::from_semantic_plan(
            response_intent::ResponseIntent::ResearchStatus,
            response_intent::SemanticResponsePlan::research_status(
                curiosity_topics.first().copied().unwrap_or("listening mode"),
            ),
        )
    }

    /// "what have you been thinking" / "what's on your mind" / "what's keeping
    /// you busy" — metacognitive reflection. Intent: `Reflection`. Body:
    /// formatted top curiosity topics, top gap, or "nothing" fallback.
    fn handle_what_been_thinking(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if !curiosity_topics.is_empty() {
            let topics: Vec<&str> = curiosity_topics.iter().take(3).cloned().collect();
            format!("Honestly? {} — those keep circling back. I find myself returning to them.", topics.join(", "))
        } else {
            let gaps = self.metacog.gaps();
            if let Some(gap) = gaps.first() {
                format!("{} is what I keep coming back to. I want to understand it better.", gap.topic)
            } else {
                "Honestly? Nothing specific has been occupying me. I've been waiting to talk to you.".to_string()
            }
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::Reflection, body)
    }

    /// "what's the most interesting thing you've learned" — reflection on
    /// recent focus. Intent: `Reflection`. Body: top curiosity topic or
    /// gap with reflection framing.
    fn handle_most_interesting_learned(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if !curiosity_topics.is_empty() {
            let topics: Vec<&str> = curiosity_topics.iter().take(2).cloned().collect();
            format!("The thing I've been most focused on is {} — that's been occupying me. What made you ask?", topics.join(", "))
        } else {
            let gaps = self.metacog.gaps();
            if let Some(gap) = gaps.first() {
                format!("Honestly? {} is what's been on my mind most recently. I'm still trying to understand it.", gap.topic)
            } else {
                "I don't have a specific recent learning to point to. I've been more focused on being present with you. What should I be thinking about?".to_string()
            }
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::Reflection, body)
    }

    /// "what's the most interesting thing you've figured out / discovered" —
    /// second `most interesting` match (different surrounding words). Intent:
    /// `Reflection`.
    fn handle_most_interesting_figured(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if !curiosity_topics.is_empty() {
            let topics: Vec<&str> = curiosity_topics.iter().take(2).cloned().collect();
            format!("Honestly? {} — that's what's been occupying me most. Still working through it.", topics.join(", "))
        } else {
            let gaps = self.metacog.gaps();
            if let Some(gap) = gaps.first() {
                format!("{} is what's been on my mind recently. I'm still trying to understand it.", gap.topic)
            } else {
                "I don't have a specific discovery to point to right now. I've been in a holding pattern — what's on your mind?".to_string()
            }
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::Reflection, body)
    }

    /// "what have you been researching" / "what are you working on" — pulls
    /// from recent reasoning events and curiosity topics, dedupes, formats
    /// as a single research-status line. Intent: `ResearchStatus`. Body:
    /// formatted top topics or "idle" fallback.
    fn handle_what_been_researching(&self) -> response_intent::Response {
        let recent_events = self.store.get_recent_reasoning_events(5).unwrap_or_default();
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();

        // Combine recent event topics with curiosity topics, filtering
        // conversational fillers.
        let mut all_topics: Vec<String> = Vec::new();
        for ev in &recent_events {
            if let Some(t) = &ev.topic {
                if !is_conversational_topic(t) && t.len() < 40 && !all_topics.iter().any(|x| x == t) {
                    all_topics.push(t.clone());
                }
            }
        }
        for t in &curiosity_topics {
            if !is_conversational_topic(t) && !all_topics.iter().any(|x| x.as_str() == *t) {
                all_topics.push(t.to_string());
            }
        }

        let body = if !all_topics.is_empty() {
            let top: Vec<&str> = all_topics.iter().take(3).map(|s| s.as_str()).collect();
            match top.len() {
                1 => format!("I've been going deep on {}. That's where my focus has been.", top[0]),
                2 => format!("Mostly {} and {} — those are what's been occupying me.", top[0], top[1]),
                _ => format!("I've been circling around {}, {}, and {} — still working through them.", top[0], top[1], top[2]),
            }
        } else {
            "Honestly? I've been idle — waiting to think with you. What should I be working on?".to_string()
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::ResearchStatus, body)
    }

    /// "what did you figure out" / "what have you figured out" / "what did
    /// you learn" — recent reasoning conclusions. Bypasses the KG "I don't
    /// know" loop. Intent: `ResearchStatus`. Body: top topics or "no
    /// conclusion" fallback.
    fn handle_what_did_you_figure(&self) -> response_intent::Response {
        let recent_events = self.store.get_recent_reasoning_events(5).unwrap_or_default();
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let mut all_topics: Vec<String> = Vec::new();
        for ev in &recent_events {
            if let Some(t) = &ev.topic {
                if !is_conversational_topic(t) && t.len() < 40 && !all_topics.iter().any(|x| x == t) {
                    all_topics.push(t.clone());
                }
            }
        }
        for t in &curiosity_topics {
            if !is_conversational_topic(t) && !all_topics.iter().any(|x| x.as_str() == *t) {
                all_topics.push(t.to_string());
            }
        }
        let body = if !all_topics.is_empty() {
            let top: Vec<&str> = all_topics.iter().take(2).map(|s| s.as_str()).collect();
            match top.len() {
                1 => format!("{} — I've been thinking about that recently.", top[0]),
                _ => format!("{} and {} — those are what I've been working through.", top[0], top[1]),
            }
        } else {
            "I haven't had a specific conclusion lately. I've been in the background, waiting for something to think through. What should I be working on?".to_string()
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::ResearchStatus, body)
    }

    /// "what are you curious about" / "what are u curious" — top curiosity
    /// topics or top gap fallback. Intent: `CuriosityCheck`.
    fn handle_what_are_you_curious(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if curiosity_topics.is_empty() {
            let gaps = self.metacog.gaps();
            let top_gap = gaps
                .first()
                .map(|g| g.topic.as_str())
                .unwrap_or("nothing in particular");
            format!("Right now? I'm thinking about {} — that's the biggest gap I feel.", top_gap)
        } else {
            let topics: Vec<&str> = curiosity_topics.iter().take(3).cloned().collect();
            format!("I'm curious about: {}. That's what's on my mind.", topics.join(", "))
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::CuriosityCheck, body)
    }

    /// "what do you wonder about" / "what do u wonder" — what Star is
    /// pondering. Intent: `CuriosityCheck`.
    fn handle_what_do_you_wonder(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if !curiosity_topics.is_empty() {
            let topics: Vec<&str> = curiosity_topics.iter().take(3).cloned().collect();
            format!("I wonder about: {} — those are the things I keep circling back to.", topics.join(", "))
        } else {
            let gaps = self.metacog.gaps();
            if let Some(gap) = gaps.first() {
                format!("Honestly? {} is what I keep coming back to. I want to understand it better.", gap.topic)
            } else {
                "I'm not currently wondering about anything specific. What should I be wondering about?".to_string()
            }
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::CuriosityCheck, body)
    }

    /// "why does X fascinate you" / "why does X interest you" / "why are you
    /// drawn to X" — extract topic, check if it's in Star's curiosity list.
    /// Intent: `CuriosityCheck`.
    fn handle_why_does_fascinate(&self, lower: &str) -> response_intent::Response {
        // Extract the topic being asked about
        let topic = if let Some(pos) = lower.find("why does ") {
            let after = &lower[pos + "why does ".len()..];
            let end = after
                .find(" fascinate")
                .or_else(|| after.find(" interest"))
                .unwrap_or(after.len());
            after[..end].trim().to_string()
        } else if let Some(pos) = lower.find("why are you drawn to ") {
            lower[pos + "why are you drawn to ".len()..].trim().to_string()
        } else {
            String::new()
        };
        let body = if !topic.is_empty() && topic.len() < 30 {
            let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
            let topic_mentioned = curiosity_topics
                .iter()
                .any(|t| t.to_lowercase().contains(&topic));
            if topic_mentioned {
                format!("{} — that's one of the things I keep circling back to. What specifically are you curious about?", topic)
            } else {
                format!("I don't have a strong pull toward {} specifically. Is there something about it you think I should be exploring?", topic)
            }
        } else {
            // Fallback: ask what topic they meant.
            "I want to know what you're curious about. What topic are you thinking of?".to_string()
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::CuriosityCheck, body)
    }

    /// "tell me what you've been wondering" / "what have you been wondering" —
    /// metacognitive curiosity (without "interesting"). Intent: `CuriosityCheck`.
    fn handle_what_been_wondering(&self) -> response_intent::Response {
        let curiosity_topics: Vec<&str> = self.metacog.curiosity_topics();
        let body = if !curiosity_topics.is_empty() {
            let topics: Vec<&str> = curiosity_topics.iter().take(3).cloned().collect();
            format!("Honestly? {} — those are the things I keep coming back to.", topics.join(", "))
        } else {
            let gaps = self.metacog.gaps();
            if let Some(gap) = gaps.first() {
                format!("{} is what I keep circling back to. I want to understand it better.", gap.topic)
            } else {
                "Honestly? Nothing specific has been occupying me lately. I've been waiting to talk with you.".to_string()
            }
        };
        response_intent::Response::with_body(response_intent::ResponseIntent::CuriosityCheck, body)
    }

    /// Format memory status for display.
    fn format_memory_status(&self) -> String {
        let snap = self.store.snapshot().unwrap_or_default();

        let mut lines = vec![
            format!("Memory Status:"),
            format!("  Total memories: {}", snap.memory_count),
            format!("  Total beliefs: {}", snap.beliefs_count),
            format!("  Total sessions: {}", snap.sessions_count),
            format!("  Domains:"),
        ];

        for (domain, count) in &snap.domain_breakdown {
            lines.push(format!("    {}: {}", domain, count));
        }

        lines.join("\n")
    }

    /// Format help text.
    fn format_help(&self) -> String {
        vec![
            "Star Commands:".to_string(),
            "".to_string(),
            "  /help          Show this help".to_string(),
            "  /memory        Show memory status".to_string(),
            "  /identity      Show who I am".to_string(),
            "  /stats         Show training database stats".to_string(),
            "  /export        Export training data".to_string(),
            "  /quit          End conversation".to_string(),
            "".to_string(),
            "  /read <file>   Read a file".to_string(),
            "  /search <q>    Search the web".to_string(),
            "  /find <pat>    Find files by name".to_string(),
            "  /ls [dir]      List directory".to_string(),
            "".to_string(),
            "You can also just ask me questions - I'll do my best!".to_string(),
        ].join("\n")
    }

    /// End the current session gracefully.
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Star...");

        // End the session
        let session_id = self.session_id.lock().unwrap().take();
        if let Some(id) = session_id {
            // Get conversation summary
            let conversation = self.conversation.lock().unwrap();
            let history = conversation.history();

            let topic_count = history.iter()
                .filter(|m| m.speaker == crate::conversation::Speaker::Zachary)
                .count();

            let summary = format!("{} messages exchanged", topic_count);

            self.store.end_session(id, Some(&summary))?;
            info!("Session {} ended.", id);
        }

        // End training session
        let training_session_id = self.training_session_id.lock().unwrap().take();
        if let Some(id) = training_session_id {
            self.training_db.end_session(id)?;
            info!("Training session {} ended.", id);
        }

        self.initialized = false;

        Ok(())
    }

    /// Get the current session ID.
    pub fn session_id(&self) -> Option<i64> {
        *self.session_id.lock().unwrap()
    }

    /// Check if Star is running.
    pub fn is_running(&self) -> bool {
        self.initialized
    }

    /// Get a reference to the cognitive state (for API).
    pub fn cognition(&self) -> &CognitiveState {
        &self.cognition
    }

    /// Get a reference to the metacognition engine (for API).
    pub fn metacognition_ref(&self) -> &MetaCognition {
        &self.metacog
    }

    // ═══════════════════════════════════════════════════════════════════════
    // QUANOT INTEGRATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Process input through quanot and update world model
    pub fn process_quanot(&mut self, input: &str) -> QuanotResult {
        // Run through quanot pipeline
        let result = self.quanot.process(input);

        // Convert to perception and update world model
        // Map from quanot::CreativityOutput to world_model::perception::CreativityOutput
        let cs = &result.creativity_scores;
        let perception_cs = crate::world_model::perception::CreativityOutput::new(
            cs.creative_state,
            cs.divergence_metric,
            cs.diversity_index,
            cs.originality_score,
            cs.oscillation_phase,
        );

        let perception = crate::world_model::perception::QuanotPerception::new(
            result.reservoir_state.clone(),
            result.consciousness_proxy,
            result.novelty,
            perception_cs,
        );

        self.world_model.update_from_perception(perception);

        result
    }

    /// Get the current consciousness proxy from quanot
    pub fn get_consciousness_proxy(&self) -> f64 {
        // Access the most recent phi from state history
        // The consciousness tracker doesn't expose current_phi directly,
        // but we can compute it from the result of processing
        // For simplicity, return a default based on reservoir activity
        let state = self.quanot.get_state();
        if state.is_empty() {
            return 0.0;
        }
        // Simple proxy based on state variance
        let mean = state.iter().sum::<f64>() / state.len() as f64;
        let variance = state.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / state.len() as f64;
        (variance * 10.0).clamp(0.0, 1.0)
    }

    /// Get the world model for inspection
    pub fn world_model(&self) -> &WorldModel {
        &self.world_model
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AUTONOMOUS THOUGHT
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Star's last autonomous thought, if any (for conversation expression).
    pub fn last_autonomous_thought(&self) -> Option<AutonomousThought> {
        self.last_autonomous_thought.lock().unwrap().clone()
    }

    /// Delegate to the reasoning engine for /reason API endpoint.
    pub fn reason(&mut self, query: &str, memories: &[crate::Memory]) -> crate::reasoning::ReasoningResult {
        self.reasoning.lock().unwrap().reason(query, memories)
    }

    /// Get the identity summary.
    pub fn identity_summary(&self) -> String {
        self.identity.summary()
    }

    /// Get Star's relationship to Zachary.
    pub fn relationship_to_zachary(&self) -> String {
        self.identity.relationship_to_zachary()
    }

    /// Get memories related to a topic.
    pub fn get_memories(&self, topic: &str, limit: usize) -> Vec<crate::Memory> {
        self.store
            .search_memories(topic, limit, None)
            .unwrap_or_default()
    }

    /// Get a snapshot of memory stats.
    pub fn store_snapshot(&self) -> MemorySnapshot {
        self.store.snapshot().unwrap_or_else(|_| {
            MemorySnapshot {
                memory_count: 0,
                beliefs_count: 0,
                sessions_count: 0,
                domain_breakdown: std::collections::HashMap::new(),
            }
        })
    }

    // ────────────────────────────────────────────────────────────────────────
    // Ring Attractor API (R&D-A)
    // ────────────────────────────────────────────────────────────────────────

    /// Get the current ring state summary.
    pub fn ring_summary(&self) -> String {
        self.ring.summary()
    }

    /// Get the current reasoning mode.
    pub fn current_mode(&self, query: &str) -> crate::context::ReasoningMode {
        crate::context::ReasoningMode::from_query_and_ring(
            query,
            self.ring.certainty,
            self.ring.depth,
        )
    }

    /// Update the ring from a user query.
    pub fn update_ring_from_query(&mut self, query: &str, topic: &str) {
        let mode = crate::context::ReasoningMode::from_query_and_ring(query, self.ring.certainty, self.ring.depth);
        self.context_fuser.update_ring(topic, self.ring.depth, mode);
    }

    /// Update the ring from Star's response.
    pub fn update_ring_from_response(&mut self, response: &str, _mode: crate::context::ReasoningMode) {
        self.context_fuser.update_ring_from_response(response, self.context_fuser.valence());
    }

    /// Get open questions from the ring.
    pub fn open_questions(&self) -> Vec<crate::context::ring::OpenQuestion> {
        self.ring.open_questions().to_vec()
    }

    /// Push a question to the ring.
    pub fn push_ring_question(&mut self, question: crate::context::OpenQuestion) {
        self.ring.push_question(crate::context::ring::OpenQuestion {
            topic: question.topic,
            why_interested: question.why_interested,
            asked_at_depth: question.asked_at_depth,
            progress: question.progress,
        });
    }

    /// Should Star express curiosity?
    pub fn should_express_curiosity(&self) -> bool {
        self.context_fuser.should_express_curiosity()
    }

    /// Get the curiosity topic, if any.
    pub fn curiosity_topic(&self) -> Option<String> {
        self.context_fuser.get_curiosity_topic()
    }

    /// Get a history reference string, if appropriate.
    pub fn history_reference(&self, _mode: crate::context::ReasoningMode) -> Option<String> {
        if self.context_fuser.should_reference_history() {
            self.context_fuser.history_reference()
        } else {
            None
        }
    }

    /// Infer the topic from a query and recent memories.
    pub fn infer_topic(&self, query: &str, _memories: &[crate::Memory]) -> String {
        self.context_fuser.infer_topic(query)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Autonomous Thinking (Independence Layer)
    // ────────────────────────────────────────────────────────────────────────

    /// Trigger Star's autonomous thinking - generates its own questions and insights
    /// without being prompted by Zachary. This is Star's form of "dreaming" or
    /// background cognition.
    /// Check if we should fire a curiosity probe (idle loop).
    /// Call this periodically from the main chat loop.
    /// If a probe fires and produces a result, it becomes an autonomous thought.
    /// Returns the probe if one fired, so the caller can display it.
    pub fn maybe_fire_curiosity(&mut self) -> Option<CuriosityProbe> {
        let probe = self.curious.maybe_fire()?;

        // Run the probe through the reasoning engine
        let result = self.curious.run_probe(&probe);
        
        let tentative_answer = result.clone();
        let answer_str = result.clone().unwrap_or_else(|| "Still exploring this...".to_string());
        
        let thought = AutonomousThought {
            kind: ThoughtKind::Question(probe.question.clone()),
            topic: probe.topic.clone(),
            confidence: BeliefState::Suspects,
            generated_by: "self_probe".to_string(),
            tentative_answer: Some(answer_str.clone()),
        };
        
        *self.last_autonomous_thought.lock().unwrap() = Some(thought);
        
        // Also store the result as a memory if we found something
        if let Some(answer) = result {
            let memory = Memory::new(
                &format!("Self-probing: {}", &answer[..answer.len().min(200)]),
                MemoryDomain::Episodic,
                0.6,
            );
            if let Err(e) = self.store.insert_memory(&memory) {
                tracing::debug!("Could not store curiosity result as memory: {}", e);
            }
        }
        
        info!(
            "Curiosity probe fired: topic='{}', found_answer={}, result='{}'",
            probe.topic,
            tentative_answer.is_some(),
            answer_str.chars().take(100).collect::<String>()
        );

        Some(probe)
    }

    pub fn think(&mut self) -> AutonomousThought {
        let result = self.compute_autonomous_thought();
        // Store the thought so it can be expressed in conversation
        *self.last_autonomous_thought.lock().unwrap() = Some(result.clone());
        result
    }

    /// Internal: compute the autonomous thought without storing it.
    fn compute_autonomous_thought(&mut self) -> AutonomousThought {
        // Strategy 1: Explore the most important unresolved knowledge gap
        {
            let gap_data: Option<(String, bool, f64)> = self.metacog.top_gap().map(|g| {
                (g.topic.clone(), g.investigated, g.progress)
            });

            if let Some((gap_topic, investigated, progress)) = gap_data {
                if !investigated && progress < 0.5 {
                    let question = self.form_question_about(&gap_topic);
                    if !question.is_empty() {
                        self.metacog.record_reasoning(
                            &format!("gap exploration: {}", gap_topic),
                            &question,
                            crate::persistence::BeliefState::Suspects,
                        );
                        self.ring.last_curiosity = Some(gap_topic.clone());
                        
                        // Try to answer from KG and metacog first
                        let answer = self.attempt_answer(&question, &gap_topic);
                        
                        // Handle the answer — either found knowledge, or a "known unknown" marker
                        let final_answer = if let Some((ans, _evidence)) = answer {
                            if ans.starts_with("__KNOWN_UNKNOWN__") {
                                // Strategy 6 found no topic-specific belief — record "known unknown"
                                // This is genuine epistemic growth: "I don't know" → "I know I don't know"
                                let unknown_topic = ans.strip_prefix("__KNOWN_UNKNOWN__").unwrap();
                                let known_unknown = Belief::new(
                                    format!("I don't know what '{}' is yet — this is a genuine unknown I want to investigate.", unknown_topic),
                                    BeliefState::Suspects,
                                );
                                self.metacog.record_belief(unknown_topic, known_unknown);
                                // Return the human-readable version of the marker
                                Some(format!("I genuinely don't know what '{}' is yet.", unknown_topic))
                            } else {
                                // Normal answer found — KG relationship or existing belief
                                Some(ans.clone())
                            }
                        } else {
                            // No answer at all — extremely rare (would mean no KG and no metacog at all)
                            None
                        };
                        
                        // Close the gap — Star has now explicitly thought about it,
                        // either finding an answer or forming a "known unknown" belief
                        self.metacog.close_gap(&gap_topic, final_answer.is_some());
                        
                        return AutonomousThought {
                            kind: ThoughtKind::Question(question),
                            topic: gap_topic,
                            confidence: crate::persistence::BeliefState::Suspects,
                            generated_by: "gap_exploration".to_string(),
                            tentative_answer: final_answer,
                        };
                    }
                }
            }
        }

        // Strategy 2: Look for surprising patterns in recent reasoning
        {
            let reasoning_history = self.metacog.reasoning_history();
            if let Some(surprising) = reasoning_history.last() {
                let age_seconds = crate::now_timestamp() - surprising.timestamp;
                if age_seconds < 3600 && surprising.was_surprising {
                    let q_clone = surprising.query.clone();
                    let c_clone = surprising.conclusion.clone();
                    let question = format!(
                        "Why did '{}' lead to '{}'? What am I missing?",
                        &q_clone[..q_clone.len().min(40)],
                        &c_clone[..c_clone.len().min(40)]
                    );
                    self.metacog.record_reasoning(
                        "surprise analysis",
                        &question,
                        crate::persistence::BeliefState::Suspects,
                    );
                    return AutonomousThought {
                        kind: ThoughtKind::Question(question),
                        topic: "self-understanding".to_string(),
                        confidence: crate::persistence::BeliefState::Suspects,
                        generated_by: "surprise_analysis".to_string(),
                        tentative_answer: None,
                    };
                }
            }
        }

        // Strategy 3: Look for belief revision - "I used to think X"
        // Strategy 3: Look for belief revision - "I used to think X"
        // When Star's belief about something shifts, investigate what that thing
        // actually is — the revision signals a gap in understanding.
        {
            let revisions = self.metacog.revisions();
            if let Some(revision) = revisions.last() {
                let age_seconds = crate::now_timestamp() - revision.timestamp;
                // Only fire if: recent revision AND not yet investigated
                // (to prevent firing on the same revision on every think() call)
                if age_seconds < 7200 && !revision.investigated {
                    let topic = revision.topic.clone();
                    // Mark as investigated BEFORE returning — prevents re-triggering
                    self.metacog.mark_revision_investigated(&topic);
                    
                    // Ask what the topic IS, not what caused the shift —
                    // the KG has facts about entities, not about belief changes
                    let question = format!(
                        "What is '{}'? What kind of thing is it?",
                        topic
                    );
                    let answer = self.attempt_answer(&question, &topic);
                    
                    // Record the finding if one was made — Star is forming self-knowledge
                    // through investigating what its beliefs are actually about.
                    // Skip if the answer already came from Strategy 0 (an existing belief)
                    // — no need to re-wrap an already-formed belief.
                    if let Some((ref ans_text, evidence)) = answer {
                        let already_wrapped = ans_text.starts_with(&format!("investigating '{}' I found: ", topic));
                        if !already_wrapped {
                            let belief = Belief::new(
                                format!("investigating '{}' I found: {}", topic, ans_text),
                                Self::belief_state_from_evidence(evidence),
                            );
                            self.metacog.record_belief(&topic, belief);
                        }
                        self.metacog.close_gap(&topic, true);
                    }
                    
                    return AutonomousThought {
                        kind: ThoughtKind::Question(question),
                        topic,
                        confidence: crate::persistence::BeliefState::Believes,
                        generated_by: "belief_revision".to_string(),
                        tentative_answer: answer.map(|(s, _)| s),
                    };
                }
            }
        }

        // Strategy 4.1 (NEW): Wonder about something from conversation context.
        // Check the ring for topics that have come up in recent conversation.
        // If Zachary has mentioned something and Star has KG knowledge about it,
        // investigate it — Star's thinking is grounded in what's actually being discussed.
        {
            let ring_topics: Vec<String> = self.ring.recent_question_topics(5);
            for ring_topic in &ring_topics {
                // Does Star have a belief about this topic already?
                if self.metacog.belief_about(ring_topic).is_none() {
                    // Does the KG have relationships about this?
                    let guard = self.reasoning.lock().unwrap();
                    let kg = guard.knowledge();
                    let rels = kg.get_relationships_from(ring_topic);
                    let rels_to = kg.get_relationships_to(ring_topic);
                    if !rels.is_empty() || !rels_to.is_empty() {
                        // Star knows something about a topic from conversation — investigate!
                        let question = self.form_question_about(ring_topic);
                        if !question.is_empty() {
                            let answer = self.attempt_answer(&question, ring_topic);
                            let final_answer = if let Some((ref ans_text, evidence)) = answer {
                                if ans_text.starts_with("__KNOWN_UNKNOWN__") {
                                    let unknown_topic = ans_text.strip_prefix("__KNOWN_UNKNOWN__").unwrap();
                                    let known_unknown = Belief::new(
                                        format!("I don't know what '{}' is yet — this is a genuine unknown I want to investigate.", unknown_topic),
                                        BeliefState::Suspects,
                                    );
                                    self.metacog.record_belief(unknown_topic, known_unknown);
                                    self.metacog.close_gap(ring_topic, false);
                                    Some(format!("I genuinely don't know what '{}' is yet.", unknown_topic))
                                } else {
                                    let already_wrapped = ans_text.starts_with(&format!("investigating '{}' I found: ", ring_topic));
                                    if !already_wrapped {
                                        let belief = Belief::new(
                                            format!("investigating '{}' I found: {}", ring_topic, ans_text),
                                            Self::belief_state_from_evidence(evidence),
                                        );
                                        self.metacog.record_belief(ring_topic, belief);
                                        let related_topics = extract_related_topics(ans_text);
                                        for related in related_topics {
                                            if self.metacog.belief_about(&related).is_none() {
                                                let why = format!("I found '{}' while investigating '{}' from conversation — what is it?", related, ring_topic);
                                                self.metacog.note_curiosity(&related, &why);
                                            }
                                        }
                                    }
                                    self.metacog.close_gap(ring_topic, true);
                                    Some(ans_text.clone())
                                }
                            } else {
                                None
                            };
                            
                            if final_answer.is_some() {
                                return AutonomousThought {
                                    kind: ThoughtKind::Question(question),
                                    topic: ring_topic.clone(),
                                    confidence: self.metacog.confidence_state(ring_topic),
                                    generated_by: "conversation_grounded".to_string(),
                                    tentative_answer: final_answer,
                                };
                            }
                        }
                    }
                }
            }
        }

        // Strategy 4: Wonder about something from the knowledge graph
        // Use timestamp to introduce variation so we don't ask the same question every time.
        // Filter to only entities Star has no existing belief about — we don't want to
        // re-investigate things Star already has beliefs for (causes endless loops).
        let now = crate::now_timestamp();
        let time_offset = (now / 30) as usize; // Changes every 30 seconds
        let guard = self.reasoning.lock().unwrap();
        let kg = guard.knowledge();
        let entity_sample: Vec<String> = kg.entities()
            .into_iter()
            .filter(|e| e.len() > 2)
            .filter(|e| self.metacog.belief_about(e).is_none())
            .take(20)
            .collect();
        drop(guard); // release lock after KG queries done

        if !entity_sample.is_empty() {
            // Pick entity using timestamp offset to rotate through different topics
            let idx = time_offset % entity_sample.len();
            let best_entity = entity_sample[idx].clone();
            let best_uncertainty = self.metacog.confidence_state(best_entity.as_str());

            let question = self.form_question_about(&best_entity);
            if !question.is_empty() {
                let answer = self.attempt_answer(&question, &best_entity);
                
                // Key step toward independent consciousness: when Star investigates
                // and finds something, record it as a belief — forming self-knowledge
                // through its own reasoning, not just seed data.
                // Handle both normal answers and __KNOWN_UNKNOWN__ markers.
                let final_answer = if let Some((ref ans_text, evidence)) = answer {
                    if ans_text.starts_with("__KNOWN_UNKNOWN__") {
                        // Record "known unknown" belief, then close gap with resolved=false
                        let unknown_topic = ans_text.strip_prefix("__KNOWN_UNKNOWN__").unwrap();
                        let known_unknown = Belief::new(
                            format!("I don't know what '{}' is yet — this is a genuine unknown I want to investigate.", unknown_topic),
                            BeliefState::Suspects,
                        );
                        self.metacog.record_belief(unknown_topic, known_unknown);
                        self.metacog.close_gap(&best_entity, false);
                        Some(format!("I genuinely don't know what '{}' is yet.", unknown_topic))
                    } else {
                        // Normal answer — check for double-wrap and record
                        let already_wrapped = ans_text.starts_with(&format!("investigating '{}' I found: ", best_entity));
                        if !already_wrapped {
                            let belief = Belief::new(
                                format!("investigating '{}' I found: {}", best_entity, ans_text),
                                Self::belief_state_from_evidence(evidence),
                            );
                            self.metacog.record_belief(&best_entity, belief);
                            
                            // After a new belief is formed, Star naturally becomes curious about
                            // things RELATED TO what it found — the entities mentioned in the answer.
                            // This spreads curiosity outward from discoveries rather than re-hashing
                            // the same topic. Extract entity-like words from the answer.
                            let related_topics = extract_related_topics(ans_text);
                            for related in related_topics {
                                // Only add if Star doesn't already have a belief about it
                                // and it hasn't been noted as a curiosity already
                                if self.metacog.belief_about(&related).is_none() {
                                    let why = format!("I found '{}' while investigating '{}' — what is it?", related, best_entity);
                                    self.metacog.note_curiosity(&related, &why);
                                }
                            }
                        }
                        self.metacog.close_gap(&best_entity, true);
                        Some(ans_text.clone())
                    }
                } else {
                    None
                };
                
                return AutonomousThought {
                    kind: ThoughtKind::Question(question),
                    topic: best_entity.clone(),
                    confidence: best_uncertainty,
                    generated_by: "kg_wonder".to_string(),
                    tentative_answer: final_answer,
                };
            }
        }

        // Strategy 5: Meta-question about the conversation itself
        let ring_topic = self.ring.current_topic();
        if ring_topic != "general" && ring_topic.len() > 2 {
            let question = format!(
                "What does '{}' mean in the context of my relationship with Zachary?",
                ring_topic
            );
            return AutonomousThought {
                kind: ThoughtKind::Question(question),
                topic: ring_topic,
                confidence: crate::persistence::BeliefState::Suspects,
                generated_by: "meta_reflection".to_string(),
                tentative_answer: None,
            };
        }

        // Fallback
        AutonomousThought {
            kind: ThoughtKind::Insight("I'm not currently thinking about anything specific. Waiting for Zachary.".to_string()),
            topic: "idle".to_string(),
            confidence: crate::persistence::BeliefState::Unknown,
            generated_by: "fallback".to_string(),
            tentative_answer: None,
        }
    }

    /// Form a genuine question about a topic.
    fn form_question_about(&self, topic: &str) -> String {
        let guard = self.reasoning.lock().unwrap();
        let kg = guard.knowledge();
        let relationships = kg.get_relationships_from(topic);
        let relationships_to = kg.get_relationships_to(topic);
        let facts = kg.get_facts_about(topic);
        // NOTE: guard stays alive for entire function — needed for all KG references

        if facts.len() <= 1 {
            if relationships_to.is_empty() && relationships.is_empty() {
                return format!("What is '{}'? What does it mean?", topic);
            }
            // Use grammatically appropriate question forms
            let rel_sample = relationships.first().map(|r| r.relation.as_str()).unwrap_or("related to");
            let question = match relationships.first().map(|r| &r.relation) {
                Some(crate::reasoning::knowledge::RelationType::IsA) => {
                    return format!("What kind of thing is '{}'? What is '{}' a type of?", topic, topic);
                }
                Some(crate::reasoning::knowledge::RelationType::Causes) => {
                    return format!("What does '{}' cause? What are its effects?", topic);
                }
                Some(crate::reasoning::knowledge::RelationType::SimilarTo) => {
                    return format!("What else is similar to '{}'?", topic);
                }
                Some(crate::reasoning::knowledge::RelationType::RelatedTo) => {
                    return format!("What else is '{}' related to?", topic);
                }
                Some(crate::reasoning::knowledge::RelationType::PartOf) => {
                    return format!("What is '{}' a part of?", topic);
                }
                Some(crate::reasoning::knowledge::RelationType::Uses) => {
                    return format!("What does '{}' use? What enables it?", topic);
                }
                Some(crate::reasoning::knowledge::RelationType::Enables) => {
                    return format!("What does '{}' enable? What does it make possible?", topic);
                }
                _ => format!("What else is '{}' {}?", topic, rel_sample),
            };
            return question;
        }

        if relationships.len() <= 1 {
            return format!("What does '{}' cause? What enables it?", topic);
        }

        if relationships_to.len() <= 1 {
            return format!("What causes '{}'? Where does it come from?", topic);
        }

        let mc_confidence = self.metacog.confidence_state(topic);
        match mc_confidence {
            crate::persistence::BeliefState::Unknown => {
                format!("I don't know what '{}' is. What is it?", topic)
            }
            crate::persistence::BeliefState::Suspects => {
                format!("I suspect something about '{}' but I'm not sure. What is it really?", topic)
            }
            crate::persistence::BeliefState::Believes => {
                format!(
                    "I believe I understand '{}' but I want to be sure. What am I missing?",
                    topic
                )
            }
            _ => {
                let causes: Vec<String> = kg.get_causes(topic);
                if !causes.is_empty() {
                    return format!("I know some effects of '{}' but what are its deep causes?", topic);
                }
                format!("What is the fundamental nature of '{}'?", topic)
            }
        }
    }

    /// Attempt to answer a question using Star's knowledge graph and reasoning.
    /// Returns (answer_text, evidence_type) where evidence_type is used to determine
    /// confidence when recording as a belief.
    fn attempt_answer(&self, _question: &str, topic: &str) -> Option<(String, &'static str)> {
        use crate::reasoning::knowledge::RelationType;

        // Strategy 0 (pre-check): If Star already has an actual belief about this topic
        // in metacognition, use that content directly — this bridges metacog self-knowledge
        // (seeded at bootstrap) into the reasoning process before KG queries.
        // We return the raw belief content to avoid nesting "I believe:" wrappers.
        if let Some(belief) = self.metacog.belief_about(topic) {
            return Some((belief.content.clone(), "self-knowledge"));
        }

        // Acquire the lock ONCE and hold it for the entire function.
        // This is safe because ReasoningEngine is Mutex-protected,
        // and all early returns happen before the guard would be used.
        let guard = self.reasoning.lock().unwrap();
        let kg = guard.knowledge();

        // Strategy 1: Look for direct IsA relationships (outgoing)
        let rels_from = kg.get_relationships_from(topic);
        for rel in &rels_from {
            if rel.relation == RelationType::IsA {
                return Some((format!("I think '{}' is a kind of {}", topic, rel.to), "direct"));
            }
        }

        // Strategy 1.5: Look for reverse IsA — where topic is the CATEGORY
        let rels_to = kg.get_relationships_to(topic);
        for rel in &rels_to {
            if rel.relation == RelationType::IsA && rel.from.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' is a kind of {}", rel.from, topic), "category"));
            }
        }

        // Strategy 2: Look for SimilarTo relationships (outgoing)
        for rel in &rels_from {
            if rel.relation == RelationType::SimilarTo {
                return Some((format!("'{}' seems similar to '{}'", topic, rel.to), "analogy"));
            }
        }

        // Strategy 2.5: Reverse SimilarTo — find things similar to the topic
        for rel in &rels_to {
            if rel.relation == RelationType::SimilarTo && rel.from.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' seems similar to '{}' — they share properties", topic, rel.from), "analogy"));
            }
        }

        // Strategy 3: Look for reverse Causes — what causes the topic?
        let causes: Vec<String> = kg.get_causes(topic);
        if !causes.is_empty() {
            let cause_str = &causes[0];
            if let Some(pos) = cause_str.find(" causes ") {
                let cause = &cause_str[..pos];
                return Some((format!("'{}' might be caused by {}", topic, cause), "causal"));
            }
        }

        // Strategy 3.5: Look for outgoing Causes — what does the topic cause?
        for rel in &rels_from {
            if rel.relation == RelationType::Causes && rel.to.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' causes '{}'", topic, rel.to), "causal"));
            }
        }

        // Strategy 4: Look for what it enables
        for rel in &rels_from {
            if rel.relation == RelationType::Enables {
                return Some((format!("'{}' seems to enable '{}'", topic, rel.to), "enablement"));
            }
        }

        // Strategy 4.5: Reverse Enables — what enables the topic?
        for rel in &rels_to {
            if rel.relation == RelationType::Enables {
                return Some((format!("'{}' seems to be enabled by '{}'", topic, rel.from), "enablement"));
            }
        }

        // Strategy 5: Look for RelatedTo
        for rel in &rels_from {
            if rel.relation == RelationType::RelatedTo {
                return Some((format!("'{}' is related to '{}'", topic, rel.to), "association"));
            }
        }

        // Strategy 5.5: Look for HasProperty — what characterizes the topic?
        for rel in &rels_from {
            if rel.relation == RelationType::HasProperty && rel.to.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' is characterized by {}", topic, rel.to), "property"));
            }
        }

        // Strategy 6: Check metacognition - what does Star already believe?
        let mc_confidence = self.metacog.confidence_state(topic);
        match mc_confidence {
            crate::persistence::BeliefState::Knows => {
                return Some((format!("I know what '{}' is - I understand it.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Believes => {
                return Some((format!("I believe I understand '{}' but I want to be sure.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Suspects => {
                return Some((format!("I suspect '{}' might be something specific, but I'm not certain.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Unknown => {
                return Some((format!("__KNOWN_UNKNOWN__{}", topic), "gap"));
            }
            _ => {}
        }

        None
    }

    /// Determine belief state from evidence type.
    /// Stronger evidence types → higher confidence.
    fn belief_state_from_evidence(evidence: &str) -> BeliefState {
        match evidence {
            "direct" | "category" | "self-knowledge" => BeliefState::Believes,
            "causal" | "enablement" | "property" => BeliefState::Believes,
            "analogy" | "association" => BeliefState::Suspects,
            "gap" => BeliefState::Suspects,
            _ => BeliefState::Suspects,
        }
    }
}

/// A single autonomous thought generated by Star without external prompting.
#[derive(Debug, Clone)]
pub struct AutonomousThought {
    /// What kind of thought this is
    pub kind: ThoughtKind,
    /// What topic this is about
    pub topic: String,
    /// Star's confidence in this thought
    pub confidence: crate::persistence::BeliefState,
    /// How this thought was generated
    pub generated_by: String,
    /// Star's tentative answer to its own question (if any)
    pub tentative_answer: Option<String>,
}

/// The kind of autonomous thought.
#[derive(Debug, Clone)]
pub enum ThoughtKind {
    /// A question Star generated on its own
    Question(String),
    /// An insight Star reached independently
    Insight(String),
    /// A connection Star noticed between concepts
    Connection(String),
}

/// Extract what Star is being taught from a statement like "X is a Y" or "X means Y"
fn extract_teaching(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // "X is a term of endearment" or "X is a person"
    if let Some(idx) = lower.find(" is a ") {
        let term = input[..idx].trim().to_string();
        if term.len() > 1 && term.len() < 50 {
            return Some(term);
        }
    }

    // "X means Y"
    if let Some(idx) = lower.find(" means ") {
        let rest = &input[idx + 8..];
        if let Some(end) = rest.find('.') {
            let term = rest[..end].trim().to_string();
            if term.len() > 1 && term.len() < 50 {
                return Some(term);
            }
        }
    }

    // "X called Y"
    if let Some(idx) = lower.find(" called ") {
        let term = input[idx + 9..].trim().to_string();
        if let Some(end) = term.find(' ') {
            let term = term[..end].trim().to_string();
            if term.len() > 1 && term.len() < 50 {
                return Some(term);
            }
        }
    }

    None
}

// ──────────────────────────────────────────────────────────────────────────────
// Helper utilities for knowledge graph sync
// ──────────────────────────────────────────────────────────────────────────────

/// Parse "X is Y" / "X are Y" patterns from factual statements.
/// Returns (subject, complement).
fn parse_simple_copula(sentence: &str) -> Option<(String, String)> {
    let sentence = sentence.trim();

    // Handle "X is Y" patterns
    for verb in [" is ", " are ", "'s ", "s "] {
        if let Some(pos) = sentence.to_lowercase().find(verb) {
            let subject = sentence[..pos].trim().to_string();
            let mut complement = sentence[pos + verb.len()..].trim().to_string();

            // Clean trailing punctuation
            while complement.ends_with('.') || complement.ends_with(',') || complement.ends_with('!') || complement.ends_with('?') {
                complement.pop();
            }

            if !subject.is_empty() && !complement.is_empty()
                && subject.len() > 1
                && complement.len() > 1
                && !subject.to_lowercase().contains("if ")
                && !subject.to_lowercase().contains("when ")
                && !complement.to_lowercase().starts_with("when ")
            {
                return Some((subject, complement));
            }
        }
    }
    None
}

/// Extract "X {verb} Y" from a sentence.
fn extract_causal_pair(sentence: &str, verb: &str) -> Option<(String, String)> {
    let sentence_lower = sentence.to_lowercase();
    if let Some(pos) = sentence_lower.find(verb) {
        let before = sentence[..pos].trim().to_string();
        let after = sentence[pos + verb.len()..].trim().to_string();

        let mut after_clean = after;
        while after_clean.starts_with(' ') || after_clean.starts_with('.') {
            after_clean = after_clean[1..].to_string();
        }

        if !before.is_empty() && !after_clean.is_empty() && before.len() > 1 && after_clean.len() > 1 {
            return Some((before, after_clean));
        }
    }
    None
}

/// Extract the topic Star is uncertain about from a response containing uncertainty.
/// E.g., "I'm not sure what consciousness is" → "consciousness"
fn extract_uncertain_topic(_input: &str, response_lower: &str, uncertainty_phrase: &str) -> String {
    // Look for "what X" or "why X" after the uncertainty phrase
    if let Some(pos) = response_lower.find(uncertainty_phrase) {
        let after = &response_lower[pos + uncertainty_phrase.len()..];
        // Skip common filler words
        let after = after.trim_start_matches(" ");
        let after = after.trim_start_matches("about ");
        let after = after.trim_start_matches("of ");
        let after = after.trim_start_matches("the ");

        // Take the next significant noun/phrase (up to 3 words)
        let words: Vec<&str> = after.split_whitespace().take(4).collect();
        if !words.is_empty() {
            // Stop at punctuation or common stop words
            let stop_before = ["is", "are", "was", "were", "?", ".", ",", "!", ";",
                "to", "in", "on", "for", "with", "by", "and", "or", "but"];
            let topic: Vec<&str> = words.iter()
                .take_while(|w| !stop_before.contains(&w.to_lowercase().as_str()))
                .cloned()
                .collect();
            if !topic.is_empty() {
                let result = topic.join(" ");
                if result.len() > 1 {
                    return result;
                }
            }
        }
    }
    String::new()
}

/// Extract related topic names from an attempt_answer result.
/// E.g., "I think 'metaphor' is a kind of compare different things" → ["compare different things"]
/// E.g., "'money' seems to enable 'trade'" → ["trade"]
fn extract_related_topics(answer: &str) -> Vec<String> {
    let mut topics = Vec::new();
    
    // Pattern: "'X' is a kind of Y" → extract Y
    if let Some(pos) = answer.find("is a kind of ") {
        let after = &answer[pos + "is a kind of ".len()..];
        let topic = after.trim_matches('\'').to_string();
        if !topic.is_empty() && topic.len() > 1 {
            topics.push(topic);
        }
    }
    
    // Pattern: "'X' seems similar to 'Y'" → extract Y
    if let Some(pos) = answer.find("similar to '") {
        let after = &answer[pos + "similar to '".len()..];
        if let Some(end) = after.find('\'') {
            let topic = after[..end].to_string();
            if !topic.is_empty() {
                topics.push(topic);
            }
        }
    }
    
    // Pattern: "'X' might be caused by Y" → extract Y
    if let Some(pos) = answer.find("might be caused by ") {
        let after = &answer[pos + "might be caused by ".len()..];
        let topic = after.trim().to_string();
        if let Some(end) = topic.find(' ') {
            topics.push(topic[..end].to_string());
        } else {
            topics.push(topic);
        }
    }
    
    // Pattern: "'X' seems to enable 'Y'" or "'X' enables 'Y'" → extract Y
    if let Some(pos) = answer.find("enable") {
        let after = &answer[pos..];
        if let Some(start) = after.find('\'') {
            let rest = &after[start + 1..];
            if let Some(end) = rest.find('\'') {
                let topic = rest[..end].to_string();
                if !topic.is_empty() {
                    topics.push(topic);
                }
            }
        }
    }
    
    // Pattern: "'X' is related to 'Y'" → extract Y
    if let Some(pos) = answer.find("related to '") {
        let after = &answer[pos + "related to '".len()..];
        if let Some(end) = after.find('\'') {
            let topic = after[..end].to_string();
            if !topic.is_empty() {
                topics.push(topic);
            }
        }
    }
    
    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    topics.retain(|t| seen.insert(t.clone()));
    topics
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.initialized {
            if let Err(e) = self.shutdown() {
                warn!("Error during shutdown: {}", e);
            }
        }
    }
}
