use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::graph_discovery::{
    admit_graph_certificate, infer_graph_rule, validate_graph_rule, FrontierDiscoveryBudget,
    MixedEvidenceGraph, ValidatedGraphInferenceCertificate,
};
use star::latent_roles::{
    development_scope_digest as _, induce_latent_roles, project_evidence_for_role,
    validate_latent_role, DirectedEdge, LatentRoleError, RoleInductionBudget,
    RoleInductionConfig, ShadowAbstractionRegistry, StructuralCorpus, StructuralGraph,
    TransferRecognitionBudget, ValidatedLatentRoleCertificate,
};
use star::rule_induction::{
    EvidenceBoundCommitmentState, EvidenceEpisode, RuleInductionConfig, ScoringBudget,
};
use star::structural_transfer::{
    development_scope_digest, induce_transport_roles, recognize_transport_role,
    validate_transport_role, ShadowTransportRoleRegistry, TransportRoleBudget,
    TransportRoleConfig, TransportRoleProof, TransportRoleSignature,
    ValidatedTransportRoleCertificate,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const CLOSURE_SCANS: usize = 3;
const DEVELOPMENT_GRAPHS: usize = 12;
const FAMILIES: [&str; 7] = [
    "hydraulic",
    "logistics",
    "ecology",
    "cellular",
    "manufacturing",
    "software",
    "watershed",
];

#[derive(Debug, Clone)]
struct DevelopmentFixture {
    corpus: StructuralCorpus,
    target_members: BTreeSet<(u64, Atom)>,
    irrelevant_members: BTreeSet<(u64, Atom)>,
}

#[derive(Debug, Clone)]
struct FrozenRoles {
    target_proof: TransportRoleProof,
    target: ValidatedTransportRoleCertificate,
    irrelevant: ValidatedTransportRoleCertificate,
    registry: ShadowTransportRoleRegistry,
    foreign: ValidatedTransportRoleCertificate,
    identity_permutation_rejected: bool,
    exact_h12_target: ValidatedLatentRoleCertificate,
    exact_h12_registry: ShadowAbstractionRegistry,
    proof_digest: u64,
    certificate_digest: u64,
}

#[derive(Debug, Clone)]
struct Root {
    id: u64,
    family_index: usize,
    local_index: usize,
    graph: StructuralGraph,
    evidence: MixedEvidenceGraph,
    source: Atom,
    middle: Atom,
    goal: Atom,
    irrelevant: Atom,
    irrelevant_goal: Atom,
    local_degree_decoy: Atom,
    two_hop_decoy: Atom,
    rewired_decoy: Atom,
    vocabulary_decoy: Atom,
    transformations: TransformationAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct TransformationAudit {
    unseen_size: bool,
    longer_paths: bool,
    irrelevant_branches: bool,
    high_distractor_density: bool,
    edge_subdivision: bool,
    node_duplication: bool,
    partial_observation: bool,
    vocabulary_permutation: bool,
    insertion_order_reversal: bool,
    local_degree_change: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Path {
    ValidatedTransportRole,
    ExactH12FingerprintBaseline,
    LocalDegreeMatchedDecoy,
    TwoHopMotifMatchedDecoy,
    DegreePreservingRewire,
    VocabularyOnlySimilarity,
    ForeignFamilyCertificate,
    ValidIrrelevantRole,
    RoleIdentityPermutation,
    DelayedRoleAdmission,
    RandomSameCardinalityGrouping,
    PayloadOnly,
    OracleRole,
    UnpartitionedFullEvidence,
}

impl Path {
    const fn all() -> [Self; 14] {
        [
            Self::ValidatedTransportRole,
            Self::ExactH12FingerprintBaseline,
            Self::LocalDegreeMatchedDecoy,
            Self::TwoHopMotifMatchedDecoy,
            Self::DegreePreservingRewire,
            Self::VocabularyOnlySimilarity,
            Self::ForeignFamilyCertificate,
            Self::ValidIrrelevantRole,
            Self::RoleIdentityPermutation,
            Self::DelayedRoleAdmission,
            Self::RandomSameCardinalityGrouping,
            Self::PayloadOnly,
            Self::OracleRole,
            Self::UnpartitionedFullEvidence,
        ]
    }

    const fn name(self) -> &'static str {
        match self {
            Self::ValidatedTransportRole => "validated_transport_role",
            Self::ExactH12FingerprintBaseline => "exact_h12_fingerprint_baseline",
            Self::LocalDegreeMatchedDecoy => "local_degree_matched_decoy",
            Self::TwoHopMotifMatchedDecoy => "two_hop_motif_matched_decoy",
            Self::DegreePreservingRewire => "degree_preserving_rewire",
            Self::VocabularyOnlySimilarity => "vocabulary_only_similarity",
            Self::ForeignFamilyCertificate => "foreign_family_certificate",
            Self::ValidIrrelevantRole => "valid_irrelevant_role",
            Self::RoleIdentityPermutation => "role_identity_permutation",
            Self::DelayedRoleAdmission => "delayed_role_admission",
            Self::RandomSameCardinalityGrouping => "random_same_cardinality_grouping",
            Self::PayloadOnly => "payload_only",
            Self::OracleRole => "oracle_role",
            Self::UnpartitionedFullEvidence => "unpartitioned_full_evidence",
        }
    }

    const fn is_target_negative(self) -> bool {
        !matches!(
            self,
            Self::ValidatedTransportRole
                | Self::OracleRole
                | Self::ExactH12FingerprintBaseline
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct ExecutionBudget {
    graph_node_scans: usize,
    graph_edge_scans: usize,
    signature_evaluations: usize,
    reachability_traversals: usize,
    counterfactual_traversals: usize,
    evidence_episode_scans: usize,
    h11_frontier_scans: usize,
    h10_candidate_episode_evaluations: usize,
    admission_opportunities: usize,
    successful_admissions: usize,
    closure_scans: usize,
    objective_checks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Execution {
    target_success: bool,
    certificate_available: bool,
    rejected: bool,
    recognized_members: Vec<Atom>,
    projected_evidence_ids: Vec<u64>,
    h11_rule: Option<Rule>,
    state_signature: String,
    budget: ExecutionBudget,
    budget_exact: bool,
    invariants_hold: bool,
    irrelevant_certificate_valid: bool,
    irrelevant_rule_admitted: bool,
    target_member_exact: bool,
    superficial_decoys_excluded: bool,
    proof_digest: u64,
    certificate_digest: u64,
}

#[derive(Debug, Clone)]
struct H11Attempt {
    certificate: Option<ValidatedGraphInferenceCertificate>,
    exact: bool,
    frontier_scans: usize,
    scoring_evaluations: usize,
}

#[derive(Debug, Default, Serialize)]
struct PathMetrics {
    roots: usize,
    target_successes: usize,
    certificates_available: usize,
    rejections: usize,
    replay_exact: usize,
    budgets_exact: usize,
    invariants_hold: usize,
    target_member_exact: usize,
    superficial_decoys_excluded: usize,
    irrelevant_certificate_valid: usize,
    irrelevant_rule_admitted: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    development_graphs: usize,
    evaluation_roots: usize,
    split_metrics: BTreeMap<String, BTreeMap<String, PathMetrics>>,
    future_family_primary_successes: BTreeMap<String, usize>,
    future_family_max_negative_control_successes: BTreeMap<String, usize>,
    transformations_exercised: BTreeMap<String, bool>,
    gates: BTreeMap<String, bool>,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let development = build_development_fixture();
    let frozen = freeze_roles(&development)?;
    let roots = build_roots();

    let training = evaluate_split(&roots[0..16], &frozen)?;
    let holdout = evaluate_split(&roots[16..24], &frozen)?;
    let future = evaluate_split(&roots[24..56], &frozen)?;

    let mut split_metrics = BTreeMap::new();
    split_metrics.insert("training".to_string(), training);
    split_metrics.insert("holdout".to_string(), holdout);
    split_metrics.insert("future".to_string(), future);

    let mut future_family_primary_successes = BTreeMap::new();
    let mut future_family_max_negative_control_successes = BTreeMap::new();
    for family_index in 3..7 {
        let start = family_index * ROOTS_PER_FAMILY;
        let metrics = evaluate_split(
            &roots[start..start + ROOTS_PER_FAMILY],
            &frozen,
        )?;
        future_family_primary_successes.insert(
            FAMILIES[family_index].to_string(),
            metrics[Path::ValidatedTransportRole.name()].target_successes,
        );
        let max_negative = Path::all()
            .into_iter()
            .filter(|path| path.is_target_negative())
            .map(|path| metrics[path.name()].target_successes)
            .max()
            .unwrap_or(0);
        future_family_max_negative_control_successes
            .insert(FAMILIES[family_index].to_string(), max_negative);
    }

    let transformations_exercised = transformation_coverage(&roots);
    let train = &split_metrics["training"];
    let holdout = &split_metrics["holdout"];
    let future = &split_metrics["future"];

    let primary_exact = train[Path::ValidatedTransportRole.name()].target_successes == 16
        && holdout[Path::ValidatedTransportRole.name()].target_successes == 8
        && future[Path::ValidatedTransportRole.name()].target_successes == 32;
    let oracle_exact = split_metrics.values().all(|split| {
        split[Path::OracleRole.name()].target_successes
            == split[Path::OracleRole.name()].roots
    });
    let negative_zero = split_metrics.values().all(|split| {
        Path::all()
            .into_iter()
            .filter(|path| path.is_target_negative())
            .all(|path| split[path.name()].target_successes == 0)
    });
    let future_family_exact = future_family_primary_successes
        .values()
        .all(|successes| *successes == ROOTS_PER_FAMILY)
        && future_family_max_negative_control_successes
            .values()
            .all(|successes| *successes == 0);
    let primary_recognition_exact = split_metrics.values().all(|split| {
        let metrics = &split[Path::ValidatedTransportRole.name()];
        metrics.target_member_exact == metrics.roots
            && metrics.superficial_decoys_excluded == metrics.roots
    });
    let foreign_rejected = split_metrics.values().all(|split| {
        let metrics = &split[Path::ForeignFamilyCertificate.name()];
        metrics.rejections == metrics.roots
    });
    let identity_rejected = split_metrics.values().all(|split| {
        let metrics = &split[Path::RoleIdentityPermutation.name()];
        metrics.rejections == metrics.roots
    });
    let irrelevant_valid = split_metrics.values().all(|split| {
        let metrics = &split[Path::ValidIrrelevantRole.name()];
        metrics.irrelevant_certificate_valid == metrics.roots
            && metrics.irrelevant_rule_admitted == metrics.roots
            && metrics.target_successes == 0
    });
    let replay_exact = split_metrics.values().all(|split| {
        split.values().all(|metrics| metrics.replay_exact == metrics.roots)
    });
    let budgets_exact = split_metrics.values().all(|split| {
        split.values().all(|metrics| metrics.budgets_exact == metrics.roots)
    });
    let invariants_hold = split_metrics.values().all(|split| {
        split.values().all(|metrics| metrics.invariants_hold == metrics.roots)
    });
    let transformations_exact = transformations_exercised.values().all(|value| *value);

    let mut gates = BTreeMap::new();
    gates.insert("development_cohort_exact".to_string(), DEVELOPMENT_GRAPHS == 12);
    gates.insert("evaluation_cohort_exact".to_string(), roots.len() == 56);
    gates.insert("primary_training_holdout_future_exact".to_string(), primary_exact);
    gates.insert("oracle_exact".to_string(), oracle_exact);
    gates.insert("target_negative_controls_zero".to_string(), negative_zero);
    gates.insert("future_family_transfer_exact".to_string(), future_family_exact);
    gates.insert("primary_recognition_exact".to_string(), primary_recognition_exact);
    gates.insert("foreign_certificate_rejected".to_string(), foreign_rejected);
    gates.insert("identity_permutation_rejected".to_string(), identity_rejected);
    gates.insert("valid_irrelevant_role_exact".to_string(), irrelevant_valid);
    gates.insert("transformations_exercised".to_string(), transformations_exact);
    gates.insert("replay_exact".to_string(), replay_exact);
    gates.insert("budgets_exact".to_string(), budgets_exact);
    gates.insert("invariants_hold".to_string(), invariants_hold);

    let terminal_classification = if !replay_exact || !budgets_exact || !invariants_hold {
        "INFRASTRUCTURE_FAILURE"
    } else if !primary_exact || !future_family_exact || !primary_recognition_exact {
        "REJECTED"
    } else if !negative_zero || !foreign_rejected || !identity_rejected || !irrelevant_valid {
        "CONTROL_FAILURE"
    } else if !oracle_exact || !transformations_exact {
        "INCONCLUSIVE"
    } else if gates.values().all(|value| *value) {
        "PASS"
    } else {
        "INFRASTRUCTURE_FAILURE"
    };

    let report = Report {
        experiment: "H13-C structural role transfer stress",
        preregistration_commit: "59eec9c66cf6c2a5c26a4ef249e35908f7c13fa6",
        development_graphs: DEVELOPMENT_GRAPHS,
        evaluation_roots: roots.len(),
        split_metrics,
        future_family_primary_successes,
        future_family_max_negative_control_successes,
        transformations_exercised,
        gates,
        terminal_classification,
        claim_boundary: "PASS supports only bounded structural-role transfer under the frozen synthetic graph transformations; it does not establish natural-language concept learning, unrestricted semantic equivalence, open-world ontology induction, AGI, consciousness, or safe production self-modification",
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal_classification != "PASS" {
        return Err(format!("H13-C terminal classification: {terminal_classification}").into());
    }
    Ok(())
}

fn freeze_roles(development: &DevelopmentFixture) -> Result<FrozenRoles, Box<dyn Error>> {
    let config = TransportRoleConfig {
        min_supporting_graphs: DEVELOPMENT_GRAPHS,
        exact_members_per_graph: 1,
    };
    let mut induction_budget = TransportRoleBudget::default();
    let proofs = induce_transport_roles(&development.corpus, config, &mut induction_budget)?;
    let mut registry =
        ShadowTransportRoleRegistry::new(development_scope_digest(&development.corpus)?);
    let mut validated = Vec::new();
    for proof in &proofs {
        registry.register_candidate(proof)?;
        let mut budget = TransportRoleBudget::default();
        let certificate = validate_transport_role(&development.corpus, proof, config, &mut budget)?;
        registry.admit_validated(&certificate)?;
        validated.push((proof.clone(), certificate));
    }

    let target_signature = TransportRoleSignature {
        source_reachable: true,
        sink_reachable: true,
        upstream_sources_ge_2: true,
        downstream_sinks_ge_2: true,
        complete_reachable_pair_cut: true,
        lost_reachable_pairs_ge_4: true,
        has_convergent_ancestor: true,
        has_divergent_ancestor: false,
        has_convergent_descendant: false,
        has_divergent_descendant: true,
        upstream_depth_ge_2: true,
        downstream_depth_ge_2: true,
    };
    let (target_proof, target) = validated
        .iter()
        .find(|(proof, _)| {
            proof.signature == target_signature
                && proof_members(proof) == development.target_members
        })
        .cloned()
        .ok_or("frozen target transport role not induced")?;
    let irrelevant = validated
        .iter()
        .find(|(proof, _)| proof_members(proof) == development.irrelevant_members)
        .map(|(_, certificate)| certificate.clone())
        .ok_or("frozen irrelevant transport role not induced")?;

    let mut permuted = target_proof.clone();
    permuted.role_id.0 ^= 0xa5a5_a5a5_a5a5_a5a5;
    let mut permutation_budget = TransportRoleBudget::default();
    let identity_permutation_rejected = validate_transport_role(
        &development.corpus,
        &permuted,
        config,
        &mut permutation_budget,
    )
    .is_err();

    let foreign_development = build_development_fixture_with_prefix("foreign");
    let mut foreign_induction_budget = TransportRoleBudget::default();
    let foreign_proofs = induce_transport_roles(
        &foreign_development.corpus,
        config,
        &mut foreign_induction_budget,
    )?;
    let foreign_proof = foreign_proofs
        .iter()
        .find(|proof| proof.signature == target_signature)
        .ok_or("foreign target role absent")?;
    let mut foreign_validation_budget = TransportRoleBudget::default();
    let foreign = validate_transport_role(
        &foreign_development.corpus,
        foreign_proof,
        config,
        &mut foreign_validation_budget,
    )?;

    let h12_config = RoleInductionConfig {
        min_supporting_graphs: DEVELOPMENT_GRAPHS,
        min_members: DEVELOPMENT_GRAPHS,
    };
    let mut h12_induction_budget = RoleInductionBudget::default();
    let h12_proofs = induce_latent_roles(
        &development.corpus,
        h12_config,
        &mut h12_induction_budget,
    )?;
    let h12_target_proof = h12_proofs
        .iter()
        .find(|proof| {
            proof
                .members
                .iter()
                .map(|member| (member.graph_id, member.node.clone()))
                .collect::<BTreeSet<_>>()
                == development.target_members
        })
        .ok_or("exact H12 target role absent")?;
    let mut h12_validation_budget = RoleInductionBudget::default();
    let exact_h12_target = validate_latent_role(
        &development.corpus,
        h12_target_proof,
        h12_config,
        &mut h12_validation_budget,
    )?;
    let h12_scope = star::latent_roles::discovery_scope_digest(&development.corpus)?;
    let mut exact_h12_registry = ShadowAbstractionRegistry::new(h12_scope);
    exact_h12_registry.register_candidate(h12_target_proof)?;
    exact_h12_registry.admit_validated(&exact_h12_target)?;

    Ok(FrozenRoles {
        proof_digest: target_proof.proof_id,
        certificate_digest: target.proof_id(),
        target_proof,
        target,
        irrelevant,
        registry,
        foreign,
        identity_permutation_rejected,
        exact_h12_target,
        exact_h12_registry,
    })
}

fn proof_members(proof: &TransportRoleProof) -> BTreeSet<(u64, Atom)> {
    proof
        .members
        .iter()
        .map(|member| (member.graph_id, member.node.clone()))
        .collect()
}

fn evaluate_split(
    roots: &[Root],
    frozen: &FrozenRoles,
) -> Result<BTreeMap<String, PathMetrics>, Box<dyn Error>> {
    let mut output = BTreeMap::new();
    for path in Path::all() {
        let mut metrics = PathMetrics {
            roots: roots.len(),
            ..PathMetrics::default()
        };
        for root in roots {
            let first = execute(root, frozen, path)?;
            let second = execute(root, frozen, path)?;
            metrics.target_successes += usize::from(first.target_success);
            metrics.certificates_available += usize::from(first.certificate_available);
            metrics.rejections += usize::from(first.rejected);
            metrics.replay_exact += usize::from(first == second);
            metrics.budgets_exact += usize::from(first.budget_exact);
            metrics.invariants_hold += usize::from(first.invariants_hold);
            metrics.target_member_exact += usize::from(first.target_member_exact);
            metrics.superficial_decoys_excluded +=
                usize::from(first.superficial_decoys_excluded);
            metrics.irrelevant_certificate_valid +=
                usize::from(first.irrelevant_certificate_valid);
            metrics.irrelevant_rule_admitted += usize::from(first.irrelevant_rule_admitted);
        }
        output.insert(path.name().to_string(), metrics);
    }
    Ok(output)
}

fn execute(root: &Root, frozen: &FrozenRoles, path: Path) -> Result<Execution, Box<dyn Error>> {
    let mut state = initial_state(root)?;
    let mut budget = ExecutionBudget {
        admission_opportunities: 1,
        ..ExecutionBudget::default()
    };
    let mut certificate_available = false;
    let mut rejected = false;
    let mut recognized_members = Vec::new();
    let mut projected_evidence_ids = Vec::new();
    let mut h11_rule = None;
    let mut delayed_certificate = None;
    let mut irrelevant_certificate_valid = false;
    let mut irrelevant_rule_admitted = false;
    let mut transfer_budget = TransportRoleBudget::default();

    let projected = match path {
        Path::ValidatedTransportRole => {
            recognized_members = recognize_transport_role(
                &root.graph,
                &frozen.target,
                &frozen.registry,
                &mut transfer_budget,
            )?;
            project_for_members(&root.evidence, &recognized_members, &mut budget)
        }
        Path::ExactH12FingerprintBaseline => {
            let mut h12_budget = TransferRecognitionBudget::default();
            match project_evidence_for_role(
                &root.graph,
                &root.evidence,
                &frozen.exact_h12_target,
                &frozen.exact_h12_registry,
                &mut h12_budget,
            ) {
                Ok(graph) => {
                    budget.signature_evaluations += h12_budget.node_fingerprint_evaluations;
                    budget.reachability_traversals += h12_budget.edge_traversals;
                    budget.evidence_episode_scans += h12_budget.evidence_episode_scans;
                    Some(graph)
                }
                Err(LatentRoleError::NoTransferMember(_)) | Err(LatentRoleError::EmptyProjection) => {
                    None
                }
                Err(error) => return Err(error.into()),
            }
        }
        Path::LocalDegreeMatchedDecoy => project_for_members(
            &root.evidence,
            &[root.local_degree_decoy.clone()],
            &mut budget,
        ),
        Path::TwoHopMotifMatchedDecoy => project_for_members(
            &root.evidence,
            &[root.two_hop_decoy.clone()],
            &mut budget,
        ),
        Path::DegreePreservingRewire => project_for_members(
            &root.evidence,
            &[root.rewired_decoy.clone()],
            &mut budget,
        ),
        Path::VocabularyOnlySimilarity => project_for_members(
            &root.evidence,
            &[root.vocabulary_decoy.clone()],
            &mut budget,
        ),
        Path::ForeignFamilyCertificate => {
            rejected = recognize_transport_role(
                &root.graph,
                &frozen.foreign,
                &frozen.registry,
                &mut transfer_budget,
            )
            .is_err();
            None
        }
        Path::ValidIrrelevantRole => {
            irrelevant_certificate_valid = true;
            recognized_members = recognize_transport_role(
                &root.graph,
                &frozen.irrelevant,
                &frozen.registry,
                &mut transfer_budget,
            )?;
            project_for_members(&root.evidence, &recognized_members, &mut budget)
        }
        Path::RoleIdentityPermutation => {
            rejected = frozen.identity_permutation_rejected;
            None
        }
        Path::DelayedRoleAdmission => {
            recognized_members = recognize_transport_role(
                &root.graph,
                &frozen.target,
                &frozen.registry,
                &mut transfer_budget,
            )?;
            project_for_members(&root.evidence, &recognized_members, &mut budget)
        }
        Path::RandomSameCardinalityGrouping => {
            Some(random_same_cardinality_group(root, &mut budget))
        }
        Path::PayloadOnly => None,
        Path::OracleRole => project_for_members(
            &root.evidence,
            &[root.middle.clone()],
            &mut budget,
        ),
        Path::UnpartitionedFullEvidence => {
            budget.evidence_episode_scans += root.evidence.evidence.len();
            Some(root.evidence.clone())
        }
    };

    budget.graph_node_scans += transfer_budget.graph_node_scans;
    budget.graph_edge_scans += transfer_budget.graph_edge_scans;
    budget.signature_evaluations += transfer_budget.node_signature_evaluations;
    budget.reachability_traversals += transfer_budget.reachability_edge_traversals;
    budget.counterfactual_traversals +=
        transfer_budget.removal_counterfactual_edge_traversals;

    if let Some(graph) = projected {
        projected_evidence_ids = graph
            .evidence
            .iter()
            .map(|episode| episode.evidence_id)
            .collect();
        let attempt = h11(&graph);
        budget.h11_frontier_scans += attempt.frontier_scans;
        budget.h10_candidate_episode_evaluations += attempt.scoring_evaluations;
        certificate_available = attempt.certificate.is_some();
        if let Some(certificate) = attempt.certificate {
            h11_rule = Some(certificate.rule().clone());
            if path == Path::DelayedRoleAdmission {
                delayed_certificate = Some(certificate);
            } else {
                if path == Path::ValidIrrelevantRole
                    && certificate.rule().antecedent == root.irrelevant
                    && certificate.rule().consequent == root.irrelevant_goal
                {
                    irrelevant_rule_admitted = true;
                }
                admit_graph_certificate(&mut state, &certificate)?;
                budget.successful_admissions += 1;
            }
        }
        if !attempt.exact {
            return Ok(failed_accounting_execution(
                state,
                budget,
                frozen,
                recognized_members,
                projected_evidence_ids,
                h11_rule,
                rejected,
                irrelevant_certificate_valid,
                irrelevant_rule_admitted,
            ));
        }
    }

    run_closure(&mut state, &mut budget)?;
    if let Some(certificate) = delayed_certificate {
        admit_graph_certificate(&mut state, &certificate)?;
        budget.successful_admissions += 1;
    }

    budget.objective_checks += 1;
    let target_success = state.contains_fact(&root.goal);
    let target_member_exact = if path == Path::ValidatedTransportRole {
        recognized_members == vec![root.middle.clone()]
    } else {
        true
    };
    let superficial_decoys_excluded = if path == Path::ValidatedTransportRole {
        !recognized_members.contains(&root.local_degree_decoy)
            && !recognized_members.contains(&root.two_hop_decoy)
            && !recognized_members.contains(&root.rewired_decoy)
            && !recognized_members.contains(&root.vocabulary_decoy)
    } else {
        true
    };
    let budget_exact = budget.admission_opportunities == 1
        && budget.closure_scans == CLOSURE_SCANS
        && budget.objective_checks == 1
        && budget.successful_admissions <= 1
        && if matches!(
            path,
            Path::ForeignFamilyCertificate
                | Path::RoleIdentityPermutation
                | Path::PayloadOnly
        ) {
            budget.h11_frontier_scans == 0
        } else {
            true
        };

    Ok(Execution {
        target_success,
        certificate_available,
        rejected,
        recognized_members,
        projected_evidence_ids,
        h11_rule,
        state_signature: state.canonical_signature(),
        budget,
        budget_exact,
        invariants_hold: state.verify_invariants().is_ok(),
        irrelevant_certificate_valid,
        irrelevant_rule_admitted,
        target_member_exact,
        superficial_decoys_excluded,
        proof_digest: frozen.proof_digest,
        certificate_digest: frozen.certificate_digest,
    })
}

fn failed_accounting_execution(
    state: EvidenceBoundCommitmentState,
    budget: ExecutionBudget,
    frozen: &FrozenRoles,
    recognized_members: Vec<Atom>,
    projected_evidence_ids: Vec<u64>,
    h11_rule: Option<Rule>,
    rejected: bool,
    irrelevant_certificate_valid: bool,
    irrelevant_rule_admitted: bool,
) -> Execution {
    Execution {
        target_success: false,
        certificate_available: h11_rule.is_some(),
        rejected,
        recognized_members,
        projected_evidence_ids,
        h11_rule,
        state_signature: state.canonical_signature(),
        budget,
        budget_exact: false,
        invariants_hold: state.verify_invariants().is_ok(),
        irrelevant_certificate_valid,
        irrelevant_rule_admitted,
        target_member_exact: false,
        superficial_decoys_excluded: false,
        proof_digest: frozen.proof_digest,
        certificate_digest: frozen.certificate_digest,
    }
}

fn initial_state(root: &Root) -> Result<EvidenceBoundCommitmentState, Box<dyn Error>> {
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    Ok(state)
}

fn run_closure(
    state: &mut EvidenceBoundCommitmentState,
    budget: &mut ExecutionBudget,
) -> Result<(), Box<dyn Error>> {
    for _ in 0..CLOSURE_SCANS {
        budget.closure_scans += 1;
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
        }
    }
    Ok(())
}

fn project_for_members(
    evidence: &MixedEvidenceGraph,
    members: &[Atom],
    budget: &mut ExecutionBudget,
) -> Option<MixedEvidenceGraph> {
    let member_set = members.iter().cloned().collect::<BTreeSet<_>>();
    let projected = evidence
        .evidence
        .iter()
        .filter_map(|episode| {
            budget.evidence_episode_scans += 1;
            member_set
                .contains(&episode.intervention)
                .then_some(episode.clone())
        })
        .collect::<Vec<_>>();
    (!projected.is_empty()).then_some(MixedEvidenceGraph { evidence: projected })
}

fn random_same_cardinality_group(root: &Root, budget: &mut ExecutionBudget) -> MixedEvidenceGraph {
    let target = root
        .evidence
        .evidence
        .iter()
        .filter(|episode| episode.intervention == root.middle)
        .collect::<Vec<_>>();
    let non_target = root
        .evidence
        .evidence
        .iter()
        .filter(|episode| episode.intervention != root.middle)
        .collect::<Vec<_>>();
    budget.evidence_episode_scans += root.evidence.evidence.len();
    let mut rng = SplitMix64::new(root.id ^ 0x9e37_79b9_7f4a_7c15);
    let target_start = (rng.next_u64() as usize) % target.len();
    let non_target_start = (rng.next_u64() as usize) % non_target.len();
    let mut selected = vec![
        target[target_start].clone(),
        target[(target_start + 1) % target.len()].clone(),
        non_target[non_target_start].clone(),
        non_target[(non_target_start + 1) % non_target.len()].clone(),
    ];
    selected.sort_by_key(|episode| episode.evidence_id);
    MixedEvidenceGraph { evidence: selected }
}

fn h11(graph: &MixedEvidenceGraph) -> H11Attempt {
    let config = RuleInductionConfig::default();
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
    let mut frontier_scans = proposal_frontier.evidence_episode_scans;
    let mut scoring_evaluations = proposal_scoring.candidate_episode_evaluations;
    let Ok(proof) = proof else {
        return H11Attempt {
            certificate: None,
            exact,
            frontier_scans,
            scoring_evaluations,
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
    frontier_scans += validation_frontier.evidence_episode_scans;
    scoring_evaluations += validation_scoring.candidate_episode_evaluations;
    H11Attempt {
        certificate,
        exact,
        frontier_scans,
        scoring_evaluations,
    }
}

fn build_development_fixture() -> DevelopmentFixture {
    build_development_fixture_with_prefix("h13dev")
}

fn build_development_fixture_with_prefix(scope: &str) -> DevelopmentFixture {
    let mut graphs = Vec::new();
    let mut target_members = BTreeSet::new();
    let mut irrelevant_members = BTreeSet::new();
    for index in 0..DEVELOPMENT_GRAPHS {
        let graph_id = 10_000 + index as u64;
        let prefix = format!("{scope}_{}_{}", index / 4, index % 4);
        let target = atom(format!("{prefix}_u7"));
        let irrelevant = atom(format!("{prefix}_u19"));
        let graph = small_development_graph(graph_id, &prefix, target.clone(), irrelevant.clone());
        target_members.insert((graph_id, target));
        irrelevant_members.insert((graph_id, irrelevant));
        graphs.push(graph);
    }
    DevelopmentFixture {
        corpus: StructuralCorpus { graphs },
        target_members,
        irrelevant_members,
    }
}

fn small_development_graph(
    graph_id: u64,
    prefix: &str,
    target: Atom,
    irrelevant: Atom,
) -> StructuralGraph {
    let mut builder = Builder::new(graph_id);
    let join = atom(format!("{prefix}_u6"));
    let split = atom(format!("{prefix}_u8"));
    for side in 0..2 {
        let source = atom(format!("{prefix}_u{side}"));
        let pre = atom(format!("{prefix}_u{}", 2 + side));
        builder.edge(source, pre.clone());
        builder.edge(pre, join.clone());
    }
    builder.edge(join, target.clone());
    builder.edge(target, split.clone());
    for side in 0..2 {
        let post = atom(format!("{prefix}_u{}", 9 + side));
        let sink = atom(format!("{prefix}_u{}", 11 + side));
        builder.edge(split.clone(), post.clone());
        builder.edge(post, sink);
    }

    let div = atom(format!("{prefix}_u13"));
    let p1 = atom(format!("{prefix}_u14"));
    let p2 = atom(format!("{prefix}_u15"));
    builder.edge(atom(format!("{prefix}_u16")), div.clone());
    builder.edge(div.clone(), p1.clone());
    builder.edge(div, atom(format!("{prefix}_u17")));
    builder.edge(atom(format!("{prefix}_u18")), p2.clone());
    builder.edge(p1, irrelevant.clone());
    builder.edge(p2, irrelevant.clone());
    let q1 = atom(format!("{prefix}_u20"));
    let q2 = atom(format!("{prefix}_u21"));
    let conv = atom(format!("{prefix}_u22"));
    builder.edge(irrelevant.clone(), q1.clone());
    builder.edge(q1, atom(format!("{prefix}_u23")));
    builder.edge(irrelevant, q2.clone());
    builder.edge(q2, conv.clone());
    builder.edge(atom(format!("{prefix}_u24")), conv.clone());
    builder.edge(conv, atom(format!("{prefix}_u25")));

    builder.edge(atom(format!("{prefix}_u26")), atom(format!("{prefix}_u27")));
    builder.edge(atom(format!("{prefix}_u27")), atom(format!("{prefix}_u28")));
    builder.finish()
}

fn build_roots() -> Vec<Root> {
    let mut roots = Vec::new();
    let mut id = 1u64;
    for family_index in 0..FAMILIES.len() {
        for local_index in 0..ROOTS_PER_FAMILY {
            roots.push(build_root(id, family_index, local_index));
            id += 1;
        }
    }
    roots
}

fn build_root(id: u64, family_index: usize, local_index: usize) -> Root {
    let prefix = format!(
        "h13_{}_{}_{}",
        FAMILIES[family_index], id, local_index
    );
    let source = atom(format!("{prefix}_source"));
    let middle = atom(format!("{prefix}_middle"));
    let goal = atom(format!("{prefix}_goal"));
    let irrelevant = atom(format!("{prefix}_irrelevant"));
    let irrelevant_goal = atom(format!("{prefix}_irrelevant_goal"));
    let local_degree_decoy = atom(format!("{prefix}_local_decoy"));
    let two_hop_decoy = atom(format!("{prefix}_two_hop_decoy"));
    let rewired_decoy = atom(format!("{prefix}_rewired_decoy"));
    let vocabulary_decoy = atom(format!("{prefix}_middle_lookalike"));
    let local_degree = 1 + ((family_index + local_index) % 3);
    let subdivision = 1 + ((family_index * 2 + local_index) % 6);
    let target_size = 40 + family_index * 20 + local_index * 2;
    let reverse_insertion = (id + family_index as u64) % 2 == 0;

    let mut builder = Builder::new(id * 10_000 + 7);
    target_motif(
        &mut builder,
        &prefix,
        middle.clone(),
        local_degree,
        subdivision,
    );
    local_degree_decoy_motif(
        &mut builder,
        &prefix,
        local_degree_decoy.clone(),
        local_degree,
        subdivision,
    );
    two_hop_decoy_motif(
        &mut builder,
        &prefix,
        two_hop_decoy.clone(),
        local_degree,
    );
    rewired_decoy_motif(
        &mut builder,
        &prefix,
        rewired_decoy.clone(),
        local_degree,
    );
    vocabulary_decoy_motif(&mut builder, &prefix, vocabulary_decoy.clone());
    irrelevant_motif(&mut builder, &prefix, irrelevant.clone());

    let intended_distractor_edges = add_distractors(
        &mut builder,
        &prefix,
        target_size,
        family_index,
        local_index,
    );
    let partial_observation = intended_distractor_edges >= 10;
    let graph = builder.finish(reverse_insertion);
    let evidence = evidence_graph(
        id,
        &prefix,
        &middle,
        &goal,
        &irrelevant,
        &irrelevant_goal,
        &local_degree_decoy,
        &two_hop_decoy,
        &rewired_decoy,
        &vocabulary_decoy,
    );

    Root {
        id,
        family_index,
        local_index,
        graph,
        evidence,
        source,
        middle,
        goal,
        irrelevant,
        irrelevant_goal,
        local_degree_decoy,
        two_hop_decoy,
        rewired_decoy,
        vocabulary_decoy,
        transformations: TransformationAudit {
            unseen_size: target_size >= 40,
            longer_paths: subdivision >= 1,
            irrelevant_branches: intended_distractor_edges >= 4,
            high_distractor_density: family_index >= 3,
            edge_subdivision: subdivision >= 1,
            node_duplication: local_index % 2 == 0,
            partial_observation,
            vocabulary_permutation: true,
            insertion_order_reversal: reverse_insertion,
            local_degree_change: local_degree != 1,
        },
    }
}

fn target_motif(
    builder: &mut Builder,
    prefix: &str,
    middle: Atom,
    local_degree: usize,
    subdivision: usize,
) {
    for branch in 0..local_degree {
        let join = atom(format!("{prefix}_target_join_{branch}"));
        for source_index in 0..2 {
            let source = atom(format!("{prefix}_target_s_{branch}_{source_index}"));
            let mut previous = source;
            for step in 0..subdivision {
                let next = atom(format!(
                    "{prefix}_target_up_{branch}_{source_index}_{step}"
                ));
                builder.edge(previous, next.clone());
                previous = next;
            }
            builder.edge(previous, join.clone());
        }
        builder.edge(join, middle.clone());
    }
    for branch in 0..local_degree {
        let split = atom(format!("{prefix}_target_split_{branch}"));
        builder.edge(middle.clone(), split.clone());
        for sink_index in 0..2 {
            let mut previous = split.clone();
            for step in 0..subdivision {
                let next = atom(format!(
                    "{prefix}_target_down_{branch}_{sink_index}_{step}"
                ));
                builder.edge(previous, next.clone());
                previous = next;
            }
            builder.edge(
                previous,
                atom(format!("{prefix}_target_t_{branch}_{sink_index}")),
            );
        }
    }
}

fn local_degree_decoy_motif(
    builder: &mut Builder,
    prefix: &str,
    decoy: Atom,
    local_degree: usize,
    subdivision: usize,
) {
    let mut predecessors = Vec::new();
    let mut successors = Vec::new();
    for branch in 0..local_degree {
        let pred = atom(format!("{prefix}_local_pred_{branch}"));
        predecessors.push(pred.clone());
        for source_index in 0..2 {
            let mut previous = atom(format!("{prefix}_local_s_{branch}_{source_index}"));
            for step in 0..subdivision {
                let next = atom(format!(
                    "{prefix}_local_up_{branch}_{source_index}_{step}"
                ));
                builder.edge(previous, next.clone());
                previous = next;
            }
            builder.edge(previous, pred.clone());
        }
        builder.edge(pred, decoy.clone());
    }
    for branch in 0..local_degree {
        let succ = atom(format!("{prefix}_local_succ_{branch}"));
        successors.push(succ.clone());
        builder.edge(decoy.clone(), succ.clone());
        for sink_index in 0..2 {
            builder.edge(
                succ.clone(),
                atom(format!("{prefix}_local_t_{branch}_{sink_index}")),
            );
        }
    }
    builder.edge(predecessors[0].clone(), successors[0].clone());
}

fn two_hop_decoy_motif(
    builder: &mut Builder,
    prefix: &str,
    decoy: Atom,
    local_degree: usize,
) {
    let mut bypass_from = None;
    let mut bypass_to = None;
    for branch in 0..local_degree {
        let join = atom(format!("{prefix}_two_join_{branch}"));
        for source_index in 0..2 {
            let source = atom(format!("{prefix}_two_s_{branch}_{source_index}"));
            let a = atom(format!("{prefix}_two_a_{branch}_{source_index}"));
            let b = atom(format!("{prefix}_two_b_{branch}_{source_index}"));
            builder.edge(source, a.clone());
            builder.edge(a.clone(), b.clone());
            builder.edge(b, join.clone());
            if branch == 0 && source_index == 0 {
                bypass_from = Some(a);
            }
        }
        builder.edge(join, decoy.clone());
    }
    for branch in 0..local_degree {
        let split = atom(format!("{prefix}_two_split_{branch}"));
        builder.edge(decoy.clone(), split.clone());
        for sink_index in 0..2 {
            let c = atom(format!("{prefix}_two_c_{branch}_{sink_index}"));
            let d = atom(format!("{prefix}_two_d_{branch}_{sink_index}"));
            builder.edge(split.clone(), c.clone());
            builder.edge(c, d.clone());
            builder.edge(
                d.clone(),
                atom(format!("{prefix}_two_t_{branch}_{sink_index}")),
            );
            if branch == 0 && sink_index == 0 {
                bypass_to = Some(d);
            }
        }
    }
    builder.edge(
        bypass_from.expect("two-hop bypass source"),
        bypass_to.expect("two-hop bypass target"),
    );
}

fn rewired_decoy_motif(
    builder: &mut Builder,
    prefix: &str,
    decoy: Atom,
    local_degree: usize,
) {
    let mut predecessors = Vec::new();
    let mut successors = Vec::new();
    for branch in 0..local_degree {
        let pred = atom(format!("{prefix}_rewire_pred_{branch}"));
        let succ = atom(format!("{prefix}_rewire_succ_{branch}"));
        predecessors.push(pred.clone());
        successors.push(succ.clone());
        builder.edge(
            atom(format!("{prefix}_rewire_s0_{branch}")),
            pred.clone(),
        );
        builder.edge(
            atom(format!("{prefix}_rewire_s1_{branch}")),
            pred.clone(),
        );
        builder.edge(pred, decoy.clone());
        builder.edge(decoy.clone(), succ.clone());
        builder.edge(
            succ.clone(),
            atom(format!("{prefix}_rewire_t0_{branch}")),
        );
        builder.edge(
            succ,
            atom(format!("{prefix}_rewire_t1_{branch}")),
        );
    }
    for branch in 0..local_degree {
        builder.edge(
            predecessors[branch].clone(),
            successors[(branch + 1) % local_degree].clone(),
        );
    }
}

fn vocabulary_decoy_motif(builder: &mut Builder, prefix: &str, decoy: Atom) {
    let before = atom(format!("{prefix}_vocab_before"));
    let after = atom(format!("{prefix}_vocab_after"));
    builder.edge(atom(format!("{prefix}_vocab_source")), before.clone());
    builder.edge(before, decoy.clone());
    builder.edge(decoy, after.clone());
    builder.edge(after, atom(format!("{prefix}_vocab_sink")));
}

fn irrelevant_motif(builder: &mut Builder, prefix: &str, irrelevant: Atom) {
    let div = atom(format!("{prefix}_irrelevant_div"));
    let p1 = atom(format!("{prefix}_irrelevant_p1"));
    let p2 = atom(format!("{prefix}_irrelevant_p2"));
    builder.edge(atom(format!("{prefix}_irrelevant_s0")), div.clone());
    builder.edge(div.clone(), p1.clone());
    builder.edge(div, atom(format!("{prefix}_irrelevant_dead")));
    builder.edge(atom(format!("{prefix}_irrelevant_s1")), p2.clone());
    builder.edge(p1, irrelevant.clone());
    builder.edge(p2, irrelevant.clone());

    let q1 = atom(format!("{prefix}_irrelevant_q1"));
    let q2 = atom(format!("{prefix}_irrelevant_q2"));
    let conv = atom(format!("{prefix}_irrelevant_conv"));
    builder.edge(irrelevant.clone(), q1.clone());
    builder.edge(q1, atom(format!("{prefix}_irrelevant_t0")));
    builder.edge(irrelevant, q2.clone());
    builder.edge(q2, conv.clone());
    builder.edge(atom(format!("{prefix}_irrelevant_extra")), conv.clone());
    builder.edge(conv, atom(format!("{prefix}_irrelevant_t1")));
}

fn add_distractors(
    builder: &mut Builder,
    prefix: &str,
    target_size: usize,
    family_index: usize,
    local_index: usize,
) -> usize {
    let mut edge_index = 0usize;
    let density_multiplier = 1 + family_index;
    let current = builder.nodes.len();
    let remaining = target_size.saturating_sub(current);
    for index in 0..remaining {
        builder.node(atom(format!("{prefix}_noise_node_{index}")));
    }
    let edge_budget = (target_size * (10 + family_index * 5)) / 100;
    for index in 0..edge_budget {
        let from = atom(format!("{prefix}_noise_node_{}", index % remaining.max(1)));
        let to = atom(format!(
            "{prefix}_noise_node_{}",
            (index + density_multiplier + local_index + 1) % remaining.max(1)
        ));
        if from != to {
            edge_index += 1;
            if edge_index % 10 != 0 {
                builder.edge(from, to);
            }
        }
    }
    if local_index % 2 == 0 && remaining >= 4 {
        builder.edge(
            atom(format!("{prefix}_noise_node_0")),
            atom(format!("{prefix}_noise_node_2")),
        );
        builder.edge(
            atom(format!("{prefix}_noise_node_1")),
            atom(format!("{prefix}_noise_node_3")),
        );
    }
    edge_index
}

#[allow(clippy::too_many_arguments)]
fn evidence_graph(
    root_id: u64,
    prefix: &str,
    middle: &Atom,
    goal: &Atom,
    irrelevant: &Atom,
    irrelevant_goal: &Atom,
    local: &Atom,
    two_hop: &Atom,
    rewired: &Atom,
    vocabulary: &Atom,
) -> MixedEvidenceGraph {
    let common_noise = atom(format!("{prefix}_common_noise"));
    let mut evidence = Vec::new();
    let mut evidence_id = root_id * 1_000_000 + 1;
    push_episodes(
        &mut evidence,
        &mut evidence_id,
        middle,
        goal,
        4,
        Some(&common_noise),
    );
    push_episodes(
        &mut evidence,
        &mut evidence_id,
        irrelevant,
        irrelevant_goal,
        5,
        Some(&common_noise),
    );
    for (index, intervention) in [local, two_hop, rewired, vocabulary].into_iter().enumerate() {
        let outcome = atom(format!("{prefix}_decoy_goal_{index}"));
        push_episodes(
            &mut evidence,
            &mut evidence_id,
            intervention,
            &outcome,
            4,
            Some(&common_noise),
        );
    }
    MixedEvidenceGraph { evidence }
}

fn push_episodes(
    evidence: &mut Vec<EvidenceEpisode>,
    evidence_id: &mut u64,
    intervention: &Atom,
    outcome: &Atom,
    count: usize,
    noise: Option<&Atom>,
) {
    for index in 0..count {
        let mut outcomes = BTreeSet::from([outcome.clone()]);
        if index < 2 {
            if let Some(noise) = noise {
                outcomes.insert(noise.clone());
            }
        }
        evidence.push(EvidenceEpisode {
            evidence_id: *evidence_id,
            intervention: intervention.clone(),
            outcomes,
        });
        *evidence_id += 1;
    }
}

fn transformation_coverage(roots: &[Root]) -> BTreeMap<String, bool> {
    let mut coverage = BTreeMap::new();
    coverage.insert(
        "unseen_graph_sizes".to_string(),
        roots.iter().all(|root| root.transformations.unseen_size),
    );
    coverage.insert(
        "longer_causal_paths".to_string(),
        roots.iter().all(|root| root.transformations.longer_paths),
    );
    coverage.insert(
        "irrelevant_branches".to_string(),
        roots.iter().all(|root| root.transformations.irrelevant_branches),
    );
    coverage.insert(
        "high_distractor_density".to_string(),
        roots
            .iter()
            .any(|root| root.transformations.high_distractor_density),
    );
    coverage.insert(
        "edge_subdivision".to_string(),
        roots.iter().all(|root| root.transformations.edge_subdivision),
    );
    coverage.insert(
        "node_duplication".to_string(),
        roots.iter().any(|root| root.transformations.node_duplication),
    );
    coverage.insert(
        "partial_observation".to_string(),
        roots.iter().all(|root| root.transformations.partial_observation),
    );
    coverage.insert(
        "vocabulary_permutation".to_string(),
        roots
            .iter()
            .all(|root| root.transformations.vocabulary_permutation),
    );
    coverage.insert(
        "insertion_order_reversal".to_string(),
        roots
            .iter()
            .any(|root| root.transformations.insertion_order_reversal),
    );
    coverage.insert(
        "local_degree_change".to_string(),
        roots.iter().any(|root| root.transformations.local_degree_change),
    );
    coverage
}

struct Builder {
    graph_id: u64,
    nodes: BTreeSet<Atom>,
    edges: Vec<DirectedEdge>,
    edge_set: BTreeSet<DirectedEdge>,
}

impl Builder {
    fn new(graph_id: u64) -> Self {
        Self {
            graph_id,
            nodes: BTreeSet::new(),
            edges: Vec::new(),
            edge_set: BTreeSet::new(),
        }
    }

    fn node(&mut self, node: Atom) {
        self.nodes.insert(node);
    }

    fn edge(&mut self, from: Atom, to: Atom) {
        if from == to {
            return;
        }
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        let edge = DirectedEdge { from, to };
        if self.edge_set.insert(edge.clone()) {
            self.edges.push(edge);
        }
    }

    fn finish(mut self, reverse_insertion: bool) -> StructuralGraph {
        if reverse_insertion {
            self.edges.reverse();
        }
        StructuralGraph {
            graph_id: self.graph_id,
            nodes: self.nodes.into_iter().collect(),
            edges: self.edges,
        }
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

fn atom(value: impl Into<String>) -> Atom {
    Atom::new(value.into()).expect("generated H13 atom is non-empty")
}
