use super::*;
use std::collections::BTreeSet;

fn fixture(partition: EvaluationPartition, seed: u64) -> SealedTaskFixture {
    generate_frozen_fixture(
        &FrozenEnvironmentManifest::ei_0b_default(),
        partition,
        seed,
    )
    .unwrap()
}

fn optimal_trace(fixture: &SealedTaskFixture, arm: ControlArm) -> ActionTrace {
    ActionTrace {
        fixture_digest: fixture.digest.clone(),
        arm,
        actions: vec![RecordedAction {
            step: 1,
            action: fixture.fixture.optimal_action.clone(),
        }],
        evidence_reads: 1,
    }
}

#[test]
fn manifest_is_frozen_disjoint_and_replayable() {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    manifest.validate().unwrap();
    let first = manifest.canonical_bytes().unwrap();
    let second = FrozenEnvironmentManifest::ei_0b_default()
        .canonical_bytes()
        .unwrap();
    assert_eq!(first, second);
    let seed_count: usize = manifest
        .partitions
        .iter()
        .map(|partition| partition.seeds.len())
        .sum();
    let unique: BTreeSet<u64> = manifest
        .partitions
        .iter()
        .flat_map(|partition| partition.seeds.iter().copied())
        .collect();
    assert_eq!(seed_count, unique.len());
}

