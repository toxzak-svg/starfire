//! Reasoning Layer (Layer 2)
//!
//! Symbolic reasoning without neural networks.
//!
//! Components:
//! - **Knowledge Graph** — entities, relationships, inferred facts
//! - **Rule Engine** — if-then, forward/backward chaining
//! - **Analogy Engine** — structure mapping, "X is to Y as A is to B"
//! - **Abduction** — hypothesize explanations from observations
//! - **Novel Synthesis** — find non-obvious intersections between knowledge
//!
//! This is where Star's intelligence lives. Not retrieval. Actual reasoning.

pub mod knowledge;
pub mod rules;
pub mod analogy;
pub mod synthesis;

use crate::persistence::{Memory, MemoryDomain, BeliefState};
use crate::persistence::memory::Belief;
use std::collections::HashMap;

/// The reasoning engine — combines all reasoning components.
pub struct ReasoningEngine {
    /// Knowledge graph
    knowledge: knowledge::KnowledgeGraph,
    /// Rule base
    rules: rules::RuleEngine,
    /// Analogy engine
    analogy: analogy::AnalogyEngine,
    /// Working memory for current reasoning session
    working_memory: Vec<WorkingItem>,
}

#[derive(Debug, Clone)]
pub struct WorkingItem {
    pub content: String,
    pub source: WorkingSource,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkingSource {
    Retrieved,
    Inferred,
    Assumed,
}

impl ReasoningEngine {
    pub fn new() -> Self {
        Self {
            knowledge: knowledge::KnowledgeGraph::new(),
            rules: rules::RuleEngine::new(),
            analogy: analogy::AnalogyEngine::new(),
            working_memory: Vec::new(),
        }
    }

    /// Reason about a query using available knowledge.
    /// 
    /// Returns a reasoning result with answer, confidence, and chain.
    pub fn reason(&mut self, query: &str, memories: &[Memory]) -> ReasoningResult {
        self.working_memory.clear();
        
        // Load memories into working memory
        for mem in memories {
            self.working_memory.push(WorkingItem {
                content: mem.content.clone(),
                source: WorkingSource::Retrieved,
                confidence: mem.confidence,
            });
            
            // Also populate the knowledge graph
            self.ingest_memory(mem);
        }
        
        // Parse the query to understand what's being asked
        let query_type = self.classify_query(query);
        
        // Attempt reasoning based on query type
        match query_type {
            QueryType::WhatIs => self.answer_what_is(query),
            QueryType::Why => self.answer_why(query),
            QueryType::How => self.answer_how(query),
            QueryType::Does => self.answer_does(query),
            QueryType::Should => self.answer_should(query),
            QueryType::Novel => self.answer_novel(query),
            QueryType::Unknown => self.answer_unknown(query),
        }
    }

    /// Classify what kind of question this is.
    fn classify_query(&self, query: &str) -> QueryType {
        let lower = query.to_lowercase();
        
        if lower.starts_with("what is") || lower.starts_with("what are") || lower.starts_with("what's") {
            QueryType::WhatIs
        } else if lower.starts_with("why") {
            QueryType::Why
        } else if lower.starts_with("how") {
            QueryType::How
        } else if lower.starts_with("does") || lower.starts_with("do ") || lower.starts_with("is ") {
            QueryType::Does
        } else if lower.starts_with("should") || lower.starts_with(" ought ") {
            QueryType::Should
        } else if lower.contains(" if ") || lower.contains(" would happen") {
            QueryType::Novel
        } else {
            QueryType::Unknown
        }
    }

    fn ingest_memory(&mut self, mem: &Memory) {
        // Extract entities and relationships from memory content
        let content = &mem.content;
        
        // Very simple entity extraction: look for capitalized words
        // and common relationship patterns
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            // Skip short words and common words
            if word.len() < 3 { continue; }
            let lower = word.to_lowercase();
            if ["the", "and", "for", "are", "but", "not", "you", "all", "can", "had", 
                "her", "was", "one", "our", "out", "has", "have", "been", "were", "they",
                "this", "that", "with", "from", "it", "its", "about", "which"].contains(&lower.as_str()) {
                continue;
            }
            
            // If capitalized, might be an entity
            if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let entity = word.trim_matches(|c| !char::is_alphanumeric(c));
                if entity.len() > 2 {
                    // Add to knowledge graph
                    let rel_type = if i > 0 && i < words.len() - 1 {
                        let prev = words[i-1].to_lowercase();
                        let next = words.get(i+1).map(|s| s.to_lowercase()).unwrap_or_default();
                        
                        if ["is", "are", "was", "were"].contains(&prev.as_str()) && next != "a" && next != "an" && next != "the" {
                            Some(relation_type_from_word(&next))
                        } else if ["is", "are", "was", "were"].contains(&next.as_str()) && prev != "a" && prev != "an" {
                            Some(relation_type_from_word(&prev))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    if let Some(rel) = rel_type {
                        let to_val = words.get(i+1).copied().unwrap_or("");
                        self.knowledge.add_relationship(entity, rel, to_val);
                    } else {
                        self.knowledge.add_entity(entity);
                    }
                }
            }
        }
        
        // Try to extract if-then patterns
        if content.to_lowercase().contains(" if ") && content.to_lowercase().contains(" then ") {
            if let Some(rule) = self.rules.parse_rule(content) {
                self.rules.add_rule(rule);
            }
        }
    }

