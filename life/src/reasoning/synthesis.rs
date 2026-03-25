//! Novel Synthesis
//!
//! Combines knowledge in non-obvious ways to produce genuine insights.
//! This is where Star can "invent" — not retrieve, not compute, but create.
//!
//! The key insight: novel synthesis isn't random combination.
//! It's structured exploration of the possibility space between knowledge domains.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// A novel synthesis — a new idea generated from combining existing knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    /// The generated insight
    pub insight: String,
    /// What sources were combined
    pub sources: Vec<String>,
    /// How novel this is (0.0 = obvious, 1.0 = never seen)
    pub novelty: f64,
    /// Confidence in the synthesis
    pub confidence: f64,
    /// The reasoning path taken
    pub path: Vec<SynthesisStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisStep {
    pub step_type: StepType,
    pub content: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// Retrieved relevant knowledge
    Retrieve,
    /// Found a pattern in the knowledge
    PatternMatch,
    /// Mapped structure from one domain to another
    MapStructure,
    /// Combined two concepts
    Combine,
    /// Hypothesized a connection
    Hypothesize,
    /// Validated against constraints
    Validate,
    /// Surprising conclusion
    Emerge,
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retrieve => write!(f, "retrieve"),
            Self::PatternMatch => write!(f, "pattern"),
            Self::MapStructure => write!(f, "map"),
            Self::Combine => write!(f, "combine"),
            Self::Hypothesize => write!(f, "hypothesize"),
            Self::Validate => write!(f, "validate"),
            Self::Emerge => write!(f, "emerge"),
        }
    }
}

/// The synthesis engine — produces novel combinations.
#[derive(Debug, Clone, Default)]
pub struct SynthesisEngine {
    /// Known synthesis patterns
    patterns: Vec<SynthesisPattern>,
    /// Domain mappings (concepts that bridge different areas)
    bridges: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SynthesisPattern {
    pub name: String,
    pub description: String,
    pub applies_to: Vec<String>,
    pub produces: String,
}

impl SynthesisEngine {
    pub fn new() -> Self {
        let mut engine = Self::default();
        engine.load_common_patterns();
        engine
    }

    /// Load common synthesis patterns.
    fn load_common_patterns(&mut self) {
        self.patterns = vec![
            SynthesisPattern {
                name: "cause_effect_chain".to_string(),
                description: "Chain causes and effects across domains".to_string(),
                applies_to: vec!["causation".to_string(), "mechanism".to_string()],
                produces: "mechanism".to_string(),
            },
            SynthesisPattern {
                name: "analogy".to_string(),
                description: "Map structure from known to unknown".to_string(),
                applies_to: vec!["structure".to_string(), "relationship".to_string()],
                produces: "analogy".to_string(),
            },
            SynthesisPattern {
                name: "constraint_satisfaction".to_string(),
                description: "Find what satisfies multiple constraints".to_string(),
                applies_to: vec!["constraint".to_string(), "requirement".to_string()],
                produces: "solution".to_string(),
            },
            SynthesisPattern {
                name: "abstraction".to_string(),
                description: "Find the common structure in specific cases".to_string(),
                applies_to: vec!["specific".to_string(), "cases".to_string()],
                produces: "generalization".to_string(),
            },
            SynthesisPattern {
                name: "composition".to_string(),
                description: "Combine properties of parts into new whole".to_string(),
                applies_to: vec!["parts".to_string(), "properties".to_string()],
                produces: "composite".to_string(),
            },
        ];
    }

