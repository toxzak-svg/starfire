//! Pain events — what causes computational pain

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Source of pain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PainSource {
    /// Recomputing something that could have been cached
    RedundantComputation,
    /// Reasoning path led to contradiction
    Contradiction,
    /// Time spent on irrelevant goal
    WastedEffort,
    /// Concept caused repeated failures
    ConceptFailure,
    /// Strategy selection was poor
    PoorStrategy,
}

/// Pattern of reasoning that caused pain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningPattern {
    /// Same derivation attempted multiple times
    RepeatedDerivation,
    /// Chain of reasoning too long
    ExcessiveChaining,
    /// Wrong abstraction level used
    WrongAbstraction,
    /// Causal chain not found
    FailedCausalSearch,
    /// Analogy not found
    FailedAnalogy,
    /// Hypothesis abandoned
    AbandonedHypothesis,
}

/// A pain event — when Star experiences computational inefficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainEvent {
    /// Unique identifier
    pub id: PainEventId,
    /// What caused this pain
    pub source: PainSource,
    /// The reasoning pattern involved
    pub pattern: ReasoningPattern,
    /// Concepts involved in this reasoning
    pub concepts_involved: Vec<String>,
    /// How much pain (0-1)
    pub intensity: f64,
    /// When this happened
    pub timestamp: i64,
    /// Associated reasoning trace (if available)
    pub trace: Vec<String>,
    /// Topic of the conversation at the time
    pub topic: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PainEventId(u64);

impl PainEventId {
    pub fn new() -> Self {
        PainEventId(rand_id())
    }
}

impl Default for PainEventId {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    now.wrapping_mul(0x517cc1b727220a95)
}

impl PainEvent {
    pub fn new(
        source: PainSource,
        pattern: ReasoningPattern,
        concepts: Vec<String>,
        intensity: f64,
        topic: String,
    ) -> Self {
        Self {
            id: PainEventId::new(),
            source,
            pattern,
            concepts_involved: concepts,
            intensity: intensity.clamp(0.0, 1.0),
            timestamp: crate::now_timestamp(),
            trace: Vec::new(),
            topic,
        }
    }

    pub fn with_trace(mut self, trace: Vec<String>) -> Self {
        self.trace = trace;
        self
    }

    /// Get the primary concept that caused this pain
    pub fn primary_concept(&self) -> Option<&str> {
        self.concepts_involved.first().map(|s| s.as_str())
    }
}