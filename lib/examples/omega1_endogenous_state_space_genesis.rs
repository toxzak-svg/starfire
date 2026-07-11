use serde::Serialize;
use star::commitment_state::Atom;
use star::representation_genesis::{
    detect_alias_defects, synthesize_refinement, validate_refinement, GenesisBudget, RawHistory,
    RefinementConfig, RefinementProblem, StateLanguage, WitnessedHistory,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const ROOTS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const DISCOVERY_HISTORIES: usize = 12;
const TRANSFER_HISTORIES: usize = 8;
const RAW_ATOMS: usize = 6;
const EXPECTED_PAIR_EVALUATIONS: usize = 66;
const EXPECTED_CANDIDATE_PROGRAMS: usize = 459;
const EXPECTED_PROGRAM_HISTORY_EVALUATIONS: usize = 5508;
const EXPECTED_ALIAS_DEFECTS: usize = 36;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_sequences",
    "transport_sequences",
    "ecological_sequences",
    "cellular_sequences",
    "manufacturing_sequences",
    "software_sequences",
    "watershed_sequences",
];

const DISCOVERY_ORDERS: [&str; DISCOVERY_HISTORIES] = [
    "BACDXY", "XDBYAC", "DCXYBA", "CDAYBX", "YBDXAC", "BYXACD", "YDCBXA",
    "CAYBXD", "DXACYB", "ADXYCB", "XACDBY", "YDCABX",
];

const TRANSFER_ORDERS: [&str; TRANSFER_HISTORIES] = [
    "XYABCD", "XYABDC", "XYACBD", "XYACDB", "YXABCD", "YXABDC", "YXACBD",
    "YXACDB",
];

#[derive(Debug, Clone)]
struct RootTask {
    root_id: u64,
    family: &'static str,
    target_problem: RefinementProblem,
    shuffled_problem: RefinementProblem,
    random_valid_problem: RefinementProblem,
    irrelevant_problem: RefinementProblem,
    transfer: Vec<(RawHistory, Atom)>,
    intervention: Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    StatefulRefinement,
    EndpointOnly,
    ProposalTextOnly,
    ScalarOnly,
    ForeignCertificate,
    CounterfeitCertificate,
    ValidIrrelevantRefinement,
    OutcomeShuffledSynthesis,
    RandomValidRefinement,
    DelayedCorrectAdmission,
}

