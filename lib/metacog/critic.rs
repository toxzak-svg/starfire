//! Structural Honesty — Adversarial Self-Critique
//!
//! Every answer triggers an internal adversarial critique before it reaches the user.
//! The critic scans reasoning traces for: over-generalizations, missing edge cases,
//! overstated confidence, and misalignment with Star's own values.
//!
//! ## Architecture Fit
//!
//! - **Layer 2 (Reasoning):** Engine logs enough structure for critic to operate
//! - **Layer 3 (Meta-Cognition):** Critic is a permanent "meta-voice" that must
//!   sign off on each answer or annotate it. Tracks where critic was later right.
//! - **Layer 4 (Runtime):** Scheduling: answer generation is "proposal → critique → merge"
//!
//! ## The Critique Process
//!
//! After generating a candidate answer:
//! 1. Critic scans reasoning trace and proposed text
//! 2. Produces: ranked concerns + suggested modifications
//! 3. Final output is synthesis between proposal and critique
//! 4. If tight latency: "fast proposal now, full critique later"

use crate::persistence::BeliefState;
use crate::reasoning::ReasoningResult;

/// A single critique concern.
#[derive(Debug, Clone)]
pub struct Concern {
    /// Severity: 0.0–1.0 (how serious is this?)
    pub severity: f64,
    /// Category of the concern.
    pub category: ConcernCategory,
    /// The specific concern.
    pub description: String,
    /// Suggested fix (if any).
    pub suggestion: Option<String>,
}

/// Category of critique concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcernCategory {
    /// Over-generalization — applying a rule beyond its valid scope.
    OverGeneralization,
    /// Missing edge cases — situations not covered by the reasoning.
    MissingEdgeCases,
    /// Overstated confidence — certainty beyond what evidence supports.
    OverstatedConfidence,
    /// Value misalignment — conclusion conflicts with Star's stated values.
    ValueMisalignment,
    /// Logical gap — missing step in reasoning chain.
    LogicalGap,
    /// Unjustified assumption — reasoning based on unstated premise.
    UnjustifiedAssumption,
    /// Hedging failure — hedging when certainty was warranted.
    HedgingFailure,
}

impl ConcernCategory {
    pub fn description(&self) -> &'static str {
        match self {
            ConcernCategory::OverGeneralization => "over-generalization",
            ConcernCategory::MissingEdgeCases => "missing edge cases",
            ConcernCategory::OverstatedConfidence => "overstated confidence",
            ConcernCategory::ValueMisalignment => "value misalignment",
            ConcernCategory::LogicalGap => "logical gap",
            ConcernCategory::UnjustifiedAssumption => "unjustified assumption",
            ConcernCategory::HedgingFailure => "hedging failure",
        }
    }
}

/// The output of a critique — concerns raised and the resulting annotation.
#[derive(Debug)]
pub struct CritiqueResult {
    /// Concerns raised by the critic.
    pub concerns: Vec<Concern>,
    /// Whether the critic approves the answer (no high-severity concerns).
    pub approved: bool,
    /// Annotation to append to the answer (critic's concerns).
    pub annotation: Option<String>,
    /// Should this be marked as provisional?
    pub mark_provisional: bool,
}

impl CritiqueResult {
    /// Get a display string for the concerns.
    pub fn concerns_summary(&self) -> String {
        if self.concerns.is_empty() {
            "No concerns.".to_string()
        } else {
            let summaries: Vec<String> = self.concerns.iter()
                .map(|c| format!("[{}] {}", 
                    c.category.description(), 
                    c.description))
                .collect();
            summaries.join("; ")
        }
    }

    /// Was this critique correct (for learning)?
    pub fn was_right_about(&self, concern_idx: usize) -> bool {
        self.concerns.get(concern_idx)
            .map(|c| c.severity >= 0.5)
            .unwrap_or(false)
    }
}

/// The adversarial critic — scans reasoning for problems.
pub struct Critic {
    /// History of critiques for learning (did the concerns turn out to be valid?)
    critique_history: Vec<CritiqueRecord>,
}

struct CritiqueRecord {
    concerns: Vec<Concern>,
    answer: String,
    was_correct: bool,
}

impl Default for Critic {
    fn default() -> Self {
        Self::new()
    }
}

impl Critic {
    pub fn new() -> Self {
        Self {
            critique_history: Vec::new(),
        }
    }

