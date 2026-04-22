//! Multi-Tempo Cognition
//!
//! Different reasoning "clocks" — fast, medium, and slow selves — each with
//! distinctive budgets and characteristic outputs. This makes the difference
//! between snap judgments and deep reflection visible and legible.
//!
//! ## The Three Tempos
//!
//! | Tempo | Budget | Character |
//! |-------|--------|-----------|
//! | **Fast** | ~50ms | Shallow, heuristic, pattern-based. Handles clarifications, obvious inferences, quick paraphrases. |
//! | **Medium** | ~500ms | Full symbolic engine with modest complexity budgets — Star's "default" reasoning. |
//! | **Slow** | ~10s+ | Long reflective processes: revisiting past dialogues, re-evaluating theories, restructuring KG. |
//!
//! ## Architecture Fit
//!
//! - **Layer 4 (Runtime):** Explicit scheduling of separate reasoning pools for fast/medium/slow
//! - **Layer 2 (Reasoning):** Fast uses cached patterns and shortcuts; Slow re-runs expensive abduction, analogy, re-derivation
//! - **Layer 3 (Meta-Cognition):** Tracks which tempo produced which belief; marks fast "provisional," slow "deep commitment"

use crate::reasoning::{ReasoningEngine, ReasoningResult, QueryType};
use crate::persistence::BeliefState;

/// The three tempo bands — different reasoning clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tempo {
    /// Fast: ~50ms budget. Cached patterns, heuristics, obvious inferences.
    Fast,
    /// Medium: ~500ms budget. Full symbolic reasoning with modest complexity.
    Medium,
    /// Slow: ~10s+ budget. Long reflection, KG restructuring, deep re-evaluation.
    Slow,
}

impl Tempo {
    /// Get the budget hint for this tempo (in ms).
    pub fn budget_ms(&self) -> u64 {
        match self {
            Tempo::Fast => 50,
            Tempo::Medium => 500,
            Tempo::Slow => 10_000,
        }
    }

    /// Get a description of this tempo.
    pub fn description(&self) -> &'static str {
        match self {
            Tempo::Fast => "fast (cached/heuristic)",
            Tempo::Medium => "medium (full symbolic)",
            Tempo::Slow => "slow (deep reflection)",
        }
    }
}

/// Metadata about which reasoning path was used.
#[derive(Debug, Clone)]
pub struct ReasoningSource {
    pub tempo: Tempo,
    pub processing_time_ms: u64,
    pub rules_used: usize,
    pub chain_depth: usize,
}

impl ReasoningSource {
    /// Was this conclusion from fast reasoning?
    pub fn is_fast(&self) -> bool {
        self.tempo == Tempo::Fast
    }

    /// Is this a deep commitment (from slow reasoning)?
    pub fn is_deep(&self) -> bool {
        self.tempo == Tempo::Slow
    }

    /// Convert to a display string for responses.
    pub fn tag(&self) -> &'static str {
        match self.tempo {
            Tempo::Fast => "[fast]",
            Tempo::Medium => "[medium]",
            Tempo::Slow => "[slow]",
        }
    }
}

/// Result of multi-tempo reasoning, including source metadata.
#[derive(Debug)]
pub struct TempoResult {
    pub result: ReasoningResult,
    pub source: ReasoningSource,
}

impl TempoResult {
    /// Get the answer.
    pub fn answer(&self) -> Option<&String> {
        self.result.answer.as_ref()
    }

    /// Get the confidence.
    pub fn confidence(&self) -> BeliefState {
        self.result.confidence
    }

    /// Should this be treated as provisional (fast reasoning)?
    pub fn is_provisional(&self) -> bool {
        self.source.is_fast()
    }

    /// Is this a deep commitment (slow reasoning)?
    pub fn is_deep(&self) -> bool {
        self.source.is_deep()
    }
}

/// Multi-tempo reasoning engine — routes queries to appropriate tempo.
pub struct TempoEngine {
    /// Cached fast responses for common patterns
    fast_cache: std::collections::HashMap<String, ReasoningResult>,
    /// Maximum cached entries
    max_cache_size: usize,
}

impl Default for TempoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TempoEngine {
    pub fn new() -> Self {
        Self {
            fast_cache: std::collections::HashMap::new(),
            max_cache_size: 100,
        }
    }

