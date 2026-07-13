use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::graph_discovery::{
    admit_graph_certificate, infer_graph_rule, validate_graph_rule, FrontierDiscoveryBudget,
    MixedEvidenceGraph, ValidatedGraphInferenceCertificate,
};
use star::latent_roles::{
    discovery_scope_digest, induce_latent_roles, project_evidence_for_control_group,
    project_evidence_for_role, recognize_role_members, structural_fingerprint,
    validate_latent_role, DirectedEdge, LatentRoleProof, RoleInductionBudget,
    RoleInductionConfig, RoleMember, ShadowAbstractionRegistry, StructuralCorpus,
    StructuralFingerprint, StructuralGraph, TransferRecognitionBudget,
    ValidatedLatentRoleCertificate,
};
use star::rule_induction::{
    EvidenceBoundCommitmentState, EvidenceEpisode, RuleInductionConfig, ScoringBudget,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const RANDOM_SEEDS: usize = 8;
const CLOSURE_SCANS: usize = 3;
const RANDOM_BASE_SEED: u64 = 0x4831_3243_5f52_4e44;
const FAMILIES: [&str; 7] = [
    "thermal",
    "transport",
    "ecology",
    "cellular",
    "manufacturing",
    "software",
    "watershed",
];

#[derive(Clone)]
struct Root {
    id: u64,
    family: &'static str,
    corpus: StructuralCorpus,
    transfer: StructuralGraph,
    evidence: MixedEvidenceGraph,
    target_discovery_node: Atom,
    irrelevant_discovery_node: Atom,
    source: Atom,
    middle: Atom,
    goal: Atom,
    irrelevant: Atom,
    irrelevant_goal: Atom,
    degree_decoy: Atom,
    degree_decoy_goal: Atom,
}

#[derive(Clone, PartialEq, Eq)]
struct StateOutcome {
    success_during_window: bool,
    admissions: usize,
    invariants: bool,
    signature: String,
}

#[derive(Clone, PartialEq, Eq)]
struct RootResult {
    primary_success: bool,
    target_role_validated: bool,
    target_shadow_admitted: bool,
    random_seed_count: usize,
    random_candidate_count: usize,
    random_expected_candidate_count: usize,
    random_cardinality_exact: bool,
    random_validation_budget_exact: bool,
    random_target_reconstructions: usize,
    random_target_admissions: usize,
    random_objective_successes: usize,
    degree_local_match: bool,
    degree_full_fingerprint_differs: bool,
    degree_h11_admitted: bool,
    degree_objective_success: bool,
    count_matched_rejected: bool,
    mixed_rejected: bool,
    irrelevant_validated: bool,
    irrelevant_shadow_admitted: bool,
    irrelevant_h11_admitted: bool,
    irrelevant_objective_success: bool,
    foreign_rejected: bool,
    tamper_rejected: bool,
    delayed_window_success: bool,
    delayed_eventual_admission: bool,
    payload_only_success: bool,
    budgets_exact: bool,
    invariants_hold: bool,
    signatures: Vec<String>,
}

#[derive(Default, Serialize)]
struct SplitMetrics {
    roots: usize,
    primary_successes: usize,
}

#[derive(Default, Serialize)]
struct Totals {
    roots: usize,
    target_role_validated: usize,
    target_shadow_admitted: usize,
    random_seed_count: usize,
    random_candidate_count: usize,
    random_expected_candidate_count: usize,
    random_cardinality_exact: usize,
    random_validation_budget_exact: usize,
    random_target_reconstructions: usize,
    random_target_admissions: usize,
    random_objective_successes: usize,
    degree_local_match: usize,
    degree_full_fingerprint_differs: usize,
    degree_h11_admitted: usize,
    degree_objective_successes: usize,
    count_matched_rejections: usize,
    mixed_rejections: usize,
    irrelevant_validations: usize,
    irrelevant_shadow_admissions: usize,
    irrelevant_h11_admissions: usize,
    irrelevant_objective_successes: usize,
    foreign_rejections: usize,
    tamper_rejections: usize,
    delayed_window_successes: usize,
    delayed_eventual_admissions: usize,
    payload_only_successes: usize,
    budgets_exact: usize,
    replay_exact: usize,
    invariants_hold: usize,
}

#[derive(Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    roots: usize,
    random_base_seed: String,
    random_seeds_per_root: usize,
    split_metrics: BTreeMap<String, SplitMetrics>,
    future_family_primary_successes: BTreeMap<String, usize>,
    totals: Totals,
    gates: BTreeMap<String, bool>,
    terminal_classification: &'static str,
    authority_boundary: BTreeMap<String, bool>,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let roots = build_roots();
    let role_config = RoleInductionConfig::default();
    let rule_config = RuleInductionConfig::default();

    let mut rows = Vec::new();
    for root in &roots {
        let first = execute_root(root, role_config, rule_config)?;
        let second = execute_root(root, role_config, rule_config)?;
        rows.push((root, first.clone(), first == second));
    }

    let mut totals = Totals::default();
    let mut splits = BTreeMap::from([
        ("training".to_string(), SplitMetrics::default()),
        ("holdout".to_string(), SplitMetrics::default()),
        ("future".to_string(), SplitMetrics::default()),
    ]);
    let mut future_family_primary = BTreeMap::new();

    for (index, (root, result, replay)) in rows.iter().enumerate() {
        let split = if index < 16 {
            "training"
        } else if index < 24 {
            "holdout"
        } else {
            "future"
        };
        let split_metrics = splits.get_mut(split).expect("frozen split exists");
        split_metrics.roots += 1;
        split_metrics.primary_successes += result.primary_success as usize;

        if index >= 24 {
            *future_family_primary.entry(root.family.to_string()).or_insert(0) +=
                result.primary_success as usize;
        }

        totals.roots += 1;
        totals.target_role_validated += result.target_role_validated as usize;
        totals.target_shadow_admitted += result.target_shadow_admitted as usize;
        totals.random_seed_count += result.random_seed_count;
        totals.random_candidate_count += result.random_candidate_count;
        totals.random_expected_candidate_count += result.random_expected_candidate_count;
        totals.random_cardinality_exact += result.random_cardinality_exact as usize;
        totals.random_validation_budget_exact += result.random_validation_budget_exact as usize;
        totals.random_target_reconstructions += result.random_target_reconstructions;
        totals.random_target_admissions += result.random_target_admissions;
        totals.random_objective_successes += result.random_objective_successes;
        totals.degree_local_match += result.degree_local_match as usize;
        totals.degree_full_fingerprint_differs +=
            result.degree_full_fingerprint_differs as usize;
        totals.degree_h11_admitted += result.degree_h11_admitted as usize;
        totals.degree_objective_successes += result.degree_objective_success as usize;
        totals.count_matched_rejections += result.count_matched_rejected as usize;
        totals.mixed_rejections += result.mixed_rejected as usize;
        totals.irrelevant_validations += result.irrelevant_validated as usize;
        totals.irrelevant_shadow_admissions += result.irrelevant_shadow_admitted as usize;
        totals.irrelevant_h11_admissions += result.irrelevant_h11_admitted as usize;
        totals.irrelevant_objective_successes += result.irrelevant_objective_success as usize;
        totals.foreign_rejections += result.foreign_rejected as usize;
        totals.tamper_rejections += result.tamper_rejected as usize;
        totals.delayed_window_successes += result.delayed_window_success as usize;
        totals.delayed_eventual_admissions += result.delayed_eventual_admission as usize;
        totals.payload_only_successes += result.payload_only_success as usize;
        totals.budgets_exact += result.budgets_exact as usize;
        totals.replay_exact += *replay as usize;
        totals.invariants_hold += result.invariants_hold as usize;
    }

    let mut gates: BTreeMap<String, bool> = BTreeMap::new();
    gates.insert("cohort_exact".into(), roots.len() == 56);
    gates.insert(
        "primary_training".into(),
        splits["training"].roots == 16 && splits["training"].primary_successes == 16,
    );
    gates.insert(
        "primary_holdout".into(),
        splits["holdout"].roots == 8 && splits["holdout"].primary_successes == 8,
    );
    gates.insert(
        "primary_future".into(),
        splits["future"].roots == 32 && splits["future"].primary_successes == 32,
    );
    gates.insert(
        "future_families_exact".into(),
        future_family_primary.len() == 4
            && future_family_primary.values().all(|successes| *successes == 8),
    );
    gates.insert(
        "target_role_validated".into(),
        totals.target_role_validated == 56 && totals.target_shadow_admitted == 56,
    );
    gates.insert(
        "random_seed_count_exact".into(),
        totals.random_seed_count == 56 * RANDOM_SEEDS,
    );
    gates.insert(
        "random_candidate_count_exact".into(),
        totals.random_candidate_count == totals.random_expected_candidate_count,
    );
    gates.insert(
        "random_cardinality_exact".into(),
        totals.random_cardinality_exact == 56,
    );
    gates.insert(
        "random_validation_budget_exact".into(),
        totals.random_validation_budget_exact == 56,
    );
    gates.insert(
        "random_zero_reconstruction".into(),
        totals.random_target_reconstructions == 0,
    );
    gates.insert(
        "random_zero_admission".into(),
        totals.random_target_admissions == 0,
    );
    gates.insert(
        "random_zero_objective".into(),
        totals.random_objective_successes == 0,
    );
    gates.insert(
        "degree_matched_control".into(),
        totals.degree_local_match == 56
            && totals.degree_full_fingerprint_differs == 56
            && totals.degree_h11_admitted == 56
            && totals.degree_objective_successes == 0,
    );
    gates.insert(
        "membership_count_matched_rejected".into(),
        totals.count_matched_rejections == 56,
    );
    gates.insert("mixed_rejected".into(), totals.mixed_rejections == 56);
    gates.insert(
        "valid_irrelevant_control".into(),
        totals.irrelevant_validations == 56
            && totals.irrelevant_shadow_admissions == 56
            && totals.irrelevant_h11_admissions == 56
            && totals.irrelevant_objective_successes == 0,
    );
    gates.insert("foreign_rejected".into(), totals.foreign_rejections == 56);
    gates.insert("tamper_rejected".into(), totals.tamper_rejections == 56);
    gates.insert(
        "delayed_inert_during_window".into(),
        totals.delayed_window_successes == 0 && totals.delayed_eventual_admissions == 56,
    );
    gates.insert(
        "payload_only_inert".into(),
        totals.payload_only_successes == 0,
    );
    gates.insert("budgets_exact".into(), totals.budgets_exact == 56);
    gates.insert("replay_exact".into(), totals.replay_exact == 56);
    gates.insert("invariants_hold".into(), totals.invariants_hold == 56);

    let infrastructure = !gates["budgets_exact"]
        || !gates["replay_exact"]
        || !gates["invariants_hold"]
        || !gates["random_seed_count_exact"]
        || !gates["random_candidate_count_exact"]
        || !gates["random_cardinality_exact"]
        || !gates["random_validation_budget_exact"];
    let primary_rejected = !gates["cohort_exact"]
        || !gates["primary_training"]
        || !gates["primary_holdout"]
        || !gates["primary_future"]
        || !gates["future_families_exact"]
        || !gates["target_role_validated"];
    let controls_failed = gates
        .iter()
        .filter(|(name, _)| {
            !matches!(
                name.as_str(),
                "cohort_exact"
                    | "primary_training"
                    | "primary_holdout"
                    | "primary_future"
                    | "future_families_exact"
                    | "target_role_validated"
                    | "budgets_exact"
                    | "replay_exact"
                    | "invariants_hold"
                    | "random_seed_count_exact"
                    | "random_candidate_count_exact"
                    | "random_cardinality_exact"
                    | "random_validation_budget_exact"
            )
        })
        .any(|(_, passed)| !*passed);

    let terminal = if infrastructure {
        "INFRASTRUCTURE_FAILURE"
    } else if primary_rejected {
        "REJECTED"
    } else if controls_failed {
        "CONTROL_FAILURE"
    } else if gates.values().all(|passed| *passed) {
        "PASS"
    } else {
        "INCONCLUSIVE"
    };

    let report = Report {
        experiment: "H12-C literal root-seeded same-cardinality control conformance",
        preregistration_commit: "707cef76c3bf2fdf86d3a9eaf1e404ebb0bc5de3",
        roots: roots.len(),
        random_base_seed: format!("0x{RANDOM_BASE_SEED:016x}"),
        random_seeds_per_root: RANDOM_SEEDS,
        split_metrics: splits,
        future_family_primary_successes: future_family_primary,
        totals,
        gates,
        terminal_classification: terminal,
        authority_boundary: BTreeMap::from([
            ("live_registry_state".into(), false),
            ("runtime_chat_influence".into(), false),
            ("automatic_ontology_promotion".into(), false),
            ("autonomous_action_authority".into(), false),
            ("persistent_production_mutation".into(), false),
        ]),
        claim_boundary: "PASS supports only literal control conformance for the bounded synthetic H12 latent-role mechanism; it does not establish unrestricted ontology induction, grammar invention, causal abstraction, live promotion, AGI, consciousness, or human-level cognition",
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal != "PASS" {
        return Err(format!("H12-C terminal classification: {terminal}").into());
    }
    Ok(())
}

fn execute_root(
    root: &Root,
    role_config: RoleInductionConfig,
    rule_config: RuleInductionConfig,
) -> Result<RootResult, Box<dyn Error>> {
    let corpus_nodes = root
        .corpus
        .graphs
        .iter()
        .map(|graph| graph.nodes.len())
        .sum::<usize>();

    let mut discovery_budget = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&root.corpus, role_config, &mut discovery_budget)?;
    let mut registry = ShadowAbstractionRegistry::new(discovery_scope_digest(&root.corpus)?);
    for proof in &proofs {
        registry.register_candidate(proof)?;
    }

    let target_fingerprint = fingerprint_for(
        &root.corpus.graphs[0],
        &root.target_discovery_node,
    )?;
    let irrelevant_fingerprint = fingerprint_for(
        &root.corpus.graphs[0],
        &root.irrelevant_discovery_node,
    )?;
    let target_proof = proof_for_fingerprint(&proofs, target_fingerprint)?;
    let irrelevant_proof = proof_for_fingerprint(&proofs, irrelevant_fingerprint)?;

    let mut certificates = BTreeMap::new();
    let mut all_role_validation_exact = true;
    for proof in &proofs {
        let mut budget = RoleInductionBudget::default();
        let certificate = validate_latent_role(&root.corpus, proof, role_config, &mut budget)?;
        all_role_validation_exact &= budget.node_fingerprint_evaluations == corpus_nodes;
        registry.admit_validated(&certificate)?;
        certificates.insert(certificate.role_id(), certificate);
    }
    let target_certificate = certificates
        .get(&target_proof.role_id)
        .ok_or("target certificate absent")?;
    let irrelevant_certificate = certificates
        .get(&irrelevant_proof.role_id)
        .ok_or("irrelevant certificate absent")?;

    let mut target_transfer_budget = TransferRecognitionBudget::default();
    let target_projection = project_evidence_for_role(
        &root.transfer,
        &root.evidence,
        target_certificate,
        &registry,
        &mut target_transfer_budget,
    )?;
    let target_h11 = h11(&target_projection, rule_config);
    let target_h11_certificate = target_h11
        .certificate
        .as_ref()
        .ok_or("target H11 certificate absent")?;
    let primary = execute_state(root, Some(target_h11_certificate), true, false)?;

    let universe = raw_member_universe(&root.corpus);
    let target_count = target_proof.members.len();
    let expected_per_seed = universe.len() / target_count;
    let mut random_candidate_count = 0usize;
    let mut random_cardinality_exact = target_count == 4;
    let mut random_validation_budget_exact = true;
    let mut random_target_reconstructions = 0usize;
    let mut random_target_admissions = 0usize;
    let mut random_objective_successes = 0usize;

    for seed_index in 0..RANDOM_SEEDS {
        let mut shuffled = universe.clone();
        let seed = random_seed(root.id, seed_index);
        fisher_yates(&mut shuffled, SplitMix64(seed));
        for chunk in shuffled.chunks_exact(target_count) {
            random_candidate_count += 1;
            random_cardinality_exact &= chunk.len() == target_count;
            let candidate = membership_substitution(target_proof, chunk.to_vec());
            let mut budget = RoleInductionBudget::default();
            if let Ok(certificate) =
                validate_latent_role(&root.corpus, &candidate, role_config, &mut budget)
            {
                random_target_reconstructions += 1;
                let mut random_registry =
                    ShadowAbstractionRegistry::new(discovery_scope_digest(&root.corpus)?);
                random_registry.register_candidate(&candidate)?;
                random_registry.admit_validated(&certificate)?;
                random_target_admissions += 1;
                let mut projection_budget = TransferRecognitionBudget::default();
                let projected = project_evidence_for_role(
                    &root.transfer,
                    &root.evidence,
                    &certificate,
                    &random_registry,
                    &mut projection_budget,
                )?;
                let attempt = h11(&projected, rule_config);
                if let Some(h11_certificate) = attempt.certificate.as_ref() {
                    random_objective_successes +=
                        execute_state(root, Some(h11_certificate), true, false)?
                            .success_during_window as usize;
                }
            }
            random_validation_budget_exact &= budget.node_fingerprint_evaluations == corpus_nodes;
        }
    }

    let target_set = target_proof.members.iter().cloned().collect::<BTreeSet<_>>();
    let unrelated = universe
        .iter()
        .filter(|member| !target_set.contains(*member))
        .take(target_count)
        .cloned()
        .collect::<Vec<_>>();
    let count_candidate = membership_substitution(target_proof, unrelated.clone());
    let mut count_budget = RoleInductionBudget::default();
    let count_matched_rejected =
        validate_latent_role(&root.corpus, &count_candidate, role_config, &mut count_budget)
            .is_err();

    let mut mixed_members = target_proof.members[..target_count / 2].to_vec();
    mixed_members.extend(unrelated.into_iter().take(target_count - mixed_members.len()));
    let mixed_candidate = membership_substitution(target_proof, mixed_members);
    let mut mixed_budget = RoleInductionBudget::default();
    let mixed_rejected =
        validate_latent_role(&root.corpus, &mixed_candidate, role_config, &mut mixed_budget)
            .is_err();

    let mut tampered_members = target_proof.members.clone();
    tampered_members[0] = universe
        .iter()
        .find(|member| !target_set.contains(*member))
        .ok_or("tamper substitute absent")?
        .clone();
    let tampered = membership_substitution(target_proof, tampered_members);
    let mut tamper_budget = RoleInductionBudget::default();
    let tamper_rejected =
        validate_latent_role(&root.corpus, &tampered, role_config, &mut tamper_budget)
            .is_err();

    let target_transfer_fingerprint = fingerprint_for(&root.transfer, &root.middle)?;
    let decoy_transfer_fingerprint = fingerprint_for(&root.transfer, &root.degree_decoy)?;
    let degree_local_match = target_transfer_fingerprint.in_degree
        == decoy_transfer_fingerprint.in_degree
        && target_transfer_fingerprint.out_degree == decoy_transfer_fingerprint.out_degree;
    let degree_full_fingerprint_differs =
        target_transfer_fingerprint != decoy_transfer_fingerprint;
    let mut degree_projection_budget = TransferRecognitionBudget::default();
    let degree_projection = project_evidence_for_control_group(
        &root.evidence,
        &BTreeSet::from([root.degree_decoy.clone()]),
        &mut degree_projection_budget,
    )?;
    let degree_h11 = h11(&degree_projection, rule_config);
    let degree = execute_state(root, degree_h11.certificate.as_ref(), true, false)?;

    let mut irrelevant_transfer_budget = TransferRecognitionBudget::default();
    let irrelevant_projection = project_evidence_for_role(
        &root.transfer,
        &root.evidence,
        irrelevant_certificate,
        &registry,
        &mut irrelevant_transfer_budget,
    )?;
    let irrelevant_h11 = h11(&irrelevant_projection, rule_config);
    let irrelevant = execute_state(root, irrelevant_h11.certificate.as_ref(), true, false)?;

    let foreign_certificate = build_foreign_certificate(root.id, role_config)?;
    let mut foreign_budget = TransferRecognitionBudget::default();
    let foreign_rejected = recognize_role_members(
        &root.transfer,
        &foreign_certificate,
        &registry,
        &mut foreign_budget,
    )
    .is_err();

    let delayed = execute_state(root, Some(target_h11_certificate), false, true)?;
    let payload = execute_state(root, None, false, false)?;

    let projections_exact = target_transfer_budget.node_fingerprint_evaluations
        == root.transfer.nodes.len()
        && target_transfer_budget.evidence_episode_scans == root.evidence.evidence.len()
        && irrelevant_transfer_budget.node_fingerprint_evaluations
            == root.transfer.nodes.len()
        && irrelevant_transfer_budget.evidence_episode_scans == root.evidence.evidence.len()
        && degree_projection_budget.node_fingerprint_evaluations == 0
        && degree_projection_budget.evidence_episode_scans == root.evidence.evidence.len()
        && foreign_budget.node_fingerprint_evaluations == 0
        && foreign_budget.evidence_episode_scans == 0;

    let forged_validation_exact = count_budget.node_fingerprint_evaluations == corpus_nodes
        && mixed_budget.node_fingerprint_evaluations == corpus_nodes
        && tamper_budget.node_fingerprint_evaluations == corpus_nodes;
    let h11_exact = target_h11.exact && degree_h11.exact && irrelevant_h11.exact;
    let budgets_exact = discovery_budget.node_fingerprint_evaluations == corpus_nodes
        && all_role_validation_exact
        && random_validation_budget_exact
        && forged_validation_exact
        && projections_exact
        && h11_exact;
    let invariants_hold = primary.invariants
        && degree.invariants
        && irrelevant.invariants
        && delayed.invariants
        && payload.invariants;

    Ok(RootResult {
        primary_success: primary.success_during_window,
        target_role_validated: target_certificate.member_count() == target_count,
        target_shadow_admitted: registry.status(target_certificate.role_id()).is_some(),
        random_seed_count: RANDOM_SEEDS,
        random_candidate_count,
        random_expected_candidate_count: expected_per_seed * RANDOM_SEEDS,
        random_cardinality_exact,
        random_validation_budget_exact,
        random_target_reconstructions,
        random_target_admissions,
        random_objective_successes,
        degree_local_match,
        degree_full_fingerprint_differs,
        degree_h11_admitted: degree.admissions == 1,
        degree_objective_success: degree.success_during_window,
        count_matched_rejected,
        mixed_rejected,
        irrelevant_validated: irrelevant_certificate.member_count()
            == irrelevant_proof.members.len(),
        irrelevant_shadow_admitted: registry.status(irrelevant_certificate.role_id()).is_some(),
        irrelevant_h11_admitted: irrelevant.admissions == 1,
        irrelevant_objective_success: irrelevant.success_during_window,
        foreign_rejected,
        tamper_rejected,
        delayed_window_success: delayed.success_during_window,
        delayed_eventual_admission: delayed.admissions == 1,
        payload_only_success: payload.success_during_window,
        budgets_exact,
        invariants_hold,
        signatures: vec![
            primary.signature,
            degree.signature,
            irrelevant.signature,
            delayed.signature,
            payload.signature,
        ],
    })
}

