use star::autonomy::{
    ActionAuthority, ActionBroker, AutonomousKernel, CognitiveOperator, Goal, OperatorContext,
    OperatorProposal, OperatorRegistry, ResourceBudget,
};
use star::environment::hidden_rule::{
    HiddenRuleAction, HiddenRuleEnvironment, HiddenRuleObservation,
};

struct InspectOperator;

impl CognitiveOperator<HiddenRuleEnvironment> for InspectOperator {
    fn id(&self) -> &str {
        "inspect-hidden-rule"
    }

    fn propose(
        &mut self,
        context: &OperatorContext<'_, HiddenRuleEnvironment>,
    ) -> Option<OperatorProposal<HiddenRuleAction>> {
        if context.observation.clue.is_some()
            || !context.available_actions.contains(&HiddenRuleAction::Inspect)
        {
            return None;
        }

        Some(OperatorProposal {
            action: HiddenRuleAction::Inspect,
            authority: ActionAuthority::Observe,
            rationale: "acquire information before committing to a candidate".into(),
            predicted_effect: "reduce epistemic uncertainty about the hidden rule".into(),
            expected_utility: 3.0,
            requested_discharge: 1.0,
            compute_cost: 1,
            declared_action_cost: 1,
        })
    }
}

struct ApplyClueOperator;

impl CognitiveOperator<HiddenRuleEnvironment> for ApplyClueOperator {
    fn id(&self) -> &str {
        "apply-observed-clue"
    }

    fn propose(
        &mut self,
        context: &OperatorContext<'_, HiddenRuleEnvironment>,
    ) -> Option<OperatorProposal<HiddenRuleAction>> {
        let clue = context.observation.clue?;
        if context.observation.candidate == Some(clue) {
            return None;
        }
        let action = HiddenRuleAction::Set(clue);
        if !context.available_actions.contains(&action) {
            return None;
        }

        Some(OperatorProposal {
            action,
            authority: ActionAuthority::ReversibleSandbox,
            rationale: "apply the independently observed rule clue".into(),
            predicted_effect: "move the candidate state toward the verified target".into(),
            expected_utility: 2.0,
            requested_discharge: 1.0,
            compute_cost: 1,
            declared_action_cost: 1,
        })
    }
}

struct SubmitVerifiedCandidateOperator;

impl CognitiveOperator<HiddenRuleEnvironment> for SubmitVerifiedCandidateOperator {
    fn id(&self) -> &str {
        "submit-verified-candidate"
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
        if submitted || clue.is_none() || candidate != clue {
            return None;
        }
        if !context.available_actions.contains(&HiddenRuleAction::Submit) {
            return None;
        }

        Some(OperatorProposal {
            action: HiddenRuleAction::Submit,
            authority: ActionAuthority::ReversibleSandbox,
            rationale: "submit only after the candidate matches observed evidence".into(),
            predicted_effect: "satisfy the externally verified objective".into(),
            expected_utility: 1.0,
            requested_discharge: 1.0,
            compute_cost: 1,
            declared_action_cost: 1,
        })
    }
}

fn registry() -> OperatorRegistry<HiddenRuleEnvironment> {
    let mut registry = OperatorRegistry::new();
    registry.register(InspectOperator).unwrap();
    registry.register(ApplyClueOperator).unwrap();
    registry.register(SubmitVerifiedCandidateOperator).unwrap();
    registry
}

fn main() {
    const SEEDS: u64 = 64;
    let mut solved = 0_u64;
    let mut total_steps = 0_u64;
    let mut denied_actions = 0_u64;
    let mut accepted_discharge = 0.0_f64;

    for seed in 0..SEEDS {
        let mut kernel = AutonomousKernel::new(
            Goal::external(seed + 1, "infer and satisfy the hidden rule"),
            registry(),
            ActionBroker::sandbox_only(),
            ResourceBudget::new(4, 4, 4),
        );
        let mut environment = HiddenRuleEnvironment::new();
        let report = kernel.run_episode(&mut environment, seed);

        if report.solved {
            solved += 1;
        }
        total_steps += report.usage.steps;
        denied_actions += report.denied_actions;
        accepted_discharge += report
            .records
            .iter()
            .map(|record| record.accepted_discharge as f64)
            .sum::<f64>();
    }

    let report = serde_json::json!({
        "experiment": "A1 bounded autonomous kernel foundation",
        "classification": if solved == SEEDS && denied_actions == 0 { "FOUNDATION_PASS" } else { "FOUNDATION_REJECTED" },
        "claim_boundary": "contract and deterministic closed-loop foundation only; not an AGI or cross-domain transfer result",
        "seeds": SEEDS,
        "solved": solved,
        "solve_rate": solved as f64 / SEEDS as f64,
        "mean_steps": total_steps as f64 / SEEDS as f64,
        "denied_actions": denied_actions,
        "total_independently_accepted_discharge": accepted_discharge,
        "live_chat_wiring": false,
        "automatic_ontology_promotion": false,
        "self_edit_authority": false
    });

    println!("{}", serde_json::to_string_pretty(&report).unwrap());

    assert_eq!(solved, SEEDS);
    assert_eq!(total_steps, SEEDS * 3);
    assert_eq!(denied_actions, 0);
}
