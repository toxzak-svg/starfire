use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::graph_discovery::{
    admit_graph_certificate, infer_graph_rule, validate_graph_rule, FrontierDiscoveryBudget,
    GraphInferenceProof, MixedEvidenceGraph, ValidatedGraphInferenceCertificate,
};
use star::rule_induction::{EvidenceBoundCommitmentState, EvidenceEpisode, RuleInductionConfig, ScoringBudget};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const EXPECTED_RAW_ATOMS: usize = 24;
const EXPECTED_EPISODES: usize = 16;
const EXPECTED_ANTECEDENTS: usize = 6;
const EXPECTED_CONSEQUENTS: usize = 6;
const EXPECTED_CANDIDATES: usize = 36;
const EXPECTED_SCORING_EVALUATIONS: usize = 576;
const EXPECTED_EXECUTOR_SCANS: usize = 3;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_systems",
    "transport_networks",
    "ecological_flows",
    "cellular_regulation",
    "manufacturing_processes",
    "software_dependency",
    "watershed_dynamics",
];

#[derive(Debug, Clone)]
struct RootTask {
    root_id: u64,
    #[allow(dead_code)] // Frozen split-family provenance.
    family: &'static str,
    source: Atom,
    middle: Atom,
    goal: Atom,
    irrelevant_source: Atom,
    #[allow(dead_code)] // Retained negative-control endpoint.
    irrelevant_goal: Atom,
    target_graph: MixedEvidenceGraph,
    irrelevant_graph: MixedEvidenceGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    Stateful,
    EndpointBlind,
    FrontierTextOnly,
    ScalarOnly,
    ForeignProof,
    FrontierTamper,
    ValidIrrelevantDiscovery,
    CounterfeitProof,
    DelayedAdmission,
}

