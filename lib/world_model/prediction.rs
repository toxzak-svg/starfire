//! Prediction — Forward modeling and outcome projection
//!
//! Uses world model state to predict future states and outcomes.

use super::{EntityId, PropertyValue, WorldModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A predicted future state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Unique ID for this prediction
    pub id: PredictionId,
    /// What we're predicting about
    pub subject: EntityId,
    /// Type of prediction
    pub prediction_type: PredictionType,
    /// The predicted content
    pub predicted: String,
    /// Confidence in the prediction (0-1)
    pub confidence: f64,
    /// How many steps in the future
    pub steps_ahead: usize,
    /// Reasoning chain for this prediction
    pub reasoning: Vec<String>,
    /// When this was created
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredictionId(u64);

impl PredictionId {
    pub fn new(n: u64) -> Self {
        Self(n)
    }
}

/// Types of predictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionType {
    /// Predicting a property value
    PropertyValue,
    /// Predicting an entity will change state
    StateChange,
    /// Predicting a causal outcome
    CausalOutcome,
    /// Predicting a relation will form
    RelationFormation,
    /// Predicting entity creation
    EntityCreation,
}

impl PredictionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PredictionType::PropertyValue => "property_value",
            PredictionType::StateChange => "state_change",
            PredictionType::CausalOutcome => "causal_outcome",
            PredictionType::RelationFormation => "relation_formation",
            PredictionType::EntityCreation => "entity_creation",
        }
    }
}

/// A candidate action for planning
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub description: String,
    pub preconditions: Vec<Precondition>,
    pub expected_outcomes: Vec<String>,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct Precondition {
    pub entity: EntityId,
    pub property: String,
    pub expected_value: PropertyValue,
}

/// Outcome of an action projection
#[derive(Debug, Clone)]
pub struct ProjectedOutcome {
    pub action: Action,
    pub predictions: Vec<Prediction>,
    pub success_probability: f64,
    pub risks: Vec<String>,
}

/// State transition probability
#[derive(Debug, Clone)]
pub struct TransitionProbability {
    pub from_state: HashMap<String, PropertyValue>,
    pub to_state: HashMap<String, PropertyValue>,
    pub probability: f64,
    pub evidence_count: usize,
}

impl WorldModel {
    /// Predict likely next state for an entity based on its relations
    pub fn predict_next_state(&self, entity_id: &EntityId) -> Option<ProjectedOutcome> {
        let entity = self.entities.get(entity_id)?;

        // Find causal neighbors
        let causal_neighbors: Vec<_> = entity
            .relations
            .iter()
            .filter(|r| r.relation_type == super::RelationType::CausallyRelated)
            .collect();

        if causal_neighbors.is_empty() {
            return None;
        }

        let mut predictions = Vec::new();
        let mut reasoning = Vec::new();

        reasoning.push(format!(
            "Based on {}'s causal relations:",
            entity.name
        ));

        for rel in &causal_neighbors {
            if let Some(target) = self.entities.get(&rel.target) {
                let predicted = format!(
                    "{} likely affects {}",
                    entity.name,
                    target.name
                );
                reasoning.push(predicted.clone());

                predictions.push(Prediction {
                    id: PredictionId(self.updates),
                    subject: rel.target.clone(),
                    prediction_type: PredictionType::CausalOutcome,
                    predicted,
                    confidence: rel.confidence * 0.7,
                    steps_ahead: 1,
                    reasoning: vec![],
                    created_at: crate::now_timestamp(),
                });
            }
        }

        Some(ProjectedOutcome {
            action: Action {
                name: format!("continue_{}", entity.name.to_lowercase()),
                description: format!("Continue observing {}", entity.name),
                preconditions: vec![],
                expected_outcomes: predictions.iter().map(|p| p.predicted.clone()).collect(),
                estimated_cost: 1.0,
            },
            predictions,
            success_probability: 0.6,
            risks: vec!["Prediction uncertainty".to_string()],
        })
    }

    /// Project outcome of an action
    pub fn project_action(&self, action: &Action) -> ProjectedOutcome {
        let mut predictions = Vec::new();
        let mut reasoning = Vec::new();
        reasoning.push(format!("Projecting action: {}", action.name));

        // Simple projection: check preconditions and generate predictions
        for pre in &action.preconditions {
            if let Some(entity) = self.entities.get(&pre.entity) {
                if let Some(actual_tp) = entity.get_current_value(&pre.property) {
                    if &actual_tp.value == &pre.expected_value {
                        reasoning.push(format!(
                            "Precondition met: {} is {:?}",
                            entity.name, pre.expected_value
                        ));
                    } else {
                        reasoning.push(format!(
                            "Precondition NOT met: {} is {:?}, expected {:?}",
                            entity.name, actual_tp.value, pre.expected_value
                        ));
                    }
                }
            }
        }

        // Generate predictions based on expected outcomes
        for outcome in &action.expected_outcomes {
            predictions.push(Prediction {
                id: PredictionId(self.updates + predictions.len() as u64),
                subject: EntityId::new("unknown".to_string()),
                prediction_type: PredictionType::StateChange,
                predicted: outcome.clone(),
                confidence: 0.5,
                steps_ahead: 1,
                reasoning: reasoning.clone(),
                created_at: crate::now_timestamp(),
            });
        }

        ProjectedOutcome {
            action: action.clone(),
            predictions,
            success_probability: 0.5,
            risks: vec!["Incomplete world model".to_string()],
        }
    }

