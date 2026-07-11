use serde::Serialize;
use star::commitment_state::Atom;
use star::representation_genesis::{
    detect_alias_defects, RawHistory, RefinementProblem, StateLanguage, WitnessedHistory,
};
use star::representation_transport::{
    synthesize_transport_refinement, validate_transport_refinement, BlockPermutation,
    CorrespondenceMode, TransformationSuite, TransportBudget, TransportConfig, TransportProof,
    TransportStateLanguage, ValidatedTransportCertificate,
};
use star::representation_transport_descendants::{
    audit_raw_expressibility, synthesize_transport_descendant, validate_transport_descendant,
    RawExpressibilityAudit, TransportDescendantBudget, TransportDescendantConfig,
    TransportDescendantError, TransportDescendantProof, TransportDescendantStateLanguage,
    ValidatedTransportDescendantCertificate,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const PREREGISTRATION_COMMIT: &str = "26e7c836200e31ccc3b8cdfdfe3755428ab7d619";
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
const EXPECTED_RAW_CANDIDATES: usize = 828;
const EXPECTED_RAW_EVALUATIONS: usize = 13_248;
const EXPECTED_RAW_UNIQUE_PARTITIONS: usize = 5;
const EXPECTED_RAW_BEST_REPAIR: usize = 32;
const EXPECTED_ANCESTOR_WINNER: usize = 64;
const EXPECTED_ANCESTOR_RUNNER_UP: usize = 32;
const EXPECTED_ANCESTOR_MARGIN: usize = 32;
const EXPECTED_ANCESTOR_SUPPORT: usize = 8;
const EXPECTED_WINNING_CLASS: usize = 72;
const EXPECTED_TRANSPORT_EVALUATIONS: usize = 2_304;
const EXPECTED_CORRECT_ZERO_VIOLATION: usize = 8;
const EXPECTED_WRONG_ZERO_VIOLATION: usize = 8;
const EXPECTED_STATIONARY_ZERO_VIOLATION: usize = 72;
const EXPECTED_DESCENDANT_CANDIDATES: usize = 1_656;
const EXPECTED_DESCENDANT_EVALUATIONS: usize = 26_496;
const EXPECTED_DESCENDANT_UNIQUE_PARTITIONS: usize = 5;
const EXPECTED_DESCENDANT_WINNER: usize = 64;
const EXPECTED_DESCENDANT_RUNNER_UP: usize = 32;
const EXPECTED_DESCENDANT_MARGIN: usize = 32;
const EXPECTED_DESCENDANT_SUPPORT: usize = 8;
const EXPECTED_STATIONARY_CORRECT: usize = 16;

const FAMILIES: [&str; TRAIN_FAMILIES + HOLDOUT_FAMILIES + FUTURE_FAMILIES] = [
    "thermal_cascade",
    "hydraulic_cascade",
    "ecological_cascade",
    "cellular_cascade",
    "manufacturing_cascade",
    "software_cascade",
    "watershed_cascade",
];

#[derive(Debug, Clone)]
struct RootTask {
    root_id: u64,
    family: &'static str,
    correct_ancestor_problem: RefinementProblem,
    wrong_ancestor_problem: RefinementProblem,
    stage2_problem: RefinementProblem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum PathKind {
    TransportDescendantChain,
    L0RawSearch,
    L0DescendantNoAncestor,
    AncestorCertificatePayloadOnly,
    WrongTransportCertifiedAncestor,
    ExactAncestorReplacedBeforeDescendantValidation,
    StationaryAncestorDescendantChain,
    DescendantPayloadOnly,
    CounterfeitDescendantProof,
    DelayedDescendantAdmission,
}

impl PathKind {
    fn all() -> [Self; 10] {
        [
            Self::TransportDescendantChain,
            Self::L0RawSearch,
            Self::L0DescendantNoAncestor,
            Self::AncestorCertificatePayloadOnly,
            Self::WrongTransportCertifiedAncestor,
            Self::ExactAncestorReplacedBeforeDescendantValidation,
            Self::StationaryAncestorDescendantChain,
            Self::DescendantPayloadOnly,
            Self::CounterfeitDescendantProof,
            Self::DelayedDescendantAdmission,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Self::TransportDescendantChain => "transport_descendant_chain",
            Self::L0RawSearch => "l0_raw_search",
            Self::L0DescendantNoAncestor => "l0_descendant_no_ancestor",
            Self::AncestorCertificatePayloadOnly => "ancestor_certificate_payload_only",
            Self::WrongTransportCertifiedAncestor => "wrong_transport_certified_ancestor",
            Self::ExactAncestorReplacedBeforeDescendantValidation => {
                "exact_ancestor_replaced_before_descendant_validation"
            }
            Self::StationaryAncestorDescendantChain => "stationary_ancestor_descendant_chain",
            Self::DescendantPayloadOnly => "descendant_payload_only",
            Self::CounterfeitDescendantProof => "counterfeit_descendant_proof",
            Self::DelayedDescendantAdmission => "delayed_descendant_admission",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
struct BudgetLedger {
    transport_proposals: Vec<TransportBudget>,
    transport_validations: Vec<TransportBudget>,
    raw_audits: Vec<TransportDescendantBudget>,
    descendant_proposals: Vec<TransportDescendantBudget>,
    descendant_validations: Vec<TransportDescendantBudget>,
    ancestor_admission_slots: usize,
    descendant_admission_slots: usize,
    heldout_transformation_applications: usize,
    discovery_key_index_passes: usize,
    prediction_attempts: usize,
    objective_checks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Execution {
    correct_predictions: usize,
    full_success: bool,
    selected_ancestor_program: Option<String>,
    correct_ancestor_proposal_succeeded: bool,
    correct_ancestor_validation_succeeded: bool,
    correct_ancestor_admitted: bool,
    wrong_ancestor_proposal_succeeded: bool,
    wrong_ancestor_validation_succeeded: bool,
    wrong_ancestor_admitted: bool,
    stationary_ancestor_proposal_succeeded: bool,
    stationary_ancestor_validation_succeeded: bool,
    stationary_ancestor_admitted: bool,
    descendant_proposal_succeeded: bool,
    descendant_validation_succeeded: bool,
    descendant_admitted_during_window: bool,
    descendant_admitted_final: bool,
    no_ancestor_rejection: bool,
    descendant_validation_rejected: bool,
    raw_audit_complete_repair: bool,
    ancestor_payload_preserved: bool,
    descendant_payload_preserved: bool,
    descendant_repaired_pairs: usize,
    descendant_candidate_count: usize,
    descendant_unique_partitions: usize,
    descendant_winner_margin: usize,
    descendant_partition_support: usize,
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
    correct_ancestor_alias_defects_exact: usize,
    wrong_ancestor_alias_defects_exact: usize,
    correct_transport_frontier_exact: usize,
    wrong_transport_frontier_exact: usize,
    stationary_transport_frontier_exact: usize,
    l0_raw_audit_exact: usize,
    l0_descendant_empty_exact: usize,
    correct_descendant_frontier_exact: usize,
}

#[derive(Debug, Clone, Serialize)]
struct PathMetrics {
    roots: usize,
    full_successes: usize,
    success_rate: f64,
    total_correct_predictions: usize,
    correct_ancestor_admissions: usize,
    wrong_ancestor_admissions: usize,
    stationary_ancestor_admissions: usize,
    descendant_proposal_successes: usize,
    descendant_validation_successes: usize,
    descendant_validation_rejections: usize,
    descendant_admissions_during_window: usize,
    descendant_final_admissions: usize,
    no_ancestor_rejections: usize,
    raw_audit_complete_repairs: usize,
    ancestor_payload_preservations: usize,
    descendant_payload_preservations: usize,
    stable_correct_ancestor_selections: usize,
    stable_wrong_ancestor_selections: usize,
    stationary_accidental_ancestor_selections: usize,
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
    stateful_correct_predictions: usize,
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
    raw_candidates: usize,
    raw_evaluations: usize,
    raw_unique_partitions: usize,
    raw_best_repair: usize,
    ancestor_winner: usize,
    ancestor_runner_up: usize,
    ancestor_margin: usize,
    ancestor_support: usize,
    ancestor_winning_class: usize,
    transport_evaluations: usize,
    correct_zero_violation_representatives: usize,
    wrong_zero_violation_representatives: usize,
    stationary_zero_violation_representatives: usize,
    descendant_candidates: usize,
    descendant_evaluations: usize,
    descendant_unique_partitions: usize,
    descendant_winner: usize,
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
    l0_raw_zero_predictions: bool,
    l0_descendant_empty_everywhere: bool,
    ancestor_payload_does_not_create_terminal: bool,
    wrong_transport_ancestor_rejected_everywhere: bool,
    exact_ancestor_replacement_rejected_everywhere: bool,
    stationary_chain_exact_half_transfer: bool,
    descendant_payload_only_inert_everywhere: bool,
    counterfeit_descendant_rejected_everywhere: bool,
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
            && self.stateful_training
            && self.stateful_holdout
            && self.stateful_future
            && self.l0_raw_zero_predictions
            && self.l0_descendant_empty_everywhere
            && self.ancestor_payload_does_not_create_terminal
            && self.wrong_transport_ancestor_rejected_everywhere
            && self.exact_ancestor_replacement_rejected_everywhere
            && self.stationary_chain_exact_half_transfer
            && self.descendant_payload_only_inert_everywhere
            && self.counterfeit_descendant_rejected_everywhere
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
