//! A concept as a software object with lifecycle

use serde::{Deserialize, Serialize};
use super::lifecycle::LifecycleStage;

/// A concept with full lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: ConceptId,
    pub name: String,
    pub definition: String,
    pub stage: LifecycleStage,
    /// When created
    pub born_at: i64,
    /// Last time this concept was used
    pub last_used: i64,
    /// Usage count
    pub usage_count: usize,
    /// Pain events associated with this concept
    pub pain_count: usize,
    /// Contradictions involving this concept
    pub contradiction_count: usize,
    /// Child concepts (derived from this)
    pub children: Vec<ConceptId>,
    /// Parent concept (this was derived from)
    pub parent: Option<ConceptId>,
    /// Confidence in this concept's correctness
    pub confidence: f64,
    /// Related entities in the knowledge graph
    pub related_entities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(u64);

impl ConceptId {
    pub fn new() -> Self {
        ConceptId(rand_id())
    }
}

impl Default for ConceptId {
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

impl Concept {
    pub fn new(name: &str, definition: &str) -> Self {
        Self {
            id: ConceptId::new(),
            name: name.to_string(),
            definition: definition.to_string(),
            stage: LifecycleStage::Birth,
            born_at: crate::now_timestamp(),
            last_used: crate::now_timestamp(),
            usage_count: 0,
            pain_count: 0,
            contradiction_count: 0,
            children: Vec::new(),
            parent: None,
            confidence: 0.5,
            related_entities: Vec::new(),
        }
    }

    /// Record usage of this concept
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = crate::now_timestamp();
    }

    /// Record a pain event involving this concept
    pub fn record_pain(&mut self) {
        self.pain_count += 1;
        self.update_stage();
    }

    /// Record a contradiction involving this concept
    pub fn record_contradiction(&mut self) {
        self.contradiction_count += 1;
        self.update_stage();
    }

    /// Add a child concept
    pub fn add_child(&mut self, child_id: ConceptId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Update the lifecycle stage based on usage patterns
    pub fn update_stage(&mut self) {
        self.stage = LifecycleStage::from_metrics(
            self.usage_count,
            self.pain_count,
            self.contradiction_count,
            self.last_used,
        );
    }

    /// Should we use this concept with caution?
    pub fn use_with_caution(&self) -> bool {
        matches!(self.stage, LifecycleStage::Adolescence | LifecycleStage::Senescence)
    }

    /// Is this concept ready for retirement?
    pub fn should_retire(&self) -> bool {
        matches!(self.stage, LifecycleStage::Senescence) && self.usage_count < 3
    }

    /// Get a human-readable status description
    pub fn status(&self) -> String {
        format!(
            "{} (stage: {:?}, used {} times, pain: {}, confidence: {:.2})",
            self.name, self.stage, self.usage_count, self.pain_count, self.confidence
        )
    }
}