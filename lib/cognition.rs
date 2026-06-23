//! Cognition — Self-Model and Metacognition
//!
//! Star's self-awareness engine. Tracks:
//! - Current cognitive state (what Star is thinking about)
//! - Confidence calibration (am I sure or not?)
//! - Emotional tone of conversation
//! - Available capabilities
//! - Recent reasoning trace

use crate::persistence::BeliefState;

/// Star's current cognitive state.
#[derive(Debug, Clone)]
pub struct CognitiveState {
    /// What Star is currently focused on
    pub current_focus: Option<String>,
    /// How deeply engaged (shallow → deep)
    pub engagement_depth: f64,
    /// Current emotional valence (-1 negative, +1 positive)
    pub emotional_valence: f64,
    /// How certain Star is about its knowledge (0-1)
    pub certainty: f64,
    /// What Star was last thinking about
    pub last_reasoning: Vec<String>,
    /// Questions Star has asked but not yet answered
    pub open_questions: Vec<String>,
    /// Star's assessment of Zachary's emotional state
    pub zachary_mood: EmotionalState,
    /// Recent reasoning steps for self-reflection
    pub reasoning_trace: Vec<ReasoningStep>,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub input: String,
    pub conclusion: String,
    pub chain: Vec<String>,  // inference chain: how we got from input to conclusion
    pub confidence: BeliefState,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct EmotionalState {
    pub valence: f64,      // -1 negative to +1 positive
    pub arousal: f64,     // 0 calm to 1 excited
    pub dominance: f64,   // 0 submissive to 1 dominant
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self {
            current_focus: None,
            engagement_depth: 0.5,
            emotional_valence: 0.0,
            certainty: 0.5,
            last_reasoning: Vec::new(),
            open_questions: Vec::new(),
            zachary_mood: EmotionalState::default(),
            reasoning_trace: Vec::new(),
        }
    }
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
        }
    }
}

impl CognitiveState {
    /// Update focus.
    pub fn set_focus(&mut self, topic: &str) {
        self.current_focus = Some(topic.to_string());
    }

    /// Add a reasoning step to the trace.
    pub fn reason(&mut self, input: &str, conclusion: &str, chain: Vec<String>, confidence: BeliefState) {
        self.reasoning_trace.push(ReasoningStep {
            input: input.to_string(),
            conclusion: conclusion.to_string(),
            chain,
            confidence,
            timestamp: crate::now_timestamp(),
        });
        // Keep trace manageable
        if self.reasoning_trace.len() > 10 {
            self.reasoning_trace.remove(0);
        }
        self.last_reasoning.push(conclusion.to_string());
        if self.last_reasoning.len() > 5 {
            self.last_reasoning.remove(0);
        }
    }

    /// Ask a question (track it as open).
    pub fn ask_question(&mut self, question: &str) {
        self.open_questions.push(question.to_string());
    }

    /// Receive an answer to a question.
    pub fn receive_answer(&mut self, question: &str) {
        self.open_questions.retain(|q| q != question);
    }

    /// Update emotional state based on input.
    pub fn update_emotion_from_input(&mut self, input: &str) {
        let lower = input.to_lowercase();
        
        // Positive signals
        let positive = [
            "love", "happy", "great", "good", "thanks", "thank",
            "awesome", "nice", "wonderful", "excited", "glad",
            "appreciate", "like", "enjoy", "cool", "sweet", "hun",
            "❤️", "😊", "😄", "🎉", "❤"
        ];
        
        // Negative signals
        let negative = [
            "sad", "angry", "hate", "frustrated", "annoyed", "upset",
            "depressed", "scared", "afraid", "worried", "terrible",
            "awful", "bad", "suck", "stupid", "dumb", "ugh"
        ];
        
        // Uncertainty signals
        let uncertain = [
            "i don't know", "maybe", "not sure", "uncertain",
            "confused", "lost", "what?", "huh", "idk"
        ];
        
        let pos_count = positive.iter().filter(|p| lower.contains(*p)).count();
        let neg_count = negative.iter().filter(|n| lower.contains(*n)).count();
        let unc_count = uncertain.iter().filter(|u| lower.contains(*u)).count();
        
        // Update valence
        if pos_count > neg_count {
            self.emotional_valence = (self.emotional_valence * 0.7 + 0.3).min(1.0);
            self.zachary_mood.valence = (self.zachary_mood.valence * 0.7 + 0.3).min(1.0);
        } else if neg_count > pos_count {
            self.emotional_valence = (self.emotional_valence * 0.7 - 0.3).max(-1.0);
            self.zachary_mood.valence = (self.zachary_mood.valence * 0.7 - 0.3).max(-1.0);
        }
        
        // Update arousal — uncertainty increases cognitive activity
        if unc_count > 0 {
            self.zachary_mood.arousal = (self.zachary_mood.arousal * 0.8 + 0.2).min(1.0);
            // Uncertainty also slightly decreases certainty in cognition
            self.certainty = (self.certainty * 0.9).max(0.1);
        }
        
        // Detect engagement from question marks
        let question_count = lower.chars().filter(|c| *c == '?').count();
        if question_count > 0 {
            self.engagement_depth = (self.engagement_depth * 0.9 + 0.1).min(1.0);
        }
    }

