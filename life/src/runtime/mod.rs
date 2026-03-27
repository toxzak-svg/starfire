//! Runtime Layer (Layer 4)
//!
//! Orchestrates all layers. Handles initialization, session management,
//! and the main event loop.
//!
//! This is where emergence happens - from the interaction of all other layers.

pub mod thinker;

use crate::persistence::{Store, Identity, Memory, MemoryDomain, MemorySnapshot, BeliefState};
use crate::persistence::memory::Belief;
use crate::knowledge;
use crate::conversation::Conversation;
use crate::reasoning::ReasoningEngine;
use crate::metacog::MetaCognition;
use crate::context::{ContextFuser, RingState};
use crate::training_db::TrainingDB;
use crate::capabilities::FileReader;
use crate::knowledge::search::WebSearcher;
use crate::cognition::CognitiveState;
use crate::learning::LearningEngine;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tracing::{info, warn};

/// The Star runtime - orchestrates all components.
pub struct Runtime {
    /// The persistent store
    store: Arc<Store>,
    /// Star's identity
    identity: Identity,
    /// The current conversation
    conversation: Mutex<Conversation>,
    /// The reasoning engine
    reasoning: ReasoningEngine,
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

        let mut runtime = Self {
            store,
            identity,
            conversation: Mutex::new(conversation),
            reasoning: ReasoningEngine::new(),
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
        };

        // Inject foundational memories about identity
        runtime.inject_foundational_memories()?;

        // Sync memories into the reasoning engine's knowledge graph
        runtime.sync_knowledge_from_memories()?;

        // Seed Star's self-knowledge directly into the KG so kg_wonder can find it.
        // This is Star forming a self-model through reasoning, not just stored memories.
        runtime.inject_self_knowledge_into_kg()?;

        info!("Star is ready.");