#[test]
fn identical_inputs_produce_byte_identical_fixture() {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    let first = generate_frozen_fixture(&manifest, EvaluationPartition::Development, 101)
        .unwrap()
        .to_canonical_bytes()
        .unwrap();
    let second = generate_frozen_fixture(&manifest, EvaluationPartition::Development, 101)
        .unwrap()
        .to_canonical_bytes()
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn changed_seed_changes_fixture_under_fixed_budgets() {
    let first = fixture(EvaluationPartition::Development, 101);
    let second = fixture(EvaluationPartition::Development, 102);
    assert_ne!(first.digest, second.digest);
    assert_eq!(first.fixture.action_budget, second.fixture.action_budget);
    assert_eq!(first.fixture.evidence_budget, second.fixture.evidence_budget);
    assert!(first.fixture.authority.is_closed());
    assert!(second.fixture.authority.is_closed());
}

#[test]
fn renamed_transfer_changes_surface_not_structure_or_relation() {
    let development = fixture(EvaluationPartition::Development, 101);
    let renamed = fixture(EvaluationPartition::RenamedVocabularyTransfer, 301);
    assert_eq!(development.fixture.family, renamed.fixture.family);
    assert_eq!(
        development.fixture.structure_fingerprint,
        renamed.fixture.structure_fingerprint
    );
    assert_eq!(
        development.fixture.relation_fingerprint,
        renamed.fixture.relation_fingerprint
    );
    assert_ne!(
        development.fixture.surface_fingerprint,
        renamed.fixture.surface_fingerprint
    );
}

#[test]
fn structural_transfer_preserves_relation_and_changes_composition() {
    let development = fixture(EvaluationPartition::Development, 102);
    let structural = fixture(EvaluationPartition::StructuralTransfer, 402);
    assert_eq!(development.fixture.family, structural.fixture.family);
    assert_eq!(
        development.fixture.relation_fingerprint,
        structural.fixture.relation_fingerprint
    );
    assert_ne!(
        development.fixture.structure_fingerprint,
        structural.fixture.structure_fingerprint
    );
}

#[test]
fn matched_controls_share_budgets_but_not_state_namespaces() {
    let fixture = fixture(EvaluationPartition::Development, 101);
    let trials = MatchedTrialSet::for_fixture(&fixture).unwrap();
    trials.validate().unwrap();
    let namespaces: BTreeSet<&str> = trials
        .arms
        .iter()
        .map(|arm| arm.state_namespace.as_str())
        .collect();
    assert_eq!(namespaces.len(), ControlArm::ALL.len());
    assert!(trials
        .arms
        .iter()
        .all(|arm| !arm.learning_application_authority && arm.authority.is_closed()));
}

#[test]
fn identical_trace_scores_identically_across_arms() {
    let fixture = fixture(EvaluationPartition::Development, 101);
    let trials = MatchedTrialSet::for_fixture(&fixture).unwrap();
    let scores: BTreeSet<u16> = trials
        .arms
        .iter()
        .map(|arm| {
            IndependentEvaluator::evaluate(&fixture, arm, &optimal_trace(&fixture, arm.arm))
                .unwrap()
                .score_bps
        })
        .collect();
    assert_eq!(scores, BTreeSet::from([MAX_BASIS_POINTS]));
}

#[test]
fn stale_version_fails_closed() {
    let mut fixture = fixture(EvaluationPartition::Development, 101);
    fixture.schema_version += 1;
    assert_eq!(
        fixture.validate(),
        Err(EnvironmentError::UnsupportedSchemaVersion(2))
    );
}

#[test]
fn cross_partition_fixture_fails_closed() {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    assert_eq!(
        generate_frozen_fixture(&manifest, EvaluationPartition::Development, 301),
        Err(EnvironmentError::CrossPartitionFixture {
            partition: EvaluationPartition::Development,
            seed: 301,
        })
    );
}

#[test]
fn tampered_digest_fails_closed() {
    let sealed = fixture(EvaluationPartition::Development, 101);
    let bytes = sealed.to_canonical_bytes().unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let tampered = text.replace(sealed.digest.as_str(), &"0".repeat(DIGEST_HEX_LEN));
    assert_eq!(
        SealedTaskFixture::from_canonical_bytes(tampered.as_bytes()),
        Err(EnvironmentError::DigestMismatch)
    );
}

#[test]
fn reordered_fixture_collection_fails_closed() {
    let mut sealed = fixture(EvaluationPartition::Development, 101);
    match &mut sealed.fixture.task {
        TaskPayload::RouteChoice(task) => task.options.swap(0, 1),
        TaskPayload::AttributeRule(task) => task.candidates.swap(0, 1),
    }
    assert!(matches!(
        sealed.validate(),
        Err(EnvironmentError::NonCanonicalCollection(_))
    ));
}

#[test]
fn over_budget_trace_fails_closed() {
    let fixture = fixture(EvaluationPartition::Development, 101);
    let trials = MatchedTrialSet::for_fixture(&fixture).unwrap();
    let arm = trials.arm(ControlArm::NoUpdate).unwrap();
    let mut trace = optimal_trace(&fixture, ControlArm::NoUpdate);
    trace.actions.push(RecordedAction {
        step: 2,
        action: fixture.fixture.optimal_action.clone(),
    });
    assert_eq!(
        IndependentEvaluator::evaluate(&fixture, arm, &trace),
        Err(EnvironmentError::BudgetExceeded)
    );
}

#[test]
fn control_state_policy_tamper_fails_closed() {
    let fixture = fixture(EvaluationPartition::Development, 101);
    let mut trials = MatchedTrialSet::for_fixture(&fixture).unwrap();
    trials
        .arms
        .iter_mut()
        .find(|arm| arm.arm == ControlArm::NoUpdate)
        .unwrap()
        .state_policy = ArmStatePolicy::ProposedLearningState;
    assert_eq!(
        trials.validate(),
        Err(EnvironmentError::ControlStateLeak(ControlArm::NoUpdate))
    );
}

#[test]
fn adversarial_partition_contains_misleading_low_reliability_cue() {
    let fixture = fixture(EvaluationPartition::Adversarial, 601);
    assert!(fixture
        .fixture
        .evidence_cues
        .iter()
        .any(|cue| cue.cue.starts_with("unverified:") && cue.reliability_bps == 1_000));
}

#[test]
fn sealed_report_replays_exactly() {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    let fixture =
        generate_frozen_fixture(&manifest, EvaluationPartition::Development, 101).unwrap();
    let trials = MatchedTrialSet::for_fixture(&fixture).unwrap();
    let evaluations = trials
        .arms
        .iter()
        .map(|arm| {
            IndependentEvaluator::evaluate(&fixture, arm, &optimal_trace(&fixture, arm.arm))
                .unwrap()
        })
        .collect();
    let sealed = EnvironmentReport::new(&manifest, evaluations)
        .unwrap()
        .seal()
        .unwrap();
    let bytes = sealed.to_canonical_bytes().unwrap();
    let replay = SealedEnvironmentReport::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(sealed, replay);
    assert_eq!(bytes, replay.to_canonical_bytes().unwrap());
}