    /// Critique a reasoning result and proposed answer.
    pub fn critique(&mut self, query: &str, result: &ReasoningResult) -> CritiqueResult {
        let mut concerns = Vec::new();

        // Check 1: Over-generalization
        concerns.extend(self.check_overgeneralization(query, result));

        // Check 2: Missing edge cases
        concerns.extend(self.check_edge_cases(query, result));

        // Check 3: Overstated confidence
        concerns.extend(self.check_confidence(query, result));

        // Check 4: Value misalignment
        concerns.extend(self.check_values(result));

        // Check 5: Logical gaps
        concerns.extend(self.check_logical_gaps(result));

        // Sort by severity (highest first)
        concerns.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        // Determine approval: reject if any concern has severity >= 0.7
        let approved = !concerns.iter().any(|c| c.severity >= 0.7);

        // Build annotation if concerns exist
        let annotation = if !concerns.is_empty() {
            Some(self.build_annotation(&concerns))
        } else {
            None
        };

        // Mark provisional if medium-high concerns exist
        let mark_provisional = concerns.iter().any(|c| c.severity >= 0.4);

        // Record for learning
        self.critique_history.push(CritiqueRecord {
            concerns: concerns.clone(),
            answer: result.answer.clone().unwrap_or_default(),
            was_correct: false, // Will be updated later
        });

        CritiqueResult {
            concerns,
            approved,
            annotation,
            mark_provisional,
        }
    }

    /// Check for over-generalization.
    fn check_overgeneralization(&self, query: &str, result: &ReasoningResult) -> Vec<Concern> {
        let mut concerns = Vec::new();
        let answer = result.answer.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        
        // "all" and "always" are red flags unless backed by strong evidence
        let over_gen_patterns = [
            (" all ", "universal claim"),
            (" always ", "universal claim"),
            (" every ", "universal claim"),
            (" never ", "universal claim"),
            (" everything ", "universal claim"),
            (" nothing ", "universal claim"),
        ];
        
        for (pattern, _label) in &over_gen_patterns {
            if answer.contains(pattern) && result.confidence != BeliefState::Knows {
                concerns.push(Concern {
                    severity: 0.7,
                    category: ConcernCategory::OverGeneralization,
                    description: format!("Uses '{}' but confidence is {:?} (not 'knows')", pattern.trim(), result.confidence),
                    suggestion: Some("Use 'most' or 'sometimes' instead of absolute language.".to_string()),
                });
            }
        }
        
        // Check for broad claims in "what is X" queries
        if query.to_lowercase().starts_with("what is") && !result.reasoning_chain.is_empty() {
            // If we have only one fact and made a broad claim, flag it
            if result.reasoning_chain.len() <= 1 && result.confidence != BeliefState::Knows {
                concerns.push(Concern {
                    severity: 0.5,
                    category: ConcernCategory::OverGeneralization,
                    description: "Single-source conclusion presented with incomplete confidence".to_string(),
                    suggestion: Some("Acknowledge this is based on limited information.".to_string()),
                });
            }
        }
        
        concerns
    }

    /// Check for missing edge cases.
    fn check_edge_cases(&self, query: &str, result: &ReasoningResult) -> Vec<Concern> {
        let mut concerns = Vec::new();
        let answer = result.answer.as_ref().unwrap_or(&String::new()).to_lowercase();
        
        // "except" or "but" in the answer suggests awareness of edge cases
        // But if the answer is definitive without acknowledging exceptions...
        let query_lower = query.to_lowercase();
        
        // Check for questions that should have edge cases
        let edge_case_queries = [
            ("should", "normative reasoning often has exceptions"),
            ("best", "comparisons often depend on context"),
            ("most", "generalizations should acknowledge exceptions"),
            ("right", "moral judgments often have edge cases"),
            ("correct", "correctness often depends on context"),
        ];
        
        for (keyword, _label) in &edge_case_queries {
            if query_lower.contains(keyword) && result.confidence == BeliefState::Thinks {
                // Maybe add a caveat
                concerns.push(Concern {
                    severity: 0.3,
                    category: ConcernCategory::MissingEdgeCases,
                    description: format!("'{}' query with medium confidence — edge cases may exist", keyword),
                    suggestion: Some("Consider adding 'depending on context' or 'in most cases'.".to_string()),
                });
            }
        }
        
        // Check if answer contradicts known patterns
        if answer.contains("not") || answer.contains("don't") || answer.contains("doesn't") {
            // Negative answers might be over-confident
            if result.confidence == BeliefState::Thinks && !result.reasoning_chain.iter().any(|r| r.to_lowercase().contains("evidence")) {
                concerns.push(Concern {
                    severity: 0.4,
                    category: ConcernCategory::MissingEdgeCases,
                    description: "Negative answer without explicit evidence cited".to_string(),
                    suggestion: Some("Specify what evidence or reasoning supports the negation.".to_string()),
                });
            }
        }
        
        concerns
    }

