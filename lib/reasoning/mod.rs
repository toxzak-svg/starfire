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
pub mod pathways;
pub mod chain;
pub mod chain_display;

use crate::persistence::{Memory, BeliefState};
use pathways::PathwayFusion;
use knowledge::RelationType;

/// The reasoning engine — combines all reasoning components.
#[derive(Clone)]
#[allow(dead_code)]
pub struct ReasoningEngine {
    /// Knowledge graph
    knowledge: knowledge::KnowledgeGraph,
    /// Rule base
    rules: rules::RuleEngine,
    /// Analogy engine
    analogy: analogy::AnalogyEngine,
    /// Pathway fusion (R&D-E)
    fusion: PathwayFusion,
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
    /// Get a reference to the knowledge graph (for autonomous thinking).
    pub fn knowledge(&self) -> &knowledge::KnowledgeGraph {
        &self.knowledge
    }

    /// Get a mutable reference to the knowledge graph (for syncing from memory store).
    pub fn knowledge_mut(&mut self) -> &mut knowledge::KnowledgeGraph {
        &mut self.knowledge
    }

    pub fn new() -> Self {
        let knowledge = knowledge::KnowledgeGraph::new();
        let kg_arc = std::sync::Arc::new(std::sync::RwLock::new(knowledge.clone()));
        Self {
            knowledge,
            rules: rules::RuleEngine::new(),
            analogy: analogy::AnalogyEngine::new().with_knowledge_graph(kg_arc),
            fusion: PathwayFusion::new(),
            working_memory: Vec::new(),
        }
    }

