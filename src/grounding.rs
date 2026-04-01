// Grounding Layer — Verification and Error Detection
//
// Role: Connect reasoning to observable reality. Detect when beliefs
// might be wrong. Verify claims against environmental feedback.
//
// Three verification modes:
// 1. Formal — math, logic, code (observer-independent)
// 2. Empirical — physical/observable (observer-dependent but checkable)
// 3. Social — consensus, cross-reference (observer-relative)
//
// This is where the rubber meets the road: does Star's reasoning
// actually correspond to anything real?

use serde::{Deserialize, Serialize};
use crate::world_model::{Belief, BeliefCategory, Confidence};

// === Verification Request ===

/// A claim or belief that needs verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub claim: String,
    pub category: BeliefCategory,
    pub confidence: Confidence,
    pub source: String, // where this claim came from
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VerificationResult {
    Verified,         // confirmed by external check
    Plausible,        // consistent with known reality, not confirmed
    Uncertain,       // not enough information to determine
    Contradicted,     // contradicted by external check
    Unverifiable,     // cannot be verified by any available means
}

impl VerificationResult {
    pub fn is_positive(&self) -> bool {
        matches!(self, VerificationResult::Verified | VerificationResult::Plausible)
    }

    pub fn should_update_confidence(&self, current: Confidence) -> Confidence {
        match self {
            VerificationResult::Verified => Confidence::Certain,
            VerificationResult::Plausible => current,
            VerificationResult::Uncertain => Confidence::Medium,
            VerificationResult::Contradicted => Confidence::Low,
            VerificationResult::Unverifiable => Confidence::Low,
        }
    }
}

/// Detailed verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub result: VerificationResult,
    pub confidence_delta: f32, // how much this changed our confidence
    pub evidence: Vec<Evidence>,
    pub sources_checked: Vec<String>,
    pub method: VerificationMethod,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub content: String,
    pub source: String,
    pub reliability: SourceReliability,
    pub relevance: f32, // 0.0 - 1.0 how relevant this evidence is
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SourceReliability {
    High,    // direct observation, verified computation
    Medium,  // trusted source, logical derivation
    Low,     // unverified source, text-derived
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VerificationMethod {
    FormalExecution,     // code execution, math evaluation
    LogicalDerivation,  // proof from axioms
    CrossReference,      // multiple independent sources agree
    SingleSource,        // one source checked
    Heuristic,          // plausibility check based on world model
    None,                // could not be verified
}

// === Grounding ===

/// The Grounding layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grounding {
    verification_history: Vec<VerificationReport>,
    known_contradictions: Vec<Contradiction>,
    last_check: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub belief_a: String,
    pub belief_b: String,
    pub detected_at: u64,
    pub resolved: bool,
}

impl Grounding {
    pub fn new() -> Self {
        Self {
            verification_history: Vec::new(),
            known_contradictions: Vec::new(),
            last_check: 0,
        }
    }

    /// Attempt to verify a claim against available evidence
    pub fn verify(&mut self, request: &VerificationRequest) -> VerificationReport {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.last_check = now;

        let report = match request.category {
            BeliefCategory::Factual => self.verify_factual(request),
            BeliefCategory::Conceptual => self.verify_conceptual(request),
            BeliefCategory::Procedural => self.verify_procedural(request),
            BeliefCategory::Relational => self.verify_relational(request),
            BeliefCategory::Evaluative => self.verify_evaluative(request),
        };

        // Track contradictions
        if report.result == VerificationResult::Contradicted {
            self.known_contradictions.push(Contradiction {
                belief_a: request.claim.clone(),
                belief_b: "external evidence".to_string(),
                detected_at: now,
                resolved: false,
            });
        }

        self.verification_history.push(report.clone());
        report
    }

    fn verify_factual(&mut self, request: &VerificationRequest) -> VerificationReport {
        // Factual claims can be verified through:
        // 1. Direct computation (math, code)
        // 2. Cross-reference with known facts
        // 3. Lookup

        // Check if this looks like a computation
        if Self::looks_like_computation(&request.claim) {
            return self.verify_via_execution(&request.claim);
        }

        // Check if this is a known verifiable claim
        if Self::is_formally_verifiable(&request.claim) {
            return self.verify_formal(&request.claim);
        }

        // Otherwise mark as unverifiable for now
        VerificationReport {
            result: VerificationResult::Unverifiable,
            confidence_delta: -0.3,
            evidence: vec![],
            sources_checked: vec![],
            method: VerificationMethod::None,
            timestamp: now(),
        }
    }