    /// Synthesize a novel insight from multiple knowledge sources.
    pub fn synthesize(&self, topic: &str, sources: &[String]) -> Option<Synthesis> {
        if sources.is_empty() {
            return None;
        }
        
        let mut steps = Vec::new();
        let mut path = Vec::new();
        
        // Step 1: Pattern recognition — find patterns across sources
        steps.push(SynthesisStep {
            step_type: StepType::PatternMatch,
            content: format!("Examining patterns across: {}", sources.join(", ")),
            reasoning: "Looking for recurring structures or relationships".to_string(),
        });
        
        let patterns = self.find_cross_domain_patterns(sources);
        
        if patterns.is_empty() {
            // No patterns found — try combination anyway
            steps.push(SynthesisStep {
                step_type: StepType::Combine,
                content: format!("Combining: {}", sources.join(" + ")),
                reasoning: "Direct combination attempted".to_string(),
            });
        } else {
            path.push(patterns.join("; "));
            
            steps.push(SynthesisStep {
                step_type: StepType::MapStructure,
                content: patterns.join("; "),
                reasoning: "Found structural similarity between sources".to_string(),
            });
        }
        
        // Step 2: Attempt synthesis based on topic
        let insight = self.generate_insight(topic, sources, &mut path);
        
        // Step 3: Validate (simple check)
        let valid = self.validate_synthesis(&insight, sources);
        
        steps.push(SynthesisStep {
            step_type: if valid { StepType::Validate } else { StepType::Hypothesize },
            content: insight.clone(),
            reasoning: if valid { 
                "Synthesis validated — no contradictions found".to_string()
            } else {
                "Hypothesis — needs further validation".to_string()
            },
        });
        
        // Step 4: Check for emergence (surprising conclusion)
        let novelty = self.assess_novelty(&insight, sources);
        
        if novelty > 0.6 {
            steps.push(SynthesisStep {
                step_type: StepType::Emerge,
                content: format!("Novel insight (novelty={:.2})", novelty),
                reasoning: "This conclusion was not obvious from the inputs".to_string(),
            });
        }
        
        Some(Synthesis {
            insight,
            sources: sources.to_vec(),
            novelty,
            confidence: if valid { 0.6 } else { 0.3 },
            path: steps,
        })
    }

    /// Find patterns that span multiple knowledge domains.
    fn find_cross_domain_patterns(&self, sources: &[String]) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Look for common words across sources
        let word_sets: Vec<HashSet<&str>> = sources.iter()
            .map(|s| s.split_whitespace().collect())
            .collect();
        
        if word_sets.len() >= 2 {
            // Find intersections
            let mut intersection = word_sets[0].clone();
            for set in &word_sets[1..] {
                intersection = intersection.intersection(set).copied().collect();
            }
            
            // Filter out common words
            let significant: Vec<_> = intersection.iter()
                .filter(|w| w.len() > 4 && !Self::is_common_word(w))
                .collect();
            
            for word in significant.iter().take(3) {
                patterns.push(format!("Common concept: {}", word));
            }
        }
        
        patterns
    }

    /// Check if a word is a common stop word.
    fn is_common_word(word: &&str) -> bool {
        matches!(word, &"that" | &"this" | &"with" | &"from" | &"have" | 
                 &"about" | &"which" | &"their" | &"what" | &"some" |
                 &"could" | &"into" | &"more" | &"them" | &"than" |
                 &"then" | &"some" | &"when" | &"where" | &"would" |
                 &"there" | &"between" | &"each" | &"these" | &"those")
    }

    /// Generate an insight from sources about a topic.
    fn generate_insight(&self, topic: &str, sources: &[String], path: &mut Vec<String>) -> String {
        let topic_lower = topic.to_lowercase();
        
        // Try different synthesis strategies
        
        // Strategy 1: "What if" — extend a concept
        if let Some(insight) = self.strategy_what_if(topic, sources) {
            path.push("strategy: what_if".to_string());
            return insight;
        }
        
        // Strategy 2: Bridge — connect through a third concept
        if let Some(insight) = self.strategy_bridge(sources) {
            path.push("strategy: bridge".to_string());
            return insight;
        }
        
        // Strategy 3: Inversion — flip a relationship
        if let Some(insight) = self.strategy_inversion(topic, sources) {
            path.push("strategy: inversion".to_string());
            return insight;
        }
        
        // Strategy 4: Abstraction — find the general principle
        if let Some(insight) = self.strategy_abstraction(sources) {
            path.push("strategy: abstraction".to_string());
            return insight;
        }
        
        // Fallback: combine sources directly
        format!(
            "{} — combined with {} — suggests that {}",
            sources.first().unwrap_or(&String::new()),
            sources.get(1).unwrap_or(&String::new()),
            topic
        )
    }

    /// "What if" strategy — extend a concept to a new domain.
    fn strategy_what_if(&self, topic: &str, sources: &[String]) -> Option<String> {
        // Look for a causal relationship in sources
        for source in sources {
            let lower = source.to_lowercase();
            if lower.contains("cause") || lower.contains("make") || lower.contains("produce") {
                // Found a cause — what if we apply it elsewhere?
                let cause_word = if lower.contains("cause") { "cause" } 
                    else if lower.contains("make") { "make" } 
                    else { "produce" };
                
                return Some(format!(
                    "What if the {} in '{}' applied to {}?",
                    cause_word, source, topic
                ));
            }
        }
        None
    }

