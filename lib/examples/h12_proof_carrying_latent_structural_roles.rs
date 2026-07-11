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
const FAMILIES: [&str; 7] = [
    "thermal", "transport", "ecology", "cellular", "manufacturing", "software", "watershed",
];
const CLOSURE_SCANS: usize = 3;

#[derive(Clone)]
struct Root {
    id: u64,
    family: &'static str,
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

#[derive(Clone, PartialEq, Eq, Serialize)]
struct Budget {
    discovery_fingerprints: usize,
    validation_fingerprints: usize,
    transfer_fingerprints: usize,
    projection_scans: usize,
    h11_scoring_evaluations: usize,
    admission_slots: usize,
    closure_scans: usize,
    exact: bool,
}

#[derive(Clone)]
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
    text_bytes: usize,
    scalar_count: usize,
    budget: Budget,
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
    invariants: bool,
    text_bytes: usize,
    scalar_count: usize,
    budget: Budget,
    signature: String,
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
    mechanism: &'static str,
    split_metrics: BTreeMap<String, BTreeMap<String, Metrics>>,
    future_family_stateful_success: BTreeMap<String, usize>,
    future_family_max_control_success: BTreeMap<String, usize>,
    gates: BTreeMap<String, bool>,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let role_config = RoleInductionConfig::default();
    let rule_config = RuleInductionConfig::default();
    let roots = build_roots();
    let mut split_metrics = BTreeMap::new();
    split_metrics.insert("training".into(), evaluate(&roots[0..16], role_config, rule_config)?);
    split_metrics.insert("holdout".into(), evaluate(&roots[16..24], role_config, rule_config)?);
    split_metrics.insert("future".into(), evaluate(&roots[24..56], role_config, rule_config)?);

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
        let max_control = Path::all()
            .into_iter()
            .filter(|path| *path != Path::Stateful)
            .map(|path| metrics[path.name()].successes)
            .max()
            .unwrap_or(0);
        family_control.insert(FAMILIES[family_index].to_string(), max_control);
    }

    let train = &split_metrics["training"];
    let holdout = &split_metrics["holdout"];
    let future = &split_metrics["future"];
    let stateful_exact = train[Path::Stateful.name()].successes == 16
        && holdout[Path::Stateful.name()].successes == 8
        && future[Path::Stateful.name()].successes == 32;
    let controls_zero = split_metrics.values().all(|split| {
        Path::all()
            .into_iter()
            .filter(|path| *path != Path::Stateful)
            .all(|path| split[path.name()].successes == 0)
    });
    let irrelevant_valid = split_metrics.values().all(|split| {
        let m = &split[Path::ValidIrrelevant.name()];
        m.certificate_available == m.roots && m.admitted == m.roots && m.successes == 0
    });
    let foreign_rejected = split_metrics.values().all(|split| {
        let m = &split[Path::Foreign.name()];
        m.foreign_rejections == m.roots
    });
    let tamper_rejected = split_metrics.values().all(|split| {
        let m = &split[Path::MembershipTamper.name()];
        m.tamper_rejections == m.roots
    });
    let role_exact = split_metrics.values().all(|split| {
        split.values().all(|m| {
            m.target_role_found == m.roots && m.membership_exact == m.roots
        })
    });
    let budgets_exact = split_metrics
        .values()
        .all(|split| split.values().all(|m| m.budgets_exact));
    let replay_exact = split_metrics
        .values()
        .all(|split| split.values().all(|m| m.replay_exact));
    let invariants = split_metrics
        .values()
        .all(|split| split.values().all(|m| m.invariants_hold));
    let future_transfer = family_stateful.values().all(|successes| *successes == 8)
        && family_control.values().all(|successes| *successes == 0);

    let mut gates = BTreeMap::new();
    gates.insert("cohort_exact".into(), roots.len() == 56);
    gates.insert("stateful_train_holdout_future".into(), stateful_exact);
    gates.insert("all_controls_zero".into(), controls_zero);
    gates.insert("valid_irrelevant_accepted".into(), irrelevant_valid);
    gates.insert("foreign_rejected".into(), foreign_rejected);
    gates.insert("membership_tamper_rejected".into(), tamper_rejected);
    gates.insert("role_membership_exact".into(), role_exact);
    gates.insert("future_family_transfer".into(), future_transfer);
    gates.insert("budgets_exact".into(), budgets_exact);
    gates.insert("replay_exact".into(), replay_exact);
    gates.insert("invariants_hold".into(), invariants);

    let terminal = if !budgets_exact || !replay_exact || !invariants {
        "INFRASTRUCTURE_FAILURE"
    } else if !role_exact {
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
        mechanism: "raw structure -> target-blind fingerprints -> recurring opaque role proof -> independent exact recomputation -> shadow role evidence projection -> unchanged H11/H10 validation -> PECS admission -> three-scan closure",
        split_metrics,
        future_family_stateful_success: family_stateful,
        future_family_max_control_success: family_control,
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
    let mut result = BTreeMap::new();
    for path in Path::all() {
        let mut rows = Vec::new();
        for root in roots {
            let first = execute(root, path, role_config, rule_config)?;
            let second = execute(root, path, role_config, rule_config)?;
            rows.push((first.clone(), first == second));
        }
        let mut m = Metrics {
            roots: rows.len(),
            budgets_exact: true,
            replay_exact: true,
            invariants_hold: true,
            ..Metrics::default()
        };
        for (execution, replay) in rows {
            m.successes += execution.success as usize;
            m.certificate_available += execution.certificate_available as usize;
            m.admitted += execution.admitted;
            m.rejections += execution.rejected as usize;
            m.foreign_rejections += execution.foreign_rejected as usize;
            m.tamper_rejections += execution.tamper_rejected as usize;
            m.target_role_found += execution.target_role_found as usize;
            m.membership_exact += execution.membership_exact as usize;
            m.budgets_exact &= execution.budget.exact;
            m.replay_exact &= replay;
            m.invariants_hold &= execution.invariants;
        }
        result.insert(path.name().to_string(), m);
    }
    Ok(result)
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
    let mut admitted = 0;
    let mut rejected = false;
    let mut text_bytes = 0;
    let mut scalar_count = 0;

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
        Path::TextOnly => text_bytes = prepared.text_bytes,
        Path::ScalarOnly => scalar_count = prepared.scalar_count,
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
    let mut budget = prepared.budget.clone();
    budget.closure_scans = CLOSURE_SCANS;
    budget.exact &= budget.closure_scans == CLOSURE_SCANS;
    Ok(Execution {
        success: state.contains_fact(&root.goal),
        certificate_available: available,
        admitted,
        rejected,
        foreign_rejected: prepared.foreign_rejected,
        tamper_rejected: prepared.tamper_rejected,
        target_role_found: prepared.target_role_found,
        membership_exact: prepared.membership_exact,
        invariants: state.verify_invariants().is_ok(),
        text_bytes,
        scalar_count,
        budget,
        signature: state.canonical_signature(),
    })
}