impl PathKind {
    fn all() -> [Self; 9] {
        [
            Self::Stateful,
            Self::EndpointBlind,
            Self::FrontierTextOnly,
            Self::ScalarOnly,
            Self::ForeignProof,
            Self::FrontierTamper,
            Self::ValidIrrelevantDiscovery,
            Self::CounterfeitProof,
            Self::DelayedAdmission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::Stateful => "stateful",
            Self::EndpointBlind => "endpoint_blind",
            Self::FrontierTextOnly => "frontier_text_only",
            Self::ScalarOnly => "scalar_only",
            Self::ForeignProof => "foreign_proof",
            Self::FrontierTamper => "frontier_tamper",
            Self::ValidIrrelevantDiscovery => "valid_irrelevant_discovery",
            Self::CounterfeitProof => "counterfeit_proof",
            Self::DelayedAdmission => "delayed_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Budget {
    proposer_frontier_passes: usize,
    validator_frontier_passes: usize,
    proposer_graph_incidence_scans: usize,
    validator_graph_incidence_scans: usize,
    discovered_antecedents: usize,
    discovered_consequents: usize,
    discovered_candidate_rules: usize,
    proposal_scoring_evaluations: usize,
    validation_recomputations: usize,
    validation_scoring_evaluations: usize,
    admission_slots: usize,
    executor_scans: usize,
    objective_checks: usize,
}

impl Budget {
    fn exact(&self) -> bool {
        self.proposer_frontier_passes == 1
            && self.validator_frontier_passes == 1
            && self.proposer_graph_incidence_scans == EXPECTED_EPISODES
            && self.validator_graph_incidence_scans == EXPECTED_EPISODES
            && self.discovered_antecedents == EXPECTED_ANTECEDENTS
            && self.discovered_consequents == EXPECTED_CONSEQUENTS
            && self.discovered_candidate_rules == EXPECTED_CANDIDATES
            && self.proposal_scoring_evaluations == EXPECTED_SCORING_EVALUATIONS
            && self.validation_recomputations == 1
            && self.validation_scoring_evaluations == EXPECTED_SCORING_EVALUATIONS
            && self.admission_slots == 1
            && self.executor_scans == EXPECTED_EXECUTOR_SCANS
            && self.objective_checks == 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    success: bool,
    discovery_succeeded: bool,
    certificate_accepted: bool,
    certificate_rejected: bool,
    admitted_executable_mutations: usize,
    closure_mutations: usize,
    frontier_digest: u64,
    inferred_rule: String,
    validation_error: Option<String>,
    budget: Budget,
    invariants_hold: bool,
    canonical_state_signature: String,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    roots: usize,
    successes: usize,
    success_rate: f64,
    discovery_successes: usize,
    certificate_acceptances: usize,
    certificate_rejections: usize,
    mean_admitted_executable_mutations: f64,
    frontier_exact: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SplitReport {
    roots: usize,
    paths: BTreeMap<String, PathMetrics>,
}

#[derive(Debug, Clone, Serialize)]
struct FutureFamilyReport {
    family: &'static str,
    roots: usize,
    stateful_success_rate: f64,
    maximum_control_success_rate: f64,
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    raw_atoms: usize,
    evidence_episodes: usize,
    frontier_antecedents: usize,
    frontier_consequents: usize,
    discovered_candidate_rules: usize,
    min_score: i32,
    min_support: usize,
    max_contradictions: usize,
    min_margin: i32,
    proposal_scoring_evaluations: usize,
    validation_scoring_evaluations: usize,
    executor_scans: usize,
    paths_per_root: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    cohort_exact: bool,
    frontier_exact_everywhere: bool,
    stateful_train: bool,
    stateful_holdout: bool,
    stateful_future: bool,
    controls_future_zero: bool,
    foreign_proof_rejected_everywhere: bool,
    frontier_tamper_rejected_everywhere: bool,
    counterfeit_proof_rejected_everywhere: bool,
    valid_irrelevant_accepted_everywhere: bool,
    all_future_families_transfer: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.frontier_exact_everywhere
            && self.stateful_train
            && self.stateful_holdout
            && self.stateful_future
            && self.controls_future_zero
            && self.foreign_proof_rejected_everywhere
            && self.frontier_tamper_rejected_everywhere
            && self.counterfeit_proof_rejected_everywhere
            && self.valid_irrelevant_accepted_everywhere
            && self.all_future_families_transfer
            && self.budgets_exact
            && self.replay_exact
            && self.invariants_hold
    }
}

#[derive(Debug, Serialize)]
struct Report {
    experiment: &'static str,
    mechanism: &'static str,
    claim_boundary: &'static str,
    frozen_contract: FrozenContract,
    training: SplitReport,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    gates: GateReport,
    terminal_classification: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = RuleInductionConfig::default();
    let roots = build_roots()?;

    let train_end = TRAIN_FAMILIES * ROOTS_PER_FAMILY;
    let holdout_end = train_end + HOLDOUT_FAMILIES * ROOTS_PER_FAMILY;
    let train = &roots[..train_end];
    let holdout = &roots[train_end..holdout_end];
    let future = &roots[holdout_end..];

    let training = evaluate_split(train, config)?;
    let holdout_report = evaluate_split(holdout, config)?;
    let future_report = evaluate_split(future, config)?;

    let mut future_families = Vec::new();
    for family_index in 0..FUTURE_FAMILIES {
        let start = holdout_end + family_index * ROOTS_PER_FAMILY;
        let end = start + ROOTS_PER_FAMILY;
        let family_report = evaluate_split(&roots[start..end], config)?;
        let stateful = path_metrics(&family_report, PathKind::Stateful);
        let maximum_control_success_rate = PathKind::all()
            .into_iter()
            .filter(|path| *path != PathKind::Stateful)
            .map(|path| path_metrics(&family_report, path).success_rate)
            .fold(0.0_f64, f64::max);
        future_families.push(FutureFamilyReport {
            family: FAMILIES[TRAIN_FAMILIES + HOLDOUT_FAMILIES + family_index],
            roots: ROOTS_PER_FAMILY,
            stateful_success_rate: stateful.success_rate,
            maximum_control_success_rate,
        });
    }

    let cohort_exact = train.len() == 16
        && holdout.len() == 8
        && future.len() == 32
        && future_families.len() == 4;
    let frontier_exact_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.frontier_exact);
    let stateful_train = path_metrics(&training, PathKind::Stateful).success_rate == 1.0;
    let stateful_holdout = path_metrics(&holdout_report, PathKind::Stateful).success_rate == 1.0;
    let stateful_future = path_metrics(&future_report, PathKind::Stateful).success_rate == 1.0;
    let controls_future_zero = PathKind::all()
        .into_iter()
        .filter(|path| *path != PathKind::Stateful)
        .all(|path| path_metrics(&future_report, path).success_rate == 0.0);
    let foreign_proof_rejected_everywhere = rejection_everywhere(
        [&training, &holdout_report, &future_report],
        PathKind::ForeignProof,
    );
    let frontier_tamper_rejected_everywhere = rejection_everywhere(
        [&training, &holdout_report, &future_report],
        PathKind::FrontierTamper,
    );
    let counterfeit_proof_rejected_everywhere = rejection_everywhere(
        [&training, &holdout_report, &future_report],
        PathKind::CounterfeitProof,
    );
    let valid_irrelevant_accepted_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::ValidIrrelevantDiscovery);
            metrics.certificate_acceptances == metrics.roots
        });
    let all_future_families_transfer = future_families.iter().all(|family| {
        family.stateful_success_rate == 1.0 && family.maximum_control_success_rate == 0.0
    });
    let budgets_exact = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.budgets_exact);
    let replay_exact = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.replay_exact);
    let invariants_hold = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.invariants_hold);

    let gates = GateReport {
        cohort_exact,
        frontier_exact_everywhere,
        stateful_train,
        stateful_holdout,
        stateful_future,
        controls_future_zero,
        foreign_proof_rejected_everywhere,
        frontier_tamper_rejected_everywhere,
        counterfeit_proof_rejected_everywhere,
        valid_irrelevant_accepted_everywhere,
        all_future_families_transfer,
        budgets_exact,
        replay_exact,
        invariants_hold,
    };

    let terminal_classification = if !budgets_exact || !invariants_hold {
        "CONTROL_FAILURE"
    } else if !replay_exact {
        "REPLAY_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "H11 graph-discovered relation induction",
        mechanism: "raw 24-atom mixed intervention graph -> incidence-derived 6x6 candidate frontier -> unchanged H10 scoring over 36 rules -> independent frontier and full-ranking recomputation -> opaque certificate -> PECS executable admission -> fixed three-scan closure",
        claim_boundary: "a PASS supports only graph-incidence candidate-frontier discovery plus evidence-bound symbolic rule induction and state-dependent executable composition under the frozen synthetic regime; it does not establish open-world discovery, learned frontier laws, ontology autonomy, AGI, consciousness, or human-level cognition",
        frozen_contract: FrozenContract {
            raw_atoms: EXPECTED_RAW_ATOMS,
            evidence_episodes: EXPECTED_EPISODES,
            frontier_antecedents: EXPECTED_ANTECEDENTS,
            frontier_consequents: EXPECTED_CONSEQUENTS,
            discovered_candidate_rules: EXPECTED_CANDIDATES,
            min_score: config.min_score,
            min_support: config.min_support,
            max_contradictions: config.max_contradictions,
            min_margin: config.min_margin,
            proposal_scoring_evaluations: EXPECTED_SCORING_EVALUATIONS,
            validation_scoring_evaluations: EXPECTED_SCORING_EVALUATIONS,
            executor_scans: EXPECTED_EXECUTOR_SCANS,
            paths_per_root: PathKind::all().len(),
        },
        training,
        holdout: holdout_report,
        future: future_report,
        future_families,
        gates,
        terminal_classification,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn rejection_everywhere<'a>(
    splits: impl IntoIterator<Item = &'a SplitReport>,
    path: PathKind,
) -> bool {
    splits.into_iter().all(|split| {
        let metrics = path_metrics(split, path);
        metrics.certificate_rejections == metrics.roots
    })
}

