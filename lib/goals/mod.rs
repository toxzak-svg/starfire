//! Goals — Hierarchical Goal Memory with subgoals, planning, and motivation
//!
//! Gives Starfire explicit goals with temporal projection and progress tracking.

pub mod planning;
pub mod tracking;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A goal with subgoals, priority, and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub content: String,
    pub parent: Option<GoalId>,
    pub subgoals: Vec<GoalId>,
    pub priority: f64,
    pub state: GoalState,
    pub created_at: i64,
    pub deadline: Option<i64>,
    pub projected_outcome: String,
}

impl Goal {
    pub fn new(content: impl Into<String>, parent: Option<GoalId>) -> Self {
        Self {
            id: GoalId::new(),
            content: content.into(),
            parent,
            subgoals: Vec::new(),
            priority: 0.5,
            state: GoalState::Active,
            created_at: crate::now_timestamp(),
            deadline: None,
            projected_outcome: String::new(),
        }
    }

    pub fn with_priority(mut self, priority: f64) -> Self {
        self.priority = priority.clamp(0.0, 1.0);
        self
    }

    /// Set priority on a mutable reference (non-consuming)
    pub fn set_priority(&mut self, priority: f64) {
        self.priority = priority.clamp(0.0, 1.0);
    }

    pub fn with_deadline(mut self, deadline: i64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.projected_outcome = outcome.into();
        self
    }

    pub fn add_subgoal(&mut self, subgoal_id: GoalId) {
        self.subgoals.push(subgoal_id);
    }

    pub fn complete(&mut self, conclusion: impl Into<String>) {
        self.state = GoalState::Completed {
            conclusion: conclusion.into(),
        };
    }

    pub fn abandon(&mut self, reason: impl Into<String>) {
        self.state = GoalState::Abandoned {
            reason: reason.into(),
        };
    }

    pub fn suspend(&mut self, reason: impl Into<String>) {
        self.state = GoalState::Suspended {
            reason: reason.into(),
        };
    }
}

/// Goal ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalId(u64);

impl GoalId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for GoalId {
    fn default() -> Self {
        Self::new()
    }
}

/// Goal state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum GoalState {
    Active,
    Suspended { reason: String },
    Completed { conclusion: String },
    Abandoned { reason: String },
}

impl GoalState {
    pub fn is_active(&self) -> bool {
        matches!(self, GoalState::Active)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, GoalState::Completed { .. } | GoalState::Abandoned { .. })
    }
}

/// Goal engine
#[derive(Debug, Clone)]
pub struct GoalEngine {
    goals: HashMap<GoalId, Goal>,
    root_goals: Vec<GoalId>,
    active_goals: Vec<GoalId>,
}

