use star::emerging_intelligence::{
    ActionId, AuthoritySnapshot, BoundedAction, CognitiveEpisode, EpisodeEvaluation, EpisodeId,
    EpisodePhase, EpisodeProvenance, EvaluationId, EvaluationPartition, EvidenceId,
    EvidenceRecord, EvidenceRef, Intention, LearningUpdate, LearningUpdateId, Observation,
    ObservationId, Outcome, OutcomeId, Prediction, PredictionAssessment, PredictionId,
    SealedCognitiveEpisode, StrategyId, StrategySelection, EI_0A_SCHEMA_VERSION,
};

fn evidence_id(value: &str) -> EvidenceId {
    EvidenceId::new(value).expect("probe evidence identifier must be valid")
}

fn evidence_ref(value: &str) -> EvidenceRef {
    EvidenceRef::new(evidence_id(value))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let episode = CognitiveEpisode {
        episode_id: EpisodeId::new("ei-0a-probe-episode-001")?,
        phase: EpisodePhase::Evaluated,
        partition: EvaluationPartition::Development,
        task_family: "bounded-route-choice".into(),
        observation: Observation {
            observation_id: ObservationId::new("ei-0a-probe-observation-001")?,
            kind: "route-state".into(),
            facts: vec!["edge:a-b:open".into(), "node:a:active".into()],
            observed_at_step: 1,
        },
        evidence: vec![
            EvidenceRecord {
                evidence_id: evidence_id("ei-0a-probe-evidence-environment-001"),
                kind: "environment".into(),
                content_digest: "fixture:probeenvironment001".into(),
            },
            EvidenceRecord {
                evidence_id: evidence_id("ei-0a-probe-evidence-memory-001"),
                kind: "memory".into(),
                content_digest: "fixture:probememory000001".into(),
            },
        ],
        predictions: vec![Prediction {
            prediction_id: PredictionId::new("ei-0a-probe-prediction-001")?,
            proposition: "route remains open".into(),
            probability_bps: 7_500,
            evidence_refs: vec![evidence_ref("ei-0a-probe-evidence-memory-001")],
            created_at_step: 2,
        }],
        selected_strategy: Some(StrategySelection {
            strategy_id: StrategyId::new("ei-0a-probe-strategy-001")?,
            rationale_evidence: vec![evidence_ref("ei-0a-probe-evidence-memory-001")],
            selected_at_step: 3,
        }),
        intention: Some(Intention {
            objective: "reach goal".into(),
            declared_at_step: 3,
        }),
        action: Some(BoundedAction {
            action_id: ActionId::new("ei-0a-probe-action-001")?,
            action: "move:a-b".into(),
            declared_cost: 1,
            performed_at_step: 4,
        }),
        outcome: Some(Outcome {
            outcome_id: OutcomeId::new("ei-0a-probe-outcome-001")?,
            action_id: ActionId::new("ei-0a-probe-action-001")?,
            objective_satisfied: true,
            score_bps: 10_000,
            evidence_refs: vec![evidence_ref("ei-0a-probe-evidence-environment-001")],
            observed_at_step: 5,
        }),
        evaluation: Some(EpisodeEvaluation {
            evaluation_id: EvaluationId::new("ei-0a-probe-evaluation-001")?,
            outcome_id: OutcomeId::new("ei-0a-probe-outcome-001")?,
            prediction_scores: vec![PredictionAssessment {
                prediction_id: PredictionId::new("ei-0a-probe-prediction-001")?,
                score_bps: 9_000,
            }],
            action_score_bps: 10_000,
            evaluator_id: "ei-0a-independent-probe-v1".into(),
            evaluated_at_step: 6,
        }),
        proposed_updates: vec![LearningUpdate {
            update_id: LearningUpdateId::new("ei-0a-probe-update-001")?,
            evaluation_id: EvaluationId::new("ei-0a-probe-evaluation-001")?,
            proposal_digest: "proposal:probe00000001".into(),
            proposed_at_step: 7,
        }],
        accepted_updates: vec![LearningUpdateId::new("ei-0a-probe-update-001")?],
        authority: AuthoritySnapshot::closed(),
        provenance: EpisodeProvenance {
            cohort_id: "ei-0a-probe-development-v1".into(),
            fixture_digest: "fixture:ei0aprobe0001".into(),
            seed: 7,
            generator_version: "ei-0a-contract-probe-v1".into(),
            source_hashes: vec!["source:probe00000001".into(), "source:probe00000002".into()],
        },
    };

    let sealed = episode.seal()?;
    let canonical = sealed.to_canonical_bytes()?;
    let replay = SealedCognitiveEpisode::from_canonical_bytes(&canonical)?;

    assert_eq!(sealed, replay);
    assert_eq!(canonical, replay.to_canonical_bytes()?);
    assert!(replay.episode.authority.is_closed());

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "classification": "pass",
            "schema_version": EI_0A_SCHEMA_VERSION,
            "digest": replay.digest.as_str(),
            "canonical_bytes": canonical.len(),
            "exact_replay": true,
            "authority_closed": true,
            "runtime_wiring": false,
            "learning_application_authority": false
        }))?
    );

    Ok(())
}
