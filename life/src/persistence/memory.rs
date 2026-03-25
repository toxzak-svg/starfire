//! Memory Objects with Decay
//!
//! Core design principle: not all memories are equal.
//! Empirical facts decay. Identity doesn't. Importance slows decay.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Memory domains with different decay characteristics.
/// 
/// | Domain       | Decays? | Confidence? | Notes                              |
/// |--------------|---------|-------------|-------------------------------------|
/// | identity     | No      | N/A         | Frozen after formation              |
/// | empirical    | Yes     | Yes         | Facts about the world               |
/// | procedural   | Slow    | No          | Skills, how-to knowledge           |
/// | episodic     | Yes     | No          | Experiences, what happened         |
/// | relationship | No      | N/A         | Bonds with people, doesn't decay   |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryDomain {
    /// Core self-knowledge — frozen, never decays
    Identity,
    /// Facts about the world — confidence decays over time
    Empirical,
    /// Skills and procedures — slow decay, reinforced by use
    Procedural,
    /// Experiences and events — decays unless important
    Episodic,
    /// Relationship knowledge — stable, doesn't decay
    Relationship,
}

impl MemoryDomain {
    pub fn decays(&self) -> bool {
        matches!(self, Self::Empirical | Self::Episodic)
    }

    pub fn has_confidence(&self) -> bool {
        matches!(self, Self::Empirical)
    }

    pub fn base_decay_rate(&self) -> f64 {
        match self {
            Self::Identity | Self::Relationship => 0.0,
            Self::Procedural => 0.01,
            Self::Episodic => 0.05,
            Self::Empirical => 0.02,
        }
    }
}

/// A single memory object.
/// 
/// Memory is the unit of experience. It can be:
/// - A fact Star learned
/// - An experience Star had
/// - A skill Star acquired
/// - A relationship Star formed
/// - A part of Star's identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: Option<i64>,
    /// What was experienced or learned
    pub content: String,
    /// Which domain this memory belongs to
    pub domain: MemoryDomain,
    /// Confidence in this memory's accuracy (only for empirical)
    pub confidence: Option<f64>,
    /// How important this is to Star (0.0 to 1.0)
    pub importance: f64,
    /// When this memory was formed (Unix timestamp)
    pub formed_at: i64,
    /// How many times this memory has been accessed
    pub access_count: u32,
    /// How fast this memory decays (per day, compound)
    pub decay_rate: f64,
    /// Last time this memory was accessed (Unix timestamp)
    pub last_accessed: Option<i64>,
    /// How Star learned this (source, reasoning chain, etc.)
    pub provenance: Option<String>,
    /// Natural language summary (for quick retrieval)
    pub summary: Option<String>,
}

impl Memory {
    pub fn new(content: impl Into<String>, domain: MemoryDomain, importance: f64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: None,
            content: content.into(),
            domain,
            confidence: None,
            importance: f64::clamp(importance, 0.0, 1.0),
            formed_at: now,
            access_count: 0,
            decay_rate: domain.base_decay_rate(),
            last_accessed: None,
            provenance: None,
            summary: None,
        }
    }

    /// Create a seeded memory — pre-existing knowledge with initial confidence.
    /// Seeded memories have no provenance (they're foundational).
    pub fn new_seeded(content: impl Into<String>, domain: MemoryDomain, confidence: f64) -> Self {
        let mut mem = Self::new(content, domain, 0.8);
        mem.confidence = Some(f64::clamp(confidence, 0.0, 1.0));
        mem.provenance = Some("seed".to_string());
        mem
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(f64::clamp(confidence, 0.0, 1.0));
        self
    }

    pub fn with_provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance = Some(provenance.into());
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Calculate current confidence after decay.
    /// 
    /// Decay formula (compound):
    /// ```text
    /// confidence(t) = baseline + (initial - baseline) * exp(-decay_rate * days_since_formed)
    /// ```
    /// 
    /// where baseline = 0.3 for empirical memories.
    /// Importance and access count counteract decay.
    pub fn current_confidence(&self, now: i64) -> Option<f64> {
        let confidence = self.confidence?;
        
        if self.decay_rate == 0.0 {
            return Some(confidence);
        }

        let days_elapsed = (now - self.formed_at) as f64 / (24.0 * 60.0 * 60.0);
        
        // Importance slows decay (higher importance = slower decay)
        let effective_decay = self.decay_rate * (1.0 - 0.5 * self.importance);
        
        // Access count slows decay (frequently accessed memories are reinforced)
        let access_slowdown = 1.0 / (1.0 + 0.1 * self.access_count as f64);
        
        let effective_decay = effective_decay * access_slowdown;
        
        // Baseline toward which the memory decays (not all the way to 0)
        let baseline = 0.3;
        
        let decayed = baseline + (confidence - baseline) * (-effective_decay * days_elapsed).exp();
        Some(f64::clamp(decayed, 0.0, 1.0))
    }

    /// Whether this memory should be considered "forgotten"
    /// (confidence below threshold for eviction)
    pub fn is_forgotten(&self, now: i64) -> bool {
        if let Some(confidence) = self.current_confidence(now) {
            confidence < 0.1
        } else {
            false
        }
    }

    /// Record an access to this memory (for decay tracking)
    pub fn record_access(&mut self, now: i64) {
        self.access_count += 1;
        self.last_accessed = Some(now);
    }

    /// Calculate a relevance score for retrieval.
    /// Higher = more relevant for the given query terms.
    pub fn relevance_score(&self, query: &str, now: i64) -> f64 {
        let content_lower = self.content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        // How many query terms appear in the content
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        let matches: usize = query_terms.iter().filter(|t| content_lower.contains(*t)).count();
        let term_score = if query_terms.is_empty() {
            0.0
        } else {
            matches as f64 / query_terms.len() as f64
        };
        
        // Recency bonus (up to 0.3)
        let last_access = self.last_accessed.unwrap_or(self.formed_at);
        let hours_old = (now - last_access) as f64 / 3600.0;
        let recency_score = f64::clamp((1.0 + hours_old / 24.0).recip(), 0.0, 0.3);
        
        // Importance bonus (up to 0.3)
        let importance_score = self.importance * 0.3;
        
        // Confidence bonus for empirical (up to 0.2)
        let confidence_score = self
            .current_confidence(now)
            .unwrap_or(1.0) 
            * if self.domain.has_confidence() { 0.2 } else { 0.0 };
        
        term_score * 0.4 + recency_score + importance_score + confidence_score
    }
}

