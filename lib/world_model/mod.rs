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
pub mod temporal;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use temporal::{
    TemporalProperty, PropertyValue, DecayFunction, DecayParams,
    compute_staleness, TemporalQuery,
};

/// A unique identifier for an entity in the world model
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// A fact about the world — entity with temporal properties and relations
///
/// Each property has a validity window (valid_from, valid_until) that tracks
/// when the fact was true. Historical values are preserved rather than overwritten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    /// Map of property_name → Vec of temporal values (history)
    /// Values are kept sorted by valid_from (most recent first within each property)
    pub temporal_properties: HashMap<String, Vec<TemporalProperty>>,
    pub relations: Vec<Relation>,
    pub last_updated: i64,
    pub confidence: f64,
}

impl Entity {
    pub fn new(id: EntityId, name: String) -> Self {
        Self {
            id,
            name,
            temporal_properties: HashMap::new(),
            relations: Vec::new(),
            last_updated: crate::now_timestamp(),
            confidence: 0.5,
        }
    }

    /// Update a property with a new value, closing the previous value's validity window
    ///
    /// This preserves the full history of property changes, enabling queries like
    /// "what was X's value at time T?" and "when did X change?".
    pub fn update_property(
        &mut self,
        key: String,
        value: PropertyValue,
        confidence: f64,
        decay_fn: DecayFunction,
    ) {
        // Use last_updated as valid_from if set, otherwise fall back to now
        // This allows callers (like time-travel tests) to set specific timestamps
        let valid_from = if self.last_updated > 0 {
            self.last_updated
        } else {
            crate::now_timestamp()
        };

        // Close out the current (open) value if it exists
        if let Some(props) = self.temporal_properties.get_mut(&key) {
            if let Some(last) = props.last_mut() {
                if last.valid_until.is_none() {
                    last.valid_until = Some(valid_from);
                }
            }
            props.push(TemporalProperty {
                value,
                valid_from,
                valid_until: None,
                confidence,
                decay_fn,
            });
        } else {
            self.temporal_properties.insert(
                key,
                vec![TemporalProperty {
                    value,
                    valid_from,
                    valid_until: None,
                    confidence,
                    decay_fn,
                }],
            );
        }
        self.last_updated = valid_from;
    }

    /// Get the current (most recent) property value, if any
    pub fn get_current_value(&self, key: &str) -> Option<&TemporalProperty> {
        let now = crate::now_timestamp();
        self.temporal_properties.get(key)?.iter().find(|p| {
            p.valid_from <= now && p.valid_until.unwrap_or(i64::MAX) > now
        })
    }

    /// Get all historical values for a property, most recent first
    pub fn get_property_history(&self, key: &str) -> Option<&Vec<TemporalProperty>> {
        self.temporal_properties.get(key)
    }

    /// Set a simple property (convenience for backward compatibility)
    /// This creates a TemporalProperty with DecayFunction::None
    pub fn set_property(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.update_property(key.into(), value, 1.0, DecayFunction::None);
    }

