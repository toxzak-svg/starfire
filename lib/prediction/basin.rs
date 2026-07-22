//! Attractor Basin Engine — constraint satisfaction as prediction
//!
//! Philosophy: The knowledge graph is a constraint satisfaction landscape.
//! Stable truths are attractor basins. The predicted future is the equilibrium
//! state the landscape converges to under constraint pressure.

use super::types::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Attractor Basin Engine — constraint satisfaction as prediction
pub struct BasinEngine {
    /// Constraint graph nodes
    nodes: HashMap<NodeId, BasinNode>,
    /// Constraints between nodes
    constraints: Vec<Constraint>,
    /// Current basin assignments
    basin_state: BasinState,
    /// Maximum nodes to track
    max_nodes: usize,
}

/// Unique node identifier in the constraint graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: &str) -> Self {
        NodeId(id.to_string())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node in the constraint graph
#[derive(Debug, Clone)]
struct BasinNode {
    /// Current most-confident value
    pub value: PropertyValue,
    /// Confidence in current value
    pub confidence: f64,
    /// Possible alternative values (other basins)
    pub alternatives: Vec<AlternativeBasin>,
}

/// An alternative basin for a node
#[derive(Debug, Clone)]
struct AlternativeBasin {
    pub value: PropertyValue,
    pub energy: f64, // Lower energy = more stable
}

/// A constraint between nodes
#[derive(Debug, Clone)]
struct Constraint {
    pub id: ConstraintId,
    pub from: NodeId,
    pub to: NodeId,
    pub constraint_type: ConstraintType,
    pub strength: f64,
    pub satisfied: bool,
}

/// Unique constraint identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstraintId(String);

impl ConstraintId {
    pub fn new(id: &str) -> Self {
        ConstraintId(id.to_string())
    }
}

/// Type of constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// If A then B
    Implication,
    /// A and B cannot both be true
    Exclusion,
    /// A and B must be equal
    Equality,
    /// A causes B
    Causation,
    /// A enables B
    Enablement,
    /// A is similar to B (constrains to similar values)
    Analogy,
}

/// Current basin state
#[derive(Debug, Clone)]
struct BasinState {
    /// Current assignment of nodes to basins
    assignments: HashMap<NodeId, PropertyValue>,
}

impl Default for BasinState {
    fn default() -> Self {
        BasinState {
            assignments: HashMap::new(),
        }
    }
}

impl BasinEngine {
    pub fn new() -> Self {
        BasinEngine {
            nodes: HashMap::new(),
            constraints: Vec::new(),
            basin_state: BasinState::default(),
            max_nodes: 100,
        }
    }

    /// Add a node to the constraint graph
    pub fn add_node(&mut self, id: &str, value: PropertyValue, confidence: f64) {
        let node_id = NodeId::new(id);
        if self.nodes.len() >= self.max_nodes && !self.nodes.contains_key(&node_id) {
            return;
        }

        // Generate alternatives based on value type
        let alternatives = Self::generate_alternatives(&value);
        
        self.nodes.insert(node_id, BasinNode {
            value,
            confidence,
            alternatives,
        });
        
        self.basin_state.assignments.insert(NodeId::new(id), PropertyValue::Unknown);
    }

    /// Generate alternative basins for a value
    fn generate_alternatives(value: &PropertyValue) -> Vec<AlternativeBasin> {
        match value {
            PropertyValue::String(s) => vec![
                AlternativeBasin { value: value.clone(), energy: 0.1 },
                AlternativeBasin { value: PropertyValue::String(format!("not {}", s)), energy: 0.5 },
            ],
            PropertyValue::Boolean(b) => vec![
                AlternativeBasin { value: PropertyValue::Boolean(*b), energy: 0.1 },
                AlternativeBasin { value: PropertyValue::Boolean(!*b), energy: 0.5 },
            ],
            _ => vec![AlternativeBasin { value: value.clone(), energy: 0.1 }],
        }
    }

    /// Add a constraint between nodes
    pub fn add_constraint(&mut self, from: &str, to: &str, constraint_type: ConstraintType, strength: f64) {
        let from_id = NodeId::new(from);
        let to_id = NodeId::new(to);
        
        // Only add if both nodes exist
        if self.nodes.contains_key(&from_id) && self.nodes.contains_key(&to_id) {
            self.constraints.push(Constraint {
                id: ConstraintId::new(&format!("{}->{:?}_{}", from, constraint_type, to)),
                from: from_id,
                to: to_id,
                constraint_type,
                strength,
                satisfied: true, // Initially satisfied
            });
        }
    }