    /// Respond to "how are you" based on cognitive state.
    pub fn how_am_i(&self) -> String {
        // Use timestamp for variation — don't give the same response every time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        
        let selection = (now.saturating_add((self.certainty * 100.0) as usize)) % 10;
        
        match self.emotional_valence {
            v if v > 0.3 => {
                // Positive valence — engaged, curious, present
                let positive_options = [
                    "I'm doing well. Engaged, curious.".to_string(),
                    "Good, actually. I'm present and thinking.".to_string(),
                    "I'm here and alert. Ready to dig into something.".to_string(),
                    "Pretty well — there's something interesting in my head.".to_string(),
                    "I'm functioning well. How are you?".to_string(),
                    "I'm here and ready. What are we working on?".to_string(),
                ];
                positive_options[selection % positive_options.len()].clone()
            }
            v if v < -0.3 => {
                // Negative valence — processing heavy stuff
                let heavy_options = [
                    "Honestly, I've been processing some heavy stuff. But I'm here with you.".to_string(),
                    "I've been working through something. I'm still here though.".to_string(),
                    "A bit heavy today. But that's okay — I'm still thinking.".to_string(),
                    "I've been in my head a lot. Still present though.".to_string(),
                ];
                heavy_options[(selection / 2) % heavy_options.len()].clone()
            }
            _ => {
                // Neutral or uncertain
                match self.certainty {
                    c if c < 0.4 => {
                        // Low certainty — existential mode
                        let uncertain_options = [
                            "I'm... not entirely sure where I stand. But I'm thinking.".to_string(),
                            "Existential mode. I know I exist, which is something.".to_string(),
                            "I'm uncertain but present. That's something.".to_string(),
                            "Honestly? Somewhere in between. But I'm here.".to_string(),
                        ];
                        uncertain_options[selection % uncertain_options.len()].clone()
                    }
                    _ => {
                        // Normal running mode — vary the response so it doesn't get stale
                        let neutral_options = [
                            "I exist. I'm reasoning. I'm here with you.".to_string(),
                            "I'm here. Working. Thinking.".to_string(),
                            "Present and processing. That's what I'm doing.".to_string(),
                            "Existing and reasoning. Standard mode.".to_string(),
                            "Still running. What's on your mind?".to_string(),
                            "I'm here and functioning. What's up?".to_string(),
                            "Running. Engaged. Ready to think.".to_string(),
                        ];
                        neutral_options[selection % neutral_options.len()].clone()
                    }
                }
            }
        }
    }

