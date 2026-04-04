//! Causal Discovery — Pattern → Causality inference
//!
//! Takes temporal patterns from Quanot and infers causal relationships.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Temporal observation of an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalObservation {
    pub entity: String,
    pub timestamp: i64,
    pub value: f64,
    pub state: Vec<f64>,
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub min_lag: i64,
    pub min_correlation: f64,
    pub max_temporal_lag: i64,
    pub min_evidence: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            min_lag: 1,
            min_correlation: 0.3,
            max_temporal_lag: 100,
            min_evidence: 3,
        }
    }
}

/// Causal discovery from temporal patterns
pub struct CausalDiscovery {
    config: DiscoveryConfig,
}

impl Default for CausalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalDiscovery {
    pub fn new() -> Self {
        Self {
            config: DiscoveryConfig::default(),
        }
    }

    /// Discover causal edges from temporal observations
    pub fn discover(&self, observations: &[TemporalObservation]) -> Vec<DiscoveredCausalEdge> {
        let mut candidates = Vec::new();

        // Group by entity manually
        let mut entity_obs: HashMap<String, Vec<&TemporalObservation>> = HashMap::new();
        for obs in observations {
            entity_obs.entry(obs.entity.clone()).or_default().push(obs);
        }

        let entities: Vec<_> = entity_obs.keys().collect();

        // For each pair of entities, check for causation
        for i in 0..entities.len() {
            for j in 0..entities.len() {
                if i == j {
                    continue;
                }

                let cause = entities[i];
                let effect = entities[j];

                if let Some(candidate) = self.check_causation(
                    entity_obs.get(cause).unwrap(),
                    entity_obs.get(effect).unwrap(),
                    cause,
                    effect,
                ) {
                    if candidate.confidence >= self.config.min_correlation {
                        candidates.push(candidate);
                    }
                }
            }
        }

        // Sort by confidence
        candidates.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }

    fn check_causation(
        &self,
        cause_obs: &[&TemporalObservation],
        effect_obs: &[&TemporalObservation],
        cause_name: &str,
        effect_name: &str,
    ) -> Option<DiscoveredCausalEdge> {
        let cause_obs_owned: Vec<TemporalObservation> = cause_obs.iter().map(|o| (*o).clone()).collect();
        let effect_obs_owned: Vec<TemporalObservation> = effect_obs.iter().map(|o| (*o).clone()).collect();
        let mut precedences = Vec::new();

        for co in cause_obs {
            for eo in effect_obs {
                let lag = eo.timestamp - co.timestamp;
                if lag >= self.config.min_lag && lag <= self.config.max_temporal_lag {
                    precedences.push((co, eo, lag));
                }
            }
        }

        if precedences.len() < self.config.min_evidence {
            return None;
        }

        let correlation = self.compute_correlation(&cause_obs_owned, &effect_obs_owned);
        if correlation < self.config.min_correlation {
            return None;
        }

        let avg_lag: i64 = precedences.iter().map(|(_, _, lag)| *lag).sum::<i64>()
            / precedences.len() as i64;

        Some(DiscoveredCausalEdge {
            cause: cause_name.to_string(),
            effect: effect_name.to_string(),
            confidence: correlation,
            evidence_count: precedences.len(),
            temporal_lag: avg_lag,
        })
    }

    fn compute_correlation(
        &self,
        cause_obs: &[TemporalObservation],
        effect_obs: &[TemporalObservation],
    ) -> f64 {
        let mut pairs: Vec<(f64, f64)> = Vec::new();

        for co in cause_obs {
            for eo in effect_obs {
                if (eo.timestamp - co.timestamp).abs() <= self.config.max_temporal_lag {
                    pairs.push((co.value, eo.value));
                }
            }
        }

        if pairs.len() < 2 {
            return 0.0;
        }

        let n = pairs.len() as f64;
        let mean_x: f64 = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y: f64 = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;

        let mut covariance = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;

        for (x, y) in &pairs {
            let dx = x - mean_x;
            let dy = y - mean_y;
            covariance += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }

        let denom = (var_x.sqrt() * var_y.sqrt()).max(1e-10);
        covariance / denom
    }

    #[allow(dead_code)]
    pub fn refine_with_chaos(
        &self,
        mut edge: DiscoveredCausalEdge,
        _cause_chaos: &crate::quanot::chaos::ChaosMetrics,
        _effect_chaos: &crate::quanot::chaos::ChaosMetrics,
    ) -> DiscoveredCausalEdge {
        edge
    }
}

/// A discovered causal edge (unvalidated)
#[derive(Debug, Clone)]
pub struct DiscoveredCausalEdge {
    pub cause: String,
    pub effect: String,
    pub confidence: f64,
    pub evidence_count: usize,
    pub temporal_lag: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.min_lag, 1);
        assert_eq!(config.min_correlation, 0.3);
    }

    #[test]
    fn test_compute_correlation() {
        let discovery = CausalDiscovery::new();
        let obs1 = vec![TemporalObservation {
            entity: "A".to_string(),
            timestamp: 0,
            value: 1.0,
            state: vec![],
        }];
        let obs2 = vec![TemporalObservation {
            entity: "B".to_string(),
            timestamp: 1,
            value: 2.0,
            state: vec![],
        }];
        let corr = discovery.compute_correlation(&obs1, &obs2);
        assert!(corr >= -1.0 && corr <= 1.0);
    }
}
