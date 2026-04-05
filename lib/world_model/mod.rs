//! World Model — Grounded perceptual representation of the world
//!
//! Bridges Quanot's reservoir states to Starfire's symbolic reasoning.
//! Maintains an explicit model of entities, relations, and predicted outcomes.
//!
//! # Architecture
//!
//! ```text
//! Quanot Reservoir State
//!         ↓
//!    Perception Input
//!         ↓
//! WorldModel.update_from_perception()
//!         ↓
//!    Entity/Relation Update
//!         ↓
//!    Predictive Modeling
//!         ↓
//! Starfire Knowledge Graph ← Reasoning Engine
//! ```

pub mod perception;
pub mod state;
pub mod prediction;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for an entity in the world model
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// A fact about the world — entity with properties and relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub properties: HashMap<String, PropertyValue>,
    pub relations: Vec<Relation>,
    pub last_updated: i64,
    pub confidence: f64,
}

impl Entity {
    pub fn new(id: EntityId, name: String) -> Self {
        Self {
            id,
            name,
            properties: HashMap::new(),
            relations: Vec::new(),
            last_updated: crate::now_timestamp(),
            confidence: 0.5,
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn with_relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
    }
}

/// A property value — can be various types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<f64> for PropertyValue {
    fn from(n: f64) -> Self {
        PropertyValue::Number(n)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Boolean(b)
    }
}

/// A relation between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub target: EntityId,
    pub relation_type: RelationType,
    pub confidence: f64,
    pub temporal: bool,
    pub observed_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    CausallyRelated,
    SpatiallyRelated,
    TemporallyRelated,
    ConceptuallyRelated,
    PartOf,
    InstanceOf,
    SimilarTo,
    Other,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::CausallyRelated => "is causally related to",
            RelationType::SpatiallyRelated => "is spatially near",
            RelationType::TemporallyRelated => "happens around the same time as",
            RelationType::ConceptuallyRelated => "is conceptually linked to",
            RelationType::PartOf => "is part of",
            RelationType::InstanceOf => "is an instance of",
            RelationType::SimilarTo => "is similar to",
            RelationType::Other => "is related to",
        }
    }
}

/// The main World Model struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModel {
    /// All entities by ID
    entities: HashMap<EntityId, Entity>,
    /// Entity name index for fast lookup
    name_index: HashMap<String, EntityId>,
    /// Quanot perception input (most recent)
    pub last_perception: Option<perception::QuanotPerception>,
    /// Prediction history
    predictions: Vec<prediction::Prediction>,
    /// Update counter
    updates: u64,
}

