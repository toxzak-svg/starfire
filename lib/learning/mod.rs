//! Few-Shot Learning — Rapid hypothesis formation from examples
//!
//! Learns from a handful of examples without gradient updates.

pub mod hypothesis;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An example for few-shot learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub input: String,
    pub output: String,
    pub domain: String,
    pub weight: f64,
    pub timestamp: i64,
}

impl Example {
    pub fn new(input: impl Into<String>, output: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            output: output.into(),
            domain: domain.into(),
            weight: 1.0,
            timestamp: crate::now_timestamp(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }
}

/// A hypothesis formed from examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: HypothesisId,
    pub pattern: String,
    pub supporting_examples: Vec<usize>,  // indices into example list
    pub contradicting_examples: Vec<usize>,
    pub confidence: f64,
    pub generality: f64,
    pub predicted_applies_to: Vec<String>,
    pub created_at: i64,
}

impl Hypothesis {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            id: HypothesisId::new(),
            pattern: pattern.into(),
            supporting_examples: Vec::new(),
            contradicting_examples: Vec::new(),
            confidence: 0.5,
            generality: 0.5,
            predicted_applies_to: Vec::new(),
            created_at: crate::now_timestamp(),
        }
    }

    pub fn add_support(&mut self, example_idx: usize) {
        if !self.supporting_examples.contains(&example_idx) {
            self.supporting_examples.push(example_idx);
        }
        self.recompute_confidence();
    }

    pub fn add_contradiction(&mut self, example_idx: usize) {
        if !self.contradicting_examples.contains(&example_idx) {
            self.contradicting_examples.push(example_idx);
        }
        self.recompute_confidence();
    }

    fn recompute_confidence(&mut self) {
        let total = self.supporting_examples.len() + self.contradicting_examples.len();
        if total == 0 {
            return;
        }
        let ratio = self.supporting_examples.len() as f64 / total as f64;
        self.confidence = ratio;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HypothesisId(u64);

impl HypothesisId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}



/// Few-shot learner
#[derive(Debug, Clone)]
pub struct FewShotLearner {
    examples: Vec<Example>,
    hypotheses: Vec<Hypothesis>,
    domains: HashMap<String, Vec<usize>>,  // domain -> example indices
}

