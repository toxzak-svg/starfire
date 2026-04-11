//! AIH — Anticipatory Intent Horizon (Layer 3)
//!
//! Rolling probability cone of likely future actions, re-evaluated on every CEF event.
//! P(intent_{t+N}) = BGE(history) × e^(-λ × N) × CEF_weight

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::cef::{CEF, CausalEvent};
use super::bge::{BGE, ArchetypeId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPrediction {
    pub rank: usize,
    pub action: String,
    pub probability: f64,
    pub horizon: usize,
    pub causal_parents: Vec<super::cef::EventId>,
    pub archetype_id: Option<ArchetypeId>,
}

impl IntentPrediction {
    pub fn new(action: &str, probability: f64, horizon: usize) -> Self {
        Self {
            rank: 0,
            action: action.to_string(),
            probability,
            horizon,
            causal_parents: Vec::new(),
            archetype_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AIH {
    lambda: f64,
    cone_depth: usize,
    predictions: Vec<IntentPrediction>,
    archetype_lambdas: HashMap<ArchetypeId, f64>,
}

impl Default for AIH {
    fn default() -> Self {
        Self::new(0.15, 5)
    }
}

impl AIH {
    pub fn new(lambda: f64, cone_depth: usize) -> Self {
        Self {
            lambda,
            cone_depth,
            predictions: Vec::new(),
            archetype_lambdas: HashMap::new(),
        }
    }

    pub fn build_cone(&mut self, bge: &BGE, cef: &CEF, event_history: &[CausalEvent]) -> Vec<IntentPrediction> {
        self.predictions.clear();

        if event_history.is_empty() {
            return Vec::new();
        }

        let current_lambda = bge.current_archetype()
            .and_then(|a| self.archetype_lambdas.get(&a.id))
            .copied()
            .unwrap_or(self.lambda);

        let mut action_scores: HashMap<String, (f64, usize, Vec<super::cef::EventId>)> = HashMap::new();

        for horizon in 1..=self.cone_depth {
            let decay = (-current_lambda * horizon as f64).exp();

            let markov_factor = bge.current_archetype()
                .and_then(|a| bge.markov().most_likely(a.id))
                .map(|(_, p)| p)
                .unwrap_or(0.5);

            let recent = cef.recent_events(3600);
            let cef_weight: f64 = if recent.is_empty() {
                0.5
            } else {
                recent.iter().map(|(_, w)| w).sum::<f64>() / recent.len() as f64
            };

            for action in self.candidate_actions(bge) {
                let regularity = self.action_regularity(&action, bge);
                let combined = decay * markov_factor * cef_weight * (0.5 + 0.5 * regularity);

                let entry = action_scores.entry(action.clone()).or_insert((0.0, horizon, Vec::new()));
                entry.0 = entry.0.max(combined);
                if entry.2.is_empty() {
                    entry.2 = self.find_causal_parents(&action, event_history);
                }
            }
        }

        let mut scored: Vec<_> = action_scores.into_iter()
            .map(|(action, (prob, horizon, parents))| IntentPrediction {
                rank: 0,
                action,
                probability: prob.clamp(0.0, 1.0),
                horizon,
                causal_parents: parents,
                archetype_id: bge.current_archetype().map(|a| a.id),
            })
            .collect();

        scored.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        for (i, pred) in scored.iter_mut().enumerate() {
            pred.rank = i + 1;
        }

        self.predictions = scored.clone();
        scored
    }

    fn candidate_actions(&self, bge: &BGE) -> Vec<String> {
        let recent: Vec<_> = bge.action_history().iter().rev().take(10).cloned().collect();
        let mut candidates = Vec::new();

        for action in &recent {
            if !candidates.contains(action) {
                candidates.push(action.clone());
            }
        }

        if let Some(arch) = bge.current_archetype() {
            for characteristic in &arch.characteristic_actions {
                if !candidates.contains(characteristic) {
                    candidates.push(characteristic.clone());
                }
            }
        }

        candidates.truncate(20);
        candidates
    }

    fn action_regularity(&self, action: &str, bge: &BGE) -> f64 {
        let count = *bge.action_counts().get(action).unwrap_or(&0) as f64;
        let total: usize = bge.action_counts().values().sum();
        if total == 0 {
            return 0.0;
        }
        count / total as f64
    }

    fn find_causal_parents(&self, action: &str, history: &[CausalEvent]) -> Vec<super::cef::EventId> {
        history.iter().filter(|e| &e.action == action).map(|e| e.id).take(3).collect()
    }

    pub fn top_predictions(&self, k: usize) -> Vec<IntentPrediction> {
        self.predictions.iter().take(k).cloned().collect()
    }

    pub fn adapt_lambda(&mut self, arch_id: &ArchetypeId, delta: &super::oafl::PredictionDelta, bge: &mut BGE) {
        let current = *self.archetype_lambdas.get(arch_id).unwrap_or(&self.lambda);
        let adjustment = match delta.quality {
            super::oafl::MatchQuality::Perfect => -0.02,
            super::oafl::MatchQuality::Partial => 0.01,
            super::oafl::MatchQuality::Miss => 0.05,
        };
        let new_lambda = (current + adjustment).clamp(0.01, 1.0);
        self.archetype_lambdas.insert(*arch_id, new_lambda);
        if let Some(arch) = bge.archetypes_mut().iter_mut().find(|a| &a.id == arch_id) {
            arch.archetype_lambda = new_lambda;
        }
    }

    pub fn predictions_len(&self) -> usize {
        self.predictions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cone_empty() {
        let aih = AIH::default();
        assert!(aih.predictions.is_empty());
    }

    #[test]
    fn test_top_predictions_limit() {
        let mut aih = AIH::new(0.15, 5);
        let bge = BGE::default();
        let cef = CEF::default();
        let events = vec![];
        let _predictions = aih.build_cone(&bge, &cef, &events);
        let top3 = aih.top_predictions(3);
        assert!(top3.len() <= 3);
    }

    #[test]
    fn test_intent_prediction_ranking() {
        let mut pred1 = IntentPrediction::new("action1", 0.8, 1);
        let mut pred2 = IntentPrediction::new("action2", 0.3, 1);
        pred1.rank = 1;
        pred2.rank = 2;
        assert!(pred1.probability > pred2.probability);
    }
}
