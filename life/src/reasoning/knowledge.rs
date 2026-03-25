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
            _ => None,
        }
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
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
        if from.len() < 2 || to.len() < 2 { return; }
        
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
        self.add_entity(entity);
        if let Some(e) = self.entities.get_mut(entity) {
            e.properties.insert(property.to_string(), value.to_string());
        }
    }

    /// Add a fact about an entity.
    pub fn add_fact(&mut self, entity: &str, fact: &str) {
        self.add_entity(entity);
        if let Some(e) = self.entities.get_mut(entity) {
            e.description = Some(fact.to_string());
        }
    }

    /// Get an entity by name.
    pub fn get_entity(&self, name: &str) -> Option<&Entity> {
        self.entities.get(name)
    }

    /// Get all entity names.
    pub fn entities(&self) -> impl Iterator<Item = &str> {
        self.entities.keys().map(|s| s.as_str())
    }

    /// Get all relationships from an entity.
    pub fn get_relationships_from(&self, entity: &str) -> Vec<&Relationship> {
        self.relationships.iter().filter(|r| r.from == entity).collect()
    }

    /// Get all relationships to an entity.
    pub fn get_relationships_to(&self, entity: &str) -> Vec<&Relationship> {
        self.relationships.iter().filter(|r| r.to == entity).collect()
    }

    /// Get all facts about an entity.
    pub fn get_facts_about(&self, entity: &str) -> Vec<String> {
        let mut facts = Vec::new();
        
        // Properties
        if let Some(e) = self.entities.get(entity) {
            for (prop, val) in &e.properties {
                facts.push(format!("{} has {}", entity, val));
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
            if rel.from.to_lowercase().contains(&topic_lower) ||
               rel.to.to_lowercase().contains(&topic_lower) {
                if rel.relation == RelationType::RelatedTo {
                    related.push(format!("{} {} {}", rel.from, rel.relation.as_str(), rel.to));
                }
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