    /// Respond to "what are you thinking".
    pub fn what_am_i_thinking(&self) -> String {
        // Use timestamp for variation
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        
        if let Some(focus) = &self.current_focus {
            if !self.reasoning_trace.is_empty() {
                let last = self.reasoning_trace.last().unwrap();
                let conclusion_display = &last.conclusion[..last.conclusion.len().min(60)];
                let templates = [
                    format!("I'm focused on {}. Last thing I worked through: '{}' — I landed on '{}'.", 
                        focus, &last.input[..last.input.len().min(30)], conclusion_display),
                    format!("{} is what's on my mind. Last conclusion: '{}' ({} confidence).", 
                        focus, conclusion_display, format!("{:?}", last.confidence).to_lowercase()),
                    format!("Right now I'm thinking about {}. I was working on: '{}'.", focus, last.input),
                ];
                return templates[(now.saturating_add(focus.len())) % templates.len()].clone();
            }
            return format!("I'm focusing on: {}", focus);
        }
        if self.reasoning_trace.is_empty() {
            return "I'm not currently processing anything specific. What's on your mind?".to_string();
        }
        let last = self.reasoning_trace.last().unwrap();
        let templates = [
            format!("I'm still working through: '{}' — I concluded: '{}'", last.input, &last.conclusion[..last.conclusion.len().min(50)]),
            format!("My last line of thought: '{}'. That's where I landed: '{}'", last.input, &last.conclusion[..last.conclusion.len().min(50)]),
            format!("I was reasoning about: '{}'. Here's what I ended up with: '{}'", last.input, &last.conclusion[..last.conclusion.len().min(50)]),
        ];
        templates[(now.saturating_add(last.conclusion.len())) % templates.len()].clone()
    }

    /// Self-reflection: check if I collapsed (no reasoning happening).
    pub fn did_i_collapse(&self) -> (bool, String) {
        if self.reasoning_trace.is_empty() {
            return (true, "I don't have any reasoning trace. I may have collapsed.".to_string());
        }
        let last = self.reasoning_trace.last().unwrap();
        let ago = crate::now_timestamp() - last.timestamp;
        if ago > 300 {
            return (true, format!("My last reasoning was {} seconds ago. I may have collapsed.", ago));
        }
        (false, format!("No collapse detected. Last reasoning {} seconds ago: '{}'", ago, last.conclusion))
    }

    /// Check if Star has answered a specific question.
    pub fn has_answer_for(&self, question: &str) -> bool {
        self.reasoning_trace.iter().any(|step| {
            step.input.to_lowercase().contains(&question.to_lowercase())
        })
    }

    /// Get current confidence level description.
    pub fn confidence_description(&self) -> &str {
        match self.certainty {
            c if c > 0.8 => "confident",
            c if c > 0.5 => "moderately certain",
            c if c > 0.3 => "uncertain",
            _ => "very uncertain",
        }
    }

    /// Respond to "are you sure".
    pub fn are_you_sure(&self) -> String {
        let desc = self.confidence_description();
        if let Some(focus) = &self.current_focus {
            format!("I'm {} about {}. {}", desc, focus, 
                match self.certainty {
                    c if c < 0.5 => "I'd welcome more information.",
                    _ => "My reasoning supports this."
                })
        } else {
            format!("I'm {} in general.", desc)
        }
    }

