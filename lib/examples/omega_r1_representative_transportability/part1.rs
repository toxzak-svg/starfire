use serde::Serialize;
use star::commitment_state::Atom;
use star::representation_genesis::{
    detect_alias_defects, synthesize_refinement, validate_refinement, GenesisBudget, RawHistory,
    RefinementConfig, RefinementProblem, RefinementProof, StateLanguage, WitnessedHistory,
};
use star::representation_transport::{
    synthesize_transport_refinement, validate_transport_refinement, BlockPermutation,
    CorrespondenceMode, TransformationSuite, TransportBudget, TransportConfig, TransportError,
    TransportProof, TransportStateLanguage, ValidatedTransportCertificate,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const PREREGISTRATION_COMMIT: &str = "a4f7472d0f69b351a2eaac9a21c26976cf1af5ce";
const ROOTS_PER_FAMILY: usize = 8;
const TRAIN_FAMILIES: usize = 2;
const HOLDOUT_FAMILIES: usize = 1;
const FUTURE_FAMILIES: usize = 4;
const DISCOVERY_HISTORIES: usize = 16;
const RAW_ATOMS: usize = 8;
const HELDOUT_TRANSFORMATIONS: usize = 2;
const PREDICTIONS_PER_ROOT: usize = DISCOVERY_HISTORIES * HELDOUT_TRANSFORMATIONS;

const EXPECTED_PAIR_EVALUATIONS: usize = 120;
const EXPECTED_ALIAS_DEFECTS: usize = 64;
const EXPECTED_CANDIDATES: usize = 828;
const EXPECTED_DISCOVERY_EVALUATIONS: usize = 13_248;
const EXPECTED_UNIQUE_PARTITIONS: usize = 5;
const EXPECTED_WINNER_REPAIR: usize = 64;
const EXPECTED_RUNNER_UP: usize = 32;
const EXPECTED_MARGIN: usize = 32;
const EXPECTED_SUPPORT: usize = 8;
const EXPECTED_WINNING_CLASS: usize = 72;
const EXPECTED_CALIBRATION_TRANSFORMS: usize = 2;
const EXPECTED_TRANSPORT_EVALUATIONS: usize = 2_304;
const EXPECTED_PRIMARY_ZERO_VIOLATION: usize = 8;
const EXPECTED_STATIONARY_ZERO_VIOLATION: usize = 72;
const EXPECTED_REWIRED_ZERO_VIOLATION: usize = 0;
const EXPECTED_REWIRED_MIN_VIOLATIONS: usize = 4;
const EXPECTED_BASELINE_CORRECT: usize = 16;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_transport",
    "hydraulic_transport",
    "ecological_transport",
    "cellular_transport",
    "manufacturing_transport",
    "software_transport",
    "watershed_transport",
];

#[derive(Debug, Clone)]
struct RootTask {
    root_id: u64,
    family: &'static str,
    problem: RefinementProblem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    OrbitAwareStateful,
    PartitionOnlyBaseline,
    TargetStationaryMatchedCalibration,
    RewiredCorrespondenceCalibration,
    TransportPayloadOnly,
    CounterfeitTransportProof,
    ForeignRootTransportCertificate,
    DelayedTransportAdmission,
}

