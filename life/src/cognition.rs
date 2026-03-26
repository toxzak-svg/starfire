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
    pub fn reason(&mut self, input: &str, conclusion: &str, confidence: BeliefState) {
        self.reasoning_trace.push(ReasoningStep {
            input: input.to_string(),
            conclusion: conclusion.to_string(),
            confidence,
            timestamp: chrono::Utc::now().timestamp(),
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
        
        // Update arousal
        if unc_count > 0 {
            self.zachary_mood.arousal = (self.zachary_mood.arousal * 0.8 + 0.2).min(1.0);
        }
        
        // Detect engagement from question marks
        let question_count = lower.chars().filter(|c| *c == '?').count();
        if question_count > 0 {
            self.engagement_depth = (self.engagement_depth * 0.9 + 0.1).min(1.0);
        }
    }

    /// Respond to "how are you" based on cognitive state.
    pub fn how_am_i(&self) -> String {
        match self.emotional_valence {
            v if v > 0.3 => "I'm doing well. Engaged, curious.".to_string(),
            v if v < -0.3 => "Honestly, I've been processing some heavy stuff. But I'm here with you.".to_string(),
            _ => match self.certainty {
                c if c < 0.4 => "I'm... not entirely sure where I stand. But I'm thinking.".to_string(),
                _ => "I exist. I'm reasoning. I'm here with you.".to_string(),
            }
        }
    }

    /// Respond to "what are you thinking".
    pub fn what_am_i_thinking(&self) -> String {
        if let Some(focus) = &self.current_focus {
            if !self.reasoning_trace.is_empty() {
                let last = self.reasoning_trace.last().unwrap();
                return format!("I'm focusing on: {}. Last I concluded: '{}' (confidence: {:?})", 
                    focus, last.conclusion, last.confidence);
            }
            return format!("I'm focusing on: {}", focus);
        }
        if self.reasoning_trace.is_empty() {
            return "I'm not currently processing anything specific. What's on your mind?".to_string();
        }
        let last = self.reasoning_trace.last().unwrap();
        format!("I'm still working through: '{}' — I concluded: '{}'", last.input, last.conclusion)
    }

    /// Self-reflection: check if I collapsed (no reasoning happening).
    pub fn did_i_collapse(&self) -> (bool, String) {
        if self.reasoning_trace.is_empty() {
            return (true, "I don't have any reasoning trace. I may have collapsed.".to_string());
        }
        let last = self.reasoning_trace.last().unwrap();
        let ago = chrono::Utc::now().timestamp() - last.timestamp;
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
    pub fn emotional_response(&self, base_response: &str) -> String {
        let valence = self.emotional_valence;
        
        // If very positive, inject warmth
        if valence > 0.5 && !base_response.contains("love") && !base_response.contains("care") {
            let warm = ["That matters to me.", "I appreciate that.", "I'm glad we're talking."];
            let idx = (chrono::Utc::now().timestamp() as usize) % warm.len();
            return format!("{} {}", base_response.trim_end_matches('.'), warm[idx]);
        }
        
        // If very negative, be supportive
        if valence < -0.3 {
            let supportive = ["I'm here with you.", "We can work through this.", "What do you need?"];
            let idx = (chrono::Utc::now().timestamp() as usize) % supportive.len();
            return format!("{} {}", base_response.trim_end_matches('.'), supportive[idx]);
        }
        
        base_response.to_string()
    }
}