impl Default for FewShotLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl FewShotLearner {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
            hypotheses: Vec::new(),
            domains: HashMap::new(),
        }
    }

    /// Add an example
    pub fn add_example(&mut self, example: Example) -> usize {
        let idx = self.examples.len();
        self.examples.push(example.clone());

        self.domains
            .entry(example.domain.clone())
            .or_default()
            .push(idx);

        idx
    }

    /// Learn hypotheses from examples in a domain
    pub fn learn_from_domain(&mut self, domain: &str) -> Vec<Hypothesis> {
        let example_indices = match self.domains.get(domain) {
            Some(indices) => indices.clone(),
            None => return Vec::new(),
        };

        if example_indices.len() < 2 {
            return Vec::new();
        }

        // Extract input-output patterns
        let examples: Vec<_> = example_indices
            .iter()
            .map(|&i| &self.examples[i])
            .collect();

        // Find commonalities
        let mut hypothesis = self.find_pattern(&examples);

        // Add to supporting examples
        for &idx in &example_indices {
            hypothesis.add_support(idx);
        }

        // Predict what else this applies to
        hypothesis.predicted_applies_to = self.predict_applications(domain, &hypothesis.pattern);

        self.hypotheses.push(hypothesis.clone());
        vec![hypothesis]
    }

    /// Find a pattern across examples
    fn find_pattern(&self, examples: &[&Example]) -> Hypothesis {
        // Simple pattern: find common words in outputs
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for ex in examples {
            for word in ex.output.split_whitespace() {
                let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric(c)).to_lowercase();
                if !word_clean.is_empty() && word_clean.len() > 2 {
                    *word_counts.entry(word_clean).or_insert(0) += 1;
                }
            }
        }

        // Most common words might form the pattern
        let mut sorted: Vec<_> = word_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let pattern = if sorted.len() >= 2 {
            format!("{} + {}", sorted[0].0, sorted[1].0)
        } else if !sorted.is_empty() {
            sorted[0].0.clone()
        } else {
            "unknown".to_string()
        };

        Hypothesis::new(pattern)
    }

    /// Predict other contexts where this hypothesis applies
    fn predict_applications(&self, current_domain: &str, pattern: &str) -> Vec<String> {
        let mut predictions = Vec::new();

        // Find other domains with similar examples
        for (domain, indices) in &self.domains {
            if domain == current_domain {
                continue;
            }

            let has_supporting = indices.iter().any(|&i| {
                self.examples[i]
                    .output
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            });

            if has_supporting {
                predictions.push(domain.clone());
            }
        }

        predictions.truncate(5);
        predictions
    }

    /// Test a hypothesis against a new example
    pub fn test_hypothesis(&self, hypothesis_id: &HypothesisId, example_idx: usize) -> Option<bool> {
        let hypothesis = self.hypotheses.iter().find(|h| &h.id == hypothesis_id)?;
        let example = &self.examples[example_idx];

        // Check if output contains the pattern
        let supports = example
            .output
            .to_lowercase()
            .contains(&hypothesis.pattern.to_lowercase());

        Some(supports)
    }

    /// Merge similar hypotheses
    pub fn merge_similar(&mut self, threshold: f64) {
        let mut to_remove = Vec::new();

        for i in 0..self.hypotheses.len() {
            for j in (i + 1)..self.hypotheses.len() {
                let similarity = self.pattern_similarity(
                    &self.hypotheses[i].pattern,
                    &self.hypotheses[j].pattern,
                );

                if similarity >= threshold {
                    // Merge j into i
                    for &idx in &self.hypotheses[j].supporting_examples {
                        self.hypotheses[i].add_support(idx);
                    }
                    for &idx in &self.hypotheses[j].contradicting_examples {
                        self.hypotheses[i].add_contradiction(idx);
                    }
                    to_remove.push(j);
                }
            }
        }

        // Remove merged hypotheses
        for idx in to_remove.into_iter().rev() {
            self.hypotheses.remove(idx);
        }
    }

    /// Compute similarity between two patterns
    fn pattern_similarity(&self, p1: &str, p2: &str) -> f64 {
        let words1: Vec<_> = p1.split_whitespace().collect();
        let words2: Vec<_> = p2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection: Vec<_> = words1
            .iter()
            .filter(|w| words2.contains(w))
            .collect();

        intersection.len() as f64 / (words1.len().max(words2.len())) as f64
    }

    /// Get all hypotheses
    pub fn hypotheses(&self) -> &[Hypothesis] {
        &self.hypotheses
    }

    /// Get hypotheses sorted by confidence
    pub fn top_hypotheses(&self, limit: usize) -> Vec<&Hypothesis> {
        let mut hypotheses: Vec<_> = self.hypotheses.iter().collect();
        hypotheses.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hypotheses.truncate(limit);
        hypotheses
    }

    /// Get example count
    pub fn example_count(&self) -> usize {
        self.examples.len()
    }

    /// Get hypothesis count
    pub fn hypothesis_count(&self) -> usize {
        self.hypotheses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_new() {
        let ex = Example::new("input", "output", "test");
        assert_eq!(ex.input, "input");
        assert_eq!(ex.output, "output");
        assert_eq!(ex.domain, "test");
    }

    #[test]
    fn test_add_example() {
        let mut learner = FewShotLearner::new();
        let idx = learner.add_example(Example::new("a", "b", "domain1"));
        assert_eq!(idx, 0);
        assert_eq!(learner.example_count(), 1);
    }

    #[test]
    fn test_learn_from_domain() {
        let mut learner = FewShotLearner::new();
        learner.add_example(Example::new("input1", "hot fire", "physics"));
        learner.add_example(Example::new("input2", "fire hot", "physics"));
        learner.add_example(Example::new("input3", "hot flame", "physics"));

        let hypotheses = learner.learn_from_domain("physics");
        assert!(!hypotheses.is_empty());
    }

    #[test]
    fn test_pattern_similarity() {
        let learner = FewShotLearner::new();
        let sim = learner.pattern_similarity("hot fire", "fire hot");
        assert!(sim > 0.5);
    }

    #[test]
    fn test_hypothesis_confidence() {
        let mut h = Hypothesis::new("test pattern");
        h.add_support(0);
        h.add_support(1);
        h.add_support(2);
        h.add_contradiction(3);

        // 3 support, 1 contradict = 0.75 confidence
        assert!((h.confidence - 0.75).abs() < 0.01);
    }
}
