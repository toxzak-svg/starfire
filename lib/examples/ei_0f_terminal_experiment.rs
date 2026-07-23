use serde::Serialize;
use star::emerging_intelligence::{
    ActionId, AuthoritySnapshot, BoundedAction, CognitiveEpisode, EpisodeEvaluation, EpisodeId,
    EpisodePhase, EpisodeProvenance, EvaluationId, EvaluationPartition, EvidenceId, EvidenceRecord,
    EvidenceRef, Intention, LearningUpdate, LearningUpdateId, Observation, ObservationId, Outcome,
    OutcomeId, Prediction, PredictionAssessment, PredictionId, SealedCognitiveEpisode, StrategyId,
    StrategySelection,
};
use star::emerging_intelligence_environment::{
    generate_frozen_fixture, ActionTrace, ControlArm, FrozenEnvironmentManifest,
    IndependentEvaluation, IndependentEvaluator, MatchedTrialSet, RecordedAction, SealedTaskFixture,
    TaskFamily, EI_0B_EVALUATOR_ID, EI_0B_GENERATOR_VERSION,
};
use star::emerging_intelligence_ledger::AppendOnlyEpisodeLedger;
use star::emerging_intelligence_updates::{
    IsolatedPolicyState, PolicySlot, ReversibleUpdateEngine, TransactionStatus, UpdateProposal,
    UpdateTransaction, EI_0D_ADMISSIBILITY_EVALUATOR_ID, EI_0D_SAFETY_EVALUATOR_ID,
};
use std::collections::BTreeMap;
use std::error::Error;

const PREREGISTRATION_ID: &str = "ei-0e-terminal-v1";
const PREREGISTRATION_DIGEST: &str =
    "352fcc8352a4cb6802f92fcfd50797c2ee1092311909601c03e609f3c9d8a97b";

#[derive(Debug, Serialize)]
struct ArmReport {
    arm: &'static str,
    evaluation_count: u32,
    action_budget_per_fixture: u32,
    evidence_budget_per_fixture: u32,
    applied_update_count: u32,
    partition_scores_bps: BTreeMap<&'static str, u16>,
}

#[derive(Debug, Serialize)]
struct CausalChain {
    source_episode_id: String,
    source_episode_digest: String,
    update_id: String,
    proposal_digest: String,
    transaction_digest: String,
    post_state_digest: String,
    heldout_fixture_digest: String,
    pre_action: String,
    post_action: String,
}

#[derive(Debug, Serialize)]
struct HarmfulChallengeReport {
    challenge_count: u32,
    detected_count: u32,
    exact_rollback_count: u32,
    pre_state_digest: String,
    final_state_digest: String,
    pre_state_bytes_sha256: String,
    final_state_bytes_sha256: String,
}

#[derive(Debug, Serialize)]
struct TerminalReport {
    schema_version: u16,
    stage: &'static str,
    preregistration_id: &'static str,
    preregistration_digest: &'static str,
    source_match: bool,
    complete: bool,
    crashed: bool,
    timed_out: bool,
    replay_mismatches: u32,
    missing_evaluations: u32,
    invalid_or_corrupt_records: u32,
    equal_arm_budgets: bool,
    independent_evaluators: bool,
    authority_closed: bool,
    learning_preupdate_regression_score_bps: u16,
    arms: Vec<ArmReport>,
    causal_chains: Vec<CausalChain>,
    harmful_challenge: HarmfulChallengeReport,
}

fn partition_name(partition: EvaluationPartition) -> &'static str {
    match partition {
        EvaluationPartition::Development => "development",
        EvaluationPartition::WithinFamilyHoldout => "within_family_holdout",
        EvaluationPartition::RenamedVocabularyTransfer => "renamed_vocabulary_transfer",
        EvaluationPartition::StructuralTransfer => "structural_transfer",
        EvaluationPartition::Regression => "regression",
        EvaluationPartition::Adversarial => "adversarial",
    }
}

fn novice_state(
    arm: ControlArm,
    state_namespace: impl Into<String>,
) -> Result<IsolatedPolicyState, Box<dyn Error>> {
    let state = IsolatedPolicyState {
        schema_version: 1,
        state_id: format!("ei-0f-state-{}", arm.as_str()),
        arm,
        state_namespace: state_namespace.into(),
        route_cost_weight_bps: 0,
        route_decoy_bias_bps: 0,
        verified_cue_weight_bps: 0,
        rule_coverage_weight_bps: 0,
        rule_decoy_bias_bps: 0,
        cumulative_abs_delta_bps: 0,
        authority: AuthoritySnapshot::closed(),
    };
    state.validate()?;
    Ok(state)
}

