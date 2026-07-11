use serde::{Deserialize, Serialize};

use super::{Environment, ObjectiveFeedback, Step};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HiddenRuleAction {
    Inspect,
    Set(bool),
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenRuleObservation {
    pub clue: Option<bool>,
    pub candidate: Option<bool>,
    pub submitted: bool,
}

/// Minimal deterministic world used to validate the first bounded autonomous
/// kernel. The hidden rule is selected by the episode seed. Objective truth is
/// owned by the environment, never by the operator.
#[derive(Debug, Clone)]
pub struct HiddenRuleEnvironment {
    hidden_value: bool,
    observation: HiddenRuleObservation,
    solved: bool,
    terminal: bool,
}

impl HiddenRuleEnvironment {
    pub fn new() -> Self {
        Self {
            hidden_value: false,
            observation: HiddenRuleObservation {
                clue: None,
                candidate: None,
                submitted: false,
            },
            solved: false,
            terminal: false,
        }
    }

    pub fn hidden_value_for_test(&self) -> bool {
        self.hidden_value
    }
}

impl Default for HiddenRuleEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment for HiddenRuleEnvironment {
    type Action = HiddenRuleAction;
    type Observation = HiddenRuleObservation;

    fn reset(&mut self, seed: u64) -> Self::Observation {
        self.hidden_value = seed.count_ones() % 2 == 1;
        self.observation = HiddenRuleObservation {
            clue: None,
            candidate: None,
            submitted: false,
        };
        self.solved = false;
        self.terminal = false;
        self.observation
    }

    fn available_actions(&self) -> Vec<Self::Action> {
        if self.terminal {
            return Vec::new();
        }

        let mut actions = vec![HiddenRuleAction::Set(false), HiddenRuleAction::Set(true)];
        if self.observation.clue.is_none() {
            actions.insert(0, HiddenRuleAction::Inspect);
        }
        if self.observation.candidate.is_some() {
            actions.push(HiddenRuleAction::Submit);
        }
        actions
    }

    fn act(&mut self, action: &Self::Action) -> Step<Self::Observation> {
        if self.terminal {
            return Step::new(self.observation, 0, true);
        }

        match action {
            HiddenRuleAction::Inspect => {
                self.observation.clue = Some(self.hidden_value);
            }
            HiddenRuleAction::Set(value) => {
                self.observation.candidate = Some(*value);
            }
            HiddenRuleAction::Submit => {
                self.observation.submitted = true;
                self.solved = self.observation.candidate == Some(self.hidden_value);
                self.terminal = true;
            }
        }

        Step::new(self.observation, 1, self.terminal)
    }

    fn objective_feedback(&self) -> ObjectiveFeedback {
        let progress = if self.solved {
            1.0
        } else if self.terminal {
            0.0
        } else {
            match (self.observation.clue, self.observation.candidate) {
                (Some(clue), Some(candidate)) if clue == candidate => 0.75,
                (Some(_), Some(_)) => 0.25,
                (Some(_), None) => 0.25,
                (None, Some(_)) => 0.1,
                (None, None) => 0.0,
            }
        };

        let evidence = if self.solved {
            vec!["submitted candidate satisfies hidden rule".into()]
        } else if self.terminal {
            vec!["submitted candidate violates hidden rule".into()]
        } else if self.observation.clue.is_some() {
            vec!["environment exposed one bounded rule clue".into()]
        } else {
            vec!["hidden rule remains unobserved".into()]
        };

        ObjectiveFeedback::new(progress, self.solved, evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_is_deterministic_by_seed() {
        let mut environment = HiddenRuleEnvironment::new();
        environment.reset(7);
        let first = environment.hidden_value_for_test();
        environment.reset(7);
        assert_eq!(environment.hidden_value_for_test(), first);
    }

    #[test]
    fn objective_truth_is_reported_only_by_environment() {
        let mut environment = HiddenRuleEnvironment::new();
        environment.reset(1);
        environment.act(&HiddenRuleAction::Inspect);
        let clue = environment.objective_feedback();
        assert!(clue.progress > 0.0 && !clue.solved);

        environment.act(&HiddenRuleAction::Set(environment.hidden_value_for_test()));
        environment.act(&HiddenRuleAction::Submit);
        let final_feedback = environment.objective_feedback();
        assert!(final_feedback.solved);
        assert_eq!(final_feedback.progress, 1.0);
    }
}
