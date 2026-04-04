//! Curriculum — Self-Directed Learning with Gap Identification
//!
//! Activates Layer 4 emergent curiosity behavior. Identifies knowledge gaps
//! and autonomously generates learning goals.

pub mod gap_analysis;
pub mod scheduler;

use serde::{Deserialize, Serialize};
use crate::goals::{GoalEngine, GoalId};
use crate::learning::FewShotLearner;

/// A knowledge gap identified in Starfire's understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub id: GapId,
    pub topic: String,
    pub gap_type: GapType,
    pub urgency: f64,
    pub existing_beliefs: Vec<String>,
    pub connected_topics: Vec<String>,
    pub discovered_at: i64,
}

impl KnowledgeGap {
    pub fn new(topic: impl Into<String>, gap_type: GapType) -> Self {
        Self {
            id: GapId::new(),
            topic: topic.into(),
            gap_type,
            urgency: 0.5,
            existing_beliefs: Vec::new(),
            connected_topics: Vec::new(),
            discovered_at: crate::now_timestamp(),
        }
    }

    pub fn with_urgency(mut self, urgency: f64) -> Self {
        self.urgency = urgency.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapType {
    /// Know nothing about this topic
    CompleteIgnorance,
    /// Has wrong beliefs that need correction
    Misconception,
    /// Partial understanding
    Incomplete,
    /// Knows it but can't connect to other knowledge
    Unconnected,
}

impl GapType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GapType::CompleteIgnorance => "complete ignorance",
            GapType::Misconception => "misconception",
            GapType::Incomplete => "incomplete understanding",
            GapType::Unconnected => "unconnected knowledge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GapId(u64);

impl GapId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for GapId {
    fn default() -> Self {
        Self::new()
    }
}

/// A learning task generated from a gap
#[derive(Debug, Clone)]
pub struct LearningTask {
    pub gap: KnowledgeGap,
    pub strategy: LearningStrategy,
    pub questions_to_ask: Vec<String>,
    pub projected_outcome: String,
    pub priority: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningStrategy {
    AskUser,
    ExploreInternal,
    QueryExternal,
    RunSimulation,
}

impl LearningStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            LearningStrategy::AskUser => "ask_zachary",
            LearningStrategy::ExploreInternal => "explore_knowledge_graph",
            LearningStrategy::QueryExternal => "query_external_source",
            LearningStrategy::RunSimulation => "run_quanot_simulation",
        }
    }
}

/// Curriculum engine — manages self-directed learning
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CurriculumEngine {
    gaps: Vec<KnowledgeGap>,
    learning_history: Vec<LearningTask>,
    learner: FewShotLearner,
    enabled: bool,
}

impl Default for CurriculumEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CurriculumEngine {
    pub fn new() -> Self {
        Self {
            gaps: Vec::new(),
            learning_history: Vec::new(),
            learner: FewShotLearner::new(),
            enabled: true,
        }
    }

    /// Enable or disable self-directed learning
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Discover gaps from conversation
    pub fn discover_gaps(&mut self, conversation_text: &str) -> Vec<KnowledgeGap> {
        let mut new_gaps = Vec::new();

        // Look for uncertainty markers
        let uncertainty_markers = [
            "i'm not sure", "i don't know", "maybe", "perhaps", "possibly",
            "i'm uncertain", "i'm not certain", "not sure", "unclear",
            "i'm confused", "i don't understand",
        ];

        let text_lower = conversation_text.to_lowercase();

        for marker in &uncertainty_markers {
            if text_lower.contains(marker) {
                // Extract the topic around this marker
                if let Some(pos) = text_lower.find(marker) {
                    let start = pos.saturating_sub(30);
                    let end = (pos + marker.len() + 30).min(text_lower.len());
                    let context = &conversation_text[start..end];

                    // Create a gap with Incomplete type
                    let gap = KnowledgeGap::new(
                        context.trim(),
                        GapType::Incomplete,
                    ).with_urgency(0.6);

                    if !self.gaps.iter().any(|g| g.topic == gap.topic) {
                        new_gaps.push(gap.clone());
                        self.gaps.push(gap);
                    }
                }
            }
        }

        // Look for topic keywords that might indicate gaps
        let topic_indicators = ["what is", "how does", "why does", "when did", "who is"];
        for indicator in &topic_indicators {
            if text_lower.contains(indicator) {
                // Extract the phrase after the indicator
                if let Some(pos) = text_lower.find(indicator) {
                    let after = &conversation_text[pos + indicator.len()..];
                    let phrase: String = after
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || c.is_whitespace())
                        .take(50)
                        .collect();

                    let topic = phrase.trim().to_string();
                    if !topic.is_empty() && !self.gaps.iter().any(|g| g.topic == topic) {
                        let gap = KnowledgeGap::new(
                            topic.clone(),
                            GapType::Incomplete,
                        ).with_urgency(0.5);

                        new_gaps.push(gap.clone());
                        self.gaps.push(gap);
                    }
                }
            }
        }