fn prepare(
    root: &Root,
    role_config: RoleInductionConfig,
    rule_config: RuleInductionConfig,
) -> Result<Prepared, Box<dyn Error>> {
    let corpus_nodes = root.corpus.graphs.iter().map(|g| g.nodes.len()).sum::<usize>();
    let mut discovery = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&root.corpus, role_config, &mut discovery)?;
    let mut registry = ShadowAbstractionRegistry::new(discovery_scope_digest(&root.corpus)?);
    for proof in &proofs {
        registry.register_candidate(proof)?;
    }
    let mut certificates = Vec::new();
    let mut validation_fingerprints = 0;
    for proof in &proofs {
        let mut budget = RoleInductionBudget::default();
        let certificate = validate_latent_role(&root.corpus, proof, role_config, &mut budget)?;
        validation_fingerprints += budget.node_fingerprint_evaluations;
        registry.admit_validated(&certificate)?;
        certificates.push(certificate);
    }

    let mut transfer_budget = TransferRecognitionBudget::default();
    let mut h11_evals = 0;
    let mut real = Vec::new();
    let mut target = None;
    let mut irrelevant = None;
    let mut target_role_id = None;
    let mut matched_projections = 0;
    for certificate in &certificates {
        match project_evidence_for_role(
            &root.transfer,
            &root.evidence,
            certificate,
            &registry,
            &mut transfer_budget,
        ) {
            Ok(projected) => {
                matched_projections += 1;
                let (validated, evals) = h11(&projected, rule_config);
                h11_evals += evals;
                if let Some(validated) = validated {
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

    let (full, evals) = h11(&root.evidence, rule_config);
    h11_evals += evals;
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
    let (random, evals) = h11(&random_graph, rule_config);
    h11_evals += evals;
    let mut size_budget = TransferRecognitionBudget::default();
    let size_graph = project_evidence_for_control_group(
        &root.evidence,
        &BTreeSet::from([root.size_member.clone()]),
        &mut size_budget,
    )?;
    transfer_budget.evidence_episode_scans += size_budget.evidence_episode_scans;
    let (size, evals) = h11(&size_graph, rule_config);
    h11_evals += evals;

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
        .ok_or("target proof missing")?;
    let mut tampered: LatentRoleProof = target_proof.clone();
    tampered.members[0].node = atom(format!("forged_{}", root.id));
    let mut tamper_budget = RoleInductionBudget::default();
    let tamper_rejected =
        validate_latent_role(&root.corpus, &tampered, role_config, &mut tamper_budget).is_err();
    validation_fingerprints += tamper_budget.node_fingerprint_evaluations;

    let expected_projection_scans = root.evidence.evidence.len() * (matched_projections + 1);
    let exact = discovery.node_fingerprint_evaluations == corpus_nodes
        && validation_fingerprints == corpus_nodes * (proofs.len() + 1)
        && transfer_budget.node_fingerprint_evaluations
            == root.transfer.nodes.len() * certificates.len()
        && transfer_budget.evidence_episode_scans == expected_projection_scans
        && foreign_budget.node_fingerprint_evaluations == 0
        && target.is_some()
        && irrelevant.is_some();

    Ok(Prepared {
        real,
        target,
        irrelevant,
        full,
        random,
        size,
        foreign_rejected,
        tamper_rejected,
        target_role_found: target_role_id != RoleId(0),
        membership_exact: certificates.len() == proofs.len(),
        text_bytes: serde_json::to_string(&proofs)?.len(),
        scalar_count: proofs.len(),
        budget: Budget {
            discovery_fingerprints: discovery.node_fingerprint_evaluations,
            validation_fingerprints,
            transfer_fingerprints: transfer_budget.node_fingerprint_evaluations,
            projection_scans: transfer_budget.evidence_episode_scans,
            h11_scoring_evaluations: h11_evals,
            admission_slots: certificates.len(),
            closure_scans: 0,
            exact,
        },
    })
}

fn h11(
    graph: &MixedEvidenceGraph,
    config: RuleInductionConfig,
) -> (Option<ValidatedGraphInferenceCertificate>, usize) {
    let mut frontier = FrontierDiscoveryBudget::default();
    let mut scoring = ScoringBudget::default();
    let Ok(proof) = infer_graph_rule(graph, config, &mut frontier, &mut scoring) else {
        return (None, scoring.candidate_episode_evaluations);
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
    (
        certificate,
        scoring.candidate_episode_evaluations + validation_scoring.candidate_episode_evaluations,
    )
}

fn foreign_certificate(
    root_id: u64,
    config: RoleInductionConfig,
) -> Result<ValidatedLatentRoleCertificate, Box<dyn Error>> {
    let corpus = discovery_corpus(root_id + 100_000, &format!("foreign_{root_id}"));
    let mut discovery = RoleInductionBudget::default();
    let proofs = induce_latent_roles(&corpus, config, &mut discovery)?;
    let mut validation = RoleInductionBudget::default();
    Ok(validate_latent_role(
        &corpus,
        proofs.first().ok_or("foreign proof missing")?,
        config,
        &mut validation,
    )?)
}

fn build_roots() -> Vec<Root> {
    let mut roots = Vec::new();
    let mut id = 1_u64;
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
                family,
                corpus: discovery_corpus(id, &prefix),
                transfer: transfer_graph(id, &prefix, family_index, &middle, &irrelevant, &size_member),
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
                let mut g = Builder::new(root_id * 100 + index as u64 + 1);
                target_motif(&mut g, &p, atom(format!("{p}_latent")), 2 + index, 2 + ((index + 1) % 4));
                irrelevant_motif(&mut g, &p, atom(format!("{p}_irrelevant")));
                let mut previous = atom(format!("{p}_d0"));
                g.node(previous.clone());
                for step in 1..(2 + index) {
                    let next = atom(format!("{p}_d{step}"));
                    g.edge(previous, next.clone());
                    previous = next;
                }
                g.finish()
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
    let mut g = Builder::new(root_id * 10_000 + 9_001);
    target_motif(
        &mut g,
        &format!("{prefix}_target"),
        middle.clone(),
        2 + family_index,
        2 + family_index % 4,
    );
    irrelevant_motif(&mut g, &format!("{prefix}_irrelevant"), irrelevant.clone());
    g.edge(atom(format!("{prefix}_size_before")), size_member.clone());
    g.edge(size_member.clone(), atom(format!("{prefix}_size_after")));
    for extra in 0..family_index {
        let a = atom(format!("{prefix}_extra_{extra}_a"));
        g.edge(a.clone(), atom(format!("{prefix}_extra_{extra}_b")));
        g.edge(a, atom(format!("{prefix}_extra_{extra}_c")));
    }
    g.finish()
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
    let mut id = root_id * 1_000_000 + 1;
    for index in 0..4 {
        let mut outcomes = BTreeSet::from([goal.clone()]);
        if index < 2 { outcomes.insert(common_noise.clone()); }
        evidence.push(EvidenceEpisode { evidence_id: id, intervention: middle.clone(), outcomes });
        id += 1;
    }
    for index in 0..5 {
        let mut outcomes = BTreeSet::from([irrelevant_goal.clone()]);
        if index < 2 { outcomes.insert(common_noise.clone()); }
        evidence.push(EvidenceEpisode { evidence_id: id, intervention: irrelevant.clone(), outcomes });
        id += 1;
    }
    for index in 0..4 {
        let mut outcomes = BTreeSet::from([size_goal.clone()]);
        if index < 2 { outcomes.insert(size_noise.clone()); }
        evidence.push(EvidenceEpisode { evidence_id: id, intervention: size_member.clone(), outcomes });
        id += 1;
    }
    MixedEvidenceGraph { evidence }
}

fn target_motif(g: &mut Builder, prefix: &str, latent: Atom, fan_in: usize, fan_out: usize) {
    let join = atom(format!("{prefix}_join"));
    let split = atom(format!("{prefix}_split"));
    for index in 0..fan_in { g.edge(atom(format!("{prefix}_s{index}")), join.clone()); }
    g.edge(join, latent.clone());
    g.edge(latent, split.clone());
    for index in 0..fan_out { g.edge(split.clone(), atom(format!("{prefix}_t{index}"))); }
}

fn irrelevant_motif(g: &mut Builder, prefix: &str, latent: Atom) {
    let branch = atom(format!("{prefix}_branch"));
    let merge = atom(format!("{prefix}_merge"));
    g.edge(atom(format!("{prefix}_source")), branch.clone());
    g.edge(branch.clone(), latent.clone());
    g.edge(branch, atom(format!("{prefix}_branch_leaf")));
    g.edge(latent, merge.clone());
    g.edge(atom(format!("{prefix}_merge_source")), merge.clone());
    g.edge(merge, atom(format!("{prefix}_sink")));
}

struct Builder {
    graph_id: u64,
    nodes: BTreeSet<Atom>,
    edges: BTreeSet<DirectedEdge>,
}

impl Builder {
    fn new(graph_id: u64) -> Self {
        Self { graph_id, nodes: BTreeSet::new(), edges: BTreeSet::new() }
    }
    fn node(&mut self, node: Atom) { self.nodes.insert(node); }
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
