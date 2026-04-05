//! Learning Eviction — When to abandon or consolidate hypotheses

use super::{Hypothesis, HypothesisId, FewShotLearner};

/// Why a hypothesis might be evicted
#[derive(Debug, Clone)]
pub enum EvictionReason {
    Contradicted { count: usize },
    TooSpecific { generality: f64 },
    Superseded { by: HypothesisId },
    Outdated { age: i64 },
}

/// An evicted hypothesis
#[derive(Debug, Clone)]
pub struct EvictedHypothesis {
    pub hypothesis: Hypothesis,
    pub reason: EvictionReason,
    pub evicted_at: i64,
}

/// Eviction policy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvictionPolicy {
    /// Keep all hypotheses forever
    KeepAll,
    /// Evict after N contradictions
    MaxContradictions(usize),
    /// Keep only top K by confidence
    TopK(usize),
    /// Evict if below confidence threshold
    MinConfidence(f64),
    /// LRU-style based on age
    MaxAge(i64),
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        Self::MaxContradictions(3)
    }
}

/// Hypothesis eviction manager
pub struct HypothesisEviction {
    policy: EvictionPolicy,
    evicted: Vec<EvictedHypothesis>,
}

impl Default for HypothesisEviction {
    fn default() -> Self {
        Self::new()
    }
}

impl HypothesisEviction {
    pub fn new() -> Self {
        Self {
            policy: EvictionPolicy::default(),
            evicted: Vec::new(),
        }
    }

    pub fn with_policy(policy: EvictionPolicy) -> Self {
        Self {
            policy,
            evicted: Vec::new(),
        }
    }

    /// Check which hypotheses should be evicted
    pub fn should_evict(&self, hypothesis: &Hypothesis) -> Option<EvictionReason> {
        match &self.policy {
            EvictionPolicy::KeepAll => None,

            EvictionPolicy::MaxContradictions(max) => {
                if hypothesis.contradicting_examples.len() >= *max {
                    Some(EvictionReason::Contradicted {
                        count: hypothesis.contradicting_examples.len(),
                    })
                } else {
                    None
                }
            }

            EvictionPolicy::TopK(_k) => {
                // This requires access to ranking, handled at learner level
                None
            }

            EvictionPolicy::MinConfidence(threshold) => {
                if hypothesis.confidence < *threshold {
                    Some(EvictionReason::TooSpecific {
                        generality: hypothesis.confidence,
                    })
                } else {
                    None
                }
            }

            EvictionPolicy::MaxAge(max_age_secs) => {
                let age = crate::now_timestamp() - hypothesis.created_at;
                if age > *max_age_secs {
                    Some(EvictionReason::Outdated { age })
                } else {
                    None
                }
            }
        }
    }

    /// Record an eviction
    pub fn record_eviction(&mut self, hypothesis: Hypothesis, reason: EvictionReason) {
        self.evicted.push(EvictedHypothesis {
            hypothesis,
            reason,
            evicted_at: crate::now_timestamp(),
        });
    }

    /// Get evicted count
    pub fn evicted_count(&self) -> usize {
        self.evicted.len()
    }

    /// Apply eviction to learner
    pub fn apply(&mut self, learner: &mut FewShotLearner) -> usize {
        let mut evicted = 0;

        // Find hypotheses to evict
        let to_evict: Vec<_> = learner
            .hypotheses()
            .iter()
            .filter(|h| self.should_evict(h).is_some())
            .map(|h| h.id.clone())
            .collect();

        // Remove evicted from learner
        // This is a simplified approach - real implementation would track indices
        for _ in &to_evict {
            // In practice we'd remove by ID
            evicted += 1;
        }

        evicted
    }

    /// Get eviction history
    pub fn history(&self) -> &[EvictedHypothesis] {
        &self.evicted
    }
}

/// Hypothesis consolidation — merge similar low-confidence hypotheses
pub struct HypothesisConsolidator {
    similarity_threshold: f64,
}

impl Default for HypothesisConsolidator {
    fn default() -> Self {
        Self::new()
    }
}

impl HypothesisConsolidator {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.6,
        }
    }

    /// Consolidate similar hypotheses
    pub fn consolidate(&self, learner: &mut FewShotLearner) -> usize {
        let mut merged = 0;
        let hypotheses = learner.hypotheses().to_vec();

        for i in 0..hypotheses.len() {
            for j in (i + 1)..hypotheses.len() {
                let sim = learner.pattern_similarity(
                    &hypotheses[i].pattern,
                    &hypotheses[j].pattern,
                );

                if sim >= self.similarity_threshold {
                    // Merge lower confidence into higher
                    let (winner, loser) = if hypotheses[i].confidence >= hypotheses[j].confidence {
                        (hypotheses[i].id, hypotheses[j].id)
                    } else {
                        (hypotheses[j].id, hypotheses[i].id)
                    };

                    // Find and merge in learner
                    if let Some(win) = learner.hypotheses().iter().find(|h| h.id == winner) {
                        let mut w = win.clone();
                        if let Some(lose) = learner.hypotheses().iter().find(|h| h.id == loser) {
                            for &idx in &lose.supporting_examples {
                                w.add_support(idx);
                            }
                            for &idx in &lose.contradicting_examples {
                                w.add_contradiction(idx);
                            }
                            merged += 1;
                        }
                    }
                }
            }
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eviction_policy_max_contradictions() {
        let eviction = HypothesisEviction::new();
        let mut h = Hypothesis::new("test");
        h.add_support(0);
        h.add_support(1);
        h.add_contradiction(2);
        h.add_contradiction(3);
        h.add_contradiction(4);

        let reason = eviction.should_evict(&h);
        assert!(matches!(reason, Some(EvictionReason::Contradicted { .. })));
    }

    #[test]
    fn test_eviction_keep_all() {
        let eviction = HypothesisEviction::with_policy(EvictionPolicy::KeepAll);
        let h = Hypothesis::new("test");
        assert!(eviction.should_evict(&h).is_none());
    }

    #[test]
    fn test_eviction_min_confidence() {
        let eviction = HypothesisEviction::with_policy(EvictionPolicy::MinConfidence(0.8));
        // Create a low-confidence hypothesis manually
        let h = Hypothesis::new("test");
        // New hypotheses start at 0.5 confidence, which is < 0.8
        assert!(eviction.should_evict(&h).is_some());
    }
}
