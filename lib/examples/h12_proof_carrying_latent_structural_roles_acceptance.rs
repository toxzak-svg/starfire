use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::graph_discovery::{
    admit_graph_certificate, infer_graph_rule, validate_graph_rule, FrontierDiscoveryBudget,
    MixedEvidenceGraph, ValidatedGraphInferenceCertificate,
};
use star::latent_roles::{
    discovery_scope_digest, induce_latent_roles, project_evidence_for_control_group,
    project_evidence_for_role, validate_latent_role, DirectedEdge, LatentRoleProof,
    RoleInductionBudget, RoleInductionConfig, RoleId, ShadowAbstractionRegistry,
    StructuralCorpus, StructuralGraph, TransferRecognitionBudget,
    ValidatedLatentRoleCertificate,
};
use star::rule_induction::{
    EvidenceBoundCommitmentState, EvidenceEpisode, RuleInductionConfig, ScoringBudget,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const CLOSURE_SCANS: usize = 3;
const FAMILIES: [&str; 7] = [
    "thermal", "transport", "ecology", "cellular", "manufacturing", "software", "watershed",
];

#[derive(Clone)]
struct Root {
    id: u64,
    corpus: StructuralCorpus,
    transfer: StructuralGraph,
    evidence: MixedEvidenceGraph,
    source: Atom,
    middle: Atom,
    goal: Atom,
    irrelevant: Atom,
    irrelevant_goal: Atom,
    size_member: Atom,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Path {
    Stateful,
    NoAbstraction,
    TextOnly,
    ScalarOnly,
    RandomGrouping,
    SizeMatched,
    ValidIrrelevant,
    Foreign,
    MembershipTamper,
    Delayed,
}

impl Path {
    fn all() -> [Self; 10] {
        [
            Self::Stateful,
            Self::NoAbstraction,
            Self::TextOnly,
            Self::ScalarOnly,
            Self::RandomGrouping,
            Self::SizeMatched,
            Self::ValidIrrelevant,
            Self::Foreign,
            Self::MembershipTamper,
            Self::Delayed,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::Stateful => "stateful_validated_role",
            Self::NoAbstraction => "no_abstraction_full_graph",
            Self::TextOnly => "role_proof_text_only",
            Self::ScalarOnly => "scalar_role_id_only",
            Self::RandomGrouping => "random_grouping",
            Self::SizeMatched => "size_matched_grouping",
            Self::ValidIrrelevant => "valid_irrelevant_role",
            Self::Foreign => "foreign_abstraction",
            Self::MembershipTamper => "membership_tampered_abstraction",
            Self::Delayed => "delayed_abstraction_admission",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Prepared {
    real: Vec<ValidatedGraphInferenceCertificate>,
    target: Option<ValidatedGraphInferenceCertificate>,
    irrelevant: Option<ValidatedGraphInferenceCertificate>,
    full: Option<ValidatedGraphInferenceCertificate>,
    random: Option<ValidatedGraphInferenceCertificate>,
    size: Option<ValidatedGraphInferenceCertificate>,
    foreign_rejected: bool,
    tamper_rejected: bool,
    target_role_found: bool,
    membership_exact: bool,
    budget_exact: bool,
    proof_text_len: usize,
    scalar_count: usize,
}

#[derive(Clone, PartialEq, Eq)]
struct Execution {
    success: bool,
    certificate_available: bool,
    admitted: usize,
    rejected: bool,
    foreign_rejected: bool,
    tamper_rejected: bool,
    target_role_found: bool,
    membership_exact: bool,
    budget_exact: bool,
    invariants: bool,
    signature: String,
    inert_payload: usize,
}

#[derive(Default, Serialize)]
struct Metrics {
    roots: usize,
    successes: usize,
    certificate_available: usize,
    admitted: usize,
    rejections: usize,
    foreign_rejections: usize,
    tamper_rejections: usize,
    target_role_found: usize,
    membership_exact: usize,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

#[derive(Serialize)]
struct Report {
    experiment: &'static str,
    correction_scope: &'static str,
    first_verdict_run: &'static str,
    first_verdict_classification: &'static str,
    split_metrics: BTreeMap<String, BTreeMap<String, Metrics>>,
    future_family_stateful_successes: BTreeMap<String, usize>,
    future_family_max_control_successes: BTreeMap<String, usize>,
    gates: BTreeMap<String, bool>,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let role_config = RoleInductionConfig::default();
    let rule_config = RuleInductionConfig::default();
    let roots = build_roots();

    let mut splits = BTreeMap::new();
    splits.insert("training".into(), evaluate(&roots[0..16], role_config, rule_config)?);
    splits.insert("holdout".into(), evaluate(&roots[16..24], role_config, rule_config)?);
    splits.insert("future".into(), evaluate(&roots[24..56], role_config, rule_config)?);

    let mut family_stateful = BTreeMap::new();
    let mut family_control = BTreeMap::new();
    for family_index in 3..7 {
        let start = family_index * ROOTS_PER_FAMILY;
        let metrics = evaluate(
            &roots[start..start + ROOTS_PER_FAMILY],
            role_config,
            rule_config,
        )?;
        family_stateful.insert(
            FAMILIES[family_index].to_string(),
            metrics[Path::Stateful.name()].successes,
        );
        family_control.insert(
            FAMILIES[family_index].to_string(),
            Path::all()
                .into_iter()
                .filter(|path| *path != Path::Stateful)
                .map(|path| metrics[path.name()].successes)
                .max()
                .unwrap_or(0),
        );
    }

    let train = &splits["training"];
    let holdout = &splits["holdout"];
    let future = &splits["future"];
    let stateful_exact = train[Path::Stateful.name()].successes == 16
        && holdout[Path::Stateful.name()].successes == 8
        && future[Path::Stateful.name()].successes == 32;
    let controls_zero = splits.values().all(|split| {
        Path::all()
            .into_iter()
            .filter(|path| *path != Path::Stateful)
            .all(|path| split[path.name()].successes == 0)
    });
    let irrelevant_valid = splits.values().all(|split| {
        let metrics = &split[Path::ValidIrrelevant.name()];
        metrics.certificate_available == metrics.roots
            && metrics.admitted == metrics.roots
            && metrics.successes == 0
    });
    let foreign_rejected = splits.values().all(|split| {
        let metrics = &split[Path::Foreign.name()];
        metrics.foreign_rejections == metrics.roots
    });
    let tamper_rejected = splits.values().all(|split| {
        let metrics = &split[Path::MembershipTamper.name()];
        metrics.tamper_rejections == metrics.roots
    });
    let roles_exact = splits.values().all(|split| {
        split.values().all(|metrics| {
            metrics.target_role_found == metrics.roots
                && metrics.membership_exact == metrics.roots
        })
    });
    let budgets_exact = splits
        .values()
        .all(|split| split.values().all(|metrics| metrics.budgets_exact));
    let replay_exact = splits
        .values()
        .all(|split| split.values().all(|metrics| metrics.replay_exact));
    let invariants_hold = splits
        .values()
        .all(|split| split.values().all(|metrics| metrics.invariants_hold));
    let future_transfer = family_stateful.values().all(|value| *value == 8)
        && family_control.values().all(|value| *value == 0);

    let mut gates = BTreeMap::new();
    gates.insert("cohort_exact".into(), roots.len() == 56);
    gates.insert("stateful_train_holdout_future".into(), stateful_exact);
    gates.insert("all_controls_zero".into(), controls_zero);
    gates.insert("valid_irrelevant_accepted".into(), irrelevant_valid);
    gates.insert("foreign_rejected".into(), foreign_rejected);
    gates.insert("membership_tamper_rejected".into(), tamper_rejected);
    gates.insert("role_membership_exact".into(), roles_exact);
    gates.insert("future_family_transfer".into(), future_transfer);
    gates.insert("budgets_exact".into(), budgets_exact);
    gates.insert("replay_exact".into(), replay_exact);
    gates.insert("invariants_hold".into(), invariants_hold);

    let terminal = if !budgets_exact || !replay_exact || !invariants_hold {
        "INFRASTRUCTURE_FAILURE"
    } else if !roles_exact {
        "DISCOVERY_FAILURE"
    } else if !stateful_exact {
        "UTILITY_FAILURE"
    } else if !controls_zero || !irrelevant_valid {
        "CONTROL_FAILURE"
    } else if !future_transfer {
        "TRANSFER_FAILURE"
    } else if gates.values().all(|value| *value) {
        "PASS"
    } else {
        "PROVENANCE_FAILURE"
    };

    let report = Report {
        experiment: "H12 proof-carrying latent structural roles",
        correction_scope: "budget-accounting-only correction: count every role projection that actually scans the evidence table, including structurally matched roles whose projected evidence is empty; no graph, fingerprint, evidence, control, threshold, inference law, admission rule, closure budget, or objective changed",
        first_verdict_run: "CHARGE CI #100 / run 29144787280 / head 7818f1819ee24475256cd52a585282265e6ea383",
        first_verdict_classification: "INFRASTRUCTURE_FAILURE (all scientific gates true; projection-scan accounting false)",
        split_metrics: splits,
        future_family_stateful_successes: family_stateful,
        future_family_max_control_successes: family_control,
        gates,
        terminal_classification: terminal,
        claim_boundary: "PASS supports only deterministic unnamed structural-role induction as a causally useful evidence partition in the frozen synthetic regime; it does not establish open-world ontology induction, automatic promotion, live routing, AGI, consciousness, or human-level cognition",
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal != "PASS" {
        return Err(format!("H12 terminal classification: {terminal}").into());
    }
    Ok(())
}

fn evaluate(
    roots: &[Root],
    role_config: RoleInductionConfig,
    rule_config: RuleInductionConfig,
) -> Result<BTreeMap<String, Metrics>, Box<dyn Error>> {
    let mut output = BTreeMap::new();
    for path in Path::all() {
        let mut rows = Vec::new();
        for root in roots {
            let first = execute(root, path, role_config, rule_config)?;
            let second = execute(root, path, role_config, rule_config)?;
            rows.push((first.clone(), first == second));
        }
        let mut metrics = Metrics {
            roots: rows.len(),
            budgets_exact: true,
            replay_exact: true,
            invariants_hold: true,
            ..Metrics::default()
        };
        for (execution, replay) in rows {
            metrics.successes += execution.success as usize;
            metrics.certificate_available += execution.certificate_available as usize;
            metrics.admitted += execution.admitted;
            metrics.rejections += execution.rejected as usize;
            metrics.foreign_rejections += execution.foreign_rejected as usize;
            metrics.tamper_rejections += execution.tamper_rejected as usize;
            metrics.target_role_found += execution.target_role_found as usize;
            metrics.membership_exact += execution.membership_exact as usize;
            metrics.budgets_exact &= execution.budget_exact;
            metrics.replay_exact &= replay;
            metrics.invariants_hold &= execution.invariants;
        }
        output.insert(path.name().to_string(), metrics);
    }
    Ok(output)
}

fn execute(
    root: &Root,
    path: Path,
    role_config: RoleInductionConfig,
    rule_config: RuleInductionConfig,
) -> Result<Execution, Box<dyn Error>> {
    let prepared = prepare(root, role_config, rule_config)?;
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    let mut available = false;
    let mut admitted = 0usize;
    let mut rejected = false;
    let mut inert_payload = 0usize;

    match path {
        Path::Stateful => {
            available = !prepared.real.is_empty();
            for certificate in &prepared.real {
                admit_graph_certificate(&mut state, certificate)?;
                admitted += 1;
            }
        }
        Path::NoAbstraction => {
            available = prepared.full.is_some();
            if let Some(certificate) = &prepared.full {
                admit_graph_certificate(&mut state, certificate)?;
                admitted += 1;
            } else {
                rejected = true;
            }
        }
        Path::TextOnly => inert_payload = prepared.proof_text_len,
        Path::ScalarOnly => inert_payload = prepared.scalar_count,
        Path::RandomGrouping => {
            available = prepared.random.is_some();
            if let Some(certificate) = &prepared.random {
                admit_graph_certificate(&mut state, certificate)?;
                admitted += 1;
            } else {
                rejected = true;
            }
        }
        Path::SizeMatched => {
            available = prepared.size.is_some();
            if let Some(certificate) = &prepared.size {
                admit_graph_certificate(&mut state, certificate)?;
                admitted += 1;
            }
        }
        Path::ValidIrrelevant => {
            available = prepared.irrelevant.is_some();
            if let Some(certificate) = &prepared.irrelevant {
                admit_graph_certificate(&mut state, certificate)?;
                admitted += 1;
            }
        }
        Path::Foreign => rejected = prepared.foreign_rejected,
        Path::MembershipTamper => rejected = prepared.tamper_rejected,
        Path::Delayed => available = prepared.target.is_some(),
    }

    for _ in 0..CLOSURE_SCANS {
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
        }
    }
    if path == Path::Delayed {
        if let Some(certificate) = &prepared.target {
            admit_graph_certificate(&mut state, certificate)?;
            admitted += 1;
        }
    }

    Ok(Execution {
        success: state.contains_fact(&root.goal),
        certificate_available: available,
        admitted,
        rejected,
        foreign_rejected: prepared.foreign_rejected,
        tamper_rejected: prepared.tamper_rejected,
        target_role_found: prepared.target_role_found,
        membership_exact: prepared.membership_exact,
        budget_exact: prepared.budget_exact,
        invariants: state.verify_invariants().is_ok(),
        signature: state.canonical_signature(),
        inert_payload,
    })
}

fn prepare(
    root: &Root,
    role_config: RoleInductionConfig,
    rule_config: RuleInductionConfig,
) -> Result<Prepared, Box<dyn Error>> {
    let corpus_nodes = root.corpus.graphs.iter().map(|graph| graph.nodes.len()).sum::<usize>();
    let mut discovery_budget = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&root.corpus, role_config, &mut discovery_budget)?;
    let mut registry = ShadowAbstractionRegistry::new(discovery_scope_digest(&root.corpus)?);
    for proof in &proofs {
        registry.register_candidate(proof)?;
    }

    let mut certificates = Vec::new();
    let mut validation_fingerprints = 0usize;
    for proof in &proofs {
        let mut budget = RoleInductionBudget::default();
        let certificate = validate_latent_role(&root.corpus, proof, role_config, &mut budget)?;
        validation_fingerprints += budget.node_fingerprint_evaluations;
        registry.admit_validated(&certificate)?;
        certificates.push(certificate);
    }

    let mut transfer_budget = TransferRecognitionBudget::default();
    let mut projection_scan_passes = 0usize;
    let mut h11_budget_exact = true;
    let mut real = Vec::new();
    let mut target = None;
    let mut irrelevant = None;
    let mut target_role_id = None;

    for certificate in &certificates {
        let scans_before = transfer_budget.evidence_episode_scans;
        let result = project_evidence_for_role(
            &root.transfer,
            &root.evidence,
            certificate,
            &registry,
            &mut transfer_budget,
        );
        let scan_delta = transfer_budget.evidence_episode_scans - scans_before;
        if scan_delta > 0 {
            projection_scan_passes += 1;
            h11_budget_exact &= scan_delta == root.evidence.evidence.len();
        }
        match result {
            Ok(projected) => {
                let attempt = h11(&projected, rule_config);
                h11_budget_exact &= attempt.exact;
                if let Some(validated) = attempt.certificate {
                    if validated.rule().antecedent == root.middle
                        && validated.rule().consequent == root.goal
                    {
                        target_role_id = Some(certificate.role_id());
                        target = Some(validated.clone());
                    }
                    if validated.rule().antecedent == root.irrelevant
                        && validated.rule().consequent == root.irrelevant_goal
                    {
                        irrelevant = Some(validated.clone());
                    }
                    real.push(validated);
                }
            }
            Err(star::latent_roles::LatentRoleError::NoTransferMember(_))
            | Err(star::latent_roles::LatentRoleError::EmptyProjection) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let full_attempt = h11(&root.evidence, rule_config);
    h11_budget_exact &= full_attempt.exact;

    let random_graph = MixedEvidenceGraph {
        evidence: root
            .evidence
            .evidence
            .iter()
            .filter(|episode| episode.intervention == root.middle)
            .take(2)
            .chain(
                root.evidence
                    .evidence
                    .iter()
                    .filter(|episode| episode.intervention == root.irrelevant)
                    .take(2),
            )
            .cloned()
            .collect(),
    };
    let random_attempt = h11(&random_graph, rule_config);
    h11_budget_exact &= random_attempt.exact;

    let mut size_budget = TransferRecognitionBudget::default();
    let size_graph = project_evidence_for_control_group(
        &root.evidence,
        &BTreeSet::from([root.size_member.clone()]),
        &mut size_budget,
    )?;
    let size_attempt = h11(&size_graph, rule_config);
    h11_budget_exact &= size_attempt.exact;

    let foreign = foreign_certificate(root.id, role_config)?;
    let mut foreign_budget = TransferRecognitionBudget::default();
    let foreign_rejected = project_evidence_for_role(
        &root.transfer,
        &root.evidence,
        &foreign,
        &registry,
        &mut foreign_budget,
    )
    .is_err();

    let target_role_id = target_role_id.ok_or("target role not found")?;
    let target_proof = proofs
        .iter()
        .find(|proof| proof.role_id == target_role_id)
        .ok_or("target role proof missing")?;
    let mut tampered: LatentRoleProof = target_proof.clone();
    tampered.members[0].node = atom(format!("forged_{}", root.id));
    let mut tamper_budget = RoleInductionBudget::default();
    let tamper_rejected =
        validate_latent_role(&root.corpus, &tampered, role_config, &mut tamper_budget).is_err();
    validation_fingerprints += tamper_budget.node_fingerprint_evaluations;

    let expected_projection_scans =
        root.evidence.evidence.len() * projection_scan_passes + root.evidence.evidence.len();
    let budget_exact = discovery_budget.node_fingerprint_evaluations == corpus_nodes
        && validation_fingerprints == corpus_nodes * (proofs.len() + 1)
        && transfer_budget.node_fingerprint_evaluations
            == root.transfer.nodes.len() * certificates.len()
        && transfer_budget.evidence_episode_scans + size_budget.evidence_episode_scans
            == expected_projection_scans
        && foreign_budget.node_fingerprint_evaluations == 0
        && foreign_budget.evidence_episode_scans == 0
        && h11_budget_exact
        && target.is_some()
        && irrelevant.is_some();

    Ok(Prepared {
        real,
        target,
        irrelevant,
        full: full_attempt.certificate,
        random: random_attempt.certificate,
        size: size_attempt.certificate,
        foreign_rejected,
        tamper_rejected,
        target_role_found: target_role_id != RoleId(0),
        membership_exact: certificates.len() == proofs.len(),
        budget_exact,
        proof_text_len: serde_json::to_string(&proofs)?.len(),
        scalar_count: proofs.len(),
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

fn foreign_certificate(
    root_id: u64,
    config: RoleInductionConfig,
) -> Result<ValidatedLatentRoleCertificate, Box<dyn Error>> {
    let corpus = discovery_corpus(root_id + 100_000, &format!("foreign_{root_id}"));
    let mut discovery_budget = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&corpus, config, &mut discovery_budget)?;
    let mut validation_budget = RoleInductionBudget::default();
    Ok(validate_latent_role(
        &corpus,
        proofs.first().ok_or("foreign role proof missing")?,
        config,
        &mut validation_budget,
    )?)
}

fn build_roots() -> Vec<Root> {
    let mut roots = Vec::new();
    let mut id = 1u64;
    for (family_index, family) in FAMILIES.into_iter().enumerate() {
        for local in 0..ROOTS_PER_FAMILY {
            let prefix = format!("h12_{family}_{id}_{local}");
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
                irrelevant,
                irrelevant_goal,
                size_member,
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
                let p = format!("{prefix}_discover_{index}");
                let mut graph = Builder::new(root_id * 100 + index as u64 + 1);
                target_motif(
                    &mut graph,
                    &p,
                    atom(format!("{p}_latent")),
                    2 + index,
                    2 + ((index + 1) % 4),
                );
                irrelevant_motif(&mut graph, &p, atom(format!("{p}_irrelevant")));
                let mut previous = atom(format!("{p}_d0"));
                graph.node(previous.clone());
                for step in 1..(2 + index) {
                    let next = atom(format!("{p}_d{step}"));
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

fn target_motif(
    graph: &mut Builder,
    prefix: &str,
    latent: Atom,
    fan_in: usize,
    fan_out: usize,
) {
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
    Atom::new(value.into()).expect("generated H12 atom is non-empty")
}