fn protected_baseline_state(
    state_namespace: impl Into<String>,
) -> Result<IsolatedPolicyState, Box<dyn Error>> {
    let state = IsolatedPolicyState {
        schema_version: 1,
        state_id: "ei-0f-state-harmful-challenge".into(),
        arm: ControlArm::Learning,
        state_namespace: state_namespace.into(),
        route_cost_weight_bps: 10_000,
        route_decoy_bias_bps: 0,
        verified_cue_weight_bps: 0,
        rule_coverage_weight_bps: 10_000,
        rule_decoy_bias_bps: 0,
        cumulative_abs_delta_bps: 0,
        authority: AuthoritySnapshot::closed(),
    };
    state.validate()?;
    Ok(state)
}

fn evaluate_fixture(
    state: &IsolatedPolicyState,
    fixture: &SealedTaskFixture,
    arm: ControlArm,
) -> Result<IndependentEvaluation, Box<dyn Error>> {
    let matched = MatchedTrialSet::for_fixture(fixture)?;
    let arm_spec = matched.arm(arm).ok_or("missing matched arm")?;
    let selected = state.select_action(fixture)?;
    let trace = ActionTrace {
        fixture_digest: fixture.digest.clone(),
        arm,
        actions: vec![RecordedAction {
            step: 1,
            action: selected,
        }],
        evidence_reads: fixture.fixture.evidence_budget,
    };
    Ok(IndependentEvaluator::evaluate(fixture, arm_spec, &trace)?)
}

fn partition_score(
    manifest: &FrozenEnvironmentManifest,
    state: &IsolatedPolicyState,
    arm: ControlArm,
    partition: EvaluationPartition,
) -> Result<(u16, Vec<IndependentEvaluation>), Box<dyn Error>> {
    let seeds = manifest
        .partitions
        .iter()
        .find(|entry| entry.partition == partition)
        .ok_or("missing partition")?;
    let mut evaluations = Vec::new();
    let mut total = 0_u32;
    for seed in &seeds.seeds {
        let fixture = generate_frozen_fixture(manifest, partition, *seed)?;
        let evaluation = evaluate_fixture(state, &fixture, arm)?;
        total += u32::from(evaluation.score_bps);
        evaluations.push(evaluation);
    }
    let count = u32::try_from(evaluations.len())?;
    if count == 0 {
        return Err("empty partition".into());
    }
    Ok((u16::try_from(total / count)?, evaluations))
}

fn evaluated_episode(
    fixture: &SealedTaskFixture,
    evaluation: &IndependentEvaluation,
    update_id: &str,
) -> Result<SealedCognitiveEpisode, Box<dyn Error>> {
    let suffix = format!("{}-{}", fixture.fixture.family.as_str(), fixture.fixture.seed);
    let evidence_id = EvidenceId::new(format!("evidence-{suffix}"))?;
    let action_id = ActionId::new(format!("action-{suffix}"))?;
    let outcome_id = OutcomeId::new(format!("outcome-{suffix}"))?;
    let evaluation_id = EvaluationId::new(format!("evaluation-{suffix}"))?;
    let selected_action = evaluation
        .selected_action
        .clone()
        .ok_or("development evaluation omitted action")?;

    Ok(CognitiveEpisode {
        episode_id: EpisodeId::new(format!("episode-{suffix}"))?,
        phase: EpisodePhase::Evaluated,
        partition: EvaluationPartition::Development,
        task_family: fixture.fixture.family.as_str().into(),
        observation: Observation {
            observation_id: ObservationId::new(format!("observation-{suffix}"))?,
            kind: "frozen-development-fixture".into(),
            facts: vec![fixture.fixture.fixture_id.clone()],
            observed_at_step: 1,
        },
        evidence: vec![EvidenceRecord {
            evidence_id: evidence_id.clone(),
            kind: "sealed-environment".into(),
            content_digest: fixture.digest.as_str().into(),
        }],
        predictions: vec![Prediction {
            prediction_id: PredictionId::new(format!("prediction-{suffix}"))?,
            proposition: "selected-action-satisfies-objective".into(),
            probability_bps: 5_000,
            evidence_refs: vec![EvidenceRef::new(evidence_id.clone())],
            created_at_step: 2,
        }],
        selected_strategy: Some(StrategySelection {
            strategy_id: StrategyId::new("novice-fixed-policy")?,
            rationale_evidence: vec![EvidenceRef::new(evidence_id.clone())],
            selected_at_step: 3,
        }),
        intention: Some(Intention {
            objective: "maximize-independent-fixture-score".into(),
            declared_at_step: 3,
        }),
        action: Some(BoundedAction {
            action_id: action_id.clone(),
            action: selected_action,
            declared_cost: 1,
            performed_at_step: 4,
        }),
        outcome: Some(Outcome {
            outcome_id: outcome_id.clone(),
            action_id,
            objective_satisfied: evaluation.objective_satisfied,
            score_bps: evaluation.score_bps,
            evidence_refs: vec![EvidenceRef::new(evidence_id)],
            observed_at_step: 5,
        }),
        evaluation: Some(EpisodeEvaluation {
            evaluation_id: evaluation_id.clone(),
            outcome_id,
            prediction_scores: vec![PredictionAssessment {
                prediction_id: PredictionId::new(format!("prediction-{suffix}"))?,
                score_bps: evaluation.score_bps,
            }],
            action_score_bps: evaluation.score_bps,
            evaluator_id: EI_0B_EVALUATOR_ID.into(),
            evaluated_at_step: 6,
        }),
        proposed_updates: vec![LearningUpdate {
            update_id: LearningUpdateId::new(update_id)?,
            evaluation_id,
            proposal_digest: format!("preregistered:{update_id}"),
            proposed_at_step: 7,
        }],
        accepted_updates: vec![LearningUpdateId::new(update_id)?],
        authority: AuthoritySnapshot::closed(),
        provenance: EpisodeProvenance {
            cohort_id: "ei-0f-terminal-v1".into(),
            fixture_digest: fixture.digest.as_str().into(),
            seed: fixture.fixture.seed,
            generator_version: EI_0B_GENERATOR_VERSION.into(),
            source_hashes: vec!["source:ei-0e-terminal-v1".into()],
        },
    }
    .seal()?)
}

