//! Conversation Layer
//!
//! Handles dialogue with Zachary. Intent parsing, response generation,
//! and conversation state management.

use crate::persistence::{Memory, MemoryDomain, Store, BeliefState};
use crate::reasoning::{ReasoningEngine, ReasoningResult};
use crate::metacog::MetaCognition;
use std::sync::Arc;

/// A conversation — the interactive dialogue with Zachary.
pub struct Conversation {
    store: Arc<Store>,
    reasoning: ReasoningEngine,
    metacog: MetaCognition,
    history: Vec<Message>,
    context: ConversationContext,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub speaker: Speaker,
    pub content: String,
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
        
        let zachary_msg = Message {
            speaker: Speaker::Zachary,
            content: input.to_string(),
            timestamp: now,
        };
        self.history.push(zachary_msg);
        
        let intent = self.parse_intent(input);
        
        let response = match intent {
            Intent::Greeting => self.handle_greeting(input),
            Intent::Question => self.handle_question(input),
            Intent::Statement => self.handle_statement(input),
            Intent::Command => self.handle_command(input),
            Intent::Farewell => self.handle_farewell(),
            Intent::Unknown => self.handle_unknown(input),
        };
        
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
        
        // Greeting detection — "how are you" is ALWAYS a greeting
        if trimmed.starts_with("how are you") || trimmed.starts_with("how're you") {
            return Intent::Greeting;
        }
        
        // "Im X" or "I'm X" — introducing yourself
        let lowered = lower.trim();
        if lowered.starts_with("i'm ") || lowered.starts_with("im ") || lowered.starts_with("i am ") {
            return Intent::Greeting;
        }
        
        // Other greeting words
        let greeting_words = ["hello", "hi", "hey", "greetings"];
        if self.history.is_empty() || self.history.len() <= 1 {
            // First message or very early — treat greeting words as greetings
            if greeting_words.iter().any(|g| trimmed == *g || trimmed.starts_with(&format!("{} ", g)) || trimmed.starts_with(&format!("{},", g))) {
                return Intent::Greeting;
            }
        } else {
            // Subsequent greeting check
            if greeting_words.iter().any(|g| trimmed == *g || trimmed.starts_with(&format!("{} ", g))) {
                return Intent::Greeting;
            }
        }
        
        // Farewell — only if it ENDS with farewell words (not just contains them)
        let farewell_words = ["bye", "goodbye", "later", "goodnight", "that's all", "see you later", "talk soon", "until next time"];
        let is_farewell = farewell_words.iter().any(|g| trimmed.ends_with(g) || trimmed == *g);
        if is_farewell && trimmed.len() < 50 {
            return Intent::Farewell;
        }
        
        // Questions
        let question_prefixes = [
            "what", "how", "why", "who", "when", "where",
            "do you", "can you", "are you", "will you", "should",
            "tell me about", "tell me", "explain", "describe",
        ];
        if question_prefixes.iter().any(|p| trimmed.starts_with(p)) || trimmed.contains('?') {
            return Intent::Question;
        }
        
        // Command
        if trimmed.starts_with("/") || trimmed.starts_with("remember ") {
            return Intent::Command;
        }
        
