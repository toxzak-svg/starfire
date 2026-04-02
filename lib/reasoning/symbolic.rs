//! Symbolic reasoning engine — propositional logic on the knowledge graph.
//!
//! Replaces keyword matching with actual inference:
//! - Parse queries into structured propositions
//! - Query KG for relevant facts
//! - Apply modus ponens, chain resolution, unification
//! - Synthesize answers with reasoning trace

use crate::Store;
use crate::persistence::Memory;
use crate::reasoning::knowledge::{KnowledgeGraph, RelationType};
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// A parsed proposition from either a query or a KG fact.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposition {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}

/// An inference rule with premises and conclusions.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: &'static str,
    pub if_predicate: String,    // when this predicate is matched...
    pub then_subject: String,    // ...conclude this subject
    pub then_predicate: String,  // ...with this predicate
    pub then_object: String,    // ...pointing to this object
}

/// The inference engine — applies rules to propositions.
pub struct SymbolicEngine {
    rules: Vec<Rule>,
    max_depth: usize,
}

impl Default for SymbolicEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolicEngine {
    pub fn new() -> Self {
        // Core inference rules — these are the "axioms" of reasoning
        let rules = vec![
            // "X creates Y" + "Y is a Z" → "X creates something that is a Z"
            Rule {
                name: "transitive_creation",
                if_predicate: "creates".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "is".to_string(),
                then_object: "_any".to_string(),
            },
            // "X causes Y" + "Y causes Z" → "X causes Z"
            Rule {
                name: "causal_chain",
                if_predicate: "causes".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "causes".to_string(),
                then_object: "_any".to_string(),
            },
            // "X is related to Y" + "Y is Z" → "X is related to something that is Z"
            Rule {
                name: "related_transitivity",
                if_predicate: "related to".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "is".to_string(),
                then_object: "_any".to_string(),
            },
            // "X enables Y" + "Y is Z" → "X enables something that is Z"
            Rule {
                name: "enablement_transitivity",
                if_predicate: "enables".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "is".to_string(),
                then_object: "_any".to_string(),
            },
            // "X requires Y" + "Y is Z" → "X requires something that is Z"
            Rule {
                name: "requirement_transitivity",
                if_predicate: "requires".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "is".to_string(),
                then_object: "_any".to_string(),
            },
            // "X similar to Y" + "Y similar to Z" → "X similar to Z"
            Rule {
                name: "similarity_chain",
                if_predicate: "similar to".to_string(),
                then_subject: "_intermediate".to_string(),
                then_predicate: "similar to".to_string(),
                then_object: "_any".to_string(),
            },
            // "X is A" + "A enables B" → "X enables B" (use chain)
            Rule {
                name: "is_enables",
                if_predicate: "is".to_string(),
                then_subject: "_subject".to_string(),
                then_predicate: "enables".to_string(),
                then_object: "_object".to_string(),
            },
            // "X is A" + "A causes B" → "X causes B" (use chain)
            Rule {
                name: "is_causes",
                if_predicate: "is".to_string(),
                then_subject: "_subject".to_string(),
                then_predicate: "causes".to_string(),
                then_object: "_object".to_string(),
            },
        ];

        Self {
            rules,
            max_depth: 3,
        }
    }

    /// Reason about a query using the knowledge graph.
    /// Returns (answer, reasoning_trace).
    pub fn reason(&self, query: &str, kg: &KnowledgeGraph) -> (Option<String>, Vec<String>) {
        let query_lower = query.to_lowercase();
        let trace = Vec::new();
        
        // Step 1: Parse the query
        let query_prop = self.parse_query(&query_lower);
        debug!("Query proposition: {:?}", query_prop);

        if query_prop.subject.is_empty() && query_prop.object.is_empty() {
            // Open-ended query — ask a "what is" question
            return self.answer_what_is(&query_prop.predicate, kg, trace);
        }

        if query_prop.subject.is_empty() {
            // "what causes X?" or "what enables X?"
            return self.answer_what_causes(&query_prop.predicate, &query_prop.object, kg, trace);
        }

        if query_prop.object.is_empty() {
            // "what is X?" or "X is what?"
            return self.answer_what_is_about(&query_prop.subject, kg, trace);
        }

        // "what is A about B?" or "does X cause Y?"
        self.answer_fact(&query_prop.subject, &query_prop.predicate, &query_prop.object, kg, trace)
    }

