//! Dream validator — validates dream hypotheses against real experiences

use super::dream_engine::{DreamEngine, DreamEpisode, DreamId};

/// How well a dream hypothesis was supported
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub dream_id: DreamId,
    pub hypothesis: String,
    pub supported: bool,
    pub confidence: f64,
    pub evidence: String,
}

/// Validates dream episodes against real conversations
pub struct DreamValidator {
    similarity_threshold: f64,
}

impl DreamValidator {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.6,
        }
    }

    /// Validate a dream episode against recent real experiences
    pub fn validate(&self, episode: &DreamEpisode, recent_experiences: &[String]) -> ValidationResult {
        let mut support_count = 0;
        let mut total_similarity = 0.0;

        for exp in recent_experiences {
            let sim = self.text_similarity(&episode.hypothesis, exp);
            if sim > self.similarity_threshold {
                support_count += 1;
            }
            total_similarity += sim;
        }

        let avg_similarity = if recent_experiences.is_empty() {
            0.5
        } else {
            total_similarity / recent_experiences.len() as f64
        };

        let supported = support_count > 0;
        let confidence = avg_similarity;

        ValidationResult {
            dream_id: episode.id,
            hypothesis: episode.hypothesis.clone(),
            supported,
            confidence,
            evidence: if supported {
                format!("{} similar experiences found", support_count)
            } else {
                "No supporting experiences found".to_string()
            },
        }
    }

    /// Text similarity (simplified)
    fn text_similarity(&self, a: &str, b: &str) -> f64 {
        let a_words: std::collections::HashSet<_> = a.split_whitespace().collect();
        let b_words: std::collections::HashSet<_> = b.split_whitespace().collect();

        if a_words.is_empty() || b_words.is_empty() {
            return 0.0;
        }

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        intersection as f64 / union as f64
    }

    /// Apply validation to the dream engine
    pub fn apply_validation(&self, engine: &mut DreamEngine, episodes: &[DreamEpisode], recent_experiences: &[String]) {
        for episode in episodes {
            if !episode.validated {
                let result = self.validate(episode, recent_experiences);
                engine.mark_validated(
                    result.dream_id,
                    result.supported,
                    result.confidence,
                );
            }
        }
    }
}

impl Default for DreamValidator {
    fn default() -> Self {
        Self::new()
    }
}