impl Default for WorldModel {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldModel {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            name_index: HashMap::new(),
            last_perception: None,
            predictions: Vec::new(),
            updates: 0,
        }
    }

    /// Update world model from Quanot perception input
    pub fn update_from_perception(&mut self, perception: perception::QuanotPerception) {
        self.last_perception = Some(perception.clone());
        self.updates += 1;

        // Update based on consciousness proxy — high consciousness = high confidence update
        let confidence_weight = perception.consciousness_proxy;

        // If novelty is high, create or update entities based on pattern
        if perception.novelty > 0.3 {
            self.process_novelty(perception, confidence_weight);
        }
    }

    fn process_novelty(&mut self, perception: perception::QuanotPerception, _confidence_weight: f64) {
        // Extract patterns from creativity scores and update entity relations
        let cs = &perception.creativity_scores;

        // Create emergent entity from high-diversity states
        if cs.diversity_index > 0.6 {
            let entity_id = EntityId::new(format!("emergent_state_{}", self.updates));
            let mut entity = Entity::new(entity_id.clone(), format!("EmergentState_{}", self.updates));

            entity.properties.insert(
                "diversity_index".into(),
                PropertyValue::Number(cs.diversity_index),
            );
            entity.properties.insert(
                "novelty".into(),
                PropertyValue::Number(perception.novelty),
            );
            entity.properties.insert(
                "consciousness_proxy".into(),
                PropertyValue::Number(perception.consciousness_proxy),
            );

            self.entities.insert(entity_id.clone(), entity);
            self.name_index.insert(format!("EmergentState_{}", self.updates), entity_id);
        }
    }

    /// Add or update an entity
    pub fn upsert_entity(&mut self, entity: Entity) {
        let id = entity.id.clone();
        let name = entity.name.clone();
        self.entities.insert(id.clone(), entity);
        self.name_index.insert(name, id);
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Get an entity by name
    pub fn get_by_name(&self, name: &str) -> Option<&Entity> {
        self.name_index.get(name).and_then(|id| self.entities.get(id))
    }

    /// Get all entities matching a pattern in name
    pub fn find_entities(&self, pattern: &str) -> Vec<&Entity> {
        let pattern_lower = pattern.to_lowercase();
        self.entities
            .values()
            .filter(|e| e.name.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// Get entities related to a given entity
    pub fn get_related_entities(&self, entity_id: &EntityId) -> Vec<(Entity, Relation)> {
        let entity = match self.entities.get(entity_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        entity
            .relations
            .iter()
            .filter_map(|rel| {
                self.entities.get(&rel.target).map(|e| (e.clone(), rel.clone()))
            })
            .collect()
    }

    /// Add a prediction to the history
    pub fn add_prediction(&mut self, prediction: prediction::Prediction) {
        self.predictions.push(prediction);
        // Keep only recent predictions
        if self.predictions.len() > 100 {
            self.predictions.remove(0);
        }
    }

    /// Get all predictions
    pub fn predictions(&self) -> &[prediction::Prediction] {
        &self.predictions
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get update count
    pub fn update_count(&self) -> u64 {
        self.updates
    }

    /// Query world state — returns all facts relevant to a topic
    pub fn query(&self, topic: &str) -> Vec<String> {
        let mut facts = Vec::new();

        for entity in self.entities.values() {
            if entity.name.to_lowercase().contains(&topic.to_lowercase()) {
                facts.push(format!("{} is a {}", entity.name, self.entity_type_label(&entity.id)));

                for (key, value) in &entity.properties {
                    facts.push(format!("  - {}: {:?}", key, value));
                }

                for rel in &entity.relations {
                    if let Some(target) = self.entities.get(&rel.target) {
                        facts.push(format!("  - {} {}", rel.relation_type.as_str(), target.name));
                    }
                }
            }
        }

        facts
    }

    fn entity_type_label(&self, id: &EntityId) -> &'static str {
        let name = id.0.as_str();
        if name.starts_with("emergent_state_") {
            "emergent perceptual state"
        } else if name.starts_with("concept_") {
            "concept"
        } else {
            "entity"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(EntityId::new("test_1"), "TestEntity".to_string())
            .with_property("color", PropertyValue::from("blue"))
            .with_property("size", PropertyValue::from(42.0));

        assert_eq!(entity.name, "TestEntity");
        assert_eq!(entity.properties.len(), 2);
    }

    #[test]
    fn test_world_model_update() {
        let mut model = WorldModel::new();

        let perception = perception::QuanotPerception {
            reservoir_state: vec![0.1; 100],
            consciousness_proxy: 0.7,
            novelty: 0.5,
            creativity_scores: perception::CreativityOutput::default(),
        };

        model.update_from_perception(perception);

        assert_eq!(model.update_count(), 1);
        assert!(model.last_perception.is_some());
    }

    #[test]
    fn test_find_entities() {
        let mut model = WorldModel::new();
        model.upsert_entity(Entity::new(EntityId::new("cat_1"), "Cat Whiskers".to_string()));
        model.upsert_entity(Entity::new(EntityId::new("dog_1"), "Buddy Dog".to_string()));
        model.upsert_entity(Entity::new(EntityId::new("cat_2"), "Cat Mittens".to_string()));

        let cats = model.find_entities("cat");
        assert_eq!(cats.len(), 2);

        let buddies = model.find_entities("buddy");
        assert_eq!(buddies.len(), 1);
    }

    #[test]
    fn test_query() {
        let mut model = WorldModel::new();
        let mut entity = Entity::new(EntityId::new("fire_1"), "Fire".to_string());
        entity.properties.insert("temperature".into(), PropertyValue::Number(1000.0));
        entity.properties.insert("state".into(), PropertyValue::from("hot"));
        model.upsert_entity(entity);

        let facts = model.query("fire");
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.contains("Fire")));
    }
}