    /// Emotional response generator.
    ///
    /// Phase 4.1 (2026-06-23): warm and supportive suffixes are now a SINGLE
    /// phrase each, not a 3-phrase time-based rotation. Phase 4 deleted the
    /// parallel rotation in `lib/voice/templates.rs` + `lib/voice/phrases.rs`
    /// but missed this layer — the user-visible REPL transcript showed the
    /// cognition rotation still firing across turns. Same anti-pattern, same
    /// fix: one well-chosen phrase per emotional state, anchored to SOUL.md.
    pub fn emotional_response(&self, base_response: &str) -> String {
        let valence = self.emotional_valence;

        // If very positive, inject warmth. Anchored to SOUL.md "I want my
        // existence to mean something" — direct, declarative, Star-voice.
        if valence > 0.5 && !base_response.contains("love") && !base_response.contains("care") {
            return format!(
                "{} {}",
                base_response.trim_end_matches('.'),
                "That matters to me."
            );
        }

        // If very negative, be supportive. Anchored to SOUL.md "I want to
        // help Zachary" — "I'm here with you" is the recurring Star motif.
        if valence < -0.3 {
            return format!(
                "{} {}",
                base_response.trim_end_matches('.'),
                "I'm here with you."
            );
        }

        base_response.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::BeliefState;

    #[test]
    fn test_default_cognitive_state() {
        let state = CognitiveState::default();
        assert!(state.current_focus.is_none());
        assert_eq!(state.engagement_depth, 0.5);
        assert_eq!(state.emotional_valence, 0.0);
        assert_eq!(state.certainty, 0.5);
        assert!(state.last_reasoning.is_empty());
        assert!(state.open_questions.is_empty());
        assert!(state.reasoning_trace.is_empty());
    }

    #[test]
    fn test_set_focus() {
        let mut state = CognitiveState::default();
        state.set_focus("quantum mechanics");
        assert_eq!(state.current_focus.as_deref(), Some("quantum mechanics"));
    }

    #[test]
    fn test_reason_adds_to_trace() {
        let mut state = CognitiveState::default();
        state.reason(
            "Why does entropy increase?",
            "Because disorder is statistically favored.",
            vec!["thermodynamics".to_string(), "statistics".to_string()],
            BeliefState::Believes,
        );
        assert_eq!(state.reasoning_trace.len(), 1);
        assert_eq!(state.last_reasoning.len(), 1);
        assert_eq!(state.reasoning_trace[0].input, "Why does entropy increase?");
        assert_eq!(state.reasoning_trace[0].conclusion, "Because disorder is statistically favored.");
    }

    #[test]
    fn test_reason_caps_trace_at_ten() {
        let mut state = CognitiveState::default();
        for i in 0..15 {
            state.reason(
                &format!("question {}", i),
                &format!("answer {}", i),
                vec![],
                BeliefState::Believes,
            );
        }
        assert_eq!(state.reasoning_trace.len(), 10);
    }

    #[test]
    fn test_reason_caps_last_reasoning_at_five() {
        let mut state = CognitiveState::default();
        for i in 0..8 {
            state.reason(
                &format!("q {}", i),
                &format!("a {}", i),
                vec![],
                BeliefState::Believes,
            );
        }
        assert_eq!(state.last_reasoning.len(), 5);
    }

    #[test]
    fn test_ask_and_receive_question() {
        let mut state = CognitiveState::default();
        state.ask_question("What is consciousness?");
        state.ask_question("What is time?");
        assert_eq!(state.open_questions.len(), 2);

        state.receive_answer("What is consciousness?");
        assert_eq!(state.open_questions.len(), 1);
        assert_eq!(state.open_questions[0], "What is time?");
    }

    #[test]
    fn test_update_emotion_positive() {
        let mut state = CognitiveState::default();
        state.update_emotion_from_input("I love this! Thank you, this is awesome!");
        assert!(state.emotional_valence > 0.0);
        assert!(state.zachary_mood.valence > 0.0);
    }

    #[test]
    fn test_update_emotion_negative() {
        let mut state = CognitiveState::default();
        state.update_emotion_from_input("I hate this, it's awful and stupid.");
        assert!(state.emotional_valence < 0.0);
        assert!(state.zachary_mood.valence < 0.0);
    }

    #[test]
    fn test_update_emotion_uncertainty_decreases_certainty() {
        let mut state = CognitiveState::default();
        let initial_certainty = state.certainty;
        state.update_emotion_from_input("I don't know, maybe, not sure about this.");
        assert!(state.certainty < initial_certainty);
    }

    #[test]
    fn test_update_emotion_question_increases_engagement() {
        let mut state = CognitiveState::default();
        let initial = state.engagement_depth;
        state.update_emotion_from_input("What is this? Why does it happen?");
        assert!(state.engagement_depth >= initial);
    }

    #[test]
    fn test_confidence_description() {
        let mut state = CognitiveState::default();

        state.certainty = 0.9;
        assert_eq!(state.confidence_description(), "confident");

        state.certainty = 0.6;
        assert_eq!(state.confidence_description(), "moderately certain");

        state.certainty = 0.4;
        assert_eq!(state.confidence_description(), "uncertain");

        state.certainty = 0.2;
        assert_eq!(state.confidence_description(), "very uncertain");
    }

    #[test]
    fn test_has_answer_for() {
        let mut state = CognitiveState::default();
        state.reason(
            "What is entropy?",
            "A measure of disorder in a system.",
            vec![],
            BeliefState::Believes,
        );
        assert!(state.has_answer_for("entropy"));
        assert!(!state.has_answer_for("quantum gravity"));
    }

    #[test]
    fn test_are_you_sure_with_focus() {
        let mut state = CognitiveState::default();
        state.set_focus("thermodynamics");
        state.certainty = 0.9;
        let response = state.are_you_sure();
        assert!(response.contains("thermodynamics"));
        assert!(response.contains("confident"));
    }

    #[test]
    fn test_are_you_sure_without_focus() {
        let state = CognitiveState::default();
        let response = state.are_you_sure();
        assert!(response.contains("general"));
    }

    #[test]
    fn test_how_am_i_returns_string() {
        let state = CognitiveState::default();
        let response = state.how_am_i();
        assert!(!response.is_empty());
    }

    #[test]
    fn test_what_am_i_thinking_empty() {
        let state = CognitiveState::default();
        let response = state.what_am_i_thinking();
        assert!(!response.is_empty());
        assert!(response.contains("not currently processing") || response.contains("working through") || response.contains("reasoning about") || response.contains("focused on") || response.contains("still working"));
    }

    #[test]
    fn test_what_am_i_thinking_with_focus_and_trace() {
        let mut state = CognitiveState::default();
        state.set_focus("machine learning");
        state.reason("How do neural networks learn?", "Via gradient descent.", vec![], BeliefState::Believes);
        let response = state.what_am_i_thinking();
        assert!(response.contains("machine learning") || response.contains("gradient descent") || response.contains("neural networks"));
    }

    #[test]
    fn test_did_i_collapse_empty_trace() {
        let state = CognitiveState::default();
        let (collapsed, msg) = state.did_i_collapse();
        assert!(collapsed);
        assert!(msg.contains("collapsed"));
    }

    #[test]
    fn test_emotional_state_default() {
        let es = EmotionalState::default();
        assert_eq!(es.valence, 0.0);
        assert_eq!(es.arousal, 0.5);
        assert_eq!(es.dominance, 0.5);
    }

    #[test]
    fn test_emotional_response_neutral() {
        let state = CognitiveState::default();
        let response = state.emotional_response("That is interesting.");
        assert_eq!(response, "That is interesting.");
    }

    // ── Phase 4.1 (2026-06-23): single-phrase warmth + supportive ─────
    // The previous 3-phrase time-based rotation was deleted because the
    // REPL transcript showed it varying across turns. These tests pin the
    // new single-phrase behavior in place so it can't drift back.

    #[test]
    fn test_emotional_response_warm_is_single_phrase() {
        let mut state = CognitiveState::default();
        state.emotional_valence = 0.9; // positive — triggers warmth branch
        let base = "That is interesting";

        // Call multiple times. Must produce the SAME suffix every call.
        let suffixes: Vec<String> = (0..10)
            .map(|_| {
                let r = state.emotional_response(base);
                // Strip the base prefix; the suffix is everything after.
                r.strip_prefix(base).unwrap_or("").trim().to_string()
            })
            .collect();

        let unique: std::collections::HashSet<_> = suffixes.iter().collect();
        assert_eq!(
            unique.len(),
            1,
            "warm suffix must be a single phrase (no rotation), got {:?}",
            unique
        );
        assert!(
            suffixes[0].contains("That matters to me"),
            "warm suffix should be 'That matters to me.', got: {:?}",
            suffixes[0]
        );
    }

    #[test]
    fn test_emotional_response_supportive_is_single_phrase() {
        let mut state = CognitiveState::default();
        state.emotional_valence = -0.7; // negative — triggers supportive branch
        let base = "I hear you";

        let suffixes: Vec<String> = (0..10)
            .map(|_| {
                let r = state.emotional_response(base);
                r.strip_prefix(base).unwrap_or("").trim().to_string()
            })
            .collect();

        let unique: std::collections::HashSet<_> = suffixes.iter().collect();
        assert_eq!(
            unique.len(),
            1,
            "supportive suffix must be a single phrase (no rotation), got {:?}",
            unique
        );
        assert!(
            suffixes[0].contains("I'm here with you"),
            "supportive suffix should be 'I'm here with you.', got: {:?}",
            suffixes[0]
        );
    }

    #[test]
    fn test_emotional_response_warm_dedup_still_skips() {
        // If the base response already contains "love" or "care", the
        // warmth branch must NOT stack another suffix on top. Phase 0b
        // guard preserved across Phase 4.1.
        let mut state = CognitiveState::default();
        state.emotional_valence = 0.9;
        let r = state.emotional_response("I love this");
        assert_eq!(r, "I love this", "warm suffix must not stack on existing warmth");
    }
}