    fn answer_what_is(&mut self, query: &str) -> ReasoningResult {
        // Extract the target of "what is X"
        let target = query
            .to_lowercase()
            .replace("what is", "")
            .replace("what are", "")
            .replace("what's", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Search knowledge graph
        let entities = self.knowledge.get_entity(&target);
        
        if let Some(entity) = &entities {
            let facts = self.knowledge.get_facts_about(&target);
            if !facts.is_empty() {
                let answer = format!("{} — {}", entity.description.as_deref().unwrap_or(&entity.name), facts.join("; "));
                return ReasoningResult {
                    answer: Some(answer),
                    confidence: BeliefState::Knows,
                    reasoning_chain: facts,
                    confidence_score: Some(0.85),
                };
            } else {
                return ReasoningResult {
                    answer: Some(format!("I know about {}.", target)),
                    confidence: BeliefState::Thinks,
                    reasoning_chain: vec![format!("Entity '{}' found in knowledge graph", target)],
                    confidence_score: Some(0.5),
                };
            }
        }
        
        // No direct knowledge — check memories
        let relevant: Vec<_> = self.working_memory.iter()
            .filter(|w| w.content.to_lowercase().contains(&target))
            .collect();
        
        if let Some(item) = relevant.first() {
            ReasoningResult {
                answer: Some(format!("Based on what I know: {}", item.content)),
                confidence: item.confidence.map(|c| if c > 0.7 { BeliefState::Thinks } else { BeliefState::Believes })
                    .unwrap_or(BeliefState::Believes),
                reasoning_chain: vec![item.content.clone()],
                confidence_score: item.confidence,
            }
        } else {
            ReasoningResult {
                answer: Some(format!("I don't know what {} is.", target)),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            }
        }
    }

    fn answer_why(&mut self, query: &str) -> ReasoningResult {
        // "Why" questions — try to find causes or reasons
        let topic = query
            .to_lowercase()
            .replace("why does", "")
            .replace("why do", "")
            .replace("why is", "")
            .replace("why are", "")
            .replace("why", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Look for causal relationships in knowledge graph
        let causes = self.knowledge.get_causes(&topic);
        
        if !causes.is_empty() {
            let answer = format!("{} because {}", topic, causes.join(" and "));
            return ReasoningResult {
                answer: Some(answer),
                confidence: BeliefState::Thinks,
                reasoning_chain: causes.clone(),
                confidence_score: Some(0.7),
            };
        }
        
        // Try abduction: hypothesize reasons
        let hypothesis = self.abduct_cause(&topic);
        if let Some(h) = hypothesis {
            ReasoningResult {
                answer: Some(format!("I don't know for certain, but: {}", h)),
                confidence: BeliefState::Suspects,
                reasoning_chain: vec![format!("Abduced cause for '{}': {}", topic, h)],
                confidence_score: Some(0.4),
            }
        } else {
            ReasoningResult {
                answer: Some(format!("I don't know why {}.", topic)),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            }
        }
    }

    fn answer_how(&mut self, query: &str) -> ReasoningResult {
        // "How" questions — try to find mechanisms or methods
        let topic = query
            .to_lowercase()
            .replace("how does", "")
            .replace("how do", "")
            .replace("how to", "")
            .replace("how", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Look for mechanism relationships
        let mechanisms = self.knowledge.get_mechanisms(&topic);
        
        if !mechanisms.is_empty() {
            let answer = format!("{} through: {}", topic, mechanisms.join(", "));
            return ReasoningResult {
                answer: Some(answer),
                confidence: BeliefState::Thinks,
                reasoning_chain: mechanisms.clone(),
                confidence_score: Some(0.6),
            };
        }
        
        ReasoningResult {
            answer: Some(format!("I don't know how {}.", topic)),
            confidence: BeliefState::Unknown,
            reasoning_chain: vec![],
            confidence_score: None,
        }
    }

    fn answer_does(&mut self, query: &str) -> ReasoningResult {
        // Yes/no questions — try to determine truth
        let normalized = query.to_lowercase()
            .replace("does ", "")
            .replace("do ", "")
            .replace("is ", "")
            .replace("are ", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Check knowledge graph for facts
        let facts = self.knowledge.get_facts_containing(&normalized);
        
        if !facts.is_empty() {
            return ReasoningResult {
                answer: Some(facts.first().cloned().unwrap()),
                confidence: BeliefState::Thinks,
                reasoning_chain: facts,
                confidence_score: Some(0.7),
            };
        }
        
        // Check working memory
        let matches: Vec<_> = self.working_memory.iter()
            .filter(|w| w.content.to_lowercase().contains(&normalized))
            .collect();
        
        if let Some(item) = matches.first() {
            ReasoningResult {
                answer: Some(item.content.clone()),
                confidence: item.confidence.map(|c| if c > 0.7 { BeliefState::Thinks } else { BeliefState::Believes })
                    .unwrap_or(BeliefState::Believes),
                reasoning_chain: vec![item.content.clone()],
                confidence_score: item.confidence,
            }
        } else {
            ReasoningResult {
                answer: Some(format!("I don't know whether that's true.")),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            }
        }
    }

    fn answer_should(&mut self, query: &str) -> ReasoningResult {
        // Normative questions — reason about values and consequences
        let topic = query
            .to_lowercase()
            .replace("should", "")
            .replace(" ought ", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        // Look for values-related knowledge
        let values = self.knowledge.get_values_related(&topic);
        
        // Try analogy: "X should Y" analogous to how other things work
        let analogies = self.analogy.find_analogies(&topic);
        
        if !analogies.is_empty() {
            let analogy = &analogies[0];
            let answer = format!(
                "Should {}? Well, {} is to {} as {} is to {}. Does that help?",
                topic, analogy.source, analogy.source_relation, 
                analogy.target, analogy.target_relation
            );
            return ReasoningResult {
                answer: Some(answer),
                confidence: BeliefState::Suspects,
                reasoning_chain: vec![format!("Analogy: {}", analogy.explanation())],
                confidence_score: Some(0.4),
            };
        }
        
        ReasoningResult {
            answer: Some(format!("I don't have a clear answer on whether {} is right.", topic)),
            confidence: BeliefState::Unknown,
            reasoning_chain: vec![],
            confidence_score: None,
        }
    }

    fn answer_novel(&mut self, query: &str) -> ReasoningResult {
        // Novel/complex questions — use full reasoning pipeline
        let topic = query.replace("?", "").trim().to_string();
        
        // Try synthesis: combine knowledge in novel way
        let synthesis = self.synthesize(&topic);
        
        if let Some(result) = synthesis {
            ReasoningResult {
                answer: Some(result.insight),
                confidence: if result.is_novel { BeliefState::Suspects } else { BeliefState::Believes },
                reasoning_chain: result.chain,
                confidence_score: Some(result.confidence),
            }
        } else {
            ReasoningResult {
                answer: Some("That's a hard one.".to_string()),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            }
        }
    }

    fn answer_unknown(&mut self, query: &str) -> ReasoningResult {
        // Fallback for unknown query types
        let topic = query.replace("?", "").trim().to_string();
        
        if topic.len() < 5 {
            return ReasoningResult {
                answer: Some("Say that again?".to_string()),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            };
        }
        
        // Try to find anything relevant
        let relevant: Vec<_> = self.working_memory.iter()
            .filter(|w| {
                w.content.to_lowercase().contains(&topic.to_lowercase()) ||
                topic.to_lowercase().contains(&w.content.to_lowercase())
            })
            .take(3)
            .collect();
        
        if !relevant.is_empty() {
            let contents: Vec<_> = relevant.iter().map(|w| w.content.clone()).collect();
            ReasoningResult {
                answer: Some(format!("I don't know directly, but: {}", contents.join("; "))),
                confidence: BeliefState::Believes,
                reasoning_chain: contents,
                confidence_score: Some(0.3),
            }
        } else {
            ReasoningResult {
                answer: Some("I don't know anything about that.".to_string()),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            }
        }
    }

    /// Abduction: hypothesize a cause for an observation.
    fn abduct_cause(&self, observation: &str) -> Option<String> {
        // Find known effects and work backwards
        let effects = self.knowledge.get_effects(observation);
        
        // Simple abductive reasoning: if X causes Y, and we see Y, maybe X
        if !effects.is_empty() {
            // Pick the most confident effect's source
            return effects.first().cloned();
        }
        
        // Fallback: look for common cause patterns
        let known_causes = vec![
            (observation, vec!["it seems connected to how things work", 
                             "maybe something about its nature",
                             "perhaps an underlying principle"]),
        ];
        
        known_causes.first().map(|(o, hints)| {
            let hint = hints[(o.len() + observation.len()) % hints.len()];
            format!("{} — {}", o, hint)
        })
    }

    /// Synthesis: combine knowledge to produce novel insights.
    fn synthesize(&self, query: &str) -> Option<SynthesisResult> {
        let query_lower = query.to_lowercase();
        
        // Get all relevant working memory
        let relevant: Vec<_> = self.working_memory.iter()
            .filter(|w| {
                let content_lower = w.content.to_lowercase();
                query_lower.split_whitespace().any(|word| 
                    content_lower.contains(word) || word.len() > 5
                )
            })
            .collect();
        
        if relevant.len() < 2 {
            return None;
        }
        
        // Try to find an analogy between two pieces of knowledge
        if let Some(analogy) = self.analogy.find_analogy_between(&relevant) {
            return Some(SynthesisResult {
                insight: format!(
                    "Here's something: {} — that reminds me of {}, except {}. {}",
                    analogy.source, analogy.target, 
                    analogy.target_relation, analogy.structure
                ),
                is_novel: true,
                confidence: 0.5,
                chain: vec![
                    format!("Source: {}", analogy.source),
                    format!("Target: {}", analogy.target),
                    analogy.explanation(),
                ],
            });
        }
        
        None
    }

    /// Check if a statement contradicts known facts.
    pub fn check_consistency(&self, statement: &str) -> ConsistencyResult {
        let lower = statement.to_lowercase();
        
        // Check against knowledge graph
        for entity in self.knowledge.entities() {
            let facts = self.knowledge.get_facts_about(entity);
            for fact in facts {
                // Simple contradiction detection
                if lower.contains("not") && fact.to_lowercase().contains("is ") && fact.to_lowercase().contains(&lower[..lower.find(' ').unwrap_or(0)]) {
                    return ConsistencyResult::Contradiction { 
                        fact: fact.clone() 
                    };
                }
            }
        }
        
        ConsistencyResult::Consistent
    }

    /// Get the knowledge graph (for inspection).
    pub fn knowledge_graph(&self) -> &knowledge::KnowledgeGraph {
        &self.knowledge
    }
}

impl Default for ReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    WhatIs,
    Why,
    How,
    Does,
    Should,
    Novel,
    Unknown,
}

/// Result of a reasoning operation.
#[derive(Debug)]
pub struct ReasoningResult {
    pub answer: Option<String>,
    pub confidence: BeliefState,
    pub reasoning_chain: Vec<String>,
    pub confidence_score: Option<f64>,
}

/// Result of a synthesis operation.
#[derive(Debug)]
pub struct SynthesisResult {
    pub insight: String,
    pub is_novel: bool,
    pub confidence: f64,
    pub chain: Vec<String>,
}

/// Result of a consistency check.
#[derive(Debug)]
pub enum ConsistencyResult {
    Consistent,
    Contradiction { fact: String },
    NeedsReview { reason: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility functions
// ─────────────────────────────────────────────────────────────────────────────

fn relation_type_from_word(word: &str) -> knowledge::RelationType {
    match word.to_lowercase().as_str() {
        "is" | "are" | "was" | "were" | "be" => knowledge::RelationType::IsA,
        "has" | "have" | "had" => knowledge::RelationType::HasProperty,
        "causes" | "caused" | "because" => knowledge::RelationType::Causes,
        "enables" | "allows" | "helps" => knowledge::RelationType::Enables,
        "prevents" | "blocks" | "stops" => knowledge::RelationType::Prevents,
        "part_of" | "within" | "inside" => knowledge::RelationType::PartOf,
        "uses" | "through" | "via" => knowledge::RelationType::Uses,
        "like" | "similar" | "resembles" => knowledge::RelationType::SimilarTo,
        "opposite" | "unlike" => knowledge::RelationType::OppositeOf,
        _ => knowledge::RelationType::RelatedTo,
    }
}
