use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::graph_discovery::{
    admit_graph_certificate, infer_graph_rule, validate_graph_rule, FrontierDiscoveryBudget,
    MixedEvidenceGraph, ValidatedGraphInferenceCertificate,
};
use star::rule_induction::{
    EvidenceBoundCommitmentState, EvidenceEpisode, RuleInductionConfig, ScoringBudget,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const CLOSURE_SCANS: usize = 3;
const TOTAL_EVIDENCE: usize = 13;
const START_SALT: u64 = 0x9e37_79b9_7f4a_7c15;
const STRIDE_SALT: u64 = 0xd1b5_4a32_d192_ed03;
const FAMILIES: [&str; 7] = [
    "thermal",
    "transport",
    "ecology",
    "cellular",
    "manufacturing",
    "software",
    "watershed",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DirectedEdge {
    from: Atom,
    to: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructuralGraph {
    graph_id: u64,
    nodes: Vec<Atom>,
    edges: Vec<DirectedEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructuralCorpus {
    graphs: Vec<StructuralGraph>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StructuralFingerprint {
    in_degree: u16,
    out_degree: u16,
    predecessor_convergence_count: u16,
    predecessor_divergence_count: u16,
    successor_convergence_count: u16,
    successor_divergence_count: u16,
    has_convergent_ancestor: bool,
    has_divergent_ancestor: bool,
    has_convergent_descendant: bool,
    has_divergent_descendant: bool,
    source_reachable: bool,
    sink_reachable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RoleMember {
    graph_id: u64,
    node: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoleProof {
    role_id: u64,
    fingerprint: StructuralFingerprint,
    members: Vec<RoleMember>,
    supporting_graph_ids: Vec<u64>,
    proof_id: u64,
}

#[derive(Debug, Clone)]
struct Root {
    id: u64,
    corpus: StructuralCorpus,
    transfer: StructuralGraph,
    evidence: MixedEvidenceGraph,
    source: Atom,
    middle: Atom,
    goal: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct H11Attempt {
    certificate: Option<ValidatedGraphInferenceCertificate>,
    exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootOutcome {
    stateful_success: bool,
    strict_control_success: bool,
    strict_target_certificate: bool,
    cardinality_exact: bool,
    includes_target: bool,
    includes_non_target: bool,
    unique_membership: bool,
    replay_exact: bool,
    h11_accounting_exact: bool,
    role_recomputation_exact: bool,
    stateful_invariants: bool,
    strict_invariants: bool,
    stateful_closure_scans: usize,
    strict_closure_scans: usize,
    strict_group_evidence_ids: Vec<u64>,
}

#[derive(Debug, Default, Serialize)]
struct SplitMetrics {
    roots: usize,
    stateful_successes: usize,
    strict_control_successes: usize,
    strict_target_certificates: usize,
    cardinality_exact: usize,
    includes_target: usize,
    includes_non_target: usize,
    unique_membership: usize,
    replay_exact: usize,
    h11_accounting_exact: usize,
    role_recomputation_exact: usize,
    invariants_hold: usize,
    closure_budget_exact: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration: &'static str,
    frozen_partition: &'static str,
    training: SplitMetrics,
    holdout: SplitMetrics,
    future: SplitMetrics,
    future_family_stateful_successes: BTreeMap<String, usize>,
    future_family_strict_control_successes: BTreeMap<String, usize>,
    gates: BTreeMap<String, bool>,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let roots = build_roots();
    let outcomes = roots
        .iter()
        .map(evaluate_root)
        .collect::<Result<Vec<_>, _>>()?;

    let training = summarize(&outcomes[0..16]);
    let holdout = summarize(&outcomes[16..24]);
    let future = summarize(&outcomes[24..56]);

    let mut future_family_stateful_successes = BTreeMap::new();
    let mut future_family_strict_control_successes = BTreeMap::new();
    for family_index in 3..7 {
        let start = family_index * ROOTS_PER_FAMILY;
        let family = summarize(&outcomes[start..start + ROOTS_PER_FAMILY]);
        future_family_stateful_successes.insert(
            FAMILIES[family_index].to_string(),
            family.stateful_successes,
        );
        future_family_strict_control_successes.insert(
            FAMILIES[family_index].to_string(),
            family.strict_control_successes,
        );
    }

    let cohort_exact = roots.len() == 56;
    let stateful_exact = training.stateful_successes == 16
        && holdout.stateful_successes == 8
        && future.stateful_successes == 32;
    let strict_zero = training.strict_control_successes == 0
        && holdout.strict_control_successes == 0
        && future.strict_control_successes == 0;
    let target_certificates_zero = training.strict_target_certificates == 0
        && holdout.strict_target_certificates == 0
        && future.strict_target_certificates == 0;
    let cardinality_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.cardinality_exact
    });
    let mixing_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.includes_target.min(metrics.includes_non_target)
    });
    let unique_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.unique_membership
    });
    let replay_exact = all_roots(&training, &holdout, &future, |metrics| metrics.replay_exact);
    let h11_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.h11_accounting_exact
    });
    let roles_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.role_recomputation_exact
    });
    let invariants_hold = all_roots(&training, &holdout, &future, |metrics| {
        metrics.invariants_hold
    });
    let closure_exact = all_roots(&training, &holdout, &future, |metrics| {
        metrics.closure_budget_exact
    });
    let future_family_exact = future_family_stateful_successes
        .values()
        .all(|successes| *successes == ROOTS_PER_FAMILY)
        && future_family_strict_control_successes
            .values()
            .all(|successes| *successes == 0);

    let mut gates = BTreeMap::new();
    gates.insert("cohort_exact".to_string(), cohort_exact);
    gates.insert(
        "stateful_training_holdout_future_exact".to_string(),
        stateful_exact,
    );
    gates.insert("strict_random_control_zero".to_string(), strict_zero);
    gates.insert(
        "strict_random_target_certificates_zero".to_string(),
        target_certificates_zero,
    );
    gates.insert("random_cardinality_exact".to_string(), cardinality_exact);
    gates.insert(
        "random_target_non_target_mix_exact".to_string(),
        mixing_exact,
    );
    gates.insert("random_membership_unique".to_string(), unique_exact);
    gates.insert("root_seeded_replay_exact".to_string(), replay_exact);
    gates.insert("h11_accounting_exact".to_string(), h11_exact);
    gates.insert("role_recomputation_exact".to_string(), roles_exact);
    gates.insert("pecs_invariants_hold".to_string(), invariants_hold);
    gates.insert("closure_budget_exact".to_string(), closure_exact);
    gates.insert(
        "future_family_transfer_exact".to_string(),
        future_family_exact,
    );

    let terminal_classification = if !h11_exact || !closure_exact {
        "INFRASTRUCTURE_FAILURE"
    } else if !roles_exact || !stateful_exact {
        "REFERENCE_FAILURE"
    } else if !cardinality_exact || !unique_exact {
        "CARDINALITY_FAILURE"
    } else if !mixing_exact {
        "MIXING_FAILURE"
    } else if !replay_exact {
        "REPLAY_FAILURE"
    } else if !invariants_hold {
        "PROVENANCE_FAILURE"
    } else if !strict_zero || !target_certificates_zero || !future_family_exact {
        "CONTROL_FAILURE"
    } else if gates.values().all(|value| *value) {
        "PASS"
    } else {
        "INFRASTRUCTURE_FAILURE"
    };

    let report = Report {
        experiment: "H12 strict root-seeded random-partition conformance",
        preregistration: "docs/experiments/H12_STRICT_RANDOM_PARTITION_CONFORMANCE.md",
        frozen_partition: "sort 13 evidence episodes by evidence_id; derive root-seeded cyclic start and non-unit stride; select the circular four-member block containing the minimum evidence_id, where four is measured from the validated target-role projection",
        training,
        holdout,
        future,
        future_family_stateful_successes,
        future_family_strict_control_successes,
        gates,
        terminal_classification,
        claim_boundary: "PASS closes only the literal H12 root-seeded same-cardinality random-grouping conformance gap in the frozen synthetic regime; it does not establish open-world ontology induction, automatic promotion, live routing, AGI, consciousness, or human-level cognition",
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal_classification != "PASS" {
        return Err(format!(
            "H12 strict random-partition conformance classification: {terminal_classification}"
        )
        .into());
    }
    Ok(())
}