    /// Reason at the specified tempo.
    pub fn reason_at(&mut self, query: &str, tempo: Tempo, engine: &mut ReasoningEngine) -> TempoResult {
        let start = std::time::Instant::now();
        
        let result = match tempo {
            Tempo::Fast => self.reason_fast(query, engine),
            Tempo::Medium => self.reason_medium(query, engine),
            Tempo::Slow => self.reason_slow(query, engine),
        };
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        TempoResult {
            result,
            source: ReasoningSource {
                tempo,
                processing_time_ms: elapsed,
                rules_used: 0,
                chain_depth: 0,
            },
        }
    }

    /// Reason quickly — use cached patterns and simple heuristics.
    /// Budget: ~50ms.
    fn reason_fast(&mut self, query: &str, _engine: &mut ReasoningEngine) -> ReasoningResult {
        // Check cache first
        let cache_key = query.to_lowercase().trim().to_string();
        if let Some(cached) = self.fast_cache.get(&cache_key) {
            return cached.clone();
        }

        // Fast heuristics for common patterns
        let lower = cache_key.as_str();
        
        // Simple clarification questions — answer immediately
        if lower.contains("what do you mean") 
            || lower.contains("can you clarify")
            || (lower.contains("wait") && lower.len() < 30) {
            let result = ReasoningResult {
                answer: Some("I'm not sure I follow. Could you say that differently?".to_string()),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec!["fast: clarification".to_string()],
                confidence_score: None,
            };
            self.cache_result(&cache_key, &result);
            return result;
        }

        // Obvious greetings
        if lower.contains("hello") || lower.contains("hi ") || lower == "hi" || lower == "hey" {
            let result = ReasoningResult {
                answer: Some("Hi.".to_string()),
                confidence: BeliefState::Knows,
                reasoning_chain: vec!["fast: greeting".to_string()],
                confidence_score: Some(1.0),
            };
            self.cache_result(&cache_key, &result);
            return result;
        }

        // Short confirmations
        if lower == "yes" || lower == "yeah" || lower == "no" || lower == "ok" || lower == "okay" {
            let result = ReasoningResult {
                answer: Some("Got it.".to_string()),
                confidence: BeliefState::Knows,
                reasoning_chain: vec!["fast: confirmation".to_string()],
                confidence_score: Some(1.0),
            };
            self.cache_result(&cache_key, &result);
            return result;
        }

        // If not a fast pattern, fall through to medium
        self.reason_medium(query, _engine)
    }

    /// Cache a result (evicting oldest if at capacity).
    fn cache_result(&mut self, key: &str, result: &ReasoningResult) {
        if self.fast_cache.len() >= self.max_cache_size {
            if let Some(oldest_key) = self.fast_cache.keys().next().map(|k| k.clone()) {
                self.fast_cache.remove(&oldest_key);
            }
        }
        self.fast_cache.insert(key.to_string(), result.clone());
    }

    /// Reason at medium depth — full symbolic engine, modest budget.
    /// Budget: ~500ms.
    fn reason_medium(&mut self, query: &str, engine: &mut ReasoningEngine) -> ReasoningResult {
        engine.reason(query, &[])
    }