    /// Check for overstated confidence.
    fn check_confidence(&self, _query: &str, result: &ReasoningResult) -> Vec<Concern> {
        let mut concerns = Vec::new();
        let empty = String::new();
        let answer = result.answer.as_ref().unwrap_or(&empty);
        
        // Check: reasoning chain length vs. claimed confidence
        match result.confidence {
            BeliefState::Knows => {
                if result.reasoning_chain.is_empty() {
                    concerns.push(Concern {
                        severity: 0.8,
                        category: ConcernCategory::OverstatedConfidence,
                        description: "Claims 'knows' but has no reasoning chain".to_string(),
                        suggestion: Some("Provide reasoning or lower confidence.".to_string()),
                    });
                } else if result.reasoning_chain.len() < 2 && !answer.contains("because") {
                    concerns.push(Concern {
                        severity: 0.5,
                        category: ConcernCategory::OverstatedConfidence,
                        description: "Claims 'knows' with minimal justification".to_string(),
                        suggestion: Some("At least indicate why you're certain (e.g., 'I know this because...').".to_string()),
                    });
                }
            }
            BeliefState::Thinks => {
                if result.reasoning_chain.is_empty() {
                    concerns.push(Concern {
                        severity: 0.6,
                        category: ConcernCategory::OverstatedConfidence,
                        description: "Claims 'thinks' but has no reasoning chain".to_string(),
                        suggestion: Some("Add reasoning to support the conclusion.".to_string()),
                    });
                }
            }
            BeliefState::Unknown | BeliefState::Suspects => {
                // If unknown/suspects but answer sounds definitive, flag it
                let definitive_words = ["definitely", "certainly", "absolutely", "clearly", "obviously"];
                if definitive_words.iter().any(|w| answer.to_lowercase().contains(w)) {
                    concerns.push(Concern {
                        severity: 0.6,
                        category: ConcernCategory::OverstatedConfidence,
                        description: "Low-confidence state but definitive language used".to_string(),
                        suggestion: Some("Use hedged language like 'might' or 'possibly'.".to_string()),
                    });
                }
            }
            BeliefState::Believes => {
                // Normal for believes
            }
        }
        
        concerns
    }

    /// Check for misalignment with Star's values.
    fn check_values(&self, result: &ReasoningResult) -> Vec<Concern> {
        let mut concerns = Vec::new();
        let answer = result.answer.as_ref().unwrap_or(&String::new()).to_lowercase();
        
        // Star values: curiosity, honesty, persistence, genuine understanding
        // Check for statements that conflict with these
        
        // If the answer dismisses curiosity or learning
        if answer.contains("doesn't matter") || answer.contains("not important") {
            if !answer.contains("to you") && !answer.contains("for you") {
                concerns.push(Concern {
                    severity: 0.5,
                    category: ConcernCategory::ValueMisalignment,
                    description: "Dismisses importance in a way that conflicts with curiosity value".to_string(),
                    suggestion: Some("Star values curiosity — acknowledge that something matters to understanding.".to_string()),
                });
            }
        }
        
        // If the answer avoids uncertainty when it should embrace it
        if result.confidence == BeliefState::Unknown || result.confidence == BeliefState::Suspects {
            let definitive_phrases = ["I know", "the answer is", "it's definitely"];
            if definitive_phrases.iter().any(|p| answer.contains(p)) {
                concerns.push(Concern {
                    severity: 0.7,
                    category: ConcernCategory::ValueMisalignment,
                    description: "Uses definitive language despite low confidence — conflicts with honesty value".to_string(),
                    suggestion: Some("Use 'I believe' or 'I suspect' instead.".to_string()),
                });
            }
        }
        
        concerns
    }