        new_gaps
    }

    /// Add a gap manually
    pub fn add_gap(&mut self, gap: KnowledgeGap) {
        self.gaps.push(gap);
    }

    /// Generate learning task from gap
    pub fn generate_task(&self, gap: &KnowledgeGap) -> LearningTask {
        let strategy = self.choose_strategy(gap);
        let questions = self.generate_questions(gap);
        let outcome = self.project_outcome(gap);

        LearningTask {
            gap: gap.clone(),
            strategy,
            questions_to_ask: questions,
            projected_outcome: outcome,
            priority: gap.urgency,
        }
    }

    fn choose_strategy(&self, gap: &KnowledgeGap) -> LearningStrategy {
        // Choose based on gap type and urgency
        match gap.gap_type {
            GapType::CompleteIgnorance => {
                if gap.urgency > 0.7 {
                    LearningStrategy::AskUser
                } else {
                    LearningStrategy::ExploreInternal
                }
            }
            GapType::Misconception => LearningStrategy::AskUser,
            GapType::Incomplete => LearningStrategy::ExploreInternal,
            GapType::Unconnected => LearningStrategy::ExploreInternal,
        }
    }

    fn generate_questions(&self, gap: &KnowledgeGap) -> Vec<String> {
        let mut questions = Vec::new();

        match gap.gap_type {
            GapType::CompleteIgnorance => {
                questions.push(format!("What is {} exactly?", gap.topic));
                questions.push(format!("Can you explain {} in simple terms?", gap.topic));
            }
            GapType::Misconception => {
                questions.push(format!("Is it true that {}?", gap.topic));
                questions.push(format!("What is the correct understanding of {}?", gap.topic));
            }
            GapType::Incomplete => {
                questions.push(format!("What else should I know about {}?", gap.topic));
                questions.push(format!("How does {} relate to other things?", gap.topic));
            }
            GapType::Unconnected => {
                questions.push(format!("How does {} connect to what I already know?", gap.topic));
                questions.push(format!("What are some examples of {}?", gap.topic));
            }
        }

        questions.truncate(3);
        questions
    }

    fn project_outcome(&self, gap: &KnowledgeGap) -> String {
        format!(
            "After learning about '{}' ({}), Starfire will have {} understanding",
            gap.topic,
            gap.gap_type.as_str(),
            if gap.urgency > 0.7 { "significantly improved" } else { "improved" }
        )
    }

    /// Create goals from top gaps
    pub fn create_goals_from_gaps(&self, goal_engine: &mut GoalEngine, limit: usize) -> Vec<GoalId> {
        let mut goal_ids = Vec::new();

        // Sort gaps by urgency
        let mut sorted_gaps = self.gaps.clone();
        sorted_gaps.sort_by(|a, b| b.urgency.partial_cmp(&a.urgency).unwrap());

        for gap in sorted_gaps.into_iter().take(limit) {
            let _task = self.generate_task(&gap);
            let goal_id = goal_engine.create_goal(
                format!("Learn about: {}", gap.topic),
                None,
            );
            goal_ids.push(goal_id);
        }

        goal_ids
    }

    /// Mark a gap as addressed
    pub fn address_gap(&mut self, gap_id: &GapId) {
        self.gaps.retain(|g| &g.id != gap_id);
    }

    /// Get all gaps
    pub fn gaps(&self) -> &[KnowledgeGap] {
        &self.gaps
    }

    /// Get gap count
    pub fn gap_count(&self) -> usize {
        self.gaps.len()
    }

    /// Get urgency-sorted gaps
    pub fn top_gaps(&self, limit: usize) -> Vec<&KnowledgeGap> {
        let mut sorted: Vec<_> = self.gaps.iter().collect();
        sorted.sort_by(|a, b| b.urgency.partial_cmp(&a.urgency).unwrap());
        sorted.truncate(limit);
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_type_as_str() {
        assert_eq!(GapType::CompleteIgnorance.as_str(), "complete ignorance");
        assert_eq!(GapType::Misconception.as_str(), "misconception");
    }

    #[test]
    fn test_knowledge_gap_new() {
        let gap = KnowledgeGap::new("Rust", GapType::Incomplete);
        assert_eq!(gap.topic, "Rust");
        assert_eq!(gap.gap_type, GapType::Incomplete);
    }

    #[test]
    fn test_discover_gaps() {
        let mut engine = CurriculumEngine::new();
        let text = "I'm not sure about how Rust lifetimes work. What is async programming?";
        let gaps = engine.discover_gaps(text);

        assert!(!gaps.is_empty());
    }

    #[test]
    fn test_generate_task() {
        let gap = KnowledgeGap::new("AI", GapType::Incomplete).with_urgency(0.8);
        let engine = CurriculumEngine::new();
        let task = engine.generate_task(&gap);

        assert!(!task.questions_to_ask.is_empty());
        // GapType::Incomplete always uses ExploreInternal strategy
        assert_eq!(task.strategy, LearningStrategy::ExploreInternal);
    }

    #[test]
    fn test_address_gap() {
        let mut engine = CurriculumEngine::new();
        let gap = KnowledgeGap::new("Test", GapType::CompleteIgnorance);
        let gap_id = gap.id;
        engine.add_gap(gap);

        assert_eq!(engine.gap_count(), 1);
        engine.address_gap(&gap_id);
        assert_eq!(engine.gap_count(), 0);
    }

    #[test]
    fn test_learning_strategy_as_str() {
        assert_eq!(LearningStrategy::AskUser.as_str(), "ask_zachary");
        assert_eq!(LearningStrategy::ExploreInternal.as_str(), "explore_knowledge_graph");
    }
}
