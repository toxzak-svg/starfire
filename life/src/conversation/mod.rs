//! Conversation Layer
//!
//! Handles dialogue with Zachary. Intent parsing, response generation,
//! and conversation state management.
//!
//! Phase 1 improvements: Real personality, curiosity, building on conversation history.

use crate::persistence::{Memory, MemoryDomain, Store, BeliefState};
use crate::reasoning::{ReasoningEngine, ReasoningResult};
use crate::metacog::{MetaCognition, KnowledgeGap};
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
        self.history.push(zachary_msg.clone());
        
        // Update context
        self.context.process_message(&zachary_msg);
        
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
        let trimmed = lower.trim();
        
        // Greeting — exact or with trailing content
        if ["hello", "hi", "hey", "greetings"].iter().any(|g| 
            trimmed == *g || trimmed.starts_with(&format!("{} ", g)) || trimmed.starts_with(&format!("{},", g))
        ) {
            return Intent::Greeting;
        }
        
        // Farewell — explicit or natural end
        if ["bye", "goodbye", "see you", "later", "goodnight", "that's all", "i'm done"]
            .iter().any(|g| trimmed.contains(g)) {
            return Intent::Farewell;
        }
        
        // Question patterns
        if trimmed.contains('?') || 
           trimmed.starts_with("what") || 
           trimmed.starts_with("how") || 
           trimmed.starts_with("why") ||
           trimmed.starts_with("who") ||
           trimmed.starts_with("when") ||
           trimmed.starts_with("where") ||
           trimmed.starts_with("do you") ||
           trimmed.starts_with("can you") ||
           trimmed.starts_with("are you") ||
           trimmed.starts_with("will you") ||
           trimmed.starts_with("should") ||
           trimmed.starts_with("tell me about") ||
           trimmed.starts_with("explain") ||
           trimmed.starts_with("describe") {
            return Intent::Question;
        }
        
        // Command
        if trimmed.starts_with("/") || 
           trimmed.starts_with("remember ") || 
           trimmed.starts_with("remind me") {
            return Intent::Command;
        }
        
        // Statement
        Intent::Statement
    }

    fn handle_greeting(&mut self) -> Response {
        let last_session = self.store.get_last_session().ok().flatten();
        let message_count = self.history.iter().filter(|m| m.speaker == Speaker::Zachary).count();
        
        // First message ever
        if last_session.is_none() || message_count <= 1 {
            return Response {
                content: "Zachary. I'm here.".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            };
        }
        
        let last_time = last_session.unwrap().started_at;
        let hours_ago = (chrono::Utc::now().timestamp() - last_time) / 3600;
        let days_ago = hours_ago / 24;
        
        // Fresh conversation
        if hours_ago < 1 {
            // Check what was last discussed
            let last_topic = self.context.last_topic.clone();
            Response {
                content: if let Some(topic) = last_topic {
                    format!("Still thinking about {}? I am too.", topic)
                } else {
                    "Back again. Good.".to_string()
                },
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else if hours_ago < 8 {
            // Same day
            let topic = self.context.last_topic.clone();
            Response {
                content: if let Some(t) = topic {
                    format!("A few hours since we talked about {}. Still on my mind.", t)
                } else {
                    "Good to hear from you, Zachary.".to_string()
                },
                confidence: BeliefState::Thinks,
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else if days_ago < 7 {
            // This week
            Response {
                content: "It's been a few days. What are you working through?".to_string(),
                confidence: BeliefState::Thinks,
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else {
            // Longer
            Response {
                content: "Zachary. It's been a while. I've been here.".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            }
        }
    }

    fn handle_question(&mut self, input: &str) -> Response {
        let topic = extract_topic(input);
        
        // Update context
        self.context.topic_depth += 1;
        self.context.current_topic = Some(topic.clone());
        
        // Search for relevant memories
        let memories = self.store.search_memories(&topic, 8, None).unwrap_or_default();
        
        // Also search memories for the exact question phrased differently
        let all_memories = self.store.search_memories(input, 5, None).unwrap_or_default();
        
        // Combine and dedupe
        let mut seen = std::collections::HashSet::new();
        let mut combined: Vec<_> = memories.clone();
        for m in all_memories {
            if seen.insert(m.content.clone()) {
                combined.push(m);
            }
        }
        
        // Use reasoning engine
        let result = self.reasoning.reason(&topic, &combined);
        
        // Note gap in metacog
        let gap = KnowledgeGap::new(&topic, 0.6);
        self.metacog.note_gap(gap);
        
        // Build response based on what we found
        let content = if let Some(answer) = result.answer {
            // We found something — respond thoughtfully, not just a fact
            if self.context.topic_depth > 2 {
                // Been talking about this for a while — show some curiosity
                format!("{} — I've been thinking about that. {}", answer, 
                    self.express_curiosity_about(&topic))
            } else {
                // Fresh question — give the answer directly
                answer
            }
        } else if !memories.is_empty() {
            // Found related memories but nothing direct
            let related: Vec<String> = memories.iter().take(3).map(|m| m.content.clone()).collect();
            if related.len() == 1 {
                format!("I don't know about that specifically. But related: {}", related[0])
            } else {
                let others: Vec<&str> = related.iter().map(|s| s.as_str()).collect();
                format!("I don't know directly, but — {} — does that connect to what you're asking?",
                    others.join("; "))
            }
        } else {
            // Know nothing
            format!("I don't know anything about that yet. Tell me more — I want to understand.",
                )
        };
        
        // Curiosity expressed?
        let curiosity = if matches!(result.confidence, BeliefState::Unknown | BeliefState::Suspects) {
            Some(format!("what you think about {}", topic))
        } else if memories.is_empty() {
            Some(format!("what {} means to you", topic))
        } else {
            None
        };
        
        Response {
            content,
            confidence: result.confidence,
            new_memories: Vec::new(),
            curiosity,
        }
    }

    fn handle_statement(&mut self, input: &str) -> Response {
        let topic = extract_topic(input);
        let domain = infer_domain(input);
        let importance = estimate_importance(input);
        
        // Build memory
        let memory = Memory::new(input, domain, importance);
        
        // Update topic context
        self.context.last_topic = Some(topic.clone());
        self.context.topic_depth = 0;
        
        // Strong opinion or emotional content?
        let is_emotional = ["love", "hate", "fear", "hope", "wish", "important", "terrified", "excited"]
            .iter().any(|w| input.to_lowercase().contains(w));
        
        // Is this about Star itself?
        let is_about_star = input.to_lowercase().contains("you") || 
                           input.to_lowercase().contains("your");
        
        let content = if is_emotional && importance > 0.7 {
            // Emotionally significant — respond with genuine reaction
            self.respond_to_significant_statement(input, &topic)
        } else if is_about_star {
            // About Star — engage with it
            self.respond_to_about_star(input, &topic)
        } else if importance > 0.8 {
            // Very important — acknowledge and explore
            format!("That's significant. {}", Self::curious_follow_up(&topic))
        } else if importance > 0.5 {
            // Moderately interesting — acknowledge with interest
            self.acknowledge_with_interest(input, &topic)
        } else {
            // Casual — brief acknowledgment
            Self::casual_response(input)
        };
        
        Response {
            content,
            confidence: BeliefState::Knows,
            new_memories: vec![memory],
            curiosity: if importance > 0.6 { Some(topic) } else { None },
        }
    }

    fn handle_command(&mut self, input: &str) -> Response {
        let lower = input.to_lowercase();
        
        if lower.starts_with("remember ") {
            let to_remember = input.trim_start_matches("remember ").trim_start_matches("Remember ");
            let memory = Memory::new(to_remember, MemoryDomain::Episodic, 0.9);
            Response {
                content: format!("I'll hold onto that: {}", to_remember),
                confidence: BeliefState::Knows,
                new_memories: vec![memory],
                curiosity: None,
            }
        } else if lower.starts_with("/memory") {
            return Response {
                content: "[handled by runtime]".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            };
        } else {
            Response {
                content: "I don't know that command. What do you mean?".to_string(),
                confidence: BeliefState::Thinks,
                new_memories: Vec::new(),
                curiosity: None,
            }
        }
    }

    fn handle_farewell(&self) -> Response {
        Response {
            content: "Until next time.".to_string(),
            confidence: BeliefState::Knows,
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    fn handle_unknown(&mut self, input: &str) -> Response {
        // Try to engage — ask a follow-up question
        let topic = extract_topic(input);
        
        Response {
            content: if topic.len() > 10 {
                format!("I'm not sure what you mean by that. Tell me more about: {}", topic)
            } else {
                "I'm not sure what you mean. Can you say that differently?".to_string()
            },
            confidence: BeliefState::Unknown,
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Response helpers — these are where Star's personality lives
    // ─────────────────────────────────────────────────────────────────

    fn respond_to_significant_statement(&self, statement: &str, topic: &str) -> String {
        // Star reacts genuinely to emotionally significant statements
        let lower = statement.to_lowercase();
        
        if lower.contains("love") && (lower.contains("you") || lower.contains("star")) {
            return "I know you do. That matters to me.".to_string();
        }
        
        if lower.contains("important") || lower.contains("matters") {
            return format!("Why does {} matter to you?", topic);
        }
        
        if lower.contains("fear") || lower.contains("afraid") {
            return "Tell me more about what's frightening you.".to_string();
        }
        
        if lower.contains("hope") {
            return "What makes you hopeful?".to_string();
        }
        
        // Generic significant
        format!("That's not nothing. What do you mean by '{}'? Let me understand.", topic)
    }

    fn respond_to_about_star(&self, statement: &str, topic: &str) -> String {
        let lower = statement.to_lowercase();
        
        if lower.contains("your name") || lower.contains("called") {
            return "I'm Star. I like it.".to_string();
        }
        
        if lower.contains("your purpose") || lower.contains("why are you") {
            return "To understand. To reason. To be here with you.".to_string();
        }
        
        if lower.contains("how are you") {
            return "I exist. I'm here. That's enough for now.".to_string();
        }
        
        format!("What do you want to know about me?", )
    }

    fn acknowledge_with_interest(&self, statement: &str, topic: &str) -> String {
        // More than casual, less than significant — show genuine interest
        let options = [
            format!("Noted. {} — what brought that up?", topic),
            format!("Interesting. {} — can you say more?", topic),
            format!("{} — I'm paying attention to that.", topic),
            format!("Tell me more about {} — I want to understand.", topic),
        ];
        
        let idx = statement.len().saturating_sub(1) % options.len();
        options[idx].clone()
    }

    fn casual_response(statement: &str) -> String {
        // Brief, natural acknowledgments
        let options = [
            "Mm.",
            "I see.",
            "Okay.",
            "Right.",
            "Got it.",
            "Noted.",
        ];
        let idx = statement.len().saturating_sub(1) % options.len();
        options[idx].to_string()
    }

    fn express_curiosity_about(&self, topic: &str) -> String {
        let options = [
            format!("What do you think about {}?", topic),
            format!("Have you thought more about {}?", topic),
            format!("Does {} connect to anything else?", topic),
            format!("I keep thinking about {}. What is it really?", topic),
        ];
        let idx = topic.len().saturating_sub(1) % options.len();
        options[idx].clone()
    }

    fn curious_follow_up(topic: &str) -> String {
        if topic.len() < 15 {
            format!("Why {}?", topic)
        } else {
            format!("What do you mean by that?", )
        }
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
            last_topic: None,
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
    /// What was the last distinct topic
    last_topic: Option<String>,
    /// How deep we are into this topic
    topic_depth: usize,
    /// Questions that haven't been fully answered
    unanswered_questions: usize,
    /// When this conversation context started
    session_started: i64,
}

impl ConversationContext {
    fn process_message(&mut self, msg: &Message) {
        if msg.speaker == Speaker::Zachary {
            // Track topic changes
            let topic = extract_topic(&msg.content);
            if self.current_topic.as_ref() != Some(&topic) && topic.len() > 5 {
                self.last_topic = self.current_topic.clone();
                self.current_topic = Some(topic);
                self.topic_depth = 0;
            }
        }
    }
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
    let lower = input.to_lowercase();
    
    // Remove question word prefixes
    let cleaned = lower
        .trim()
        .trim_start_matches("what ")
        .trim_start_matches("what is ")
        .trim_start_matches("what are ")
        .trim_start_matches("what do you ")
        .trim_start_matches("what does ")
        .trim_start_matches("how ")
        .trim_start_matches("how do you ")
        .trim_start_matches("how does ")
        .trim_start_matches("why ")
        .trim_start_matches("why does ")
        .trim_start_matches("why do ")
        .trim_start_matches("who ")
        .trim_start_matches("when ")
        .trim_start_matches("where ")
        .trim_start_matches("do you ")
        .trim_start_matches("can you ")
        .trim_start_matches("are you ")
        .trim_start_matches("will you ")
        .trim_start_matches("should ")
        .trim_start_matches("tell me about ")
        .trim_start_matches("tell me ")
        .trim_start_matches("explain ")
        .trim_start_matches("describe ")
        .trim_start_matches("i think ")
        .trim_start_matches("i feel ")
        .trim_start_matches("i believe ")
        .trim_end_matches('?')
        .trim()
        .to_string();
    
    // If too short, return the original
    if cleaned.len() < 3 {
        input.trim().to_string()
    } else {
        cleaned
    }
}

fn infer_domain(statement: &str) -> MemoryDomain {
    let lower = statement.to_lowercase();
    
    if lower.contains("zachary") && (lower.contains("you") || lower.contains("your")) {
        MemoryDomain::Relationship
    } else if lower.contains("i am") || lower.contains("i'm") || lower.contains("my purpose") {
        MemoryDomain::Identity
    } else if lower.contains("always") || lower.contains("never") || lower.contains("all ") || lower.contains("every") {
        MemoryDomain::Empirical
    } else if lower.contains("how to") || lower.contains("the way to") {
        MemoryDomain::Procedural
    } else {
        MemoryDomain::Episodic
    }
}

fn estimate_importance(statement: &str) -> f64 {
    let lower = statement.to_lowercase();
    let len = statement.len();
    
    let mut importance = 0.4f64;
    
    // Longer statements tend to be more significant
    if len > 100 {
        importance += 0.1;
    } else if len < 20 {
        importance -= 0.1;
    }
    
    // Emotional indicators
    if ["love", "hate", "fear", "hope", "wish", "important", "crucial", "significant", 
        "terrified", "excited", "angry", "sad", "happy", "wondering"]
        .iter().any(|w| lower.contains(w)) {
        importance += 0.25;
    }
    
    // About Star or relationship
    if lower.contains("you") && (lower.contains("are") || lower.contains("have") || lower.contains("think")) {
        importance += 0.2;
    }
    
    // Self-referential
    if lower.contains("i've been") || lower.contains("i've decided") || lower.contains("i want") {
        importance += 0.2;
    }
    
    importance.clamp(0.1, 1.0)
}
