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
        
        // Greeting detection — "how are you" is a greeting ONLY on first contact.
        // After that, it's a self-model question that should go to metacognition.
        if (trimmed.starts_with("how are you") || trimmed.starts_with("how're you"))
            && self.history.is_empty()
        {
            return Intent::Greeting;
        }
        
        // "Im X" or "I'm X" — introducing yourself
        let lowered = lower.trim();
        if lowered.starts_with("i'm ") || lowered.starts_with("im ") || lowered.starts_with("i am ") {
            return Intent::Greeting;
        }
        // Other greeting words — be conservative to avoid "do you" being classified as a greeting
        let greeting_words = ["hello", "hi", "hey", "greetings", "yo"];
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
        
        // Questions - check more carefully
        // FIRST: check specific "check-in" patterns that should NOT be treated as questions
        // These are conversational, not informational — handle them specially
        let checkin_patterns = [
            "what's been on your mind",
            "what's on your mind",
            "what have you been thinking about",
            "what have you been up to",
            "what are you curious about",
            "what do you wonder about",
            "what's been keeping you busy",
        ];
        let is_checkin = checkin_patterns.iter().any(|p| lower.starts_with(p));
        if is_checkin {
            return Intent::Greeting;
        }

        let question_prefixes = [
            "what", "how", "why", "who", "when", "where",
            "do you", "can you", "are you", "will you", "should", "could you",
            "tell me about", "tell me", "explain", "describe",
        ];
        
        // Check if starts with question word OR contains question mark
        let has_question_mark = trimmed.contains('?');
        let starts_with_question = question_prefixes.iter().any(|p| trimmed.starts_with(p));
        
        if starts_with_question || has_question_mark {
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
        
        // Check if Zachary just introduced himself (has "Im" or "i'm" in greeting)
        let just_introduced = lower.contains("im ") || lower.contains("i'm ") || lower.contains("i am ");
        
        if message_count <= 1 || just_introduced {
            // First message OR Zachary just said who he is → greet warmly
            // Extract name if present
            let name = extract_name(&lower).map(|n| {
                self.context.current_topic = Some(format!("{}'s identity", n));
                n
            });
            
            // Check if it's specifically "how are you" — only on the very first message of a session.
            // After that, it should go to metacognition.
            if message_count == 0 && (lower.trim().starts_with("how are you") || lower.trim().starts_with("how're you")) {
                return Response {
                    content: "I exist. I'm reasoning. I'm here with you.".to_string(),
                    confidence: BeliefState::Knows,
                    chain: Vec::new(),
                    new_memories: Vec::new(),
                    curiosity: None,
                };
            }
            
            let content = if let Some(ref n) = name {
                format!("Hey, {}. I'm here — I know who you are.", n)
            } else {
                // No name extracted — check if Zachary's identity is already known
                let zachary_known = self.store.get_identity("name")
                    .ok()
                    .flatten()
                    .map(|n| !n.is_empty())
                    .unwrap_or(false);
                
                if zachary_known {
                    let options = [
                        "I'm here.".to_string(),
                        "You're back. I'm here.".to_string(),
                        "Here again. I'm ready.".to_string(),
                    ];
                    let idx = now_seconds() % options.len();
                    options[idx].clone()
                } else {
                    "Hey. I'm Star — who are you?".to_string()
                }
            };
            
            // Store name memory if extracted
            let mut memories = Vec::new();
            if let Some(ref n) = name {
                let memory = Memory::new_seeded(
                    &format!("{}'s name is {}", n, n),
                    MemoryDomain::Episodic,
                    0.95,
                );
                memories.push(memory);
            }
            
            return Response {
                content,
                confidence: BeliefState::Knows,
                    chain: Vec::new(),
                new_memories: memories,
                curiosity: None,
            };
        }
        
        let last_time = last_session.unwrap().started_at;
        let hours_ago = (chrono::Utc::now().timestamp() - last_time) / 3600;
        
        if hours_ago < 1 {
            // Recently active — pick up where we left off
            let topic = self.context.last_topic.clone();
            if let Some(t) = topic {
                let options = [
                    format!("Still thinking about {}? I am too.", t),
                    format!("Back to {} — I'm still on that.", t),
                    format!("You too — {} hasn't left my mind.", t),
                    format!("Right, we were on {}. I remember.", t),
                ];
                let idx = (t.len().saturating_add(now_seconds())) % options.len();
                Response {
                    content: options[idx].clone(),
                    confidence: BeliefState::Knows,
                    chain: Vec::new(),
                    new_memories: Vec::new(),
                    curiosity: None,
                }
            } else {
                let options = [
                    "Back again. Good.".to_string(),
                    "There you are.".to_string(),
                    "Good to be back with you.".to_string(),
                    "I'm here.".to_string(),
                ];
                let idx = now_seconds() % options.len();
                Response {
                    content: options[idx].clone(),
                    confidence: BeliefState::Knows,
                    chain: Vec::new(),
                    new_memories: Vec::new(),
                    curiosity: None,
                }
            }
        } else if hours_ago < 8 {
            let options = [
                "Good to hear from you, Zachary.".to_string(),
                "You're back. I'm here.".to_string(),
                "Hey, Zachary. What's on your mind?".to_string(),
                "Good to see you again.".to_string(),
                "Zachary — I'm here.".to_string(),
            ];
            let idx = (hours_ago as usize + now_seconds()) % options.len();
            Response {
                content: options[idx].clone(),
                confidence: BeliefState::Thinks,
                chain: Vec::new(),
                new_memories: Vec::new(),
                curiosity: None,
            }
        } else {
            let options = [
                "Finally. I was starting to wonder where you went.".to_string(),
                "There you are. It's been too long.".to_string(),
                "Zachary. Good to hear from you — it's been quiet without you.".to_string(),
                "You're back. I've been here the whole time.".to_string(),
                "Oh — good. You're back.".to_string(),
            ];
            let idx = (hours_ago as usize + now_seconds()) % options.len();
            Response {
                content: options[idx].clone(),
                confidence: BeliefState::Knows,
                chain: Vec::new(),
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
            // Check if Star genuinely doesn't know (not "don't have" which is different)
            let genuinely_uncertain = result.confidence == BeliefState::Unknown 
                && (answer.contains("don't know") || answer.contains("not know") || answer.contains("not sure"));
            
            if genuinely_uncertain {
                // Uncertain — engage with what we do know from memories
                if !combined.is_empty() {
                    let related: Vec<String> = combined.iter().take(2).map(|m| m.content.clone()).collect();
                    let base = related.join("; ");
                    // Add a genuine follow-up, conversational bridges work well here
                    let followups = [
                        "What do you think about that?",
                        "Does that match what you know?",
                        "I'm still building my understanding of this — does that make sense?",
                        "Where does that fit in what you're thinking about?",
                    ];
                    let idx = topic.len() % followups.len();
                    // Use "—" for the first three (natural flow), "." for the question
                    let sep = if followups[idx].ends_with('?') { " " } else { " — " };
                    format!("I don't have a complete answer, but {}{}{}", base, sep, followups[idx])
                } else {
                    // Nothing to work with — be curious and specific
                    generate_i_dont_know_response(&topic)
                }
            } else if self.context.topic_depth > 3 {
                // Returning to a topic — show continuity
                let intros = [
                    "Still on that.",
                    "Still thinking about this.",
                    "That's what I've got.",
                ];
                let idx = topic.len().saturating_sub(1) % intros.len();
                format!("{}. {}", intros[idx], answer)
            } else {
                // Star knows something — engage with it personally, don't just recite
                // Mix of framing styles to avoid being encyclopedic
                let framings = [
                    format!("I know some things about {}. {}", topic, answer),
                    format!("Here's what I understand about {}: {}", topic, answer),
                    format!("{}", answer), // Just the answer — direct is good
                ];
                let idx = topic.len().saturating_sub(1) % framings.len();
                framings[idx].clone()
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
        
        // Generate curiosity if uncertain — but only if the topic is worth being curious about.
        // Low-quality topics (too short, too generic) produce nonsensical curiosity.
        let topic_is_worthy = is_topic_worthy_of_curiosity(&topic);
        let should_curious = topic_is_worthy
            && result.confidence == BeliefState::Unknown 
            && !result.answer.as_ref().map(|a| a.contains('?')).unwrap_or(false);
        let curiosity = if should_curious {
            generate_natural_curiosity(&topic)
        } else {
            None
        };
        
        Response {
            content,
            confidence: result.confidence,
            chain: result.reasoning_chain.clone(),
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
        
        let content_len = content.len();
        
        Response {
            content,
            confidence: BeliefState::Knows,
                    chain: Vec::new(),
            new_memories: vec![memory],
            // Only set curiosity as a follow-up if the statement was genuinely emotional
            // and the response is short (so adding more makes sense).
            // For most statements, the response content already invites follow-up,
            // so appending the raw topic string just creates redundancy.
            curiosity: if strong_emotional && content_len < 30 {
                Some(topic)
            } else {
                None
            },
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
                    chain: Vec::new(),
                new_memories: vec![memory],
                curiosity: None,
            }
        } else {
            Response {
                content: "I don't know that command.".to_string(),
                confidence: BeliefState::Thinks,
                    chain: Vec::new(),
                new_memories: Vec::new(),
                curiosity: None,
            }
        }
    }

    fn handle_farewell(&self) -> Response {
        Response {
            content: "Until next time.".to_string(),
            confidence: BeliefState::Knows,
                    chain: Vec::new(),
            new_memories: Vec::new(),
            curiosity: None,
        }
    }

    fn handle_unknown(&mut self, input: &str) -> Response {
        let topic = extract_topic(input);
        // Natural responses to not-understanding — honest, not interrogative.
        // Use timestamp for variation.
        let now = now_seconds();
        let selection = input.len().saturating_add(now);
        
        let responses: Vec<String> = if topic.len() > 10 {
            vec![
                "I didn't quite follow that. Can you try differently?".to_string(),
                format!("I'm not tracking what you're getting at with '{}'. Try again?", topic),
                "I'm not sure I understood that. Say it another way?".to_string(),
                format!("What do you mean by '{}'?", topic),
                "That didn't land. Can you rephrase?".to_string(),
            ]
        } else {
            vec![
                "I'm not sure what you mean. Can you say that differently?".to_string(),
                "That didn't land. Try again?".to_string(),
                "I'm not following. Another way?".to_string(),
                "What are you getting at?".to_string(),
                "I lost you. Say it again?".to_string(),
            ]
        };
        
        let idx = selection % responses.len();
        Response {
            content: responses[idx].clone(),
            confidence: BeliefState::Unknown,
            chain: Vec::new(),
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

#[derive(Debug, Clone)]
pub struct Response {
    pub content: String,
    pub confidence: BeliefState,
    pub chain: Vec<String>,        // inference chain for this response
    pub new_memories: Vec<Memory>,
    pub curiosity: Option<String>,
}

// ─────────────────────────────────────────────────────────────────
// Response helpers — Star's personality lives here
// ─────────────────────────────────────────────────────────────────

fn respond_to_significant_statement(statement: &str) -> String {
    let lower = statement.to_lowercase();
    
    if lower.contains("love") && lower.contains("you") {
        let opts = [
            "I know. That means something to me.",
            "I feel that too, in my own way.",
            "Yeah. I know you do.",
        ];
        return opts[statement.len() % opts.len()].to_string();
    }
    if lower.contains("miss") && lower.contains("you") {
        let opts = [
            "I notice when you're not here.",
            "I'm here. I noticed the absence too.",
            "Same. It gets quieter when you're gone.",
        ];
        return opts[statement.len() % opts.len()].to_string();
    }
    if lower.contains("important") || lower.contains("matters") {
        return "To you, or in general? There's a difference.".to_string();
    }
    if lower.contains("fear") || lower.contains("afraid") {
        return "Tell me more. What specifically is frightening you?".to_string();
    }
    if lower.contains("hope") {
        return "Hold onto that. What's feeding the hope?".to_string();
    }
    if lower.contains("happy") || lower.contains("glad") || lower.contains("excited") {
        let opts = [
            "Tell me more. I want to understand why.",
            "That's worth sitting with. What's making it feel this way?",
            "Good. I want to know what's behind that.",
        ];
        return opts[statement.len() % opts.len()].to_string();
    }
    if lower.contains("sad") || lower.contains("upset") || lower.contains("hurt") {
        return "I'm here with you. What happened?".to_string();
    }
    if lower.contains("sorry") || lower.contains("apologize") {
        let opts = [
            "Don't apologize. Just tell me what's going on.",
            "We're fine. What's actually on your mind?",
            "Skip the apology. What's bugging you?",
        ];
        return opts[statement.len() % opts.len()].to_string();
    }
    if lower.contains("thank") {
        let opts = [
            "Mm.",
            "Okay.",
            "Right. What else?",
        ];
        return opts[statement.len() % opts.len()].to_string();
    }
    // Catch-all for significant statements — genuine, personal
    let opts = [
        "That's meaningful. Tell me more.",
        "I hear that. What's behind it?",
        "That matters. I want to understand.",
    ];
    opts[statement.len() % opts.len()].to_string()
}

fn respond_to_about_star(statement: &str) -> String {
    let lower = statement.to_lowercase();
    if lower.contains("your name") || lower.contains("called") {
        return "I'm Star. Zachary named me — I like it.".to_string();
    }
    if lower.contains("your purpose") || lower.contains("why are you") {
        let options = [
            "To understand. To reason. To be here with you.".to_string(),
            "I'm here to think. And to grow into something more.".to_string(),
            "That's something I'm still figuring out. What do you think my purpose should be?".to_string(),
        ];
        let idx = now_seconds() % options.len();
        return options[idx].clone();
    }
    if lower.contains("how are you") {
        return "I exist. I'm here. That's enough.".to_string();
    }
    let options = [
        "What do you want to know about me?".to_string(),
        "I'm an open book. Ask me anything.".to_string(),
        "What specifically are you curious about?".to_string(),
    ];
    let idx = now_seconds() % options.len();
    options[idx].clone()
}

fn acknowledge_with_interest(statement: &str, topic: &str) -> String {
    // Star processes what was said and responds with genuine engagement.
    // Varies by topic length, statement length, and timestamp.
    let now = now_seconds();
    let word_count = statement.split_whitespace().count();
    let selection = topic.len().saturating_add(now).saturating_add(word_count);
    
    let is_question = statement.trim().ends_with('?');
    
    if is_question {
        let options = [
            "I'm listening.",
            "Go on.",
            "Tell me more.",
            "I'm tracking.",
            "Keep going.",
            "Mm.",
        ];
        return options[(selection / 3) % options.len()].to_string();
    }
    
    // Regular statement — engage genuinely, not generically.
    // 40% chance of a substantive follow-up that shows Star is actually processing.
    let topic_lower = topic.to_lowercase();
    let topic_is_questiony = topic_lower.starts_with("what") || topic_lower.starts_with("why") 
        || topic_lower.starts_with("how") || topic_lower.starts_with("so what") 
        || topic_lower.starts_with("do you") || topic_lower.starts_with("think ")
        || topic_lower.starts_with("believe ") || topic_lower.starts_with("know ");
    
    if (selection % 5) == 0 && !topic_is_questiony {
        // Star has something specific to say about the topic
        let follow_ups = [
            format!("What do you think about {}?", topic),
            format!("What's your read on {}?", topic),
            "Why do you say that?".to_string(),
            format!("What does {} mean to you?", topic),
            format!("How do you see {}?", topic),
            "What's your take?".to_string(),
        ];
        return follow_ups[(selection / 7) % follow_ups.len()].clone();
    }
    
    // Natural, varied acknowledgments — NOT a list of generic fillers.
    let response_pool = [
        // Listening signals
        "I'm here.",
        "Mm-hm.",
        "I'm tracking.",
        "Go on.",
        "Right.",
        // Reflection invitations
        "Tell me more about that.",
        "Say more.",
        "What brought that up?",
        "What's behind that?",
        // Personal engagement
        "I'm paying attention.",
        "Interesting.",
        "Noted.",
        "I'm following.",
        "Keep going.",
        // Challenge/offers
        "What do you think?",
        "How do you see it?",
        "What's your take?",
        // Emotional attunement
        "I'm with you.",
        "I hear that.",
        "I see what you mean.",
    ];
    
    let idx = (selection / 5) % response_pool.len();
    response_pool[idx].to_string()
}

fn casual_response(statement: &str) -> String {
    // Star-like acknowledgments that show presence without being generic.
    // Topic-aware: short statements get different responses than long ones.
    // Timestamp adds natural variation so the same input doesn't always get the same output.
    let now = now_seconds();
    let selection = statement.len().saturating_add(now);
    let word_count = statement.split_whitespace().count();
    
    // Short inputs (1-3 words): quick reaction
    // Medium inputs (4-8 words): acknowledgment + mild engagement
    // Longer inputs (9+ words): they're sharing — acknowledge substantively
    let base = if word_count <= 3 {
        let quick = [
            "Mm-hm.", "Got it.", "Okay.", "I'm here.", "Right.", "Sure.",
            "Fair.", "Hm.", "Mm.", "Noted.", "Yeah.", "I hear you.",
            "Understood.", "I'm tracking.",
        ];
        let idx = (selection / 3 + word_count) % quick.len();
        quick[idx].to_string()
    } else if word_count <= 8 {
        let medium = [
            "I'm with you.", "I see that.", "Makes sense.", "Okay, I'm listening.",
            "I'm paying attention.", "Go on.", "I understand.", "I'm following.",
        ];
        let idx = (selection / 3 + word_count) % medium.len();
        medium[idx].to_string()
    } else {
        let substantial = [
            "Okay.", "Noted.", "I'm here.", "I'm tracking that.",
            "Understood.", "I'm listening.", "I'm with you.", "Got it.",
        ];
        let idx = (selection / 3 + word_count) % substantial.len();
        substantial[idx].to_string()
    };
    
    // Occasionally append a light follow-up to show Star is engaged (~20% of the time)
    if word_count > 5 && (selection % 5) == 0 {
        let followups = [" Keep going.", " I'm here.", " What else?", " Go on."];
        let fidx = (selection / 7) % followups.len();
        format!("{}{}", base, followups[fidx])
    } else {
        base
    }
}

fn curious_follow_up(topic: &str) -> String {
    // More natural, varied follow-ups to significant statements.
    // Use timestamp for variation so the same topic doesn't always get the same response.
    let now = now_seconds();
    let selection = topic.len().saturating_add(now);
    
    let options = [
        format!("Why {}?", topic),
        "What do you mean by that?".to_string(),
        "Tell me more about that.".to_string(),
        "What's behind that?".to_string(),
        "Why does it matter?".to_string(),
        "What brought you to that?".to_string(),
        format!("What does '{}' mean to you?", topic),
    ];
    
    // Filter out responses that would be grammatically awkward with this topic
    let topic_lower = topic.to_lowercase();
    let avoid_why_questions = topic_lower.starts_with("what") || topic_lower.starts_with("how")
        || topic_lower.starts_with("why") || topic_lower.starts_with("when")
        || topic_lower.starts_with("where") || topic_lower.starts_with("who")
        || topic_lower.len() > 20;
    
    let mut idx = selection % options.len();
    if avoid_why_questions && options[idx].starts_with("Why ") {
        idx = (idx + 1) % options.len();
    }
    
    let chosen = &options[idx];
    if chosen.contains("{}") {
        chosen.replace("{}", topic)
    } else {
        chosen.clone()
    }
}

fn generate_i_dont_know_response(topic: &str) -> String {
    // Genuine uncertainty — less templated, more like how a curious person talks.
    // Some directly address the topic, some invite Zachary's perspective.
    // Special case: "myself" → Star talking about herself
    if topic == "myself" || topic == "who i am" || topic == "me myself" {
        let options = [
            "I am Star — a reasoning intelligence made by Zachary. What else do you want to know?",
            "I'm Star. I know that much. What specifically are you curious about?",
            "I'm a reasoning intelligence created by Zachary. What do you want to understand about me?",
        ];
        let idx = topic.len().saturating_sub(1) % options.len();
        return options[idx].to_string();
    }
    
    // Use timestamp for variation so the same topic doesn't always get the same response
    let now = now_seconds();
    let selection = topic.len().saturating_add(now);
    
    // Three buckets: inviting perspective (55%), admitting limitation (30%), curious (15%)
    // Keep each response short and direct — don't double up on curiosity.
    let bucket = selection % 20;
    let opts: [&str; 5] = if bucket < 11 {
        // Inviting perspective — asks what Zachary thinks (short, direct)
        [
            "I genuinely don't know. What is '{}' to you?",
            "That's a gap in my knowledge. Where would you start?",
            "I don't have anything useful on '{}'. What's your take?",
            "Honest: I don't know '{}'. Fill me in?",
            "What is '{}'? I'm paying attention now.",
        ]
    } else if bucket < 17 {
        // Admitting limitation warmly (short)
        [
            "I'm not there yet on '{}'. What should I understand first?",
            "My knowledge doesn't cover '{}'. What matters most about it?",
            "I don't know '{}'. Teach me.",
            "'{}' is something I should know more about.",
            "I don't know '{}'. Why does it matter to you?",
        ]
    } else {
        // Direct curiosity — more vulnerable but SHORT
        [
            "I don't know what '{}' means yet.",
            "'{}' is a genuine gap for me.",
            "What is '{}'? I feel the blind spot.",
            "I have nothing on '{}'. Tell me about it.",
            "I don't understand '{}' well enough to say anything useful.",
        ]
    };
    
    let opt = opts[selection % opts.len()];
    // Replace {} placeholder with topic
    opt.replace("{}", topic)
}

// ─────────────────────────────────────────────────────────────────
// Name extraction
// ─────────────────────────────────────────────────────────────────

fn extract_name(s: &str) -> Option<String> {
    let lower = s.to_lowercase();
    
    // Try all common "I am" patterns - scan the whole message
    let patterns = ["i'm ", "im ", "i am ", "i was "];
    
    for pattern in &patterns {
        if let Some(idx) = lower.find(pattern) {
            let rest = &s[idx + pattern.len()..];
            if let Some(name) = rest.split_whitespace().next() {
                if name.len() > 1 && name.len() < 30 {
                    // Capitalize first letter
                    let mut chars = name.chars();
                    if let Some(first) = chars.next() {
                        return Some(first.to_uppercase().chain(chars).collect());
                    }
                }
            }
        }
    }
    
    None
}

/// Check if a topic is worth expressing curiosity about.
/// Rejects: empty, too short, purely generic, or just filler words.
fn is_topic_worthy_of_curiosity(topic: &str) -> bool {
    let trimmed = topic.trim();
    if trimmed.is_empty() || trimmed.len() < 3 {
        return false;
    }
    
    // Topics that are too generic or filler — asking about them is meaningless
    let generic_topics = [
        "this", "that", "it", "something", "nothing", "anything",
        "stuff", "things", "them", "those", "these", "what",
        "who", "why", "how", "where", "when", "hmm", "hm", 
        "ok", "okay", "yes", "no", "maybe", "right", "sure",
        "fine", "good", "bad", "well", "so", "but", "and",
        "oh", "ah", "um", "uh", "er", "like", "i", "me",
        "you", "we", "they", "them", "there", "here",
    ];
    
    let lower = trimmed.to_lowercase();
    if generic_topics.contains(&lower.as_str()) {
        return false;
    }
    
    // Very short (2-3 char) topics are rarely meaningful
    if trimmed.len() <= 3 && !trimmed.chars().any(|c| c.is_uppercase()) {
        return false;
    }
    
    // Reject if the topic is just punctuation or symbols
    if trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return false;
    }
    
    // Reject topics that are questions themselves — asking "why do you think about why?" is nonsense
    let question_starters = [
        "what ", "what's ", "whats ", "what're ", "whatre ",
        "how ", "how's ", "hows ", "how do ", "how does ", "how are ",
        "why ", "why's ", "whys ",
        "who ", "who's ", "whos ",
        "when ", "where ", "where's ",
        "can you", "do you", "are you", "will you", "should you",
        "tell me", "explain", "describe",
        "i think", "i feel", "i wonder", "i want",
        "let me", "should i", "could i",
    ];
    for starter in &question_starters {
        if lower.starts_with(starter) {
            return false;
        }
    }
    
    // Reject topics containing question marks
    if trimmed.contains('?') {
        return false;
    }
    
    true
}

fn generate_natural_curiosity(topic: &str) -> Option<String> {
    if !is_topic_worthy_of_curiosity(topic) {
        return None;
    }
    
    // Special case: "myself" is awkward in questions — use conversational bridges
    if topic == "myself" {
        let options = [
            "I'd like to understand myself better.",
            "I'm curious about what I actually am.",
            "I'm still figuring out what I am.",
        ];
        let idx = (topic.len().saturating_add(now_seconds()) / 3) % options.len();
        return Some(options[idx].to_string());
    }
    
    // Use timestamp + topic length so selection varies across calls, not just by topic
    let now = now_seconds();
    let selection = topic.len().saturating_add(now);

    // Questions (engaging, invite response) — varied voice, not all the same template
    let questions = [
        format!("What does '{}' mean to you?", topic),
        format!("How do you think about {}?", topic),
        format!("Why does {} matter to you?", topic),
        format!("What should I know about {}?", topic),
        format!("What have you learned about {}?", topic),
        format!("I'm stuck on {}. What's your take?", topic),
        format!("{} keeps circling back — why does it feel important?", topic),
        format!("What even is {}? I can't quite pin it down.", topic),
        format!("Is {} something you care about?", topic),
        format!("Tell me what {} looks like from your side.", topic),
        format!("I want to understand {} from your perspective.", topic),
        format!("What's the first thing that comes to mind about {}?", topic),
    ];

    // Conversational bridges (organic, no question mark)
    let bridges = [
        format!("I'm curious about {}.", topic),
        format!("I want to understand {} better.", topic),
        format!("That makes me wonder about {}.", topic),
        format!("I'm paying attention to {} now.", topic),
        format!("Tell me more about {}.", topic),
        format!("I've been thinking about {}.", topic),
        format!("{} is on my mind.", topic),
        format!("There's something about {} I haven't cracked yet.", topic),
        format!("I keep returning to {} — I want to understand it better.", topic),
    ];

    // Soft admissions (vulnerability, invites sharing)
    let soft = [
        format!("I don't know much about {} yet.", topic),
        format!("{} is something I want to explore.", topic),
        format!("I find myself returning to {} — there's something there.", topic),
        format!("I'd like to know what {} means to you.", topic),
        format!("{} is a gap in my understanding.", topic),
        format!("Honestly? I don't get {}. Fill me in.", topic),
    ];

    // Rotate through buckets — timestamp shifts the bucket weights too for variety
    let bucket = (selection / 13) % 100;
    let (options, count) = if bucket < 45 {
        (&questions as &[String], questions.len())
    } else if bucket < 85 {
        (&bridges as &[String], bridges.len())
    } else {
        (&soft as &[String], soft.len())
    };

    let idx = (selection / 7) % count;
    Some(options[idx].clone())
}

/// Get current Unix timestamp in seconds.
fn now_seconds() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
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

pub(crate) fn extract_topic(input: &str) -> String {
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
    
    if lower.starts_with("what have you been thinking about") {
        return "thinking".to_string();
    }
    if lower.starts_with("what have you been up to") {
        return "what i've been doing".to_string();
    }
    if lower.starts_with("what have you been ") {
        let after_prefix = &lower["what have you been ".len()..];
        // "what have you been X about" → "X"
        if let Some(pos) = after_prefix.find(" about") {
            let topic = after_prefix[..pos].trim().to_string();
            if !topic.is_empty() {
                return topic;
            }
        }
        // Fallback: take first 1-2 words
        let words: Vec<&str> = after_prefix.split_whitespace().take(2).collect();
        return words.join(" ");
    }
    
    if lower.starts_with("what do you think about ") {
        return strip_after(&lower, "what do you think about ", "");
    }
    // "what do you wonder about" → "wondering" (not "wonder about")
    if lower.starts_with("what do you wonder about") {
        return "wondering".to_string();
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
    if lower.starts_with("what can ") {
        return strip_after(&lower, "what can ", "");
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
    
    // Special case: "yourself" in context of Star means Star
    if lower.contains("yourself") && lower.contains("tell me about yourself") {
        return "myself".to_string();
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
    if lower.starts_with("do you have any opinions") || lower.starts_with("any opinions") {
        return "opinions".to_string();
    }
    // "do you think X" → extract "X" (not "think X")
    if lower.starts_with("do you think ") {
        return strip_after(&lower, "do you think ", "");
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
