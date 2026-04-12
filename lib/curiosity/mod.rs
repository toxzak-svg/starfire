//! Curiosity Engine — Autonomous Exploration and Discovery
//!
//! Starfire's curiosity engine drives genuine exploration between messages.
//! It generates questions, discovers connections, and expresses 
//! autonomous interest in the world.

pub mod probes;
pub mod connection;

use crate::Store;
use crate::reasoning::ReasoningEngine;
pub use probes::{CuriosityProbe, CuriosityDepth, ProbeStatus};
use connection::ConnectionFinder;
use std::sync::{Arc, Mutex};

/// Curiosity engine — generates and runs autonomous curiosity probes.
pub struct CuriousEngine {
    /// Persistent store
    store: Arc<Store>,
    /// Reasoning engine for exploration
    reasoning: Arc<Mutex<ReasoningEngine>>,
    /// Connection finder for analogical exploration
    connection_finder: ConnectionFinder,
    /// Active probes being explored
    active_probes: Vec<CuriosityProbe>,
    /// Probes that have been answered
    completed_probes: Vec<CuriosityProbe>,
    /// Probes abandoned (couldn't resolve)
    abandoned_probes: Vec<CuriosityProbe>,
    /// Time of last probe (for idle timing)
    last_probe_time: Option<i64>,
    /// Minimum idle seconds between probes
    idle_min_seconds: i64,
}

impl CuriousEngine {
    /// Create a new curiosity engine.
    pub fn new(store: Arc<Store>, reasoning: Arc<Mutex<ReasoningEngine>>) -> Self {
        Self {
            store,
            reasoning,
            connection_finder: ConnectionFinder::new(),
            active_probes: Vec::new(),
            completed_probes: Vec::new(),
            abandoned_probes: Vec::new(),
            last_probe_time: None,
            idle_min_seconds: 30, // At least 30 seconds between probes
        }
    }

    /// Note that activity occurred (reset idle timer).
    pub fn note_activity(&mut self) {
        self.last_probe_time = None;
    }

    /// Check if a new probe should be fired (based on idle time).
    pub fn should_fire(&self) -> bool {
        if self.active_probes.len() >= 2 {
            return false; // Already exploring
        }
        
        if let Some(last) = self.last_probe_time {
            let now = crate::now_timestamp();
            (now - last) >= self.idle_min_seconds
        } else {
            true // Never fired or been reset
        }
    }

    /// Generate a new curiosity probe.
    pub fn generate_probe(&mut self) -> Option<CuriosityProbe> {
        // Pick a probe generation strategy based on state
        let strategy = if self.completed_probes.len() < 3 {
            // Early stage: generate many probes
            ProbeStrategy::RandomExploration
        } else if self.completed_probes.len() < 10 {
            // Medium stage: mix of exploration and deepening
            ProbeStrategy::DeepenOrExplore
        } else {
            // Mature stage: favor deep exploration of existing questions
            ProbeStrategy::DeepExploration
        };

        let probe = match strategy {
            ProbeStrategy::RandomExploration => self.generate_random_probe(),
            ProbeStrategy::DeepenOrExplore => {
                if rand_one_in(2) {
                    self.generate_deepening_probe()
                } else {
                    self.generate_random_probe()
                }
            }
            ProbeStrategy::DeepExploration => {
                self.generate_deepening_probe()
            }
        };

        if probe.is_some() {
            self.last_probe_time = Some(crate::now_timestamp());
        }

        probe
    }

    /// Generate a random exploration probe.
    fn generate_random_probe(&self) -> Option<CuriosityProbe> {
        // Pick from knowledge gaps or random concept pairs
        let question = self.connection_finder.generate_question();
        
        Some(CuriosityProbe {
            id: uuid_simple(),
            question,
            topic: "general".to_string(),
            why_interested: "I want to understand how things connect".to_string(),
            related_concepts: Vec::new(),
            depth: CuriosityDepth::Medium,
            status: ProbeStatus::Probing,
            tentative_answer: None,
            confidence: crate::persistence::BeliefState::Suspects,
            discovered_at: crate::now_timestamp(),
        })
    }

    /// Generate a deepening probe — extend an existing question.
    fn generate_deepening_probe(&self) -> Option<CuriosityProbe> {
        // Get most recent completed probe
        let recent = self.completed_probes.last()?;
        
        let deepening = self.connection_finder.deepen_question(&recent.question)?;
        
        Some(CuriosityProbe {
            id: uuid_simple(),
            question: deepening,
            topic: recent.topic.clone(),
            why_interested: "I want to understand this more deeply".to_string(),
            related_concepts: recent.related_concepts.clone(),
            depth: CuriosityDepth::Deep,
            status: ProbeStatus::Probing,
            tentative_answer: None,
            confidence: crate::persistence::BeliefState::Suspects,
            discovered_at: crate::now_timestamp(),
        })
    }