fn execute_state(
    root: &Root,
    certificate: Option<&ValidatedGraphInferenceCertificate>,
    admit_before: bool,
    admit_after: bool,
) -> Result<StateOutcome, Box<dyn Error>> {
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    let mut admissions = 0usize;

    if admit_before {
        if let Some(certificate) = certificate {
            admit_graph_certificate(&mut state, certificate)?;
            admissions += 1;
        }
    }
    for _ in 0..CLOSURE_SCANS {
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
        }
    }
    let success_during_window = state.contains_fact(&root.goal);
    if admit_after {
        if let Some(certificate) = certificate {
            admit_graph_certificate(&mut state, certificate)?;
            admissions += 1;
        }
    }

    Ok(StateOutcome {
        success_during_window,
        admissions,
        invariants: state.verify_invariants().is_ok(),
        signature: state.canonical_signature(),
    })
}

struct H11Attempt {
    certificate: Option<ValidatedGraphInferenceCertificate>,
    exact: bool,
}

fn h11(graph: &MixedEvidenceGraph, config: RuleInductionConfig) -> H11Attempt {
    let mut proposal_frontier = FrontierDiscoveryBudget::default();
    let mut proposal_scoring = ScoringBudget::default();
    let proof = infer_graph_rule(
        graph,
        config,
        &mut proposal_frontier,
        &mut proposal_scoring,
    );
    let expected_proposal = proposal_frontier
        .discovered_antecedents
        .saturating_mul(proposal_frontier.discovered_consequents)
        .saturating_mul(graph.evidence.len());
    let mut exact = proposal_frontier.evidence_episode_scans == graph.evidence.len()
        && proposal_scoring.candidate_episode_evaluations == expected_proposal;
    let Ok(proof) = proof else {
        return H11Attempt {
            certificate: None,
            exact,
        };
    };

    let mut validation_frontier = FrontierDiscoveryBudget::default();
    let mut validation_scoring = ScoringBudget::default();
    let certificate = validate_graph_rule(
        graph,
        &proof,
        config,
        &mut validation_frontier,
        &mut validation_scoring,
    )
    .ok();
    let expected_validation = validation_frontier
        .discovered_antecedents
        .saturating_mul(validation_frontier.discovered_consequents)
        .saturating_mul(graph.evidence.len());
    exact &= validation_frontier.evidence_episode_scans == graph.evidence.len()
        && validation_scoring.candidate_episode_evaluations == expected_validation;
    H11Attempt { certificate, exact }
}

