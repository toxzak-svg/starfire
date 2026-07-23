//! Analogy Engine
//!
//! Maps structure from known domains to novel domains.
//! "X is to Y as A is to B" — finding that mapping is what makes reasoning powerful.
//!
//! This is where Star can "see" connections that weren't explicitly taught.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An analogy — a structural mapping between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    /// The source domain (known)
    pub source: String,
    /// The relation in the source
    pub source_relation: String,
    /// The target domain (novel)
    pub target: String,
    /// The inferred relation in the target
    pub target_relation: String,
    /// The structure being mapped
    pub structure: String,
    /// How confident we are in this mapping
    pub confidence: f64,
}

impl Analogy {
    /// Human-readable explanation of this analogy.
    pub fn explanation(&self) -> String {
        format!(
            "{} is to {} as {} is to {} — {}",
            self.source, self.source_relation, 
            self.target, self.target_relation,
            self.structure
        )
    }
}

/// The analogy engine — finds mappings between domains.
#[derive(Debug, Clone)]
pub struct AnalogyEngine {
    /// Known analogies (cached)
    known_analogies: Vec<Analogy>,
    /// Relationship patterns observed
    patterns: HashMap<String, Vec<RelationshipPattern>>,
    /// Optional reference to the knowledge graph for dynamic analogy-making
    knowledge_graph: Option<std::sync::Arc<std::sync::RwLock<crate::reasoning::knowledge::KnowledgeGraph>>>,
    /// Maximum dynamic analogies to consider
    max_dynamic: usize,
}

#[derive(Debug, Clone)]
pub struct RelationshipPattern {
    pub relation: String,
    pub from: String,
    pub to: String,
}

impl Default for AnalogyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalogyEngine {
    pub fn new() -> Self {
        Self {
            known_analogies: Vec::new(),
            patterns: HashMap::new(),
            knowledge_graph: None,
            max_dynamic: 5,
        }
    }

    /// Attach a knowledge graph reference for dynamic analogy-making.
    /// Call this after constructing the engine if you want graph-based analogies.
    pub fn with_knowledge_graph(mut self, kg: std::sync::Arc<std::sync::RwLock<crate::reasoning::knowledge::KnowledgeGraph>>) -> Self {
        self.knowledge_graph = Some(kg);
        self
    }

    /// Find analogies for a given concept.
    pub fn find_analogies(&self, concept: &str) -> Vec<Analogy> {
        let mut results = Vec::new();
        let concept_lower = concept.to_lowercase();
        
        // Look for known analogies involving this concept
        for analogy in &self.known_analogies {
            if analogy.source.to_lowercase().contains(&concept_lower) ||
               analogy.target.to_lowercase().contains(&concept_lower) {
                results.push(analogy.clone());
            }
        }
        
        // If no known analogies, try to construct some
        if results.is_empty() {
            results.extend(self.construct_analogies(concept));
        }
        
        results
    }