fn all_roots(
    training: &SplitMetrics,
    holdout: &SplitMetrics,
    future: &SplitMetrics,
    count: impl Fn(&SplitMetrics) -> usize,
) -> bool {
    count(training) == training.roots
        && count(holdout) == holdout.roots
        && count(future) == future.roots
}

fn summarize(outcomes: &[RootOutcome]) -> SplitMetrics {
    let mut metrics = SplitMetrics {
        roots: outcomes.len(),
        ..SplitMetrics::default()
    };
    for outcome in outcomes {
        metrics.stateful_successes += usize::from(outcome.stateful_success);
        metrics.strict_control_successes += usize::from(outcome.strict_control_success);
        metrics.strict_target_certificates += usize::from(outcome.strict_target_certificate);
        metrics.cardinality_exact += usize::from(outcome.cardinality_exact);
        metrics.includes_target += usize::from(outcome.includes_target);
        metrics.includes_non_target += usize::from(outcome.includes_non_target);
        metrics.unique_membership += usize::from(outcome.unique_membership);
        metrics.replay_exact += usize::from(outcome.replay_exact);
        metrics.h11_accounting_exact += usize::from(outcome.h11_accounting_exact);
        metrics.role_recomputation_exact += usize::from(outcome.role_recomputation_exact);
        metrics.invariants_hold +=
            usize::from(outcome.stateful_invariants && outcome.strict_invariants);
        metrics.closure_budget_exact += usize::from(
            outcome.stateful_closure_scans == CLOSURE_SCANS
                && outcome.strict_closure_scans == CLOSURE_SCANS,
        );
    }
    metrics
}

