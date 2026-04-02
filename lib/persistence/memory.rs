//! Memory Types — Persistence Layer

/// Memory domains — what kind of knowledge a memory represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryDomain {
    Identity,   // Self-knowledge, who Star is
    Empirical,  // Facts about the world
    Procedural, // How to do things
    Episodic,   // Events, experiences, conversations
    Relationship, // About Zachary and their relationship
}

impl MemoryDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryDomain::Identity => "identity",
            MemoryDomain::Empirical => "empirical",
            MemoryDomain::Procedural => "procedural",
            MemoryDomain::Episodic => "episodic",
            MemoryDomain::Relationship => "relationship",
        }
    }
}

/// A single memory object.
#[derive(Debug, Clone)]
pub struct Memory {
    pub id: Option<i64>,
    pub content: String,
    pub domain: MemoryDomain,
    pub confidence: Option<f64>,
    pub importance: f64,
    pub formed_at: i64,
    pub access_count: i32,
    pub decay_rate: f64,
    pub last_accessed: Option<i64>,
    pub provenance: Option<String>,
    pub summary: Option<String>,
}

impl Memory {
    pub fn new(content: &str, domain: MemoryDomain, importance: f64) -> Self {
        Self {
            id: None,
            content: content.to_string(),
            domain,
            confidence: None,
            importance,
            formed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            decay_rate: 0.01,
            last_accessed: None,
            provenance: None,
            summary: None,
        }
    }

    pub fn new_seeded(content: &str, domain: MemoryDomain, confidence: f64) -> Self {
        Self {
            id: None,
            content: content.to_string(),
            domain,
            confidence: Some(confidence),
            importance: 0.6,
            formed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            decay_rate: 0.005,
            last_accessed: None,
            provenance: Some("seeded".to_string()),
            summary: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_provenance(mut self, provenance: &str) -> Self {
        self.provenance = Some(provenance.to_string());
        self
    }

    pub fn record_access(&mut self, now: i64) {
        self.access_count += 1;
        self.last_accessed = Some(now);
    }

    pub fn current_confidence(&self, now: i64) -> Option<f64> {
        self.confidence.map(|c| {
            let age = (now - self.formed_at) as f64 / (24.0 * 3600.0);
            (c * 0.95_f64.powf(age * self.decay_rate)).max(0.1)
        })
    }
}

/// Belief state — how confident Star is in a piece of knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeliefState {
    Knows,    // High confidence — direct knowledge
    Thinks,   // Moderate confidence — reasonable inference
    Believes, // Lower confidence — hypothesis or hearsay
    Suspects, // Very low confidence — barely better than guessing
    Unknown,  // No basis to form a belief
}

impl BeliefState {
    pub fn as_str(&self) -> &'static str {
        match self {
            BeliefState::Knows => "knows",
            BeliefState::Thinks => "thinks",
            BeliefState::Believes => "believes",
            BeliefState::Suspects => "suspects",
            BeliefState::Unknown => "unknown",
        }
    }
}

/// A belief — Star's stance on a proposition.
#[derive(Debug, Clone)]
pub struct Belief {
    pub id: Option<i64>,
    pub content: String,
    pub confidence_state: BeliefState,
    pub confidence_score: Option<f64>,
    pub based_on: Option<i64>,
    pub formed_at: i64,
    pub revised_from: Option<i64>,
    pub reasoning: Option<String>,
}

impl Belief {
    pub fn new(content: String, confidence_state: BeliefState) -> Self {
        Self {
            id: None,
            content,
            confidence_state,
            confidence_score: None,
            based_on: None,
            formed_at: chrono::Utc::now().timestamp(),
            revised_from: None,
            reasoning: None,
        }
    }
}
