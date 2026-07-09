//! Domain-independent interaction contracts for closed cognitive-cycle research.
//!
//! Chat is one possible environment. The cognitive core should be able to use the
//! same observe/predict/act/judge loop in unfamiliar symbolic worlds, bounded
//! filesystems, code tasks, research tasks, and other controlled environments.

use std::fmt::Debug;

/// Objective evidence exposed by an environment independently of a cognitive
/// component's self-reported confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveFeedback {
    /// Normalized objective progress in the inclusive range [0, 1].
    pub progress: f64,
    /// Whether the environment's objective is objectively satisfied.
    pub solved: bool,
    /// Human-readable evidence or verifier notes supporting the feedback.
    pub evidence: Vec<String>,
}

impl ObjectiveFeedback {
    pub fn new(progress: f64, solved: bool, evidence: Vec<String>) -> Self {
        Self {
            progress: normalize_unit(progress),
            solved,
            evidence,
        }
    }
}

/// Result of applying one action to an environment.
#[derive(Debug, Clone, PartialEq)]
pub struct Step<O> {
    pub observation: O,
    /// Declared action cost used for matched-budget experiments.
    pub action_cost: u64,
    /// Whether the environment episode has terminated.
    pub terminal: bool,
}

impl<O> Step<O> {
    pub fn new(observation: O, action_cost: u64, terminal: bool) -> Self {
        Self {
            observation,
            action_cost,
            terminal,
        }
    }
}

/// Minimal interaction boundary for a Starfire environment.
///
/// The environment owns objective truth about its state. A cognitive operator may
/// propose an action, but only the environment can return the resulting
/// observation and objective feedback.
pub trait Environment {
    type Action: Clone + Debug;
    type Observation: Clone + Debug;

    /// Reset to a deterministic episode selected by `seed`.
    fn reset(&mut self, seed: u64) -> Self::Observation;

    /// Enumerate actions currently available to the agent.
    fn available_actions(&self) -> Vec<Self::Action>;

    /// Apply one action and return the resulting observation.
    fn act(&mut self, action: &Self::Action) -> Step<Self::Observation>;

    /// Return independently measured objective progress.
    fn objective_feedback(&self) -> ObjectiveFeedback;
}

fn normalize_unit(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_feedback_clamps_invalid_progress() {
        assert_eq!(ObjectiveFeedback::new(1.5, true, vec![]).progress, 1.0);
        assert_eq!(ObjectiveFeedback::new(-0.5, false, vec![]).progress, 0.0);
        assert_eq!(ObjectiveFeedback::new(f64::NAN, false, vec![]).progress, 0.0);
    }

    #[test]
    fn step_preserves_cost_and_terminal_state() {
        let step = Step::new("state-1", 3, true);
        assert_eq!(step.observation, "state-1");
        assert_eq!(step.action_cost, 3);
        assert!(step.terminal);
    }
}