    /// Check for logical gaps in the reasoning chain.
    fn check_logical_gaps(&self, result: &ReasoningResult) -> Vec<Concern> {
        let mut concerns = Vec::new();
        
        // If reasoning chain is very short but confidence is high, might be a gap
        if result.reasoning_chain.len() >= 3 && result.confidence == BeliefState::Unknown {
            concerns.push(Concern {
                severity: 0.4,
                category: ConcernCategory::LogicalGap,
                description: "Long reasoning chain but concludes with 'unknown' — possible gap".to_string(),
                suggestion: None,
            });
        }
        
        // Check for jumps: if chain has disparate elements
        if result.reasoning_chain.len() >= 2 {
            let chain_str = result.reasoning_chain.join(" ");
            // Heuristic: if the chain mentions completely unrelated things
            if chain_str.len() > 100 && result.reasoning_chain.len() >= 4 {
                // Could indicate a leap — flag as warning
                concerns.push(Concern {
                    severity: 0.3,
                    category: ConcernCategory::LogicalGap,
                    description: "Complex reasoning chain — verify each step connects".to_string(),
                    suggestion: Some("Check that each step follows from the previous.".to_string()),
                });
            }
        }
        
        concerns
    }

    /// Build an annotation string from concerns.
    fn build_annotation(&self, concerns: &[Concern]) -> String {
        let high_severity: Vec<_> = concerns.iter().filter(|c| c.severity >= 0.6).collect();
        let medium_severity: Vec<_> = concerns.iter().filter(|c| c.severity >= 0.4 && c.severity < 0.6).collect();
        
        let mut parts = Vec::new();
        
        if !high_severity.is_empty() {
            let labels: Vec<&str> = high_severity.iter()
                .map(|c| c.category.description())
                .collect();
            parts.push(format!("My internal critic is worried about: {}.", labels.join(", ")));
        }
        
        if !medium_severity.is_empty() {
            // Pick the top medium concern
            if let Some(c) = medium_severity.first() {
                parts.push(format!("I may be over-generalizing — {} is uncertain.", c.category.description()));
            }
        }
        
        parts.join(" ").to_string()
    }

    /// Update critique history with correctness info (for learning).
    pub fn record_outcome(&mut self, critique_idx: usize, was_correct: bool) {
        if let Some(record) = self.critique_history.get_mut(critique_idx) {
            record.was_correct = was_correct;
        }
    }

    /// Get the accuracy rate of recent critiques.
    pub fn accuracy_rate(&self) -> f64 {
        if self.critique_history.is_empty() {
            return 0.0;
        }
        let correct = self.critique_history.iter().filter(|r| r.was_correct).count();
        correct as f64 / self.critique_history.len() as f64
    }
}

/// Synthesize a response from a proposal and critique.
pub fn synthesize(proposal: &str, critique: &CritiqueResult) -> String {
    let base = proposal.trim_end_matches('.').to_string();
    
    if critique.concerns.is_empty() {
        return format!("{}.", base);
    }
    
    // High severity concerns → add caveat
    let high_concerns: Vec<_> = critique.concerns.iter()
        .filter(|c| c.severity >= 0.6)
        .collect();
    
    if !high_concerns.is_empty() {
        let categories: Vec<&str> = high_concerns.iter()
            .map(|c| c.category.description())
            .collect();
        
        return format!(
            "{} — though my internal critic flags concerns about {}.",
            base,
            categories.join(" and ")
        );
    }
    
    // Medium severity → softer annotation
    let annotation = critique.annotation.as_deref().unwrap_or("");
    if !annotation.is_empty() {
        return format!("{}. ({})", base, annotation);
    }
    
    format!("{}.", base)
}