    /// Add a causal relationship (A causes B)
    pub fn add_causal(&mut self, cause: &str, effect: &str, strength: f64) {
        self.add_constraint(cause, effect, ConstraintType::Causation, strength);
    }

    /// Predict the equilibrium state given current constraints
    /// Returns necessary truths and predicted state changes
    pub fn predict_equilibrium(&self) -> Vec<Prediction> {
        let mut predictions = Vec::new();

        // 1. Find all nodes that are "under pressure" (conflicts, missing values)
        let pressure_nodes = self.find_pressure_nodes();

        // 2. For each pressured node, compute where it wants to go
        for node_id in pressure_nodes {
            if let Some(pressure) = self.compute_pressure(&node_id) {
                if let Some(prediction) = self.basinto_prediction(node_id.clone(), &pressure) {
                    predictions.push(prediction);
                }
            }
        }

        // 3. Find paths of necessary propagation
        let necessary_truths = self.find_necessary_propagations();
        predictions.extend(necessary_truths);

        predictions
    }

    /// Find nodes under constraint pressure
    fn find_pressure_nodes(&self) -> Vec<NodeId> {
        let mut pressured = Vec::new();

        for (node_id, node) in &self.nodes {
            // Pressure = conflict OR low confidence + many constraints
            let has_conflict = node.alternatives.len() > 1 
                && node.alternatives.first().map(|a| a.energy)
                    .unwrap_or(0.0) 
                    < node.alternatives.get(1).map(|a| a.energy).unwrap_or(1.0) - 0.3;

            let low_confidence_constraint = node.confidence < 0.7
                && self.node_constraint_count(node_id) >= 2;

            if has_conflict || low_confidence_constraint {
                pressured.push(node_id.clone());
            }
        }

        pressured
    }

    /// Count constraints on a node
    fn node_constraint_count(&self, node_id: &NodeId) -> usize {
        self.constraints
            .iter()
            .filter(|constraint| {
                constraint.satisfied
                    && (constraint.from == *node_id || constraint.to == *node_id)
            })
            .count()
    }

