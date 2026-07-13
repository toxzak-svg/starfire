use serde::Serialize;
use star::commitment_state::Atom;
use star::latent_roles::{
    induce_latent_roles, recognize_role_members, validate_latent_role, DirectedEdge,
    LatentRoleError, LatentRoleProof, RoleInductionBudget, RoleInductionConfig,
    ShadowAbstractionRegistry, StructuralCorpus, StructuralGraph, TransferRecognitionBudget,
};
use star::structural_transfer::{
    propose_transport_role, recognize_transport_members, validate_transport_role,
    ShadowTransportRegistry, TransportBudget, TransportError, TransportRoleProof,
    ValidatedTransportRoleCertificate,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::Path;

const DEVELOPMENT_GRAPHS: usize = 8;
const HOLDOUT_GRAPHS: usize = 8;
const ROOTS_PER_FAMILY: usize = 8;
const FUTURE_FAMILIES: usize = 7;
const FUTURE_GRAPHS: usize = ROOTS_PER_FAMILY * FUTURE_FAMILIES;
const RANDOM_SALT: u64 = 0x6a09_e667_f3bc_c909;
const FUTURE_FAMILY_NAMES: [&str; FUTURE_FAMILIES] = [
    "path_subdivision",
    "irrelevant_branch_expansion",
    "higher_distractor_density",
    "local_degree_change",
    "bounded_partial_observation",
    "vocabulary_permutation",
    "combined_stress",
];

#[derive(Debug, Clone, Copy)]
enum Transform {
    Development,
    Holdout,
    PathSubdivision,
    IrrelevantBranchExpansion,
    HigherDistractorDensity,
    LocalDegreeChange,
    BoundedPartialObservation,
    VocabularyPermutation,
    CombinedStress,
}

impl Transform {
    fn path_subdivision(self) -> bool {
        matches!(
            self,
            Self::Holdout | Self::PathSubdivision | Self::CombinedStress
        )
    }

    fn branch_expansion(self) -> bool {
        matches!(
            self,
            Self::Holdout
                | Self::IrrelevantBranchExpansion
                | Self::BoundedPartialObservation
                | Self::CombinedStress
        )
    }

    fn high_distractors(self) -> bool {
        matches!(self, Self::HigherDistractorDensity | Self::CombinedStress)
    }

    fn local_degree_change(self) -> bool {
        matches!(self, Self::LocalDegreeChange | Self::CombinedStress)
    }

    fn vocabulary_permutation(self) -> bool {
        matches!(self, Self::VocabularyPermutation | Self::CombinedStress)
    }
}

#[derive(Debug, Clone)]
struct GraphCase {
    graph: StructuralGraph,
    target: Atom,
    target_sources: Vec<Atom>,
    target_sinks: Vec<Atom>,
    local_degree_decoy: Atom,
    two_hop_decoy: Atom,
    vocabulary_control: Atom,
}

#[derive(Debug, Clone, Default)]
struct GraphDraft {
    nodes: BTreeSet<Atom>,
    edges: BTreeSet<(Atom, Atom)>,
}

impl GraphDraft {
    fn add_node(&mut self, node: Atom) {
        self.nodes.insert(node);
    }

    fn add_edge(&mut self, from: &Atom, to: &Atom) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        self.edges.insert((from.clone(), to.clone()));
    }

    fn finish(self, graph_id: u64) -> StructuralGraph {
        StructuralGraph {
            graph_id,
            nodes: self.nodes.into_iter().collect(),
            edges: self
                .edges
                .into_iter()
                .map(|(from, to)| DirectedEdge { from, to })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct SplitMetrics {
    graphs: usize,
    exact_role_successes: usize,
    false_positive_members: usize,
    local_degree_decoy_selections: usize,
    two_hop_decoy_selections: usize,
    vocabulary_control_selections: usize,
    graph_budget_passes: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
struct ControlMetrics {
    rewired_control_selections: usize,
    vocabulary_invariance_successes: usize,
    irrelevant_role_target_selections: usize,
    random_control_exact_successes: usize,
    oracle_successes: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
struct BudgetTotals {
    node_signature_evaluations: usize,
    reachability_edge_traversals: usize,
    candidate_matches: usize,
    graph_budget_passes: usize,
    graph_budget_checks: usize,
}

#[derive(Debug, Clone, Serialize)]
struct Report {
    experiment: &'static str,
    program_alias: &'static str,
    preregistration: &'static str,
    preregistration_commit: &'static str,
    development_graphs: usize,
    development_h12_members: usize,
    transport_proof_members_recomputed: usize,
    holdout: SplitMetrics,
    future: SplitMetrics,
    future_family_successes: BTreeMap<String, usize>,
    controls: ControlMetrics,
    foreign_certificate_rejected: bool,
    tampered_proof_rejected: bool,
    delayed_admission_rejected: bool,
    replay_byte_identical: bool,
    source_corpus_immutable: bool,
    shadow_authority_invariants: bool,
    budgets: BudgetTotals,
    gates: BTreeMap<String, bool>,
    terminal_classification: String,
    claim_boundary: &'static str,
}

struct PreparedMechanism {
    development: StructuralCorpus,
    source_proof: LatentRoleProof,
    transport_proof: TransportRoleProof,
    certificate: ValidatedTransportRoleCertificate,
    registry: ShadowTransportRegistry,
    irrelevant_proof: LatentRoleProof,
    irrelevant_registry: ShadowAbstractionRegistry,
    proposal_budget: TransportBudget,
    validation_budget: TransportBudget,
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run_experiment()?;
    let first_without_replay = serde_json::to_string_pretty(&first)?;
    let second = run_experiment()?;
    let second_without_replay = serde_json::to_string_pretty(&second)?;
    let replay_byte_identical = first_without_replay == second_without_replay;

    let mut final_report = first;
    final_report.replay_byte_identical = replay_byte_identical;
    final_report
        .gates
        .insert("replay_byte_identical".to_string(), replay_byte_identical);
    final_report.terminal_classification = classify(&final_report).to_string();

    let rendered = serde_json::to_string_pretty(&final_report)?;
    println!("{rendered}");
    let output = Path::new("target/h13c-structural-role-transfer-report.json");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, format!("{rendered}\n"))?;

    if final_report.terminal_classification != "PASS" {
        return Err(format!(
            "H13-C terminal classification: {}",
            final_report.terminal_classification
        )
        .into());
    }
    Ok(())
}

fn run_experiment() -> Result<Report, Box<dyn Error>> {
    let mut prepared = prepare_mechanism(0, "development")?;
    let source_snapshot = prepared.development.clone();

    let holdout_cases = (0..HOLDOUT_GRAPHS)
        .map(|seed| {
            build_case(
                10_000 + seed as u64,
                &format!("holdout_{seed}"),
                Transform::Holdout,
                seed as u64,
            )
        })
        .collect::<Vec<_>>();

    let future_cases = FUTURE_FAMILY_NAMES
        .iter()
        .enumerate()
        .flat_map(|(family_index, family)| {
            (0..ROOTS_PER_FAMILY).map(move |seed| {
                let transform = match family_index {
                    0 => Transform::PathSubdivision,
                    1 => Transform::IrrelevantBranchExpansion,
                    2 => Transform::HigherDistractorDensity,
                    3 => Transform::LocalDegreeChange,
                    4 => Transform::BoundedPartialObservation,
                    5 => Transform::VocabularyPermutation,
                    6 => Transform::CombinedStress,
                    _ => unreachable!(),
                };
                build_case(
                    20_000 + (family_index * ROOTS_PER_FAMILY + seed) as u64,
                    &format!("{family}_{seed}"),
                    transform,
                    (family_index * ROOTS_PER_FAMILY + seed) as u64,
                )
            })
        })
        .collect::<Vec<_>>();

    let mut totals = BudgetTotals::default();
    let holdout = evaluate_split(
        &holdout_cases,
        &prepared.certificate,
        &prepared.registry,
        &mut totals,
    )?;
    let future = evaluate_split(
        &future_cases,
        &prepared.certificate,
        &prepared.registry,
        &mut totals,
    )?;

    let mut family_successes = BTreeMap::new();
    for (family_index, family_name) in FUTURE_FAMILY_NAMES.iter().enumerate() {
        let start = family_index * ROOTS_PER_FAMILY;
        let end = start + ROOTS_PER_FAMILY;
        let successes = future_cases[start..end]
            .iter()
            .map(|case| {
                let mut budget = TransportBudget::default();
                recognize_transport_members(
                    &case.graph,
                    &prepared.certificate,
                    &prepared.registry,
                    &mut budget,
                )
                .map(|members| usize::from(members == vec![case.target.clone()]))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sum();
        family_successes.insert((*family_name).to_string(), successes);
    }

    let controls = evaluate_controls(&future_cases, &prepared, &mut totals)?;
    let foreign_certificate_rejected = foreign_certificate_control(&holdout_cases[0], &prepared)?;
    let tampered_proof_rejected = tampered_proof_control(&prepared);
    let delayed_admission_rejected = delayed_admission_control(&holdout_cases[0], &prepared)?;

    prepared.registry.mark_transfer_validated(&prepared.certificate)?;
    let shadow_authority_invariants = prepared
        .registry
        .entries()
        .all(|entry| {
            matches!(
                entry.status,
                star::structural_transfer::TransportStatus::ValidatedShadow
                    | star::structural_transfer::TransportStatus::TransferValidated
            )
        }) && prepared.registry.entries().count() == 1;
    let source_corpus_immutable = source_snapshot == prepared.development;

    let mut gates = BTreeMap::new();
    gates.insert(
        "development_h12_proof_validated".to_string(),
        prepared.certificate.supporting_member_count() == DEVELOPMENT_GRAPHS,
    );
    gates.insert(
        "transport_proof_recomputed_exactly".to_string(),
        prepared.validation_budget.proof_members_recomputed == DEVELOPMENT_GRAPHS,
    );
    gates.insert(
        "holdout_exact_8_of_8".to_string(),
        holdout.exact_role_successes == HOLDOUT_GRAPHS,
    );
    gates.insert(
        "future_exact_56_of_56".to_string(),
        future.exact_role_successes == FUTURE_GRAPHS,
    );
    gates.insert(
        "future_false_positives_zero".to_string(),
        future.false_positive_members == 0,
    );
    gates.insert(
        "local_degree_decoys_zero".to_string(),
        future.local_degree_decoy_selections == 0,
    );
    gates.insert(
        "two_hop_decoys_zero".to_string(),
        future.two_hop_decoy_selections == 0,
    );
    gates.insert(
        "rewired_controls_zero".to_string(),
        controls.rewired_control_selections == 0,
    );
    gates.insert(
        "vocabulary_invariance_56_of_56".to_string(),
        controls.vocabulary_invariance_successes == FUTURE_GRAPHS,
    );
    gates.insert(
        "foreign_certificate_rejected".to_string(),
        foreign_certificate_rejected,
    );
    gates.insert(
        "irrelevant_role_target_selections_zero".to_string(),
        controls.irrelevant_role_target_selections == 0,
    );
    gates.insert(
        "tampered_proof_rejected".to_string(),
        tampered_proof_rejected,
    );
    gates.insert(
        "delayed_admission_rejected".to_string(),
        delayed_admission_rejected,
    );
    gates.insert(
        "random_control_exact_zero".to_string(),
        controls.random_control_exact_successes == 0,
    );
    gates.insert(
        "oracle_56_of_56".to_string(),
        controls.oracle_successes == FUTURE_GRAPHS,
    );
    gates.insert(
        "future_families_8_of_8".to_string(),
        family_successes
            .values()
            .all(|successes| *successes == ROOTS_PER_FAMILY),
    );
    gates.insert(
        "source_corpus_immutable".to_string(),
        source_corpus_immutable,
    );
    gates.insert(
        "shadow_authority_invariants".to_string(),
        shadow_authority_invariants,
    );
    gates.insert(
        "all_graph_budgets_pass".to_string(),
        totals.graph_budget_passes == totals.graph_budget_checks,
    );
    gates.insert("replay_byte_identical".to_string(), false);

    let mut report = Report {
        experiment: "H13-C structural role transfer stress",
        program_alias: "R1-A",
        preregistration: "docs/experiments/H13C_STRUCTURAL_ROLE_TRANSFER_STRESS.md",
        preregistration_commit: "92f2cdeaf3ad4cba85b61a5443c0d470092413ab",
        development_graphs: DEVELOPMENT_GRAPHS,
        development_h12_members: prepared.source_proof.members.len(),
        transport_proof_members_recomputed: prepared.validation_budget.proof_members_recomputed,
        holdout,
        future,
        future_family_successes: family_successes,
        controls,
        foreign_certificate_rejected,
        tampered_proof_rejected,
        delayed_admission_rejected,
        replay_byte_identical: false,
        source_corpus_immutable,
        shadow_authority_invariants,
        budgets: totals,
        gates,
        terminal_classification: "INCONCLUSIVE".to_string(),
        claim_boundary: "A PASS supports only bounded proof-carrying structural-role transfer under the frozen synthetic graph transformations. It does not establish unrestricted ontology learning, semantic equivalence, natural-language understanding, AGI, or production authority.",
    };
    report.terminal_classification = classify(&report).to_string();
    Ok(report)
}

fn classify(report: &Report) -> &'static str {
    let infrastructure_keys = [
        "development_h12_proof_validated",
        "transport_proof_recomputed_exactly",
        "source_corpus_immutable",
        "shadow_authority_invariants",
        "all_graph_budgets_pass",
        "replay_byte_identical",
    ];
    if infrastructure_keys
        .iter()
        .any(|key| !report.gates.get(*key).copied().unwrap_or(false))
    {
        return "INFRASTRUCTURE_FAILURE";
    }

    let control_keys = [
        "future_false_positives_zero",
        "local_degree_decoys_zero",
        "two_hop_decoys_zero",
        "rewired_controls_zero",
        "foreign_certificate_rejected",
        "irrelevant_role_target_selections_zero",
        "tampered_proof_rejected",
        "delayed_admission_rejected",
        "random_control_exact_zero",
    ];
    if control_keys
        .iter()
        .any(|key| !report.gates.get(*key).copied().unwrap_or(false))
    {
        return "CONTROL_FAILURE";
    }

    let transfer_keys = [
        "holdout_exact_8_of_8",
        "future_exact_56_of_56",
        "vocabulary_invariance_56_of_56",
        "oracle_56_of_56",
        "future_families_8_of_8",
    ];
    if transfer_keys
        .iter()
        .any(|key| !report.gates.get(*key).copied().unwrap_or(false))
    {
        return "REJECTED";
    }
    if report.gates.values().all(|value| *value) {
        "PASS"
    } else {
        "INCONCLUSIVE"
    }
}

fn prepare_mechanism(
    graph_id_offset: u64,
    prefix: &str,
) -> Result<PreparedMechanism, Box<dyn Error>> {
    let development = StructuralCorpus {
        graphs: (0..DEVELOPMENT_GRAPHS)
            .map(|seed| {
                build_case(
                    graph_id_offset + seed as u64,
                    &format!("{prefix}_family{}_{}", seed / 4, seed),
                    Transform::Development,
                    seed as u64,
                )
                .graph
            })
            .collect(),
    };
    let config = RoleInductionConfig {
        min_supporting_graphs: DEVELOPMENT_GRAPHS,
        min_members: DEVELOPMENT_GRAPHS,
    };
    let mut induction_budget = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&development, config, &mut induction_budget)?;

    let mut eligible = Vec::new();
    for proof in &proofs {
        let mut budget = TransportBudget::default();
        if let Ok(transport) = propose_transport_role(&development, proof, config, &mut budget) {
            eligible.push((proof.clone(), transport, budget));
        }
    }
    if eligible.len() != 1 {
        return Err(format!(
            "expected exactly one transport-eligible H12 role, found {}",
            eligible.len()
        )
        .into());
    }
    let (source_proof, transport_proof, proposal_budget) = eligible.remove(0);
    let mut validation_budget = TransportBudget::default();
    let certificate = validate_transport_role(
        &development,
        &source_proof,
        &transport_proof,
        config,
        &mut validation_budget,
    )?;
    let mut registry = ShadowTransportRegistry::new(transport_proof.development_scope_digest);
    registry.register_candidate(&transport_proof)?;
    registry.admit_validated(&certificate)?;

    let irrelevant_proof = proofs
        .into_iter()
        .find(|proof| proof.role_id != source_proof.role_id)
        .ok_or("no independently valid irrelevant H12 role")?;
    let irrelevant_certificate = validate_latent_role(
        &development,
        &irrelevant_proof,
        config,
        &mut RoleInductionBudget::default(),
    )?;
    let mut irrelevant_registry =
        ShadowAbstractionRegistry::new(irrelevant_proof.discovery_scope_digest);
    irrelevant_registry.register_candidate(&irrelevant_proof)?;
    irrelevant_registry.admit_validated(&irrelevant_certificate)?;

    Ok(PreparedMechanism {
        development,
        source_proof,
        transport_proof,
        certificate,
        registry,
        irrelevant_proof,
        irrelevant_registry,
        proposal_budget,
        validation_budget,
    })
}

fn evaluate_split(
    cases: &[GraphCase],
    certificate: &ValidatedTransportRoleCertificate,
    registry: &ShadowTransportRegistry,
    totals: &mut BudgetTotals,
) -> Result<SplitMetrics, TransportError> {
    let mut metrics = SplitMetrics {
        graphs: cases.len(),
        ..SplitMetrics::default()
    };
    for case in cases {
        let mut budget = TransportBudget::default();
        let members = recognize_transport_members(&case.graph, certificate, registry, &mut budget)?;
        if members == vec![case.target.clone()] {
            metrics.exact_role_successes += 1;
        }
        metrics.false_positive_members += members
            .iter()
            .filter(|member| **member != case.target)
            .count();
        metrics.local_degree_decoy_selections +=
            usize::from(members.contains(&case.local_degree_decoy));
        metrics.two_hop_decoy_selections += usize::from(members.contains(&case.two_hop_decoy));
        metrics.vocabulary_control_selections +=
            usize::from(members.contains(&case.vocabulary_control));
        let budget_pass = graph_budget_passes(&case.graph, &budget);
        metrics.graph_budget_passes += usize::from(budget_pass);
        add_budget(totals, &budget, budget_pass);
    }
    Ok(metrics)
}

fn evaluate_controls(
    cases: &[GraphCase],
    prepared: &PreparedMechanism,
    totals: &mut BudgetTotals,
) -> Result<ControlMetrics, Box<dyn Error>> {
    let irrelevant_certificate = validate_latent_role(
        &prepared.development,
        &prepared.irrelevant_proof,
        RoleInductionConfig {
            min_supporting_graphs: DEVELOPMENT_GRAPHS,
            min_members: DEVELOPMENT_GRAPHS,
        },
        &mut RoleInductionBudget::default(),
    )?;

    let mut metrics = ControlMetrics::default();
    for (index, case) in cases.iter().enumerate() {
        let rewired = rewired_control(case, 80_000 + index as u64);
        let mut rewired_budget = TransportBudget::default();
        let rewired_members = recognize_transport_members(
            &rewired,
            &prepared.certificate,
            &prepared.registry,
            &mut rewired_budget,
        )?;
        metrics.rewired_control_selections += rewired_members.len();
        let rewired_budget_pass = graph_budget_passes(&rewired, &rewired_budget);
        add_budget(totals, &rewired_budget, rewired_budget_pass);

        let (relabeled, relabeled_target) = relabel_graph(case, 90_000 + index as u64)?;
        let mut vocabulary_budget = TransportBudget::default();
        let relabeled_members = recognize_transport_members(
            &relabeled,
            &prepared.certificate,
            &prepared.registry,
            &mut vocabulary_budget,
        )?;
        metrics.vocabulary_invariance_successes +=
            usize::from(relabeled_members == vec![relabeled_target]);
        let vocabulary_budget_pass = graph_budget_passes(&relabeled, &vocabulary_budget);
        add_budget(totals, &vocabulary_budget, vocabulary_budget_pass);

        let mut h12_budget = TransferRecognitionBudget::default();
        let irrelevant_members = match recognize_role_members(
            &case.graph,
            &irrelevant_certificate,
            &prepared.irrelevant_registry,
            &mut h12_budget,
        ) {
            Ok(members) => members,
            Err(LatentRoleError::NoTransferMember(_)) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        metrics.irrelevant_role_target_selections +=
            usize::from(irrelevant_members.contains(&case.target));

        let random = root_seeded_random_member(&case.graph, index as u64);
        metrics.random_control_exact_successes += usize::from(random == case.target);
        metrics.oracle_successes += usize::from(case.graph.nodes.contains(&case.target));
    }
    Ok(metrics)
}

fn foreign_certificate_control(
    holdout: &GraphCase,
    prepared: &PreparedMechanism,
) -> Result<bool, Box<dyn Error>> {
    let foreign = prepare_mechanism(50_000, "foreign")?;
    Ok(matches!(
        recognize_transport_members(
            &holdout.graph,
            &foreign.certificate,
            &prepared.registry,
            &mut TransportBudget::default(),
        ),
        Err(TransportError::ForeignDevelopmentScope { .. })
    ))
}

fn tampered_proof_control(prepared: &PreparedMechanism) -> bool {
    let mut tampered = prepared.transport_proof.clone();
    tampered.transport_role_id ^= 1;
    matches!(
        validate_transport_role(
            &prepared.development,
            &prepared.source_proof,
            &tampered,
            RoleInductionConfig {
                min_supporting_graphs: DEVELOPMENT_GRAPHS,
                min_members: DEVELOPMENT_GRAPHS,
            },
            &mut TransportBudget::default(),
        ),
        Err(TransportError::ProofMismatch("transport_role_id"))
    )
}

fn delayed_admission_control(
    holdout: &GraphCase,
    prepared: &PreparedMechanism,
) -> Result<bool, TransportError> {
    let mut candidate_registry =
        ShadowTransportRegistry::new(prepared.transport_proof.development_scope_digest);
    candidate_registry.register_candidate(&prepared.transport_proof)?;
    Ok(matches!(
        recognize_transport_members(
            &holdout.graph,
            &prepared.certificate,
            &candidate_registry,
            &mut TransportBudget::default(),
        ),
        Err(TransportError::RoleNotValidated { .. })
    ))
}

fn graph_budget_passes(graph: &StructuralGraph, budget: &TransportBudget) -> bool {
    let scale = graph.nodes.len().saturating_add(graph.edges.len());
    let traversal_limit = 32usize.saturating_mul(scale.saturating_mul(scale));
    budget.node_signature_evaluations <= graph.nodes.len()
        && budget.reachability_edge_traversals <= traversal_limit
        && budget.candidate_matches <= graph.nodes.len()
}

fn add_budget(totals: &mut BudgetTotals, budget: &TransportBudget, passed: bool) {
    totals.node_signature_evaluations = totals
        .node_signature_evaluations
        .saturating_add(budget.node_signature_evaluations);
    totals.reachability_edge_traversals = totals
        .reachability_edge_traversals
        .saturating_add(budget.reachability_edge_traversals);
    totals.candidate_matches = totals
        .candidate_matches
        .saturating_add(budget.candidate_matches);
    totals.graph_budget_checks = totals.graph_budget_checks.saturating_add(1);
    totals.graph_budget_passes = totals.graph_budget_passes.saturating_add(usize::from(passed));
}

fn build_case(
    graph_id: u64,
    prefix: &str,
    transform: Transform,
    seed: u64,
) -> GraphCase {
    let mut draft = GraphDraft::default();
    let opaque_prefix = if transform.vocabulary_permutation() {
        format!("q{:016x}", splitmix64(seed ^ 0xa5a5_a5a5_a5a5_a5a5))
    } else {
        prefix.to_string()
    };
    let node = |name: &str| atom(format!("{opaque_prefix}_{name}"));

    let s1 = node("source_a");
    let s2 = node("source_b");
    let a = node("upstream_a");
    let b = node("upstream_b");
    let target = node("zz_gateway");
    let c = node("downstream_a");
    let d = node("downstream_b");
    let k1 = node("sink_a");
    let k2 = node("sink_b");

    if transform.path_subdivision() {
        let subdivide_in = node("subdivide_in");
        let subdivide_out = node("subdivide_out");
        draft.add_edge(&s1, &subdivide_in);
        draft.add_edge(&subdivide_in, &a);
        draft.add_edge(&c, &subdivide_out);
        draft.add_edge(&subdivide_out, &k1);
    } else {
        draft.add_edge(&s1, &a);
        draft.add_edge(&c, &k1);
    }
    draft.add_edge(&s2, &b);
    draft.add_edge(&a, &target);
    draft.add_edge(&b, &target);
    draft.add_edge(&target, &c);
    draft.add_edge(&target, &d);
    draft.add_edge(&d, &k2);

    let mut target_sources = vec![s1.clone(), s2.clone()];
    let mut target_sinks = vec![k1.clone(), k2.clone()];
    if transform.local_degree_change() {
        let s3 = node("source_c");
        let k3 = node("sink_c");
        draft.add_edge(&s3, &target);
        draft.add_edge(&target, &k3);
        target_sources.push(s3);
        target_sinks.push(k3);
    }

    if transform.branch_expansion() {
        let branch_source = node("irrelevant_branch_source");
        let branch_mid = node("irrelevant_branch_mid");
        let branch_sink_a = node("irrelevant_branch_sink_a");
        draft.add_edge(&branch_source, &branch_mid);
        draft.add_edge(&branch_mid, &branch_sink_a);
        if !matches!(transform, Transform::BoundedPartialObservation) {
            let branch_sink_b = node("irrelevant_branch_sink_b");
            draft.add_edge(&branch_mid, &branch_sink_b);
        }
    }

    let distractor_chains = if transform.high_distractors() { 16 } else { 4 };
    for index in 0..distractor_chains {
        let x = node(&format!("distractor_{index}_a"));
        let y = node(&format!("distractor_{index}_b"));
        let z = node(&format!("distractor_{index}_c"));
        draft.add_edge(&x, &y);
        draft.add_edge(&y, &z);
    }

    let target_degree = if transform.local_degree_change() { 3 } else { 2 };
    let local_degree_decoy = add_bypass_decoy(
        &mut draft,
        &opaque_prefix,
        "degree_control",
        target_degree,
        false,
    );
    let two_hop_decoy = add_bypass_decoy(
        &mut draft,
        &opaque_prefix,
        "motif_control",
        target_degree,
        true,
    );
    let _rewired_decoy = add_bypass_decoy(
        &mut draft,
        &opaque_prefix,
        "rewired_control",
        target_degree,
        false,
    );

    let vocabulary_control = node("target_gateway_role");
    let vocabulary_source = node("vocabulary_source");
    let vocabulary_sink = node("vocabulary_sink");
    draft.add_edge(&vocabulary_source, &vocabulary_control);
    draft.add_edge(&vocabulary_control, &vocabulary_sink);

    for index in 0..96 {
        draft.add_node(node(&format!("padding_{index:03}")));
    }

    GraphCase {
        graph: draft.finish(graph_id),
        target,
        target_sources,
        target_sinks,
        local_degree_decoy,
        two_hop_decoy,
        vocabulary_control,
    }
}

fn add_bypass_decoy(
    draft: &mut GraphDraft,
    prefix: &str,
    name: &str,
    degree: usize,
    two_hop: bool,
) -> Atom {
    let decoy = atom(format!("{prefix}_{name}_candidate"));
    let alternate = atom(format!("{prefix}_{name}_alternate"));
    for index in 0..degree {
        let source = atom(format!("{prefix}_{name}_source_{index}"));
        let sink = atom(format!("{prefix}_{name}_sink_{index}"));
        if two_hop {
            let pre = atom(format!("{prefix}_{name}_pre_{index}"));
            let post = atom(format!("{prefix}_{name}_post_{index}"));
            draft.add_edge(&source, &pre);
            draft.add_edge(&pre, &decoy);
            draft.add_edge(&decoy, &post);
            draft.add_edge(&post, &sink);
        } else {
            draft.add_edge(&source, &decoy);
            draft.add_edge(&decoy, &sink);
        }
        draft.add_edge(&source, &alternate);
        draft.add_edge(&alternate, &sink);
    }
    decoy
}

fn rewired_control(case: &GraphCase, graph_id: u64) -> StructuralGraph {
    let mut graph = case.graph.clone();
    graph.graph_id = graph_id;
    let bypass = atom(format!("rewired_{graph_id}_bypass"));
    graph.nodes.push(bypass.clone());
    for source in &case.target_sources {
        graph.edges.push(DirectedEdge {
            from: source.clone(),
            to: bypass.clone(),
        });
    }
    for sink in &case.target_sinks {
        graph.edges.push(DirectedEdge {
            from: bypass.clone(),
            to: sink.clone(),
        });
    }
    graph.nodes.sort();
    graph.edges.sort_by(|left, right| {
        (&left.from, &left.to).cmp(&(&right.from, &right.to))
    });
    graph
}

fn relabel_graph(case: &GraphCase, graph_id: u64) -> Result<(StructuralGraph, Atom), Box<dyn Error>> {
    let mut nodes = case.graph.nodes.clone();
    nodes.sort();
    let mapping = nodes
        .iter()
        .enumerate()
        .map(|(index, old)| {
            Ok((
                old.clone(),
                Atom::new(format!("opaque_{graph_id}_{index:04}"))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, star::commitment_state::CommitmentStateError>>()?;
    let relabeled_nodes = nodes
        .iter()
        .map(|node| mapping.get(node).expect("complete mapping").clone())
        .collect();
    let relabeled_edges = case
        .graph
        .edges
        .iter()
        .map(|edge| DirectedEdge {
            from: mapping.get(&edge.from).expect("complete mapping").clone(),
            to: mapping.get(&edge.to).expect("complete mapping").clone(),
        })
        .collect();
    let target = mapping
        .get(&case.target)
        .expect("target mapping")
        .clone();
    Ok((
        StructuralGraph {
            graph_id,
            nodes: relabeled_nodes,
            edges: relabeled_edges,
        },
        target,
    ))
}

fn root_seeded_random_member(graph: &StructuralGraph, root: u64) -> Atom {
    let mut nodes = graph.nodes.clone();
    nodes.sort();
    let index = (splitmix64(root ^ RANDOM_SALT) as usize) % nodes.len();
    nodes[index].clone()
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn atom(value: impl Into<String>) -> Atom {
    Atom::new(value.into()).expect("fixture atoms are non-empty")
}