    /// Run a probe through the reasoning engine.
    pub fn run_probe(&mut self, probe: &CuriosityProbe) -> Option<String> {
        let mut reasoning = self.reasoning.lock().ok()?;
        
        // Try to reason about the question
        let result = reasoning.reason(&probe.question, &[]);
        
        if result.answer.is_some() {
            result.answer
        } else if !result.reasoning_chain.is_empty() {
            Some(result.reasoning_chain.join(" → "))
        } else {
            None
        }
    }

    /// Record that a probe has been answered.
    pub fn record_answer(&mut self, probe_id: &str, answer: &str) {
        if let Some(probe) = self.active_probes.iter_mut().find(|p| p.id == probe_id) {
            probe.status = ProbeStatus::Answered;
            probe.tentative_answer = Some(answer.to_string());
            if let Some(idx) = self.active_probes.iter().position(|p| p.id == probe_id) {
                let _ = self.active_probes.remove(idx);
            }
        }
    }

    /// Record that a probe was abandoned.
    pub fn abandon_probe(&mut self, probe_id: &str) {
        if let Some(pos) = self.active_probes.iter().position(|p| p.id == probe_id) {
            let mut probe = self.active_probes.remove(pos);
            probe.status = ProbeStatus::Abandoned;
            self.abandoned_probes.push(probe);
        }
    }

    /// Get the most recent interesting completed probe.
    pub fn recent_insight(&self) -> Option<&CuriosityProbe> {
        self.completed_probes.last()
    }

    /// Get all curiosity topics Starfire is actively exploring.
    pub fn active_topics(&self) -> Vec<&str> {
        self.active_probes
            .iter()
            .map(|p| p.topic.as_str())
            .collect()
    }

    /// Get curiosity statistics.
    pub fn stats(&self) -> CuriosityStats {
        CuriosityStats {
            active_probes: self.active_probes.len(),
            completed_probes: self.completed_probes.len(),
            abandoned_probes: self.abandoned_probes.len(),
            idle_seconds: self.last_probe_time
                .map(|t| crate::now_timestamp() - t)
                .unwrap_or(9999),
        }
    }

    /// Load active curiosity probes from the database (cross-session persistence).
    pub fn load_persisted_probes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::persistence::AutonomyState;
        
        let probes = self.store.get_active_curiosity_probes()?;
        
        for probe_state in &probes {
            // Convert AutonomyState back to CuriosityProbe
            let probe = CuriosityProbe {
                id: format!("persisted_{}", probe_state.id),
                question: format!("What is '{}'?", probe_state.content),
                topic: probe_state.content.clone(),
                why_interested: "I was curious about this in a previous session".to_string(),
                related_concepts: Vec::new(),
                depth: CuriosityDepth::Medium,
                status: ProbeStatus::Probing,
                tentative_answer: None,
                confidence: crate::persistence::BeliefState::Suspects,
                discovered_at: probe_state.created_at,
            };
            self.active_probes.push(probe);
        }
        
        if !probes.is_empty() {
            tracing::info!("Loaded {} persisted curiosity probes", probes.len());
        }
        
        Ok(())
    }

    /// Persist active curiosity probe to database.
    pub fn persist_probe(&self, probe: &CuriosityProbe) -> Result<i64, Box<dyn std::error::Error>> {
        let id = self.store.save_autonomy_state(
            crate::persistence::Store::AUTONOMY_CURIOSITY,
            &probe.topic,
            0.6,
            None,
        )?;
        Ok(id)
    }
}

/// Statistics about the curiosity engine.
#[derive(Debug, Clone)]
pub struct CuriosityStats {
    pub active_probes: usize,
    pub completed_probes: usize,
    pub abandoned_probes: usize,
    pub idle_seconds: i64,
}

/// Probe generation strategies.
#[derive(Debug, Clone, Copy)]
enum ProbeStrategy {
    /// Just explore randomly
    RandomExploration,
    /// Either deepen existing or explore new
    DeepenOrExplore,
    /// Focus on deepening existing understanding
    DeepExploration,
}

impl Clone for CuriousEngine {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            reasoning: Arc::clone(&self.reasoning),
            connection_finder: ConnectionFinder::new(),
            active_probes: Vec::new(), // Reset on clone
            completed_probes: self.completed_probes.clone(),
            abandoned_probes: self.abandoned_probes.clone(),
            last_probe_time: None,
            idle_min_seconds: self.idle_min_seconds,
        }
    }
}

/// Simple random helper.
fn rand_one_in(n: u32) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    nanos.is_multiple_of(n)
}

/// Generate a simple UUID (not cryptographically secure, just for IDs).
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