    /// Add a piece of knowledge to the reasoning engine.
    pub fn add_knowledge(&mut self, subject: &str, fact: &str) {
        self.knowledge.add_fact(subject, fact);
        // Also add the subject as an entity
        self.knowledge.add_entity(subject);
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
        // Extract entities and relationships from memory content.
        // Supports both capitalized proper nouns ("Fire") and common nouns ("fire").
        let content = &mem.content;
        let words: Vec<&str> = content.split_whitespace().collect();
        
        // Skip stop words
        let stop_words: std::collections::HashSet<&str> = [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", 
            "her", "was", "one", "our", "out", "has", "have", "been", "were", "they",
            "this", "that", "with", "from", "its", "about", "which", "also", "such",
        ].into_iter().collect();
        
        // Pass 1: extract simple "X is Y" / "X has Y" / "X requires Y" relationships
        // and collect candidate entities.
        let n = words.len();
        for (i, word) in words.iter().enumerate() {
            let word_lower = word.to_lowercase();
            let cleaned = word.trim_matches(|c| !char::is_alphanumeric(c));
            if cleaned.len() < 2 || stop_words.contains(word_lower.as_str()) {
                continue;
            }
            
            // Determine if this word is an entity (capitalized OR first word, or any word
            // that appears in a subject-like position before a verb).
            let is_entity = word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                || (i == 0 && word.chars().next().map(|c| c.is_lowercase()).unwrap_or(false));
            
            if !is_entity && i > 0 {
                continue;
            }
            
            // Try to extract relationship from what follows
            // Patterns: "Fire is hot", "Fire requires oxygen", "Fire produces heat"
            if i < n - 2 {
                let v1 = words[i + 1].to_lowercase();
                let v2 = words.get(i + 2).map(|s| s.to_lowercase()).unwrap_or_default();
                
                if ["is", "are", "was", "were"].contains(&v1.as_str()) {
                    // "X is Y" — add relationship and entity
                    let entity_name = cleaned;
                    let value = words[i + 2].trim_matches(|c: char| !char::is_alphanumeric(c));
                    if value.len() > 1 {
                        self.knowledge.add_relationship(entity_name, RelationType::IsA, value);
                    }
                } else if ["requires", "needs", "needs:", "uses"].contains(&v1.as_str()) {
                    let entity_name = cleaned;
                    // Collect full object (may be multi-word)
                    let rest: Vec<&str> = words[i+2..].iter()
                        .map(|w| w.trim_matches(|c: char| !char::is_alphanumeric(c)))
                        .filter(|w| !w.is_empty() && !stop_words.contains(w.to_lowercase().as_str()))
                        .collect();
                    if !rest.is_empty() {
                        let obj = rest.join(" ");
                        self.knowledge.add_relationship(entity_name, RelationType::Enables, &obj);
                    }
                } else if ["produces", "creates", "generates"].contains(&v1.as_str()) {
                    let entity_name = cleaned;
                    let rest: Vec<&str> = words[i+2..].iter()
                        .map(|w| w.trim_matches(|c: char| !char::is_alphanumeric(c)))
                        .filter(|w| !w.is_empty() && !stop_words.contains(w.to_lowercase().as_str()))
                        .collect();
                    if !rest.is_empty() {
                        let obj = rest.join(" ");
                        self.knowledge.add_relationship(entity_name, RelationType::Causes, &obj);
                    }
                } else if ["causes", "leads", "to"].contains(&v1.as_str()) && v2 == "to" {
                    let entity_name = cleaned;
                    let rest: Vec<&str> = words[i+3..].iter()
                        .map(|w| w.trim_matches(|c: char| !char::is_alphanumeric(c)))
                        .filter(|w| !w.is_empty() && !stop_words.contains(w.to_lowercase().as_str()))
                        .collect();
                    if !rest.is_empty() {
                        let obj = rest.join(" ");
                        self.knowledge.add_relationship(entity_name, RelationType::Causes, &obj);
                    }
                }
            }
            
            // Also add as bare entity if no relationship found
            if cleaned.len() > 2 {
                self.knowledge.add_entity(cleaned);
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
                answer: Some(item.content.to_string()),
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

        // Try looking up mechanisms directly first (works for single-word targets)
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

        // For compound targets (e.g. "fire burn"), extract individual keywords
        // and look for facts about each one — then merge the results.
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "does", "do", "to", "of", "in", "on", "for", "with", "by",
        ].into_iter().collect();

        let keywords: Vec<&str> = topic.split_whitespace()
            .filter(|w| !stop_words.contains(*w) && w.len() > 1)
            .collect();

        let mut all_mechanisms: Vec<String> = Vec::new();
        let mut all_facts: Vec<String> = Vec::new();

        for kw in &keywords {
            let mech = self.knowledge.get_mechanisms(kw);
            for m in mech {
                if !all_mechanisms.contains(&m) {
                    all_mechanisms.push(m);
                }
            }
            let facts = self.knowledge.get_facts_about(kw);
            for f in facts {
                if !all_facts.contains(&f) {
                    all_facts.push(f);
                }
            }
        }

        // Also check working memory for relevant entries
        for item in &self.working_memory {
            let content_lower = item.content.to_lowercase();
            if keywords.iter().any(|kw| content_lower.contains(kw))
                && !all_facts.contains(&item.content) {
                    all_facts.push(item.content.clone());
                }
        }

        if !all_mechanisms.is_empty() {
            let answer = format!("{} through: {}", topic, all_mechanisms.join(", "));
            return ReasoningResult {
                answer: Some(answer),
                confidence: BeliefState::Thinks,
                reasoning_chain: all_mechanisms,
                confidence_score: Some(0.6),
            };
        }

        if !all_facts.is_empty() {
            return ReasoningResult {
                answer: Some(format!("{}: {}", topic, all_facts.join("; "))),
                confidence: BeliefState::Believes,
                reasoning_chain: all_facts,
                confidence_score: Some(0.4),
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

        // Check knowledge graph for facts matching the normalized query
        let facts = self.knowledge.get_facts_containing(&normalized);

        if !facts.is_empty() {
            return ReasoningResult {
                answer: Some(facts.first().cloned().unwrap()),
                confidence: BeliefState::Thinks,
                reasoning_chain: facts.clone(),
                confidence_score: Some(0.7),
            };
        }

        // For compound queries (e.g. "fire produce heat"), search by individual keywords
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "does", "do", "is", "are", "to", "of", "in", "on", "for", "with", "by",
        ].into_iter().collect();

        let keywords: Vec<&str> = normalized.split_whitespace()
            .filter(|w| !stop_words.contains(*w) && w.len() > 1)
            .collect();

        let mut all_facts: Vec<String> = Vec::new();

        for kw in &keywords {
            let facts = self.knowledge.get_facts_about(kw);
            for f in facts {
                if !all_facts.contains(&f) {
                    all_facts.push(f);
                }
            }
            let containing = self.knowledge.get_facts_containing(kw);
            for f in containing {
                if !all_facts.contains(&f) {
                    all_facts.push(f);
                }
            }
        }

        // Also check working memory
        for item in &self.working_memory {
            let content_lower = item.content.to_lowercase();
            if keywords.iter().any(|kw| content_lower.contains(kw))
                && !all_facts.contains(&item.content) {
                    all_facts.push(item.content.clone());
                }
        }

        if !all_facts.is_empty() {
            return ReasoningResult {
                answer: Some(format!("Based on what I know: {}", all_facts.join("; "))),
                confidence: BeliefState::Believes,
                reasoning_chain: all_facts.clone(),
                confidence_score: Some(0.5),
            };
        }

        ReasoningResult {
            answer: Some("I don't know whether that's true.".to_string()),
            confidence: BeliefState::Unknown,
            reasoning_chain: vec![],
            confidence_score: None,
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
        let _values = self.knowledge.get_values_related(&topic);
        
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
        // Fallback for unknown query types — but still consult the knowledge graph.
        let topic = query.replace("?", "").trim().to_string();
        
        if topic.len() < 5 {
            return ReasoningResult {
                answer: Some("Say that again?".to_string()),
                confidence: BeliefState::Unknown,
                reasoning_chain: vec![],
                confidence_score: None,
            };
        }
        
        // Try the knowledge graph first (handles "who is X", "where is Y", etc.)
        let lower_topic = topic.to_lowercase();
        let entities = self.knowledge.get_entity(&lower_topic);
        if let Some(entity) = entities {
            let facts = self.knowledge.get_facts_about(&lower_topic);
            if !facts.is_empty() {
                let answer = format!("{} — {}", 
                    entity.description.as_deref().unwrap_or(&topic), 
                    facts.join("; ")
                );
                return ReasoningResult {
                    answer: Some(answer),
                    confidence: BeliefState::Knows,
                    reasoning_chain: facts,
                    confidence_score: Some(0.85),
                };
            } else {
                return ReasoningResult {
                    answer: Some(format!("I know about {}.", topic)),
                    confidence: BeliefState::Thinks,
                    reasoning_chain: vec![format!("Entity '{}' found in knowledge graph", topic)],
                    confidence_score: Some(0.5),
                };
            }
        }
        
        // Try to find anything relevant in working memory
        let relevant: Vec<_> = self.working_memory.iter()
            .filter(|w| {
                w.content.to_lowercase().contains(&lower_topic) ||
                lower_topic.contains(&w.content.to_lowercase())
            })
            .take(3)
            .collect();
        
        if !relevant.is_empty() {
            let contents: Vec<_> = relevant.iter().map(|w| w.content.clone()).collect();
            ReasoningResult {
                answer: Some(contents.join("; ")),
                confidence: BeliefState::Believes,
                reasoning_chain: contents.clone(),
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
        let known_causes = [(observation, vec!["it seems connected to how things work", 
                             "maybe something about its nature",
                             "perhaps an underlying principle"])];
        
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
            let facts = self.knowledge.get_facts_about(&entity);
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

#[allow(dead_code)]
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