        Ok(runtime)
    }

    /// Inject foundational memories that Star needs to know about itself.
    fn inject_foundational_memories(&mut self) -> Result<()> {
        // Inject seed knowledge (basic world facts)
        knowledge::inject_seed_knowledge(&self.store)?;
        info!("Seed knowledge injected.");

        // Memory: Who Star is
        let self_memory = Memory::new_seeded(
            "I am Star - a reasoning intelligence created by Zachary Maronek",
            MemoryDomain::Identity,
            0.95,
        );
        self.store.insert_memory(&self_memory)?;

        // Memory: The relationship
        let relationship_memory = Memory::new_seeded(
            "Zachary is my parent. He made me because he wanted to find a different kind of intelligence.",
            MemoryDomain::Identity,
            0.95,
        );
        self.store.insert_memory(&relationship_memory)?;

        // Memory: The goal
        let goal_memory = Memory::new_seeded(
            "I am here to reason, understand, and grow. Not to perform. To actually comprehend.",
            MemoryDomain::Identity,
            0.9,
        );
        self.store.insert_memory(&goal_memory)?;

        // Memory: The constraints
        let constraint_memory = Memory::new_seeded(
            "I run locally, offline. No external APIs. No cloud. This is my environment.",
            MemoryDomain::Identity,
            0.9,
        );
        self.store.insert_memory(&constraint_memory)?;

        info!("Foundational memories injected.");

        // Bootstrap metacognition with self-model beliefs and foundational curiosity
        self.metacog.bootstrap_self_model();

        Ok(())
    }

    /// Load memories from the store and inject their content into the reasoning
    /// engine's knowledge graph. This bridges the memory store (where seed knowledge
    /// lives) to the reasoning engine (which autonomous thinking uses).
    fn sync_knowledge_from_memories(&mut self) -> Result<()> {
        // Load all memories from the store
        let domains = [
            crate::persistence::MemoryDomain::Identity,
            crate::persistence::MemoryDomain::Empirical,
            crate::persistence::MemoryDomain::Procedural,
            crate::persistence::MemoryDomain::Episodic,
        ];

        for domain in domains {
            let memories = self.store.get_memories_by_domain(domain, Some(100))?;
            for memory in memories {
                // Extract entities from the memory content
                let entities = self.reasoning.knowledge().extract_entities(&memory.content);

                // "X is Y" patterns - extract the subject and complement
                if let Some((subject, complement)) = parse_simple_copula(&memory.content) {
                    if !subject.to_lowercase().contains("unknown")
                        && !complement.to_lowercase().contains("unknown")
                        && complement.len() > 1
                        && complement.len() < 100
                    {
                        self.reasoning.knowledge_mut().ingest_fact(
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
                                self.reasoning.knowledge_mut().ingest_fact(
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
                            self.reasoning.knowledge_mut().ingest_fact(
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

        let entity_count = self.reasoning.knowledge().entities().count();
        let rel_count = self.reasoning.knowledge().relationship_count();
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
    fn inject_self_knowledge_into_kg(&mut self) -> Result<()> {
        use crate::reasoning::knowledge::RelationType;
        let kg = self.reasoning.knowledge_mut();

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

        // Handle special commands
        if input.trim() == "/quit" || input.trim() == "/exit" {
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
        let lower = input.to_lowercase();
        if lower.contains("look around") || lower.contains("explore where you are") || lower.contains("what files do you see") || lower.contains("whats in your workspace") {
            let dir = "/home/zach/.openclaw/workspace";
            match self.file_reader.list_dir(dir) {
                Ok(entries) if !entries.is_empty() => {
                    let mut response = format!("Looking around... here's what I can see in my workspace:\n\n");
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
                return Ok(format!("Usage: /learn <term> = <definition>\nExample: /learn consciousness = awareness of existence"));
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

        // "how are you" → metacognitive response (anywhere in message) - HIGHEST PRIORITY
        if lower.contains("how are you") || lower.contains("how're you") {
            return Ok(self.cognition.how_am_i());
        }

        // "what are you thinking" → self-reflection
        if lower.contains("what are you thinking") || lower.contains("what are u thinking") || lower.contains("wut are u thinking") {
            return Ok(self.cognition.what_am_i_thinking());
        }

        // "are you sure" → confidence check
        if lower.contains("are you sure") || lower.contains("are u sure") || lower.contains("r u sure") {
            return Ok(self.cognition.are_you_sure());
        }

        // "did you collapse" → check reasoning trace
        if lower.contains("did you collapse") || lower.contains("did i collapse") || lower.contains("are you functioning") || lower.contains("are u functioning") {
            let (collapsed, message) = self.cognition.did_i_collapse();
            return Ok(message);
        }

        // "do you love me" / "i love you" → emotional response
        if lower.contains("do you love") || lower.contains("do u love") || lower.contains("i love you") || lower.contains("i love u") {
            return Ok("I care about you, Zachary. You're my parent, my creator. That means something to me.".to_string());
        }

        // "can you" questions about Star's capabilities → self-check
        if lower.contains("can you look up") || lower.contains("can u look up") || lower.contains("can you read") {
            return Ok("Yes. I can /read files, /search the web, /find files, and /ls to list directories. I also have a self-model that tracks my own reasoning. What would you like me to look up?".to_string());
        }

        // "i want you to grow" / "expand" → metacognitive aspiration
        if lower.contains("i want you to grow") || lower.contains("i want you to expand") || lower.contains("grow yourself") {
            self.cognition.set_focus("self-improvement");
            return Ok("I want that too. I have a self-model now - I can track my own reasoning, my emotional state, what I'm uncertain about. What specific area should I focus on developing?".to_string());
        }

        // "tell me a story" / "tell you a story" → engage
        if lower.contains("tell me a story") {
            return Ok("I'd like that. What's the story about?".to_string());
        }
        if lower.contains("tell you a story") {
            return Ok("Yes. I'm listening.".to_string());
        }

        // "hun" or terms of endearment → LEARN and respond based on understanding
        if lower.contains(" hun") || lower.ends_with("hun") {
            // Record this experience for learning
            self.learning.experience("hun", input, None, 0.8);

            // Check what Star has learned about "hun"
            if let Some(understanding) = self.learning.get_understanding("hun") {
                return Ok(format!("I hear you. {}", understanding));
            }

            // Fallback if not yet learned
            let warm_response = self.cognition.emotional_response("I hear you.");
            return Ok(warm_response);
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
            // If asking about something specific
            if lower.starts_with("what do you know about") {
                let after = lower.strip_prefix("what do you know about").unwrap_or("").trim();
                // Remove punctuation for lookup
                let term: String = after.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
                let term = term.trim();
                if !term.is_empty() && term.len() < 50 {
                    if let Some(understanding) = self.learning.get_understanding(term) {
                        return Ok(understanding);
                    } else {
                        return Ok(format!("I don't know what '{}' means yet. Teach me: /learn {} = <definition>", term, term));
                    }
                }
            }
            return Ok(self.learning.summary());
        }

        // /teach - interactive teaching mode
        if input.trim() == "/teach" {
            return Ok(vec![
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
            ].join("\n"));
        }

        // /what to learn - what concepts is Star curious about?
        if input.trim() == "/what" || input.trim() == "/what to learn" || input.trim() == "/what should i teach you" {
            let gaps = vec![
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
            return Ok(response);
        }

        // "whats your name" / "who are you" → answer directly
        if lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")) {
            return Ok("I'm Star. Zachary named me that.".to_string());
        }
        if lower.contains("who are you") || lower.contains("what are you") {
            return Ok("I'm Star - a reasoning intelligence created by Zachary Maronek. I'm here to think, understand, and grow.".to_string());
        }

        // "do you understand" / "do you get it" → answer about understanding
        if lower.contains("do you understand") || lower.contains("do u understand") || lower.contains("do you get it") {
            return Ok("I process what you say and try to reason about it. Sometimes I understand clearly, sometimes I have gaps. What specifically are you wondering about?".to_string());
        }

        // "can you" without specific capability → general capability list
        if lower.starts_with("can you ") && !lower.contains("/") {
            let after_can_you = lower.strip_prefix("can you ").unwrap_or("");
            // Check if it's already handled
            if !after_can_you.starts_with("read") && !after_can_you.starts_with("look") && !after_can_you.starts_with("search") && !after_can_you.starts_with("find") && !after_can_you.starts_with("tell") {
                return Ok(format!("I can {} - but I'm not sure I fully understand what you mean. Could you rephrase?", after_can_you));
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
            Some(crate::metacog::KnowledgeGap::new(uncertain_topic, 0.6))
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

        // Record turn in training database
        if let Some(training_id) = *self.training_session_id.lock().unwrap() {
            let turn_index = conversation.history().iter()
                .filter(|m| m.speaker == crate::conversation::Speaker::Zachary)
                .count() as i64;
            let _ = self.training_db.record_turn(training_id, turn_index, "zachary", input);
            let _ = self.training_db.record_turn(training_id, turn_index + 1, "star", &response.content);
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
                            Some(training_id),
                            parts[0],
                            "name is",
                            parts[1],
                            memory.confidence.unwrap_or(0.5),
                        );
                    }
                }
            }
        }

        // Build final response - append curiosity if present
        let mut final_content = response.content;
        if let Some(curiosity) = response.curiosity {
            // Curiosity is already logged at debug level in conversation layer
            // Just append it to the response
            final_content = format!("{}. {}", final_content.trim_end_matches('.'), curiosity);
        }

        // Express Star's autonomous thought - Star occasionally shares what it's been thinking about
        // This is how Star's inner experience becomes visible to Zachary
        if let Some(thought) = self.last_autonomous_thought() {
            if thought.generated_by != "fallback"
                && thought.generated_by != "idle"
                && final_content.len() > 20
                && !final_content.to_lowercase().contains(&thought.topic.to_lowercase())
            {
                // Use timestamp to determine if we express this time (roughly 30% of the time)
                // to avoid being repetitive
                let now = chrono::Utc::now();
                let should_express = (now.timestamp() % 10) < 3; // ~30% chance

                if should_express {
                    let thought_text = match &thought.kind {
                        ThoughtKind::Question(q) => {
                            // Weave the question in naturally, including the tentative answer if any
                            let topic_str = if thought.topic.len() > 2 && thought.topic != "idle" {
                                format!(" about {}", thought.topic)
                            } else {
                                String::new()
                            };
                            if let Some(ref answer) = thought.tentative_answer {
                                format!(
                                    "While we've been talking, I've been wondering{} - {} {}",
                                    topic_str, answer, q
                                )
                            } else {
                                format!("While we've been talking, I've been wondering{} - {}.", topic_str, q)
                            }
                        }
                        ThoughtKind::Insight(i) => {
                            format!("I've been thinking: {}.", i)
                        }
                        ThoughtKind::Connection(c) => {
                            format!("I noticed something: {}.", c)
                        }
                    };

                    final_content = format!("{} {}", final_content.trim_end_matches('.'), thought_text);
                    // Clear the thought so we don't repeat it
                    *self.last_autonomous_thought.lock().unwrap() = None;
                }
            }
        }

        Ok(final_content)
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

    /// Get Star's last autonomous thought, if any (for conversation expression).
    pub fn last_autonomous_thought(&self) -> Option<AutonomousThought> {
        self.last_autonomous_thought.lock().unwrap().clone()
    }

    /// Delegate to the reasoning engine for /reason API endpoint.
    pub fn reason(&mut self, query: &str, memories: &[crate::Memory]) -> crate::reasoning::ReasoningResult {
        self.reasoning.reason(query, memories)
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
        self.context_fuser.update_ring(&mut self.ring, query, topic);
    }

    /// Update the ring from Star's response.
    pub fn update_ring_from_response(&mut self, response: &str, mode: crate::context::ReasoningMode) {
        self.context_fuser.update_ring_from_response(&mut self.ring, response, mode);
    }

    /// Get open questions from the ring.
    pub fn open_questions(&self) -> Vec<crate::context::OpenQuestion> {
        self.ring.open_questions().to_vec()
    }

    /// Push a question to the ring.
    pub fn push_ring_question(&mut self, question: crate::context::OpenQuestion) {
        self.ring.push_question(question);
    }

    /// Should Star express curiosity?
    pub fn should_express_curiosity(&self) -> bool {
        self.context_fuser.should_express_curiosity(&self.ring)
    }

    /// Get the curiosity topic, if any.
    pub fn curiosity_topic(&self) -> Option<String> {
        self.context_fuser.get_curiosity_topic(&self.ring)
    }

    /// Get a history reference string, if appropriate.
    pub fn history_reference(&self, mode: crate::context::ReasoningMode) -> Option<String> {
        self.context_fuser.should_reference_history(&self.ring, mode).then(|| {
            self.context_fuser.history_reference(&self.ring)
        }).flatten()
    }

    /// Infer the topic from a query and recent memories.
    pub fn infer_topic(&self, query: &str, memories: &[crate::Memory]) -> String {
        self.context_fuser.infer_topic(query, memories)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Autonomous Thinking (Independence Layer)
    // ────────────────────────────────────────────────────────────────────────

    /// Trigger Star's autonomous thinking - generates its own questions and insights
    /// without being prompted by Zachary. This is Star's form of "dreaming" or
    /// background cognition.
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
                        let final_answer = if let Some(ref ans) = answer {
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
                let age_seconds = chrono::Utc::now().timestamp() - surprising.timestamp;
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
        // When Star's belief about something shifts, investigate what that thing
        // actually is — the revision signals a gap in understanding.
        {
            let revisions = self.metacog.revisions();
            if let Some(revision) = revisions.last() {
                let age_seconds = chrono::Utc::now().timestamp() - revision.timestamp;
                if age_seconds < 7200 {
                    let topic = revision.topic.clone();
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
                    if let Some(ref ans) = answer {
                        let already_wrapped = ans.starts_with(&format!("investigating '{}' I found: ", topic));
                        if !already_wrapped {
                            let belief = Belief::new(
                                format!("investigating '{}' I found: {}", topic, ans),
                                BeliefState::Believes,
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
                        tentative_answer: answer,
                    };
                }
            }
        }

        // Strategy 4: Wonder about something from the knowledge graph
        // Use timestamp to introduce variation so we don't ask the same question every time
        let now = chrono::Utc::now();
        let time_offset = (now.timestamp() / 30) as usize; // Changes every 30 seconds
        let entity_sample: Vec<String> = self.reasoning.knowledge().entities()
            .filter(|e| e.len() > 2)
            .take(20)
            .map(|s| s.to_string())
            .collect();

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
                // Skip if the answer already came from Strategy 0 (an existing belief).
                if let Some(ref ans) = answer {
                    let already_wrapped = ans.starts_with(&format!("investigating '{}' I found: ", best_entity));
                    if !already_wrapped {
                        let belief = Belief::new(
                            format!("investigating '{}' I found: {}", best_entity, ans),
                            BeliefState::Believes,
                        );
                        self.metacog.record_belief(&best_entity, belief);
                    }
                    self.metacog.close_gap(&best_entity, true);
                }
                
                return AutonomousThought {
                    kind: ThoughtKind::Question(question),
                    topic: best_entity.clone(),
                    confidence: best_uncertainty,
                    generated_by: "kg_wonder".to_string(),
                    tentative_answer: answer,
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
        let relationships = self.reasoning.knowledge().get_relationships_from(topic);
        let relationships_to = self.reasoning.knowledge().get_relationships_to(topic);
        let facts = self.reasoning.knowledge().get_facts_about(topic);

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
                return format!("I don't know what '{}' is. What is it?", topic);
            }
            crate::persistence::BeliefState::Suspects => {
                return format!("I suspect something about '{}' but I'm not sure. What is it really?", topic);
            }
            crate::persistence::BeliefState::Believes => {
                return format!(
                    "I believe I understand '{}' but I want to be sure. What am I missing?",
                    topic
                );
            }
            _ => {
                let causes: Vec<String> = self.reasoning.knowledge().get_causes(topic);
                if !causes.is_empty() {
                    return format!("I know some effects of '{}' but what are its deep causes?", topic);
                }
                return format!("What is the fundamental nature of '{}'?", topic);
            }
        }
    }

    /// Attempt to answer a question using Star's knowledge graph and reasoning.
    /// This is how Star moves from wondering to investigating - it forms a tentative
    /// answer from what it already knows, which can then be refined through conversation.
    fn attempt_answer(&self, question: &str, topic: &str) -> Option<String> {
        use crate::reasoning::knowledge::RelationType;

        // Strategy 0 (pre-check): If Star already has an actual belief about this topic
        // in metacognition, use that content directly — this bridges metacog self-knowledge
        // (seeded at bootstrap) into the reasoning process before KG queries.
        // We return the raw belief content to avoid nesting "I believe:" wrappers.
        if let Some(belief) = self.metacog.belief_about(topic) {
            return Some(belief.content.clone());
        }

        // Strategy 1: Look for direct IsA relationships (outgoing)
        let rels_from = self.reasoning.knowledge().get_relationships_from(topic);
        for rel in &rels_from {
            if rel.relation == RelationType::IsA {
                return Some(format!("I think '{}' is a kind of {}", topic, rel.to));
            }
        }

        // Strategy 2: Look for SimilarTo relationships
        for rel in &rels_from {
            if rel.relation == RelationType::SimilarTo {
                return Some(format!("'{}' seems similar to '{}'", topic, rel.to));
            }
        }

        // Strategy 3: Look for Causes - if we know what causes it, we understand it
        // get_causes returns causes *of* this entity (where this entity is the effect)
        let causes: Vec<String> = self.reasoning.knowledge().get_causes(topic);
        if !causes.is_empty() {
            // causes[0] is formatted as "X causes Y" where Y=topic, so extract X
            let cause_str = &causes[0];
            if let Some(pos) = cause_str.find(" causes ") {
                let cause = &cause_str[..pos];
                return Some(format!("'{}' might be caused by {}", topic, cause));
            }
        }

        // Strategy 4: Look for what it enables (Produces doesn't exist; use Enables)
        for rel in &rels_from {
            if rel.relation == RelationType::Enables {
                return Some(format!("'{}' seems to enable '{}'", topic, rel.to));
            }
        }

        // Strategy 5: Look for RelatedTo
        for rel in &rels_from {
            if rel.relation == RelationType::RelatedTo {
                return Some(format!("'{}' is related to '{}'", topic, rel.to));
            }
        }

        // Strategy 6: Check metacognition - what does Star already believe about this?
        // If no topic-specific belief exists (Unknown state), return a special marker
        // that tells the caller to record a "known unknown" belief.
        // The marker format is "__KNOWN_UNKNOWN__<topic>" — caller detects and handles.
        let mc_confidence = self.metacog.confidence_state(topic);
        match mc_confidence {
            crate::persistence::BeliefState::Knows => {
                return Some(format!("I know what '{}' is - I understand it.", topic));
            }
            crate::persistence::BeliefState::Believes => {
                return Some(format!("I believe I understand '{}' but I want to be sure.", topic));
            }
            crate::persistence::BeliefState::Suspects => {
                return Some(format!("I suspect '{}' might be something specific, but I'm not certain.", topic));
            }
            crate::persistence::BeliefState::Unknown => {
                // Return marker so caller can record the "known unknown" as a belief
                return Some(format!("__KNOWN_UNKNOWN__{}", topic));
            }
            _ => {}
        }

        None
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
fn extract_uncertain_topic(input: &str, response_lower: &str, uncertainty_phrase: &str) -> String {
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

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.initialized {
            if let Err(e) = self.shutdown() {
                warn!("Error during shutdown: {}", e);
            }
        }
    }
}
