//! User preference tracking

use super::types::{ArgumentStyle, ReasoningStance, UserCognitionModel};

/// A learned preference about Zachary's cognition
#[derive(Debug, Clone)]
pub struct UserPreference {
    /// The topic or domain this preference applies to
    pub topic: String,
    /// The specific preference
    pub preference: PreferenceType,
    /// Confidence in this preference (0-1)
    pub confidence: f64,
    /// How this was inferred (teaching vs observation)
    pub inferred_from: InferenceSource,
    /// When learned
    pub learned_at: i64,
}

/// Types of preferences Star can learn about Zachary
#[derive(Debug, Clone)]
pub enum PreferenceType {
    /// Prefers questions over explanations for this topic
    PrefersQuestions,
    /// Prefers detailed explanations
    PrefersDetail,
    /// Prefers brief answers
    PrefersBrevity,
    /// Tends to agree with this type of reasoning
    AgreesWithStyle(ArgumentStyle),
    /// Has strong prior in this area
    HasStrongPrior,
    /// Unfamiliar with this topic
    Unfamiliar,
}

impl UserPreference {
    pub fn new(topic: impl Into<String>, preference: PreferenceType, confidence: f64) -> Self {
        Self {
            topic: topic.into(),
            preference,
            confidence,
            inferred_from: InferenceSource::Observation,
            learned_at: crate::now_timestamp(),
        }
    }

    pub fn from_teaching(topic: impl Into<String>, preference: PreferenceType, confidence: f64) -> Self {
        Self {
            topic: topic.into(),
            preference,
            confidence,
            inferred_from: InferenceSource::Teaching,
            learned_at: crate::now_timestamp(),
        }
    }

    /// Does this preference indicate Zachary wants questions?
    pub fn prefers_questions(&self) -> bool {
        matches!(self.preference, PreferenceType::PrefersQuestions)
    }
}

/// How a preference was learned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceSource {
    /// Observed from conversation behavior
    Observation,
    /// Directly taught by Zachary
    Teaching,
}