fn evaluate_split(
    roots: &[RootTask],
    config: RuleInductionConfig,
) -> Result<SplitReport, Box<dyn Error>> {
    let donor_proofs = precompute_donor_proofs(roots, config)?;
    let mut executions = BTreeMap::<PathKind, Vec<(Execution, bool)>>::new();

    for (index, root) in roots.iter().enumerate() {
        let donor_index = (index + 1) % roots.len().max(1);
        let donor_proof = donor_proofs.get(donor_index).ok_or("missing donor proof")?;
        for path in PathKind::all() {
            let first = execute_path(root, path, donor_proof, config)?;
            let second = execute_path(root, path, donor_proof, config)?;
            let replay_exact = first == second;
            executions.entry(path).or_default().push((first, replay_exact));
        }
    }

    let paths = PathKind::all()
        .into_iter()
        .map(|path| {
            let rows = executions.get(&path).map(Vec::as_slice).unwrap_or(&[]);
            let roots = rows.len();
            let successes = rows.iter().filter(|(execution, _)| execution.success).count();
            let discovery_successes = rows
                .iter()
                .filter(|(execution, _)| execution.discovery_succeeded)
                .count();
            let certificate_acceptances = rows
                .iter()
                .filter(|(execution, _)| execution.certificate_accepted)
                .count();
            let certificate_rejections = rows
                .iter()
                .filter(|(execution, _)| execution.certificate_rejected)
                .count();
            let mean_admitted_executable_mutations = rows
                .iter()
                .map(|(execution, _)| execution.admitted_executable_mutations as f64)
                .sum::<f64>()
                / roots.max(1) as f64;
            let frontier_exact = rows.iter().all(|(execution, _)| {
                execution.budget.discovered_antecedents == EXPECTED_ANTECEDENTS
                    && execution.budget.discovered_consequents == EXPECTED_CONSEQUENTS
                    && execution.budget.discovered_candidate_rules == EXPECTED_CANDIDATES
            });
            let metrics = PathMetrics {
                roots,
                successes,
                success_rate: successes as f64 / roots.max(1) as f64,
                discovery_successes,
                certificate_acceptances,
                certificate_rejections,
                mean_admitted_executable_mutations,
                frontier_exact,
                budgets_exact: rows.iter().all(|(execution, _)| execution.budget.exact()),
                replay_exact: rows.iter().all(|(_, replay)| *replay),
                invariants_hold: rows
                    .iter()
                    .all(|(execution, _)| execution.invariants_hold),
            };
            (path.name().to_string(), metrics)
        })
        .collect();

    Ok(SplitReport {
        roots: roots.len(),
        paths,
    })
}

