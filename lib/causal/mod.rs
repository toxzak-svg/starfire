//! Causal Discovery — Extract causal relationships from temporal patterns
//!
//! Uses chaos metrics (Lyapunov exponents, attractor dynamics) from Quanot
//! to discover causal relationships between entities and feed them into
//! Starfire's knowledge graph.
//!
//! # Theory
//!
//! Temporal precedence + correlation + causal mechanism = causation

pub mod discovery;
pub mod graph;
pub mod validation;

pub use discovery::DiscoveredCausalEdge;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A causal edge in the causal graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub id: CausalEdgeId,
    pub cause: String,
    pub effect: String,
    pub confidence: f64,
    pub evidence_count: usize,
    pub temporal_lag: Option<i64>,
    pub mechanism: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausalEdgeId(u64);

impl CausalEdgeId {
    pub fn new(n: u64) -> Self {
        Self(n)
    }
}

/// Confidence level of a causal hypothesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceState {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl ConfidenceState {
    pub fn from_score(score: f64) -> Self {
        if score < 0.3 {
            ConfidenceState::Low
        } else if score < 0.5 {
            ConfidenceState::Medium
        } else if score < 0.8 {
            ConfidenceState::High
        } else {
            ConfidenceState::VeryHigh
        }
    }
}

/// A causal hypothesis with supporting evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalHypothesis {
    pub candidate: CausalEdge,
    pub supporting_observations: usize,
    pub contradicting_observations: usize,
    pub confidence: ConfidenceState,
    pub created_at: i64,
}

impl CausalHypothesis {
    pub fn new(edge: CausalEdge) -> Self {
        let score = edge.confidence;
        Self {
            candidate: edge,
            supporting_observations: 0,
            contradicting_observations: 0,
            confidence: ConfidenceState::from_score(score),
            created_at: crate::now_timestamp(),
        }
    }

    pub fn update(&mut self, supports: bool) {
        if supports {
            self.supporting_observations += 1;
        } else {
            self.contradicting_observations += 1;
        }
        self.recompute_confidence();
    }

    fn recompute_confidence(&mut self) {
        let total = self.supporting_observations + self.contradicting_observations;
        if total == 0 {
            return;
        }
        let ratio = self.supporting_observations as f64 / total as f64;
        self.confidence = ConfidenceState::from_score(ratio);
    }
}

/// Causal engine
#[derive(Debug, Clone)]
pub struct CausalEngine {
    edges: HashMap<CausalEdgeId, CausalEdge>,
    hypotheses: Vec<CausalHypothesis>,
    next_id: u64,
}

impl Default for CausalEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalEngine {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            hypotheses: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a candidate causal edge
    pub fn add_edge(&mut self, cause: &str, effect: &str, confidence: f64, temporal_lag: Option<i64>) -> CausalEdgeId {
        let id = CausalEdgeId::new(self.next_id);
        self.next_id += 1;

        let edge = CausalEdge {
            id,
            cause: cause.to_string(),
            effect: effect.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            evidence_count: 1,
            temporal_lag,
            mechanism: None,
        };

        self.edges.insert(id, edge.clone());

        let hypothesis = CausalHypothesis::new(edge);
        self.hypotheses.push(hypothesis);

        id
    }

    /// Update an edge with new evidence
    pub fn update_edge(&mut self, edge_id: &CausalEdgeId, supports: bool) -> bool {
        let edge = match self.edges.get_mut(edge_id) {
            Some(e) => e,
            None => return false,
        };

        edge.evidence_count += 1;
        if supports {
            edge.confidence = (edge.confidence * 0.9 + 0.1).min(1.0);
        } else {
            edge.confidence = (edge.confidence * 0.9).max(0.0);
        }

        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.candidate.id == *edge_id) {
            h.update(supports);
        }

        true
    }

    /// Get all edges
    pub fn edges(&self) -> &HashMap<CausalEdgeId, CausalEdge> {
        &self.edges
    }

    /// Get edges where cause is the given entity
    pub fn get_causes_of(&self, effect: &str) -> Vec<&CausalEdge> {
        self.edges.values().filter(|e| &e.effect == effect).collect()
    }

    /// Get edges where effect is the given entity
    pub fn get_effects_of(&self, cause: &str) -> Vec<&CausalEdge> {
        self.edges.values().filter(|e| &e.cause == cause).collect()
    }

    /// Get hypotheses sorted by confidence
    pub fn top_hypotheses(&self, limit: usize) -> Vec<&CausalHypothesis> {
        let mut hypotheses: Vec<_> = self.hypotheses.iter().collect();
        hypotheses.sort_by(|a, b| {
            b.candidate.confidence
                .partial_cmp(&a.candidate.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hypotheses.truncate(limit);
        hypotheses
    }

    /// Get total edge count
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_engine_add_edge() {
        let mut engine = CausalEngine::new();
        let _id = engine.add_edge("fire", "heat", 0.8, Some(1));

        let edges = engine.get_effects_of("fire");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].cause, "fire");
    }

    #[test]
    fn test_causal_engine_update() {
        let mut engine = CausalEngine::new();
        let id = engine.add_edge("fire", "heat", 0.5, Some(1));

        engine.update_edge(&id, true);
        let edges = engine.get_effects_of("fire");
        assert!(edges[0].confidence > 0.5);
    }

    #[test]
    fn test_top_hypotheses() {
        let mut engine = CausalEngine::new();
        engine.add_edge("A", "B", 0.3, None);
        engine.add_edge("X", "Y", 0.9, None);
        engine.add_edge("M", "N", 0.6, None);

        let top = engine.top_hypotheses(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].candidate.cause, "X");
    }

    #[test]
    fn test_confidence_state() {
        assert_eq!(ConfidenceState::from_score(0.2), ConfidenceState::Low);
        assert_eq!(ConfidenceState::from_score(0.4), ConfidenceState::Medium);
        assert_eq!(ConfidenceState::from_score(0.7), ConfidenceState::High);
        assert_eq!(ConfidenceState::from_score(0.9), ConfidenceState::VeryHigh);
    }
}