fn evaluate_root(root: &Root) -> Result<RootOutcome, Box<dyn Error>> {
    let proofs = induce_roles(&root.corpus);
    if proofs.is_empty() {
        return Err(format!("root {} induced no recurring structural roles", root.id).into());
    }
    let role_recomputation_exact = proofs
        .iter()
        .all(|proof| validate_role(&root.corpus, proof));

    let config = RuleInductionConfig::default();
    let mut role_certificates = Vec::<ValidatedGraphInferenceCertificate>::new();
    let mut target_projection = None::<MixedEvidenceGraph>;
    let mut h11_accounting_exact = true;

    for proof in &proofs {
        if !validate_role(&root.corpus, proof) {
            continue;
        }
        let members = recognize_members(&root.transfer, proof.fingerprint);
        let projected = project_evidence(&root.evidence, &members);
        let Some(projected) = projected else {
            continue;
        };
        let attempt = h11(&projected, config);
        h11_accounting_exact &= attempt.exact;
        if let Some(certificate) = attempt.certificate {
            if certificate.rule().antecedent == root.middle
                && certificate.rule().consequent == root.goal
            {
                target_projection = Some(projected.clone());
            }
            if !role_certificates
                .iter()
                .any(|existing| existing.rule() == certificate.rule())
            {
                role_certificates.push(certificate);
            }
        }
    }

    let target_projection = target_projection.ok_or_else(|| {
        format!(
            "root {} failed to recover the validated target-role projection",
            root.id
        )
    })?;

    let mut stateful = initial_state(root)?;
    for certificate in &role_certificates {
        admit_graph_certificate(&mut stateful, certificate)?;
    }
    let stateful_closure_scans = run_closure(&mut stateful)?;
    let stateful_success = stateful.contains_fact(&root.goal);
    let stateful_invariants = stateful.verify_invariants().is_ok();

    let strict_group =
        strict_random_partition(root.id, &root.evidence, target_projection.evidence.len())?;
    let replay_group =
        strict_random_partition(root.id, &root.evidence, target_projection.evidence.len())?;
    let strict_group_evidence_ids = strict_group
        .evidence
        .iter()
        .map(|episode| episode.evidence_id)
        .collect::<Vec<_>>();
    let replay_ids = replay_group
        .evidence
        .iter()
        .map(|episode| episode.evidence_id)
        .collect::<Vec<_>>();
    let replay_exact = strict_group_evidence_ids == replay_ids;
    let unique_membership = strict_group_evidence_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len()
        == strict_group_evidence_ids.len();
    let cardinality_exact = strict_group.evidence.len() == target_projection.evidence.len();
    let includes_target = strict_group
        .evidence
        .iter()
        .any(|episode| episode.intervention == root.middle);
    let includes_non_target = strict_group
        .evidence
        .iter()
        .any(|episode| episode.intervention != root.middle);

    let strict_attempt = h11(&strict_group, config);
    h11_accounting_exact &= strict_attempt.exact;
    let strict_target_certificate =
        strict_attempt
            .certificate
            .as_ref()
            .is_some_and(|certificate| {
                certificate.rule().antecedent == root.middle
                    && certificate.rule().consequent == root.goal
            });

    let mut strict = initial_state(root)?;
    if let Some(certificate) = strict_attempt.certificate.as_ref() {
        admit_graph_certificate(&mut strict, certificate)?;
    }
    let strict_closure_scans = run_closure(&mut strict)?;
    let strict_control_success = strict.contains_fact(&root.goal);
    let strict_invariants = strict.verify_invariants().is_ok();

    Ok(RootOutcome {
        stateful_success,
        strict_control_success,
        strict_target_certificate,
        cardinality_exact,
        includes_target,
        includes_non_target,
        unique_membership,
        replay_exact,
        h11_accounting_exact,
        role_recomputation_exact,
        stateful_invariants,
        strict_invariants,
        stateful_closure_scans,
        strict_closure_scans,
        strict_group_evidence_ids,
    })
}

