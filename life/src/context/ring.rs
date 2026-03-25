//! Symbolic Ring Attractor
//!
//! A structured context record that persists across conversation turns.
//! Unlike a neural ring (continuous activation dynamics), this is a symbolic ring:
//! discrete states connected by reasoning transitions.
//!
//! The "ring" has a 2D structure metaphorically modeled after cortical rings:
//! - Angle (topic_phase): which topic/concept we're focused on
//! - Radial (depth): how deeply we've explored this topic
//!
//! Continuity: the ring state from turn N influences turn N+1.
//! This gives Star the "thread" it needs to maintain coherent conversation.

use std::collections::HashMap;

/// The persistent ring state — survives across conversation turns.
///
/// This is Star's "working memory" for conversation context.
/// Unlike the conversation layer's message history, this is a *structured*
/// representation of what Star was thinking about.
#[derive(Debug, Clone)]
pub struct RingState {
    /// Current topic focus (the "angle" on the ring)
    pub topic_phase: TopicPhase,
    /// How deeply we've explored the current topic (radial depth)
    pub depth: f64,
    /// Open questions — things Star wants to follow up on
    pub open_questions: Vec<OpenQuestion>,
    /// Topic history — what we've discussed, in order
    pub topic_history: Vec<String>,
    /// Topic depth map — how deep we went on each topic
    topic_depths: HashMap<String, f64>,
    /// Transitions — how we got from topic to topic
    transitions: Vec<TopicTransition>,
    /// What Star was last curious about
    pub last_curiosity: Option<String>,
    /// Current certainty about the topic (0-1)
    /// Higher = more certain, lower = more uncertain / exploring
    pub certainty: f64,
}

impl RingState {
    pub fn new() -> Self {
        Self {
            topic_phase: TopicPhase::Unfocused,
            depth: 0.0,
            open_questions: Vec::new(),
            topic_history: Vec::new(),
            topic_depths: HashMap::new(),
            transitions: Vec::new(),
            last_curiosity: None,
            certainty: 0.5,
        }
    }

    /// Update the ring based on a new user query.
    /// This is the "attractor dynamics" — the query pulls the ring toward a new phase.
    pub fn update_from_query(&mut self, query: &str, inferred_topic: &str) {
        let topic_lower = inferred_topic.to_lowercase();
        
        // Check if this is a new topic or continuation
        let is_new_topic = match &self.topic_phase {
            TopicPhase::Focused(t) => !t.to_lowercase().contains(&topic_lower) 
                && topic_lower.len() > 3,
            TopicPhase::Unfocused => true,
            TopicPhase::Transitioning(from, _) => {
                !from.to_lowercase().contains(&topic_lower)
            }
        };

        if is_new_topic {
            // Store previous topic depth
            if let TopicPhase::Focused(prev) = &self.topic_phase {
                if !prev.is_empty() {
                    *self.topic_depths.entry(prev.clone()).or_insert(0.0) = self.depth;
                    self.transitions.push(TopicTransition {
                        from: prev.clone(),
                        to: inferred_topic.to_string(),
                        depth_at_transition: self.depth,
                    });
                }
            }
            
            // Shift to new topic
            self.topic_phase = TopicPhase::Focused(inferred_topic.to_string());
            if !self.topic_history.iter().any(|t| t.to_lowercase() == topic_lower) {
                self.topic_history.push(inferred_topic.to_string());
            }
            self.depth = 0.1; // Start shallow on new topic
            
            // Reduce certainty on new topic
            self.certainty = (self.certainty * 0.7).max(0.2);
        } else {
            // Same topic — go deeper
            self.depth = (self.depth + 0.15).min(1.0);
            
            // Increase certainty if we've been on topic for a while
            self.certainty = (self.certainty + 0.05).min(0.95);
        }
    }