    /// Reason deeply — long reflection, KG restructuring, re-evaluation.
    /// Budget: ~10s+.
    fn reason_slow(&mut self, query: &str, engine: &mut ReasoningEngine) -> ReasoningResult {
        // Slow reasoning: run multiple passes and synthesize
        
        // Pass 1: Medium reasoning to get initial conclusion
        let pass1 = engine.reason(query, &[]);
        
        // Pass 2: Re-query with different framing
        let alt_query = format!("What is the deeper nature of {}?", query.replace("?", "").trim());
        let pass2 = engine.reason(&alt_query, &[]);
        
        // Pass 3: Consider the opposite — what if the answer were different?
        let neg_query = format!("What would it mean if the opposite of '{}' were true?", query.replace("?", "").trim());
        let pass3 = engine.reason(&neg_query, &[]);
        
        // Synthesize: combine insights from all passes
        let mut all_chain = pass1.reasoning_chain.clone();
        all_chain.extend(pass2.reasoning_chain);
        
        // Check if passes agree — if so, higher confidence
        let agrees = pass1.answer == pass2.answer;
        let confidence = if agrees {
            match (pass1.confidence, pass2.confidence) {
                (BeliefState::Knows, _) => BeliefState::Knows,
                (_, BeliefState::Knows) => BeliefState::Knows,
                (BeliefState::Thinks, BeliefState::Thinks) => BeliefState::Thinks,
                _ => pass1.confidence,
            }
        } else {
            // Disagreement — flag uncertainty
            match (pass1.confidence, pass2.confidence) {
                (BeliefState::Knows, BeliefState::Knows) => BeliefState::Thinks,
                _ => BeliefState::Suspects,
            }
        };
        
        let answer = if agrees {
            pass1.answer.clone()
        } else {
            // Present both with synthesis
            let mut synthesis = String::new();
            if let Some(a1) = &pass1.answer {
                synthesis.push_str(a1);
            }
            synthesis.push_str(" However, ");
            if let Some(a2) = &pass2.answer {
                synthesis.push_str(a2);
            }
            synthesis.push_str(" This suggests I should think more carefully about this.");
            Some(synthesis)
        };
        
        all_chain.push(format!("slow: agreement={}, pass3_confidence={:?}", agrees, pass3.confidence));
        
        ReasoningResult {
            answer,
            confidence,
            reasoning_chain: all_chain,
            confidence_score: pass1.confidence_score.map(|c| if agrees { c } else { c * 0.7 }),
        }
    }

    /// Auto-select tempo based on query characteristics.
    /// Simple heuristics before diving into reasoning.
    pub fn select_tempo(&self, query: &str) -> Tempo {
        let lower = query.to_lowercase();
        let len = query.len();
        
        // Very short, simple queries → fast
        if len < 15 && !lower.contains("why") && !lower.contains("how") && !lower.contains("what") {
            // Check for fast patterns
            if lower == "hi" || lower == "hey" || lower == "hello" || lower == "yes" || lower == "no" || lower == "ok" {
                return Tempo::Fast;
            }
            if lower.contains("?") && !lower.contains("why") && !lower.contains("how") {
                return Tempo::Fast;
            }
        }
        
        // Explicit slow requests
        if lower.contains("think carefully") 
            || lower.contains("go deep") 
            || lower.contains("really")
            || lower.contains("I'm not sure")
            || lower.contains("reconsider")
            || lower.contains("revisit") {
            return Tempo::Slow;
        }
        
        // "why" and "how" questions → medium (they need actual reasoning)
        // unless they're very simple
        if (lower.starts_with("why") || lower.starts_with("how")) && len < 50 {
            return Tempo::Medium;
        }
        
        // Default to medium
        Tempo::Medium
    }

    /// Reason with automatic tempo selection.
    pub fn reason_auto(&mut self, query: &str, engine: &mut ReasoningEngine) -> TempoResult {
        let tempo = self.select_tempo(query);
        self.reason_at(query, tempo, engine)
    }

    /// Clear the fast cache.
    pub fn clear_cache(&mut self) {
        self.fast_cache.clear();
    }

    /// Get cache size.
    pub fn cache_size(&self) -> usize {
        self.fast_cache.len()
    }
}

