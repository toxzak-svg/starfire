//! Fractured Reasoning Pathways — R&D-E
//!
//! Inspired by SRMoE's multi-head architecture: separate processing pathways
//! that run in parallel and fuse for final decision.
//!
//! Star's pathways:
//! - LOGIC_PATH: deductive/rule-based reasoning
//! - ANALOGY_PATH: structure mapping from known to unknown
//! - ABDUCTION_PATH: hypothesize best explanation
//! - SYNTHESIS_PATH: novel combination of disparate ideas
//!
//! Each pathway produces a vote with confidence. Fusion combines votes
//! into a final answer, weighted by pathway confidence and recency.

use crate::persistence::Memory;
use std::collections::HashMap;

/// A reasoning pathway — separate processor for a type of reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pathway {
    /// Deductive/logical reasoning — rule-based
    Logic,
    /// Analogical reasoning — structure mapping
    Analogy,
    /// Abductive reasoning — hypothesize explanations
    Abduction,
    /// Novel synthesis — creative combination
    Synthesis,
}

impl Pathway {
    pub fn all() -> &'static [Pathway] {
        &[
            Pathway::Logic,
            Pathway::Analogy,
            Pathway::Abduction,
            Pathway::Synthesis,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Pathway::Logic => "logic",
            Pathway::Analogy => "analogy",
            Pathway::Abduction => "abduction",
            Pathway::Synthesis => "synthesis",
        }
    }
}

/// A vote from a pathway — its contribution to the final answer.
#[derive(Debug, Clone)]
pub struct PathwayVote {
    /// Which pathway produced this vote
    pub pathway: Pathway,
    /// The contribution (partial answer, explanation, or question)
    pub content: String,
    /// Confidence in this vote (0-1)
    pub confidence: f64,
    /// Evidence supporting this vote
    pub evidence: Vec<String>,
    /// Whether this vote should override others (strong signal)
    pub is_strong: bool,
}