    /// Update the ring after Star produces a response.
    pub fn update_from_response(&mut self, response: &str, mode: super::ReasoningMode) {
        // Did we express uncertainty? Reduce certainty slightly
        let uncertain_markers = ["i don't know", "i'm not sure", "i'm uncertain", 
                                 "i'm not certain", "i don't have enough information"];
        let lower = response.to_lowercase();
        if uncertain_markers.iter().any(|m| lower.contains(m)) {
            self.certainty = (self.certainty - 0.1).max(0.1);
        }
        
        // Did we express a strong opinion? Increase certainty
        let assertive_markers = ["i believe", "i think", "i'm sure", "i know", 
                                  "it's clear that", "the evidence suggests"];
        if assertive_markers.iter().any(|m| lower.contains(m)) {
            self.certainty = (self.certainty + 0.05).min(0.95);
        }
        
        // Did we express curiosity? Update curiosity cursor
        if lower.contains("i wonder") || lower.contains("i'm curious") || lower.contains("what is") {
            // Extract the thing we're curious about
            if let Some(wonder_start) = lower.find("i wonder") {
                let slice = &response[wonder_start..];
                let words: Vec<&str> = slice.split_whitespace().take(10).collect();
                if words.len() >= 3 {
                    self.last_curiosity = Some(words[2..].join(" "));
                }
            }
        }
    }

    /// Push a new open question.
    pub fn push_question(&mut self, question: OpenQuestion) {
        // Don't add duplicate topics
        if !self.open_questions.iter().any(|q| q.topic == question.topic) {
            self.open_questions.push(question);
        }
    }

    /// Mark a question as answered (remove it from open questions).
    pub fn answer_question(&mut self, question_topic: &str) {
        self.open_questions.retain(|q| !q.topic.to_lowercase().contains(&question_topic.to_lowercase()));
    }

    /// Get all currently open questions.
    pub fn open_questions(&self) -> &[OpenQuestion] {
        &self.open_questions
    }

    /// Get the current topic string.
    pub fn current_topic(&self) -> String {
        match &self.topic_phase {
            TopicPhase::Focused(t) => t.clone(),
            TopicPhase::Unfocused => "general".to_string(),
            TopicPhase::Transitioning(from, to) => format!("{} → {}", from, to),
        }
    }

    /// Get how deep we went on a previous topic.
    pub fn depth_on_topic(&self, topic: &str) -> f64 {
        *self.topic_depths.get(topic).unwrap_or(&0.5)
    }

    /// Get the last N topic transitions.
    pub fn recent_transitions(&self, n: usize) -> Vec<&TopicTransition> {
        self.transitions.iter().rev().take(n).collect()
    }

    /// Get a context summary string (for reasoning engine).
    pub fn summary(&self) -> String {
        let topic = self.current_topic();
        let questions: Vec<&str> = self.open_questions.iter().map(|q| q.topic.as_str()).collect();
        let questions_str = if questions.is_empty() {
            "none".to_string()
        } else {
            questions.join(", ")
        };
        
        format!(
            "topic={}, depth={:.2}, certainty={:.2}, questions=[{}]",
            topic, self.depth, self.certainty, questions_str
        )
    }
}

impl Default for RingState {
    fn default() -> Self {
        Self::new()
    }
}

/// The current phase of the ring attractor.
#[derive(Debug, Clone)]
pub enum TopicPhase {
    /// Actively focused on a topic
    Focused(String),
    /// Between topics, transitioning
    Transitioning(String, String),
    /// Not yet focused on anything
    Unfocused,
}

/// An open question that Star wants to follow up on.
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

impl OpenQuestion {
    pub fn new(topic: impl Into<String>, why: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            why_interested: why.into(),
            asked_at_depth: 0.0,
            progress: 0.0,
        }
    }
}

/// A transition between topics.
#[derive(Debug, Clone)]
pub struct TopicTransition {
    pub from: String,
    pub to: String,
    pub depth_at_transition: f64,
}
