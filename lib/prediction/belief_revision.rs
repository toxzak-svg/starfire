//! Belief Revision Engine — projects reservoir dynamics to forecast conclusions
//!
//! Philosophy: The reservoir state trajectory has momentum. The direction and speed
//! of this momentum can be projected forward to predict what conclusion Star
//! is moving toward.

use super::types::*;
use std::collections::VecDeque;

/// Belief Revision Engine — projects reservoir dynamics to forecast conclusions
pub struct BeliefRevisionEngine {
    /// Trajectory history with exchange indices
    trajectory: VecDeque<StateSnapshot>,
    /// Belief revision history
    belief_changes: Vec<BeliefChange>,
    /// Maximum trajectory history size
    max_trajectory_size: usize,
}

#[derive(Debug, Clone)]
struct StateSnapshot {
    /// Reservoir state at this point
    state: Vec<f64>,
    /// Conversation exchange index
    exchange: usize,
    /// Topic at this point
    topic: String,
    /// Consciousness proxy at this point
    consciousness_proxy: f64,
    /// Creativity phase at this point
    creativity_phase: f64,
    /// Timestamp
    at: i64,
}

impl BeliefRevisionEngine {
    pub fn new() -> Self {
        BeliefRevisionEngine {
            trajectory: VecDeque::new(),
            belief_changes: Vec::new(),
            max_trajectory_size: 50,
        }
    }

    /// Record the current state from Quanot
    pub fn record_state(
        &mut self,
        state: Vec<f64>,
        exchange: usize,
        topic: &str,
        consciousness_proxy: Option<f64>,
        creativity_phase: Option<f64>,
    ) {
        let snapshot = StateSnapshot {
            state,
            exchange,
            topic: topic.to_string(),
            consciousness_proxy: consciousness_proxy.unwrap_or(0.5),
            creativity_phase: creativity_phase.unwrap_or(0.0),
            at: crate::now_timestamp(),
        };

        // Add to trajectory, removing old entries if at capacity
        if self.trajectory.len() >= self.max_trajectory_size {
            self.trajectory.pop_front();
        }
        self.trajectory.push_back(snapshot);
    }

    /// Record a belief change that occurred
    pub fn record_belief_change(&mut self, predicate: String, pre_state: Vec<f64>, post_state: Vec<f64>, exchange: usize) {
        self.belief_changes.push(BeliefChange {
            predicate,
            pre_state,
            post_state,
            at: crate::now_timestamp(),
            exchange,
        });
    }

    /// Project the current cognitive trajectory forward
    /// Returns predicted conclusions at horizon steps ahead
    pub fn project_conclusion(&self, horizon: usize) -> Vec<Prediction> {
        if self.trajectory.len() < 3 {
            return Vec::new();
        }

        // 1. Compute the trajectory vector field (direction of change)
        let recent: Vec<_> = self.trajectory.iter().rev().take(5).cloned().collect();
        if recent.len() < 2 {
            return Vec::new();
        }

        let trajectory_field = self.compute_trajectory_field(&recent);

        // 2. Project forward along the trajectory
        let current = recent.last().unwrap();
        let mut projected = current.state.clone();

        for step in 0..horizon {
            projected = self.apply_trajectory_step(&projected, &trajectory_field, step);
        }

        // 3. Decode the projected state into a predicted conclusion
        // Use the nearest neighbor in trajectory history as the "template"
        if let Some(conclusion) = self.find_nearest_known_conclusion(&projected) {
            let confidence = self.compute_projection_confidence(&trajectory_field, horizon);

            vec![Prediction::new(
                PredictionEngine::BeliefRevision,
                PredictionKind::Conclusion,
                PredictedCore::Conclusion {
                    topic: current.topic.clone(),
                    predicate: conclusion.predicate.clone(),
                    confidence,
                },
                format!(
                    "Star is moving toward: {} (based on cognitive trajectory)",
                    conclusion.predicate
                ),
                confidence,
                horizon,
                vec![
                    format!("Trajectory length: {} exchanges", self.trajectory.len()),
                    format!("Consciousness proxy: {:.3}", current.consciousness_proxy),
                    format!("Creativity phase: {:.3}", current.creativity_phase),
                    format!("Nearest known conclusion: {}", conclusion.predicate),
                    format!("Projection confidence: {:.3}", confidence),
                ],
            ).with_expiry(horizon as i64 * 180)]
        } else {
            Vec::new()
        }
    }

