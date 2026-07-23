use star::emerging_intelligence::EvaluationPartition;
use star::emerging_intelligence_environment::{
    generate_frozen_fixture, ActionTrace, ControlArm, EnvironmentReport, FrozenEnvironmentManifest,
    IndependentEvaluator, MatchedTrialSet, RecordedAction, SealedEnvironmentReport,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    manifest.validate()?;

    let mut evaluations = Vec::new();
    let mut fixture_count = 0usize;
    for partition in &manifest.partitions {
        for seed in &partition.seeds {
            let fixture = generate_frozen_fixture(&manifest, partition.partition, *seed)?;
            let trials = MatchedTrialSet::for_fixture(&fixture)?;
            fixture_count += 1;

            for arm in &trials.arms {
                let trace = ActionTrace {
                    fixture_digest: fixture.digest.clone(),
                    arm: arm.arm,
                    actions: vec![RecordedAction {
                        step: 1,
                        action: fixture.fixture.optimal_action.clone(),
                    }],
                    evidence_reads: 1,
                };
                evaluations.push(IndependentEvaluator::evaluate(&fixture, arm, &trace)?);
            }
        }
    }

    let development = generate_frozen_fixture(&manifest, EvaluationPartition::Development, 101)?;
    let renamed = generate_frozen_fixture(
        &manifest,
        EvaluationPartition::RenamedVocabularyTransfer,
        301,
    )?;
    let structural =
        generate_frozen_fixture(&manifest, EvaluationPartition::StructuralTransfer, 401)?;

    let renamed_relation_preserved =
        development.fixture.relation_fingerprint == renamed.fixture.relation_fingerprint;
    let renamed_structure_preserved =
        development.fixture.structure_fingerprint == renamed.fixture.structure_fingerprint;
    let renamed_surface_changed =
        development.fixture.surface_fingerprint != renamed.fixture.surface_fingerprint;
    let structural_relation_preserved =
        development.fixture.relation_fingerprint == structural.fixture.relation_fingerprint;
    let structural_composition_changed =
        development.fixture.structure_fingerprint != structural.fixture.structure_fingerprint;

    let evaluation_count = evaluations.len();
    let sealed = EnvironmentReport::new(&manifest, evaluations)?.seal()?;
    let canonical = sealed.to_canonical_bytes()?;
    let replay = SealedEnvironmentReport::from_canonical_bytes(&canonical)?;
    let replay_bytes = replay.to_canonical_bytes()?;

    assert_eq!(sealed, replay);
    assert_eq!(canonical, replay_bytes);
    assert!(renamed_relation_preserved);
    assert!(renamed_structure_preserved);
    assert!(renamed_surface_changed);
    assert!(structural_relation_preserved);
    assert!(structural_composition_changed);
    assert_eq!(evaluation_count, fixture_count * ControlArm::ALL.len());

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "classification": "pass",
            "claim": "ei-0b-experiment-infrastructure-only",
            "schema_version": replay.schema_version,
            "manifest_digest": replay.report.manifest_digest.as_str(),
            "report_digest": replay.digest.as_str(),
            "fixture_count": fixture_count,
            "evaluation_count": evaluation_count,
            "partition_count": manifest.partitions.len(),
            "arm_count": ControlArm::ALL.len(),
            "exact_replay": canonical == replay_bytes,
            "renamed_relation_preserved": renamed_relation_preserved,
            "renamed_structure_preserved": renamed_structure_preserved,
            "renamed_surface_changed": renamed_surface_changed,
            "structural_relation_preserved": structural_relation_preserved,
            "structural_composition_changed": structural_composition_changed,
            "matched_budgets": true,
            "isolated_control_state": true,
            "independent_evaluator": true,
            "runtime_wiring": false,
            "persistence_authority": false,
            "learning_application_authority": false,
            "ontology_promotion_authority": false
        }))?
    );

    Ok(())
}
