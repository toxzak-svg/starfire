//! Goal Tracking — Progress monitoring and motivation computation

use super::{Goal, GoalEngine, GoalId};

/// Progress update
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub goal_id: GoalId,
    pub progress: f64,
    pub event: ProgressEvent,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Started,
    SubgoalCompleted { subgoal_id: GoalId },
    ObstacleEncountered { description: String },
    Breakthrough { description: String },
    MilestoneReached { milestone: String },
    Stall { reason: String },
}

/// Goal tracker
pub struct GoalTracker {
    updates: Vec<ProgressUpdate>,
    milestones: std::collections::HashMap<GoalId, Vec<Milestone>>,
}

impl Default for GoalTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalTracker {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            milestones: std::collections::HashMap::new(),
        }
    }

    /// Record progress on a goal
    pub fn record(&mut self, update: ProgressUpdate) {
        self.updates.push(update);
    }

    /// Get progress for a goal (0.0 to 1.0)
    pub fn get_progress(&self, goal_id: &GoalId, engine: &GoalEngine) -> f64 {
        let goal = match engine.get(goal_id) {
            Some(g) => g,
            None => return 0.0,
        };

        if goal.state.is_terminal() {
            return 1.0;
        }

        let updates: Vec<_> = self.updates
            .iter()
            .filter(|u| u.goal_id == *goal_id)
            .collect();

        if updates.is_empty() {
            return 0.0;
        }

        // Progress based on events
        let mut progress = 0.0;

        for update in &updates {
            match update.event {
                ProgressEvent::Started => progress += 0.1,
                ProgressEvent::SubgoalCompleted { .. } => progress += 0.3,
                ProgressEvent::MilestoneReached { .. } => progress += 0.2,
                ProgressEvent::Breakthrough { .. } => progress += 0.25,
                ProgressEvent::Stall { .. } => progress -= 0.05,
                ProgressEvent::ObstacleEncountered { .. } => progress -= 0.1,
            }
        }

        // Also factor in subgoals completed
        if !goal.subgoals.is_empty() {
            let completed_subgoals = goal.subgoals.iter()
                .filter(|sid| {
                    engine.get(sid)
                        .map(|g| g.state.is_terminal())
                        .unwrap_or(false)
                })
                .count();

            let subgoal_progress = completed_subgoals as f64 / goal.subgoals.len() as f64;
            progress = (progress + subgoal_progress) / 2.0;
        }

        progress.clamp(0.0, 1.0)
    }

    /// Get all updates for a goal
    pub fn get_updates(&self, goal_id: &GoalId) -> Vec<&ProgressUpdate> {
        self.updates
            .iter()
            .filter(|u| u.goal_id == *goal_id)
            .collect()
    }

    /// Get milestone count
    pub fn milestone_count(&self, goal_id: &GoalId) -> usize {
        self.milestones.get(goal_id).map(|m| m.len()).unwrap_or(0)
    }
}

/// A milestone in goal progress
#[derive(Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub reached_at: i64,
    pub description: String,
}

/// Motivation engine
pub struct MotivationEngine {
    curiosity_weight: f64,
    competence_weight: f64,
    autonomy_weight: f64,
}

impl Default for MotivationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MotivationEngine {
    pub fn new() -> Self {
        Self {
            curiosity_weight: 0.4,
            competence_weight: 0.3,
            autonomy_weight: 0.3,
        }
    }

    /// Compute motivation score for a goal
    pub fn motivation_score(&self, goal: &Goal, progress: f64) -> f64 {
        // Intrinsic motivation: curiosity driven by novelty
        let novelty = self.compute_novelty(goal);

        // Competence: motivated by achievable challenges
        let achievability = self.compute_achievability(goal, progress);

        // Autonomy: motivated by self-chosen goals
        let autonomy = if goal.parent.is_none() { 1.0 } else { 0.6 };

        (novelty * self.curiosity_weight
            + achievability * self.competence_weight
            + autonomy * self.autonomy_weight)
            .clamp(0.0, 1.0)
    }

    fn compute_novelty(&self, goal: &Goal) -> f64 {
        // Goals that explore new domains are higher novelty
        let novel_keywords = ["new", "explore", "discover", "create", "design"];
        let content_lower = goal.content.to_lowercase();

        if novel_keywords.iter().any(|k| content_lower.contains(k)) {
            0.8
        } else {
            0.4
        }
    }

    fn compute_achievability(&self, goal: &Goal, progress: f64) -> f64 {
        // Goals that are partially done are more achievable
        // But goals too close to done might lose motivation
        let base = if progress < 0.1 {
            0.5 // Just started
        } else if progress > 0.9 {
            0.9 // Almost done
        } else {
            0.6 // In progress
        };

        // High priority goals are more achievable
        (base + goal.priority) / 2.0
    }

    /// Sort goals by motivation
    pub fn rank_goals(&self, engine: &GoalEngine, tracker: &GoalTracker) -> Vec<(&Goal, f64)> {
        let mut scored: Vec<_> = engine.active_goals_sorted()
            .iter()
            .map(|g| {
                let progress = tracker.get_progress(&g.id, engine);
                let score = self.motivation_score(g, progress);
                (*g, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_update() {
        let update = ProgressUpdate {
            goal_id: GoalId::new(),
            progress: 0.5,
            event: ProgressEvent::Started,
            timestamp: 0,
        };

        match update.event {
            ProgressEvent::Started => {},
            _ => panic!("Expected Started"),
        }
    }

    #[test]
    fn test_motivation_score() {
        let engine = MotivationEngine::new();
        let goal = Goal::new("Explore new architecture", None);

        let score = engine.motivation_score(&goal, 0.3);
        assert!(score > 0.5); // High novelty goal
    }
}
