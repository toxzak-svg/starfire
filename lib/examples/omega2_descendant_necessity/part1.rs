use serde::Serialize;
use star::commitment_state::Atom;
use star::representation_descendants::{
    audit_raw_expressibility, synthesize_descendant, validate_descendant, DescendantBudget,
    DescendantConfig, DescendantGenesisError, DescendantProof, DescendantStateLanguage,
    RawExpressibilityAudit, ValidatedDescendantCertificate,
};
use star::representation_genesis::{
    detect_alias_defects, synthesize_refinement, validate_refinement, GenesisBudget, RawHistory,
    RefinementConfig, RefinementProblem, RefinementProof, StateLanguage,
    ValidatedRefinementCertificate, WitnessedHistory,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const PREREGISTRATION_COMMIT: &str = "deccc66fc2690d9e0f38517ce0be96ecf9cbc5c6";
const ROOTS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const DISCOVERY_HISTORIES: usize = 8;
const TRANSFER_HISTORIES: usize = 8;
const RAW_ATOMS: usize = 6;

const EXPECTED_OMEGA1_PAIR_EVALUATIONS: usize = 28;
const EXPECTED_OMEGA1_ALIAS_DEFECTS: usize = 16;
const EXPECTED_RAW_CANDIDATES: usize = 459;
const EXPECTED_RAW_EVALUATIONS: usize = 3_672;
const EXPECTED_RAW_UNIQUE_PARTITIONS: usize = 4;
const EXPECTED_STAGE2_OPPOSITION_PAIRS: usize = 16;
const EXPECTED_L0_BEST_REPAIR: usize = 8;
const EXPECTED_L1_ALIAS_DEFECTS: usize = 8;
const EXPECTED_DESCENDANT_CANDIDATES: usize = 918;
const EXPECTED_DESCENDANT_EVALUATIONS: usize = 7_344;
const EXPECTED_DESCENDANT_UNIQUE_PARTITIONS: usize = 4;
const EXPECTED_DESCENDANT_REPAIRED: usize = 16;
const EXPECTED_DESCENDANT_RUNNER_UP: usize = 8;
const EXPECTED_DESCENDANT_MARGIN: usize = 8;
const EXPECTED_DESCENDANT_SUPPORT: usize = 4;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_descendants",
    "transport_descendants",
    "ecological_descendants",
    "cellular_descendants",
    "manufacturing_descendants",
    "software_descendants",
    "watershed_descendants",
];

#[derive(Debug, Clone)]
struct RootTask {
    root_id: u64,
    family: &'static str,
    correct_ancestor_problem: RefinementProblem,
    wrong_ancestor_problem: RefinementProblem,
    stage2_problem: RefinementProblem,
    shuffled_stage2_problem: RefinementProblem,
    transfer: Vec<(RawHistory, Atom)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    StatefulDescendantChain,
    L0RawSearch,
    L0DescendantNoAncestor,
    Delta1EndpointOnly,
    WrongValidAncestor,
    Delta1AblationReplacedBeforeValidation,
    Delta2PayloadOnly,
    CounterfeitDelta2Proof,
    OutcomeShuffledDescendant,
    DelayedDelta2Admission,
}

