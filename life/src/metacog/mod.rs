//! Meta-Cognition Layer (Layer 3)
//!
//! Thinks about thinking. Monitors confidence. Detects gaps.
//!
//! Phase 1: Confidence tracking only. Full metacognition in Phase 3.

use crate::persistence::memory::{Belief, BeliefState};

/// Meta-cognitive monitor.
/// 
/// Tracks what Star knows, what it doesn't know, and what it's uncertain about.
/// Provides the "inner voice" that questions its own conclusions.
pub struct MetaCognition {
    /// Current active beliefs being considered
    active_beliefs: Vec<Belief>,
    /// Knowledge gaps identified
    gaps: Vec<KnowledgeGap>,
    /// Reasoning chains being monitored
    monitored_chains: Vec<MonitoredChain>,
}

impl MetaCognition {
    pub fn new() -> Self {
        Self {
            active_beliefs: Vec::new(),
            gaps: Vec::new(),
            monitored_chains: Vec::new(),
        }
    }

    /// Add a belief to active consideration.
    pub fn consider(&mut self, belief: Belief) {
        self.active_beliefs.push(belief);
    }

    /// Identify a gap in knowledge.
    pub fn note_gap(&mut self, gap: KnowledgeGap) {
        self.gaps.push(gap);
    }

    /// Get all current gaps, sorted by importance.
    pub fn gaps(&self) -> &[KnowledgeGap] {
        &self.gaps
    }

    /// Get the current confidence state about a topic.
    pub fn confidence_about(&self, topic: &str) -> BeliefState {
        for belief in &self.active_beliefs {
            if belief.content.to_lowercase().contains(&topic.to_lowercase()) {
                return belief.confidence_state;
            }
        }
        BeliefState::Unknown
    }

    /// Express uncertainty appropriately.
    pub fn express_uncertainty(&self, topic: &str) -> String {
        let state = self.confidence_about(topic);
        match state {
            BeliefState::Knows => format!("I know about {}", topic),
            BeliefState::Thinks => format!("I think I know about {} but I'm not certain", topic),
            BeliefState::Believes => format!("I believe I know something about {} but I'm not sure", topic),
            BeliefState::Suspects => format!("I'm not sure about {}. Let me think...", topic),
            BeliefState::Unknown => format!("I don't know anything about {} yet.", topic),
        }
    }

    /// Monitor a reasoning chain for quality.
    pub fn monitor_chain(&mut self, chain: MonitoredChain) {
        // Check for signs of weak reasoning
        let warnings = self.check_chain_quality(&chain);
        if !warnings.is_empty() {
            // Attach warnings to the chain
            let mut chain = chain;
            chain.warnings = warnings;
            self.monitored_chains.push(chain);
        }
    }

    fn check_chain_quality(&self, chain: &MonitoredChain) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if chain.steps.len() > 5 {
            warnings.push("Long chain — risk of error accumulation".to_string());
        }
        
        if chain.assumptions > 2 {
            warnings.push("Many assumptions — confidence should be lower".to_string());
        }
        
        warnings
    }

    /// Should Star seek more information about this topic?
    pub fn should_investigate(&self, topic: &str) -> bool {
        // Star should investigate if:
        // - It has low confidence about the topic
        // - The topic has been accessed frequently but never resolved
        // - There are active contradictions
        
        let state = self.confidence_about(topic);
        matches!(state, BeliefState::Suspects | BeliefState::Unknown)
            || self.gaps.iter().any(|g| g.topic.to_lowercase() == topic.to_lowercase())
    }
}

impl Default for MetaCognition {
    fn default() -> Self {
        Self::new()
    }
}

/// A gap in Star's knowledge.
#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    /// What Star doesn't know
    pub topic: String,
    /// Why it matters
    pub importance: f64,
    /// When it was noticed
    pub noticed_at: i64,
    /// Whether Star has tried to fill this gap
    pub investigated: bool,
}

impl KnowledgeGap {
    pub fn new(topic: impl Into<String>, importance: f64) -> Self {
        Self {
            topic: topic.into(),
            importance,
            noticed_at: chrono::Utc::now().timestamp(),
            investigated: false,
        }
    }

    pub fn mark_investigated(&mut self) {
        self.investigated = true;
    }
}

/// A reasoning chain being monitored for quality.
#[derive(Debug)]
pub struct MonitoredChain {
    /// The steps in the chain
    pub steps: Vec<String>,
    /// How many explicit assumptions were made
    pub assumptions: usize,
    /// Any warnings about this chain
    pub warnings: Vec<String>,
}

impl MonitoredChain {
    pub fn new(steps: Vec<String>, assumptions: usize) -> Self {
        Self {
            steps,
            assumptions,
            warnings: Vec::new(),
        }
    }
}
