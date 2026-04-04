//! Goal Planning — Temporal projection and action planning

use super::{Goal, GoalEngine, GoalId};
use std::collections::HashMap;

/// An action to take toward a goal
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub description: String,
    pub preconditions: Vec<Precondition>,
    pub estimated_cost: f64,
}

impl Action {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            preconditions: Vec::new(),
            estimated_cost: 1.0,
        }
    }

    pub fn with_precondition(mut self, entity: &str, property: &str, expected: &str) -> Self {
        self.preconditions.push(Precondition {
            entity: entity.to_string(),
            property: property.to_string(),
            expected: expected.to_string(),
        });
        self
    }
}

/// A precondition for an action
#[derive(Debug, Clone)]
pub struct Precondition {
    pub entity: String,
    pub property: String,
    pub expected: String,
}

/// Projected outcome of an action
#[derive(Debug, Clone)]
pub struct ProjectedOutcome {
    pub goal_id: GoalId,
    pub action: Action,
    pub success_probability: f64,
    pub projected_state: String,
    pub risks: Vec<String>,
}

/// Goal planner
pub struct GoalPlanner {
    action_templates: HashMap<String, Vec<ActionTemplate>>,
}

impl Default for GoalPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalPlanner {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Generic action templates
        templates.insert("research".to_string(), vec![
            ActionTemplate::new("Search", "Search for information")
                .with_precondition("query", "exists", "true"),
            ActionTemplate::new("Read", "Read and summarize")
                .with_precondition("source", "accessible", "true"),
        ]);

        templates.insert("build".to_string(), vec![
            ActionTemplate::new("Design", "Design the solution")
                .with_precondition("requirements", "defined", "true"),
            ActionTemplate::new("Implement", "Write the code")
                .with_precondition("design", "approved", "true"),
            ActionTemplate::new("Test", "Test the implementation")
                .with_precondition("code", "written", "true"),
        ]);

        templates.insert("learn".to_string(), vec![
            ActionTemplate::new("Find resources", "Locate learning materials")
                .with_precondition("topic", "defined", "true"),
            ActionTemplate::new("Study", "Study the materials")
                .with_precondition("resources", "found", "true"),
            ActionTemplate::new("Practice", "Practice with exercises")
                .with_precondition("knowledge", "acquired", "true"),
        ]);

        Self {
            action_templates: templates,
        }
    }

    /// Generate action candidates for achieving a goal
    pub fn generate_actions(&self, goal: &Goal) -> Vec<Action> {
        let mut actions = Vec::new();

        // Try to match goal content to templates
        let content_lower = goal.content.to_lowercase();

        for (keyword, templates) in &self.action_templates {
            if content_lower.contains(keyword) {
                for template in templates {
                    actions.push(template.to_action());
                }
            }
        }

        // If no match, return generic actions
        if actions.is_empty() {
            actions.push(Action::new("Investigate", format!("Investigate: {}", goal.content)));
            actions.push(Action::new("Research", format!("Research: {}", goal.content)));
        }

        actions
    }

    /// Project outcome of an action for a goal
    pub fn project_outcome(&self, goal: &Goal, action: &Action) -> ProjectedOutcome {
        let mut risks = Vec::new();

        // Estimate success probability based on preconditions
        let mut satisfied = 0;
        for pre in &action.preconditions {
            // In a real system, we'd check world state
            // For now, assume 50% chance per precondition
            if rand::random::<bool>() {
                satisfied += 1;
            } else {
                risks.push(format!("Precondition '{}' may not hold", pre.property));
            }
        }

        let success_prob = if action.preconditions.is_empty() {
            0.7
        } else {
            satisfied as f64 / action.preconditions.len() as f64 * 0.8
        };

        ProjectedOutcome {
            goal_id: goal.id.clone(),
            action: action.clone(),
            success_probability: success_prob,
            projected_state: format!(
                "After '{}': goal '{}' progress made",
                action.name, goal.content
            ),
            risks,
        }
    }

    /// Generate a plan (sequence of actions) for a goal
    pub fn make_plan(&self, goal: &Goal) -> Vec<Action> {
        let actions = self.generate_actions(goal);
        // Simple: return all actions in order
        // In a real system, would do HTN or STRIPS planning
        actions
    }
}

/// Action template
#[derive(Debug, Clone)]
pub struct ActionTemplate {
    name: String,
    description: String,
    preconditions: Vec<(String, String, String)>, // (entity, property, value)
}

impl ActionTemplate {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            preconditions: Vec::new(),
        }
    }

    pub fn with_precondition(mut self, entity: &str, property: &str, value: &str) -> Self {
        self.preconditions.push((entity.to_string(), property.to_string(), value.to_string()));
        self
    }

    pub fn to_action(&self) -> Action {
        Action {
            name: self.name.clone(),
            description: self.description.clone(),
            preconditions: self.preconditions
                .iter()
                .map(|(e, p, v)| Precondition {
                    entity: e.clone(),
                    property: p.clone(),
                    expected: v.clone(),
                })
                .collect(),
            estimated_cost: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_template() {
        let template = ActionTemplate::new("Test", "Run tests")
            .with_precondition("code", "exists", "true");

        let action = template.to_action();
        assert_eq!(action.name, "Test");
        assert_eq!(action.preconditions.len(), 1);
    }

    #[test]
    fn test_generate_actions() {
        let planner = GoalPlanner::new();
        let goal = super::super::Goal::new("Build a system", None);

        let actions = planner.generate_actions(&goal);
        assert!(!actions.is_empty());
    }
}
