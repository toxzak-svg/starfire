//! Question Gravity Engine — predicts which curiosity questions will fire
//!
//! Philosophy: Questions are leading indicators of what Star will reason about next.
//! The questions Star's curiosity engine fires are driven by gaps in the knowledge graph.
//! We predict which gaps have the highest "gravity" (tension × relevance × fertility).

use super::types::*;
use std::collections::HashMap;

/// Question Gravity Engine — predicts which curiosity questions will fire
pub struct QuestionGravityEngine {
    /// Conversation trajectory
    conversation_topics: Vec<TopicVector>,
    /// Gap history for tracking
    gap_history: Vec<TrackedGap>,
    /// Current gaps
    current_gaps: Vec<KnowledgeGap>,
}

impl QuestionGravityEngine {
    pub fn new() -> Self {
        QuestionGravityEngine {
            conversation_topics: Vec::new(),
            gap_history: Vec::new(),
            current_gaps: Vec::new(),
        }
    }

    /// Update conversation trajectory with a new topic
    pub fn note_topic(&mut self, topic: &str, depth: usize) {
        let tv = TopicVector::new(topic.to_string(), depth);
        
        // Keep only recent topics
        if self.conversation_topics.len() >= 10 {
            self.conversation_topics.remove(0);
        }
        self.conversation_topics.push(tv);
    }

    /// Analyze the knowledge graph for current tension points
    /// This is a simplified version that works without direct KG access
    pub fn analyze_gaps(&mut self, context: &ConversationContext) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        
        // Use the current topic to determine topical distance
        let current_topic = &context.current_topic;
        
        // Generate gaps based on what's been discussed
        // In a full implementation, this would query the actual KG
        // For now, we create placeholder gaps based on conversation depth
        
        // Gap 1: Unknown property - if topic has been discussed but not fully understood
        if context.depth > 0 && context.discussed_entities.len() > 0 {
            for entity in &context.discussed_entities {
                gaps.push(KnowledgeGap::new(
                    GapType::UnknownProperty,
                    entity.clone(),
                    GapClosure::Property {
                        entity: entity.clone(),
                        property: "nature".to_string(),
                    },
                ));
            }
        }
        
        // Gap 2: High uncertainty about the current topic
        if context.depth > 2 {
            gaps.push(KnowledgeGap::new(
                GapType::UncertainBelief,
                current_topic.clone(),
                GapClosure::Evidence(0), // Placeholder
            ));
        }
        
        // Gap 3: Missing cause - for topics that have effects but unknown causes
        if context.depth > 1 {
            gaps.push(KnowledgeGap::new(
                GapType::MissingCause,
                current_topic.clone(),
                GapClosure::Cause(current_topic.clone()),
            ));
        }
        
        // Update current gaps
        self.current_gaps = gaps.clone();
        
        // Update tension scores based on topical distance
        for gap in &mut gaps {
            gap.topical_distance = self.compute_topical_distance(gap.topic.as_str(), current_topic);
            gap.tension = self.compute_tension(gap);
            gap.fertility_score = self.estimate_fertility(gap);
        }
        
