//! Hypothesis — Hypothesis generation, evaluation, and refinement

use super::{Example, Hypothesis, HypothesisId, FewShotLearner};
use std::collections::HashMap;

/// Hypothesis evaluator
pub struct HypothesisEvaluator {
    accuracy_weight: f64,
    simplicity_weight: f64,
    generality_weight: f64,
}

impl Default for HypothesisEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl HypothesisEvaluator {
    pub fn new() -> Self {
        Self {
            accuracy_weight: 0.5,
            simplicity_weight: 0.2,
            generality_weight: 0.3,
        }
    }

    /// Score a hypothesis
    pub fn score(&self, hypothesis: &Hypothesis) -> f64 {
        // Accuracy
        let total = hypothesis.supporting_examples.len() + hypothesis.contradicting_examples.len();
        let accuracy = if total > 0 {
            hypothesis.supporting_examples.len() as f64 / total as f64
        } else {
            0.5
        };

        // Simplicity (shorter patterns are simpler)
        let word_count = hypothesis.pattern.split_whitespace().count();
        let simplicity = 1.0 / (1.0 + word_count as f64 * 0.1);

        // Generality (applies to more domains)
        let generality = hypothesis.generality;

        accuracy * self.accuracy_weight
            + simplicity * self.simplicity_weight
            + generality * self.generality_weight
    }

    /// Compare two hypotheses
    pub fn is_better(&self, a: &Hypothesis, b: &Hypothesis) -> bool {
        self.score(a) > self.score(b)
    }
}

/// Hypothesis generator
pub struct HypothesisGenerator {
    min_support: usize,
    max_pattern_length: usize,
}

impl Default for HypothesisGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl HypothesisGenerator {
    pub fn new() -> Self {
        Self {
            min_support: 2,
            max_pattern_length: 10,
        }
    }

    /// Generate candidate hypotheses from examples
    pub fn generate(&self, examples: &[&Example]) -> Vec<Hypothesis> {
        let mut candidates = Vec::new();

        // Extract n-gram patterns from outputs
        for ex in examples {
            let words: Vec<_> = ex.output.split_whitespace().collect();
            let n = words.len().min(self.max_pattern_length);

            for len in 2..=n.max(2) {
                for window in words.windows(len) {
                    let pattern = window.join(" ");
                    if pattern.len() > 3 {
                        candidates.push(Hypothesis::new(pattern));
                    }
                }
            }
        }

        // Deduplicate by pattern
        let mut unique: HashMap<String, Hypothesis> = HashMap::new();
        for candidate in candidates {
            let pattern_lower = candidate.pattern.to_lowercase();
            unique.entry(pattern_lower).or_insert(candidate);
        }

        let mut results: Vec<_> = unique.into_values().collect();

        // Filter by support count (need at least min_support examples)
        let example_outputs: Vec<_> = examples.iter().map(|e| e.output.to_lowercase()).collect();
        for h in &mut results {
            h.supporting_examples = example_outputs
                .iter()
                .enumerate()
                .filter(|(_, out)| out.contains(&h.pattern.to_lowercase()))
                .map(|(i, _)| i)
                .collect();
            h.supporting_examples.retain(|_| {
                // This is simplified - real implementation would track actual indices
                true
            });
        }

        results.retain(|h| h.supporting_examples.len() >= self.min_support);
        results
    }

    /// Generate abstraction over patterns
    pub fn generalize(&self, hypothesis: &Hypothesis, more_examples: &[&Example]) -> Hypothesis {
        let mut generalized = hypothesis.clone();

        // Find commonalities across examples
        let mut word_freq: HashMap<String, usize> = HashMap::new();

        for ex in more_examples {
            for word in ex.output.split_whitespace() {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                if !clean.is_empty() {
                    *word_freq.entry(clean).or_insert(0) += 1;
                }
            }
        }

        // Words in most examples become part of generalized pattern
        let threshold = more_examples.len() / 2;
        let generalized_words: Vec<_> = word_freq
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(word, _)| word)
            .collect();

        if !generalized_words.is_empty() {
            generalized.pattern = generalized_words.join(" ");
            generalized.generality = 0.8;
        }

        generalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_evaluator_score() {
        let evaluator = HypothesisEvaluator::new();
        let mut h = Hypothesis::new("test");
        h.add_support(0);
        h.add_support(1);
        h.add_support(2);

        let score = evaluator.score(&h);
        assert!(score > 0.0);
    }

    #[test]
    fn test_hypothesis_generator() {
        let generator = HypothesisGenerator::new();
        let examples = vec![
            &Example::new("in1", "big fire", "test"),
            &Example::new("in2", "fire big", "test"),
        ];

        let candidates = generator.generate(&examples);
        // Should find "fire" and "big" patterns
        assert!(!candidates.is_empty());
    }
}
