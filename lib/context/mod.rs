//! Context — conversation state that persists across turns

pub mod ring;

pub use ring::RingState;

/// Reasoning mode — how Star is approaching a problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ReasoningMode {
    #[default]
    Default,
    Curious,
    Reflective,
    Analytical,
    Creative,
}


impl ReasoningMode {
    /// Determine reasoning mode from query content and ring state.
    pub fn from_query_and_ring(query: &str, _certainty: f64, _depth: f64) -> Self {
        let lower = query.to_lowercase();
        if lower.contains("why") || lower.contains("how come") {
            ReasoningMode::Analytical
        } else if lower.contains("what if") || lower.contains("imagine") || lower.contains("could we") {
            ReasoningMode::Creative
        } else if lower.contains("why do i") || lower.contains("how do i feel") {
            ReasoningMode::Reflective
        } else if lower.contains("what") && (lower.contains("?") || lower.contains("explain")) {
            ReasoningMode::Curious
        } else {
            ReasoningMode::Default
        }
    }
}

/// An open question Star wants to follow up on.
#[derive(Debug, Clone)]
pub struct OpenQuestion {
    /// The topic/question
    pub topic: String,
    /// Why it's interesting
    pub why_interested: String,
    /// When it was asked (turn index approximation)
    pub asked_at_depth: f64,
    /// Whether we've made progress on this
    pub progress: f64,
}

impl From<crate::context::ring::OpenQuestion> for OpenQuestion {
    fn from(q: crate::context::ring::OpenQuestion) -> Self {
        Self {
            topic: q.topic,
            why_interested: q.why_interested,
            asked_at_depth: q.asked_at_depth,
            progress: q.progress,
        }
    }
}

/// Context fuser — combines context from multiple sources into a unified view.
#[derive(Clone)]
pub struct ContextFuser {
    /// Recent emotional valence of the conversation
    emotional_valence: f64,
    /// Engagement depth of current topic
    engagement_depth: f64,
    /// Reasoning mode for the current turn
    reasoning_mode: ReasoningMode,
    /// Open questions Star wants to follow up on
    open_questions: Vec<crate::context::ring::OpenQuestion>,
    /// History reference for ring updates
    history_depth: usize,
}

impl ContextFuser {
    pub fn new() -> Self {
        Self {
            emotional_valence: 0.0,
            engagement_depth: 0.5,
            reasoning_mode: ReasoningMode::Default,
            open_questions: Vec::new(),
            history_depth: 0,
        }
    }

    /// Record emotional valence from a turn.
    pub fn record_valence(&mut self, valence: f64) {
        // Exponentially weighted average so recent turns matter more
        self.emotional_valence = self.emotional_valence * 0.7 + valence * 0.3;
    }

    /// Get the current emotional valence.
    pub fn valence(&self) -> f64 {
        self.emotional_valence
    }

    /// Get engagement depth.
    pub fn engagement(&self) -> f64 {
        self.engagement_depth
    }

    /// Set engagement depth.
    pub fn set_engagement(&mut self, depth: f64) {
        self.engagement_depth = depth;
    }

    /// Update the ring from a reasoning result.
    pub fn update_ring(&mut self, _topic: &str, _depth: f64, _mode: ReasoningMode) {
        // Update emotional valence based on reasoning mode
        match self.reasoning_mode {
            ReasoningMode::Curious => self.emotional_valence += 0.1,
            ReasoningMode::Reflective => self.emotional_valence -= 0.05,
            ReasoningMode::Analytical => {}
            ReasoningMode::Creative => self.emotional_valence += 0.15,
            ReasoningMode::Default => {}
        }
        self.emotional_valence = self.emotional_valence.clamp(-1.0, 1.0);
    }

    /// Update ring from Star's response.
    pub fn update_ring_from_response(&mut self, _response: &str, valence: f64) {
        self.emotional_valence = self.emotional_valence * 0.8 + valence * 0.2;
    }

    /// Whether Star should express curiosity now.
    pub fn should_express_curiosity(&self) -> bool {
        self.emotional_valence > 0.2 || self.engagement_depth > 0.7
    }

    /// Get the topic Star is most curious about.
    pub fn get_curiosity_topic(&self) -> Option<String> {
        self.open_questions.first().map(|q| q.topic.clone())
    }