impl PathwayVote {
    pub fn new(pathway: Pathway, content: &str, confidence: f64) -> Self {
        Self {
            pathway,
            content: content.to_string(),
            confidence,
            evidence: Vec::new(),
            is_strong: false,
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<&str>) -> Self {
        self.evidence = evidence.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn strong(mut self) -> Self {
        self.is_strong = true;
        self
    }
}

/// Fusion engine — combines pathway votes into a final answer.
#[derive(Clone)]
pub struct PathwayFusion {
    /// Historical pathway confidence (recency-weighted)
    pathway_trust: HashMap<Pathway, f64>,
    /// Maximum evidence to keep per vote
    max_evidence: usize,
}

impl PathwayFusion {
    pub fn new() -> Self {
        let mut fusion = Self {
            pathway_trust: HashMap::new(),
            max_evidence: 5,
        };
        // Start with equal trust
        for pathway in Pathway::all() {
            fusion.pathway_trust.insert(*pathway, 0.5);
        }
        fusion
    }

    /// Fuse multiple pathway votes into a final answer.
    pub fn fuse(&mut self, votes: Vec<PathwayVote>, query_type: &str) -> FusedResult {
        if votes.is_empty() {
            return FusedResult {
                answer: "I don't know how to approach this.".to_string(),
                confidence: crate::persistence::memory::BeliefState::Unknown,
                pathway_contributions: HashMap::new(),
                reasoning_chain: Vec::new(),
                conflict_detected: false,
            };
        }

        // Check for conflicts between pathways
        let conflict_detected = self.detect_conflict(&votes);

        // Weight votes by pathway trust and confidence
        let mut weighted_scores: HashMap<Pathway, f64> = HashMap::new();
        for vote in &votes {
            let trust = *self.pathway_trust.get(&vote.pathway).unwrap_or(&0.5);
            let score = vote.confidence * trust * if vote.is_strong { 1.5 } else { 1.0 };
            *weighted_scores.entry(vote.pathway).or_insert(0.0) += score;
        }

        // Find the winning pathway
        let winner = weighted_scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(p, _)| *p);

        // Build the answer from all votes
        let answer = self.build_answer(&votes, winner, query_type);
        
        // Determine overall confidence
        let overall_confidence = if conflict_detected {
            crate::persistence::memory::BeliefState::Suspects
        } else {
            let max_score = weighted_scores.values()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .copied()
                .unwrap_or(0.5);
            match max_score {
                s if s > 0.8 => crate::persistence::memory::BeliefState::Knows,
                s if s > 0.6 => crate::persistence::memory::BeliefState::Thinks,
                s if s > 0.4 => crate::persistence::memory::BeliefState::Believes,
                _ => crate::persistence::memory::BeliefState::Suspects,
            }
        };

        // Update pathway trust based on what won
        self.update_trust(&votes, winner);

        // Build reasoning chain
        let reasoning_chain: Vec<String> = votes.iter().map(|v| {
            format!("[{}] {}", v.pathway.name(), v.content)
        }).collect();

        // Build contributions map
        let pathway_contributions: HashMap<String, f64> = weighted_scores.iter()
            .map(|(p, s)| (p.name().to_string(), *s))
            .collect();

        FusedResult {
            answer,
            confidence: overall_confidence,
            pathway_contributions,
            reasoning_chain,
            conflict_detected,
        }
    }

    /// Detect if pathways strongly disagree.
    fn detect_conflict(&self, votes: &[PathwayVote]) -> bool {
        if votes.len() < 2 {
            return false;
        }

        // Get high-confidence votes
        let high_conf: Vec<_> = votes.iter()
            .filter(|v| v.confidence > 0.6)
            .collect();

        if high_conf.len() < 2 {
            return false;
        }

        // Check if top votes contradict each other
        let top_vote = high_conf[0];
        for vote in &high_conf[1..] {
            // Simple conflict: one says "yes" and other says "no"
            let a_has_negation = top_vote.content.to_lowercase().contains("not") 
                || top_vote.content.to_lowercase().contains("no");
            let b_has_negation = vote.content.to_lowercase().contains("not")
                || vote.content.to_lowercase().contains("no");
            
            if a_has_negation != b_has_negation {
                return true;
            }
        }

        false
    }

    /// Build a coherent answer from multiple votes.
    fn build_answer(&self, votes: &[PathwayVote], winner: Option<Pathway>, query_type: &str) -> String {
        if votes.is_empty() {
            return "I don't know.".to_string();
        }

        // Sort by confidence
        let mut sorted: Vec<&PathwayVote> = votes.iter().collect();
        sorted.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let winner_vote = &sorted[0];
        
        // If there's a strong winner, lead with that
        if winner_vote.is_strong || winner_vote.confidence > 0.8 {
            let mut answer = winner_vote.content.clone();
            
            // If other pathways had something to add, acknowledge it
            if sorted.len() > 1 && sorted[1].confidence > 0.4 {
                let other = sorted[1].content.clone();
                if !other.is_empty() && other != answer {
                    answer.push_str(" ");
                    answer.push_str(&other);
                }
            }
            
            return answer;
        }

        // Multiple pathways contributed — weave together
        if sorted.len() == 1 {
            return sorted[0].content.clone();
        }

        // Build multi-pathway answer
        let mut parts: Vec<String> = Vec::new();
        
        for vote in &sorted[..sorted.len().min(3)] {
            if vote.content.len() > 5 {
                parts.push(format!("({}: {})", vote.pathway.name(), vote.content));
            }
        }

        parts.join(" ")
    }

    /// Update pathway trust based on what worked.
    fn update_trust(&mut self, votes: &[PathwayVote], winner: Option<Pathway>) {
        if let Some(winning_pathway) = winner {
            // Increase trust in winner, slightly decrease others
            for pathway in Pathway::all() {
                let current = *self.pathway_trust.get(pathway).unwrap_or(&0.5);
                let delta = if *pathway == winning_pathway {
                    0.05 // Winner gets small boost
                } else {
                    -0.02 // Others stay roughly the same
                };
                let new_trust = (current + delta).clamp(0.1, 0.95);
                self.pathway_trust.insert(*pathway, new_trust);
            }
        }
    }

    /// Get current trust scores for all pathways.
    pub fn pathway_confidence(&self) -> HashMap<String, f64> {
        self.pathway_trust.iter()
            .map(|(p, c)| (p.name().to_string(), *c))
            .collect()
    }
}

impl Default for PathwayFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of fusing multiple pathway votes.
#[derive(Debug)]
pub struct FusedResult {
    /// The final answer
    pub answer: String,
    /// Overall confidence
    pub confidence: crate::persistence::memory::BeliefState,
    /// How much each pathway contributed
    pub pathway_contributions: HashMap<String, f64>,
    /// Reasoning chain showing pathway contributions
    pub reasoning_chain: Vec<String>,
    /// Whether pathways conflicted
    pub conflict_detected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_no_conflict() {
        let mut fusion = PathwayFusion::new();
        let votes = vec![
            PathwayVote::new(Pathway::Logic, "Fire requires oxygen", 0.9),
            PathwayVote::new(Pathway::Analogy, "Like breathing but faster", 0.7),
        ];
        
        let result = fusion.fuse(votes, "whatis");
        assert!(!result.conflict_detected);
        assert!(result.answer.contains("Fire requires oxygen"));
    }

    #[test]
    fn test_fusion_detects_conflict() {
        let mut fusion = PathwayFusion::new();
        let votes = vec![
            PathwayVote::new(Pathway::Logic, "Fire is hot", 0.9),
            PathwayVote::new(Pathway::Abduction, "Fire is not hot", 0.7),
        ];
        
        let result = fusion.fuse(votes, "whatis");
        assert!(result.conflict_detected);
    }

    #[test]
    fn test_pathway_trust_update() {
        let mut fusion = PathwayFusion::new();
        let votes = vec![
            PathwayVote::new(Pathway::Logic, "Answer", 0.9),
        ];
        
        fusion.fuse(votes, "whatis");
        
        let trust = fusion.pathway_confidence();
        assert!(trust.get("logic").copied().unwrap_or(0.0) > 0.5);
    }
}
