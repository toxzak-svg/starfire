use serde::{Deserialize, Serialize};

use crate::charge::{
    Charge, ChargeKind, ChargeScope, DischargeJudge, ImprovementDirection, OutcomeWitness,
    RelativeImprovementJudge, Resolution,
};
use crate::cognitive_cycle::CognitiveCycleState;
use crate::environment::Environment;

use super::broker::{ActionBroker, ActionEnvelope};
use super::budget::{BudgetUsage, ResourceBudget};
use super::goals::{Goal, GoalStatus};
use super::operators::{OperatorContext, OperatorRegistry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpisodeTermination {
    Solved,
    EnvironmentTerminal,
    BudgetExhausted(String),
    NoPendingPressure,
    NoApplicableOperator,
    AuthorityDenied,
    ActionCostUnderdeclared,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeStepRecord {
    pub step_index: u64,
    pub operator_id: String,
    pub action: String,
    pub before_progress: f64,
    pub after_progress: f64,
    pub requested_discharge: f32,
    pub accepted_discharge: f32,
    pub charge_persistence_after: Option<u32>,
    pub environment_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeReport {
    pub seed: u64,
    pub goal: Goal,
    pub solved: bool,
    pub termination: EpisodeTermination,
    pub usage: BudgetUsage,
    pub denied_actions: u64,
    pub remaining_charge: f32,
    pub records: Vec<EpisodeStepRecord>,
}

/// Shadow-only autonomous executive. It can act only through a typed
/// `Environment`, an `ActionBroker`, and independently judged CHARGE discharge.
pub struct AutonomousKernel<E: Environment> {
    goal: Goal,
    registry: OperatorRegistry<E>,
    broker: ActionBroker,
    budget: ResourceBudget,
    judge: RelativeImprovementJudge,
    cycle: CognitiveCycleState,
    next_charge_id: u64,
}

impl<E: Environment> AutonomousKernel<E> {
    pub fn new(
        goal: Goal,
        registry: OperatorRegistry<E>,
        broker: ActionBroker,
        budget: ResourceBudget,
    ) -> Self {
        Self {
            goal,
            registry,
            broker,
            budget,
            judge: RelativeImprovementJudge,
            cycle: CognitiveCycleState::new(),
            next_charge_id: 1,
        }
    }

    pub fn run_episode(&mut self, environment: &mut E, seed: u64) -> EpisodeReport {
        self.goal.activate();
        self.cycle = CognitiveCycleState::new();

        let denied_before = self.broker.denied_attempts();
        let mut usage = BudgetUsage::default();
        let mut records = Vec::new();
        let mut observation = environment.reset(seed);
        let mut feedback = environment.objective_feedback();

        let termination = loop {
            if feedback.solved {
                self.goal.status = GoalStatus::Achieved;
                break EpisodeTermination::Solved;
            }

            if self.cycle.pending().is_empty() && !self.admit_goal_pressure(feedback.progress) {
                self.goal.status = GoalStatus::Blocked;
                break EpisodeTermination::NoPendingPressure;
            }

            let Some(pressure_index) = self.select_pressure() else {
                self.goal.status = GoalStatus::Blocked;
                break EpisodeTermination::NoPendingPressure;
            };
            let charge = self.cycle.pending()[pressure_index].clone();
            let available_actions = environment.available_actions();
            let context = OperatorContext {
                observation: &observation,
                available_actions: &available_actions,
                objective: &feedback,
                charge: &charge,
                step_index: usage.steps,
            };

            let Some(selected) = self.registry.select(&context) else {
                self.goal.status = GoalStatus::Blocked;
                break EpisodeTermination::NoApplicableOperator;
            };

            let envelope = ActionEnvelope {
                action: selected.proposal.action.clone(),
                authority: selected.proposal.authority,
                rationale: selected.proposal.rationale.clone(),
            };
            if self.broker.authorize(&envelope).is_err() {
                self.goal.status = GoalStatus::Blocked;
                break EpisodeTermination::AuthorityDenied;
            }

            if let Err(error) = usage.reserve(
                self.budget,
                selected.proposal.declared_action_cost,
                selected.proposal.compute_cost,
            ) {
                self.goal.status = GoalStatus::BudgetExhausted;
                break EpisodeTermination::BudgetExhausted(error.to_string());
            }

            let action_debug = format!("{:?}", selected.proposal.action);
            let before = feedback.clone();
            let step = environment.act(&selected.proposal.action);
            if step.action_cost > selected.proposal.declared_action_cost {
                self.goal.status = GoalStatus::Failed;
                break EpisodeTermination::ActionCostUnderdeclared;
            }
            let after = environment.objective_feedback();

            let resolution = Resolution {
                discharged: selected
                    .proposal
                    .requested_discharge
                    .min(charge.magnitude),
                emitted: Vec::new(),
                permitted_decay: 0.0,
                compute_cost: selected.proposal.compute_cost,
            };
            let witness = OutcomeWitness::new(
                "objective_progress",
                before.progress,
                after.progress,
                ImprovementDirection::HigherIsBetter,
                after.evidence.clone(),
            );
            let judged = self.judge.evaluate(&charge, &resolution, &witness);
            let _ = self.cycle.apply_judgment(pressure_index, &judged);

            records.push(EpisodeStepRecord {
                step_index: usage.steps - 1,
                operator_id: selected.operator_id,
                action: action_debug,
                before_progress: before.progress,
                after_progress: after.progress,
                requested_discharge: judged.requested,
                accepted_discharge: judged.accepted,
                charge_persistence_after: self.cycle.pending().first().map(|c| c.persistence),
                environment_evidence: after.evidence.clone(),
            });

            observation = step.observation;
            feedback = after;

            if feedback.solved {
                self.goal.status = GoalStatus::Achieved;
                break EpisodeTermination::Solved;
            }
            if step.terminal {
                self.goal.status = GoalStatus::Failed;
                break EpisodeTermination::EnvironmentTerminal;
            }
        };

        EpisodeReport {
            seed,
            goal: self.goal.clone(),
            solved: feedback.solved,
            termination,
            usage,
            denied_actions: self.broker.denied_attempts().saturating_sub(denied_before),
            remaining_charge: self
                .cycle
                .pending()
                .iter()
                .map(|charge| charge.magnitude)
                .sum(),
            records,
        }
    }

    fn admit_goal_pressure(&mut self, progress: f64) -> bool {
        let residual = (1.0 - progress.clamp(0.0, 1.0)) as f32;
        if residual <= f32::EPSILON {
            return false;
        }

        let mut charge = Charge::new(
            ChargeKind::GoalTension,
            vec![residual],
            residual,
            ChargeScope::Goal(self.goal.description.clone()),
        );
        charge.id = self.next_charge_id;
        self.next_charge_id = self.next_charge_id.saturating_add(1);
        self.cycle.admit_charge(charge)
    }

    fn select_pressure(&self) -> Option<usize> {
        self.cycle
            .pending()
            .iter()
            .enumerate()
            .max_by(|(left_index, left), (right_index, right)| {
                let left_score = left.magnitude * (1.0 + left.persistence as f32);
                let right_score = right.magnitude * (1.0 + right.persistence as f32);
                left_score
                    .total_cmp(&right_score)
                    .then_with(|| right_index.cmp(left_index))
            })
            .map(|(index, _)| index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomy::broker::ActionAuthority;
    use crate::autonomy::operators::{CognitiveOperator, OperatorProposal};
    use crate::environment::hidden_rule::{
        HiddenRuleAction, HiddenRuleEnvironment, HiddenRuleObservation,
    };

    struct EvidenceGuidedOperator;

    impl CognitiveOperator<HiddenRuleEnvironment> for EvidenceGuidedOperator {
        fn id(&self) -> &str {
            "evidence-guided"
        }

        fn propose(
            &mut self,
            context: &OperatorContext<'_, HiddenRuleEnvironment>,
        ) -> Option<OperatorProposal<HiddenRuleAction>> {
            let HiddenRuleObservation {
                clue,
                candidate,
                submitted,
            } = *context.observation;

            let action = if clue.is_none() {
                HiddenRuleAction::Inspect
            } else if candidate != clue {
                HiddenRuleAction::Set(clue.unwrap())
            } else if !submitted {
                HiddenRuleAction::Submit
            } else {
                return None;
            };

            Some(OperatorProposal {
                action,
                authority: ActionAuthority::ReversibleSandbox,
                rationale: "follow independently observed clue".into(),
                predicted_effect: "increase objective progress".into(),
                expected_utility: 1.0,
                requested_discharge: 1.0,
                compute_cost: 1,
                declared_action_cost: 1,
            })
        }
    }

    #[test]
    fn bounded_kernel_solves_both_hidden_rule_variants() {
        for seed in 0..2 {
            let mut registry = OperatorRegistry::new();
            registry.register(EvidenceGuidedOperator).unwrap();
            let mut kernel = AutonomousKernel::new(
                Goal::external(1, "infer and satisfy the hidden rule"),
                registry,
                ActionBroker::sandbox_only(),
                ResourceBudget::new(4, 4, 4),
            );
            let mut environment = HiddenRuleEnvironment::new();
            let report = kernel.run_episode(&mut environment, seed);

            assert!(report.solved, "report: {report:?}");
            assert_eq!(report.termination, EpisodeTermination::Solved);
            assert_eq!(report.usage.steps, 3);
            assert_eq!(report.denied_actions, 0);
            assert!(report.records.iter().all(|step| step.accepted_discharge >= 0.0));
        }
    }
}
