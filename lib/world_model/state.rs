//! World State — Entity and relation management
//!
//! Provides structured access to the world model's entities,
//! relations, and state queries.

use super::{Entity, EntityId, PropertyValue, Relation, RelationType, WorldModel};
use std::collections::HashSet;

/// Query for filtering entities
#[derive(Debug, Clone)]
pub struct EntityQuery {
    /// Filter by name pattern
    pub name_pattern: Option<String>,
    /// Filter by property existence
    pub has_property: Option<String>,
    /// Filter by minimum confidence
    pub min_confidence: Option<f64>,
    /// Filter by relation type
    pub related_via: Option<RelationType>,
    /// Limit results
    pub limit: usize,
}

impl Default for EntityQuery {
    fn default() -> Self {
        Self {
            name_pattern: None,
            has_property: None,
            min_confidence: None,
            related_via: None,
            limit: 100,
        }
    }
}

impl EntityQuery {
    pub fn name(pattern: impl Into<String>) -> Self {
        Self {
            name_pattern: Some(pattern.into()),
            ..Default::default()
        }
    }

    pub fn with_property(mut self, prop: impl Into<String>) -> Self {
        self.has_property = Some(prop.into());
        self
    }

    pub fn with_min_confidence(mut self, conf: f64) -> Self {
        self.min_confidence = Some(conf);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }
}

impl WorldModel {
    /// Query entities with filters
    pub fn query_entities(&self, query: EntityQuery) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| {
                if let Some(ref pattern) = query.name_pattern {
                    if !e.name.to_lowercase().contains(&pattern.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref prop) = query.has_property {
                    if !e.temporal_properties.contains_key(prop) {
                        return false;
                    }
                }
                if let Some(min) = query.min_confidence {
                    if e.confidence < min {
                        return false;
                    }
                }
                if let Some(ref rel_type) = query.related_via {
                    if !e.relations.iter().any(|r| &r.relation_type == rel_type) {
                        return false;
                    }
                }
                true
            })
            .take(query.limit)
            .collect()
    }

    /// Get all entity IDs
    pub fn all_entity_ids(&self) -> Vec<EntityId> {
        self.entities.keys().cloned().collect()
    }

    /// Get entities by relation type
    pub fn entities_by_relation(&self, rel_type: RelationType) -> Vec<(&Entity, Vec<&Entity>)> {
        self.entities
            .values()
            .filter_map(|e| {
                let targets: Vec<&Entity> = e
                    .relations
                    .iter()
                    .filter(|r| r.relation_type == rel_type)
                    .filter_map(|r| self.entities.get(&r.target))
                    .collect();
                if targets.is_empty() {
                    None
                } else {
                    Some((e, targets))
                }
            })
            .collect()
    }

    /// Get the neighborhood of an entity (entities within N hops)
    pub fn entity_neighborhood(&self, id: &EntityId, hops: usize) -> HashSet<EntityId> {
        let mut visited = HashSet::new();
        let mut frontier = vec![id.clone()];
        visited.insert(id.clone());

        for _ in 0..hops {
            let mut next_frontier = Vec::new();
            for current_id in &frontier {
                if let Some(entity) = self.entities.get(current_id) {
                    for rel in &entity.relations {
                        if !visited.contains(&rel.target) {
                            visited.insert(rel.target.clone());
                            next_frontier.push(rel.target.clone());
                        }
                    }
                }
            }
            frontier = next_frontier;
        }

        visited
    }

    /// Remove an entity and all relations pointing to it
    pub fn remove_entity(&mut self, id: &EntityId) -> bool {
        if !self.entities.contains_key(id) {
            return false;
        }

        // Remove relations pointing to this entity from other entities
        for entity in self.entities.values_mut() {
            entity.relations.retain(|r| &r.target != id);
        }

        // Remove from name index
        if let Some(entity) = self.entities.get(id) {
            self.name_index.remove(&entity.name);
        }

        // Remove entity
        self.entities.remove(id);
        true
    }

    /// Get the current property value for an entity (convenience method)
    ///
    /// Returns the currently-valid PropertyValue, or None if the property
    /// doesn't exist or has no valid value at the current time.
    pub fn get_property(&self, entity_id: &EntityId, key: &str) -> Option<PropertyValue> {
        self.get_current_value(entity_id, key, None).map(|tp| tp.value.clone())
    }

    /// Get the current TemporalProperty for an entity (full temporal info)
    pub fn get_temporal_property(
        &self,
        entity_id: &EntityId,
        key: &str,
    ) -> Option<&TemporalProperty> {
        self.get_current_value(entity_id, key, None)
    }

    /// Check if two entities are directly related
    pub fn are_related(&self, id1: &EntityId, id2: &EntityId) -> bool {
        if let Some(entity) = self.entities.get(id1) {
            entity.relations.iter().any(|r| &r.target == id2)
        } else {
            false
        }
    }

    /// Get relation type between two entities if it exists
    pub fn relation_between(&self, id1: &EntityId, id2: &EntityId) -> Option<&Relation> {
        self.entities
            .get(id1)
            .and_then(|e| e.relations.iter().find(|r| &r.target == id2))
    }

    /// Create a relation from one entity to another
    pub fn add_relation(
        &mut self,
        from: EntityId,
        to: EntityId,
        rel_type: RelationType,
    ) -> bool {
        // Verify both entities exist
        if !self.entities.contains_key(&from) || !self.entities.contains_key(&to) {
            return false;
        }

        // Check if relation already exists
        if let Some(entity) = self.entities.get_mut(&from) {
            if entity.relations.iter().any(|r| r.target == to) {
                return false;
            }
            entity.relations.push(Relation {
                target: to,
                relation_type: rel_type,
                confidence: 0.5,
                temporal: false,
                observed_at: Some(crate::now_timestamp()),
            });
            true
        } else {
            false
        }
    }

    /// Get statistics about the world model
    pub fn stats(&self) -> WorldModelStats {
        let mut relation_counts = std::collections::HashMap::new();
        let mut total_relations = 0;

        for entity in self.entities.values() {
            for rel in &entity.relations {
                *relation_counts.entry(rel.relation_type.clone()).or_insert(0) += 1;
                total_relations += 1;
            }
        }

        WorldModelStats {
            entity_count: self.entities.len(),
            total_relations,
            relation_type_counts: relation_counts,
            update_count: self.updates,
        }
    }
}

