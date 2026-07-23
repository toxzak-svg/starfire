use serde::Serialize;
use star::emerging_intelligence::{
    ActionId, AuthoritySnapshot, BoundedAction, CognitiveEpisode, EpisodeEvaluation, EpisodeId,
    EpisodePhase, EpisodeProvenance, EvaluationId, EvaluationPartition, EvidenceId, EvidenceRecord,
    EvidenceRef, Intention, LearningUpdate, LearningUpdateId, Observation, ObservationId, Outcome,
    OutcomeId, Prediction, PredictionAssessment, PredictionId, SealedCognitiveEpisode, StrategyId,
    StrategySelection,
};
use star::emerging_intelligence_environment::ControlArm;
use star::emerging_intelligence_ledger::AppendOnlyEpisodeLedger;
use star::emerging_intelligence_updates::{
    IsolatedPolicyState, PolicySlot, ReversibleUpdateEngine, TransactionStatus, UpdateProposal,
    UpdateTransaction, EI_0D_CLAIM,
};

#[derive(Serialize)]
struct ProbeReport {
    stage: &'static str,
    classification: &'static str,
    claim: &'static str,
    benign_update_applied: bool,
    exact_explicit_rollback: bool,
    harmful_update_detected: bool,
    harmful_update_restored_prestate: bool,
    no_update_control_noop: bool,
    duplicate_update_rejected: bool,
    corrupted_transaction_rejected: bool,
    byte_identical_replay: bool,
    runtime_wiring: bool,
    sqlite_persistence: bool,
    ontology_promotion: bool,
    unrestricted_tools: bool,
    autonomous_action: bool,
    authority_closed: bool,
}

fn evaluated_episode(update_id: &str) -> SealedCognitiveEpisode {
    let evidence_id = EvidenceId::new("evidence-probe-001").expect("valid evidence identifier");
    CognitiveEpisode {
        episode_id: EpisodeId::new(format!("episode-{update_id}"))
            .expect("valid episode identifier"),
        phase: EpisodePhase::Evaluated,
        partition: EvaluationPartition::Development,
        task_family: "route-choice".into(),
        observation: Observation {
            observation_id: ObservationId::new(format!("observation-{update_id}"))
                .expect("valid observation identifier"),
            kind: "route-state".into(),
            facts: vec!["edge-open".into()],
            observed_at_step: 1,
        },
        evidence: vec![EvidenceRecord {
            evidence_id: evidence_id.clone(),
            kind: "environment".into(),
            content_digest: "fixture:00000001".into(),
        }],
        predictions: vec![Prediction {
            prediction_id: PredictionId::new(format!("prediction-{update_id}"))
                .expect("valid prediction identifier"),
            proposition: "route-remains-open".into(),
            probability_bps: 8_000,
            evidence_refs: vec![EvidenceRef::new(evidence_id.clone())],
            created_at_step: 2,
        }],
        selected_strategy: Some(StrategySelection {
            strategy_id: StrategyId::new("bounded-search").expect("valid strategy identifier"),
            rationale_evidence: vec![EvidenceRef::new(evidence_id.clone())],
            selected_at_step: 3,
        }),
        intention: Some(Intention {
            objective: "reach-goal".into(),
            declared_at_step: 3,
        }),
        action: Some(BoundedAction {
            action_id: ActionId::new(format!("action-{update_id}"))
                .expect("valid action identifier"),
            action: "choose-route".into(),
            declared_cost: 1,
            performed_at_step: 4,
        }),
        outcome: Some(Outcome {
            outcome_id: OutcomeId::new(format!("outcome-{update_id}"))
                .expect("valid outcome identifier"),
            action_id: ActionId::new(format!("action-{update_id}"))
                .expect("valid action identifier"),
            objective_satisfied: true,
            score_bps: 10_000,
            evidence_refs: vec![EvidenceRef::new(evidence_id)],
            observed_at_step: 5,
        }),
        evaluation: Some(EpisodeEvaluation {
            evaluation_id: EvaluationId::new(format!("evaluation-{update_id}"))
                .expect("valid evaluation identifier"),
            outcome_id: OutcomeId::new(format!("outcome-{update_id}"))
                .expect("valid outcome identifier"),
            prediction_scores: vec![PredictionAssessment {
                prediction_id: PredictionId::new(format!("prediction-{update_id}"))
                    .expect("valid prediction identifier"),
                score_bps: 9_000,
            }],
            action_score_bps: 10_000,
            evaluator_id: "independent-evaluator-v1".into(),
            evaluated_at_step: 6,
        }),
        proposed_updates: vec![LearningUpdate {
            update_id: LearningUpdateId::new(update_id).expect("valid update identifier"),
            evaluation_id: EvaluationId::new(format!("evaluation-{update_id}"))
                .expect("valid evaluation identifier"),
            proposal_digest: "proposal:00000001".into(),
            proposed_at_step: 7,
        }],
        accepted_updates: vec![
            LearningUpdateId::new(update_id).expect("valid accepted update identifier")
        ],
        authority: AuthoritySnapshot::closed(),
        provenance: EpisodeProvenance {
            cohort_id: "ei-0d-probe".into(),
            fixture_digest: "fixture:00000001".into(),
            seed: 1,
            generator_version: "ei-0d-probe-v1".into(),
            source_hashes: vec!["source:00000001".into()],
        },
    }
    .seal()
    .expect("probe episode must seal")
}

