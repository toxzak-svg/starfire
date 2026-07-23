use serde::Serialize;
use star::commitment_state::{Atom, Rule};
use star::rule_induction::{
    expected_scoring_evaluations, infer_rule, validate_rule_inference,
    EvidenceBoundCommitmentState, EvidenceEpisode, InferenceProblem, RuleInductionConfig,
    RuleInferenceProof, ScoringBudget, ValidatedInferenceCertificate,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const EXPECTED_CANDIDATES: usize = 9;
const EXPECTED_EPISODES: usize = 10;
const EXPECTED_SCORING_EVALUATIONS: usize = 90;
const EXPECTED_VALIDATION_EVALUATIONS: usize = 90;
const EXPECTED_EXECUTOR_SCANS: usize = 3;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal",
    "network",
    "ecology",
    "biology",
    "manufacturing",
    "software",
    "hydrology",
];

#[derive(Debug, Clone)]
struct RootTask {
    #[allow(dead_code)] // Frozen root provenance.
    root_id: u64,
    #[allow(dead_code)] // Frozen split-family provenance.
    family: &'static str,
    source: Atom,
    middle: Atom,
    goal: Atom,
    decoy_goal: Atom,
    problem: InferenceProblem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    Stateful,
    EndpointBlind,
    TextOnly,
    ScalarOnly,
    RewiredForeignProof,
    ValidPermutedInference,
    CounterfeitProof,
    DelayedAdmission,
}