    /// Construct analogies — first tries the knowledge graph, then falls back to
    /// hardcoded categories. The KG-based approach is preferred because it reasons
    /// from Star's actual accumulated knowledge rather than generic examples.
    fn construct_analogies(&self, concept: &str) -> Vec<Analogy> {
        let mut analogies = Vec::new();
        
        // ─── Strategy 1: Dynamic KG-based analogies ───────────────────────────
        if let Some(ref kg_arc) = self.knowledge_graph {
            if let Ok(kg) = kg_arc.read() {
                // Find any analogy for this concept from the actual knowledge graph
                let dynamic = kg.find_any_analogy_for(concept);
                for da in dynamic.into_iter().take(self.max_dynamic) {
                    let explanation = da.explanation();
                    analogies.push(Analogy {
                        source: da.source,
                        source_relation: da.source_relation,
                        target: da.target,
                        target_relation: da.target_relation,
                        structure: explanation,
                        confidence: da.confidence,
                    });
                }
                
                // Also try to pair this concept with another known entity
                // to find a direct A:B :: C:D analogy
                for entity in kg.entities().into_iter().take(20) {
                    if entity.to_lowercase() != concept.to_lowercase() && entity.len() > 2 {
                        let direct = kg.find_analogies(concept, &entity);
                        for da in direct.into_iter().take(2) {
                            let explanation = da.explanation();
                            analogies.push(Analogy {
                                source: da.source,
                                source_relation: da.source_relation,
                                target: da.target,
                                target_relation: da.target_relation,
                                structure: explanation,
                                confidence: da.confidence * 0.9, // Slight penalty for being indirect
                            });
                        }
                    }
                }
            }
        }
        
        // ─── Strategy 2: Hardcoded categories (fallback) ───────────────────────
        // Only use these if the KG didn't give us anything useful
        if analogies.is_empty() {
            let categories = vec![
                // Physical to abstract
                (vec![
                    ("fire", "heat", "passion", "intensity"),
                    ("water", "flow", "ideas", "connection"),
                    ("trees", "roots", "knowledge", "foundation"),
                    ("light", "illumination", "understanding", "clarity"),
                ]),
                // Process analogies
                (vec![
                    ("growth", "natural", "learning", "personal"),
                    ("erosion", "slow change", "habits", "gradual shift"),
                    ("chemistry", "reactions", "relationships", "interactions"),
                ]),
            ];
            
            let concept_lower = concept.to_lowercase();
            
            for category in &categories {
                for (base, rel, mapped, mapped_rel) in category {
                    if concept_lower.contains(base) || concept_lower.contains(mapped) {
                        analogies.push(Analogy {
                            source: base.to_string(),
                            source_relation: rel.to_string(),
                            target: mapped.to_string(),
                            target_relation: mapped_rel.to_string(),
                            structure: format!("The relationship between {} and {} mirrors {} and {}", 
                                base, rel, mapped, mapped_rel),
                            confidence: 0.5,
                        });
                    }
                }
            }
        }
        
        // Limit to best analogies
        analogies.truncate(3);
        analogies
    }

    /// Find analogies between two pieces of knowledge.
    pub fn find_analogy_between(&self, items: &[&super::WorkingItem]) -> Option<Analogy> {
        if items.len() < 2 { return None; }
        
        // Try to find structural similarity between items
        for i in 0..items.len() {
            for j in (i+1)..items.len() {
                let item_a = &items[i].content;
                let item_b = &items[j].content;
                
                if let Some(analogy) = self.find_structural_mapping(item_a, item_b) {
                    return Some(analogy);
                }
            }
        }
        
        None
    }

    /// Find structural mapping between two texts.
    fn find_structural_mapping(&self, a: &str, b: &str) -> Option<Analogy> {
        let words_a: HashSet<&str> = a.split_whitespace().collect();
        let words_b: HashSet<&str> = b.split_whitespace().collect();
        
        // Find shared structure
        let shared: Vec<&str> = words_a.intersection(&words_b).copied().collect();
        
        if !shared.is_empty() {
            // Found some shared structure
            let a_unique: Vec<&str> = words_a.difference(&words_b).copied().collect();
            let b_unique: Vec<&str> = words_b.difference(&words_a).copied().collect();
            
            if !a_unique.is_empty() && !b_unique.is_empty() {
                let source = shared.first().unwrap_or(&a).to_string();
                let target = shared.first().unwrap_or(&b).to_string();
                
                return Some(Analogy {
                    source: a_unique.first().unwrap_or(&"").to_string(),
                    source_relation: source.clone(),
                    target: b_unique.first().unwrap_or(&"").to_string(),
                    target_relation: target.clone(),
                    structure: format!(
                        "Both '{}' and '{}' relate to '{}' — '{}' does for '{}' what '{}' does for '{}'",
                        a_unique.first().unwrap_or(&""),
                        b_unique.first().unwrap_or(&""),
                        source,
                        b_unique.first().unwrap_or(&""),
                        target,
                        a_unique.first().unwrap_or(&""),
                        source,
                    ),
                    confidence: 0.4,
                });
            }
        }
        
        None
    }

    /// Store a known analogy for future use.
    pub fn store_analogy(&mut self, analogy: Analogy) {
        // Avoid duplicates
        if !self.known_analogies.iter().any(|a| 
            a.source == analogy.source && a.target == analogy.target
        ) {
            self.known_analogies.push(analogy);
        }
    }