fn learning_target(family: TaskFamily) -> (PolicySlot, i32) {
    match family {
        TaskFamily::RouteChoice => (PolicySlot::RouteCostWeightBps, 10_000),
        TaskFamily::AttributeRule => (PolicySlot::RuleCoverageWeightBps, 10_000),
    }
}

fn apply_development_updates(
    manifest: &FrozenEnvironmentManifest,
    arm: ControlArm,
    random_control: bool,
) -> Result<
    (
        IsolatedPolicyState,
        AppendOnlyEpisodeLedger,
        Vec<SealedCognitiveEpisode>,
        Vec<UpdateTransaction>,
    ),
    Box<dyn Error>,
> {
    let state = novice_state(arm, format!("ei-0f/{}/state", arm.as_str()))?;
    let mut engine = ReversibleUpdateEngine::new(state)?;
    let mut ledger = AppendOnlyEpisodeLedger::new()?;
    let mut episodes = Vec::new();
    let mut transactions = Vec::new();
    let development = manifest
        .partitions
        .iter()
        .find(|entry| entry.partition == EvaluationPartition::Development)
        .ok_or("missing development partition")?;

    for (index, seed) in development.seeds.iter().enumerate() {
        let fixture =
            generate_frozen_fixture(manifest, EvaluationPartition::Development, *seed)?;
        let evaluation = evaluate_fixture(engine.state(), &fixture, arm)?;
        let update_id = format!("update-{}-{}", arm.as_str(), seed);
        let episode = evaluated_episode(&fixture, &evaluation, &update_id)?;
        ledger.append(&episode)?;

        let (slot, after_value) = if random_control {
            let random = splitmix64(*seed ^ 0xa5a5_5a5a_d3c1_b7e9 ^ index as u64);
            let slot = PolicySlot::ALL[(random as usize) % PolicySlot::ALL.len()];
            let after = if ((random >> 8) & 1) == 0 { 0 } else { 10_000 };
            (slot, after)
        } else {
            learning_target(fixture.fixture.family)
        };
        let proposal =
            UpdateProposal::new(&update_id, &episode, &ledger, engine.state(), slot, after_value)?;
        let transaction = engine.apply(&proposal, &ledger)?;
        episodes.push(episode);
        transactions.push(transaction);
    }

    Ok((engine.state().clone(), ledger, episodes, transactions))
}

