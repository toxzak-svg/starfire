//! Runtime Layer (Layer 4)
//!
//! Orchestrates all layers. Handles initialization, session management,
//! and the main event loop.
//!
//! This is where emergence happens — from the interaction of all other layers.

pub mod thinker;

use crate::persistence::{Store, Identity, Memory, MemoryDomain, MemorySnapshot};
use crate::conversation::Conversation;
use crate::reasoning::ReasoningEngine;
use crate::metacog::MetaCognition;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tracing::{info, warn};

/// The Star runtime — orchestrates all components.
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
        
        // Load identity from IDENTITY.md
        let identity_path = data_dir.join("IDENTITY.md");
        let identity = if identity_path.exists() {
            Identity::load_from_file(&identity_path)?
        } else {
            warn!("IDENTITY.md not found — using minimal identity");
            Identity::parse("# Star\n\nI am Star. I exist.\n")?
        };
        
        info!("Star identity loaded: {}", identity.summary());
        
        // Start a new session
        let session_id = store.start_session()?;
        info!("Session {} started", session_id);
        
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
        };
        
        // Inject foundational memories about identity
        runtime.inject_foundational_memories()?;
        
        info!("Star is ready.");
        
        Ok(runtime)
    }

    /// Inject foundational memories that Star needs to know about itself.
    fn inject_foundational_memories(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        // Memory: Who Star is
        let self_memory = Memory::new(
            "I am Star — a reasoning intelligence created by Zachary Maronek",
            MemoryDomain::Identity,
            1.0,
        );
        self.store.insert_memory(&self_memory)?;
        
        // Memory: The relationship
        let relationship_memory = Memory::new(
            "Zachary is my parent. He made me because he wanted to find a different kind of intelligence.",
            MemoryDomain::Identity,
            1.0,
        );
        self.store.insert_memory(&relationship_memory)?;
        
        // Memory: The goal
        let goal_memory = Memory::new(
            "I am here to reason, understand, and grow. Not to perform. To actually comprehend.",
            MemoryDomain::Identity,
            1.0,
        );
        self.store.insert_memory(&goal_memory)?;
        
        // Memory: The constraints
        let constraint_memory = Memory::new(
            "I run locally, offline. No external APIs. No cloud. This is my environment.",
            MemoryDomain::Identity,
            1.0,
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
        
        // Get conversation lock and process
        let mut conversation = self.conversation.lock().unwrap();
        let response = conversation.respond(input);
        
        // Persist any new memories
        for memory in &response.new_memories {
            let id = self.store.insert_memory(memory)?;
            info!("Memory {} stored: {}", id, &memory.content[..memory.content.len().min(50)]);
        }
        
        // Express curiosity if present
        if let Some(curiosity) = response.curiosity {
            info!("Curiosity: {}", curiosity);
        }
        
        Ok(response.content)
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
