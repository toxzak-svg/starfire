//! Gap Analysis — Analyzes knowledge for gaps

use super::{GapType, KnowledgeGap, CurriculumEngine};

/// Gap analyzer
pub struct GapAnalyzer {
    min_confidence_for_knowledge: f64,
}

impl Default for GapAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapAnalyzer {
    pub fn new() -> Self {
        Self {
            min_confidence_for_knowledge: 0.3,
        }
    }

    /// Analyze text for potential knowledge gaps
    pub fn analyze(&self, text: &str, known_topics: &[&str]) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        let text_lower = text.to_lowercase();

        for topic in known_topics {
            if !text_lower.contains(&topic.to_lowercase()) {
                // Topic not mentioned — might be a gap
                let gap = KnowledgeGap::new(
                    topic.to_string(),
                    GapType::CompleteIgnorance,
                ).with_urgency(0.4);

                gaps.push(gap);
            }
        }

        gaps
    }

    /// Assess a topic for gap type
    pub fn assess_gap_type(&self, mentions: &[&str], misconceptions: &[&str]) -> GapType {
        if !misconceptions.is_empty() {
            GapType::Misconception
        } else if mentions.len() == 1 {
            GapType::Incomplete
        } else if mentions.len() > 3 {
            GapType::Unconnected
        } else {
            GapType::CompleteIgnorance
        }
    }

    /// Compute urgency score for a gap
    pub fn compute_urgency(&self, gap: &KnowledgeGap, context_importance: f64) -> f64 {
        let base = match gap.gap_type {
            GapType::CompleteIgnorance => 0.7,
            GapType::Misconception => 0.8, // Misconceptions are dangerous
            GapType::Incomplete => 0.5,
            GapType::Unconnected => 0.3,
        };

        (base + context_importance) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_analyzer_analyze() {
        let analyzer = GapAnalyzer::new();
        let known = vec!["Rust", "AI", "Programming"];
        let gaps = analyzer.analyze("I know about Rust", &known);

        // "AI" and "Programming" not mentioned
        assert!(gaps.len() >= 2);
    }

    #[test]
    fn test_assess_gap_type() {
        let analyzer = GapAnalyzer::new();

        let gap_type = analyzer.assess_gap_type(&["rust"], &[]);
        assert_eq!(gap_type, GapType::Incomplete);

        let gap_type = analyzer.assess_gap_type(&[], &["rust is slow"]);
        assert_eq!(gap_type, GapType::Misconception);
    }
}