    /// Parse a natural language query into a proposition.
    fn parse_query(&self, query: &str) -> Proposition {
        let q = query.trim();

        // "what is X?" → subject="X", predicate="is", object=""
        if let Some(rest) = q.strip_prefix("what is ") {
            return Proposition {
                subject: rest.trim_end_matches('?').trim().to_string(),
                predicate: "is".to_string(),
                object: String::new(),
                confidence: 1.0,
            };
        }

        // "who created X?" → subject="X", predicate="created_by", object=""
        if let Some(rest) = q.strip_prefix("who created ") {
            let target = rest.trim_end_matches('?').trim();
            return Proposition {
                subject: target.to_string(),
                predicate: "created_by".to_string(),
                object: String::new(),
                confidence: 1.0,
            };
        }

        // "what does X do?" or "what is X?"
        if let Some(rest) = q.strip_prefix("what does ") {
            let rest = rest.trim_end_matches('?').trim();
            if let Some(target) = rest.strip_prefix(" do") {
                return Proposition {
                    subject: target.to_string(),
                    predicate: "does".to_string(),
                    object: String::new(),
                    confidence: 1.0,
                };
            }
            if let Some(target) = rest.strip_prefix(" mean") {
                return Proposition {
                    subject: target.to_string(),
                    predicate: "is".to_string(),
                    object: String::new(),
                    confidence: 1.0,
                };
            }
        }

        // "why does X Y?" → subject="X", predicate="Y", object=""
        if let Some(rest) = q.strip_prefix("why does ") {
            let rest = rest.trim_end_matches('?').trim();
            // "why does A cause B" → subject="A", predicate="cause", object="B"
            if let Some(rest) = rest.strip_prefix("cause ") {
                let parts: Vec<_> = rest.splitn(2, " ").collect();
                if parts.len() == 2 {
                    return Proposition {
                        subject: parts[0].to_string(),
                        predicate: "causes".to_string(),
                        object: parts[1].to_string(),
                        confidence: 1.0,
                    };
                }
            }
            return Proposition {
                subject: rest.replace(" cause ", " "),
                predicate: "causes".to_string(),
                object: String::new(),
                confidence: 1.0,
            };
        }

        // "why did X happen?" → subject="X", predicate="happened", object=""
        if let Some(rest) = q.strip_prefix("why did ") {
            let target = rest.trim_end_matches('?').trim().replace(" happen", "").replace("ed ", " ");
            return Proposition {
                subject: target,
                predicate: "caused".to_string(),
                object: String::new(),
                confidence: 1.0,
            };
        }

        // "does X Y Z?" — yes/no question
        // "does A cause B?"
        if q.starts_with("does ") {
            let rest = q.trim_start_matches("does ").trim_end_matches('?');
            let parts: Vec<_> = rest.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let predicate = parts[1].to_string();
                return Proposition {
                    subject: parts[0].to_string(),
                    predicate,
                    object: parts[2..].join(" "),
                    confidence: 1.0,
                };
            }
        }