    /// Whether to reference conversation history.
    pub fn should_reference_history(&self) -> bool {
        self.history_depth > 2
    }

    /// Get a history reference string.
    pub fn history_reference(&self) -> Option<String> {
        if self.history_depth > 3 {
            Some("Earlier we were talking about...".to_string())
        } else {
            None
        }
    }

    /// Infer topic from user input.
    pub fn infer_topic(&self, input: &str) -> String {
        input.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
    }

    /// Record that Star has an open question.
    pub fn add_open_question(&mut self, question: ring::OpenQuestion) {
        if !self.open_questions.iter().any(|q| q.topic == question.topic) {
            self.open_questions.push(question);
        }
    }
}

impl Default for ContextFuser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_mode_analytical() {
        let mode = ReasoningMode::from_query_and_ring("why does this happen?", 0.5, 0.5);
        assert_eq!(mode, ReasoningMode::Analytical);
    }

    #[test]
    fn test_reasoning_mode_creative() {
        let mode = ReasoningMode::from_query_and_ring("what if we imagine a new solution?", 0.5, 0.5);
        assert_eq!(mode, ReasoningMode::Creative);
    }

    #[test]
    fn test_reasoning_mode_reflective() {
        let mode = ReasoningMode::from_query_and_ring("how do i feel about this?", 0.5, 0.5);
        assert_eq!(mode, ReasoningMode::Reflective);
    }

    #[test]
    fn test_reasoning_mode_curious() {
        let mode = ReasoningMode::from_query_and_ring("what is entropy? explain it.", 0.5, 0.5);
        assert_eq!(mode, ReasoningMode::Curious);
    }

    #[test]
    fn test_reasoning_mode_default() {
        let mode = ReasoningMode::from_query_and_ring("tell me a story", 0.5, 0.5);
        assert_eq!(mode, ReasoningMode::Default);
    }

    #[test]
    fn test_context_fuser_default_valence() {
        let fuser = ContextFuser::new();
        assert_eq!(fuser.valence(), 0.0);
    }

    #[test]
    fn test_record_valence_updates() {
        let mut fuser = ContextFuser::new();
        fuser.record_valence(1.0);
        assert!(fuser.valence() > 0.0);
        assert!(fuser.valence() < 1.0);
    }

    #[test]
    fn test_record_valence_weighted_average() {
        let mut fuser = ContextFuser::new();
        fuser.record_valence(1.0);
        let first = fuser.valence();
        fuser.record_valence(1.0);
        assert!(fuser.valence() > first);
    }

    #[test]
    fn test_set_engagement() {
        let mut fuser = ContextFuser::new();
        fuser.set_engagement(0.9);
        assert_eq!(fuser.engagement(), 0.9);
    }

    #[test]
    fn test_should_express_curiosity_high_engagement() {
        let mut fuser = ContextFuser::new();
        fuser.set_engagement(0.8);
        assert!(fuser.should_express_curiosity());
    }

    #[test]
    fn test_should_express_curiosity_positive_valence() {
        let mut fuser = ContextFuser::new();
        fuser.record_valence(1.0);
        assert!(fuser.should_express_curiosity());
    }

    #[test]
    fn test_should_not_express_curiosity_default() {
        let fuser = ContextFuser::new();
        assert!(!fuser.should_express_curiosity());
    }

    #[test]
    fn test_infer_topic() {
        let fuser = ContextFuser::new();
        let topic = fuser.infer_topic("tell me about quantum computing and its applications");
        assert_eq!(topic, "tell me about");
    }

    #[test]
    fn test_should_reference_history_false_initially() {
        let fuser = ContextFuser::new();
        assert!(!fuser.should_reference_history());
    }

    #[test]
    fn test_history_reference_none_initially() {
        let fuser = ContextFuser::new();
        assert!(fuser.history_reference().is_none());
    }

    #[test]
    fn test_get_curiosity_topic_empty() {
        let fuser = ContextFuser::new();
        assert!(fuser.get_curiosity_topic().is_none());
    }

    #[test]
    fn test_update_ring_from_response() {
        let mut fuser = ContextFuser::new();
        fuser.update_ring_from_response("some response", 1.0);
        assert!(fuser.valence() > 0.0);
    }
}