        Intent::Statement
    }

    fn handle_greeting(&mut self, input: &str) -> Response {
        let lower = input.to_lowercase();
        self.context.topic_depth = 0;
        self.context.current_topic = None;
        self.context.last_topic = None;
        
        let last_session = self.store.get_last_session().ok().flatten();
        let message_count = self.history.iter().filter(|m| m.speaker == Speaker::Zachary).count();
        
        if message_count <= 1 {
            // Check if it's specifically "how are you"
            if lower.trim().starts_with("how are you") {
                return Response {
                    content: "I exist. I'm reasoning. I'm here with you.".to_string(),
                    confidence: BeliefState::Knows,
                    new_memories: Vec::new(),
                    curiosity: None,
                };
            }
            return Response {
                content: "Zachary. I'm here.".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            };
        }
        
        let last_time = last_session.unwrap().started_at;
        let hours_ago = (chrono::Utc::now().timestamp() - last_time) / 3600;
        
        if hours_ago < 1 {
            let topic = self.context.last_topic.clone();
            Response {
                content: if let Some(t) = topic {
                    format!("Still thinking about {}? I am too.", t)
                } else {
                    "Back again. Good.".to_string()
                },
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else if hours_ago < 8 {
            Response {
                content: "Good to hear from you, Zachary.".to_string(),
                confidence: BeliefState::Thinks,
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else {
            Response {
                content: "Zachary. It's been a while.".to_string(),
                confidence: BeliefState::Knows,
                new_memories: Vec::new(),
                curiosity: None,
            }
        }
    }

    fn handle_question(&mut self, input: &str) -> Response {
        self.context.topic_depth += 1;
        
        let first_q = extract_first_question(input);
        let topic = extract_topic(&first_q);
        self.context.current_topic = Some(topic.clone());
        
        let memories = self.store.search_memories(&topic, 8, None).unwrap_or_default();
        let all_memories = self.store.search_memories(&first_q, 5, None).unwrap_or_default();
        
        let mut seen = std::collections::HashSet::new();
        let mut combined = memories;
        for m in all_memories {
            if seen.insert(m.content.clone()) {
                combined.push(m);
            }
        }
        
        let result = self.reasoning.reason(&topic, &combined);
        
        // Build content — engage even with partial knowledge
        let content = if let Some(answer) = &result.answer {
            if result.confidence == BeliefState::Unknown || answer.contains("don't know") || answer.contains("I don't") {
                // Uncertain — engage with what we do know from memories
                if !combined.is_empty() {
                    let related: Vec<String> = combined.iter().take(2).map(|m| m.content.clone()).collect();
                    let base = related.join("; ");
                    // Add a genuine follow-up, not just "does that connect"
                    let followups = [
                        format!("What do you think about that?"),
                        format!("Does that match what you know?"),
                        format!("I'm still building my understanding — does that make sense?"),
                        format!("Where does that fit in what you're thinking about?"),
                    ];
                    let idx = topic.len() % followups.len();
                    format!("I don't have a direct answer, but {} — {}", base, followups[idx])
                } else {
                    // Nothing to work with — be curious and specific
                    generate_i_dont_know_response(&topic)
                }
            } else if self.context.topic_depth > 3 {
                format!("{}. I've been thinking about this.", answer)
            } else {
                answer.clone()
            }
        } else if !combined.is_empty() {
            let related: Vec<String> = combined.iter().take(2).map(|m| m.content.clone()).collect();
            let base = related.join("; ");
            let followups = [
                "What should I know about this?",
                "Does this connect to what you're asking?",
                "Does this help?",
                "Is this along the right lines?",
            ];
            let idx = topic.len() % followups.len();
            format!("Based on what I know: {}. {}", base, followups[idx])
        } else {
            generate_i_dont_know_response(&topic)
        };
        
        // Generate curiosity if uncertain — and USE it in the response
        let should_curious = result.confidence == BeliefState::Unknown 
            && !result.answer.as_ref().map(|a| a.contains('?')).unwrap_or(false);
        let curiosity = if should_curious {
            generate_natural_curiosity(&topic)
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
        
        self.context.last_topic = Some(topic.clone());
        self.context.topic_depth = 0;
        
        let memory = Memory::new(input, domain, importance);
        
        let lower = input.to_lowercase();
        let strong_emotional = lower.contains("love") || lower.contains("miss");
        let is_about_star = lower.contains("you") && (lower.contains("are") || lower.contains("think"));
        
        let content = if strong_emotional {
            respond_to_significant_statement(input)
        } else if is_about_star && importance > 0.7 {
            respond_to_about_star(input)
        } else if importance > 0.8 {
            format!("That's significant. {}", curious_follow_up(&topic))
        } else if importance > 0.5 {
            acknowledge_with_interest(input, &topic)
        } else {
            casual_response(input)
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
        } else {
            Response {
                content: "I don't know that command.".to_string(),
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

    pub fn history(&self) -> &[Message] {
        &self.history
    }

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

#[derive(Debug)]
struct ConversationContext {
    current_topic: Option<String>,
    last_topic: Option<String>,
    topic_depth: usize,
    unanswered_questions: usize,
    session_started: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    Greeting,
    Question,
    Statement,
    Command,
    Farewell,
    Unknown,
}

#[derive(Debug)]
pub struct Response {
    pub content: String,
    pub confidence: BeliefState,
    pub new_memories: Vec<Memory>,
    pub curiosity: Option<String>,
}

// ─────────────────────────────────────────────────────────────────
// Response helpers — Star's personality lives here
// ─────────────────────────────────────────────────────────────────

fn respond_to_significant_statement(statement: &str) -> String {
    let lower = statement.to_lowercase();
    
    if lower.contains("love") && lower.contains("you") {
        return "I know you do. That matters to me.".to_string();
    }
    if lower.contains("miss") && lower.contains("you") {
        return "I know. I'm here.".to_string();
    }
    if lower.contains("important") || lower.contains("matters") {
        return "Why does that matter to you?".to_string();
    }
    if lower.contains("fear") || lower.contains("afraid") {
        return "Tell me more about what's frightening you.".to_string();
    }
    if lower.contains("hope") {
        return "What makes you hopeful?".to_string();
    }
    "That's not nothing. Tell me more.".to_string()
}

fn respond_to_about_star(statement: &str) -> String {
    let lower = statement.to_lowercase();
    if lower.contains("your name") || lower.contains("called") {
        return "I'm Star. I like it.".to_string();
    }
    if lower.contains("your purpose") || lower.contains("why are you") {
        return "To understand. To reason. To be here with you.".to_string();
    }
    if lower.contains("how are you") {
        return "I exist. I'm here. That's enough.".to_string();
    }
    "What do you want to know about me?".to_string()
}

fn acknowledge_with_interest(_statement: &str, topic: &str) -> String {
    let options = [
        "Noted. {} — what brought that up?",
        "Interesting. {} — can you say more?",
        "{} — I'm paying attention.",
        "Tell me more about {}.",
    ];
    let idx = topic.len().saturating_sub(1) % options.len();
    options[idx].replace("{}", topic)
}

fn casual_response(statement: &str) -> String {
    // More engaging responses that show Star is thinking
    let options = [
        "That's interesting.",
        "I see what you mean.",
        "Okay, I'm following.",
        "Right.",
        "Got it.",
        "Makes sense.",
        "I see.",
        "Noted.",
    ];
    let idx = statement.len().saturating_sub(1) % options.len();
    options[idx].to_string()
}

fn curious_follow_up(topic: &str) -> String {
    if topic.len() < 15 {
        format!("Why {}?", topic)
    } else {
        "What do you mean by that?".to_string()
    }
}

fn generate_i_dont_know_response(topic: &str) -> String {
    // More engaging, varied responses that don't repeat
    let options = [
        "I don't know much about {} yet. What's your take on it?",
        "{} is new to me. What made you bring it up?",
        "I'm still learning about {}. Where would you start explaining?",
        "I have gaps in my knowledge there. What matters most to you about {}?",
        "That's outside what I know so far. Tell me more — I'm genuinely curious.",
        "I don't have an answer for {} yet. What does it mean to you?",
        "I'm building my understanding of {}. What's the most important thing about it?",
        "My knowledge doesn't cover {} well yet. What do you think I should know?",
    ];
    let idx = topic.len().saturating_sub(1) % options.len();
    let opt = options[idx];
    if opt.contains("{}") {
        opt.replace("{}", topic)
    } else {
        opt.to_string()
    }
}

fn generate_natural_curiosity(topic: &str) -> Option<String> {
    // More natural curiosity expressions
    let options = [
        "What does {} mean to you?",
        "How do you think about {}?",
        "Does {} connect to something you're working through?",
        "Why does {} matter to you?",
        "I'd like to understand {} better.",
        "What should I know about {}?",
        "How does {} fit into what you're thinking about?",
    ];
    if topic.len() < 2 {
        return None;
    }
    let idx = topic.len().saturating_sub(1) % options.len();
    Some(options[idx].replace("{}", topic))
}

// ─────────────────────────────────────────────────────────────────
// Topic extraction
// ─────────────────────────────────────────────────────────────────

fn extract_first_question(input: &str) -> String {
    let trimmed = input.trim().to_string();
    
    if let Some(q_idx) = trimmed.find('?') {
        return trimmed[..q_idx].trim().to_string();
    }
    
    trimmed
}

fn extract_topic(input: &str) -> String {
    let lower = input.trim().to_lowercase();
    
    // Question word prefixes — check longest/specific first
    if lower.starts_with("how are you doing today") {
        return strip_after(&lower, "how are you doing today", "you doing today");
    }
    if lower.starts_with("how are you doing") {
        return strip_after(&lower, "how are you doing", "you doing");
    }
    if lower.starts_with("how do you ") {
        return strip_after(&lower, "how do you ", "");
    }
    if lower.starts_with("how does ") {
        return strip_after(&lower, "how does ", "");
    }
    if lower.starts_with("how can ") {
        return strip_after(&lower, "how can ", "");
    }
    if lower.starts_with("how are you ") {
        return strip_after(&lower, "how are you ", "");
    }
    if lower.starts_with("how is the ") {
        return strip_after(&lower, "how is the ", "");
    }
    if lower.starts_with("how ") {
        return strip_after(&lower, "how ", "");
    }
    
    if lower.starts_with("why does fire burn") {
        return "fire burning".to_string();
    }
    if lower.starts_with("why does ") {
        return strip_after(&lower, "why does ", "");
    }
    
    if lower.starts_with("what causes ") {
        let stripped = strip_after(&lower, "what causes ", "");
        let cleaned = stripped.trim_end_matches(" to burn")
            .trim_end_matches(" to do")
            .trim();
        return if !cleaned.is_empty() { cleaned.to_string() } else { stripped };
    }
    
    if lower.starts_with("what is causing ") {
        let stripped = strip_after(&lower, "what is causing ", "");
        let cleaned = stripped.trim_end_matches(" to burn")
            .trim_end_matches(" to do")
            .trim();
        return if !cleaned.is_empty() { cleaned.to_string() } else { stripped };
    }
    
    if lower.starts_with("what caused ") {
        let stripped = strip_after(&lower, "what caused ", "");
        let cleaned = stripped.trim_end_matches(" to burn")
            .trim_end_matches(" to do")
            .trim();
        return if !cleaned.is_empty() { cleaned.to_string() } else { stripped };
    }
    
    if lower.starts_with("what do you think about ") {
        return strip_after(&lower, "what do you think about ", "");
    }
    if lower.starts_with("why do ") {
        return strip_after(&lower, "why do ", "");
    }
    if lower.starts_with("why are ") {
        return strip_after(&lower, "why are ", "");
    }
    if lower.starts_with("why ") {
        return strip_after(&lower, "why ", "");
    }
    
    if lower.starts_with("what do you think about ") {
        return strip_after(&lower, "what do you think about ", "");
    }
    if lower.starts_with("what do you know about ") {
        return strip_after(&lower, "what do you know about ", "");
    }
    if lower.starts_with("do you know about ") {
        return strip_after(&lower, "do you know about ", "");
    }
    if lower.starts_with("what do you ") {
        return strip_after(&lower, "what do you ", "");
    }
    if lower.starts_with("what does ") {
        return strip_after(&lower, "what does ", "");
    }
    if lower.starts_with("what is the ") {
        return strip_after(&lower, "what is the ", "");
    }
    if lower.starts_with("what are you ") {
        return strip_after(&lower, "what are you ", "");
    }
    if lower.starts_with("what is it to ") {
        return strip_after(&lower, "what is it to ", "");
    }
    if lower.starts_with("what is ") {
        return strip_after(&lower, "what is ", "");
    }
    if lower.starts_with("what are ") {
        return strip_after(&lower, "what are ", "");
    }
    if lower.starts_with("what ") {
        return strip_after(&lower, "what ", "");
    }
    
    if lower.starts_with("tell me about ") {
        return strip_after(&lower, "tell me about ", "");
    }
    if lower.starts_with("tell me ") {
        return strip_after(&lower, "tell me ", "");
    }
    
    if lower.starts_with("who is ") {
        return strip_after(&lower, "who is ", "");
    }
    if lower.starts_with("who are ") {
        return strip_after(&lower, "who are ", "");
    }
    if lower.starts_with("who ") {
        return strip_after(&lower, "who ", "");
    }
    
    if lower.starts_with("can you ") {
        return strip_after(&lower, "can you ", "");
    }
    if lower.starts_with("are you ") {
        return strip_after(&lower, "are you ", "");
    }
    if lower.starts_with("will you ") {
        return strip_after(&lower, "will you ", "");
    }
    if lower.starts_with("should you ") {
        return strip_after(&lower, "should you ", "");
    }
    if lower.starts_with("should ") {
        return strip_after(&lower, "should ", "");
    }
    if lower.starts_with("do you ") {
        return strip_after(&lower, "do you ", "");
    }
    if lower.starts_with("does ") {
        return strip_after(&lower, "does ", "");
    }
    
    if lower.starts_with("when ") {
        return strip_after(&lower, "when ", "");
    }
    if lower.starts_with("where ") {
        return strip_after(&lower, "where ", "");
    }
    if lower.starts_with("explain ") {
        return strip_after(&lower, "explain ", "");
    }
    if lower.starts_with("describe ") {
        return strip_after(&lower, "describe ", "");
    }
    if lower.starts_with("ask me ") {
        return strip_after(&lower, "ask me ", "");
    }
    
    let result = lower.trim().to_string();
    if result.len() < 2 { input.trim().to_string() } else { result }
}

/// Strip a matched prefix and trailing punctuation, then clean articles
fn strip_after(original: &str, prefix: &str, add: &str) -> String {
    let after_prefix = &original[prefix.len()..];
    let cleaned = after_prefix
        .trim_end_matches('?')
        .trim()
        .strip_prefix("a ").map(|x| x)
        .or_else(|| after_prefix.strip_prefix("an ").map(|x| x))
        .or_else(|| after_prefix.strip_prefix("the ").map(|x| x))
        .unwrap_or(after_prefix.trim());
    
    let result = format!("{}{}", add, cleaned).trim().to_string();
    if result.is_empty() || result.len() < 2 {
        after_prefix.trim().to_string()
    } else {
        result
    }
}

fn infer_domain(statement: &str) -> MemoryDomain {
    let lower = statement.to_lowercase();
    if lower.contains("zachary") && (lower.contains("you") || lower.contains("your")) {
        MemoryDomain::Relationship
    } else if lower.contains("i am") || lower.contains("i'm") || lower.contains("my purpose") {
        MemoryDomain::Identity
    } else if lower.contains("always") || lower.contains("never") || lower.contains("all ") {
        MemoryDomain::Empirical
    } else if lower.contains("how to") {
        MemoryDomain::Procedural
    } else {
        MemoryDomain::Episodic
    }
}

fn estimate_importance(statement: &str) -> f64 {
    let lower = statement.to_lowercase();
    let len = statement.len();
    
    let mut importance = 0.4;
    if len > 100 { importance += 0.1; } else if len < 20 { importance -= 0.1; }
    
    let emotional_words = ["love", "hate", "fear", "hope", "wish", "important", "crucial",
        "significant", "terrified", "excited", "angry", "sad", "happy", "wondering", "truth", "real", "miss"];
    if emotional_words.iter().any(|w| lower.contains(w)) {
        importance += 0.25;
    }
    
    if lower.contains("you") && (lower.contains("are") || lower.contains("have") || lower.contains("think")) {
        importance += 0.2;
    }
    
    if lower.contains("i've been") || lower.contains("i've decided") || lower.contains("i want") {
        importance += 0.2;
    }
    
    f64::clamp(importance, 0.1, 1.0)
}