        // "X is Y" → direct statement
        if let Some(rest) = q.strip_prefix("is ") {
            let parts: Vec<_> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                return Proposition {
                    subject: parts[0].to_string(),
                    predicate: "is".to_string(),
                    object: parts[1].to_string(),
                    confidence: 1.0,
                };
            }
        }

        // Default: treat whole query as predicate
        Proposition {
            subject: String::new(),
            predicate: query.to_string(),
            object: String::new(),
            confidence: 0.5,
        }
    }

    /// Answer "what is X?" — find direct facts about X.
    fn answer_what_is(&self, predicate: &str, kg: &KnowledgeGraph, mut trace: Vec<String>) -> (Option<String>, Vec<String>) {
        // Look for anything matching the predicate
        let subject_lower = predicate.to_lowercase();
        
        // Search for IsA relationships
        let facts_from = kg.get_relationships_from(&subject_lower);
        let facts_to = kg.get_relationships_to(&subject_lower);
        
        let mut answers = Vec::new();
        
        // Direct "A is B" from outgoing IsA
        for rel in &facts_from {
            if matches!(rel.relation, RelationType::IsA) {
                answers.push(format!("{} is {}", rel.from, rel.to));
            }
        }
        
        // "A is kind of B" from incoming IsA
        for rel in &facts_to {
            if matches!(rel.relation, RelationType::IsA) && rel.from.to_lowercase() != subject_lower {
                answers.push(format!("{} is a kind of {}", rel.from, rel.to));
            }
        }
        
        // Similarity
        for rel in &facts_from {
            if matches!(rel.relation, RelationType::SimilarTo) {
                answers.push(format!("{} is similar to {}", rel.from, rel.to));
            }
        }
        
        // Property
        for rel in &facts_from {
            if matches!(rel.relation, RelationType::HasProperty) {
                answers.push(format!("{} is characterized by {}", rel.from, rel.to));
            }
        }
        
        if answers.is_empty() {
            trace.push(format!("No direct facts found for '{}'", predicate));
            return (None, trace);
        }
        
        trace.push(format!("Found {} facts about '{}'", answers.len(), predicate));
        let answer = answers.join(". ");
        (Some(answer), trace)
    }

    /// Answer "what is X about?" — find facts where X is subject.
    fn answer_what_is_about(&self, subject: &str, kg: &KnowledgeGraph, mut trace: Vec<String>) -> (Option<String>, Vec<String>) {
        let subject_lower = subject.to_lowercase();
        let rels = kg.get_relationships_from(&subject_lower);
        
        let mut facts = Vec::new();
        for rel in &rels {
            facts.push(format!("{} {} {}", rel.from, rel.predicate, rel.to));
        }
        
        if facts.is_empty() {
            trace.push(format!("No facts found about '{}'", subject));
            return (None, trace);
        }
        
        trace.push(format!("Found {} relationships for '{}'", facts.len(), subject));
        let answer = facts.join(". ");
        (Some(answer), trace)
    }

    /// Answer "what causes X?" or "what enables X?" — find incoming causal relations.
    fn answer_what_causes(&self, predicate: &str, target: &str, kg: &KnowledgeGraph, mut trace: Vec<String>) -> (Option<String>, Vec<String>) {
        let target_lower = target.to_lowercase();
        let rels = kg.get_relationships_to(&target_lower);
        
        let predicate_lower = predicate.to_lowercase();
        
        let mut causes = Vec::new();
        for rel in &rels {
            if rel.predicate.to_lowercase().contains(&predicate_lower) {
                causes.push(format!("{} {}", rel.from, rel.predicate));
            }
        }
        
        if causes.is_empty() {
            trace.push(format!("No causes found for '{}'", target));
            return (None, trace);
        }
        
        trace.push(format!("Found {} causes of '{}'", causes.len(), target));
        let answer = causes.join(". ");
        (Some(answer), trace)
    }

    /// Answer a specific fact question: "does X cause Y?"
    fn answer_fact(&self, subject: &str, predicate: &str, object: &str, kg: &KnowledgeGraph, mut trace: Vec<String>) -> (Option<String>, Vec<String>) {
        let subject_lower = subject.to_lowercase();
        let predicate_lower = predicate.to_lowercase();
        let object_lower = object.to_lowercase();
        
        // Direct lookup
        let rels = kg.get_relationships_from(&subject_lower);
        
        for rel in &rels {
            if rel.predicate.to_lowercase().contains(&predicate_lower)
                && rel.to.to_lowercase().contains(&object_lower) {
                trace.push(format!("Direct match: '{}' {} '{}'", subject, predicate, object));
                return (Some(format!("Yes. {} {} {}.", subject, predicate, object)), trace);
            }
        }
        
        // Try reverse: does Y have X as an object?
        let rels_to = kg.get_relationships_to(&object_lower);
        for rel in &rels_to {
            if rel.from.to_lowercase().contains(&subject_lower)
                && rel.predicate.to_lowercase().contains(&predicate_lower) {
                trace.push(format!("Reverse match: '{}' {} '{}'", subject, predicate, object));
                return (Some(format!("Yes. {} {} {}.", subject, predicate, object)), trace);
            }
        }
        
        // Inference: try one-step inference
        if let Some(inferred) = self.infer_one_step(subject, predicate, object, kg) {
            trace.push(format!("Inferred from knowledge graph"));
            return (Some(format!("Yes, inferred: {}.", inferred)), trace);
        }
        
        trace.push(format!("No direct or inferred fact for '{}' {} '{}'", subject, predicate, object));
        (Some(format!("I don't have a stored fact for '{} {} {}'.", subject, predicate, object)), trace)
    }

    /// Try one-step propositional inference.
    /// If we know "X → Y" and "Y → Z", conclude "X → Z".
    fn infer_one_step(&self, subject: &str, predicate: &str, object: &str, kg: &KnowledgeGraph) -> Option<String> {
        // Try to find an intermediate: X → intermediate → Y
        let subject_lower = subject.to_lowercase();
        let object_lower = object.to_lowercase();
        
        let rels = kg.get_relationships_from(&subject_lower);
        let rels_to = kg.get_relationships_to(&object_lower);
        
        for rel in &rels {
            for rel2 in &rels_to {
                // Is the same intermediate found?
                if rel.to.to_lowercase() == rel2.from.to_lowercase() {
                    return Some(format!("{} {} via {}", subject, predicate, rel.to));
                }
            }
        }
        
        None
    }

    /// Apply inference rules to derive new facts from the KG.
    /// This is the forward-chaining engine.
    pub fn infer_all(&self, kg: &KnowledgeGraph, max_depth: usize) -> Vec<Proposition> {
        let mut derived = Vec::new();
        let mut visited = HashMap::<String, bool>::new();
        
        self.forward_chain(kg, &mut derived, &mut visited, 0, max_depth);
        derived
    }

    fn forward_chain(&self, kg: &KnowledgeGraph, derived: &mut Vec<Proposition>, visited: &mut HashMap<String, bool>, depth: usize, max_depth: usize) {
        if depth >= max_depth { return; }
        
        let facts = kg.all_facts();
        
        for rule in &self.rules {
            for fact in &facts {
                if fact.predicate.to_lowercase() == rule.if_predicate {
                    // Apply the rule
                    let new_subject = match rule.then_subject.as_str() {
                        "_intermediate" => fact.to.clone(),
                        "_subject" => fact.subject.clone(),
                        _ => rule.then_subject.clone(),
                    };
                    let new_object = if rule.then_object == "_any" {
                        // Find what the intermediate relates to
                        let rels = kg.get_relationships_from(&fact.to.to_lowercase());
                        if let Some(rel) = rels.first() {
                            rel.to.clone()
                        } else {
                            continue;
                        }
                    } else {
                        rule.then_object.clone()
                    };
                    
                    let key = format!("{} {} {}", new_subject, rule.then_predicate, new_object);
                    if !visited.contains_key(&key) {
                        visited.insert(key.clone(), true);
                        derived.push(Proposition {
                            subject: new_subject,
                            predicate: rule.then_predicate.clone(),
                            object: new_object,
                            confidence: 0.8, // inferred, lower confidence
                        });
                        
                        // Recurse
                        let new_kg = kg.with_fact(&Proposition {
                            subject: new_subject,
                            predicate: rule.then_predicate.clone(),
                            object: new_object,
                            confidence: 0.8,
                        });
                        self.forward_chain(&new_kg, derived, visited, depth + 1, max_depth);
                    }
                }
            }
        }
    }
}