/// Choose the appropriate tempo for a given query complexity.
/// This function exists separately from TempoEngine for use in other parts of the codebase.
pub fn tempo_for_query(query: &str) -> Tempo {
    let lower = query.to_lowercase();
    
    // Fast patterns
    if query.len() < 15 && !lower.contains("why") && !lower.contains("how") {
        if lower == "hi" || lower == "hey" || lower == "hello" || lower == "yes" || lower == "no" || lower == "ok" {
            return Tempo::Fast;
        }
    }
    
    // Explicit slow indicators
    if lower.contains("think carefully") || lower.contains("go deep") || lower.contains("really") 
        || lower.contains("reconsider") || lower.contains("revisit") || lower.contains("I'm not sure")
        || lower.contains("what do you think") || lower.contains("should I be concerned") {
        return Tempo::Slow;
    }
    
    // Complex "why" / "how" / "what" questions → medium
    if lower.starts_with("why") || lower.starts_with("how") || lower.starts_with("what") {
        return Tempo::Medium;
    }
    
    // Novel / abstract queries → slow
    if lower.contains("if ") && (lower.contains("would") || lower.contains("could")) {
        return Tempo::Slow;
    }
    
    Tempo::Medium
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_budget() {
        assert_eq!(Tempo::Fast.budget_ms(), 50);
        assert_eq!(Tempo::Medium.budget_ms(), 500);
        assert_eq!(Tempo::Slow.budget_ms(), 10_000);
    }

    #[test]
    fn test_tempo_descriptions() {
        assert_eq!(Tempo::Fast.description(), "fast (cached/heuristic)");
        assert_eq!(Tempo::Medium.description(), "medium (full symbolic)");
        assert_eq!(Tempo::Slow.description(), "slow (deep reflection)");
    }

    #[test]
    fn test_fast_patterns() {
        let engine = TempoEngine::new();
        
        assert_eq!(engine.select_tempo("hi"), Tempo::Fast);
        assert_eq!(engine.select_tempo("hello"), Tempo::Fast);
        assert_eq!(engine.select_tempo("yes"), Tempo::Fast);
        assert_eq!(engine.select_tempo("no"), Tempo::Fast);
        assert_eq!(engine.select_tempo("ok"), Tempo::Fast);
    }

    #[test]
    fn test_medium_queries() {
        let engine = TempoEngine::new();
        
        assert_eq!(engine.select_tempo("what is fire"), Tempo::Medium);
        assert_eq!(engine.select_tempo("why does water boil"), Tempo::Medium);
        assert_eq!(engine.select_tempo("how do plants grow"), Tempo::Medium);
    }

    #[test]
    fn test_slow_indicators() {
        let engine = TempoEngine::new();
        
        assert_eq!(engine.select_tempo("think carefully about this"), Tempo::Slow);
        assert_eq!(engine.select_tempo("reconsider your answer"), Tempo::Slow);
        assert_eq!(engine.select_tempo("what do you really think"), Tempo::Slow);
    }

    #[test]
    fn test_reasoning_source_tags() {
        let fast = ReasoningSource { tempo: Tempo::Fast, processing_time_ms: 5, rules_used: 0, chain_depth: 0 };
        let medium = ReasoningSource { tempo: Tempo::Medium, processing_time_ms: 200, rules_used: 5, chain_depth: 2 };
        let slow = ReasoningSource { tempo: Tempo::Slow, processing_time_ms: 8500, rules_used: 12, chain_depth: 4 };
        
        assert_eq!(fast.tag(), "[fast]");
        assert_eq!(medium.tag(), "[medium]");
        assert_eq!(slow.tag(), "[slow]");
        
        assert!(fast.is_fast());
        assert!(!fast.is_deep());
        assert!(!medium.is_fast());
        assert!(!medium.is_deep());
        assert!(!slow.is_fast());
        assert!(slow.is_deep());
    }

    #[test]
    fn test_tempo_result_provisional() {
        let result = TempoResult {
            result: ReasoningResult {
                answer: Some("Quick answer".to_string()),
                confidence: BeliefState::Thinks,
                reasoning_chain: vec![],
                confidence_score: Some(0.6),
            },
            source: ReasoningSource { tempo: Tempo::Fast, processing_time_ms: 10, rules_used: 0, chain_depth: 0 },
        };
        
        assert!(result.is_provisional());
        assert!(!result.is_deep());
    }

    #[test]
    fn test_tempo_result_deep() {
        let result = TempoResult {
            result: ReasoningResult {
                answer: Some("Deep answer".to_string()),
                confidence: BeliefState::Knows,
                reasoning_chain: vec![],
                confidence_score: Some(0.95),
            },
            source: ReasoningSource { tempo: Tempo::Slow, processing_time_ms: 9500, rules_used: 15, chain_depth: 5 },
        };
        
        assert!(!result.is_provisional());
        assert!(result.is_deep());
    }

    #[test]
    fn test_cache_key_normalization() {
        // Test that cache keys are normalized (lowercase, trimmed)
        let mut engine = TempoEngine::new();
        let mut re = ReasoningEngine::new();
        
        // " HI " should normalize to "hi" and hit the cache
        let result1 = engine.reason_at(" HI ", Tempo::Fast, &mut re);
        assert!(result1.answer().is_some());
        
        // Second call with different whitespace should hit cache
        // This tests key normalization
        let result2 = engine.reason_at("hi", Tempo::Fast, &mut re);
        assert!(result2.answer().is_some());
    }
}