fn path_metrics(report: &SplitReport, path: PathKind) -> &PathMetrics {
    report.paths.get(path.name()).expect("missing path metrics")
}

fn precompute_donor_proofs(
    roots: &[RootTask],
    config: RuleInductionConfig,
) -> Result<Vec<GraphInferenceProof>, Box<dyn Error>> {
    roots
        .iter()
        .map(|root| {
            let mut frontier_budget = FrontierDiscoveryBudget::default();
            let mut scoring_budget = ScoringBudget::default();
            let proof = infer_graph_rule(
                &root.target_graph,
                config,
                &mut frontier_budget,
                &mut scoring_budget,
            )?;
            if frontier_budget.evidence_episode_scans != EXPECTED_EPISODES
                || frontier_budget.discovered_antecedents != EXPECTED_ANTECEDENTS
                || frontier_budget.discovered_consequents != EXPECTED_CONSEQUENTS
                || scoring_budget.candidate_episode_evaluations != EXPECTED_SCORING_EVALUATIONS
            {
                return Err("donor proof construction budget mismatch".into());
            }
            Ok(proof)
        })
        .collect()
}

fn execute_path(
    root: &RootTask,
    path: PathKind,
    foreign_proof: &GraphInferenceProof,
    config: RuleInductionConfig,
) -> Result<Execution, Box<dyn Error>> {
    let graph = if path == PathKind::ValidIrrelevantDiscovery {
        &root.irrelevant_graph
    } else {
        &root.target_graph
    };

    let mut proposal_frontier_budget = FrontierDiscoveryBudget::default();
    let mut proposal_scoring_budget = ScoringBudget::default();
    let current_proof = infer_graph_rule(
        graph,
        config,
        &mut proposal_frontier_budget,
        &mut proposal_scoring_budget,
    )?;
    let discovery_succeeded = true;

    let proof_for_validation = match path {
        PathKind::ForeignProof => foreign_proof.clone(),
        PathKind::FrontierTamper => {
            let mut tampered = current_proof.clone();
            tampered.discovered_antecedents[0] = atom(&format!(
                "h11_frontier_tamper_root_{}",
                root.root_id
            ))?;
            tampered
        }
        PathKind::CounterfeitProof => {
            let mut counterfeit = current_proof.clone();
            counterfeit.score += 1;
            counterfeit
        }
        _ => current_proof.clone(),
    };

    let mut validation_frontier_budget = FrontierDiscoveryBudget::default();
    let mut validation_scoring_budget = ScoringBudget::default();
    let validation_result = validate_graph_rule(
        graph,
        &proof_for_validation,
        config,
        &mut validation_frontier_budget,
        &mut validation_scoring_budget,
    );
    let certificate_accepted = validation_result.is_ok();
    let certificate_rejected = validation_result.is_err();
    let validation_error = validation_result.as_ref().err().map(ToString::to_string);
    let certificate: Option<ValidatedGraphInferenceCertificate> = validation_result.ok();

    let mut state = initial_state(root)?;
    let mut admitted_executable_mutations = 0_usize;
    let mut closure_mutations = 0_usize;
    let mut text_history = String::new();
    let mut scalar_history = (0_usize, 0_usize, 0_i32, 0_i32);

    match path {
        PathKind::Stateful | PathKind::ValidIrrelevantDiscovery => {
            if let Some(certificate) = certificate.as_ref() {
                admit_graph_certificate(&mut state, certificate)?;
                admitted_executable_mutations += 1;
            }
        }
        PathKind::FrontierTextOnly => {
            text_history = serde_json::to_string(&proof_for_validation)?;
        }
        PathKind::ScalarOnly => {
            scalar_history = (
                proof_for_validation.discovered_antecedents.len(),
                proof_for_validation.discovered_consequents.len(),
                proof_for_validation.score,
                proof_for_validation.score - proof_for_validation.runner_up_score,
            );
        }
        PathKind::EndpointBlind
        | PathKind::ForeignProof
        | PathKind::FrontierTamper
        | PathKind::CounterfeitProof
        | PathKind::DelayedAdmission => {}
    }

    for _ in 0..EXPECTED_EXECUTOR_SCANS {
        if let Some(delta) = state.enabled_derivations().into_iter().next() {
            state.apply_delta(delta)?;
            closure_mutations += 1;
        }
    }

    if path == PathKind::DelayedAdmission {
        if let Some(certificate) = certificate.as_ref() {
            admit_graph_certificate(&mut state, certificate)?;
            admitted_executable_mutations += 1;
        }
    }

    let _non_executable_history = (text_history, scalar_history);
    let success = state.contains_fact(&root.goal);
    let invariants_hold = state.verify_invariants().is_ok();
    let canonical_state_signature = state.canonical_signature();

    let budget = Budget {
        proposer_frontier_passes: 1,
        validator_frontier_passes: 1,
        proposer_graph_incidence_scans: proposal_frontier_budget.evidence_episode_scans,
        validator_graph_incidence_scans: validation_frontier_budget.evidence_episode_scans,
        discovered_antecedents: proposal_frontier_budget.discovered_antecedents,
        discovered_consequents: proposal_frontier_budget.discovered_consequents,
        discovered_candidate_rules: proposal_frontier_budget
            .discovered_antecedents
            .saturating_mul(proposal_frontier_budget.discovered_consequents),
        proposal_scoring_evaluations: proposal_scoring_budget.candidate_episode_evaluations,
        validation_recomputations: 1,
        validation_scoring_evaluations: validation_scoring_budget.candidate_episode_evaluations,
        admission_slots: 1,
        executor_scans: EXPECTED_EXECUTOR_SCANS,
        objective_checks: 1,
    };

    Ok(Execution {
        success,
        discovery_succeeded,
        certificate_accepted,
        certificate_rejected,
        admitted_executable_mutations,
        closure_mutations,
        frontier_digest: current_proof.frontier_digest,
        inferred_rule: rule_name(&current_proof.rule),
        validation_error,
        budget,
        invariants_hold,
        canonical_state_signature,
    })
}