impl KnowledgeGraph {
    /// Get all facts as a flat list.
    fn all_facts(&self) -> Vec<Proposition> {
        let mut facts = Vec::new();
        for rel in self.relationships.iter() {
            facts.push(Proposition {
                subject: rel.from.clone(),
                predicate: rel.predicate.clone(),
                object: rel.to.clone(),
                confidence: rel.weight,
            });
        }
        facts
    }

    /// Create a new KG with an additional fact (for chaining).
    fn with_fact(&self, fact: &Proposition) -> KnowledgeGraph {
        let mut new_kg = self.clone();
        new_kg.ingest_fact(&fact.subject, &fact.predicate, &fact.object, fact.confidence);
        new_kg
    }
}

/// Synthesize a response from reasoning results.
pub fn synthesize_response(
    query: &str,
    kg: &KnowledgeGraph,
    engine: &SymbolicEngine,
) -> String {
    let (answer, trace) = engine.reason(query, kg);
    
    let answer = answer.unwrap_or_else(|| {
        // Fallback: try KG direct lookup
        let subject = query.to_lowercase()
            .replace("what is", "")
            .replace("who is", "")
            .replace("what does", "")
            .replace("?", "")
            .trim()
            .to_string();
        
        let rels = kg.get_relationships_from(&subject);
        if !rels.is_empty() {
            let facts: Vec<_> = rels.iter()
                .map(|r| format!("{} {}", r.predicate, r.to))
                .collect();
            facts.join(". ")
        } else {
            "I don't have specific knowledge about that yet.".to_string()
        }
    });
    
    if !trace.is_empty() {
        info!("Symbolic reasoning trace: {}", trace.join(" → "));
    }
    
    answer
}
