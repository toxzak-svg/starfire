#[cfg(not(feature = "emerging-intelligence-environment"))]
fn main() {
    eprintln!("enable --features emerging-intelligence-environment");
}

#[cfg(feature = "emerging-intelligence-environment")]
mod enabled {
    use serde::Serialize;
    use star::emerging_intelligence::{
        ActionId, AuthoritySnapshot, BoundedAction, CognitiveEpisode, EpisodeContractError,
        EpisodeEvaluation, EpisodeId, EpisodePhase, EpisodeProvenance, EvaluationId,
        EvaluationPartition, EvidenceId, EvidenceRecord, EvidenceRef, Intention, LearningUpdate,
        LearningUpdateId, Observation, ObservationId, Outcome, OutcomeId, Prediction,
        PredictionAssessment, PredictionId, SealedCognitiveEpisode, StrategyId, StrategySelection,
    };
    use star::emerging_intelligence_environment::ControlArm;

    const PREREGISTRATION_ID: &str = "ei-0f-remediation-v1";

    #[derive(Debug, Serialize)]
    struct ProbeReport {
        stage: &'static str,
        preregistration_id: &'static str,
        classification: &'static str,
        original_random_update_digest_rejected: bool,
        normalized_arm_count: usize,
        all_normalized_episodes_sealed: bool,
        exact_canonical_replay: bool,
        original_ei_0f_modified: bool,
        thresholds_changed: bool,
        arms_changed: bool,
        partitions_changed: bool,
        budgets_changed: bool,
        evaluator_boundary_changed: bool,
        runtime_wiring: bool,
        live_learning_authority: bool,
        ontology_promotion_authority: bool,
        unrestricted_tool_authority: bool,
    }

    fn normalized_proposal_digest(update_id: &str) -> String {
        let normalized_update_id = update_id.replace('_', "-");
        format!("preregistered:{normalized_update_id}")
    }