fn initial_state(root: &RootTask) -> Result<EvidenceBoundCommitmentState, Box<dyn Error>> {
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_fact(root.irrelevant_source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    Ok(state)
}

fn build_roots() -> Result<Vec<RootTask>, Box<dyn Error>> {
    let mut roots = Vec::new();
    let mut root_id = 1_u64;
    for family in FAMILIES {
        for index in 0..ROOTS_PER_FAMILY {
            roots.push(build_root(root_id, family, index)?);
            root_id += 1;
        }
    }
    Ok(roots)
}

fn build_root(root_id: u64, family: &'static str, index: usize) -> Result<RootTask, Box<dyn Error>> {
    let prefix = format!("{family}_{index}");
    let source = atom(&format!("{prefix}_source"))?;
    let middle = atom(&format!("{prefix}_middle"))?;
    let goal = atom(&format!("{prefix}_goal"))?;
    let irrelevant_source = atom(&format!("{prefix}_irrelevant_source"))?;
    let irrelevant_goal = atom(&format!("{prefix}_irrelevant_goal"))?;

    let target_graph = build_graph(
        root_id,
        &prefix,
        &middle,
        &goal,
        &irrelevant_source,
        &irrelevant_goal,
        true,
    )?;
    let irrelevant_graph = build_graph(
        root_id,
        &prefix,
        &middle,
        &goal,
        &irrelevant_source,
        &irrelevant_goal,
        false,
    )?;

    for graph in [&target_graph, &irrelevant_graph] {
        if graph.evidence.len() != EXPECTED_EPISODES {
            return Err("H11 frozen evidence episode count mismatch".into());
        }
        if count_graph_atoms(graph) != EXPECTED_RAW_ATOMS {
            return Err("H11 frozen raw atom count mismatch".into());
        }
        let mut frontier_budget = FrontierDiscoveryBudget::default();
        let mut scoring_budget = ScoringBudget::default();
        let proof = infer_graph_rule(
            graph,
            RuleInductionConfig::default(),
            &mut frontier_budget,
            &mut scoring_budget,
        )?;
        if frontier_budget.discovered_antecedents != EXPECTED_ANTECEDENTS
            || frontier_budget.discovered_consequents != EXPECTED_CONSEQUENTS
            || scoring_budget.candidate_episode_evaluations != EXPECTED_SCORING_EVALUATIONS
            || proof.score != 12
            || proof.support != 4
            || proof.runner_up_score != 6
        {
            return Err("H11 frozen graph shape or score contract mismatch".into());
        }
    }

    Ok(RootTask {
        root_id,
        family,
        source,
        middle,
        goal,
        irrelevant_source,
        irrelevant_goal,
        target_graph,
        irrelevant_graph,
    })
}

fn build_graph(
    root_id: u64,
    prefix: &str,
    middle: &Atom,
    goal: &Atom,
    irrelevant_source: &Atom,
    irrelevant_goal: &Atom,
    target_primary: bool,
) -> Result<MixedEvidenceGraph, Box<dyn Error>> {
    let distractor_antecedents = (0..4)
        .map(|index| atom(&format!("{prefix}_distractor_a{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let distractor_consequents = (0..4)
        .map(|index| atom(&format!("{prefix}_distractor_b{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let noise_interventions = (0..2)
        .map(|index| atom(&format!("{prefix}_noise_intervention_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let noise_outcomes = (0..10)
        .map(|index| atom(&format!("{prefix}_noise_outcome_{index}")))
        .collect::<Result<Vec<_>, _>>()?;

    let target_support = if target_primary { 4 } else { 2 };
    let irrelevant_support = if target_primary { 2 } else { 4 };
    let mut episode_specs = Vec::<(Atom, Option<Atom>)>::new();

    for _ in 0..target_support {
        episode_specs.push((middle.clone(), Some(goal.clone())));
    }
    for _ in 0..irrelevant_support {
        episode_specs.push((
            irrelevant_source.clone(),
            Some(irrelevant_goal.clone()),
        ));
    }
    for index in 0..4 {
        for _ in 0..2 {
            episode_specs.push((
                distractor_antecedents[index].clone(),
                Some(distractor_consequents[index].clone()),
            ));
        }
    }
    episode_specs.push((noise_interventions[0].clone(), None));
    episode_specs.push((noise_interventions[1].clone(), None));

    if episode_specs.len() != EXPECTED_EPISODES {
        return Err("internal H11 episode construction mismatch".into());
    }

    let graph_offset = if target_primary { 0_u64 } else { 5_000_u64 };
    let base_id = root_id * 10_000 + graph_offset;
    let mut evidence = Vec::with_capacity(EXPECTED_EPISODES);
    for (index, (intervention, structured_outcome)) in episode_specs.into_iter().enumerate() {
        let mut outcomes = BTreeSet::new();
        if let Some(outcome) = structured_outcome {
            outcomes.insert(outcome);
        }
        if index < noise_outcomes.len() {
            outcomes.insert(noise_outcomes[index].clone());
        }
        evidence.push(EvidenceEpisode {
            evidence_id: base_id + index as u64 + 1,
            intervention,
            outcomes,
        });
    }

    Ok(MixedEvidenceGraph { evidence })
}

fn count_graph_atoms(graph: &MixedEvidenceGraph) -> usize {
    let mut atoms = BTreeSet::new();
    for episode in &graph.evidence {
        atoms.insert(episode.intervention.clone());
        atoms.extend(episode.outcomes.iter().cloned());
    }
    atoms.len()
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}

fn rule_name(rule: &Rule) -> String {
    format!(
        "{} -> {}",
        rule.antecedent.as_str(),
        rule.consequent.as_str()
    )
}
