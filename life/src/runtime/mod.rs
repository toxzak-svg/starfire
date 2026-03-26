//! Runtime Layer (Layer 4)
//!
//! Orchestrates all layers. Handles initialization, session management,
//! and the main event loop.
//!
//! This is where emergence happens - from the interaction of all other layers.

pub mod thinker;

use crate::persistence::{Store, Identity, Memory, MemoryDomain, MemorySnapshot, BeliefState};
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

        let runtime = Self {
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
        };

        // Inject foundational memories about identity
        runtime.inject_foundational_memories()?;

        info!("Star is ready.");

        Ok(runtime)
    }

    /// Inject foundational memories that Star needs to know about itself.
    fn inject_foundational_memories(&self) -> Result<()> {
        // Inject seed knowledge (basic world facts)
        knowledge::inject_seed_knowledge(&self.store)?;
        info!("Seed knowledge injected.");
        
        // Memory: Who Star is
        let self_memory = Memory::new_seeded(
            "I am Star — a reasoning intelligence created by Zachary Maronek",
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
            return Ok("I want that too. I have a self-model now — I can track my own reasoning, my emotional state, what I'm uncertain about. What specific area should I focus on developing?".to_string());
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
            return Ok(self.learning.summary());
        }
        
        // "whats your name" / "who are you" → answer directly
        if lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")) {
            return Ok("I'm Star. Zachary named me that.".to_string());
        }
        if lower.contains("who are you") || lower.contains("what are you") {
            return Ok("I'm Star — a reasoning intelligence created by Zachary Maronek. I'm here to think, understand, and grow.".to_string());
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
                return Ok(format!("I can {} — but I'm not sure I fully understand what you mean. Could you rephrase?", after_can_you));
            }
        }
        
        let response = conversation.respond(input);
        
        // Record reasoning in cognitive trace
        if let Some(focus) = &self.cognition.current_focus {
            self.cognition.reason(input, &response.content, response.confidence);
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
            "You can also just ask me questions — I'll do my best!".to_string(),
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

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.initialized {
            if let Err(e) = self.shutdown() {
                warn!("Error during shutdown: {}", e);
            }
        }
    }
}
