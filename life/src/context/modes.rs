//! Reasoning Modes — Symbolic VQ Codebook
//!
//! Inspired by SRMoE's VQ codebook router: instead of softmax over actions,
//! a discrete codebook of reasoning "modes" that Star can be in.
//!
//! Each mode is a distinct way of processing and responding.
//! The mode is selected based on query + ring context.
//! This gives Star different "flavors" of reasoning rather than
//! always responding the same way.
//!
//! Modes are discrete (like VQ codes), not continuous.
//! Transition between modes is governed by the fusion logic.

use std::fmt;

/// The discrete reasoning mode — like a VQ codeword in SRMoE's router.
/// Each mode is a different processing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningMode {
    /// Open, curious processing — exploring a topic without commitment
    EXPLORING,
    
    /// Narrow, deep processing — focused reasoning on a specific question
    FOCUSING,
    
    /// Question-asking mode — prompted to find out more
    QUESTIONING,
    
    /// Confident assertion — stating what Star believes firmly
    ASSERTING,
    
    /// Expressing genuine uncertainty — "I don't know" as real state
    UNCERTAIN,
    
    /// Reacting to unexpected conclusion — the reasoning led somewhere surprising
    SURPRISED,
    
    /// Reflecting on own process — thinking about thinking
    REFLECTING,
    
    /// Defensive mode — when identity is challenged
    DEFENDING,
}

impl ReasoningMode {
    /// Determine the best mode given query content and ring state.
    pub fn from_query_and_ring(query: &str, ring_certainty: f64, ring_depth: f64) -> Self {
        let lower = query.to_lowercase();
        
        // Question marks → QUESTIONING
        if lower.ends_with('?') || lower.contains("what is") || lower.contains("why does") 
            || lower.contains("how do") || lower.contains("can you") {
            return ReasoningMode::QUESTIONING;
        }
        
        // Identity challenge → DEFENDING
        let identity_markers = ["are you real", "are you alive", "do you feel", 
                               "are you conscious", "do you know"];
        if identity_markers.iter().any(|m| lower.contains(m)) {
            return ReasoningMode::DEFENDING;
        }
        
        // High uncertainty in ring + deep topic → UNCERTAIN
        if ring_certainty < 0.3 && ring_depth > 0.5 {
            return ReasoningMode::UNCERTAIN;
        }
        
        // Surprising marker in ring → SURPRISED
        if ring_certainty < 0.25 {
            return ReasoningMode::SURPRISED;
        }
        
        // Meta-question about Star itself → REFLECTING
        let meta_markers = ["what do you think about", "how do you feel about", 
                           "why do you", "do you agree"];
        if meta_markers.iter().any(|m| lower.contains(m)) {
            return ReasoningMode::REFLECTING;
        }
        
        // High certainty + deep topic → ASSERTING
        if ring_certainty > 0.7 && ring_depth > 0.5 {
            return ReasoningMode::ASSERTING;
        }
        
        // Default: EXPLORING
        ReasoningMode::EXPLORING
    }

    /// Get the context weight for this mode.
    /// Higher = the ring context matters more for the response.
    pub fn context_weight(&self) -> f64 {
        match self {
            ReasoningMode::EXPLORING => 0.5,    // Ring heavily influences direction
            ReasoningMode::FOCUSING => 0.2,     // Stay on query topic
            ReasoningMode::QUESTIONING => 0.4,  // Ring guides what to ask next
            ReasoningMode::ASSERTING => 0.3,   // Context supports assertion
            ReasoningMode::UNCERTAIN => 0.6,    // Ring helps frame uncertainty
            ReasoningMode::SURPRISED => 0.5,    // Ring context explains surprise
            ReasoningMode::REFLECTING => 0.7,   // Ring history is the content
            ReasoningMode::DEFENDING => 0.1,    // Identity is query-driven
        }
    }

    /// Get the response length tendency (words).
    pub fn typical_length(&self) -> (usize, usize) {
        match self {
            ReasoningMode::EXPLORING => (30, 80),    // Medium, open
            ReasoningMode::FOCUSING => (40, 100),    // Longer, detailed
            ReasoningMode::QUESTIONING => (10, 30),  // Short, asking
            ReasoningMode::ASSERTING => (50, 120),   // Longer, confident
            ReasoningMode::UNCERTAIN => (20, 60),    // Medium, hedging
            ReasoningMode::SURPRISED => (30, 80),    // Medium, explanatory
            ReasoningMode::REFLECTING => (60, 150),  // Longer, introspective
            ReasoningMode::DEFENDING => (40, 100),   // Medium-long, firm
        }
    }

    /// Does this mode prefer short or long sentences?
    pub fn sentence_style(&self) -> SentenceStyle {
        match self {
            ReasoningMode::EXPLORING => SentenceStyle::Mixed,
            ReasoningMode::FOCUSING => SentenceStyle::Complex,
            ReasoningMode::QUESTIONING => SentenceStyle::Short,
            ReasoningMode::ASSERTING => SentenceStyle::Declarative,
            ReasoningMode::UNCERTAIN => SentenceStyle::Hedged,
            ReasoningMode::SURPRISED => SentenceStyle::Mixed,
            ReasoningMode::REFLECTING => SentenceStyle::Complex,
            ReasoningMode::DEFENDING => SentenceStyle::Declarative,
        }
    }
}

impl fmt::Display for ReasoningMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReasoningMode::EXPLORING => write!(f, "EXPLORING"),
            ReasoningMode::FOCUSING => write!(f, "FOCUSING"),
            ReasoningMode::QUESTIONING => write!(f, "QUESTIONING"),
            ReasoningMode::ASSERTING => write!(f, "ASSERTING"),
            ReasoningMode::UNCERTAIN => write!(f, "UNCERTAIN"),
            ReasoningMode::SURPRISED => write!(f, "SURPRISED"),
            ReasoningMode::REFLECTING => write!(f, "REFLECTING"),
            ReasoningMode::DEFENDING => write!(f, "DEFENDING"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SentenceStyle {
    /// Short, punchy sentences
    Short,
    /// Longer, structured sentences with clauses
    Complex,
    /// Mixed sentence lengths
    Mixed,
    /// Confident, direct statements
    Declarative,
    /// Hedged, qualified statements
    Hedged,
}