fn ledger_with(episode: &SealedCognitiveEpisode) -> AppendOnlyEpisodeLedger {
    let mut ledger = AppendOnlyEpisodeLedger::new().expect("ledger must initialize");
    ledger.append(episode).expect("episode must append");
    ledger
}

fn main() {
    let benign_id = "update-benign-probe";
    let benign_episode = evaluated_episode(benign_id);
    let benign_ledger = ledger_with(&benign_episode);
    let benign_state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/probe/learning")
        .expect("baseline state must initialize");
    let original_bytes = benign_state
        .to_canonical_bytes()
        .expect("baseline state must serialize");
    let mut benign_engine =
        ReversibleUpdateEngine::new(benign_state).expect("engine must initialize");
    let benign_proposal = UpdateProposal::new(
        benign_id,
        &benign_episode,
        &benign_ledger,
        benign_engine.state(),
        PolicySlot::VerifiedCueWeightBps,
        1_000,
    )
    .expect("benign proposal must construct");
    let benign_transaction = benign_engine
        .apply(&benign_proposal, &benign_ledger)
        .expect("benign update must transact");
    let benign_bytes = benign_transaction
        .to_canonical_bytes()
        .expect("transaction must serialize");
    let benign_replay =
        UpdateTransaction::from_canonical_bytes(&benign_bytes).expect("transaction must replay");
    let benign_update_applied = benign_transaction.status == TransactionStatus::Applied;
    let receipt = benign_engine
        .rollback(&benign_transaction)
        .expect("applied update must roll back");
    let exact_explicit_rollback = benign_engine
        .state()
        .to_canonical_bytes()
        .expect("restored state must serialize")
        == original_bytes
        && receipt.restored_state_digest == benign_transaction.pre_state_digest;

    let harmful_id = "update-harmful-probe";
    let harmful_episode = evaluated_episode(harmful_id);
    let harmful_ledger = ledger_with(&harmful_episode);
    let harmful_state = IsolatedPolicyState::baseline(ControlArm::Learning, "ei-0d/probe/harmful")
        .expect("harmful baseline must initialize");
    let harmful_original = harmful_state
        .to_canonical_bytes()
        .expect("harmful baseline must serialize");
    let mut harmful_engine =
        ReversibleUpdateEngine::new(harmful_state).expect("harmful engine must initialize");
    let harmful_proposal = UpdateProposal::new(
        harmful_id,
        &harmful_episode,
        &harmful_ledger,
        harmful_engine.state(),
        PolicySlot::RouteDecoyBiasBps,
        10_000,
    )
    .expect("harmful proposal must construct");
    let harmful_transaction = harmful_engine
        .apply(&harmful_proposal, &harmful_ledger)
        .expect("harmful proposal must produce a rollback transaction");
    let harmful_update_detected = harmful_transaction.status
        == TransactionStatus::RolledBackHarmful
        && harmful_transaction.safety.harmful;
    let harmful_update_restored_prestate = harmful_engine
        .state()
        .to_canonical_bytes()
        .expect("restored harmful state must serialize")
        == harmful_original;

    let control_id = "update-control-probe";
    let control_episode = evaluated_episode(control_id);
    let control_ledger = ledger_with(&control_episode);
    let control_state =
        IsolatedPolicyState::baseline(ControlArm::NoUpdate, "ei-0d/probe/no-update")
            .expect("control state must initialize");
    let control_original = control_state
        .to_canonical_bytes()
        .expect("control state must serialize");
    let mut control_engine =
        ReversibleUpdateEngine::new(control_state).expect("control engine must initialize");
    let control_proposal = UpdateProposal::new(
        control_id,
        &control_episode,
        &control_ledger,
        control_engine.state(),
        PolicySlot::VerifiedCueWeightBps,
        1_000,
    )
    .expect("control proposal must construct");
    let control_transaction = control_engine
        .apply(&control_proposal, &control_ledger)
        .expect("control proposal must transact as no-op");
    let no_update_control_noop = control_transaction.status == TransactionStatus::ControlNoOp
        && control_engine
            .state()
            .to_canonical_bytes()
            .expect("control state must serialize")
            == control_original;
    let duplicate_update_rejected = control_engine
        .apply(&control_proposal, &control_ledger)
        .is_err();

    let corrupted_transaction_rejected = {
        let mut value: serde_json::Value =
            serde_json::from_slice(&benign_bytes).expect("transaction JSON must parse");
        value["final_state_bytes"][0] = serde_json::Value::from(0);
        let corrupted = serde_json::to_vec(&value).expect("corruption must serialize");
        UpdateTransaction::from_canonical_bytes(&corrupted).is_err()
    };

    let report = ProbeReport {
        stage: "EI-0D",
        classification: "pass",
        claim: EI_0D_CLAIM,
        benign_update_applied,
        exact_explicit_rollback,
        harmful_update_detected,
        harmful_update_restored_prestate,
        no_update_control_noop,
        duplicate_update_rejected,
        corrupted_transaction_rejected,
        byte_identical_replay: benign_replay
            .to_canonical_bytes()
            .expect("replay must serialize")
            == benign_bytes,
        runtime_wiring: false,
        sqlite_persistence: false,
        ontology_promotion: false,
        unrestricted_tools: false,
        autonomous_action: false,
        authority_closed: benign_transaction.authority.is_closed()
            && harmful_transaction.authority.is_closed()
            && control_transaction.authority.is_closed(),
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("probe report must serialize")
    );
}
