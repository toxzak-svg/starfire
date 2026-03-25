//! Context Fusion
//!
//! Combines ring state + query + mode to produce a ContextState.
//! This is the "fusion" step from SRMoE's fractured architecture —
//! multiple information sources combine to produce a coherent output.
//!
//! Fusion decisions:
//! 1. Which mode are we in? (from query + ring)
//! 2. How much does ring context weight? (from mode)
//! 3. What's in working memory? (recent turns)
//! 4. What questions are open? (from ring)
//!
//! The result is a ContextState that reasoning and conversation layers consume.

use super::{RingState, ReasoningMode, ContextState, TopicPhase};
use crate::persistence::Memory;

/// Fuses ring state + query + mode into a complete ContextState.
pub struct ContextFuser {
    /// Maximum working memory turns to keep
    max_working_memory: usize,
    /// Minimum curiosity threshold to act on
    curiosity_threshold: f64,
}

impl ContextFuser {
    pub fn new() -> Self {
        Self {
            max_working_memory: 10,
            curiosity_threshold: 0.4,
        }
    }

    /// Fuse everything into a ContextState.
    pub fn fuse(
        &self,
        ring: &RingState,
        mode: ReasoningMode,
        query: &str,
        working_memory: Vec<(String, String)>, // user, star turns
    ) -> ContextState {
        let context_weight = mode.context_weight();
        
        // Determine turns on topic
        let turns_on_topic = ring.topic_history.iter()
            .filter(|t| {
                match &ring.topic_phase {
                    TopicPhase::Focused(cur) => cur.contains(*t) || *t == cur,
                    _ => false,
                }
            })
            .count();

        // Extract curiosity cursor — what Star was last curious about
        let curiosity_cursor = ring.last_curiosity.clone();
        
        // Build working turn records
        let working_turns = working_memory.into_iter()
            .take(self.max_working_memory)
            .map(|(user, star)| super::WorkingTurn {
                user,
                star,
                ring_phase_at_time: ring.topic_phase.clone(),
            })
            .collect();

        ContextState {
            ring: ring.clone(),
            mode,
            context_weight,
            working_memory: working_turns,
            curiosity_cursor,
            turns_on_topic,
        }
    }

    /// Given a query and memories, infer the topic for ring update.
    pub fn infer_topic(&self, query: &str, memories: &[Memory]) -> String {
        let lower = query.to_lowercase();
        
        // Extract question-word prefixes
        let stripped = strip_question_prefix(&lower);
        
        // Check memories for relevant topics
        let mut best_topic = stripped.clone();
        let mut best_score = 0.0;
        
        for mem in memories {
            let mem_lower = mem.content.to_lowercase();
            // Does this memory share significant words with the query?
            let score = word_overlap(&stripped, &mem_lower);
            if score > best_score && score > 0.2 {
                best_score = score;
                best_topic = mem.content.clone();
            }
        }
        
        // If no strong memory match, use the stripped query as topic
        if best_score < 0.3 {
            best_topic = stripped;
        }
        
        best_topic
    }

    /// Update ring state from a new user message.
    pub fn update_ring(
        &self,
        ring: &mut RingState,
        query: &str,
        inferred_topic: &str,
    ) {
        ring.update_from_query(query, inferred_topic);
    }

    /// Update ring state from Star's response.
    pub fn update_ring_from_response(
        &self,
        ring: &mut RingState,
        response: &str,
        mode: ReasoningMode,
    ) {
        ring.update_from_response(response, mode);
    }

    /// Should Star express curiosity? Given the ring state.
    pub fn should_express_curiosity(&self, ring: &RingState) -> bool {
        // Express curiosity if:
        // 1. We have a low certainty (genuinely uncertain)
        // 2. We have an open question that's not been addressed
        // 3. We're in a new topic with low depth
        let has_open_question = ring.open_questions.iter().any(|q| q.progress < 0.5);
        let new_topic_shallow = matches!(&ring.topic_phase, TopicPhase::Focused(_)) 
            && ring.depth < 0.3;
        let uncertain = ring.certainty < self.curiosity_threshold;
        
        has_open_question || new_topic_shallow || uncertain
    }

    /// Get what Star should be curious about, if anything.
    pub fn get_curiosity_topic(&self, ring: &RingState) -> Option<String> {
        // Priority: open questions > curiosity cursor > gap in ring
        if let Some(q) = ring.open_questions.first() {
            if q.progress < 0.3 {
                return Some(q.topic.clone());
            }
        }
        
        if let Some(ref cursor) = ring.last_curiosity {
            if ring.certainty < 0.5 {
                return Some(cursor.clone());
            }
        }
        
        // If we're shallow on a topic and uncertain, express curiosity about going deeper
        if ring.depth < 0.4 && ring.certainty < 0.4 {
            return Some(ring.current_topic());
        }
        
        None
    }

    /// Given the ring and mode, should Star acknowledge the conversation history?
    pub fn should_reference_history(&self, ring: &RingState, mode: ReasoningMode) -> bool {
        // Don't reference history if we're DEFENDING or ASSERTING with high certainty
        if mode == ReasoningMode::DEFENDING || mode == ReasoningMode::ASSERTING {
            return ring.certainty > 0.8;
        }
        
        // Reference history if we've been on topic for a while
        if ring.topic_history.len() > 3 && ring.depth > 0.5 {
            return true;
        }
        
        // Reference history if there are open questions
        if !ring.open_questions.is_empty() {
            return true;
        }
        
        false
    }

    /// Generate a natural history reference string.
    pub fn history_reference(&self, ring: &RingState) -> Option<String> {
        if ring.topic_history.is_empty() {
            return None;
        }
        
        let last_topic = ring.topic_history.last()?;
        
        if ring.depth > 0.7 {
            Some(format!("As I was saying about {}...", last_topic))
        } else if ring.depth > 0.4 {
            Some(format!("Continuing on {}...", last_topic))
        } else {
            Some(format!("Going back to {}...", last_topic))
        }
    }
}

impl Default for ContextFuser {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip question-word prefixes from a query.
fn strip_question_prefix(query: &str) -> String {
    let prefixes = [
        "what is ",
        "what are ",
        "what does ",
        "what do ",
        "tell me about ",
        "tell me ",
        "why does ",
        "why do ",
        "why is ",
        "why are ",
        "how do ",
        "how does ",
        "how is ",
        "can you ",
        "could you ",
        "who is ",
        "who are ",
        "where is ",
        "where are ",
    ];
    
    let lower = query.to_lowercase();
    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            return query[prefix.len()..].trim_start_matches('"').to_string();
        }
    }
    
    query.to_string()
}

/// Compute word overlap between two strings (0-1).
fn word_overlap(a: &str, b: &str) -> f64 {
    let a_words: std::collections::HashSet<&str> = 
        a.split_whitespace().map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation())).collect();
    let b_words: std::collections::HashSet<&str> = 
        b.split_whitespace().map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation())).collect();
    
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    
    let intersection = a_words.intersection(&b_words).count() as f64;
    let union = a_words.union(&b_words).count() as f64;
    
    intersection / union
}