impl Default for GoalEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalEngine {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            root_goals: Vec::new(),
            active_goals: Vec::new(),
        }
    }

    /// Create a new goal
    pub fn create_goal(&mut self, content: impl Into<String>, parent: Option<GoalId>) -> GoalId {
        let parent_clone = parent.clone();
        let goal = Goal::new(content, parent);
        let id = goal.id.clone();

        if parent_clone.is_none() {
            self.root_goals.push(id.clone());
        } else if let Some(pid) = &parent_clone {
            if let Some(parent_goal) = self.goals.get_mut(pid) {
                parent_goal.add_subgoal(id.clone());
            }
        }

        if goal.state.is_active() {
            self.active_goals.push(id.clone());
        }

        self.goals.insert(id.clone(), goal);
        id
    }

    /// Get a goal by ID
    pub fn get(&self, id: &GoalId) -> Option<&Goal> {
        self.goals.get(id)
    }

    /// Get a mutable goal
    pub fn get_mut(&mut self, id: &GoalId) -> Option<&mut Goal> {
        self.goals.get_mut(id)
    }

    /// Complete a goal
    pub fn complete(&mut self, id: &GoalId, conclusion: impl Into<String>) {
        if let Some(goal) = self.goals.get_mut(id) {
            goal.complete(conclusion);
            self.active_goals.retain(|g| g != id);
        }
    }

    /// Abandon a goal
    pub fn abandon(&mut self, id: &GoalId, reason: impl Into<String>) {
        if let Some(goal) = self.goals.get_mut(id) {
            goal.abandon(reason);
            self.active_goals.retain(|g| g != id);
        }
    }

    /// Suspend a goal
    pub fn suspend(&mut self, id: &GoalId, reason: impl Into<String>) {
        if let Some(goal) = self.goals.get_mut(id) {
            goal.suspend(reason);
            self.active_goals.retain(|g| g != id);
        }
    }

    /// Decompose a goal into subgoals
    pub fn decompose(&mut self, id: &GoalId, subgoals: &[(&str, f64)]) -> Vec<GoalId> {
        let mut created = Vec::new();

        for (content, priority) in subgoals {
            let subgoal_id = self.create_goal(
                *content,
                Some(id.clone()),
            );
            if let Some(subgoal) = self.goals.get_mut(&subgoal_id) {
                subgoal.set_priority(*priority);
            }
            created.push(subgoal_id);
        }

        created
    }

    /// Get all active goals sorted by priority
    pub fn active_goals_sorted(&self) -> Vec<&Goal> {
        let mut active: Vec<_> = self.active_goals
            .iter()
            .filter_map(|id| self.goals.get(id))
            .collect();

        active.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });

        active
    }

    /// Get root goals
    pub fn root_goals(&self) -> Vec<&Goal> {
        self.root_goals
            .iter()
            .filter_map(|id| self.goals.get(id))
            .collect()
    }

    /// Get total goal count
    pub fn len(&self) -> usize {
        self.goals.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }

    /// Get goals due soon (within window seconds)
    pub fn goals_due_soon(&self, window_secs: i64) -> Vec<&Goal> {
        let now = crate::now_timestamp();
        self.active_goals
            .iter()
            .filter_map(|id| self.goals.get(id))
            .filter(|g| {
                if let Some(deadline) = g.deadline {
                    deadline - now <= window_secs
                } else {
                    false
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_goal() {
        let mut engine = GoalEngine::new();
        let id = engine.create_goal("Test goal", None);

        let goal = engine.get(&id).unwrap();
        assert_eq!(goal.content, "Test goal");
        assert!(goal.parent.is_none());
    }

    #[test]
    fn test_create_subgoal() {
        let mut engine = GoalEngine::new();
        let parent = engine.create_goal("Parent", None);
        let child = engine.create_goal("Child", Some(parent.clone()));

        let parent_goal = engine.get(&parent).unwrap();
        assert!(parent_goal.subgoals.contains(&child));
    }

    #[test]
    fn test_complete_goal() {
        let mut engine = GoalEngine::new();
        let id = engine.create_goal("Test", None);
        engine.complete(&id, "Done!");

        let goal = engine.get(&id).unwrap();
        assert!(matches!(goal.state, GoalState::Completed { .. }));
    }

    #[test]
    fn test_decompose() {
        let mut engine = GoalEngine::new();
        let parent = engine.create_goal("Build AI", None);
        let subgoals = engine.decompose(&parent, &[
            ("Design architecture", 0.9),
            ("Implement core", 0.8),
            ("Test", 0.7),
        ]);

        assert_eq!(subgoals.len(), 3);
    }

    #[test]
    fn test_priority_sort() {
        let mut engine = GoalEngine::new();
        let low = engine.create_goal("Low", None);
        let high = engine.create_goal("High", None);
        let medium = engine.create_goal("Medium", None);

        // Set priorities
        if let Some(g) = engine.get_mut(&low) { g.priority = 0.3; }
        if let Some(g) = engine.get_mut(&high) { g.priority = 0.9; }
        if let Some(g) = engine.get_mut(&medium) { g.priority = 0.6; }

        let sorted = engine.active_goals_sorted();
        assert_eq!(sorted[0].content, "High");
        assert_eq!(sorted[1].content, "Medium");
        assert_eq!(sorted[2].content, "Low");
    }
}