impl PathKind {
    fn all() -> [Self; 8] {
        [
            Self::OrbitAwareStateful,
            Self::PartitionOnlyBaseline,
            Self::TargetStationaryMatchedCalibration,
            Self::RewiredCorrespondenceCalibration,
            Self::TransportPayloadOnly,
            Self::CounterfeitTransportProof,
            Self::ForeignRootTransportCertificate,
            Self::DelayedTransportAdmission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::OrbitAwareStateful => "orbit_aware_stateful",
            Self::PartitionOnlyBaseline => "partition_only_baseline",
            Self::TargetStationaryMatchedCalibration => "target_stationary_matched_calibration",
            Self::RewiredCorrespondenceCalibration => "rewired_correspondence_calibration",
            Self::TransportPayloadOnly => "transport_payload_only",
            Self::CounterfeitTransportProof => "counterfeit_transport_proof",
            Self::ForeignRootTransportCertificate => "foreign_root_transport_certificate",
            Self::DelayedTransportAdmission => "delayed_transport_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
struct BudgetLedger {
    omega1_proposals: Vec<GenesisBudget>,
    omega1_validations: Vec<GenesisBudget>,
    transport_proposals: Vec<TransportBudget>,
    transport_validations: Vec<TransportBudget>,
    transport_admission_slots: usize,
    heldout_transformation_applications: usize,
    discovery_key_index_passes: usize,
    prediction_attempts: usize,
    objective_checks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    correct_predictions: usize,
    full_success: bool,
    selected_program: Option<String>,
    baseline_program: Option<String>,
    proposal_succeeded: bool,
    validation_succeeded: bool,
    validation_rejected: bool,
    admission_succeeded_during_window: bool,
    final_admission_succeeded: bool,
    admission_rejected: bool,
    payload_preserved: bool,
    candidate_program_count: usize,
    unique_partition_count: usize,
    repaired_pairs: usize,
    runner_up_repaired_pairs: usize,
    winner_margin: usize,
    partition_support_min: usize,
    winning_class_representatives: usize,
    zero_violation_representatives: usize,
    minimum_transport_violations: usize,
    selected_transport_violations: usize,
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
    alias_defects_exact: usize,
    omega1_partition_search_exact: usize,
    omega1_accidental_representative_exact: usize,
    primary_transport_frontier_exact: usize,
    stationary_transport_frontier_exact: usize,
    rewired_transport_frontier_exact: usize,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    roots: usize,
    full_successes: usize,
    success_rate: f64,
    total_correct_predictions: usize,
    proposal_successes: usize,
    validation_successes: usize,
    validation_rejections: usize,
    admissions_during_window: usize,
    final_admissions: usize,
    admission_rejections: usize,
    payload_preservations: usize,
    expected_stable_program_selections: usize,
    expected_accidental_program_selections: usize,
    minimum_zero_violation_representatives: usize,
    maximum_zero_violation_representatives: usize,
    minimum_transport_violations: usize,
    maximum_transport_violations: usize,
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
    orbit_aware_success_rate: f64,
    orbit_aware_correct_predictions: usize,
    partition_only_success_rate: f64,
    partition_only_correct_predictions: usize,
    stationary_success_rate: f64,
    stationary_correct_predictions: usize,
}

#[derive(Debug, Serialize)]
struct FrozenContract {
    preregistration_commit: &'static str,
    train_roots: usize,
    holdout_roots: usize,
    future_roots: usize,
    future_families: usize,
    raw_atoms: usize,
    discovery_histories: usize,
    heldout_transformations: usize,
    predictions_per_root: usize,
    alias_defects: usize,
    candidate_programs: usize,
    discovery_program_history_evaluations: usize,
    unique_partitions: usize,
    winner_repair: usize,
    runner_up_repair: usize,
    winner_margin: usize,
    partition_support: usize,
    winning_class_representatives: usize,
    calibration_transformations: usize,
    transport_program_history_evaluations: usize,
    primary_zero_violation_representatives: usize,
    stationary_zero_violation_representatives: usize,
    rewired_zero_violation_representatives: usize,
    rewired_minimum_violations: usize,
    paths_per_root: usize,
}

#[derive(Debug, Serialize)]
struct GateReport {
    cohort_exact: bool,
    structural_audits_exact: bool,
    orbit_aware_training: bool,
    orbit_aware_holdout: bool,
    orbit_aware_future: bool,
    partition_only_exact_half_transfer: bool,
    stationary_exact_half_transfer: bool,
    rewired_full_search_rejected_everywhere: bool,
    payload_only_inert_everywhere: bool,
    counterfeit_rejected_everywhere: bool,
    foreign_root_rejected_everywhere: bool,
    delayed_zero_during_window: bool,
    delayed_eventual_admission: bool,
    all_future_families_transfer: bool,
    budgets_exact: bool,
    replay_exact: bool,
    invariants_hold: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.structural_audits_exact
            && self.orbit_aware_training
            && self.orbit_aware_holdout
            && self.orbit_aware_future
            && self.partition_only_exact_half_transfer
            && self.stationary_exact_half_transfer
            && self.rewired_full_search_rejected_everywhere
            && self.payload_only_inert_everywhere
            && self.counterfeit_rejected_everywhere
            && self.foreign_root_rejected_everywhere
            && self.delayed_zero_during_window
            && self.delayed_eventual_admission
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
        let report = evaluate_split(&roots[start..end])?;
        let orbit = path_metrics(&report, PathKind::OrbitAwareStateful);
        let baseline = path_metrics(&report, PathKind::PartitionOnlyBaseline);
        let stationary = path_metrics(&report, PathKind::TargetStationaryMatchedCalibration);
        future_families.push(FutureFamilyReport {
            family: FAMILIES[TRAIN_FAMILIES + HOLDOUT_FAMILIES + family_index],
            roots: ROOTS_PER_FAMILY,
            orbit_aware_success_rate: orbit.success_rate,
            orbit_aware_correct_predictions: orbit.total_correct_predictions,
            partition_only_success_rate: baseline.success_rate,
            partition_only_correct_predictions: baseline.total_correct_predictions,
            stationary_success_rate: stationary.success_rate,
            stationary_correct_predictions: stationary.total_correct_predictions,
        });
    }

    let cohort_exact = train.len() == 16
        && holdout.len() == 8
        && future.len() == 32
        && future_families.len() == FUTURE_FAMILIES;
    let structural_audits_exact = root_audit.alias_defects_exact == roots.len()
        && root_audit.omega1_partition_search_exact == roots.len()
        && root_audit.omega1_accidental_representative_exact == roots.len()
        && root_audit.primary_transport_frontier_exact == roots.len()
        && root_audit.stationary_transport_frontier_exact == roots.len()
        && root_audit.rewired_transport_frontier_exact == roots.len();
    let orbit_aware_training = perfect_orbit(&training);
    let orbit_aware_holdout = perfect_orbit(&holdout_report);
    let orbit_aware_future = perfect_orbit(&future_report);
    let partition_only_exact_half_transfer = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| exact_half_transfer(split, PathKind::PartitionOnlyBaseline));
    let stationary_exact_half_transfer = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| exact_half_transfer(split, PathKind::TargetStationaryMatchedCalibration));
    let rewired_full_search_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::RewiredCorrespondenceCalibration);
            metrics.validation_successes == 0
                && metrics.validation_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
                && metrics.minimum_zero_violation_representatives == 0
                && metrics.maximum_zero_violation_representatives == 0
                && metrics.minimum_transport_violations == EXPECTED_REWIRED_MIN_VIOLATIONS
                && metrics.maximum_transport_violations == EXPECTED_REWIRED_MIN_VIOLATIONS
        });
    let payload_only_inert_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::TransportPayloadOnly);
            metrics.validation_successes == metrics.roots
                && metrics.admissions_during_window == 0
                && metrics.final_admissions == 0
                && metrics.payload_preservations == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let counterfeit_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::CounterfeitTransportProof);
            metrics.validation_successes == 0
                && metrics.validation_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let foreign_root_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::ForeignRootTransportCertificate);
            metrics.validation_successes == metrics.roots
                && metrics.admission_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let delayed_zero_during_window = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| path_metrics(split, PathKind::DelayedTransportAdmission).total_correct_predictions == 0);
    let delayed_eventual_admission = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::DelayedTransportAdmission);
            metrics.admissions_during_window == 0 && metrics.final_admissions == metrics.roots
        });
    let all_future_families_transfer = future_families.iter().all(|family| {
        family.orbit_aware_success_rate == 1.0
            && family.orbit_aware_correct_predictions == ROOTS_PER_FAMILY * PREDICTIONS_PER_ROOT
            && family.partition_only_success_rate == 0.0
            && family.partition_only_correct_predictions == ROOTS_PER_FAMILY * EXPECTED_BASELINE_CORRECT
            && family.stationary_success_rate == 0.0
            && family.stationary_correct_predictions == ROOTS_PER_FAMILY * EXPECTED_BASELINE_CORRECT
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
        orbit_aware_training,
        orbit_aware_holdout,
        orbit_aware_future,
        partition_only_exact_half_transfer,
        stationary_exact_half_transfer,
        rewired_full_search_rejected_everywhere,
        payload_only_inert_everywhere,
        counterfeit_rejected_everywhere,
        foreign_root_rejected_everywhere,
        delayed_zero_during_window,
        delayed_eventual_admission,
        all_future_families_transfer,
        budgets_exact,
        replay_exact,
        invariants_hold,
    };

    let terminal_classification = if !budgets_exact
        || !invariants_hold
        || !foreign_root_rejected_everywhere
        || !counterfeit_rejected_everywhere
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
        experiment: "ΩR1 representative transportability",
        mechanism: "retain every executable program in the winning discovery behavioral partition, then independently select and validate a representative by same-history consistency over a preregistered structure-preserving calibration transformation suite without transformed outcome labels",
        claim_boundary: "a PASS supports transport-stable representative identification under the frozen symbolic pair-block transformation regime; it does not establish automatic discovery of transformation groups, unrestricted semantic equivalence, a solved descendant chain, AGI, consciousness, or human-level cognition",
        frozen_contract: FrozenContract {
            preregistration_commit: PREREGISTRATION_COMMIT,
            train_roots: 16,
            holdout_roots: 8,
            future_roots: 32,
            future_families: FUTURE_FAMILIES,
            raw_atoms: RAW_ATOMS,
            discovery_histories: DISCOVERY_HISTORIES,
            heldout_transformations: HELDOUT_TRANSFORMATIONS,
            predictions_per_root: PREDICTIONS_PER_ROOT,
            alias_defects: EXPECTED_ALIAS_DEFECTS,
            candidate_programs: EXPECTED_CANDIDATES,
            discovery_program_history_evaluations: EXPECTED_DISCOVERY_EVALUATIONS,
            unique_partitions: EXPECTED_UNIQUE_PARTITIONS,
            winner_repair: EXPECTED_WINNER_REPAIR,
            runner_up_repair: EXPECTED_RUNNER_UP,
            winner_margin: EXPECTED_MARGIN,
            partition_support: EXPECTED_SUPPORT,
            winning_class_representatives: EXPECTED_WINNING_CLASS,
            calibration_transformations: EXPECTED_CALIBRATION_TRANSFORMS,
            transport_program_history_evaluations: EXPECTED_TRANSPORT_EVALUATIONS,
            primary_zero_violation_representatives: EXPECTED_PRIMARY_ZERO_VIOLATION,
            stationary_zero_violation_representatives: EXPECTED_STATIONARY_ZERO_VIOLATION,
            rewired_zero_violation_representatives: EXPECTED_REWIRED_ZERO_VIOLATION,
            rewired_minimum_violations: EXPECTED_REWIRED_MIN_VIOLATIONS,
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