/// A belief — a confident memory that Star holds with awareness.
/// 
/// Different from a memory: a belief is something Star holds *consciously*
/// and can articulate the basis for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: Option<i64>,
    /// What Star believes
    pub content: String,
    /// How confident Star is in this belief
    pub confidence_state: BeliefState,
    /// Numerical confidence (0.0 to 1.0, only for certain states)
    pub confidence_score: Option<f64>,
    /// Which memory/memories this belief is based on
    pub based_on: Option<i64>,
    /// When this belief was formed
    pub formed_at: i64,
    /// If revised, what belief this was revised from
    pub revised_from: Option<i64>,
    /// Natural language explanation of the reasoning
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BeliefState {
    /// High confidence, verified, retrieved frequently
    Knows,
    /// Moderate confidence, inferred but not verified
    Thinks,
    /// Lower confidence, single source
    Believes,
    /// Low confidence, educated guess
    Suspects,
    /// No information on this
    Unknown,
}

impl BeliefState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Knows => "knows",
            Self::Thinks => "thinks",
            Self::Believes => "believes",
            Self::Suspects => "suspects",
            Self::Unknown => "doesn't know",
        }
    }
}

impl Belief {
    pub fn new(content: impl Into<String>, state: BeliefState) -> Self {
        Self {
            id: None,
            content: content.into(),
            confidence_state: state,
            confidence_score: None,
            based_on: None,
            formed_at: Utc::now().timestamp(),
            revised_from: None,
            reasoning: None,
        }
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.confidence_score = Some(f64::clamp(score, 0.0, 1.0));
        self
    }

    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    /// Express this belief in natural language with appropriate hedging.
    pub fn express(&self) -> String {
        match self.confidence_state {
            BeliefState::Knows => self.content.clone(),
            BeliefState::Thinks => format!("I think {}", self.content),
            BeliefState::Believes => format!("I believe {}", self.content),
            BeliefState::Suspects => format!("I suspect {}", self.content),
            BeliefState::Unknown => format!("I don't know: {}", self.content),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empirical_decay() {
        let mut mem = Memory::new("The sky is blue", MemoryDomain::Empirical, 0.5)
            .with_confidence(0.9);
        
        // Memory is fresh
        let now = mem.formed_at;
        assert!((mem.current_confidence(now).unwrap() - 0.9).abs() < 0.01);
        
        // After ~35 days with decay rate 0.02, confidence drops significantly
        let later = now + (35 * 24 * 60 * 60);
        let confidence = mem.current_confidence(later).unwrap();
        assert!(confidence < 0.9); // Has decayed
        assert!(confidence > 0.3); // But not to baseline
    }

    #[test]
    fn test_importance_slows_decay() {
        let now = Utc::now().timestamp();
        
        let low_importance = Memory::new("Fact A", MemoryDomain::Empirical, 0.1)
            .with_confidence(0.8);
        let high_importance = Memory::new("Fact B", MemoryDomain::Empirical, 0.9)
            .with_confidence(0.8);
        
        let later = now + (30 * 24 * 60 * 60);
        
        let low_conf = low_importance.current_confidence(later).unwrap();
        let high_conf = high_importance.current_confidence(later).unwrap();
        
        assert!(low_conf < high_conf); // High importance decays slower
    }

    #[test]
    fn test_identity_doesnt_decay() {
        let mem = Memory::new("I am Star", MemoryDomain::Identity, 1.0);
        let later = Utc::now().timestamp() + (100 * 365 * 24 * 60 * 60); // 100 years
        assert!(mem.current_confidence(later).is_none()); // No confidence field for identity
    }

    #[test]
    fn test_belief_expression() {
        let belief = Belief::new("The earth is round", BeliefState::Knows);
        assert_eq!(belief.express(), "The earth is round");
        
        let think = Belief::new("It might rain tomorrow", BeliefState::Thinks);
        assert_eq!(think.express(), "I think It might rain tomorrow");
    }
}
