//! Knowledge Graph
//!
//! Represents entities, relationships, and inferred facts.
//! The substrate for all reasoning — without this, reasoning has nothing to work with.

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

/// A knowledge graph — entities, relationships, and facts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGraph {
    /// Entities by name
    entities: HashMap<String, Entity>,
    /// Relationships between entities
    relationships: Vec<Relationship>,
    /// Named concepts (abstract entities)
    concepts: HashMap<String, Concept>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub properties: HashMap<String, String>,
    pub description: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub name: String,
    pub definition: String,
    pub examples: Vec<String>,
    pub related_concepts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub relation: RelationType,
    pub to: String,
    pub confidence: f64,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    IsA,
    HasProperty,
    Causes,
    Enables,
    Prevents,
    PartOf,
    Uses,
    SimilarTo,
    OppositeOf,
    RelatedTo,
    InstanceOf,
    CausedBy,
    EnabledBy,
    UsedBy,
    HasPart,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IsA => "is a",
            Self::HasProperty => "has property",
            Self::Causes => "causes",
            Self::Enables => "enables",
            Self::Prevents => "prevents",
            Self::PartOf => "part of",
            Self::Uses => "uses",
            Self::SimilarTo => "similar to",
            Self::OppositeOf => "opposite of",
            Self::RelatedTo => "related to",
            Self::InstanceOf => "instance of",
            Self::CausedBy => "caused by",
            Self::EnabledBy => "enabled by",
            Self::UsedBy => "used by",
            Self::HasPart => "has part",
        }
    }

    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::IsA => Some(Self::InstanceOf),
            Self::InstanceOf => Some(Self::IsA),
            Self::HasProperty => None,
            Self::Causes => Some(Self::CausedBy),
            Self::CausedBy => Some(Self::Causes),
            Self::Enables => Some(Self::EnabledBy),
            Self::EnabledBy => Some(Self::Enables),
            Self::Prevents => None,
            Self::PartOf => Some(Self::HasPart),
            Self::HasPart => Some(Self::PartOf),
            Self::Uses => Some(Self::UsedBy),
            Self::UsedBy => Some(Self::Uses),
            Self::SimilarTo => Some(Self::SimilarTo),
            Self::OppositeOf => Some(Self::OppositeOf),
            Self::RelatedTo => Some(Self::RelatedTo),
        }
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest a simple factual statement as a relationship.
    /// Parses "subject verb object" into a relationship between entities.
    /// 
    /// Examples:
    /// - ingest_fact("fire", "causes", "heat", 0.9)
    /// - ingest_fact("star", "is", "reasoning intelligence", 0.95)
    pub fn ingest_fact(&mut self, subject: &str, verb: &str, object: &str, confidence: f64) {
        // Normalize
        let subject = subject.trim().to_lowercase();
        let verb = verb.trim().to_lowercase();
        let object = object.trim().to_lowercase();

        if subject.len() < 2 || object.len() < 2 { return; }
        if verb.is_empty() { return; }

        // Map common verbs to relation types (only valid enum variants)
        let rel_type = match verb.as_str() {
            "is" | "are" | "'s" | "was" | "be" => RelationType::IsA,
            "causes" | "cause" | "lead to" | "leads to" => RelationType::Causes,
            "requires" | "need" | "needs" | "depend on" => RelationType::Causes, // closest match
            "produces" | "create" | "creates" | "make" | "makes" => RelationType::Causes,
            "enables" | "allow" | "allows" => RelationType::Enables,
            "uses" | "use" | "using" => RelationType::Uses,
            "related to" | "related" | "like" | "similar to" | "similar" => RelationType::SimilarTo,
            "part of" | "part" => RelationType::PartOf,
            "has" | "have" | "having" => RelationType::HasProperty,
            "can" | "able to" => RelationType::Enables,
            "prevents" | "stop" | "stops" => RelationType::Prevents,
            _ => RelationType::RelatedTo,
        };

        // Create the relationship
        let rel = Relationship {
            from: subject.clone(),
            to: object.clone(),
            relation: rel_type,
            confidence,
            source: Some(format!("{} {} {}", subject, verb, object)),
        };

        // Add entities
        self.add_entity(&subject);
        self.add_entity(&object);

        // Deduplicate: only add if not already present
        let key = format!("{}:{}:{}", rel.from, rel.relation.as_str(), rel.to);
        let exists = self.relationships.iter().any(|r| 
            format!("{}:{}:{}", r.from, r.relation.as_str(), r.to) == key
        );
        if !exists {
            self.relationships.push(rel);
        }
    }

    /// Extract entities (nouns and noun phrases) from text using simple pattern matching.
    pub fn extract_entities(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        
        // Pattern 1: Capitalized words (proper nouns)
        let mut prev_was_cap = false;
        let mut current_phrase = String::new();
        for word in text.split_whitespace() {
            let first_char = word.chars().next().unwrap_or(' ');
            if first_char.is_uppercase() && first_char.is_alphabetic() {
                if prev_was_cap && !current_phrase.is_empty() {
                    current_phrase.push(' ');
                }
                current_phrase.push_str(word);
                prev_was_cap = true;
            } else {
                if !current_phrase.is_empty() && current_phrase.len() > 1 {
                    entities.push(current_phrase.clone());
                }
                current_phrase.clear();
                prev_was_cap = false;
            }
        }
        if !current_phrase.is_empty() && current_phrase.len() > 1 {
            entities.push(current_phrase);
        }
        
        // Pattern 2: Quoted phrases
        for (i, ch) in text.char_indices() {
            if ch == '"' {
                if let Some(end) = text[i+1..].find('"') {
                    let phrase = &text[i+1..i+1+end];
                    if phrase.len() > 2 {
                        entities.push(phrase.to_string());
                    }
                }
                break;
            }
        }
        
        entities.dedup();
        entities
    }

    /// Add an entity.
    pub fn add_entity(&mut self, name: &str) {
        if name.len() < 2 { return; }
        let name = name.trim_matches(|c| !char::is_alphanumeric(c)).to_string();
        if name.len() < 2 { return; }
        
        self.entities.entry(name.clone()).or_insert_with(|| Entity {
            name: name.clone(),
            properties: HashMap::new(),
            description: None,
            confidence: 0.5,
        });
    }

    /// Add a relationship between entities.
    pub fn add_relationship(&mut self, from: &str, relation: RelationType, to: &str) {
        if from.is_empty() || to.is_empty() { return; }

        let from = from.trim_matches(|c| !char::is_alphanumeric(c)).to_string();
        let to = to.trim_matches(|c| !char::is_alphanumeric(c)).to_string();
        
        if from.is_empty() || to.is_empty() { return; }
        
        self.add_entity(&from);
        self.add_entity(&to);
        
        // Avoid duplicate relationships
        if self.relationships.iter().any(|r| r.from == from && r.relation == relation && r.to == to) {
            return;
        }
        
        self.relationships.push(Relationship {
            from,
            relation,
            to,
            confidence: 0.7,
            source: None,
        });
    }

    /// Add a property to an entity.
    pub fn add_property(&mut self, entity: &str, property: &str, value: &str) {
        let normalized_entity = entity.trim_matches(|c| !char::is_alphanumeric(c)).to_string();
        self.add_entity(&normalized_entity);
        if let Some(e) = self.entities.get_mut(&normalized_entity) {
            e.properties.insert(property.to_string(), value.to_string());
        }
    }

    /// Add a fact about an entity.
    pub fn add_fact(&mut self, entity: &str, fact: &str) {
        let normalized_entity = entity.trim_matches(|c| !char::is_alphanumeric(c)).to_string();
        self.add_entity(&normalized_entity);
        if let Some(e) = self.entities.get_mut(&normalized_entity) {
            e.description = Some(fact.to_string());
        }
    }

    /// Get an entity by name.
    pub fn get_entity(&self, name: &str) -> Option<&Entity> {
        // Case-insensitive lookup
        self.entities.get(name)
            .or_else(|| self.entities.get(&name.to_lowercase()))
            .or_else(|| {
                // Try case-insensitive search through all entities
                self.entities.iter()
                    .find(|(k, _)| k.to_lowercase() == name.to_lowercase())
                    .map(|(_, v)| v)
            })
    }

    /// Get all entity names (owned Strings).
    pub fn entities(&self) -> Vec<String> {
        self.entities.keys().cloned().collect()
    }

    /// Get all relationships from an entity (case-insensitive, owned).
    pub fn get_relationships_from(&self, entity: &str) -> Vec<Relationship> {
        let entity_lower = entity.to_lowercase();
        self.relationships.iter()
            .filter(|r| r.from.to_lowercase() == entity_lower)
            .cloned()
            .collect()
    }

    /// Get all relationships to an entity (case-insensitive, owned).
    pub fn get_relationships_to(&self, entity: &str) -> Vec<Relationship> {
        let entity_lower = entity.to_lowercase();
        self.relationships.iter()
            .filter(|r| r.to.to_lowercase() == entity_lower)
            .cloned()
            .collect()
    }

    /// Get all facts about an entity.
    pub fn get_facts_about(&self, entity: &str) -> Vec<String> {
        let mut facts = Vec::new();
        
        // Use case-insensitive entity lookup
        if let Some(e) = self.get_entity(entity) {
            for (_prop, val) in &e.properties {
                facts.push(format!("{} has {}", e.name, val));
            }
            if let Some(desc) = &e.description {
                facts.push(desc.clone());
            }
        }
        
        // Relationships
        for rel in self.get_relationships_from(entity) {
            facts.push(format!("{} {} {}", entity, rel.relation.as_str(), rel.to));
        }
        for rel in self.get_relationships_to(entity) {
            if rel.relation == RelationType::IsA || rel.relation == RelationType::InstanceOf {
                facts.push(format!("{} {} {}", entity, rel.relation.as_str(), rel.to));
            }
        }
        
        facts
    }

    /// Get facts containing a term.
    pub fn get_facts_containing(&self, term: &str) -> Vec<String> {
        let term_lower = term.to_lowercase();
        let mut facts = Vec::new();
        
        for entity in self.entities.keys() {
            let entity_lower = entity.to_lowercase();
            if entity_lower.contains(&term_lower) {
                facts.extend(self.get_facts_about(entity));
            }
        }
        
        facts
    }

    /// Get causes of an entity (what causes it).
    pub fn get_causes(&self, entity: &str) -> Vec<String> {
        self.relationships.iter()
            .filter(|r| r.to == entity && r.relation == RelationType::Causes)
            .map(|r| format!("{} {}", r.from, r.relation.as_str()))
            .collect()
    }

    /// Get effects of an entity (what it causes).
    pub fn get_effects(&self, entity: &str) -> Vec<String> {
        self.relationships.iter()
            .filter(|r| r.from == entity && r.relation == RelationType::Causes)
            .map(|r| format!("{} {}", r.relation.as_str(), r.to))
            .collect()
    }

    /// Get mechanisms related to an entity.
    pub fn get_mechanisms(&self, entity: &str) -> Vec<String> {
        let mut mechanisms = Vec::new();
        
        for rel in self.relationships.iter() {
            if rel.from == entity && rel.relation == RelationType::Uses {
                mechanisms.push(format!("uses {}", rel.to));
            }
            if rel.to == entity && rel.relation == RelationType::Enables {
                mechanisms.push(format!("enabled by {}", rel.from));
            }
        }
        
        mechanisms
    }

    /// Get values-related entities.
    pub fn get_values_related(&self, topic: &str) -> Vec<String> {
        let topic_lower = topic.to_lowercase();
        let mut related = Vec::new();
        
        // Look for entities that have "good" or "value" in their relationships
        for rel in &self.relationships {
            if (rel.from.to_lowercase().contains(&topic_lower) ||
               rel.to.to_lowercase().contains(&topic_lower))
                && rel.relation == RelationType::RelatedTo {
                    related.push(format!("{} {} {}", rel.from, rel.relation.as_str(), rel.to));
                }
        }
        
        related
    }

    /// Infer new relationships using transitive reasoning.
    pub fn infer_transitive(&self, from: &str, relation: &RelationType, depth: usize) -> Vec<String> {
        if depth == 0 { return Vec::new(); }
        
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((from.to_string(), depth));
        
        while let Some((current, remaining)) = queue.pop_front() {
            if visited.contains(&current) { continue; }
            visited.insert(current.clone());
            
            for rel in self.get_relationships_from(&current) {
                if &rel.relation == relation {
                    results.push(rel.to.clone());
                    if remaining > 1 {
                        queue.push_back((rel.to.clone(), remaining - 1));
                    }
                }
            }
        }
        
        results
    }

    /// Find entities connected by a chain of relationships.
    pub fn find_connection(&self, from: &str, to: &str, max_depth: usize) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![from.to_string()]);
        
        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();
            
            if current == to {
                return Some(path);
            }
            
            if path.len() >= max_depth { continue; }
            if visited.contains(current) { continue; }
            visited.insert(current.clone());
            
            for rel in self.get_relationships_from(current) {
                let mut new_path = path.clone();
                new_path.push(rel.to.clone());
                queue.push_back(new_path);
            }
        }
        
        None
    }

    /// Add a concept (abstract idea with definition).
    pub fn add_concept(&mut self, name: &str, definition: &str) {
        self.concepts.insert(name.to_string(), Concept {
            name: name.to_string(),
            definition: definition.to_string(),
            examples: Vec::new(),
            related_concepts: Vec::new(),
        });
    }

    /// Get a concept.
    pub fn get_concept(&self, name: &str) -> Option<&Concept> {
        self.concepts.get(name)
    }

    /// Number of entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Number of relationships.
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    /// Find analogies between two concepts by exploring shared relational structure.
    /// 
    /// This is the core of dynamic analogy-making: instead of hardcoded categories,
    /// we inspect the *actual* relationships in the knowledge graph and look for
    /// structural parallels. "A:B :: C:D" when A→X and C→Y are the same *type*
    /// of relationship, even if X≠Y.
    /// 
    /// Returns a list of discovered analogies sorted by confidence.
    pub fn find_analogies(&self, concept_a: &str, concept_b: &str) -> Vec<DynamicAnalogy> {
        let mut analogies = Vec::new();
        let a_lower = concept_a.to_lowercase();
        let b_lower = concept_b.to_lowercase();

        // Get all relationships from A and B
        let rels_from_a: Vec<_> = self.relationships.iter()
            .filter(|r| r.from.to_lowercase() == a_lower)
            .collect();
        let rels_from_b: Vec<_> = self.relationships.iter()
            .filter(|r| r.from.to_lowercase() == b_lower)
            .collect();

        for rel_a in &rels_from_a {
            for rel_b in &rels_from_b {
                // Same relation type → structural parallel
                if rel_a.relation == rel_b.relation {
                    let target_same = rel_a.to.to_lowercase() == rel_b.to.to_lowercase();
                    
                    let analogy = DynamicAnalogy {
                        source: concept_a.to_string(),
                        source_relation: format!("{} {}", rel_a.relation.as_str(), rel_a.to),
                        source_rel_type: rel_a.relation,
                        target: concept_b.to_string(),
                        target_relation: format!("{} {}", rel_b.relation.as_str(), rel_b.to),
                        target_rel_type: rel_b.relation,
                        is_parallel: target_same,
                        confidence: if target_same { 0.9 } else { 0.7 },
                    };
                    analogies.push(analogy);
                }
                
                // Inverse relation → potential contrast/opposite
                if let Some(inv) = rel_a.relation.inverse() {
                    if inv == rel_b.relation && rel_a.from.to_lowercase() != rel_b.from.to_lowercase() {
                        analogies.push(DynamicAnalogy {
                            source: concept_a.to_string(),
                            source_relation: format!("{} {}", rel_a.relation.as_str(), rel_a.to),
                            source_rel_type: rel_a.relation,
                            target: concept_b.to_string(),
                            target_relation: format!("{} {}", rel_b.relation.as_str(), rel_b.to),
                            target_rel_type: rel_b.relation,
                            is_parallel: false,
                            confidence: 0.5,
                        });
                    }
                }
            }
        }

        // Sort by confidence and deduplicate
        analogies.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        analogies.dedup_by(|a, b| 
            a.source == b.source && a.target == b.target 
            && a.source_rel_type == b.source_rel_type
        );

        analogies
    }

    /// Find the best analogy connecting two arbitrary concepts by traversing the graph.
    /// 
    /// Unlike `find_analogies` which takes two specific concepts, this one searches
    /// the graph for any two concepts that share structural similarity — useful when
    /// we want to understand "what is X like?" without knowing Y in advance.
    pub fn find_any_analogy_for(&self, concept: &str) -> Vec<DynamicAnalogy> {
        let concept_lower = concept.to_lowercase();
        let mut all_analogies = Vec::new();

        // Find concepts that share a relationship with this one
        let rels_from = self.relationships.iter()
            .filter(|r| r.from.to_lowercase() == concept_lower || r.to.to_lowercase() == concept_lower)
            .collect::<Vec<_>>();

        for rel in &rels_from {
            // Find another relationship of the SAME type
            let others: Vec<_> = self.relationships.iter()
                .filter(|r| r.relation == rel.relation && r.from != rel.from && r.to != rel.to)
                .collect();

            for other in others {
                let shared_targets = rel.to.to_lowercase() == other.to.to_lowercase();
                let shared_sources = rel.from.to_lowercase() == other.from.to_lowercase();
                
                if !shared_targets && !shared_sources {
                    // Genuinely different — might be an analogy
                    let (src, tgt, oth_src, oth_tgt) = if rel.from.to_lowercase() == concept_lower {
                        (&rel.from, &rel.to, &other.from, &other.to)
                    } else {
                        (&rel.to, &rel.from, &other.from, &other.to)
                    };

                    all_analogies.push(DynamicAnalogy {
                        source: src.clone(),
                        source_relation: format!("{} {}", rel.relation.as_str(), tgt),
                        source_rel_type: rel.relation,
                        target: oth_src.clone(),
                        target_relation: format!("{} {}", other.relation.as_str(), oth_tgt),
                        target_rel_type: other.relation,
                        is_parallel: false,
                        confidence: 0.5,
                    });
                }
            }
        }

        // Also try transitive analogies: if A→X→Y and B→Z→W, and X≈Z, then A:Y :: B:W
        let rels_from_concept: Vec<_> = self.relationships.iter()
            .filter(|r| r.from.to_lowercase() == concept_lower)
            .collect();
        
        for rel in &rels_from_concept {
            let second_hop: Vec<_> = self.relationships.iter()
                .filter(|r| r.from.to_lowercase() == rel.to.to_lowercase())
                .collect();
            
            for sh in &second_hop {
                // Find another chain of same length with same relation type at both hops
                let matches: Vec<_> = self.relationships.iter()
                    .filter(|r| r.relation == rel.relation && r.from.to_lowercase() != concept_lower)
                    .collect();
                
                for m in &matches {
                    let m2: Vec<_> = self.relationships.iter()
                        .filter(|r| r.from.to_lowercase() == m.to.to_lowercase() && r.relation == sh.relation)
                        .collect();
                    
                    for m2 in m2 {
                        if rel.to.to_lowercase() != m.to.to_lowercase() {
                            all_analogies.push(DynamicAnalogy {
                                source: concept.to_string(),
                                source_relation: format!("{} → {} {}", rel.relation.as_str(), rel.to, sh.relation.as_str()),
                                source_rel_type: rel.relation,
                                target: m.from.clone(),
                                target_relation: format!("{} → {} {}", m.relation.as_str(), m.to, m2.relation.as_str()),
                                target_rel_type: m.relation,
                                is_parallel: false,
                                confidence: 0.6,
                            });
                        }
                    }
                }
            }
        }

        all_analogies.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        all_analogies.truncate(5);
        all_analogies
    }

    /// Get concepts that are related to the given concept (1-hop neighbors).
    pub fn neighbors(&self, concept: &str) -> Vec<(&Relationship, String)> {
        let concept_lower = concept.to_lowercase();
        let mut result = Vec::new();

        for rel in &self.relationships {
            if rel.from.to_lowercase() == concept_lower {
                result.push((rel, rel.to.clone()));
            }
            if rel.to.to_lowercase() == concept_lower {
                if let Some(_inv) = rel.relation.inverse() {
                    result.push((rel, rel.from.clone()));
                }
            }
        }

        result
    }
}