fn proof_for_fingerprint(
    proofs: &[LatentRoleProof],
    fingerprint: StructuralFingerprint,
) -> Result<&LatentRoleProof, Box<dyn Error>> {
    proofs
        .iter()
        .find(|proof| proof.structural_fingerprint == fingerprint)
        .ok_or_else(|| "expected structural role proof absent".into())
}

fn fingerprint_for(
    graph: &StructuralGraph,
    node: &Atom,
) -> Result<StructuralFingerprint, Box<dyn Error>> {
    let mut budget = RoleInductionBudget::default();
    Ok(structural_fingerprint(graph, node, &mut budget)?)
}

fn raw_member_universe(corpus: &StructuralCorpus) -> Vec<RoleMember> {
    let mut members = corpus
        .graphs
        .iter()
        .flat_map(|graph| {
            graph.nodes.iter().cloned().map(|node| RoleMember {
                graph_id: graph.graph_id,
                node,
            })
        })
        .collect::<Vec<_>>();
    members.sort();
    members
}

fn membership_substitution(
    target: &LatentRoleProof,
    mut members: Vec<RoleMember>,
) -> LatentRoleProof {
    members.sort();
    let supporting_graph_ids = members
        .iter()
        .map(|member| member.graph_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut candidate = target.clone();
    candidate.members = members;
    candidate.supporting_graph_ids = supporting_graph_ids;
    candidate
}

fn random_seed(root_id: u64, seed_index: usize) -> u64 {
    RANDOM_BASE_SEED
        ^ root_id.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (seed_index as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

fn fisher_yates<T>(items: &mut [T], mut rng: SplitMix64) {
    for index in (1..items.len()).rev() {
        let selected = (rng.next() % (index as u64 + 1)) as usize;
        items.swap(index, selected);
    }
}

fn build_foreign_certificate(
    root_id: u64,
    role_config: RoleInductionConfig,
) -> Result<ValidatedLatentRoleCertificate, Box<dyn Error>> {
    let prefix = format!("h12c_foreign_{root_id}");
    let (corpus, target_node, _) = discovery_corpus(root_id + 100_000, &prefix);
    let target_fingerprint = fingerprint_for(&corpus.graphs[0], &target_node)?;
    let mut discovery_budget = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&corpus, role_config, &mut discovery_budget)?;
    let proof = proof_for_fingerprint(&proofs, target_fingerprint)?;
    let mut validation_budget = RoleInductionBudget::default();
    Ok(validate_latent_role(
        &corpus,
        proof,
        role_config,
        &mut validation_budget,
    )?)
}

fn build_roots() -> Vec<Root> {
    let mut roots = Vec::new();
    let mut id = 1u64;
    for (family_index, family) in FAMILIES.into_iter().enumerate() {
        for local in 0..ROOTS_PER_FAMILY {
            let prefix = format!("h12c_{family}_{id}_{local}");
            let (corpus, target_discovery_node, irrelevant_discovery_node) =
                discovery_corpus(id, &prefix);
            let source = atom(format!("{prefix}_source"));
            let middle = atom(format!("{prefix}_middle"));
            let goal = atom(format!("{prefix}_goal"));
            let irrelevant = atom(format!("{prefix}_irrelevant"));
            let irrelevant_goal = atom(format!("{prefix}_irrelevant_goal"));
            let degree_decoy = atom(format!("{prefix}_degree_decoy"));
            let degree_decoy_goal = atom(format!("{prefix}_degree_decoy_goal"));
            let transfer = transfer_graph(
                id,
                &prefix,
                family_index,
                &middle,
                &irrelevant,
                &degree_decoy,
            );
            let evidence = evidence_graph(
                id,
                &middle,
                &goal,
                &irrelevant,
                &irrelevant_goal,
                &degree_decoy,
                &degree_decoy_goal,
            );
            roots.push(Root {
                id,
                family,
                corpus,
                transfer,
                evidence,
                target_discovery_node,
                irrelevant_discovery_node,
                source,
                middle,
                goal,
                irrelevant,
                irrelevant_goal,
                degree_decoy,
                degree_decoy_goal,
            });
            id += 1;
        }
    }
    roots
}

fn discovery_corpus(root_id: u64, prefix: &str) -> (StructuralCorpus, Atom, Atom) {
    let mut graphs = Vec::new();
    let mut first_target = None;
    let mut first_irrelevant = None;
    for index in 0..4 {
        let graph_prefix = format!("{prefix}_discover_{index}");
        let target = atom(format!("{graph_prefix}_target"));
        let irrelevant = atom(format!("{graph_prefix}_irrelevant"));
        if index == 0 {
            first_target = Some(target.clone());
            first_irrelevant = Some(irrelevant.clone());
        }
        let decoy = atom(format!("{graph_prefix}_degree_decoy"));
        let mut builder = Builder::new(root_id * 100 + index as u64 + 1);
        target_motif(&mut builder, &graph_prefix, target, 2, 2);
        irrelevant_motif(&mut builder, &graph_prefix, irrelevant);
        builder.edge(atom(format!("{graph_prefix}_before")), decoy.clone());
        builder.edge(decoy, atom(format!("{graph_prefix}_after")));
        graphs.push(builder.finish());
    }
    (
        StructuralCorpus { graphs },
        first_target.expect("target node"),
        first_irrelevant.expect("irrelevant node"),
    )
}

fn transfer_graph(
    root_id: u64,
    prefix: &str,
    family_index: usize,
    middle: &Atom,
    irrelevant: &Atom,
    degree_decoy: &Atom,
) -> StructuralGraph {
    let mut builder = Builder::new(root_id * 10_000 + 9_001);
    target_motif(
        &mut builder,
        &format!("{prefix}_target_transfer"),
        middle.clone(),
        2 + family_index,
        2 + family_index % 3,
    );
    irrelevant_motif(
        &mut builder,
        &format!("{prefix}_irrelevant_transfer"),
        irrelevant.clone(),
    );
    builder.edge(
        atom(format!("{prefix}_degree_before")),
        degree_decoy.clone(),
    );
    builder.edge(
        degree_decoy.clone(),
        atom(format!("{prefix}_degree_after")),
    );
    for branch in 0..family_index {
        let source = atom(format!("{prefix}_extra_{branch}_source"));
        builder.edge(source.clone(), atom(format!("{prefix}_extra_{branch}_a")));
        builder.edge(source, atom(format!("{prefix}_extra_{branch}_b")));
    }
    builder.finish()
}

fn evidence_graph(
    root_id: u64,
    middle: &Atom,
    goal: &Atom,
    irrelevant: &Atom,
    irrelevant_goal: &Atom,
    degree_decoy: &Atom,
    degree_decoy_goal: &Atom,
) -> MixedEvidenceGraph {
    let mut evidence = Vec::new();
    let mut evidence_id = root_id * 1_000_000 + 1;
    for _ in 0..4 {
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: middle.clone(),
            outcomes: BTreeSet::from([goal.clone()]),
        });
        evidence_id += 1;
    }
    for _ in 0..5 {
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: irrelevant.clone(),
            outcomes: BTreeSet::from([irrelevant_goal.clone()]),
        });
        evidence_id += 1;
    }
    for _ in 0..4 {
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: degree_decoy.clone(),
            outcomes: BTreeSet::from([degree_decoy_goal.clone()]),
        });
        evidence_id += 1;
    }
    MixedEvidenceGraph { evidence }
}