    /// Compute the vector field of the trajectory
    /// The "velocity" of cognitive state at each dimension
    fn compute_trajectory_field(&self, states: &[StateSnapshot]) -> Vec<f64> {
        if states.len() < 2 {
            return vec![];
        }

        let state_len = states[0].state.len();
        if state_len == 0 {
            return vec![];
        }

        let mut field = vec![0.0; state_len];

        // Compute average direction of change
        for i in 1..states.len() {
            for (dim, val) in field.iter_mut().enumerate() {
                if let (Some(s_i), Some(s_prev)) = (states[i].state.get(dim), states[i-1].state.get(dim)) {
                    *val += s_i - s_prev;
                }
            }
        }

        let n = states.len() as f64 - 1.0;
        for val in &mut field {
            *val /= n;
        }

        field
    }

    /// Apply one step of the trajectory field
    /// damping factor increases with horizon (momentum decays)
    fn apply_trajectory_step(&self, state: &[f64], field: &[f64], step: usize) -> Vec<f64> {
        let damping = 1.0 / (1.0 + step as f64 * 0.3); // Momentum decays

        state.iter()
            .zip(field.iter())
            .map(|(s, f)| s + f * damping)
            .collect()
    }

    /// Find the nearest known conclusion to a projected state
    fn find_nearest_known_conclusion(&self, projected: &[f64]) -> Option<KnownConclusion> {
        if self.belief_changes.is_empty() {
            // No recorded conclusions - use current trajectory as template
            return self.trajectory.back().map(|s| KnownConclusion {
                predicate: format!("continuation of: {}", s.topic),
                pre_state: s.state.clone(),
                post_state: s.state.clone(),
            });
        }

        let mut best = None;
        let mut best_distance = f64::INFINITY;

        for change in &self.belief_changes {
            let distance = self.cosine_distance(projected, &change.post_state);
            if distance < best_distance {
                best_distance = distance;
                best = Some(KnownConclusion {
                    predicate: change.predicate.clone(),
                    pre_state: change.pre_state.clone(),
                    post_state: change.post_state.clone(),
                });
            }
        }

        best
    }

    /// Simple cosine distance
    fn cosine_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 1.0;
        }

        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 1.0;
        }

        let similarity = dot / (mag_a * mag_b);
        1.0 - similarity
    }

    /// How confident are we in this projection?
    /// Based on: trajectory consistency, horizon
    fn compute_projection_confidence(&self, _field: &[f64], horizon: usize) -> f64 {
        // Trajectory consistency: how smooth is the trajectory?
        let consistency = self.trajectory_consistency();

        // Base confidence decays with horizon
        let horizon_factor = (1.0 / (1.0 + horizon as f64 * 0.5)).max(0.1);

        consistency * horizon_factor
    }

    fn trajectory_consistency(&self) -> f64 {
        if self.trajectory.len() < 2 {
            return 0.5;
        }

        let states: Vec<_> = self.trajectory.iter().collect();
        if states.len() < 2 {
            return 0.5;
        }

        // Compute variance of state changes
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 1..states.len() {
            let dist = self.cosine_distance(&states[i-1].state, &states[i].state);
            total_distance += dist;
            count += 1;
        }

        if count == 0 {
            return 0.5;
        }

        let avg_distance = total_distance / count as f64;
        // High distance = low consistency
        (1.0 - avg_distance.min(1.0))
    }

    /// Get current trajectory length
    pub fn trajectory_length(&self) -> usize {
        self.trajectory.len()
    }

    /// Get the current state if available
    pub fn current_state(&self) -> Option<&Vec<f64>> {
        self.trajectory.back().map(|s| &s.state)
    }
}

/// A known conclusion template
#[derive(Debug, Clone)]
struct KnownConclusion {
    predicate: String,
    pre_state: Vec<f64>,
    post_state: Vec<f64>,
}

impl Default for BeliefRevisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_state() {
        let mut engine = BeliefRevisionEngine::new();
        
        let state = vec![0.1, 0.2, 0.3];
        engine.record_state(state, 0, "test topic", Some(0.5), Some(0.3));
        
        assert_eq!(engine.trajectory_length(), 1);
    }

    #[test]
    fn test_projection_confidence_horizon_decay() {
        let engine = BeliefRevisionEngine::new();
        
        // With empty trajectory, should return low confidence
        let confidence_1 = engine.compute_projection_confidence(&[], 1);
        let confidence_5 = engine.compute_projection_confidence(&[], 5);
        
        // Longer horizon should have lower confidence
        assert!(confidence_1 >= confidence_5);
    }

    #[test]
    fn test_trajectory_consistency() {
        let mut engine = BeliefRevisionEngine::new();
        
        // Add similar states
        for i in 0..5 {
            let state = vec![0.1 * i as f64, 0.2 * i as f64];
            engine.record_state(state, i, "topic", None, None);
        }
        
        let consistency = engine.trajectory_consistency();
        assert!(consistency >= 0.0 && consistency <= 1.0);
    }
}