    /// "Bridge" strategy — connect through a third concept.
    fn strategy_bridge(&self, sources: &[String]) -> Option<String> {
        if sources.len() < 2 { return None; }
        
        // Find common connectors
        let bridge_words = vec![
            ("through", "by means of"),
            ("because", "due to"),
            ("enables", "allowing"),
            ("despite", "even though"),
        ];
        
        let first = sources.first()?;
        let second = sources.get(1)?;
        
        for (word, alternative) in &bridge_words {
            if first.to_lowercase().contains(*word) || second.to_lowercase().contains(*word) {
                return Some(format!(
                    "{} {} {} — they connect through the principle of '{}'",
                    first, alternative, second, word
                ));
            }
        }
        
        None
    }

    /// "Inversion" strategy — flip a relationship.
    fn strategy_inversion(&self, topic: &str, sources: &[String]) -> Option<String> {
        for source in sources {
            let lower = source.to_lowercase();
            
            if lower.contains(" leads to ") || lower.contains(" causes ") {
                // Find the relationship and invert it
                return Some(format!(
                    "What if '{}' led to the opposite of what we expect for {}?",
                    source, topic
                ));
            }
        }
        None
    }

    /// "Abstraction" strategy — find the general principle.
    fn strategy_abstraction(&self, sources: &[String]) -> Option<String> {
        if sources.len() < 2 { return None; }
        
        // Find common structure
        let patterns = self.find_cross_domain_patterns(sources);
        
        if patterns.len() >= 1 {
            return Some(format!(
                "Underlying pattern: {} — this suggests a general principle about {}",
                patterns.join(", "),
                sources.first().unwrap_or(&String::new())
            ));
        }
        
        None
    }

    /// Validate a synthesis against known facts.
    fn validate_synthesis(&self, insight: &str, sources: &[String]) -> bool {
        let insight_lower = insight.to_lowercase();
        
        // Simple validation: check for obvious contradictions
        for source in sources {
            let source_lower = source.to_lowercase();
            
            // Check for negation contradictions
            if source_lower.contains("not ") && insight_lower.contains("always ") {
                return false;
            }
        }
        
        true
    }

    /// Assess how novel a synthesis is.
    fn assess_novelty(&self, insight: &str, sources: &[String]) -> f64 {
        let insight_words: HashSet<&str> = insight.split_whitespace().collect();
        let source_words: HashSet<&str> = sources.iter()
            .flat_map(|s| s.split_whitespace())
            .collect();
        
        // How much of the insight is NOT in the sources?
        let new_words: Vec<_> = insight_words.difference(&source_words).collect();
        
        if new_words.is_empty() {
            0.1 // Very obvious
        } else {
            // Score based on proportion of new words
            let ratio = new_words.len() as f64 / insight_words.len().max(1) as f64;
            // But also penalize if too disconnected
            if source_words.is_empty() {
                0.3
            } else {
                (ratio * 0.8).clamp(0.1, 0.9)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis() {
        let engine = SynthesisEngine::new();
        
        let sources = vec![
            "Fire produces heat through combustion".to_string(),
            "Water flows downhill due to gravity".to_string(),
        ];
        
        let synthesis = engine.synthesize("energy", &sources);
        assert!(synthesis.is_some());
        
        let synthesis = synthesis.unwrap();
        println!("Insight: {}", synthesis.insight);
        println!("Novelty: {:.2}", synthesis.novelty);
        println!("Path: {:?}", synthesis.path.iter().map(|s| s.step_type.to_string()).collect::<Vec<_>>());
    }

    #[test]
    fn test_find_cross_domain_patterns() {
        let engine = SynthesisEngine::new();
        
        let sources = vec![
            "Fire produces heat".to_string(),
            "Fire produces light".to_string(),
            "Fire produces ash".to_string(),
        ];
        
        let patterns = engine.find_cross_domain_patterns(&sources);
        println!("Patterns: {:?}", patterns);
        assert!(!patterns.is_empty());
    }
}