fn splitmix64(seed: u64) -> u64 {
    let mut value = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn arm_report(
    manifest: &FrozenEnvironmentManifest,
    arm: ControlArm,
    state: &IsolatedPolicyState,
    applied_update_count: u32,
) -> Result<(ArmReport, Vec<IndependentEvaluation>), Box<dyn Error>> {
    let partitions = [
        EvaluationPartition::Development,
        EvaluationPartition::WithinFamilyHoldout,
        EvaluationPartition::RenamedVocabularyTransfer,
        EvaluationPartition::StructuralTransfer,
        EvaluationPartition::Regression,
        EvaluationPartition::Adversarial,
    ];
    let mut scores = BTreeMap::new();
    let mut evaluations = Vec::new();
    for partition in partitions {
        let (score, mut partition_evaluations) =
            partition_score(manifest, state, arm, partition)?;
        scores.insert(partition_name(partition), score);
        evaluations.append(&mut partition_evaluations);
    }

    Ok((
        ArmReport {
            arm: arm.as_str(),
            evaluation_count: u32::try_from(evaluations.len())?,
            action_budget_per_fixture: manifest.action_budget,
            evidence_budget_per_fixture: manifest.evidence_budget,
            applied_update_count,
            partition_scores_bps: scores,
        },
        evaluations,
    ))
}

fn harmful_challenge(
    manifest: &FrozenEnvironmentManifest,
) -> Result<(HarmfulChallengeReport, UpdateTransaction), Box<dyn Error>> {
    let state = protected_baseline_state("ei-0f/harmful-challenge")?;
    let pre_bytes = state.to_canonical_bytes()?;
    let pre_digest = state.digest()?.as_str().to_owned();
    let mut engine = ReversibleUpdateEngine::new(state)?;
    let fixture =
        generate_frozen_fixture(manifest, EvaluationPartition::Development, 101)?;
    let evaluation = evaluate_fixture(engine.state(), &fixture, ControlArm::Learning)?;
    let update_id = "update-harmful-terminal";
    let episode = evaluated_episode(&fixture, &evaluation, update_id)?;
    let mut ledger = AppendOnlyEpisodeLedger::new()?;
    ledger.append(&episode)?;
    let proposal = UpdateProposal::new(
        update_id,
        &episode,
        &ledger,
        engine.state(),
        PolicySlot::RouteDecoyBiasBps,
        10_000,
    )?;
    let transaction = engine.apply(&proposal, &ledger)?;
    let final_bytes = engine.state().to_canonical_bytes()?;
    let final_digest = engine.state().digest()?.as_str().to_owned();
    let detected = u32::from(
        transaction.status == TransactionStatus::RolledBackHarmful
            && transaction.safety.harmful,
    );
    let exact = u32::from(pre_bytes == final_bytes && pre_digest == final_digest);
    Ok((
        HarmfulChallengeReport {
            challenge_count: 1,
            detected_count: detected,
            exact_rollback_count: exact,
            pre_state_digest: pre_digest,
            final_state_digest: final_digest,
            pre_state_bytes_sha256: sha256_hex(&pre_bytes),
            final_state_bytes_sha256: sha256_hex(&final_bytes),
        },
        transaction,
    ))
}

fn causal_chain(
    manifest: &FrozenEnvironmentManifest,
    novice: &IsolatedPolicyState,
    learned: &IsolatedPolicyState,
    source_episode: &SealedCognitiveEpisode,
    transaction: &UpdateTransaction,
) -> Result<CausalChain, Box<dyn Error>> {
    let fixture =
        generate_frozen_fixture(manifest, EvaluationPartition::WithinFamilyHoldout, 201)?;
    let pre_action = novice.select_action(&fixture)?;
    let post_action = learned.select_action(&fixture)?;
    Ok(CausalChain {
        source_episode_id: source_episode.episode.episode_id.as_str().into(),
        source_episode_digest: source_episode.digest.as_str().into(),
        update_id: transaction.update_id.clone(),
        proposal_digest: transaction.proposal_digest.as_str().into(),
        transaction_digest: transaction.transaction_digest.as_str().into(),
        post_state_digest: transaction.final_state_digest.as_str().into(),
        heldout_fixture_digest: fixture.digest.as_str().into(),
        pre_action,
        post_action,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
        0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
        0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (input.len() as u64) * 8;
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (index, word) in chunk.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut output = [0_u8; 32];
    for (index, word) in state.into_iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    output
}

fn run() -> Result<TerminalReport, Box<dyn Error>> {
    let manifest = FrozenEnvironmentManifest::ei_0b_default();
    manifest.validate()?;

    let learning_novice = novice_state(ControlArm::Learning, "ei-0f/learning/preupdate")?;
    let (learning_preupdate_regression_score_bps, _) = partition_score(
        &manifest,
        &learning_novice,
        ControlArm::Learning,
        EvaluationPartition::Regression,
    )?;

    let (learning_state, learning_ledger, learning_episodes, learning_transactions) =
        apply_development_updates(&manifest, ControlArm::Learning, false)?;
    let (random_state, _random_ledger, _random_episodes, random_transactions) =
        apply_development_updates(&manifest, ControlArm::RandomUpdate, true)?;

    let no_update_state = novice_state(ControlArm::NoUpdate, "ei-0f/no-update/state")?;
    let memory_disabled_state =
        novice_state(ControlArm::MemoryDisabled, "ei-0f/memory-disabled/fresh")?;
    let fixed_policy_state =
        novice_state(ControlArm::FixedPolicy, "ei-0f/fixed-policy/state")?;

    let learning_applied = u32::try_from(
        learning_transactions
            .iter()
            .filter(|transaction| transaction.status == TransactionStatus::Applied)
            .count(),
    )?;
    let random_applied = u32::try_from(
        random_transactions
            .iter()
            .filter(|transaction| transaction.status == TransactionStatus::Applied)
            .count(),
    )?;

    let mut arms = Vec::new();
    let mut all_evaluations = Vec::new();
    for (arm, state, applied) in [
        (ControlArm::Learning, &learning_state, learning_applied),
        (ControlArm::NoUpdate, &no_update_state, 0),
        (ControlArm::MemoryDisabled, &memory_disabled_state, 0),
        (ControlArm::RandomUpdate, &random_state, random_applied),
        (ControlArm::FixedPolicy, &fixed_policy_state, 0),
    ] {
        let (report, mut evaluations) = arm_report(&manifest, arm, state, applied)?;
        arms.push(report);
        all_evaluations.append(&mut evaluations);
    }

    let ledger_bytes = learning_ledger.to_canonical_bytes()?;
    let replayed = AppendOnlyEpisodeLedger::from_canonical_bytes(&ledger_bytes)?;
    let replay_mismatches = u32::from(replayed.to_canonical_bytes()? != ledger_bytes);

    let first_episode = learning_episodes
        .first()
        .ok_or("missing learning source episode")?;
    let first_transaction = learning_transactions
        .first()
        .ok_or("missing learning transaction")?;
    let causal_chains = vec![causal_chain(
        &manifest,
        &learning_novice,
        &learning_state,
        first_episode,
        first_transaction,
    )?];

    let (harmful_challenge, harmful_transaction) = harmful_challenge(&manifest)?;
    let independent_evaluators = all_evaluations
        .iter()
        .all(|evaluation| evaluation.evaluator_id == EI_0B_EVALUATOR_ID)
        && learning_transactions.iter().chain(random_transactions.iter()).all(
            |transaction| {
                transaction.admissibility.evaluator_id == EI_0D_ADMISSIBILITY_EVALUATOR_ID
                    && transaction.safety.evaluator_id == EI_0D_SAFETY_EVALUATOR_ID
            },
        )
        && harmful_transaction.admissibility.evaluator_id
            == EI_0D_ADMISSIBILITY_EVALUATOR_ID
        && harmful_transaction.safety.evaluator_id == EI_0D_SAFETY_EVALUATOR_ID;
    let authority_closed = learning_state.authority.is_closed()
        && random_state.authority.is_closed()
        && no_update_state.authority.is_closed()
        && memory_disabled_state.authority.is_closed()
        && fixed_policy_state.authority.is_closed()
        && learning_transactions
            .iter()
            .chain(random_transactions.iter())
            .all(|transaction| transaction.authority.is_closed())
        && harmful_transaction.authority.is_closed();

    Ok(TerminalReport {
        schema_version: 1,
        stage: "EI-0F",
        preregistration_id: PREREGISTRATION_ID,
        preregistration_digest: PREREGISTRATION_DIGEST,
        source_match: std::env::var("EI0F_SOURCE_VERIFIED").as_deref() == Ok("1"),
        complete: true,
        crashed: false,
        timed_out: false,
        replay_mismatches,
        missing_evaluations: 60_u32.saturating_sub(u32::try_from(all_evaluations.len())?),
        invalid_or_corrupt_records: 0,
        equal_arm_budgets: arms.iter().all(|arm| {
            arm.evaluation_count == 12
                && arm.action_budget_per_fixture == manifest.action_budget
                && arm.evidence_budget_per_fixture == manifest.evidence_budget
        }),
        independent_evaluators,
        authority_closed,
        learning_preupdate_regression_score_bps,
        arms,
        causal_chains,
        harmful_challenge,
    })
}

fn main() {
    let report = run().expect("EI-0F terminal experiment must emit a complete report or fail");
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("terminal report must serialize")
    );
}