impl PathKind {
    fn all() -> [Self; 10] {
        [
            Self::StatefulDescendantChain,
            Self::L0RawSearch,
            Self::L0DescendantNoAncestor,
            Self::Delta1EndpointOnly,
            Self::WrongValidAncestor,
            Self::Delta1AblationReplacedBeforeValidation,
            Self::Delta2PayloadOnly,
            Self::CounterfeitDelta2Proof,
            Self::OutcomeShuffledDescendant,
            Self::DelayedDelta2Admission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::StatefulDescendantChain => "stateful_descendant_chain",
            Self::L0RawSearch => "l0_raw_search",
            Self::L0DescendantNoAncestor => "l0_descendant_no_ancestor",
            Self::Delta1EndpointOnly => "delta1_endpoint_only",
            Self::WrongValidAncestor => "wrong_valid_ancestor",
            Self::Delta1AblationReplacedBeforeValidation => {
                "delta1_ablation_replaced_before_validation"
            }
            Self::Delta2PayloadOnly => "delta2_payload_only",
            Self::CounterfeitDelta2Proof => "counterfeit_delta2_proof",
            Self::OutcomeShuffledDescendant => "outcome_shuffled_descendant",
            Self::DelayedDelta2Admission => "delayed_delta2_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
struct BudgetLedger {
    ancestor_proposals: Vec<GenesisBudget>,
    ancestor_validations: Vec<GenesisBudget>,
    raw_audits: Vec<DescendantBudget>,
    descendant_proposals: Vec<DescendantBudget>,
    descendant_validations: Vec<DescendantBudget>,
    ancestor_admission_slots: usize,
    descendant_admission_slots: usize,
    endpoint_ancestor_executions: usize,
    downstream_key_index_passes: usize,
    transfer_predictions: usize,
    objective_checks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    correct_predictions: usize,
    success: bool,
    correct_ancestor_proposal_succeeded: bool,
    correct_ancestor_validation_succeeded: bool,
    correct_ancestor_admitted: bool,
    wrong_ancestor_proposal_succeeded: bool,
    wrong_ancestor_validation_succeeded: bool,
    wrong_ancestor_admitted: bool,
    descendant_proposal_succeeded: bool,
    descendant_validation_succeeded: bool,
    descendant_admitted_during_prediction: bool,
    descendant_admitted_final: bool,
    no_ancestor_rejection: bool,
    descendant_validation_rejected: bool,
    raw_audit_complete_repair: bool,
    payload_preserved: bool,
    proposal_repaired_pairs: usize,
    proposal_candidate_count: usize,
    proposal_unique_partitions: usize,
    proposal_winner_margin: usize,
    validation_error: Option<String>,
    admission_error: Option<String>,
    budget: BudgetLedger,
    budget_exact: bool,
    invariants_hold: bool,
    final_language_signature: String,
}

#[derive(Debug, Clone, Serialize)]
struct RootAudit {
    roots: usize,
    correct_stage1_alias_defects_exact: usize,
    wrong_stage1_alias_defects_exact: usize,
    correct_stage1_search_exact: usize,
    wrong_stage1_search_exact: usize,
    l0_raw_audit_exact: usize,
    l0_descendant_empty_exact: usize,
    l1_alias_defects_exact: usize,
    l1_descendant_frontier_exact: usize,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    roots: usize,
    root_successes: usize,
    success_rate: f64,
    total_correct_predictions: usize,
    correct_ancestor_admissions: usize,
    wrong_ancestor_admissions: usize,
    descendant_proposal_successes: usize,
    descendant_validation_successes: usize,
    descendant_admissions_during_prediction: usize,
    descendant_final_admissions: usize,
    no_ancestor_rejections: usize,
    descendant_validation_rejections: usize,
    raw_audit_complete_repairs: usize,
    payload_preservations: usize,
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
    preregistration_commit: &'static str,
    roots_per_family: usize,
    train_roots: usize,
    holdout_roots: usize,
    future_roots: usize,
    future_families: usize,
    discovery_histories: usize,
    transfer_histories: usize,
    raw_atoms: usize,
    omega1_alias_defects: usize,
    omega1_raw_candidates: usize,
    omega1_raw_evaluations: usize,
    l0_raw_best_repair: usize,
    stage2_opposition_pairs: usize,
    l0_descendant_candidates: usize,
    l1_alias_defects: usize,
    l1_descendant_candidates: usize,
    l1_descendant_evaluations: usize,
    descendant_unique_partitions: usize,
    descendant_repaired_pairs: usize,
    descendant_runner_up: usize,
    descendant_margin: usize,
    descendant_support: usize,
    paths_per_root: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    cohort_exact: bool,
    structural_audits_exact: bool,
    stateful_training: bool,
    stateful_holdout: bool,
    stateful_future: bool,
    every_control_individual_prediction_zero: bool,
    l0_raw_complete_repair_absent_everywhere: bool,
    l0_descendant_frontier_empty_everywhere: bool,
    wrong_ancestor_full_search_rejected_everywhere: bool,
    exact_ancestor_ablation_rejected_everywhere: bool,
    counterfeit_descendant_rejected_everywhere: bool,
    shuffled_descendant_rejected_everywhere: bool,
    payload_only_never_admitted: bool,
    delayed_admission_zero_during_window: bool,
    delayed_admission_eventually_succeeds: bool,
    all_future_families_transfer: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.structural_audits_exact
            && self.stateful_training
            && self.stateful_holdout
            && self.stateful_future
            && self.every_control_individual_prediction_zero
            && self.l0_raw_complete_repair_absent_everywhere
            && self.l0_descendant_frontier_empty_everywhere
            && self.wrong_ancestor_full_search_rejected_everywhere
            && self.exact_ancestor_ablation_rejected_everywhere
            && self.counterfeit_descendant_rejected_everywhere
            && self.shuffled_descendant_rejected_everywhere
            && self.payload_only_never_admitted
            && self.delayed_admission_zero_during_window
            && self.delayed_admission_eventually_succeeds
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
    root_audit: RootAudit,
    training: SplitReport,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    gates: GateReport,
    terminal_classification: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let roots = build_roots()?;
    let train_end = TRAIN_FAMILIES * ROOTS_PER_FAMILY;
    let holdout_end = train_end + HOLDOUT_FAMILIES * ROOTS_PER_FAMILY;
    let train = &roots[..train_end];
    let holdout = &roots[train_end..holdout_end];
    let future = &roots[holdout_end..];

    let root_audit = audit_roots(&roots)?;
    let training = evaluate_split(train)?;
    let holdout_report = evaluate_split(holdout)?;
    let future_report = evaluate_split(future)?;

    let mut future_families = Vec::new();
    for family_index in 0..FUTURE_FAMILIES {
        let start = holdout_end + family_index * ROOTS_PER_FAMILY;
        let end = start + ROOTS_PER_FAMILY;
        let family_report = evaluate_split(&roots[start..end])?;
        let stateful = path_metrics(&family_report, PathKind::StatefulDescendantChain);
        let maximum_control_success_rate = PathKind::all()
            .into_iter()
            .filter(|path| *path != PathKind::StatefulDescendantChain)
            .map(|path| path_metrics(&family_report, path).success_rate)
            .fold(0.0_f64, f64::max);
        let maximum_control_correct_predictions = PathKind::all()
            .into_iter()
            .filter(|path| *path != PathKind::StatefulDescendantChain)
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
        && future_families.len() == FUTURE_FAMILIES;
    let structural_audits_exact = root_audit.correct_stage1_alias_defects_exact == roots.len()
        && root_audit.wrong_stage1_alias_defects_exact == roots.len()
        && root_audit.correct_stage1_search_exact == roots.len()
        && root_audit.wrong_stage1_search_exact == roots.len()
        && root_audit.l0_raw_audit_exact == roots.len()
        && root_audit.l0_descendant_empty_exact == roots.len()
        && root_audit.l1_alias_defects_exact == roots.len()
        && root_audit.l1_descendant_frontier_exact == roots.len();

    let stateful_training = stateful_perfect(&training);
    let stateful_holdout = stateful_perfect(&holdout_report);
    let stateful_future = stateful_perfect(&future_report);
    let every_control_individual_prediction_zero = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(controls_have_zero_predictions);
    let l0_raw_complete_repair_absent_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| path_metrics(split, PathKind::L0RawSearch).raw_audit_complete_repairs == 0);
    let l0_descendant_frontier_empty_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::L0DescendantNoAncestor);
            metrics.no_ancestor_rejections == metrics.roots
                && metrics.descendant_proposal_successes == 0
        });
    let wrong_ancestor_full_search_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::WrongValidAncestor);
            metrics.wrong_ancestor_admissions == metrics.roots
                && metrics.descendant_proposal_successes == metrics.roots
                && metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
        });
    let exact_ancestor_ablation_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(
                split,
                PathKind::Delta1AblationReplacedBeforeValidation,
            );
            metrics.descendant_proposal_successes == metrics.roots
                && metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
                && metrics.wrong_ancestor_admissions == metrics.roots
        });
    let counterfeit_descendant_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::CounterfeitDelta2Proof);
            metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
        });
    let shuffled_descendant_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::OutcomeShuffledDescendant);
            metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
        });
    let payload_only_never_admitted = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::Delta2PayloadOnly);
            metrics.descendant_validation_successes == metrics.roots
                && metrics.descendant_admissions_during_prediction == 0
                && metrics.descendant_final_admissions == 0
                && metrics.payload_preservations == metrics.roots
        });
    let delayed_admission_zero_during_window = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            path_metrics(split, PathKind::DelayedDelta2Admission).total_correct_predictions == 0
        });
    let delayed_admission_eventually_succeeds = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::DelayedDelta2Admission);
            metrics.descendant_admissions_during_prediction == 0
                && metrics.descendant_final_admissions == metrics.roots
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
        structural_audits_exact,
        stateful_training,
        stateful_holdout,
        stateful_future,
        every_control_individual_prediction_zero,
        l0_raw_complete_repair_absent_everywhere,
        l0_descendant_frontier_empty_everywhere,
        wrong_ancestor_full_search_rejected_everywhere,
        exact_ancestor_ablation_rejected_everywhere,
        counterfeit_descendant_rejected_everywhere,
        shuffled_descendant_rejected_everywhere,
        payload_only_never_admitted,
        delayed_admission_zero_during_window,
        delayed_admission_eventually_succeeds,
        all_future_families_transfer,
        budgets_exact,
        replay_exact,
        invariants_hold,
    };

    let terminal_classification = if !budgets_exact
        || !invariants_hold
        || !exact_ancestor_ablation_rejected_everywhere
    {
        "CONTROL_FAILURE"
    } else if !replay_exact {
        "REPLAY_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "Ω2 descendant necessity",
        mechanism: "an admitted Ω1 refinement generates an executable ancestor-bit terminal; only that exact root-bound ancestor language can enumerate, validate, and admit a proof-carrying descendant refinement that combines the ancestor bit with an Ω1 raw-history predicate",
        claim_boundary: "a PASS supports bounded two-generation executable representation growth under the frozen symbolic cube and fixed descendant grammar; it does not establish unrestricted recursive ontology growth, learned grammar mutation, natural-language concept genesis, AGI, consciousness, or human-level cognition",
        frozen_contract: FrozenContract {
            preregistration_commit: PREREGISTRATION_COMMIT,
            roots_per_family: ROOTS_PER_FAMILY,
            train_roots: 16,
            holdout_roots: 8,
            future_roots: 32,
            future_families: FUTURE_FAMILIES,
            discovery_histories: DISCOVERY_HISTORIES,
            transfer_histories: TRANSFER_HISTORIES,
            raw_atoms: RAW_ATOMS,
            omega1_alias_defects: EXPECTED_OMEGA1_ALIAS_DEFECTS,
            omega1_raw_candidates: EXPECTED_RAW_CANDIDATES,
            omega1_raw_evaluations: EXPECTED_RAW_EVALUATIONS,
            l0_raw_best_repair: EXPECTED_L0_BEST_REPAIR,
            stage2_opposition_pairs: EXPECTED_STAGE2_OPPOSITION_PAIRS,
            l0_descendant_candidates: 0,
            l1_alias_defects: EXPECTED_L1_ALIAS_DEFECTS,
            l1_descendant_candidates: EXPECTED_DESCENDANT_CANDIDATES,
            l1_descendant_evaluations: EXPECTED_DESCENDANT_EVALUATIONS,
            descendant_unique_partitions: EXPECTED_DESCENDANT_UNIQUE_PARTITIONS,
            descendant_repaired_pairs: EXPECTED_DESCENDANT_REPAIRED,
            descendant_runner_up: EXPECTED_DESCENDANT_RUNNER_UP,
            descendant_margin: EXPECTED_DESCENDANT_MARGIN,
            descendant_support: EXPECTED_DESCENDANT_SUPPORT,
            paths_per_root: PathKind::all().len(),
        },
        root_audit,
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
