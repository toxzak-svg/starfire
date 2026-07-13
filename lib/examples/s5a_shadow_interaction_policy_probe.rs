use serde::Serialize;
use star::companion_interaction_policy::{
    DetailLevel, DialogueMode, ExplanationStyle, PolicyContext, PolicyVariant, ShadowPolicyPlanner,
    VocabularyLevel,
};
use star::companion_prediction_ledger::{PredictionLedger, PredictionStatus};
use star::companion_state::{
    ClaimId, ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
struct S5AReport {
    proposal_variants: usize,
    candidate_source_claims: usize,
    candidate_predictions_enrolled: usize,
    candidate_abstentions: usize,
    conflict_predictions_enrolled: usize,
    conflict_abstentions: usize,
    deterministic_planning: bool,
    exact_ledger_replay: bool,
    candidate_policy_expected: bool,
    provenance_complete: bool,
    sensitive_default_exclusion: bool,
    contradiction_abstains: bool,
    controls_present: bool,
    policy_digests_unique: bool,
    all_predictions_pending: bool,
    source_state_unchanged: bool,
    gate_passed: bool,
    runtime_chat_wiring: bool,
    response_text_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    action_authority: bool,
}

fn add_claim(
    state: &mut CompanionState,
    key: &str,
    value: &str,
    confidence_bps: u16,
    sensitivity: Sensitivity,
    at: u64,
) -> ClaimId {
    state
        .record_claim(
            state.version,
            ClaimInput {
                key: key.to_owned(),
                value: value.to_owned(),
                source: ClaimSource::UserStatement,
                confidence_bps,
                sensitivity,
                retention: Retention::Session,
                observed_at_ms: at,
            },
        )
        .expect("frozen fixture claim must be valid")
        .claim_id
        .expect("recorded claim must have an id")
}

fn context(subject_scope_digest: u64) -> PolicyContext {
    PolicyContext {
        context_digest: 0x51a5_0001,
        subject_scope_digest,
        domain: Some("rust".to_owned()),
        technical_context: true,
        asks_for_explanation: true,
        emotional_signal: false,
        issued_at_ms: 10_000,
        not_before_ms: 10_100,
        expires_at_ms: 20_000,
    }
}

fn populated_state() -> CompanionState {
    let mut state = CompanionState::new();
    add_claim(
        &mut state,
        "preference.detail.general",
        "yes",
        9_000,
        Sensitivity::Personal,
        100,
    );
    add_claim(
        &mut state,
        "preference.questions.general",
        "yes",
        8_500,
        Sensitivity::Personal,
        200,
    );
    add_claim(
        &mut state,
        "preference.argument_style.general",
        "concrete",
        8_000,
        Sensitivity::Personal,
        300,
    );
    add_claim(
        &mut state,
        "knowledge.strong_domain.rust",
        "rust",
        9_500,
        Sensitivity::Personal,
        400,
    );
    state
}

fn main() {
    let planner = ShadowPolicyPlanner::default();
    let state = populated_state();
    let before = state.clone();
    let first = planner
        .plan(&state, context(0x1111))
        .expect("candidate plan must succeed");
    let second = planner
        .plan(&state, context(0x1111))
        .expect("repeated candidate plan must succeed");
    let deterministic_planning = first == second;

    let candidate = first
        .proposal(PolicyVariant::CompanionDerived)
        .expect("candidate arm must exist");
    let candidate_policy_expected = !candidate.is_abstention()
        && candidate.policy.detail == DetailLevel::Detailed
        && candidate.policy.dialogue == DialogueMode::QuestionLed
        && candidate.policy.explanation_style == ExplanationStyle::Concrete
        && candidate.policy.vocabulary == VocabularyLevel::Technical;
    let provenance_complete = candidate.source_claim_ids() == vec![1, 2, 3, 4];
    let controls_present = PolicyVariant::all()
        .into_iter()
        .all(|variant| first.proposal(variant).is_some());
    let policy_digests_unique = first
        .proposals
        .iter()
        .map(|proposal| proposal.policy_digest_fnv1a64)
        .collect::<BTreeSet<_>>()
        .len()
        == first.proposals.len();

    let mut ledger = PredictionLedger::new();
    let enrollment = planner
        .enroll(&state, &mut ledger, 0, context(0x1111))
        .expect("candidate enrollment must succeed");
    let events = enrollment
        .transitions
        .iter()
        .map(|transition| transition.event.clone())
        .collect::<Vec<_>>();
    let exact_ledger_replay = PredictionLedger::replay(&events)
        .is_ok_and(|replayed| replayed == ledger);
    let all_predictions_pending = ledger.predictions().values().all(|prediction| {
        matches!(prediction.status, PredictionStatus::Pending)
            && prediction.subject_scope.starts_with("s5a/")
    });

    let mut conflict_state = state.clone();
    add_claim(
        &mut conflict_state,
        "preference.brevity.general",
        "yes",
        9_200,
        Sensitivity::Personal,
        500,
    );
    let conflict_plan = planner
        .plan(&conflict_state, context(0x2222))
        .expect("conflict plan must be representable");
    let contradiction_abstains = conflict_plan
        .proposal(PolicyVariant::CompanionDerived)
        .is_some_and(|proposal| {
            proposal.is_abstention()
                && proposal
                    .abstention_reason
                    .as_deref()
                    .is_some_and(|reason| reason.contains("detail and brevity"))
        });
    let mut conflict_ledger = PredictionLedger::new();
    let conflict_enrollment = planner
        .enroll(&conflict_state, &mut conflict_ledger, 0, context(0x2222))
        .expect("conflict enrollment must succeed through abstention");

    let mut sensitive_state = CompanionState::new();
    add_claim(
        &mut sensitive_state,
        "preference.detail.general",
        "yes",
        10_000,
        Sensitivity::Sensitive,
        100,
    );
    let sensitive_default_exclusion = planner
        .plan(&sensitive_state, context(0x3333))
        .expect("sensitive exclusion plan must succeed")
        .proposal(PolicyVariant::CompanionDerived)
        .is_some_and(|proposal| proposal.is_abstention() && proposal.evidence.is_empty());

    let source_state_unchanged = state == before;
    let gate_passed = first.proposals.len() == PolicyVariant::all().len()
        && deterministic_planning
        && exact_ledger_replay
        && candidate_policy_expected
        && provenance_complete
        && sensitive_default_exclusion
        && contradiction_abstains
        && controls_present
        && policy_digests_unique
        && all_predictions_pending
        && source_state_unchanged
        && enrollment.prediction_ids.len() == 6
        && enrollment.abstention_ids.is_empty()
        && conflict_enrollment.prediction_ids.len() == 5
        && conflict_enrollment.abstention_ids.len() == 1;

    let report = S5AReport {
        proposal_variants: first.proposals.len(),
        candidate_source_claims: candidate.evidence.len(),
        candidate_predictions_enrolled: enrollment.prediction_ids.len(),
        candidate_abstentions: enrollment.abstention_ids.len(),
        conflict_predictions_enrolled: conflict_enrollment.prediction_ids.len(),
        conflict_abstentions: conflict_enrollment.abstention_ids.len(),
        deterministic_planning,
        exact_ledger_replay,
        candidate_policy_expected,
        provenance_complete,
        sensitive_default_exclusion,
        contradiction_abstains,
        controls_present,
        policy_digests_unique,
        all_predictions_pending,
        source_state_unchanged,
        gate_passed,
        runtime_chat_wiring: false,
        response_text_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("S5-A report must serialize")
    );
    assert!(gate_passed, "S5-A shadow interaction-policy gate failed");
}