fn initial_state(root: &Root) -> Result<EvidenceBoundCommitmentState, Box<dyn Error>> {
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    Ok(state)
}

fn run_closure(state: &mut EvidenceBoundCommitmentState) -> Result<usize, Box<dyn Error>> {
    for _ in 0..CLOSURE_SCANS {
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
        }
    }
    Ok(CLOSURE_SCANS)
}

fn h11(graph: &MixedEvidenceGraph, config: RuleInductionConfig) -> H11Attempt {
    let mut proposal_frontier = FrontierDiscoveryBudget::default();
    let mut proposal_scoring = ScoringBudget::default();
    let proof = infer_graph_rule(graph, config, &mut proposal_frontier, &mut proposal_scoring);
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

fn strict_random_partition(
    root_id: u64,
    evidence: &MixedEvidenceGraph,
    group_size: usize,
) -> Result<MixedEvidenceGraph, Box<dyn Error>> {
    if evidence.evidence.len() != TOTAL_EVIDENCE {
        return Err(format!(
            "strict H12 partition expected {TOTAL_EVIDENCE} evidence episodes, got {}",
            evidence.evidence.len()
        )
        .into());
    }
    if group_size == 0 || group_size > evidence.evidence.len() {
        return Err(format!("invalid strict H12 group size {group_size}").into());
    }

    let mut canonical = evidence.evidence.clone();
    canonical.sort_by_key(|episode| episode.evidence_id);
    let count = canonical.len();
    let start = (splitmix64(root_id ^ START_SALT) as usize) % count;
    let stride = 2 + (splitmix64(root_id ^ STRIDE_SALT) as usize % 10);

    let permutation = (0..count)
        .map(|position| (start + position * stride) % count)
        .collect::<Vec<_>>();
    if permutation.iter().copied().collect::<BTreeSet<_>>().len() != count {
        return Err(format!("root {root_id} produced a non-bijective cyclic permutation").into());
    }

    let minimum_id = canonical
        .first()
        .ok_or("strict H12 evidence unexpectedly empty")?
        .evidence_id;
    let minimum_position = permutation
        .iter()
        .position(|index| canonical[*index].evidence_id == minimum_id)
        .ok_or("minimum evidence id absent from strict H12 permutation")?;
    let block_start = (minimum_position / group_size) * group_size;

    let mut selected = (0..group_size)
        .map(|offset| {
            let permutation_position = (block_start + offset) % count;
            canonical[permutation[permutation_position]].clone()
        })
        .collect::<Vec<_>>();
    selected.sort_by_key(|episode| episode.evidence_id);
    Ok(MixedEvidenceGraph { evidence: selected })
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn induce_roles(corpus: &StructuralCorpus) -> Vec<RoleProof> {
    let groups = fingerprint_groups(corpus);
    let mut proofs = groups
        .into_iter()
        .filter_map(|(fingerprint, mut members)| {
            members.sort();
            let supporting_graph_ids = members
                .iter()
                .map(|member| member.graph_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if supporting_graph_ids.len() < 4 || members.len() < 4 {
                return None;
            }
            let role_id = fingerprint_digest(fingerprint);
            let proof_id = proof_digest(role_id, fingerprint, &members, &supporting_graph_ids);
            Some(RoleProof {
                role_id,
                fingerprint,
                members,
                supporting_graph_ids,
                proof_id,
            })
        })
        .collect::<Vec<_>>();
    proofs.sort_by_key(|proof| proof.role_id);
    proofs
}

fn validate_role(corpus: &StructuralCorpus, proof: &RoleProof) -> bool {
    induce_roles(corpus)
        .into_iter()
        .any(|recomputed| recomputed == *proof)
}

fn fingerprint_groups(
    corpus: &StructuralCorpus,
) -> BTreeMap<StructuralFingerprint, Vec<RoleMember>> {
    let mut groups = BTreeMap::<StructuralFingerprint, Vec<RoleMember>>::new();
    let mut graphs = corpus.graphs.iter().collect::<Vec<_>>();
    graphs.sort_by_key(|graph| graph.graph_id);
    for graph in graphs {
        let index = GraphIndex::new(graph);
        let mut nodes = graph.nodes.clone();
        nodes.sort();
        for node in nodes {
            let fingerprint = compute_fingerprint(&index, &node);
            groups.entry(fingerprint).or_default().push(RoleMember {
                graph_id: graph.graph_id,
                node,
            });
        }
    }
    groups
}

fn recognize_members(
    graph: &StructuralGraph,
    fingerprint: StructuralFingerprint,
) -> BTreeSet<Atom> {
    let index = GraphIndex::new(graph);
    graph
        .nodes
        .iter()
        .filter(|node| compute_fingerprint(&index, node) == fingerprint)
        .cloned()
        .collect()
}

fn project_evidence(
    evidence: &MixedEvidenceGraph,
    members: &BTreeSet<Atom>,
) -> Option<MixedEvidenceGraph> {
    let projected = evidence
        .evidence
        .iter()
        .filter(|episode| members.contains(&episode.intervention))
        .cloned()
        .collect::<Vec<_>>();
    (!projected.is_empty()).then_some(MixedEvidenceGraph {
        evidence: projected,
    })
}

struct GraphIndex {
    outgoing: BTreeMap<Atom, Vec<Atom>>,
    incoming: BTreeMap<Atom, Vec<Atom>>,
}

impl GraphIndex {
    fn new(graph: &StructuralGraph) -> Self {
        let mut outgoing = graph
            .nodes
            .iter()
            .cloned()
            .map(|node| (node, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        let mut incoming = outgoing.clone();
        for edge in &graph.edges {
            outgoing
                .get_mut(&edge.from)
                .expect("builder registered edge source")
                .push(edge.to.clone());
            incoming
                .get_mut(&edge.to)
                .expect("builder registered edge destination")
                .push(edge.from.clone());
        }
        for neighbors in outgoing.values_mut() {
            neighbors.sort();
        }
        for neighbors in incoming.values_mut() {
            neighbors.sort();
        }
        Self { outgoing, incoming }
    }

    fn in_degree(&self, node: &Atom) -> usize {
        self.incoming.get(node).map_or(0, Vec::len)
    }

    fn out_degree(&self, node: &Atom) -> usize {
        self.outgoing.get(node).map_or(0, Vec::len)
    }
}

fn compute_fingerprint(index: &GraphIndex, node: &Atom) -> StructuralFingerprint {
    let predecessors = index
        .incoming
        .get(node)
        .expect("fingerprinted node exists in incoming index");
    let successors = index
        .outgoing
        .get(node)
        .expect("fingerprinted node exists in outgoing index");
    let predecessor_convergence_count = predecessors
        .iter()
        .filter(|candidate| index.in_degree(candidate) >= 2)
        .count();
    let predecessor_divergence_count = predecessors
        .iter()
        .filter(|candidate| index.out_degree(candidate) >= 2)
        .count();
    let successor_convergence_count = successors
        .iter()
        .filter(|candidate| index.in_degree(candidate) >= 2)
        .count();
    let successor_divergence_count = successors
        .iter()
        .filter(|candidate| index.out_degree(candidate) >= 2)
        .count();
    let ancestors = reachable(index, node, true);
    let descendants = reachable(index, node, false);

    StructuralFingerprint {
        in_degree: saturating_u16(index.in_degree(node)),
        out_degree: saturating_u16(index.out_degree(node)),
        predecessor_convergence_count: saturating_u16(predecessor_convergence_count),
        predecessor_divergence_count: saturating_u16(predecessor_divergence_count),
        successor_convergence_count: saturating_u16(successor_convergence_count),
        successor_divergence_count: saturating_u16(successor_divergence_count),
        has_convergent_ancestor: ancestors
            .iter()
            .any(|candidate| index.in_degree(candidate) >= 2),
        has_divergent_ancestor: ancestors
            .iter()
            .any(|candidate| index.out_degree(candidate) >= 2),
        has_convergent_descendant: descendants
            .iter()
            .any(|candidate| index.in_degree(candidate) >= 2),
        has_divergent_descendant: descendants
            .iter()
            .any(|candidate| index.out_degree(candidate) >= 2),
        source_reachable: index.in_degree(node) == 0
            || ancestors
                .iter()
                .any(|candidate| index.in_degree(candidate) == 0),
        sink_reachable: index.out_degree(node) == 0
            || descendants
                .iter()
                .any(|candidate| index.out_degree(candidate) == 0),
    }
}

fn reachable(index: &GraphIndex, start: &Atom, reverse: bool) -> BTreeSet<Atom> {
    let adjacency = if reverse {
        &index.incoming
    } else {
        &index.outgoing
    };
    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::new();
    if let Some(neighbors) = adjacency.get(start) {
        queue.extend(neighbors.iter().cloned());
    }
    while let Some(node) = queue.pop_front() {
        if !reached.insert(node.clone()) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&node) {
            queue.extend(neighbors.iter().cloned());
        }
    }
    reached
}

fn saturating_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn fingerprint_digest(fingerprint: StructuralFingerprint) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&fingerprint.in_degree.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.out_degree.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.predecessor_convergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.predecessor_divergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.successor_convergence_count.to_le_bytes());
    bytes.extend_from_slice(&fingerprint.successor_divergence_count.to_le_bytes());
    bytes.extend_from_slice(&[
        u8::from(fingerprint.has_convergent_ancestor),
        u8::from(fingerprint.has_divergent_ancestor),
        u8::from(fingerprint.has_convergent_descendant),
        u8::from(fingerprint.has_divergent_descendant),
        u8::from(fingerprint.source_reachable),
        u8::from(fingerprint.sink_reachable),
    ]);
    fnv1a64(&bytes)
}

fn proof_digest(
    role_id: u64,
    fingerprint: StructuralFingerprint,
    members: &[RoleMember],
    supporting_graph_ids: &[u64],
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&role_id.to_le_bytes());
    bytes.extend_from_slice(&fingerprint_digest(fingerprint).to_le_bytes());
    for member in members {
        bytes.extend_from_slice(&member.graph_id.to_le_bytes());
        bytes.extend_from_slice(member.node.as_str().as_bytes());
        bytes.push(0xff);
    }
    for graph_id in supporting_graph_ids {
        bytes.extend_from_slice(&graph_id.to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn build_roots() -> Vec<Root> {
    let mut roots = Vec::new();
    let mut id = 1_u64;
    for (family_index, family) in FAMILIES.into_iter().enumerate() {
        for local in 0..ROOTS_PER_FAMILY {
            let prefix = format!("h12_conformance_{family}_{id}_{local}");
            let middle = atom(format!("{prefix}_latent_bridge"));
            let irrelevant = atom(format!("{prefix}_irrelevant_bridge"));
            let size_member = atom(format!("{prefix}_size_member"));
            let goal = atom(format!("{prefix}_goal"));
            let irrelevant_goal = atom(format!("{prefix}_irrelevant_goal"));
            let size_goal = atom(format!("{prefix}_size_goal"));
            roots.push(Root {
                id,
                corpus: discovery_corpus(id, &prefix),
                transfer: transfer_graph(
                    id,
                    &prefix,
                    family_index,
                    &middle,
                    &irrelevant,
                    &size_member,
                ),
                evidence: evidence_graph(
                    id,
                    &prefix,
                    &middle,
                    &goal,
                    &irrelevant,
                    &irrelevant_goal,
                    &size_member,
                    &size_goal,
                ),
                source: atom(format!("{prefix}_source")),
                middle,
                goal,
            });
            id += 1;
        }
    }
    roots
}

fn discovery_corpus(root_id: u64, prefix: &str) -> StructuralCorpus {
    StructuralCorpus {
        graphs: (0..4)
            .map(|index| {
                let graph_prefix = format!("{prefix}_discover_{index}");
                let mut graph = Builder::new(root_id * 100 + index as u64 + 1);
                target_motif(
                    &mut graph,
                    &graph_prefix,
                    atom(format!("{graph_prefix}_latent")),
                    2 + index,
                    2 + ((index + 1) % 4),
                );
                irrelevant_motif(
                    &mut graph,
                    &graph_prefix,
                    atom(format!("{graph_prefix}_irrelevant")),
                );
                let mut previous = atom(format!("{graph_prefix}_d0"));
                graph.node(previous.clone());
                for step in 1..(2 + index) {
                    let next = atom(format!("{graph_prefix}_d{step}"));
                    graph.edge(previous, next.clone());
                    previous = next;
                }
                graph.finish()
            })
            .collect(),
    }
}

fn transfer_graph(
    root_id: u64,
    prefix: &str,
    family_index: usize,
    middle: &Atom,
    irrelevant: &Atom,
    size_member: &Atom,
) -> StructuralGraph {
    let mut graph = Builder::new(root_id * 10_000 + 9_001);
    target_motif(
        &mut graph,
        &format!("{prefix}_target"),
        middle.clone(),
        2 + family_index,
        2 + family_index % 4,
    );
    irrelevant_motif(
        &mut graph,
        &format!("{prefix}_irrelevant"),
        irrelevant.clone(),
    );
    graph.edge(atom(format!("{prefix}_size_before")), size_member.clone());
    graph.edge(size_member.clone(), atom(format!("{prefix}_size_after")));
    for extra in 0..family_index {
        let branch = atom(format!("{prefix}_extra_{extra}_a"));
        graph.edge(branch.clone(), atom(format!("{prefix}_extra_{extra}_b")));
        graph.edge(branch, atom(format!("{prefix}_extra_{extra}_c")));
    }
    graph.finish()
}

fn evidence_graph(
    root_id: u64,
    prefix: &str,
    middle: &Atom,
    goal: &Atom,
    irrelevant: &Atom,
    irrelevant_goal: &Atom,
    size_member: &Atom,
    size_goal: &Atom,
) -> MixedEvidenceGraph {
    let common_noise = atom(format!("{prefix}_common_noise"));
    let size_noise = atom(format!("{prefix}_size_noise"));
    let mut evidence = Vec::new();
    let mut evidence_id = root_id * 1_000_000 + 1;

    for index in 0..4 {
        let mut outcomes = BTreeSet::from([goal.clone()]);
        if index < 2 {
            outcomes.insert(common_noise.clone());
        }
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: middle.clone(),
            outcomes,
        });
        evidence_id += 1;
    }
    for index in 0..5 {
        let mut outcomes = BTreeSet::from([irrelevant_goal.clone()]);
        if index < 2 {
            outcomes.insert(common_noise.clone());
        }
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: irrelevant.clone(),
            outcomes,
        });
        evidence_id += 1;
    }
    for index in 0..4 {
        let mut outcomes = BTreeSet::from([size_goal.clone()]);
        if index < 2 {
            outcomes.insert(size_noise.clone());
        }
        evidence.push(EvidenceEpisode {
            evidence_id,
            intervention: size_member.clone(),
            outcomes,
        });
        evidence_id += 1;
    }

    MixedEvidenceGraph { evidence }
}