    fn evaluated_episode(
        arm: ControlArm,
        proposal_digest: String,
    ) -> Result<SealedCognitiveEpisode, EpisodeContractError> {
        let arm_name = arm.as_str();
        let evidence_id = EvidenceId::new(format!("evidence-remediation-{arm_name}"))?;
        let prediction_id = PredictionId::new(format!("prediction-remediation-{arm_name}"))?;
        let action_id = ActionId::new(format!("action-remediation-{arm_name}"))?;
        let outcome_id = OutcomeId::new(format!("outcome-remediation-{arm_name}"))?;
        let evaluation_id = EvaluationId::new(format!("evaluation-remediation-{arm_name}"))?;
        let update_id = LearningUpdateId::new(format!("update-{arm_name}-101"))?;

        CognitiveEpisode {
            episode_id: EpisodeId::new(format!("episode-remediation-{arm_name}"))?,
            phase: EpisodePhase::Evaluated,
            partition: EvaluationPartition::Development,
            task_family: "digest-remediation-preflight".into(),
            observation: Observation {
                observation_id: ObservationId::new(format!("observation-remediation-{arm_name}"))?,
                kind: "frozen-preflight-fixture".into(),
                facts: vec!["proposal-digest-required".into()],
                observed_at_step: 1,
            },
            evidence: vec![EvidenceRecord {
                evidence_id: evidence_id.clone(),
                kind: "sealed-preflight".into(),
                content_digest: "fixture:00000001".into(),
            }],
            predictions: vec![Prediction {
                prediction_id: prediction_id.clone(),
                proposition: "proposal-digest-seals-canonically".into(),
                probability_bps: 10_000,
                evidence_refs: vec![EvidenceRef::new(evidence_id.clone())],
                created_at_step: 2,
            }],
            selected_strategy: Some(StrategySelection {
                strategy_id: StrategyId::new("normalize-digest-text-only")?,
                rationale_evidence: vec![EvidenceRef::new(evidence_id.clone())],
                selected_at_step: 3,
            }),
            intention: Some(Intention {
                objective: "prove-five-arm-digest-conformance".into(),
                declared_at_step: 3,
            }),
            action: Some(BoundedAction {
                action_id: action_id.clone(),
                action: "seal-remediation-record".into(),
                declared_cost: 1,
                performed_at_step: 4,
            }),
            outcome: Some(Outcome {
                outcome_id: outcome_id.clone(),
                action_id,
                objective_satisfied: true,
                score_bps: 10_000,
                evidence_refs: vec![EvidenceRef::new(evidence_id)],
                observed_at_step: 5,
            }),
            evaluation: Some(EpisodeEvaluation {
                evaluation_id: evaluation_id.clone(),
                outcome_id,
                prediction_scores: vec![PredictionAssessment {
                    prediction_id,
                    score_bps: 10_000,
                }],
                action_score_bps: 10_000,
                evaluator_id: "ei-0f-r1-preflight-evaluator-v1".into(),
                evaluated_at_step: 6,
            }),
            proposed_updates: vec![LearningUpdate {
                update_id: update_id.clone(),
                evaluation_id,
                proposal_digest,
                proposed_at_step: 7,
            }],
            accepted_updates: vec![update_id],
            authority: AuthoritySnapshot::closed(),
            provenance: EpisodeProvenance {
                cohort_id: PREREGISTRATION_ID.into(),
                fixture_digest: "fixture:00000001".into(),
                seed: 101,
                generator_version: "ei-0f-r1-preflight-v1".into(),
                source_hashes: vec!["source:ei-0f-r1-preflight".into()],
            },
        }
        .seal()
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let random_update_arm = ControlArm::RandomUpdate.as_str();
        let original_random_update_digest = format!("preregistered:update-{random_update_arm}-101");
        let original_random_update_digest_rejected = matches!(
            evaluated_episode(ControlArm::RandomUpdate, original_random_update_digest),
            Err(EpisodeContractError::InvalidDigestText(
                "learning proposal digest"
            ))
        );

        let mut normalized_arm_count = 0_usize;
        let mut all_normalized_episodes_sealed = true;
        let mut exact_canonical_replay = true;

        for arm in ControlArm::ALL {
            let arm_name = arm.as_str();
            let update_id = format!("update-{arm_name}-101");
            let sealed = match evaluated_episode(arm, normalized_proposal_digest(&update_id)) {
                Ok(sealed) => sealed,
                Err(_) => {
                    all_normalized_episodes_sealed = false;
                    continue;
                }
            };
            normalized_arm_count += 1;
            let bytes = sealed.to_canonical_bytes()?;
            let replay = SealedCognitiveEpisode::from_canonical_bytes(&bytes)?;
            exact_canonical_replay &= replay == sealed && replay.to_canonical_bytes()? == bytes;
        }

        let passed = original_random_update_digest_rejected
            && normalized_arm_count == ControlArm::ALL.len()
            && all_normalized_episodes_sealed
            && exact_canonical_replay;

        let report = ProbeReport {
            stage: "EI-0F-R1",
            preregistration_id: PREREGISTRATION_ID,
            classification: if passed { "PASS" } else { "FAIL" },
            original_random_update_digest_rejected,
            normalized_arm_count,
            all_normalized_episodes_sealed,
            exact_canonical_replay,
            original_ei_0f_modified: false,
            thresholds_changed: false,
            arms_changed: false,
            partitions_changed: false,
            budgets_changed: false,
            evaluator_boundary_changed: false,
            runtime_wiring: false,
            live_learning_authority: false,
            ontology_promotion_authority: false,
            unrestricted_tool_authority: false,
        };

        let output = serde_json::to_string_pretty(&report)?;
        println!("{output}");
        if !passed {
            return Err("EI-0F-R1 digest remediation preflight failed".into());
        }
        Ok(())
    }
}

#[cfg(feature = "emerging-intelligence-environment")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    enabled::run()
}