impl PathKind {
    fn all() -> [Self; 8] {
        [
            Self::Stateful,
            Self::EndpointBlind,
            Self::TextOnly,
            Self::ScalarOnly,
            Self::RewiredForeignProof,
            Self::ValidPermutedInference,
            Self::CounterfeitProof,
            Self::DelayedAdmission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::Stateful => "stateful",
            Self::EndpointBlind => "endpoint_blind",
            Self::TextOnly => "text_only",
            Self::ScalarOnly => "scalar_only",
            Self::RewiredForeignProof => "rewired_foreign_proof",
            Self::ValidPermutedInference => "valid_permuted_inference",
            Self::CounterfeitProof => "counterfeit_proof",
            Self::DelayedAdmission => "delayed_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Budget {
    inference_calls: usize,
    candidates_scored: usize,
    evidence_episodes_per_candidate: usize,
    proposal_scoring_evaluations: usize,
    validation_recomputations: usize,
    validation_scoring_evaluations: usize,
    admission_slots: usize,
    executor_scans: usize,
    objective_checks: usize,
}

impl Budget {
    fn exact(&self) -> bool {
        self.inference_calls == 1
            && self.candidates_scored == EXPECTED_CANDIDATES
            && self.evidence_episodes_per_candidate == EXPECTED_EPISODES
            && self.proposal_scoring_evaluations == EXPECTED_SCORING_EVALUATIONS
            && self.validation_recomputations == 1
            && self.validation_scoring_evaluations == EXPECTED_VALIDATION_EVALUATIONS
            && self.admission_slots == 1
            && self.executor_scans == EXPECTED_EXECUTOR_SCANS
            && self.objective_checks == 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    success: bool,
    inference_succeeded: bool,
    certificate_accepted: bool,
    certificate_rejected: bool,
    admitted_executable_mutations: usize,
    closure_mutations: usize,
    inferred_rule: Option<String>,
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
    inference_successes: usize,
    certificate_acceptances: usize,
    certificate_rejections: usize,
    mean_admitted_executable_mutations: f64,
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
    candidate_antecedents: usize,
    candidate_consequents: usize,
    candidate_rules: usize,
    evidence_episodes: usize,
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
    stateful_train: bool,
    stateful_holdout: bool,
    stateful_future: bool,
    controls_future_zero: bool,
    foreign_proof_rejected_everywhere: bool,
    counterfeit_proof_rejected_everywhere: bool,
    valid_permuted_accepted_everywhere: bool,
    all_future_families_transfer: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.stateful_train
            && self.stateful_holdout
            && self.stateful_future
            && self.controls_future_zero
            && self.foreign_proof_rejected_everywhere
            && self.counterfeit_proof_rejected_everywhere
            && self.valid_permuted_accepted_everywhere
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
    let stateful_train = path_metrics(&training, PathKind::Stateful).success_rate == 1.0;
    let stateful_holdout = path_metrics(&holdout_report, PathKind::Stateful).success_rate == 1.0;
    let stateful_future = path_metrics(&future_report, PathKind::Stateful).success_rate == 1.0;
    let controls_future_zero = PathKind::all()
        .into_iter()
        .filter(|path| *path != PathKind::Stateful)
        .all(|path| path_metrics(&future_report, path).success_rate == 0.0);
    let foreign_proof_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::RewiredForeignProof);
            metrics.certificate_rejections == metrics.roots
        });
    let counterfeit_proof_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::CounterfeitProof);
            metrics.certificate_rejections == metrics.roots
        });
    let valid_permuted_accepted_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::ValidPermutedInference);
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
        stateful_train,
        stateful_holdout,
        stateful_future,
        controls_future_zero,
        foreign_proof_rejected_everywhere,
        counterfeit_proof_rejected_everywhere,
        valid_permuted_accepted_everywhere,
        all_future_families_transfer,
        budgets_exact,
        replay_exact,
        invariants_hold,
    };

    let terminal_classification = if !budgets_exact || !invariants_hold {
        "INFRASTRUCTURE_FAILURE"
    } else if !replay_exact {
        "REPLAY_FAILURE"
    } else if !stateful_train {
        "INFERENCE_FAILURE"
    } else if !stateful_holdout || !stateful_future || !all_future_families_transfer {
        "TRANSFER_FAILURE"
    } else if !controls_future_zero
        || !foreign_proof_rejected_everywhere
        || !counterfeit_proof_rejected_everywhere
        || !valid_permuted_accepted_everywhere
    {
        "CONTROL_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "H10 evidence-bound rule induction",
        mechanism: "target-blind deterministic induction over a 3x3 candidate universe from ten intervention episodes -> independent full-ranking proof recomputation -> opaque certificate -> PECS executable rule admission -> fixed three-scan closure",
        claim_boundary: "a PASS supports only evidence-bound symbolic rule induction and state-dependent executable composition under the frozen synthetic intervention regime; it does not establish open-world causal discovery, natural-language induction, ontology autonomy, AGI, consciousness, or human-level cognition",
        frozen_contract: FrozenContract {
            candidate_antecedents: 3,
            candidate_consequents: 3,
            candidate_rules: EXPECTED_CANDIDATES,
            evidence_episodes: EXPECTED_EPISODES,
            min_score: config.min_score,
            min_support: config.min_support,
            max_contradictions: config.max_contradictions,
            min_margin: config.min_margin,
            proposal_scoring_evaluations: EXPECTED_SCORING_EVALUATIONS,
            validation_scoring_evaluations: EXPECTED_VALIDATION_EVALUATIONS,
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
            let inference_successes = rows
                .iter()
                .filter(|(execution, _)| execution.inference_succeeded)
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
            let metrics = PathMetrics {
                roots,
                successes,
                success_rate: successes as f64 / roots.max(1) as f64,
                inference_successes,
                certificate_acceptances,
                certificate_rejections,
                mean_admitted_executable_mutations,
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
) -> Result<Vec<RuleInferenceProof>, Box<dyn Error>> {
    roots
        .iter()
        .map(|root| {
            let mut budget = ScoringBudget::default();
            let proof = infer_rule(&root.problem, config, &mut budget)?;
            if budget.candidate_episode_evaluations != EXPECTED_SCORING_EVALUATIONS {
                return Err("donor proof construction budget mismatch".into());
            }
            Ok(proof)
        })
        .collect()
}

fn execute_path(
    root: &RootTask,
    path: PathKind,
    foreign_proof: &RuleInferenceProof,
    config: RuleInductionConfig,
) -> Result<Execution, Box<dyn Error>> {
    let inference_problem = if path == PathKind::ValidPermutedInference {
        permuted_problem(root)
    } else {
        root.problem.clone()
    };

    if expected_scoring_evaluations(&inference_problem) != EXPECTED_SCORING_EVALUATIONS {
        return Err("H10 frozen candidate/evidence budget mismatch".into());
    }

    let mut proposal_budget = ScoringBudget::default();
    let current_proof = infer_rule(&inference_problem, config, &mut proposal_budget)?;
    let inference_succeeded = true;

    let proof_for_validation = match path {
        PathKind::RewiredForeignProof => foreign_proof.clone(),
        PathKind::CounterfeitProof => {
            let mut counterfeit = current_proof.clone();
            counterfeit.score += 1;
            counterfeit
        }
        _ => current_proof.clone(),
    };

    let mut validation_budget = ScoringBudget::default();
    let validation_result = validate_rule_inference(
        &inference_problem,
        &proof_for_validation,
        config,
        &mut validation_budget,
    );
    let certificate_accepted = validation_result.is_ok();
    let certificate_rejected = validation_result.is_err();
    let validation_error = validation_result.as_ref().err().map(ToString::to_string);
    let certificate: Option<ValidatedInferenceCertificate> = validation_result.ok();

    let mut state = initial_state(root)?;
    let mut admitted_executable_mutations = 0_usize;
    let mut closure_mutations = 0_usize;
    let mut text_history = String::new();
    let mut scalar_history = (0_i32, 0_i32);

    match path {
        PathKind::Stateful | PathKind::ValidPermutedInference => {
            if let Some(certificate) = certificate.as_ref() {
                state.admit_certificate(certificate)?;
                admitted_executable_mutations += 1;
            }
        }
        PathKind::TextOnly => {
            text_history = serde_json::to_string(&proof_for_validation)?;
        }
        PathKind::ScalarOnly => {
            scalar_history = (
                proof_for_validation.score,
                proof_for_validation.score - proof_for_validation.runner_up_score,
            );
        }
        PathKind::EndpointBlind
        | PathKind::RewiredForeignProof
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
            state.admit_certificate(certificate)?;
            admitted_executable_mutations += 1;
        }
    }

    let _non_executable_history = (text_history, scalar_history);
    let success = state.contains_fact(&root.goal);
    let invariants_hold = state.verify_invariants().is_ok();
    let canonical_state_signature = state.canonical_signature();
    let inferred_rule = Some(rule_name(&current_proof.rule));

    let budget = Budget {
        inference_calls: 1,
        candidates_scored: inference_problem.antecedents.len() * inference_problem.consequents.len(),
        evidence_episodes_per_candidate: inference_problem.evidence.len(),
        proposal_scoring_evaluations: proposal_budget.candidate_episode_evaluations,
        validation_recomputations: 1,
        validation_scoring_evaluations: validation_budget.candidate_episode_evaluations,
        admission_slots: 1,
        executor_scans: EXPECTED_EXECUTOR_SCANS,
        objective_checks: 1,
    };

    Ok(Execution {
        success,
        inference_succeeded,
        certificate_accepted,
        certificate_rejected,
        admitted_executable_mutations,
        closure_mutations,
        inferred_rule,
        validation_error,
        budget,
        invariants_hold,
        canonical_state_signature,
    })
}

fn initial_state(root: &RootTask) -> Result<EvidenceBoundCommitmentState, Box<dyn Error>> {
    let mut state = EvidenceBoundCommitmentState::new();
    state.seed_fact(root.source.clone())?;
    state.seed_rule(Rule::new(root.source.clone(), root.middle.clone())?)?;
    Ok(state)
}

fn permuted_problem(root: &RootTask) -> InferenceProblem {
    let mut problem = root.problem.clone();
    for episode in &mut problem.evidence {
        let mut outcomes = BTreeSet::new();
        for outcome in &episode.outcomes {
            if *outcome == root.goal {
                outcomes.insert(root.decoy_goal.clone());
            } else if *outcome == root.decoy_goal {
                outcomes.insert(root.goal.clone());
            } else {
                outcomes.insert(outcome.clone());
            }
        }
        episode.outcomes = outcomes;
    }
    problem
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
    let decoy_source = atom(&format!("{prefix}_decoy_source"))?;
    let decoy_goal = atom(&format!("{prefix}_decoy_goal"))?;
    let noise_source = atom(&format!("{prefix}_noise_source"))?;
    let noise_goal = atom(&format!("{prefix}_noise_goal"))?;

    let base_id = root_id * 100;
    let mut evidence = Vec::with_capacity(EXPECTED_EPISODES);
    for offset in 1..=4 {
        evidence.push(EvidenceEpisode {
            evidence_id: base_id + offset,
            intervention: middle.clone(),
            outcomes: BTreeSet::from([goal.clone()]),
        });
    }
    for offset in 5..=7 {
        evidence.push(EvidenceEpisode {
            evidence_id: base_id + offset,
            intervention: decoy_source.clone(),
            outcomes: BTreeSet::from([decoy_goal.clone()]),
        });
    }
    evidence.push(EvidenceEpisode {
        evidence_id: base_id + 8,
        intervention: decoy_source.clone(),
        outcomes: BTreeSet::new(),
    });
    evidence.push(EvidenceEpisode {
        evidence_id: base_id + 9,
        intervention: noise_source.clone(),
        outcomes: BTreeSet::from([noise_goal.clone()]),
    });
    evidence.push(EvidenceEpisode {
        evidence_id: base_id + 10,
        intervention: noise_source.clone(),
        outcomes: BTreeSet::new(),
    });

    Ok(RootTask {
        root_id,
        family,
        source,
        middle: middle.clone(),
        goal: goal.clone(),
        decoy_goal: decoy_goal.clone(),
        problem: InferenceProblem {
            antecedents: vec![middle, decoy_source, noise_source],
            consequents: vec![goal, decoy_goal, noise_goal],
            evidence,
        },
    })
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}

fn rule_name(rule: &Rule) -> String {
    format!(
        "{}->{}",
        rule.antecedent.as_str(),
        rule.consequent.as_str()
    )
}