fn target_motif(graph: &mut Builder, prefix: &str, latent: Atom, fan_in: usize, fan_out: usize) {
    let join = atom(format!("{prefix}_join"));
    let split = atom(format!("{prefix}_split"));
    for index in 0..fan_in {
        graph.edge(atom(format!("{prefix}_s{index}")), join.clone());
    }
    graph.edge(join, latent.clone());
    graph.edge(latent, split.clone());
    for index in 0..fan_out {
        graph.edge(split.clone(), atom(format!("{prefix}_t{index}")));
    }
}

fn irrelevant_motif(graph: &mut Builder, prefix: &str, latent: Atom) {
    let branch = atom(format!("{prefix}_branch"));
    let merge = atom(format!("{prefix}_merge"));
    graph.edge(atom(format!("{prefix}_source")), branch.clone());
    graph.edge(branch.clone(), latent.clone());
    graph.edge(branch, atom(format!("{prefix}_branch_leaf")));
    graph.edge(latent, merge.clone());
    graph.edge(atom(format!("{prefix}_merge_source")), merge.clone());
    graph.edge(merge, atom(format!("{prefix}_sink")));
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

    fn node(&mut self, node: Atom) {
        self.nodes.insert(node);
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
    Atom::new(value.into()).expect("generated H12 conformance atom is non-empty")
}