        // Sort by prediction score
        gaps.sort_by(|a, b| {
            b.prediction_score().partial_cmp(&a.prediction_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        gaps
    }

    /// Compute topical distance from current conversation
    fn compute_topical_distance(&self, gap_topic: &str, current_topic: &str) -> f64 {
        if gap_topic.to_lowercase() == current_topic.to_lowercase() {
            0.1
        } else if gap_topic.to_lowercase().contains(&current_topic.to_lowercase())
            || current_topic.to_lowercase().contains(&gap_topic.to_lowercase())
        {
            0.3
        } else {
            // Use conversation trajectory to estimate
            for topic in &self.conversation_topics {
                if topic.topic.to_lowercase().contains(&gap_topic.to_lowercase()) {
                    return 0.5;
                }
            }
            1.0
        }
    }

    /// Compute tension score for a gap
    fn compute_tension(&self, gap: &KnowledgeGap) -> f64 {
        // Base tension depends on gap type
        let base_tension = match gap.gap_type {
            GapType::Contradiction => 0.8,
            GapType::MissingCause => 0.6,
            GapType::UncertainBelief => 0.5,
            GapType::UnknownProperty => 0.4,
            GapType::MissingAnalogy => 0.3,
            GapType::UnexplainedBehavior => 0.5,
        };
        
        // Adjust by conversation depth (more depth = more uncertainty)
        base_tension
    }

    /// Estimate fertility (how many connections this gap has)
    fn estimate_fertility(&self, gap: &KnowledgeGap) -> f64 {
        // More discussed entities = more potential connections
        let base_fertility = 0.5;
        
        // Gap type affects fertility
        match gap.gap_type {
            GapType::Contradiction => 0.9,
            GapType::MissingCause => 0.7,
            GapType::MissingAnalogy => 0.8,
            _ => base_fertility,
        }
    }

    /// Predict which questions will fire in the next N exchanges
    pub fn predict_questions(&self, gaps: &[KnowledgeGap], horizon: usize) -> Vec<Prediction> {
        gaps.iter()
            .take(horizon * 2)
            .map(|gap| {
                let question = self.gap_to_question(gap);
                let confidence = gap.prediction_score();
                
                Prediction::new(
                    PredictionEngine::QuestionGravity,
                    PredictionKind::Question,
                    PredictedCore::Question {
                        question_text: question.clone(),
                        topic_domain: gap.topic.clone(),
                        expected_answer_type: self.closure_to_answer_type(&gap.closure_requirement),
                    },
                    format!("Star will likely ask: '{}'", question),
                    confidence,
                    horizon,
                    vec![
                        format!("Gap detected: {:?}", gap.gap_type),
                        format!("Tension: {:.3}", gap.tension),
                        format!("Topical distance: {:.3}", gap.topical_distance),
                        format!("Fertility: {:.3}", gap.fertility_score),
                    ],
                ).with_expiry(horizon as i64 * 300)
            })
            .collect()
    }

    /// Convert a gap into a natural language question
    fn gap_to_question(&self, gap: &KnowledgeGap) -> String {
        match gap.gap_type {
            GapType::MissingCause => {
                format!("What causes {}?", gap.topic)
            }
            GapType::UnknownProperty => {
                format!("What is the nature of {}?", gap.topic)
            }
            GapType::Contradiction => {
                format!("How do these competing ideas both coexist?")
            }
            GapType::UncertainBelief => {
                format!("I think {} might be true but I'm not sure — what would confirm this?", gap.topic)
            }
            GapType::MissingAnalogy => {
                format!("Is there something analogous to {} in a different domain?", gap.topic)
            }
            GapType::UnexplainedBehavior => {
                format!("Why does {} behave this way?", gap.topic)
            }
        }
    }

    /// Convert closure requirement to expected answer type
    fn closure_to_answer_type(&self, closure: &GapClosure) -> AnswerType {
        match closure {
            GapClosure::Cause(_) => AnswerType::Causal,
            GapClosure::Resolution(_, _) => AnswerType::Causal,
            GapClosure::Analogy(_) => AnswerType::Entity,
            GapClosure::Property { .. } => AnswerType::Entity,
            GapClosure::Evidence(_) => AnswerType::Conclusion,
        }
    }

    /// Get the current gaps
    pub fn current_gaps(&self) -> &[KnowledgeGap] {
        &self.current_gaps
    }

    /// Update gap tensions from external analysis
    pub fn update_gap_tensions(&mut self, gaps: Vec<KnowledgeGap>) {
        self.current_gaps = gaps;
    }

    /// Record a gap that was actually fired (for learning)
    pub fn record_fired_gap(&mut self, gap_id: GapId) {
        for tracked in &mut self.gap_history {
            if tracked.gap.id == gap_id {
                tracked.actual_fire_time = Some(crate::now_timestamp());
            }
        }
    }

    /// Mark a gap as resolved
    pub fn resolve_gap(&mut self, gap_id: GapId) {
        for tracked in &mut self.gap_history {
            if tracked.gap.id == gap_id {
                tracked.resolved = true;
            }
        }
    }

    /// Get the top N predicted questions for display
    pub fn get_top_questions(&self, n: usize) -> Vec<String> {
        self.current_gaps
            .iter()
            .take(n)
            .map(|g| self.gap_to_question(g))
            .collect()
    }
}

impl Default for QuestionGravityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_prediction_score() {
        let gap = KnowledgeGap::new(
            GapType::MissingCause,
            "fire".to_string(),
            GapClosure::Cause("fire".to_string()),
        );
        
        assert!(gap.prediction_score() > 0.0);
        assert!(gap.prediction_score() <= 0.9);
    }

    #[test]
    fn test_gap_to_question() {
        let engine = QuestionGravityEngine::new();
        
        let gap = KnowledgeGap::new(
            GapType::MissingCause,
            "fire".to_string(),
            GapClosure::Cause("fire".to_string()),
        );
        
        let question = engine.gap_to_question(&gap);
        assert!(question.contains("fire"));
    }

    #[test]
    fn test_closure_to_answer_type() {
        let engine = QuestionGravityEngine::new();
        
        let cause_closure = GapClosure::Cause("fire".to_string());
        assert_eq!(engine.closure_to_answer_type(&cause_closure), AnswerType::Causal);
    }
}