    /// Evaluate probability of a property value
    pub fn evaluate_property_probability(
        &self,
        entity_id: &EntityId,
        property: &str,
        value: &PropertyValue,
    ) -> f64 {
        // Look at similar entities
        if let Some(entity) = self.entities.get(entity_id) {
            if let Some(current_tp) = entity.get_current_value(property) {
                if &current_tp.value == value {
                    return 0.9; // Already true
                }

                // Check causal neighbors for similar patterns
                let mut similar_count = 0;
                let mut total = 0;

                for rel in &entity.relations {
                    if rel.relation_type == super::RelationType::CausallyRelated {
                        total += 1;
                        if let Some(neighbor) = self.entities.get(&rel.target) {
                            if let Some(neighbor_tp) = neighbor.get_current_value(property) {
                                if &neighbor_tp.value == value {
                                    similar_count += 1;
                                }
                            }
                        }
                    }
                }

                if total > 0 {
                    return (similar_count as f64) / (total as f64) * 0.7;
                }
            }
        }
        0.1 // Default low probability
    }

    /// Find entities that might affect a given entity
    pub fn find_causal_influences(&self, entity_id: &EntityId) -> Vec<(EntityId, f64)> {
        let mut influences = Vec::new();

        for (id, entity) in &self.entities {
            if id == entity_id {
                continue;
            }

            // Check if this entity causally relates to our target
            if let Some(rel) = entity.relations.iter().find(|r| &r.target == entity_id) {
                if rel.relation_type == super::RelationType::CausallyRelated {
                    influences.push((id.clone(), rel.confidence));
                }
            }
        }

        influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        influences
    }

    /// Generate action candidates for achieving a goal
    pub fn generate_action_candidates(&self, goal: &str) -> Vec<Action> {
        let mut actions = Vec::new();

        // Find entities relevant to the goal
        let relevant: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.name.to_lowercase().contains(&goal.to_lowercase()))
            .collect();

        for entity in relevant {
            // For each relevant entity, suggest actions based on its relations
            for rel in &entity.relations {
                if let Some(target) = self.entities.get(&rel.target) {
                    let action = Action {
                        name: format!("affect_via_{}", rel.relation_type.as_str().replace(' ', "_")),
                        description: format!(
                            "Affect {} by modifying {} ({})",
                            entity.name, target.name, rel.relation_type.as_str()
                        ),
                        preconditions: vec![],
                        expected_outcomes: vec![format!(
                            "Changes to {} may affect {}",
                            target.name, entity.name
                        )],
                        estimated_cost: 1.0 + (1.0 - rel.confidence),
                    };
                    actions.push(action);
                }
            }
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_model::Entity;

    fn test_model() -> WorldModel {
        let mut model = WorldModel::new();

        let fire = Entity::new(EntityId::new("fire"), "Fire".to_string())
            .with_property("temperature", PropertyValue::Number(1000.0))
            .with_property("state", PropertyValue::from("burning"));
        let heat = Entity::new(EntityId::new("heat"), "Heat".to_string());
        let wood = Entity::new(EntityId::new("wood"), "Wood".to_string());

        model.upsert_entity(fire);
        model.upsert_entity(heat);
        model.upsert_entity(wood);

        model.add_relation(
            EntityId::new("fire"),
            EntityId::new("heat"),
            RelationType::CausallyRelated,
        );
        model.add_relation(
            EntityId::new("fire"),
            EntityId::new("wood"),
            RelationType::CausallyRelated,
        );

        model
    }

    #[test]
    fn test_predict_next_state() {
        let model = test_model();
        let outcome = model.predict_next_state(&EntityId::new("fire"));
        assert!(outcome.is_some());
        let outcome = outcome.unwrap();
        assert!(!outcome.predictions.is_empty());
    }

    #[test]
    fn test_find_causal_influences() {
        let model = test_model();
        let influences = model.find_causal_influences(&EntityId::new("heat"));
        assert!(influences.iter().any(|(id, _)| id.0 == "fire"));
    }

    #[test]
    fn test_generate_action_candidates() {
        let model = test_model();
        let actions = model.generate_action_candidates("fire");
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_evaluate_property_probability() {
        let model = test_model();
        let prob = model.evaluate_property_probability(
            &EntityId::new("fire"),
            "temperature",
            &PropertyValue::Number(1000.0),
        );
        assert!(prob > 0.8); // Already true
    }

    #[test]
    fn test_project_action() {
        let model = test_model();
        let action = Action {
            name: "test_action".to_string(),
            description: "Test".to_string(),
            preconditions: vec![],
            expected_outcomes: vec!["outcome1".to_string()],
            estimated_cost: 1.0,
        };
        let outcome = model.project_action(&action);
        assert_eq!(outcome.predictions.len(), 1);
    }
}