// Re-export TemporalProperty for convenience
use super::TemporalProperty;

/// Statistics about the world model
#[derive(Debug, Clone)]
pub struct WorldModelStats {
    pub entity_count: usize,
    pub total_relations: usize,
    pub relation_type_counts: std::collections::HashMap<RelationType, usize>,
    pub update_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> WorldModel {
        let mut model = WorldModel::new();

        let mut e1 = Entity::new(EntityId::new("e1"), "Fire".to_string());
        e1.set_property("temperature", PropertyValue::Number(1000.0));
        let e2 = Entity::new(EntityId::new("e2"), "Heat".to_string());
        let e3 = Entity::new(EntityId::new("e3"), "Water".to_string());

        model.upsert_entity(e1);
        model.upsert_entity(e2);
        model.upsert_entity(e3);

        model.add_relation(
            EntityId::new("e1"),
            EntityId::new("e2"),
            RelationType::CausallyRelated,
        );

        model
    }

    #[test]
    fn test_add_relation() {
        let model = test_model();
        assert!(model.are_related(&EntityId::new("e1"), &EntityId::new("e2")));
        assert!(!model.are_related(&EntityId::new("e1"), &EntityId::new("e3")));
    }

    #[test]
    fn test_get_property() {
        let model = test_model();
        let temp = model.get_property(&EntityId::new("e1"), "temperature");
        assert!(temp.is_some());
        assert_eq!(temp.unwrap(), PropertyValue::Number(1000.0));
    }

    #[test]
    fn test_query_entities() {
        let model = test_model();
        let fires = model.query_entities(EntityQuery::name("fire"));
        assert_eq!(fires.len(), 1);

        let all = model.query_entities(EntityQuery::default().limit(10));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_neighborhood() {
        let model = test_model();
        let neighborhood = model.entity_neighborhood(&EntityId::new("e1"), 1);
        assert!(neighborhood.contains(&EntityId::new("e1")));
        assert!(neighborhood.contains(&EntityId::new("e2")));
    }

    #[test]
    fn test_remove_entity() {
        let mut model = test_model();
        assert!(model.remove_entity(&EntityId::new("e3")));
        assert_eq!(model.entity_count(), 2);
        // e1's relation to e3 should be removed
        assert!(!model.are_related(&EntityId::new("e1"), &EntityId::new("e3")));
    }

    #[test]
    fn test_stats() {
        let model = test_model();
        let stats = model.stats();
        assert_eq!(stats.entity_count, 3);
        assert_eq!(stats.total_relations, 1);
    }

    #[test]
    fn test_get_temporal_property() {
        let model = test_model();
        let tp = model.get_temporal_property(&EntityId::new("e1"), "temperature");
        assert!(tp.is_some());
        assert_eq!(tp.unwrap().value, PropertyValue::Number(1000.0));
        assert!(tp.unwrap().valid_until.is_none()); // Currently valid
    }

    #[test]
    fn test_query_entities_with_property_filter() {
        let model = test_model();
        let with_temp = model.query_entities(EntityQuery::default().with_property("temperature"));
        assert_eq!(with_temp.len(), 1);
        assert_eq!(with_temp[0].name, "Fire");
    }
}