    /// Set a property and return self for chaining (builder pattern)
    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.set_property(key, value);
        self
    }

    /// Add a relation to another entity
    pub fn with_relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
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

            entity.set_property(
                "diversity_index",
                PropertyValue::Number(cs.diversity_index),
            );
            entity.set_property(
                "novelty",
                PropertyValue::Number(perception.novelty),
            );
            entity.set_property(
                "consciousness_proxy",
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

    /// Get the currently valid value for entity.property at given time
    ///
    /// Returns the property value that was valid at the specified time,
    /// or the current value if `valid_at` is None.
    pub fn get_current_value(
        &self,
        entity_id: &EntityId,
        property: &str,
        valid_at: Option<i64>,
    ) -> Option<&TemporalProperty> {
        let now = valid_at.unwrap_or_else(crate::now_timestamp);
        self.entities
            .get(entity_id)?
            .temporal_properties
            .get(property)?
            .iter()
            .find(|p| p.valid_from <= now && p.valid_until.unwrap_or(i64::MAX) > now)
    }

    /// Get property history (all versions), filtered by staleness
    ///
    /// Returns all historical values of a property whose staleness score
    /// is below the given threshold. Useful for "show me recent changes"
    /// or "when did X change?" queries.
    pub fn get_property_history(
        &self,
        entity_id: &EntityId,
        property: &str,
        max_staleness: f64,
    ) -> Vec<&TemporalProperty> {
        let now = crate::now_timestamp();
        self.entities
            .get(entity_id)
            .and_then(|e| e.temporal_properties.get(property))
            .map(|props| {
                props.iter()
                    .filter(|p| {
                        compute_staleness(p, now, &DecayParams::default()) <= max_staleness
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query entities with temporal filtering
    ///
    /// This extends `query_entities` with temporal awareness:
    /// - Filters to entities with properties valid at the specified time
    /// - Optionally filters by staleness threshold
    /// - Can include entities with only stale (superseded) properties
    pub fn query_temporal(&self, query: TemporalQuery) -> Vec<&Entity> {
        let now = query.valid_at.unwrap_or_else(crate::now_timestamp);
        let decay_params = query.decay_override.unwrap_or_default();

        self.entities
            .values()
            .filter(|e| {
                // Apply base EntityQuery filters
                if let Some(ref pattern) = query.base.name_pattern {
                    if !e.name.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(min) = query.base.min_confidence {
                    if e.confidence < min {
                        return false;
                    }
                }
                if let Some(ref prop) = query.base.has_property {
                    if !e.temporal_properties.contains_key(prop) {
                        return false;
                    }
                }

                // Temporal filter: at least one property valid at `now`
                let has_valid = e.temporal_properties.values().any(|props| {
                    props.iter().any(|p| p.valid_from <= now && p.valid_until.unwrap_or(i64::MAX) > now)
                });
                if !has_valid && !query.include_stale {
                    return false;
                }

                // Staleness filter
                if let Some(max_stale) = query.max_staleness {
                    let min_staleness = e.temporal_properties.values()
                        .flatten()
                        .map(|p| compute_staleness(p, now, &decay_params))
                        .fold(f64::MAX, f64::min);
                    if min_staleness > max_stale {
                        return false;
                    }
                }

                // Min recency filter (inverted staleness)
                if let Some(min_recency) = query.min_recency {
                    let max_allowed_stale = 1.0 - min_recency;
                    let min_staleness = e.temporal_properties.values()
                        .flatten()
                        .map(|p| compute_staleness(p, now, &decay_params))
                        .fold(f64::MAX, f64::min);
                    if min_staleness > max_allowed_stale {
                        return false;
                    }
                }

                true
            })
            .take(query.base.limit)
            .collect()
    }

    /// Query world state — returns all facts relevant to a topic
    pub fn query(&self, topic: &str) -> Vec<String> {
        let mut facts = Vec::new();

        for entity in self.entities.values() {
            if entity.name.to_lowercase().contains(&topic.to_lowercase()) {
                facts.push(format!("{} is a {}", entity.name, self.entity_type_label(&entity.id)));

                for (key, props) in &entity.temporal_properties {
                    if let Some(current) = props.iter().find(|p| {
                        let now = crate::now_timestamp();
                        p.valid_from <= now && p.valid_until.unwrap_or(i64::MAX) > now
                    }) {
                        facts.push(format!("  - {}: {:?}", key, current.value));
                    }
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
        let mut entity = Entity::new(EntityId::new("test_1"), "TestEntity".to_string());
        entity.set_property("color", PropertyValue::from("blue"));
        entity.set_property("size", PropertyValue::from(42.0));

        assert_eq!(entity.name, "TestEntity");
        assert_eq!(entity.temporal_properties.len(), 2);
        assert_eq!(entity.get_current_value("color").unwrap().value, PropertyValue::String("blue".to_string()));
    }

    #[test]
    fn test_entity_update_property_closes_previous() {
        let mut entity = Entity::new(EntityId::new("test_1"), "TestEntity".to_string());
        entity.update_property(
            "status".to_string(),
            PropertyValue::from("active"),
            1.0,
            DecayFunction::None,
        );

        let history = entity.get_property_history("status").unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].valid_until.is_none());

        // Update the property - should close the previous one
        entity.update_property(
            "status".to_string(),
            PropertyValue::from("inactive"),
            1.0,
            DecayFunction::None,
        );

        let history = entity.get_property_history("status").unwrap();
        assert_eq!(history.len(), 2);
        // First entry should now be closed
        assert!(history[0].valid_until.is_some());
        // Second entry should still be open
        assert!(history[1].valid_until.is_none());
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
        entity.set_property("temperature", PropertyValue::Number(1000.0));
        entity.set_property("state", PropertyValue::from("hot"));
        model.upsert_entity(entity);

        let facts = model.query("fire");
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.contains("Fire")));
    }

    #[test]
    fn test_get_current_value_with_time_travel() {
        let mut model = WorldModel::new();
        let mut entity = Entity::new(EntityId::new("temp_1"), "Temperature".to_string());

        // Simulate setting a value at time 100
        entity.last_updated = 100;
        entity.update_property(
            "value".to_string(),
            PropertyValue::Number(20.0),
            1.0,
            DecayFunction::None,
        );

        // Set a new value at time 200
        entity.last_updated = 200;
        entity.update_property(
            "value".to_string(),
            PropertyValue::Number(25.0),
            1.0,
            DecayFunction::None,
        );

        model.upsert_entity(entity);

        // Query at time 150 - should get the first value
        let val = model.get_current_value(&EntityId::new("temp_1"), "value", Some(150));
        assert!(val.is_some());
        assert_eq!(val.unwrap().value, PropertyValue::Number(20.0));

        // Query at time 250 - should get the second value
        let val = model.get_current_value(&EntityId::new("temp_1"), "value", Some(250));
        assert!(val.is_some());
        assert_eq!(val.unwrap().value, PropertyValue::Number(25.0));
    }

    #[test]
    fn test_query_temporal() {
        let mut model = WorldModel::new();
        let mut entity = Entity::new(EntityId::new("fact_1"), "TestFact".to_string());
        entity.set_property("status", PropertyValue::from("active"));
        model.upsert_entity(entity);

        // Query for entities with valid properties
        let query = TemporalQuery {
            base: state::EntityQuery::default(),
            valid_at: None,
            min_recency: None,
            max_staleness: Some(1.0), // Allow any staleness
            include_stale: false,
            decay_override: None,
        };

        let results = model.query_temporal(query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_property_history_with_staleness() {
        let mut model = WorldModel::new();
        let mut entity = Entity::new(EntityId::new("hist_1"), "HistoryTest".to_string());

        // Add properties with different decay rates
        entity.update_property(
            "fast_fact".to_string(),
            PropertyValue::from("fast_value"),
            1.0,
            DecayFunction::DomainHalfLife { half_life_hours: 1.0 },
        );
        entity.update_property(
            "slow_fact".to_string(),
            PropertyValue::from("slow_value"),
            1.0,
            DecayFunction::DomainHalfLife { half_life_hours: 168.0 },
        );

        model.upsert_entity(entity);

        // Both should be fresh right now
        let fast_history = model.get_property_history(&EntityId::new("hist_1"), "fast_fact", 1.0);
        assert_eq!(fast_history.len(), 1);

        let slow_history = model.get_property_history(&EntityId::new("hist_1"), "slow_fact", 1.0);
        assert_eq!(slow_history.len(), 1);
    }
}