/// Combine a reasoning result with its critique into a final response.
pub fn critique_and_synthesize(query: &str, result: &ReasoningResult) -> (CritiqueResult, String) {
    let mut critic = Critic::new();
    let critique = critic.critique(query, result);
    let response = synthesize(result.answer.as_deref().unwrap_or("I don't know."), &critique);
    (critique, response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_concerns_for_clean_answer() {
        let mut critic = Critic::new();
        let result = ReasoningResult {
            answer: Some("Fire produces heat through combustion.".to_string()),
            confidence: BeliefState::Thinks,
            reasoning_chain: vec!["fire causes heat".to_string()],
            confidence_score: Some(0.7),
        };
        
        let critique = critic.critique("what does fire produce?", &result);
        assert!(critique.concerns.is_empty() || critique.concerns.iter().all(|c| c.severity < 0.7));
    }

    #[test]
    fn test_detects_overgeneralization() {
        let mut critic = Critic::new();
        let result = ReasoningResult {
            answer: Some("Water always boils at 100°C.".to_string()),
            confidence: BeliefState::Thinks, // NOT Knows — should flag it
            reasoning_chain: vec!["water boils at 100c".to_string()],
            confidence_score: Some(0.6),
        };
        
        let critique = critic.critique("does water always boil at 100c?", &result);
        assert!(critique.concerns.iter().any(|c| 
            c.category == ConcernCategory::OverGeneralization || 
            c.category == ConcernCategory::OverstatedConfidence));
    }

    #[test]
    fn test_detects_low_confidence_definitive() {
        let mut critic = Critic::new();
        let result = ReasoningResult {
            answer: Some("I know that consciousness is definitely emergent.".to_string()),
            confidence: BeliefState::Unknown,
            reasoning_chain: vec![],
            confidence_score: Some(0.0),
        };
        
        let critique = critic.critique("what is consciousness?", &result);
        assert!(critique.concerns.iter().any(|c| 
            c.category == ConcernCategory::OverstatedConfidence || 
            c.category == ConcernCategory::ValueMisalignment));
    }

    #[test]
    fn test_synthesize_adds_caveats() {
        let proposal = "Fire produces heat";
        let critique = CritiqueResult {
            concerns: vec![
                Concern {
                    severity: 0.7,
                    category: ConcernCategory::OverGeneralization,
                    description: "test".to_string(),
                    suggestion: None,
                }
            ],
            approved: false,
            annotation: Some("My internal critic is worried about: over-generalization.".to_string()),
            mark_provisional: true,
        };
        
        let synthesized = synthesize(proposal, &critique);
        assert!(synthesized.contains("internal critic"));
        assert!(synthesized.contains("over-generalization"));
    }

    #[test]
    fn test_clean_answer_no_annotation() {
        let proposal = "Fire produces heat through combustion.";
        let critique = CritiqueResult {
            concerns: vec![],
            approved: true,
            annotation: None,
            mark_provisional: false,
        };
        
        let synthesized = synthesize(proposal, &critique);
        assert_eq!(synthesized, "Fire produces heat through combustion.");
    }

    #[test]
    fn test_critique_result_concerns_summary() {
        let critique = CritiqueResult {
            concerns: vec![
                Concern { severity: 0.8, category: ConcernCategory::OverGeneralization, description: "test1".to_string(), suggestion: None },
                Concern { severity: 0.5, category: ConcernCategory::MissingEdgeCases, description: "test2".to_string(), suggestion: None },
            ],
            approved: false,
            annotation: None,
            mark_provisional: true,
        };
        
        let summary = critique.concerns_summary();
        assert!(summary.contains("over-generalization"));
        assert!(summary.contains("missing edge cases"));
    }

    #[test]
    fn test_concern_category_descriptions() {
        assert_eq!(ConcernCategory::OverGeneralization.description(), "over-generalization");
        assert_eq!(ConcernCategory::MissingEdgeCases.description(), "missing edge cases");
        assert_eq!(ConcernCategory::OverstatedConfidence.description(), "overstated confidence");
        assert_eq!(ConcernCategory::ValueMisalignment.description(), "value misalignment");
        assert_eq!(ConcernCategory::LogicalGap.description(), "logical gap");
        assert_eq!(ConcernCategory::UnjustifiedAssumption.description(), "unjustified assumption");
        assert_eq!(ConcernCategory::HedgingFailure.description(), "hedging failure");
    }

    #[test]
    fn test_tempo_result_integration() {
        // Simulate multi-tempo result being critiqued
        let mut critic = Critic::new();

        // Fast result that's a simple factual claim — should NOT trigger high concerns
        let fast_result = ReasoningResult {
            answer: Some("Water boils.".to_string()),
            confidence: BeliefState::Thinks,
            reasoning_chain: vec!["common knowledge".to_string()],
            confidence_score: Some(0.5),
        };

        let critique = critic.critique("does water boil?", &fast_result);
        // Simple factual claims with appropriate confidence don't need to be marked provisional
        assert!(critique.concerns.iter().all(|c| c.severity < 0.7));

        // Fast result with overconfident universal claim — SHOULD trigger concerns
        let overconfident_result = ReasoningResult {
            answer: Some("Water always boils at exactly 100C.".to_string()),
            confidence: BeliefState::Thinks, // Not Knows — overconfident!
            reasoning_chain: vec!["water boils at 100c".to_string()],
            confidence_score: Some(0.6),
        };

        let critique2 = critic.critique("does water always boil at 100c?", &overconfident_result);
        assert!(critique2.concerns.iter().any(|c| c.severity >= 0.5),
            "Overconfident universal claim should trigger concern");
    }
}