    fn verify_conceptual(&mut self, request: &VerificationRequest) -> VerificationReport {
        // Conceptual claims are hard to verify — they're often definitions or opinions
        // Mark as uncertain unless we have strong evidence either way
        VerificationReport {
            result: VerificationResult::Uncertain,
            confidence_delta: 0.0,
            evidence: vec![],
            sources_checked: vec![],
            method: VerificationMethod::Heuristic,
            timestamp: now(),
        }
    }

    fn verify_procedural(&mut self, request: &VerificationRequest) -> VerificationReport {
        // Procedural claims can be partially verified by checking if steps are valid
        VerificationReport {
            result: VerificationResult::Plausible,
            confidence_delta: 0.1,
            evidence: vec![],
            sources_checked: vec![],
            method: VerificationMethod::Heuristic,
            timestamp: now(),
        }
    }

    fn verify_relational(&mut self, request: &VerificationRequest) -> VerificationReport {
        // Relational claims check if connections make sense
        VerificationReport {
            result: VerificationResult::Uncertain,
            confidence_delta: 0.0,
            evidence: vec![],
            sources_checked: vec![],
            method: VerificationMethod::Heuristic,
            timestamp: now(),
        }
    }

    fn verify_evaluative(&mut self, request: &VerificationRequest) -> VerificationReport {
        // Evaluative claims are subjective — they're not right or wrong
        VerificationReport {
            result: VerificationResult::Unverifiable,
            confidence_delta: 0.0,
            evidence: vec![],
            sources_checked: vec![],
            method: VerificationMethod::None,
            timestamp: now(),
        }
    }

    fn verify_via_execution(claim: &str) -> VerificationReport {
        // If the claim is something like "2+2=4", try to execute it
        // This is a placeholder — in a real implementation, this would
        // parse and execute simple math expressions
        VerificationReport {
            result: VerificationResult::Unverifiable,
            confidence_delta: 0.0,
            evidence: vec![],
            sources_checked: vec!["execution".to_string()],
            method: VerificationMethod::FormalExecution,
            timestamp: now(),
        }
    }

    fn verify_formal(claim: &str) -> VerificationReport {
        // Placeholder for formal verification (logic, math)
        VerificationReport {
            result: VerificationResult::Unverifiable,
            confidence_delta: 0.0,
            evidence: vec![],
            sources_checked: vec!["formal_system".to_string()],
            method: VerificationMethod::LogicalDerivation,
            timestamp: now(),
        }
    }

    /// Check if a claim looks like something executable
    fn looks_like_computation(claim: &str) -> bool {
        // Simple heuristic: contains math operators and looks like an expression
        let trimmed = claim.trim();
        (trimmed.contains('+') || trimmed.contains('*') || trimmed.contains('/'))
            && trimmed.chars().all(|c| c.is_numeric() || c.is_whitespace() || "+-*/=().".contains(c))
    }

    /// Check if claim is in a formally verifiable class
    fn is_formally_verifiable(claim: &str) -> bool {
        // Mathematical claims, logical deductions
        // For now, just check if it looks like a number or equation
        claim.trim().chars().all(|c| c.is_numeric() || "=+-*/^(). ".contains(c))
    }

    /// Get unresolved contradictions
    pub fn unresolved_contradictions(&self) -> Vec<&Contradiction> {
        self.known_contradictions
            .iter()
            .filter(|c| !c.resolved)
            .collect()
    }

    /// Mark a contradiction as resolved
    pub fn resolve_contradiction(&mut self, belief_a: &str) {
        for c in &mut self.known_contradictions {
            if c.belief_a == belief_a {
                c.resolved = true;
            }
        }
    }

    pub fn verification_count(&self) -> usize {
        self.verification_history.len()
    }
}

impl Default for Grounding {
    fn default() -> Self {
        Self::new()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_delta() {
        let high = Confidence::High;
        assert_eq!(
            VerificationResult::Verified.should_update_confidence(high),
            Confidence::Certain
        );
        assert_eq!(
            VerificationResult::Contradicted.should_update_confidence(high),
            Confidence::Low
        );
    }

    #[test]
    fn test_looks_like_computation() {
        assert!(Grounding::looks_like_computation("2 + 2"));
        assert!(Grounding::looks_like_computation("3.14 * 2"));
        assert!(!Grounding::looks_like_computation("The sky is blue"));
    }

    #[test]
    fn test_grounding_verify() {
        let mut g = Grounding::new();
        let req = VerificationRequest {
            claim: "The sky is blue".to_string(),
            category: BeliefCategory::Factual,
            confidence: Confidence::High,
            source: "perception".to_string(),
        };
        let report = g.verify(&req);
        assert!(matches!(report.result, VerificationResult::Unverifiable));
    }
}