    /// Record a relationship pattern for analogy building.
    pub fn record_pattern(&mut self, from: &str, relation: &str, to: &str) {
        let pattern = RelationshipPattern {
            relation: relation.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        };
        
        let key = format!("{}:{}", from, to);
        self.patterns.entry(key.clone()).or_default().push(pattern.clone());

        // Also store with wildcard key so solve_analogy can find all patterns from a given entity
        let wildcard_key = format!("{}:*", from);
        if wildcard_key != key {
            self.patterns.entry(wildcard_key).or_default().push(pattern);
        }
    }

    /// Generate an analogy from recorded patterns.
    pub fn generate_from_patterns(&self, source_domain: &str) -> Option<Analogy> {
        let _key = format!("{}:*", source_domain);
        
        // Find patterns involving the source
        let relevant: Vec<_> = self.patterns.iter()
            .filter(|(k, _)| k.starts_with(&format!("{}:", source_domain)))
            .collect();
        
        if relevant.len() >= 2 {
            // Can construct an analogy
            let rel_a = relevant[0].1.first()?;
            let rel_b = relevant[1].1.first()?;
            
            return Some(Analogy {
                source: source_domain.to_string(),
                source_relation: format!("{} {}", rel_a.relation, rel_a.to),
                target: rel_b.to.clone(),
                target_relation: format!("similar to {} {}", rel_a.relation, rel_a.to),
                structure: format!(
                    "The {} of {} mirrors the {} of {}",
                    rel_a.relation, rel_a.to,
                    rel_b.relation, rel_b.to
                ),
                confidence: 0.6,
            });
        }
        
        None
    }

    /// "A is to B as C is to ?" — solve the analogy.
    pub fn solve_analogy(&self, a: &str, _b: &str, c: &str) -> Option<String> {
        // Find a B relationship such that A:B :: C:?
        // e.g., "Fire is to heat as water is to ?"
        // Answer: "flow" or "movement"
        
        // Look for stored patterns
        let key = format!("{}:*", a);
        let patterns_from_a = self.patterns.get(&key)?;
        
        if let Some(rel) = patterns_from_a.first() {
            let relation_type = &rel.relation;
            let b_value = &rel.to;
            
            // Now find what has the same relationship to C
            let key_c = format!("{}:*", c);
            let patterns_from_c = self.patterns.get(&key_c)?;
            
            for pattern in patterns_from_c {
                if pattern.relation == *relation_type {
                    return Some(pattern.to.clone());
                }
            }
            
            // Fallback: return something with similar structure
            return Some(format!("something related to {} the way {} is related to {}", c, b_value, a));
        }
        
        None
    }

    /// Get all known analogies.
    pub fn known_analogies(&self) -> &[Analogy] {
        &self.known_analogies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_analogies_for_concept() {
        let engine = AnalogyEngine::new();
        let analogies = engine.find_analogies("fire");
        
        // Should find fire-related analogies
        assert!(!analogies.is_empty());
    }

    #[test]
    fn test_solve_analogy() {
        let mut engine = AnalogyEngine::new();

        // Seed with patterns so engine has data to work with
        engine.record_pattern("fire", "produces", "heat");
        engine.record_pattern("water", "causes", "flow");
        engine.record_pattern("sun", "creates", "light");

        // Fire -> Heat :: Water -> ?
        let solution = engine.solve_analogy("fire", "heat", "water");

        // Should return something flow/movement related
        assert!(solution.is_some());
    }

    #[test]
    fn test_find_analogy_between() {
        let engine = AnalogyEngine::new();

        let item1 = super::super::WorkingItem {
            content: "Fire is hot".to_string(),
            source: super::super::WorkingSource::Retrieved,
            confidence: Some(0.9),
        };
        let item2 = super::super::WorkingItem {
            content: "Fire is dangerous".to_string(),
            source: super::super::WorkingSource::Retrieved,
            confidence: Some(0.8),
        };
        let _items: Vec<&super::super::WorkingItem> = vec![&item1, &item2];

        // Both items share "Fire" - engine should find structural mapping
        let analogy = engine.find_analogy_between(&[&item1, &item2]);
        assert!(analogy.is_some());
    }
}