/// A dynamically discovered analogy from the knowledge graph.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicAnalogy {
    /// The source concept
    pub source: String,
    /// How the source relates (e.g., "causes heat")
    pub source_relation: String,
    /// The type of the source relationship
    pub source_rel_type: RelationType,
    /// The target concept
    pub target: String,
    /// How the target relates
    pub target_relation: String,
    /// The type of the target relationship
    pub target_rel_type: RelationType,
    /// Whether this is a parallel (same thing) or contrast (different)
    pub is_parallel: bool,
    /// Confidence in this analogy
    pub confidence: f64,
}

impl DynamicAnalogy {
    /// Human-readable explanation of this analogy.
    pub fn explanation(&self) -> String {
        if self.is_parallel {
            format!(
                "{} is to {} as {} is to {} — they share the same relational structure",
                self.source, self.source_relation, self.target, self.target_relation
            )
        } else {
            format!(
                "{} is like {}: {} — parallel structure across different domains",
                self.source, self.target, self.source_relation
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_operations() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity("Star");
        kg.add_fact("Star", "A reasoning intelligence");
        
        assert!(kg.get_entity("Star").is_some());
        assert_eq!(kg.entity_count(), 1);
    }

    #[test]
    fn test_relationships() {
        let mut kg = KnowledgeGraph::new();
        kg.add_relationship("Fire", RelationType::Causes, "Heat");
        kg.add_relationship("Heat", RelationType::Causes, "Expansion");
        
        let effects = kg.get_effects("Fire");
        assert!(effects.contains(&"causes Heat".to_string()));
    }

    #[test]
    fn test_transitive_inference() {
        let mut kg = KnowledgeGraph::new();
        kg.add_relationship("A", RelationType::IsA, "B");
        kg.add_relationship("B", RelationType::IsA, "C");
        kg.add_relationship("C", RelationType::IsA, "D");
        
        let inferred = kg.infer_transitive("A", &RelationType::IsA, 3);
        assert!(inferred.contains(&"B".to_string()));
        assert!(inferred.contains(&"C".to_string()));
        assert!(inferred.contains(&"D".to_string()));
    }

    #[test]
    fn test_find_connection() {
        let mut kg = KnowledgeGraph::new();
        kg.add_relationship("Fire", RelationType::Causes, "Heat");
        kg.add_relationship("Heat", RelationType::Causes, "Expansion");
        kg.add_relationship("Expansion", RelationType::Causes, "Pressure");
        
        let path = kg.find_connection("Fire", "Pressure", 5);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], "Fire");
        assert_eq!(path.last().unwrap(), "Pressure");
    }
}
