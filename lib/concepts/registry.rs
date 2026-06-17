//! Concept registry — manages all concepts

use std::collections::HashMap;
use super::concept::Concept;
use super::concept::ConceptId;
use super::lifecycle::{LifecycleStage, LifecycleEvent};

/// Registry of all concepts
#[derive(Debug, Clone)]
pub struct ConceptRegistry {
    concepts: HashMap<ConceptId, Concept>,
    by_name: HashMap<String, ConceptId>,
    lifecycle_history: Vec<LifecycleEvent>,
    max_concepts: usize,
}

impl ConceptRegistry {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            by_name: HashMap::new(),
            lifecycle_history: Vec::new(),
            max_concepts: 1000,
        }
    }

    /// Register a new concept
    pub fn register(&mut self, name: &str, definition: &str) -> ConceptId {
        if let Some(existing_id) = self.by_name.get(name) {
            return *existing_id;
        }

        let concept = Concept::new(name, definition);
        let id = concept.id;

        if self.concepts.len() >= self.max_concepts {
            self.evict_senescent();
        }

        self.concepts.insert(id, concept);
        self.by_name.insert(name.to_string(), id);

        id
    }

    /// Get a concept by ID
    pub fn get(&self, id: &ConceptId) -> Option<&Concept> {
        self.concepts.get(id)
    }

    /// Get a concept by name
    pub fn get_by_name(&self, name: &str) -> Option<&Concept> {
        self.by_name.get(name).and_then(|id| self.concepts.get(id))
    }

    /// Record usage of a concept
    pub fn record_usage(&mut self, id: &ConceptId) {
        if let Some(concept) = self.concepts.get_mut(id) {
            concept.record_usage();
        }
    }

    /// Record a pain event
    pub fn record_pain(&mut self, id: &ConceptId) {
        if let Some(concept) = self.concepts.get_mut(id) {
            let old_stage = concept.stage;
            concept.record_pain();
            if concept.stage != old_stage {
                self.lifecycle_history.push(LifecycleEvent::new(
                    *id, old_stage, concept.stage, "pain threshold exceeded"
                ));
            }
        }
    }

    /// Record a contradiction
    pub fn record_contradiction(&mut self, id: &ConceptId) {
        if let Some(concept) = self.concepts.get_mut(id) {
            let old_stage = concept.stage;
            concept.record_contradiction();
            if concept.stage != old_stage {
                self.lifecycle_history.push(LifecycleEvent::new(
                    *id, old_stage, concept.stage, "contradiction detected"
                ));
            }
        }
    }

    /// Get concepts by lifecycle stage
    pub fn by_stage(&self, stage: LifecycleStage) -> Vec<&Concept> {
        self.concepts.values()
            .filter(|c| c.stage == stage)
            .collect()
    }

    /// Get all concepts in adolescence (need attention)
    pub fn adolescents(&self) -> Vec<&Concept> {
        self.by_stage(LifecycleStage::Adolescence)
    }

    /// Get concepts that should be retired
    pub fn candidates_for_retirement(&self) -> Vec<&Concept> {
        self.concepts.values()
            .filter(|c| c.should_retire())
            .collect()
    }

    /// Promote a concept to maturity
    pub fn promote(&mut self, id: &ConceptId) {
        if let Some(concept) = self.concepts.get_mut(id) {
            let old_stage = concept.stage;
            if matches!(concept.stage, LifecycleStage::Adolescence) {
                concept.stage = LifecycleStage::Maturity;
                concept.confidence = (concept.confidence * 0.8 + 0.8).min(1.0);
                self.lifecycle_history.push(LifecycleEvent::new(
                    *id, old_stage, LifecycleStage::Maturity, "promoted to maturity"
                ));
            }
        }
    }

    /// Retire a concept
    pub fn retire(&mut self, id: &ConceptId) {
        if let Some(concept) = self.concepts.get_mut(id) {
            let old_stage = concept.stage;
            concept.stage = LifecycleStage::Death;
            self.lifecycle_history.push(LifecycleEvent::new(
                *id, old_stage, LifecycleStage::Death, "retired"
            ));
        }
    }

    /// Evict senescent concepts to make room
    fn evict_senescent(&mut self) {
        let senescent: Vec<_> = self.concepts.values()
            .filter(|c| matches!(c.stage, LifecycleStage::Senescence))
            .map(|c| c.id)
            .collect();

        for id in senescent.into_iter().take(10) {
            if let Some(concept) = self.concepts.remove(&id) {
                self.by_name.remove(&concept.name);
            }
        }
    }

    /// Get concept count by stage
    pub fn stage_counts(&self) -> HashMap<LifecycleStage, usize> {
        let mut counts = HashMap::new();
        for concept in self.concepts.values() {
            *counts.entry(concept.stage).or_insert(0) += 1;
        }
        counts
    }

    /// Get recent lifecycle transitions
    pub fn recent_transitions(&self, n: usize) -> Vec<&LifecycleEvent> {
        self.lifecycle_history.iter().rev().take(n).collect()
    }

    /// Summary statistics
    pub fn summary(&self) -> String {
        let counts = self.stage_counts();
        format!(
            "Concepts: {} total. Birth: {}, Adolescence: {}, Maturity: {}, Senescence: {}, Death: {}",
            self.concepts.len(),
            counts.get(&LifecycleStage::Birth).unwrap_or(&0),
            counts.get(&LifecycleStage::Adolescence).unwrap_or(&0),
            counts.get(&LifecycleStage::Maturity).unwrap_or(&0),
            counts.get(&LifecycleStage::Senescence).unwrap_or(&0),
            counts.get(&LifecycleStage::Death).unwrap_or(&0),
        )
    }
}

impl Default for ConceptRegistry {
    fn default() -> Self {
        Self::new()
    }
}