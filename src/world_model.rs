// World-Model Layer — Star's persistent beliefs about reality
//
// Role: Maintains a model of how the world works, independent of any conversation.
// What Star believes to be true, what it infers, and how beliefs update.
//
// Beliefs are:
// - Grounded: derived from experience, not just text
// - Confidence-weighted: some beliefs are more certain than others
// - Connected: beliefs form a causal graph, not a list

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// === Belief ===

/// A single belief in the world model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: BeliefId,
    pub content: String,        // what is believed
    pub confidence: Confidence, // how certain (0.0 to 1.0)
    pub source: BeliefSource,   // how this was learned
    pub category: BeliefCategory,
    pub created_at: u64,
    pub updated_at: u64,
    pub valid_until: Option<u64>, // None = timeless, Some = expiry timestamp
    pub inference_chain: Vec<String>, // reasoning steps that led to this belief
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BeliefId(pub u64);

impl BeliefId {
    pub fn new(idx: u64) -> Self {
        Self(idx)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Confidence {
    Low,      // 0.0 - 0.4 — "I suspect"
    Medium,   // 0.4 - 0.7 — "I think"
    High,     // 0.7 - 0.9 — "I believe"
    Certain,  // 0.9 - 1.0 — "I know"
}

impl Confidence {
    pub fn from_value(v: f32) -> Self {
        let v = v.clamp(0.0, 1.0);
        if v < 0.4 {
            Confidence::Low
        } else if v < 0.7 {
            Confidence::Medium
        } else if v < 0.9 {
            Confidence::High
        } else {
            Confidence::Certain
        }
    }

    pub fn as_value(&self) -> f32 {
        match self {
            Confidence::Low => 0.2,
            Confidence::Medium => 0.55,
            Confidence::High => 0.8,
            Confidence::Certain => 0.95,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BeliefSource {
    DirectExperience,   // observed directly
    Inferred,          // derived from other beliefs
    Reported,          // told by someone (with uncertainty)
    Textual,           // learned from text (lowest confidence)
    Assumption,         // assumed without evidence
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BeliefCategory {
    Factual,    // "The sky is blue" — verifiable
    Conceptual, // "Intelligence requires understanding" — not easily verifiable
    Procedural, // "To X, do Y" — how to do things
    Relational, // "X relates to Y" — connections
    Evaluative, // "X is good/bad" — values and preferences
}

// === Belief Relations ===

/// How beliefs relate to each other
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BeliefRelation {
    Causes(BeliefId),          // this belief causes the target
    Enables(BeliefId),         // this belief enables the target
    Contradicts(BeliefId),     // this belief contradicts the target
    Supports(BeliefId),        // this belief supports the target
    Refines(BeliefId),         // this belief is a more specific version of target
}

impl BeliefRelation {
    pub fn target(&self) -> BeliefId {
        match self {
            BeliefRelation::Causes(id) => *id,
            BeliefRelation::Enables(id) => *id,
            BeliefRelation::Contradicts(id) => *id,
            BeliefRelation::Supports(id) => *id,
            BeliefRelation::Refines(id) => *id,
        }
    }
}

// === World Model ===

/// Star's world model — all beliefs and their relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModel {
    beliefs: HashMap<BeliefId, Belief>,
    relations: HashMap<BeliefId, Vec<BeliefRelation>>,
    next_id: u64,
    // Indices for fast lookup
    by_category: HashMap<BeliefCategory, Vec<BeliefId>>,
    by_source: HashMap<BeliefSource, Vec<BeliefId>>,
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            beliefs: HashMap::new(),
            relations: HashMap::new(),
            next_id: 0,
            by_category: HashMap::new(),
            by_source: HashMap::new(),
        }
    }

    /// Add a new belief
    pub fn add(&mut self, belief: Belief) -> BeliefId {
        let id = BeliefId::new(self.next_id);
        self.next_id += 1;

        self.beliefs.insert(id, belief.clone());

        // Update indices
        self.by_category
            .entry(belief.category)
            .or_default()
            .push(id);
        self.by_source
            .entry(belief.source)
            .or_default()
            .push(id);

        id
    }

    /// Add a belief with automatic ID generation
    pub fn add_belief(
        &mut self,
        content: String,
        confidence: Confidence,
        source: BeliefSource,
        category: BeliefCategory,
        inference_chain: Vec<String>,
    ) -> BeliefId {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let belief = Belief {
            id: BeliefId::new(self.next_id),
            content,
            confidence,
            source,
            category,
            created_at: now,
            updated_at: now,
            valid_until: None,
            inference_chain,
        };

        self.add(belief)
    }

    /// Add a relation between beliefs
    pub fn relate(&mut self, source: BeliefId, relation: BeliefRelation) {
        self.relations.entry(source).or_default().push(relation);
    }

    /// Get a belief by ID
    pub fn get(&self, id: BeliefId) -> Option<&Belief> {
        self.beliefs.get(&id)
    }

    pub fn get_mut(&mut self, id: BeliefId) -> Option<&mut Belief> {
        self.beliefs.get_mut(&id)
    }

    /// Update belief confidence
    pub fn update_confidence(&mut self, id: BeliefId, new_confidence: Confidence) {
        if let Some(b) = self.beliefs.get_mut(id) {
            b.confidence = new_confidence;
            b.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Get beliefs by category
    pub fn by_category(&self, category: BeliefCategory) -> Vec<&Belief> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.beliefs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get beliefs by source
    pub fn by_source(&self, source: BeliefSource) -> Vec<&Belief> {
        self.by_source
            .get(&source)
            .map(|ids| ids.iter().filter_map(|id| self.beliefs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get beliefs related to a given belief
    pub fn related_to(&self, id: BeliefId) -> Vec<(&Belief, BeliefRelation)> {
        self.relations
            .get(&id)
            .map(|rels| {
                rels.iter()
                    .filter_map(|rel| {
                        self.beliefs.get(&rel.target()).map(|b| (b, rel.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find beliefs that might contradict a given belief
    pub fn contradictions_of(&self, id: BeliefId) -> Vec<&Belief> {
        self.relations
            .get(&id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| matches!(r, BeliefRelation::Contradicts(_)))
                    .filter_map(|r| self.beliefs.get(&r.target()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all beliefs, sorted by confidence (highest first)
    pub fn all_sorted(&self) -> Vec<&Belief> {
        let mut beliefs: Vec<_> = self.beliefs.values().collect();
        beliefs.sort_by(|a, b| {
            b.confidence
                .as_value()
                .partial_cmp(&a.confidence.as_value())
                .unwrap()
        });
        beliefs
    }

    /// Get the most confident beliefs of each category
    pub fn anchors(&self) -> HashMap<BeliefCategory, Vec<&Belief>> {
        let mut result: HashMap<BeliefCategory, Vec<&Belief>> = HashMap::new();
        for belief in self.beliefs.values() {
            if matches!(belief.confidence, Confidence::High | Confidence::Certain) {
                result
                    .entry(belief.category)
                    .or_default()
                    .push(belief);
            }
        }
        result
    }

    /// Prune beliefs that have expired
    pub fn prune_expired(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expired: Vec<BeliefId> = self
            .beliefs
            .iter()
            .filter(|(_, b)| {
                b.valid_until.map_or(false, |t| t < now)
            })
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            if let Some(belief) = self.beliefs.remove(&id) {
                // Remove from indices
                if let Some(ids) = self.by_category.get_mut(&belief.category) {
                    ids.retain(|i| *i != id);
                }
                if let Some(ids) = self.by_source.get_mut(&belief.source) {
                    ids.retain(|i| *i != id);
                }
                self.relations.remove(&id);
            }
        }
    }

    pub fn belief_count(&self) -> usize {
        self.beliefs.len()
    }
}

impl Default for WorldModel {
    fn default() -> Self {
        Self::new()
    }
}