    /// Compute where a basin wants to move under constraint pressure
    fn compute_pressure(&self, node_id: &NodeId) -> Option<BasinPressure> {
        let _node = self.nodes.get(node_id)?;

        // Sum constraint forces
        let mut force_direction: HashMap<String, f64> = HashMap::new();
        let constraints_on: Vec<_> = self
            .constraints
            .iter()
            .filter(|constraint| constraint.satisfied && constraint.to == *node_id)
            .collect();

        for constraint in constraints_on {
            // Get source node's value
            if let Some(source_node) = self.nodes.get(&constraint.from) {
                let source_val = format!("{}", source_node.value);
                let force = constraint.strength;
                *force_direction.entry(source_val).or_insert(0.0) += force;
            }
        }

        // Find the value with highest attractive force
        force_direction
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(value, force)| BasinPressure {
                value: PropertyValue::from_str(&value),
                force,
            })
    }

    /// Convert basin pressure to a prediction
    fn basinto_prediction(&self, node_id: NodeId, pressure: &BasinPressure) -> Option<Prediction> {
        let node = self.nodes.get(&node_id)?;
        let active_constraint_ids = self
            .constraints
            .iter()
            .filter(|constraint| {
                constraint.satisfied
                    && (constraint.from == node_id || constraint.to == node_id)
            })
            .map(|constraint| constraint.id.0.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Some(Prediction::new(
            PredictionEngine::Basin,
            PredictionKind::StateChange,
            PredictedCore::StateChange {
                entity_id: node_id.0.clone(),
                property: "value".to_string(),
                from: node.value.clone(),
                to: pressure.value.clone(),
            },
            format!("Node '{}' under pressure toward: {}", node_id, pressure.value),
            (node.confidence * pressure.force).clamp(0.1, 0.9),
            1, // Short horizon for basin predictions
            vec![
                format!("Current value: {}", node.value),
                format!("Pressure force: {:.3}", pressure.force),
                format!("Constraint count: {}", self.node_constraint_count(&node_id)),
                format!("Active constraints: {}", active_constraint_ids),
                format!("Active constraints: {}", active_constraint_ids),
            ],
        ))
    }

    /// Propagate necessity through causal chains
    /// If A causes B and B causes C and A is true → C is a necessary truth
    fn find_necessary_propagations(&self) -> Vec<Prediction> {
        let mut predictions = Vec::new();

        // Find all causal chains where the root is true with high confidence
        for (node_id, node) in &self.nodes {
            if node.confidence > 0.8 {
                // Trace causal chains forward
                let chain = self.trace_causal_chain(node_id);

                if chain.len() > 1 {
                    // Every node in the chain after the first is a necessary truth
                    for chain_node in chain.iter().skip(1) {
                        if let Some(node) = self.nodes.get(chain_node) {
                            let necessary_value = node.alternatives.first()
                                .map(|a| a.value.clone())
                                .unwrap_or(PropertyValue::Unknown);

                            predictions.push(Prediction::new(
                                PredictionEngine::Basin,
                                PredictionKind::NecessaryTruth,
                                PredictedCore::NecessaryTruth {
                                    entity_id: chain_node.0.clone(),
                                    property: "value".to_string(),
                                    value: necessary_value,
                                    constraint_source: format!("causal chain from {}", node_id.0),
                                },
                                format!(
                                    "Necessarily true (via causal chain): {}",
                                    chain_node.0
                                ),
                                node.confidence * 0.9,
                                chain.len(),
                                vec![
                                    format!("Root of chain: {} (conf={:.2})", node_id.0, node.confidence),
                                    format!("Chain length: {}", chain.len()),
                                    format!("Constraint type: causation"),
                                ],
                            ));
                        }
                    }
                }
            }
        }

        predictions
    }

    /// Trace causal chain forward from a node
    fn trace_causal_chain(&self, start: &NodeId) -> Vec<NodeId> {
        let mut chain = vec![start.clone()];
        let mut current = start.clone();
        let mut visited: HashSet<NodeId> = HashSet::new();
        visited.insert(start.clone());

        // Follow causal constraints
        for _ in 0..10 { // Max chain length
            let next = self.constraints.iter()
                .filter(|constraint| {
                    constraint.satisfied
                        && constraint.constraint_type == ConstraintType::Causation
                        && constraint.from == current
                })
                .filter_map(|c| Some(c.to.clone()))
                .find(|n| !visited.contains(n));

            if let Some(next_node) = next {
                visited.insert(next_node.clone());
                chain.push(next_node.clone());
                current = next_node;
            } else {
                break;
            }
        }

        chain
    }

    /// Get current node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get current constraint count
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    /// Clone nodes (for counterfactual)
    pub fn clone_nodes(&self) -> Vec<(String, PropertyValue, f64)> {
        self.nodes.iter()
            .map(|(id, node)| (id.0.clone(), node.value.clone(), node.confidence))
            .collect()
    }
}

impl Default for BasinEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Pressure on a basin
#[derive(Debug, Clone)]
struct BasinPressure {
    value: PropertyValue,
    force: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut engine = BasinEngine::new();
        
        engine.add_node("fire", PropertyValue::String("heat".to_string()), 0.8);
        
        assert_eq!(engine.node_count(), 1);
    }

    #[test]
    fn test_add_causal() {
        let mut engine = BasinEngine::new();
        
        engine.add_node("fire", PropertyValue::String("heat".to_string()), 0.8);
        engine.add_node("heat", PropertyValue::String("energy".to_string()), 0.7);
        engine.add_causal("fire", "heat", 0.9);
        
        assert_eq!(engine.constraint_count(), 1);
    }

    #[test]
    fn test_predict_equilibrium() {
        let mut engine = BasinEngine::new();
        
        engine.add_node("fire", PropertyValue::String("heat".to_string()), 0.8);
        engine.add_node("heat", PropertyValue::String("energy".to_string()), 0.7);
        engine.add_causal("fire", "heat", 0.9);
        
        let predictions = engine.predict_equilibrium();
        
        assert!(
            !predictions.is_empty(),
            "a causal constraint should produce an equilibrium prediction"
        );
    }

    #[test]
    fn test_causal_chain() {
        let mut engine = BasinEngine::new();
        
        engine.add_node("a", PropertyValue::Boolean(true), 0.9);
        engine.add_node("b", PropertyValue::Boolean(true), 0.8);
        engine.add_node("c", PropertyValue::Boolean(true), 0.7);
        
        engine.add_causal("a", "b", 0.9);
        engine.add_causal("b", "c", 0.9);
        
        let chain = engine.trace_causal_chain(&NodeId::new("a"));
        
        assert!(chain.len() >= 2);
    }
}