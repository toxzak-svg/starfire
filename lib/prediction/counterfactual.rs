//! Counterfactual Projection — "what if" reasoning for the Prediction Center
//!
//! Philosophy: Enable "what if" reasoning by projecting what WOULD be true
//! if a given assumption held. Used by the goals system and reasoning engine.

use super::types::*;
use super::basin::BasinEngine;
use std::collections::HashMap;

/// Counterfactual Engine — projects "what if" scenarios
pub struct CounterfactualEngine {
    /// Cache of recent counterfactual results
    cache: HashMap<String, CounterfactualResult>,
    /// Maximum cache size
    max_cache_size: usize,
}

impl CounterfactualEngine {
    pub fn new() -> Self {
        CounterfactualEngine {
            cache: HashMap::new(),
            max_cache_size: 20,
        }
    }

    /// Project what WOULD be true if the given assumption held
    /// Used by ReasoningEngine for abduction + counterfactual simulation
    pub fn project_counterfactual(
        &mut self,
        assumption: &str,
        basin: &BasinEngine,
    ) -> CounterfactualResult {
        // Check cache first
        if let Some(cached) = self.cache.get(assumption) {
            return cached.clone();
        }

        // Parse the assumption into a constraint
        let constraint = self.parse_assumption(assumption);

        // Create a temporary BasinEngine with this constraint added
        let mut temp_basin = basin.clone();

        // Add the constraint
        if let Some((from, to, constraint_type)) = constraint {
            temp_basin.add_constraint(&from, &to, constraint_type, 0.9);
        }

        // Find the new equilibrium
        let new_predictions = temp_basin.predict_equilibrium();

        // Compare to baseline (without assumption)
        let baseline = basin.predict_equilibrium();
        let divergence = self.compute_divergence(&new_predictions, &baseline);

        // Confidence = how strong is the constraint?
        let confidence = 0.7; // Default confidence for counterfactual

        let result = CounterfactualResult {
            assumption: assumption.to_string(),
            divergence_from_baseline: divergence,
            confidence,
        };

        // Cache the result
        if self.cache.len() >= self.max_cache_size {
            // Remove oldest entry
            if let Some(oldest) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest);
            }
        }
        self.cache.insert(assumption.to_string(), result.clone());

        result
    }

    /// Parse an assumption string into a constraint
    pub fn parse_assumption(&self, assumption: &str) -> Option<(String, String, super::basin::ConstraintType)> {
        // Simple parsing: "X causes Y" or "X implies Y"
        let lower = assumption.to_lowercase();
        
        if lower.contains("causes") {
            let parts: Vec<&str> = lower.split("causes").collect();
            if parts.len() == 2 {
                let from = parts[0].trim().to_string();
                let to = parts[1].trim().to_string();
                return Some((from, to, super::basin::ConstraintType::Causation));
            }
        }
        
        if lower.contains("implies") {
            let parts: Vec<&str> = lower.split("implies").collect();
            if parts.len() == 2 {
                let from = parts[0].trim().to_string();
                let to = parts[1].trim().to_string();
                return Some((from, to, super::basin::ConstraintType::Implication));
            }
        }
        
        // Default: assume it's a boolean truth
        None
    }

    /// Compute the divergence between two prediction sets
    fn compute_divergence(&self, new: &[Prediction], baseline: &[Prediction]) -> Vec<PredictionDelta> {
        let mut divergence = Vec::new();
        
        // Find predictions that differ between new and baseline
        for new_pred in new {
            let baseline_pred = baseline.iter()
                .find(|b| b.kind == new_pred.kind && b.engine == new_pred.engine);
            
            if let Some(baseline) = baseline_pred {
                if (new_pred.confidence - baseline.confidence).abs() > 0.1 {
                    divergence.push(PredictionDelta {
                        prediction_id: new_pred.id,
                        before: baseline.confidence,
                        after: new_pred.confidence,
                    });
                }
            } else {
                // New prediction didn't exist in baseline
                divergence.push(PredictionDelta {
                    prediction_id: new_pred.id,
                    before: 0.0,
                    after: new_pred.confidence,
                });
            }
        }
        
        divergence
    }

    /// "What would have had to be true for X to happen?" — backward projection
    pub fn project_backward(
        &self,
        outcome: &str,
        basin: &BasinEngine,
    ) -> Vec<NecessaryPrecondition> {
        let mut preconditions = Vec::new();
        
        // Trace backward through causal chains
        // For outcome O: what causal path leads to O? What must be true?
        
        // Find all constraints that lead to this outcome
        // This is a simplified version
        
        preconditions.push(NecessaryPrecondition {
            condition: format!("{} must be true", outcome),
            confidence: 0.8,
            source: "backward_tracing".to_string(),
        });
        
        preconditions
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CounterfactualEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A necessary precondition for an outcome
#[derive(Debug, Clone)]
pub struct NecessaryPrecondition {
    pub condition: String,
    pub confidence: f64,
    pub source: String,
}

// Need to implement Clone for BasinEngine
// Since BasinEngine doesn't implement Clone, we'll create a workaround
impl BasinEngine {
    /// Create a copy of this basin engine (simplified)
    fn clone(&self) -> BasinEngine {
        let mut new_basin = BasinEngine::new();
        
        // Copy nodes using the public method
        for (id, value, confidence) in self.clone_nodes() {
            new_basin.add_node(&id, value, confidence);
        }
        
        // Copy constraints (simplified - just count)
        // In a full implementation, we'd copy actual constraints
        
        new_basin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assumption_causes() {
        let engine = CounterfactualEngine::new();
        
        let result = engine.parse_assumption("fire causes heat");
        
        assert!(result.is_some());
        let (from, to, _constraint_type) = result.unwrap();
        assert_eq!(from, "fire");
        assert_eq!(to, "heat");
    }

    #[test]
    fn test_parse_assumption_implies() {
        let engine = CounterfactualEngine::new();
        
        let result = engine.parse_assumption("rain implies wet ground");
        
        assert!(result.is_some());
    }

    #[test]
    fn test_cache() {
        let mut engine = CounterfactualEngine::new();
        
        // Create a minimal basin
        let basin = BasinEngine::new();
        
        // Project twice with same assumption
        let _ = engine.project_counterfactual("test assumption", &basin);
        let _ = engine.project_counterfactual("test assumption", &basin);
        
        // Cache should have the assumption
        assert!(engine.cache.contains_key("test assumption"));
    }

    #[test]
    fn test_project_backward() {
        let engine = CounterfactualEngine::new();
        let basin = BasinEngine::new();
        
        let preconditions = engine.project_backward("fire", &basin);
        
        assert!(!preconditions.is_empty());
    }
}