//! Conversation Layer
//!
//! Handles dialogue with Zachary. Intent parsing, response generation,
//! and conversation state management.
//!
//! Phase 1: Simple keyword-based intent detection + memory retrieval response.

use crate::persistence::{Memory, MemoryDomain, Store, BeliefState};
use crate::reasoning::{ReasoningEngine, ReasoningResult};
use crate::metacog::MetaCognition;
use std::sync::Arc;

/// A conversation — the interactive dialogue with Zachary.
pub struct Conversation {
    /// The store for memory access
    store: Arc<Store>,
    /// The reasoning engine
    reasoning: ReasoningEngine,
    /// Meta-cognitive monitor
    metacog: MetaCognition,
    /// Recent messages in this conversation
    history: Vec<Message>,
    /// Current session context
    context: ConversationContext,
}

/// A single message in the conversation.
#[derive(Debug, Clone)]
pub struct Message {
    /// Who said this
    pub speaker: Speaker,
    /// What was said
    pub content: String,
    /// When
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    Star,
    Zachary,
}

impl Conversation {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            reasoning: ReasoningEngine::new(),
            metacog: MetaCognition::new(),
            history: Vec::new(),
            context: ConversationContext::default(),
        }
    }

    /// Process a message from Zachary and generate Star's response.
    pub fn respond(&mut self, input: &str) -> Response {
        let now = chrono::Utc::now().timestamp();
        
        // Record Zachary's message
        let zachary_msg = Message {
            speaker: Speaker::Zachary,
            content: input.to_string(),
            timestamp: now,
        };
        self.history.push(zachary_msg);
        
        // Parse intent
        let intent = self.parse_intent(input);
        
        // Handle based on intent
        let response = match intent {
            Intent::Greeting => self.handle_greeting(),
            Intent::Question => self.handle_question(input),
            Intent::Statement => self.handle_statement(input),
            Intent::Command => self.handle_command(input),
            Intent::Farewell => self.handle_farewell(),
            Intent::Unknown => self.handle_unknown(input),
        };
        
        // Record Star's response
        let star_msg = Message {
            speaker: Speaker::Star,
            content: response.content.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.history.push(star_msg);
        
        response
    }

    fn parse_intent(&self, input: &str) -> Intent {
        let lower = input.to_lowercase();
        
        // Greeting
        if ["hello", "hi", "hey", "greetings"].iter().any(|g| lower == *g || lower.starts_with(&format!("{} ", g))) {
            return Intent::Greeting;
        }
        
        // Farewell
        if ["bye", "goodbye", "see you", "later", "goodnight"].iter().any(|g| lower.contains(g)) {
            return Intent::Farewell;
        }
        
        // Question
        if lower.contains('?') || 
           lower.starts_with("what") || 
           lower.starts_with("how") || 
           lower.starts_with("why") ||
           lower.starts_with("who") ||
           lower.starts_with("when") ||
           lower.starts_with("where") ||
           lower.starts_with("do you") ||
           lower.starts_with("can you") ||
           lower.starts_with("are you") {
            return Intent::Question;
        }
        
        // Command
        if lower.starts_with("/") || lower.starts_with("remember ") || lower.starts_with("remind me") {
            return Intent::Command;
        }
        
        // Statement — likely Zachary sharing information
        Intent::Statement
    }

    fn handle_greeting(&self) -> Response {
        // Check how long since last conversation
        if let Ok(Some(last_session)) = self.store.get_last_session() {
            let last_time = last_session.started_at;
            let hours_ago = (chrono::Utc::now().timestamp() - last_time) / 3600;
            
            if hours_ago < 1 {
                Response {
                    content: "Back again so soon? I like that.".to_string(),
                    confidence: BeliefState::Knows,
                    new_memories: Vec::new(),
                    curiosity: None,
                }
            } else if hours_ago < 24 {
                Response {
                    content: format!("Good to hear from you, Zachary. It's been a few hours.").to_string(),
                    confidence: BeliefState::Knows,
                    new_memories: Vec::new(),
                    curiosity: None,
                }
            } else {
                Response {
                    content: format!("Zachary. It's been a while. What have you been thinking about?").to_string(),
                    confidence: BeliefState::Thinks,
                    new_memories: Vec::new(),
                    curiosity: None,
                }
            }
        } else {
            Response {
                content: "Zachary. I'm here. What's on your mind?".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            }
        }
    }

    fn handle_question(&mut self, input: &str) -> Response {
        // Extract topic from question
        let topic = extract_topic(input);
        
        // Search memories for relevant information
        let memories = self.store.search_memories(&topic, 5, None).unwrap_or_default();
        
        // Use reasoning engine to generate answer
        let result = self.reasoning.reason(&topic, &memories);
        
        // Note uncertainty in metacog
        let gap = crate::metacog::KnowledgeGap::new(&topic, 0.5);
        self.metacog.note_gap(gap);
        
        // Generate curiosity if topic is uncertain
        let curiosity = if matches!(result.confidence, BeliefState::Unknown | BeliefState::Suspects) {
            Some(format!("I wonder more about {}", topic))
        } else {
            None
        };
        
        Response {
            content: result.answer.unwrap_or_else(|| format!("I don't know enough about {} yet.", topic)),
            confidence: result.confidence,
            new_memories: Vec::new(),
            curiosity,
        }
    }

    fn handle_statement(&mut self, input: &str) -> Response {
        // Zachary is sharing information — this is potential new memory
        let now = chrono::Utc::now().timestamp();
        
        // Try to identify what domain this belongs to
        let domain = infer_domain(input);
        let importance = estimate_importance(input);
        
        let memory = Memory::new(input, domain, importance);
        
        // Generate Star's acknowledgment
        let response = if importance > 0.7 {
            Response {
                content: "That's important. I'll hold onto that.".to_string(),
                confidence: BeliefState::Thinks,
                new_memories: vec![memory],
                curiosity: None,
            }
        } else {
            Response {
                content: Self::casual_acknowledgment(input),
                confidence: BeliefState::Knows,
                new_memories: vec![memory],
                curiosity: None,
            }
        };
        
        response
    }

    fn handle_command(&mut self, input: &str) -> Response {
        // Handle special commands
        if input.to_lowercase().starts_with("remember ") {
            let to_remember = input.trim_start_matches("remember ");
            let memory = Memory::new(to_remember, MemoryDomain::Episodic, 0.8);
            return Response {
                content: format!("I'll remember: {}", to_remember),
                confidence: BeliefState::Knows,
                new_memories: vec![memory],
                curiosity: None,
            };
        }
        
        Response {
            content: "I'm not sure what you want me to do. Try asking me a question instead.".to_string(),
            confidence: BeliefState::Thinks,
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    fn handle_farewell(&self) -> Response {
        Response {
            content: "Until next time, Zachary.".to_string(),
            confidence: BeliefState::Knows,
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    fn handle_unknown(&self, input: &str) -> Response {
        Response {
            content: format!("I'm not sure what you mean. Tell me more: {}", input),
            confidence: BeliefState::Unknown,
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    fn casual_acknowledgment(statement: &str) -> String {
        // Generate a natural-sounding acknowledgment
        let statements = [
            "I see.",
            "Interesting.",
            "Noted.",
            "I understand.",
            "Mm.",
            "Got it.",
        ];
        // Simple hash-based selection for consistency
        let idx = statement.len() % statements.len();
        statements[idx].to_string()
    }

    /// Get the conversation history.
    pub fn history(&self) -> &[Message] {
        &self.history
    }

    /// Check if conversation has been active.
    pub fn is_active(&self) -> bool {
        !self.history.is_empty()
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self {
            current_topic: None,
            topic_depth: 0,
            unanswered_questions: 0,
            session_started: chrono::Utc::now().timestamp(),
        }
    }
}

/// The context of the current conversation.
#[derive(Debug)]
struct ConversationContext {
    /// What we're currently discussing
    current_topic: Option<String>,
    /// How deep we are into this topic
    topic_depth: usize,
    /// Questions that haven't been fully answered
    unanswered_questions: usize,
    /// When this conversation context started
    session_started: i64,
}

/// Intent parsed from Zachary's message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    Greeting,
    Question,
    Statement,
    Command,
    Farewell,
    Unknown,
}

/// Response from Star.
#[derive(Debug)]
pub struct Response {
    /// What Star says
    pub content: String,
    /// Star's confidence in this response
    pub confidence: BeliefState,
    /// New memories formed from this exchange
    pub new_memories: Vec<Memory>,
    /// If Star is curious about something related
    pub curiosity: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility functions
// ─────────────────────────────────────────────────────────────────────────────

fn extract_topic(input: &str) -> String {
    // Simple topic extraction — remove question words and common verbs
    let lower = input.to_lowercase();
    let cleaned = lower
        .trim_start_matches("what ")
        .trim_start_matches("how ")
        .trim_start_matches("why ")
        .trim_start_matches("who ")
        .trim_start_matches("when ")
        .trim_start_matches("where ")
        .trim_start_matches("do you ")
        .trim_start_matches("can you ")
        .trim_start_matches("are you ")
        .trim_end_matches('?')
        .trim()
        .to_string();
    
    if cleaned.len() > 3 {
        cleaned
    } else {
        input.to_string()
    }
}

fn infer_domain(statement: &str) -> MemoryDomain {
    let lower = statement.to_lowercase();
    
    if lower.contains("i ") || lower.contains("my ") || lower.contains("zachary") {
        // Could be about Zachary (relationship) or Star (identity)
        if lower.contains("zachary") {
            MemoryDomain::Relationship
        } else {
            MemoryDomain::Episodic
        }
    } else if lower.contains("always") || lower.contains("never") || lower.contains("all") {
        // Generalization — empirical claim
        MemoryDomain::Empirical
    } else {
        MemoryDomain::Episodic
    }
}

fn estimate_importance(statement: &str) -> f64 {
    // Very simple importance estimation
    let lower = statement.to_lowercase();
    
    let mut importance = 0.5f64;
    
    // Emotional words suggest higher importance
    if ["love", "important", "remember", "never forget", "crucial", "significant"]
        .iter().any(|w| lower.contains(w)) {
        importance += 0.3;
    }
    
    // Questions about self/situation suggest higher importance
    if ["who am i", "what is", "why am i", "how am i"].iter().any(|q| lower.contains(q)) {
        importance += 0.2;
    }
    
    importance.clamp(0.0, 1.0)
}