impl PathKind {
    fn all() -> [Self; 10] {
        [
            Self::StatefulRefinement,
            Self::EndpointOnly,
            Self::ProposalTextOnly,
            Self::ScalarOnly,
            Self::ForeignCertificate,
            Self::CounterfeitCertificate,
            Self::ValidIrrelevantRefinement,
            Self::OutcomeShuffledSynthesis,
            Self::RandomValidRefinement,
            Self::DelayedCorrectAdmission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::StatefulRefinement => "stateful_refinement",
            Self::EndpointOnly => "endpoint_only",
            Self::ProposalTextOnly => "proposal_text_only",
            Self::ScalarOnly => "scalar_only",
            Self::ForeignCertificate => "foreign_certificate",
            Self::CounterfeitCertificate => "counterfeit_certificate",
            Self::ValidIrrelevantRefinement => "valid_irrelevant_refinement",
            Self::OutcomeShuffledSynthesis => "outcome_shuffled_synthesis",
            Self::RandomValidRefinement => "random_valid_refinement",
            Self::DelayedCorrectAdmission => "delayed_correct_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Budget {
    proposal: GenesisBudget,
    validation: GenesisBudget,
    admission_slots: usize,
    downstream_key_index_passes: usize,
    transfer_predictions: usize,
    objective_checks: usize,
}

impl Budget {
    fn exact(&self) -> bool {
        self.proposal.vocabulary_history_scans == DISCOVERY_HISTORIES
            && self.validation.vocabulary_history_scans == DISCOVERY_HISTORIES
            && self.proposal.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
            && self.validation.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
            && self.proposal.candidate_programs == EXPECTED_CANDIDATE_PROGRAMS
            && self.validation.candidate_programs == EXPECTED_CANDIDATE_PROGRAMS
            && self.proposal.program_history_evaluations
                == EXPECTED_PROGRAM_HISTORY_EVALUATIONS
            && self.validation.program_history_evaluations
                == EXPECTED_PROGRAM_HISTORY_EVALUATIONS
            && self.proposal.unique_partitions > 1
            && self.proposal.unique_partitions == self.validation.unique_partitions
            && self.admission_slots == 1
            && self.downstream_key_index_passes == 1
            && self.transfer_predictions == TRANSFER_HISTORIES
            && self.objective_checks == TRANSFER_HISTORIES
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    correct_predictions: usize,
    success: bool,
    proposal_succeeded: bool,
    validation_succeeded: bool,
    admission_accepted: bool,
    admission_rejected: bool,
    admitted_refinements_during_prediction: usize,
    admitted_refinements_final: usize,
    detected_defects: usize,
    repaired_defects: usize,
    winner_margin: usize,
    synthesized_program: String,
    validation_error: Option<String>,
    admission_error: Option<String>,
    endpoint_program_executions: usize,
    budget: Budget,
    invariants_hold: bool,
    canonical_language_signature: String,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    roots: usize,
    root_successes: usize,
    success_rate: f64,
    total_correct_predictions: usize,
    validation_successes: usize,
    admission_acceptances: usize,
    admission_rejections: usize,
    mean_admitted_refinements_during_prediction: f64,
    all_controls_zero_individual_predictions: bool,
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
    maximum_control_correct_predictions: usize,
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    discovery_histories: usize,
    transfer_histories: usize,
    raw_history_atoms: usize,
    expected_alias_defects: usize,
    expected_history_pair_evaluations: usize,
    expected_candidate_programs: usize,
    expected_program_history_evaluations: usize,
    min_partition_support: usize,
    min_winner_margin: usize,
    paths_per_root: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    cohort_exact: bool,
    base_alias_defects_exact: bool,
    stateful_training: bool,
    stateful_holdout: bool,
    stateful_future: bool,
    every_control_individual_prediction_zero: bool,
    foreign_certificates_rejected_everywhere: bool,
    counterfeit_proofs_rejected_everywhere: bool,
    valid_irrelevant_admitted_everywhere: bool,
    random_valid_admitted_everywhere: bool,
    shuffled_synthesis_rejected_everywhere: bool,
    all_future_families_transfer: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.base_alias_defects_exact
            && self.stateful_training
            && self.stateful_holdout
            && self.stateful_future
            && self.every_control_individual_prediction_zero
            && self.foreign_certificates_rejected_everywhere
            && self.counterfeit_proofs_rejected_everywhere
            && self.valid_irrelevant_admitted_everywhere
            && self.random_valid_admitted_everywhere
            && self.shuffled_synthesis_rejected_everywhere
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
    preregistration_commit: &'static str,
    frozen_contract: FrozenContract,
    training: SplitReport,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    gates: GateReport,
    terminal_classification: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = RefinementConfig::default();
    let roots = build_roots()?;
    let train_end = TRAIN_FAMILIES * ROOTS_PER_FAMILY;
    let holdout_end = train_end + HOLDOUT_FAMILIES * ROOTS_PER_FAMILY;
    let train = &roots[..train_end];
    let holdout = &roots[train_end..holdout_end];
    let future = &roots[holdout_end..];

    let base_alias_defects_exact = roots.iter().all(|root| {
        let language = StateLanguage::new(root.root_id);
        let mut budget = GenesisBudget::default();
        detect_alias_defects(&root.target_problem, &language, &mut budget)
            .map(|defects| defects.len() == EXPECTED_ALIAS_DEFECTS)
            .unwrap_or(false)
    });

    let training = evaluate_split(train, config)?;
    let holdout_report = evaluate_split(holdout, config)?;
    let future_report = evaluate_split(future, config)?;

    let mut future_families = Vec::new();
    for family_index in 0..FUTURE_FAMILIES {
        let start = holdout_end + family_index * ROOTS_PER_FAMILY;
        let end = start + ROOTS_PER_FAMILY;
        let family_report = evaluate_split(&roots[start..end], config)?;
        let stateful = path_metrics(&family_report, PathKind::StatefulRefinement);
        let maximum_control_success_rate = PathKind::all()
            .into_iter()
            .filter(|path| *path != PathKind::StatefulRefinement)
            .map(|path| path_metrics(&family_report, path).success_rate)
            .fold(0.0_f64, f64::max);
        let maximum_control_correct_predictions = PathKind::all()
            .into_iter()
            .filter(|path| *path != PathKind::StatefulRefinement)
            .map(|path| path_metrics(&family_report, path).total_correct_predictions)
            .max()
            .unwrap_or(0);
        future_families.push(FutureFamilyReport {
            family: FAMILIES[TRAIN_FAMILIES + HOLDOUT_FAMILIES + family_index],
            roots: ROOTS_PER_FAMILY,
            stateful_success_rate: stateful.success_rate,
            maximum_control_success_rate,
            maximum_control_correct_predictions,
        });
    }

    let cohort_exact = train.len() == 16
        && holdout.len() == 8
        && future.len() == 32
        && future_families.len() == 4;
    let stateful_training = stateful_perfect(&training);
    let stateful_holdout = stateful_perfect(&holdout_report);
    let stateful_future = stateful_perfect(&future_report);
    let every_control_individual_prediction_zero = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(controls_have_zero_predictions);
    let foreign_certificates_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::ForeignCertificate);
            metrics.admission_rejections == metrics.roots
        });
    let counterfeit_proofs_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::CounterfeitCertificate);
            metrics.validation_successes == 0
        });
    let valid_irrelevant_admitted_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::ValidIrrelevantRefinement);
            metrics.admission_acceptances == metrics.roots
        });
    let random_valid_admitted_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::RandomValidRefinement);
            metrics.admission_acceptances == metrics.roots
        });
    let shuffled_synthesis_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::OutcomeShuffledSynthesis);
            metrics.validation_successes == 0
        });
    let all_future_families_transfer = future_families.iter().all(|family| {
        family.stateful_success_rate == 1.0
            && family.maximum_control_success_rate == 0.0
            && family.maximum_control_correct_predictions == 0
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
        base_alias_defects_exact,
        stateful_training,
        stateful_holdout,
        stateful_future,
        every_control_individual_prediction_zero,
        foreign_certificates_rejected_everywhere,
        counterfeit_proofs_rejected_everywhere,
        valid_irrelevant_admitted_everywhere,
        random_valid_admitted_everywhere,
        shuffled_synthesis_rejected_everywhere,
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
        experiment: "Ω1 endogenous state-space genesis",
        mechanism: "witnessed base-language alias defect -> raw-vocabulary bounded predicate synthesis -> canonical behavioral partition ranking -> independent full recomputation -> opaque refinement certificate -> executable StateLanguage key extension -> representation-bound withheld prediction",
        claim_boundary: "a PASS supports only one-step endogenous executable state-key refinement under the frozen symbolic sequence regime and fixed bounded synthesis grammar; it does not establish unrestricted ontology invention, learned grammar growth, recursive descendant concept genesis, live-routing readiness, AGI, consciousness, or human-level cognition",
        preregistration_commit: "3088d4cdc34133cf30ee0253f951b9e1f84f907d",
        frozen_contract: FrozenContract {
            discovery_histories: DISCOVERY_HISTORIES,
            transfer_histories: TRANSFER_HISTORIES,
            raw_history_atoms: RAW_ATOMS,
            expected_alias_defects: EXPECTED_ALIAS_DEFECTS,
            expected_history_pair_evaluations: EXPECTED_PAIR_EVALUATIONS,
            expected_candidate_programs: EXPECTED_CANDIDATE_PROGRAMS,
            expected_program_history_evaluations: EXPECTED_PROGRAM_HISTORY_EVALUATIONS,
            min_partition_support: config.min_partition_support,
            min_winner_margin: config.min_winner_margin,
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
    if terminal_classification != "PASS" {
        std::process::exit(1);
    }
    Ok(())
}

fn evaluate_split(
    roots: &[RootTask],
    config: RefinementConfig,
) -> Result<SplitReport, Box<dyn Error>> {
    let mut executions = BTreeMap::<PathKind, Vec<(Execution, Execution)>>::new();
    for path in PathKind::all() {
        let mut path_executions = Vec::new();
        for root in roots {
            let first = execute_path(root, path, config)?;
            let second = execute_path(root, path, config)?;
            path_executions.push((first, second));
        }
        executions.insert(path, path_executions);
    }

    let mut paths = BTreeMap::new();
    for path in PathKind::all() {
        let pairs = &executions[&path];
        let roots_count = pairs.len();
        let root_successes = pairs.iter().filter(|(first, _)| first.success).count();
        let total_correct_predictions = pairs
            .iter()
            .map(|(first, _)| first.correct_predictions)
            .sum::<usize>();
        let validation_successes = pairs
            .iter()
            .filter(|(first, _)| first.validation_succeeded)
            .count();
        let admission_acceptances = pairs
            .iter()
            .filter(|(first, _)| first.admission_accepted)
            .count();
        let admission_rejections = pairs
            .iter()
            .filter(|(first, _)| first.admission_rejected)
            .count();
        let mean_admitted_refinements_during_prediction = if roots_count == 0 {
            0.0
        } else {
            pairs
                .iter()
                .map(|(first, _)| first.admitted_refinements_during_prediction)
                .sum::<usize>() as f64
                / roots_count as f64
        };
        let all_controls_zero_individual_predictions = if path == PathKind::StatefulRefinement {
            true
        } else {
            pairs
                .iter()
                .all(|(first, _)| first.correct_predictions == 0)
        };
        let budgets_exact = pairs.iter().all(|(first, _)| first.budget.exact());
        let replay_exact = pairs.iter().all(|(first, second)| first == second);
        let invariants_hold = pairs.iter().all(|(first, _)| first.invariants_hold);
        paths.insert(
            path.name().to_string(),
            PathMetrics {
                roots: roots_count,
                root_successes,
                success_rate: if roots_count == 0 {
                    0.0
                } else {
                    root_successes as f64 / roots_count as f64
                },
                total_correct_predictions,
                validation_successes,
                admission_acceptances,
                admission_rejections,
                mean_admitted_refinements_during_prediction,
                all_controls_zero_individual_predictions,
                budgets_exact,
                replay_exact,
                invariants_hold,
            },
        );
    }

    Ok(SplitReport {
        roots: roots.len(),
        paths,
    })
}

fn execute_path(
    root: &RootTask,
    path: PathKind,
    config: RefinementConfig,
) -> Result<Execution, Box<dyn Error>> {
    let synthesis_problem = match path {
        PathKind::ForeignCertificate => build_target_problem(
            root.root_id + 1_000_000,
            root.family,
            "foreign",
        )?,
        PathKind::ValidIrrelevantRefinement => root.irrelevant_problem.clone(),
        PathKind::OutcomeShuffledSynthesis => root.shuffled_problem.clone(),
        PathKind::RandomValidRefinement => root.random_valid_problem.clone(),
        _ => root.target_problem.clone(),
    };

    let mut proposal_budget = GenesisBudget::default();
    let mut proof = synthesize_refinement(&synthesis_problem, config, &mut proposal_budget)?;
    let proposal_succeeded = true;
    if path == PathKind::CounterfeitCertificate {
        proof.winner_margin = proof.winner_margin.saturating_add(1);
    }

    let synthesized_program = proof.program.canonical_string();
    let detected_defects = proof.detected_defects;
    let repaired_defects = proof.repaired_defects;
    let winner_margin = proof.winner_margin;

    let mut validation_budget = GenesisBudget::default();
    let validation = validate_refinement(
        &synthesis_problem,
        &proof,
        config,
        &mut validation_budget,
    );
    let validation_succeeded = validation.is_ok();
    let validation_error = validation.as_ref().err().map(ToString::to_string);

    let mut language = StateLanguage::new(root.root_id);
    let mut admission_accepted = false;
    let mut admission_rejected = false;
    let mut admission_error = None;
    let mut endpoint_program_executions = 0;

    match path {
        PathKind::StatefulRefinement
        | PathKind::ForeignCertificate
        | PathKind::ValidIrrelevantRefinement
        | PathKind::RandomValidRefinement => {
            if let Ok(certificate) = &validation {
                match language.admit_certificate(certificate) {
                    Ok(()) => admission_accepted = true,
                    Err(error) => {
                        admission_rejected = true;
                        admission_error = Some(error.to_string());
                    }
                }
            } else {
                admission_rejected = true;
            }
        }
        PathKind::EndpointOnly => {
            for (history, _) in &root.transfer {
                let _ = proof.program.execute(history);
                endpoint_program_executions += 1;
            }
        }
        PathKind::ProposalTextOnly => {
            let _proposal_text = format!("{:?}", proof);
        }
        PathKind::ScalarOnly => {
            let _scalars = (
                proof.repaired_defects,
                proof.partition_support_min,
                proof.winner_margin,
            );
        }
        PathKind::CounterfeitCertificate | PathKind::OutcomeShuffledSynthesis => {
            if validation.is_err() {
                admission_rejected = true;
            }
        }
        PathKind::DelayedCorrectAdmission => {}
    }

    let admitted_refinements_during_prediction = language.refinement_count();
    let outcome_index = build_outcome_index(&language, &root.target_problem.discovery);
    let mut correct_predictions = 0;
    let mut transfer_predictions = 0;
    let mut objective_checks = 0;
    for (history, expected) in &root.transfer {
        transfer_predictions += 1;
        let key = language.state_key(history, &root.intervention);
        let prediction = outcome_index
            .get(&key)
            .and_then(|outcomes| (outcomes.len() == 1).then(|| outcomes.iter().next().unwrap()));
        objective_checks += 1;
        if prediction == Some(expected) {
            correct_predictions += 1;
        }
    }

    if path == PathKind::DelayedCorrectAdmission {
        if let Ok(certificate) = &validation {
            match language.admit_certificate(certificate) {
                Ok(()) => admission_accepted = true,
                Err(error) => {
                    admission_rejected = true;
                    admission_error = Some(error.to_string());
                }
            }
        } else {
            admission_rejected = true;
        }
    }

    let invariants_hold = language.verify_invariants().is_ok();
    let canonical_language_signature = language.canonical_signature();
    let admitted_refinements_final = language.refinement_count();
    let success = correct_predictions == TRANSFER_HISTORIES;

    Ok(Execution {
        correct_predictions,
        success,
        proposal_succeeded,
        validation_succeeded,
        admission_accepted,
        admission_rejected,
        admitted_refinements_during_prediction,
        admitted_refinements_final,
        detected_defects,
        repaired_defects,
        winner_margin,
        synthesized_program,
        validation_error,
        admission_error,
        endpoint_program_executions,
        budget: Budget {
            proposal: proposal_budget,
            validation: validation_budget,
            admission_slots: 1,
            downstream_key_index_passes: 1,
            transfer_predictions,
            objective_checks,
        },
        invariants_hold,
        canonical_language_signature,
    })
}

fn build_outcome_index(
    language: &StateLanguage,
    discovery: &[WitnessedHistory],
) -> BTreeMap<star::representation_genesis::StateKey, BTreeSet<Atom>> {
    let mut index = BTreeMap::new();
    for episode in discovery {
        let key = language.state_key(&episode.history, &episode.intervention);
        index
            .entry(key)
            .or_insert_with(BTreeSet::new)
            .insert(episode.outcome.clone());
    }
    index
}

fn build_roots() -> Result<Vec<RootTask>, Box<dyn Error>> {
    let mut roots = Vec::new();
    let mut root_id = 1_u64;
    for family in FAMILIES {
        for local_index in 0..ROOTS_PER_FAMILY {
            roots.push(build_root(root_id, family, local_index)?);
            root_id += 1;
        }
    }
    Ok(roots)
}

fn build_root(
    root_id: u64,
    family: &'static str,
    local_index: usize,
) -> Result<RootTask, Box<dyn Error>> {
    let prefix = format!("{family}_{local_index}");
    let roles = role_atoms(&prefix)?;
    let intervention = atom(&format!("{prefix}_probe"))?;
    let positive = atom(&format!("{prefix}_positive"))?;
    let negative = atom(&format!("{prefix}_negative"))?;

    let target_problem = problem_from_orders(
        root_id,
        &roles,
        &intervention,
        &positive,
        &negative,
        |order| precedes(order, 'X', 'Y'),
    )?;

    let mut shuffled_problem = target_problem.clone();
    let original_outcomes = shuffled_problem
        .discovery
        .iter()
        .map(|episode| episode.outcome.clone())
        .collect::<Vec<_>>();
    for index in 0..shuffled_problem.discovery.len() {
        let source = if index == 0 {
            original_outcomes.len() - 1
        } else {
            index - 1
        };
        shuffled_problem.discovery[index].outcome = original_outcomes[source].clone();
    }

    let random_positive = atom(&format!("{prefix}_control_positive"))?;
    let random_negative = atom(&format!("{prefix}_control_negative"))?;
    let random_valid_problem = problem_from_orders(
        root_id,
        &roles,
        &intervention,
        &random_positive,
        &random_negative,
        |order| precedes(order, 'X', 'A'),
    )?;

    let irrelevant_prefix = format!("{prefix}_irrelevant");
    let irrelevant_roles = role_atoms(&irrelevant_prefix)?;
    let irrelevant_intervention = atom(&format!("{irrelevant_prefix}_probe"))?;
    let irrelevant_positive = atom(&format!("{irrelevant_prefix}_positive"))?;
    let irrelevant_negative = atom(&format!("{irrelevant_prefix}_negative"))?;
    let irrelevant_problem = problem_from_orders(
        root_id,
        &irrelevant_roles,
        &irrelevant_intervention,
        &irrelevant_positive,
        &irrelevant_negative,
        |order| precedes(order, 'X', 'Y'),
    )?;

    let transfer = TRANSFER_ORDERS
        .iter()
        .enumerate()
        .map(|(index, order)| {
            let history = RawHistory {
                history_id: root_id * 100_000 + 50_000 + index as u64,
                events: map_order(order, &roles),
            };
            let expected = if precedes(order, 'X', 'Y') {
                positive.clone()
            } else {
                negative.clone()
            };
            (history, expected)
        })
        .collect();

    Ok(RootTask {
        root_id,
        family,
        target_problem,
        shuffled_problem,
        random_valid_problem,
        irrelevant_problem,
        transfer,
        intervention,
    })
}

fn build_target_problem(
    root_id: u64,
    family: &str,
    suffix: &str,
) -> Result<RefinementProblem, Box<dyn Error>> {
    let prefix = format!("{family}_{suffix}_{root_id}");
    let roles = role_atoms(&prefix)?;
    let intervention = atom(&format!("{prefix}_probe"))?;
    let positive = atom(&format!("{prefix}_positive"))?;
    let negative = atom(&format!("{prefix}_negative"))?;
    problem_from_orders(
        root_id,
        &roles,
        &intervention,
        &positive,
        &negative,
        |order| precedes(order, 'X', 'Y'),
    )
}

fn problem_from_orders<F>(
    root_id: u64,
    roles: &[Atom; RAW_ATOMS],
    intervention: &Atom,
    positive: &Atom,
    negative: &Atom,
    label: F,
) -> Result<RefinementProblem, Box<dyn Error>>
where
    F: Fn(&str) -> bool,
{
    let discovery = DISCOVERY_ORDERS
        .iter()
        .enumerate()
        .map(|(index, order)| WitnessedHistory {
            evidence_id: root_id * 10_000 + index as u64 + 1,
            history: RawHistory {
                history_id: root_id * 100_000 + index as u64 + 1,
                events: map_order(order, roles),
            },
            intervention: intervention.clone(),
            outcome: if label(order) {
                positive.clone()
            } else {
                negative.clone()
            },
        })
        .collect();
    Ok(RefinementProblem { root_id, discovery })
}

fn role_atoms(prefix: &str) -> Result<[Atom; RAW_ATOMS], Box<dyn Error>> {
    Ok([
        atom(&format!("{prefix}_x"))?,
        atom(&format!("{prefix}_y"))?,
        atom(&format!("{prefix}_a"))?,
        atom(&format!("{prefix}_b"))?,
        atom(&format!("{prefix}_c"))?,
        atom(&format!("{prefix}_d"))?,
    ])
}

fn map_order(order: &str, roles: &[Atom; RAW_ATOMS]) -> Vec<Atom> {
    order
        .chars()
        .map(|role| match role {
            'X' => roles[0].clone(),
            'Y' => roles[1].clone(),
            'A' => roles[2].clone(),
            'B' => roles[3].clone(),
            'C' => roles[4].clone(),
            'D' => roles[5].clone(),
            _ => unreachable!("frozen Ω1 role alphabet"),
        })
        .collect()
}

fn precedes(order: &str, left: char, right: char) -> bool {
    order.find(left).unwrap() < order.find(right).unwrap()
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}

fn path_metrics(report: &SplitReport, path: PathKind) -> &PathMetrics {
    &report.paths[path.name()]
}

fn stateful_perfect(report: &SplitReport) -> bool {
    let metrics = path_metrics(report, PathKind::StatefulRefinement);
    metrics.success_rate == 1.0
        && metrics.total_correct_predictions == metrics.roots * TRANSFER_HISTORIES
        && metrics.admission_acceptances == metrics.roots
}

fn controls_have_zero_predictions(report: &SplitReport) -> bool {
    PathKind::all()
        .into_iter()
        .filter(|path| *path != PathKind::StatefulRefinement)
        .all(|path| {
            let metrics = path_metrics(report, path);
            metrics.total_correct_predictions == 0
                && metrics.all_controls_zero_individual_predictions
        })
}