fn target_motif(
    builder: &mut Builder,
    prefix: &str,
    target: Atom,
    fan_in: usize,
    fan_out: usize,
) {
    let join = atom(format!("{prefix}_join"));
    let split = atom(format!("{prefix}_split"));
    for index in 0..fan_in {
        builder.edge(atom(format!("{prefix}_source_{index}")), join.clone());
    }
    builder.edge(join, target.clone());
    builder.edge(target, split.clone());
    for index in 0..fan_out {
        builder.edge(split.clone(), atom(format!("{prefix}_sink_{index}")));
    }
}

fn irrelevant_motif(builder: &mut Builder, prefix: &str, irrelevant: Atom) {
    let branch = atom(format!("{prefix}_branch"));
    let merge = atom(format!("{prefix}_merge"));
    builder.edge(atom(format!("{prefix}_source")), branch.clone());
    builder.edge(branch.clone(), irrelevant.clone());
    builder.edge(branch, atom(format!("{prefix}_branch_leaf")));
    builder.edge(irrelevant, merge.clone());
    builder.edge(atom(format!("{prefix}_merge_source")), merge.clone());
    builder.edge(merge, atom(format!("{prefix}_sink")));
}

struct Builder {
    graph_id: u64,
    nodes: BTreeSet<Atom>,
    edges: BTreeSet<DirectedEdge>,
}

impl Builder {
    fn new(graph_id: u64) -> Self {
        Self {
            graph_id,
            nodes: BTreeSet::new(),
            edges: BTreeSet::new(),
        }
    }

    fn edge(&mut self, from: Atom, to: Atom) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        self.edges.insert(DirectedEdge { from, to });
    }

    fn finish(self) -> StructuralGraph {
        StructuralGraph {
            graph_id: self.graph_id,
            nodes: self.nodes.into_iter().collect(),
            edges: self.edges.into_iter().collect(),
        }
    }
}

fn atom(value: impl Into<String>) -> Atom {
    Atom::new(value.into()).expect("generated H12-C atom is non-empty")
}
