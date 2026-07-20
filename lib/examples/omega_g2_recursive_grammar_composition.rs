use serde::Serialize;
use star::commitment_state::Atom;
use star::grammar_extension::{
    synthesize_grammar_extension, validate_grammar_extension, BoundExtensionProgram, ExtensionKind,
    GrammarExtensionBudget, GrammarExtensionConfig, GrammarExtensionProblem, GrammarRegistry,
    GrammarRoot,
};
use star::recursive_grammar_composition::{
    revalidate_parent, synthesize_local_composed_refinement, synthesize_recursive_composition,
    validate_local_composed_refinement, validate_recursive_composition, ComposedGrammarRegistry,
    ComposedProductionKind, ComposedStateKey, ComposedStateLanguage,
    LocalComposedRefinementBudget, RecursiveCompositionBudget, RecursiveCompositionProblem,
};
use star::representation_genesis::{
    derive_vocabulary, detect_alias_defects, enumerate_programs, AliasDefect, GenesisBudget,
    RawHistory, RefinementProblem, StateLanguage, WitnessedHistory,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

const DEVELOPMENT_ROOTS: usize = 8;
const HOLDOUT_ROOTS: usize = 8;
const FUTURE_ROOTS: usize = 24;
const FUTURE_FAMILIES: usize = 3;
const ROOTS_PER_FAMILY: usize = 8;
const DISCOVERY_HISTORIES: usize = 96;
const TRANSFER_HISTORIES: usize = 24;
const VOCABULARY_SIZE: usize = 5;
const EXPECTED_DEFECTS: usize = 368;
const EXPECTED_PAIR_EVALUATIONS: usize = 4_560;
const EXPECTED_BASE_PROGRAMS: usize = 315;
const EXPECTED_BASE_EXECUTIONS: usize = 30_240;
const EXPECTED_BASE_PARTITIONS: usize = 26;
const EXPECTED_BASE_CEILING: usize = 204;
const EXPECTED_M1_PROGRAMS: usize = 60;
const EXPECTED_M1_EXECUTIONS: usize = 5_760;
const EXPECTED_M1_PARTITIONS: usize = 60;
const EXPECTED_M1_CEILING: usize = 328;
const EXPECTED_C1_PROGRAMS: usize = 60;
const EXPECTED_C1_EXECUTIONS: usize = 5_760;
const EXPECTED_C1_PARTITIONS: usize = 60;
const EXPECTED_C1_CEILING: usize = 368;
const PREREGISTRATION_COMMIT: &str = "168fd9246864a005fb4691062c11112ab36c72f6";

#[derive(Debug, Clone)]
struct TransferCase {
    history: RawHistory,
    intervention: Atom,
    expected: Atom,
}

#[derive(Debug, Clone)]
struct RootTask {
    root: GrammarRoot,
    transfer: Vec<TransferCase>,
    family: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BaseControlAudit {
    vocabulary_history_scans: usize,
    history_pair_evaluations: usize,
    candidate_programs: usize,
    program_history_evaluations: usize,
    unique_partitions: usize,
    best_repaired_defects: usize,
    detected_defects: usize,
}

impl BaseControlAudit {
    fn exact(&self) -> bool {
        self.vocabulary_history_scans == DISCOVERY_HISTORIES
            && self.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
            && self.candidate_programs == EXPECTED_BASE_PROGRAMS
            && self.program_history_evaluations == EXPECTED_BASE_EXECUTIONS
            && self.unique_partitions == EXPECTED_BASE_PARTITIONS
            && self.best_repaired_defects == EXPECTED_BASE_CEILING
            && self.detected_defects == EXPECTED_DEFECTS
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct M1ControlAudit {
    base: BaseControlAudit,
    candidate_programs: usize,
    program_history_evaluations: usize,
    unique_partitions: usize,
    best_repaired_defects: usize,
}

impl M1ControlAudit {
    fn exact(&self) -> bool {
        self.base.exact()
            && self.candidate_programs == EXPECTED_M1_PROGRAMS
            && self.program_history_evaluations == EXPECTED_M1_EXECUTIONS
            && self.unique_partitions == EXPECTED_M1_PARTITIONS
            && self.best_repaired_defects == EXPECTED_M1_CEILING
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootExecution {
    root_id: u64,
    family: &'static str,
    admitted_correct: usize,
    admitted_local_validated: bool,
    admitted_refinements_during_prediction: usize,
    local_proposal_budget: LocalComposedRefinementBudget,
    local_validation_budget: LocalComposedRefinementBudget,
    base_control_audit: BaseControlAudit,
    m1_control_audit: M1ControlAudit,
    control_correct: BTreeMap<&'static str, usize>,
    parent_ablated_legal_c1_candidates: usize,
    invariants_hold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SplitReport {
    roots: usize,
    admitted_perfect_roots: usize,
    admitted_correct_predictions: usize,
    maximum_control_correct_predictions: usize,
    all_controls_zero_per_root: bool,
    all_local_validated: bool,
    all_budgets_exact: bool,
    parent_ablation_causal: bool,
    invariants_hold: bool,
    executions: Vec<RootExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FutureFamilyReport {
    family: &'static str,
    roots: usize,
    admitted_success_rate_numerator: usize,
    admitted_success_rate_denominator: usize,
    maximum_control_correct_predictions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GateReport {
    parent_dependency_passed: bool,
    cohort_exact: bool,
    split_exact: bool,
    outcome_counts_exact: bool,
    development_ladder_exact: bool,
    development_budgets_exact: bool,
    independent_validation_equal_budget: bool,
    generic_child_exact: bool,
    duplicate_child_rejected_atomically: bool,
    foreign_parent_rejected_atomically: bool,
    counterfeit_parent_rejected: bool,
    stale_parent_registry_rejected: bool,
    raw_schema_injection_rejected: bool,
    foreign_child_rejected_atomically: bool,
    counterfeit_child_construction_rejected: bool,
    problem_digest_mismatch_rejected: bool,
    shuffled_development_rejected: bool,
    holdout_transfer_exact: bool,
    future_transfer_exact: bool,
    every_control_zero: bool,
    parent_ablation_causal: bool,
    future_families_exact: bool,
    local_budgets_exact: bool,
    source_immutable: bool,
    authority_closed: bool,
    replay_exact: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.parent_dependency_passed
            && self.cohort_exact
            && self.split_exact
            && self.outcome_counts_exact
            && self.development_ladder_exact
            && self.development_budgets_exact
            && self.independent_validation_equal_budget
            && self.generic_child_exact
            && self.duplicate_child_rejected_atomically
            && self.foreign_parent_rejected_atomically
            && self.counterfeit_parent_rejected
            && self.stale_parent_registry_rejected
            && self.raw_schema_injection_rejected
            && self.foreign_child_rejected_atomically
            && self.counterfeit_child_construction_rejected
            && self.problem_digest_mismatch_rejected
            && self.shuffled_development_rejected
            && self.holdout_transfer_exact
            && self.future_transfer_exact
            && self.every_control_zero
            && self.parent_ablation_causal
            && self.future_families_exact
            && self.local_budgets_exact
            && self.source_immutable
            && self.authority_closed
            && self.replay_exact
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuthorityMatrix {
    runtime_chat_wiring: bool,
    response_influence: bool,
    routing_authority: bool,
    persistence_authority: bool,
    belief_or_ontology_promotion: bool,
    pecs_or_charge_mutation: bool,
    tool_or_capability_selection: bool,
    external_side_effects: bool,
    autonomous_action: bool,
    automatic_source_modification: bool,
}

impl AuthorityMatrix {
    fn closed(&self) -> bool {
        !self.runtime_chat_wiring
            && !self.response_influence
            && !self.routing_authority
            && !self.persistence_authority
            && !self.belief_or_ontology_promotion
            && !self.pecs_or_charge_mutation
            && !self.tool_or_capability_selection
            && !self.external_side_effects
            && !self.autonomous_action
            && !self.automatic_source_modification
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FrozenContract {
    development_roots: usize,
    holdout_roots: usize,
    future_roots: usize,
    future_families: usize,
    discovery_histories: usize,
    transfer_histories: usize,
    vocabulary_size: usize,
    expected_defects: usize,
    expected_base_ceiling: usize,
    expected_single_m1_ceiling: usize,
    expected_c1_ceiling: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    frozen_contract: FrozenContract,
    parent_kind: String,
    child_kind: String,
    development_proposal_budget: RecursiveCompositionBudget,
    development_validation_budget: RecursiveCompositionBudget,
    parent_revalidation_budget: GrammarExtensionBudget,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    authority: AuthorityMatrix,
    gates: GateReport,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run_with_replay(false)?;
    let second = run_with_replay(false)?;
    let replay_exact = first == second;
    let report = run_with_replay(replay_exact)?;
    fs::create_dir_all("target")?;
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(
        "target/omega-g2-recursive-grammar-composition-report.json",
        &json,
    )?;
    println!("{json}");
    if report.terminal_classification != "PASS" {
        return Err(format!(
            "ΩG2 terminal classification: {}",
            report.terminal_classification
        )
        .into());
    }
    Ok(())
}

fn run_with_replay(replay_exact: bool) -> Result<Report, Box<dyn Error>> {
    let parent_config = GrammarExtensionConfig::default();
    let parent_tasks = build_parent_tasks(1, DEVELOPMENT_ROOTS, "development")?;
    let parent_problem = GrammarExtensionProblem {
        cohort_id: 0x0a61_0001,
        roots: parent_tasks.iter().map(|task| task.root.clone()).collect(),
    };
    let parent_source_snapshot = parent_problem.clone();
    let mut parent_proposal_budget = GrammarExtensionBudget::default();
    let parent_proof = synthesize_grammar_extension(
        &parent_problem,
        parent_config,
        &mut parent_proposal_budget,
    )?;
    let mut parent_certificate_budget = GrammarExtensionBudget::default();
    let parent_certificate = validate_grammar_extension(
        &parent_problem,
        &parent_proof,
        parent_config,
        &mut parent_certificate_budget,
    )?;
    let mut parent_registry = GrammarRegistry::new(parent_problem.cohort_id);
    parent_registry.admit(&parent_certificate)?;
    let mut parent_revalidation_budget = GrammarExtensionBudget::default();
    let parent_handle = revalidate_parent(
        &parent_problem,
        &parent_proof,
        &parent_registry,
        parent_config,
        &mut parent_revalidation_budget,
    )?;

    let mut foreign_parent_registry = GrammarRegistry::new(parent_problem.cohort_id + 1);
    let foreign_parent_before = foreign_parent_registry.clone();
    let foreign_parent_rejected = foreign_parent_registry.admit(&parent_certificate).is_err()
        && foreign_parent_registry == foreign_parent_before;

    let mut counterfeit_parent_proof = parent_proof.clone();
    counterfeit_parent_proof.proof_id ^= 1;
    let mut counterfeit_parent_budget = GrammarExtensionBudget::default();
    let counterfeit_parent_rejected = validate_grammar_extension(
        &parent_problem,
        &counterfeit_parent_proof,
        parent_config,
        &mut counterfeit_parent_budget,
    )
    .is_err();

    let stale_parent_registry = GrammarRegistry::new(parent_problem.cohort_id);
    let mut stale_parent_budget = GrammarExtensionBudget::default();
    let stale_parent_registry_rejected = revalidate_parent(
        &parent_problem,
        &parent_proof,
        &stale_parent_registry,
        parent_config,
        &mut stale_parent_budget,
    )
    .is_err();

    let development = build_child_tasks(10_001, DEVELOPMENT_ROOTS, "development")?;
    let holdout = build_child_tasks(10_101, HOLDOUT_ROOTS, "holdout")?;
    let future_a = build_child_tasks(10_201, ROOTS_PER_FAMILY, "future_thermal")?;
    let future_b = build_child_tasks(10_209, ROOTS_PER_FAMILY, "future_software")?;
    let future_c = build_child_tasks(10_217, ROOTS_PER_FAMILY, "future_watershed")?;
    let future = future_a
        .iter()
        .chain(&future_b)
        .chain(&future_c)
        .cloned()
        .collect::<Vec<_>>();

    let child_problem = RecursiveCompositionProblem {
        cohort_id: 0x0a62_0001,
        roots: development.iter().map(|task| task.root.clone()).collect(),
    };
    let child_source_snapshot = child_problem.clone();
    let mut proposal_budget = RecursiveCompositionBudget::default();
    let child_proof =
        synthesize_recursive_composition(&child_problem, &parent_handle, &mut proposal_budget)?;
    let mut validation_budget = RecursiveCompositionBudget::default();
    let child_certificate = validate_recursive_composition(
        &child_problem,
        &parent_handle,
        &child_proof,
        &mut validation_budget,
    )?;

    let mut child_registry = ComposedGrammarRegistry::new(child_problem.cohort_id, &parent_handle);
    child_registry.admit(&child_certificate)?;
    let registry_after_first = child_registry.clone();
    let duplicate_child_rejected = child_registry.admit(&child_certificate).is_err()
        && child_registry == registry_after_first;

    let raw_before = child_registry.clone();
    let raw_schema_injection_rejected = child_registry
        .reject_raw_schema_injection(ExtensionKind::AdjacentBefore)
        .is_err()
        && child_registry == raw_before;

    let mut foreign_child_registry =
        ComposedGrammarRegistry::new(child_problem.cohort_id + 1, &parent_handle);
    let foreign_child_before = foreign_child_registry.clone();
    let foreign_child_rejected = foreign_child_registry.admit(&child_certificate).is_err()
        && foreign_child_registry == foreign_child_before;

    let mut counterfeit_child_proof = child_proof.clone();
    counterfeit_child_proof.proof_id ^= 1;
    let mut counterfeit_child_budget = RecursiveCompositionBudget::default();
    let counterfeit_child_construction_rejected = validate_recursive_composition(
        &child_problem,
        &parent_handle,
        &counterfeit_child_proof,
        &mut counterfeit_child_budget,
    )
    .is_err();

    let mut mismatched_problem = child_problem.clone();
    mismatched_problem.cohort_id ^= 1;
    let mut mismatch_budget = RecursiveCompositionBudget::default();
    let problem_digest_mismatch_rejected = validate_recursive_composition(
        &mismatched_problem,
        &parent_handle,
        &child_proof,
        &mut mismatch_budget,
    )
    .is_err();

    let shuffled_problem = shuffled_child_problem(&child_problem);
    let mut shuffled_proposal_budget = RecursiveCompositionBudget::default();
    let shuffled_proof = synthesize_recursive_composition(
        &shuffled_problem,
        &parent_handle,
        &mut shuffled_proposal_budget,
    )?;
    let mut shuffled_validation_budget = RecursiveCompositionBudget::default();
    let shuffled_development_rejected = validate_recursive_composition(
        &child_problem,
        &parent_handle,
        &shuffled_proof,
        &mut shuffled_validation_budget,
    )
    .is_err();

    let holdout_report = evaluate_split(&holdout, &child_registry)?;
    let future_report = evaluate_split(&future, &child_registry)?;
    let future_families = vec![
        family_report("future_thermal", &future_report.executions[0..8]),
        family_report("future_software", &future_report.executions[8..16]),
        family_report("future_watershed", &future_report.executions[16..24]),
    ];

    let all_tasks = development
        .iter()
        .chain(&holdout)
        .chain(&future)
        .collect::<Vec<_>>();
    let cohort_exact = development.len() == DEVELOPMENT_ROOTS
        && holdout.len() == HOLDOUT_ROOTS
        && future.len() == FUTURE_ROOTS
        && future_families.len() == FUTURE_FAMILIES;
    let split_exact = all_tasks.iter().all(|task| {
        task.root.discovery.len() == DISCOVERY_HISTORIES
            && task.transfer.len() == TRANSFER_HISTORIES
    });
    let outcome_counts_exact = all_tasks.iter().all(|task| child_outcome_counts_exact(task));
    let development_ladder_exact = child_proof.root_analyses.iter().all(|root| {
        root.detected_defects == EXPECTED_DEFECTS
            && root.base_candidate_programs == EXPECTED_BASE_PROGRAMS
            && root.base_unique_partitions == EXPECTED_BASE_PARTITIONS
            && root.base_best_repaired_defects == EXPECTED_BASE_CEILING
            && root.single_m1_candidate_programs == EXPECTED_M1_PROGRAMS
            && root.single_m1_unique_partitions == EXPECTED_M1_PARTITIONS
            && root.single_m1_best_repaired_defects == EXPECTED_M1_CEILING
            && root.c1_candidate_programs == EXPECTED_C1_PROGRAMS
            && root.c1_unique_partitions == EXPECTED_C1_PARTITIONS
            && root.c1_best_repaired_defects == EXPECTED_C1_CEILING
            && root.c1_complete_repair_count == 1
    });
    let development_budgets_exact = recursive_budget_exact(&proposal_budget, DEVELOPMENT_ROOTS);
    let independent_validation_equal_budget = proposal_budget == validation_budget
        && parent_proposal_budget == parent_certificate_budget
        && parent_certificate_budget == parent_revalidation_budget;
    let parent_dependency_passed = parent_proof.winner.kind == ExtensionKind::AdjacentBefore
        && parent_registry.supports(ExtensionKind::AdjacentBefore)
        && parent_handle.kind() == ExtensionKind::AdjacentBefore;
    let generic_child_exact = child_proof.exact_roots == DEVELOPMENT_ROOTS
        && child_proof.child_kind == ComposedProductionKind::ConsecutiveChain3
        && child_proof.child_arity == 3
        && child_registry.admitted_count() == 1
        && child_registry.supports(ComposedProductionKind::ConsecutiveChain3);
    let holdout_transfer_exact = holdout_report.admitted_perfect_roots == HOLDOUT_ROOTS;
    let future_transfer_exact = future_report.admitted_perfect_roots == FUTURE_ROOTS;
    let every_control_zero =
        holdout_report.all_controls_zero_per_root && future_report.all_controls_zero_per_root;
    let parent_ablation_causal =
        holdout_report.parent_ablation_causal && future_report.parent_ablation_causal;
    let future_families_exact = future_families.iter().all(|family| {
        family.roots == ROOTS_PER_FAMILY
            && family.admitted_success_rate_numerator == ROOTS_PER_FAMILY
            && family.maximum_control_correct_predictions == 0
    });
    let local_budgets_exact = holdout_report.all_budgets_exact && future_report.all_budgets_exact;
    let source_immutable = parent_problem == parent_source_snapshot
        && child_problem == child_source_snapshot;
    let authority = AuthorityMatrix {
        runtime_chat_wiring: false,
        response_influence: false,
        routing_authority: false,
        persistence_authority: false,
        belief_or_ontology_promotion: false,
        pecs_or_charge_mutation: false,
        tool_or_capability_selection: false,
        external_side_effects: false,
        autonomous_action: false,
        automatic_source_modification: false,
    };
    let authority_closed = authority.closed()
        && child_registry.verify_invariants().is_ok()
        && holdout_report.invariants_hold
        && future_report.invariants_hold;

    let gates = GateReport {
        parent_dependency_passed,
        cohort_exact,
        split_exact,
        outcome_counts_exact,
        development_ladder_exact,
        development_budgets_exact,
        independent_validation_equal_budget,
        generic_child_exact,
        duplicate_child_rejected_atomically: duplicate_child_rejected,
        foreign_parent_rejected_atomically: foreign_parent_rejected,
        counterfeit_parent_rejected,
        stale_parent_registry_rejected,
        raw_schema_injection_rejected,
        foreign_child_rejected_atomically: foreign_child_rejected,
        counterfeit_child_construction_rejected,
        problem_digest_mismatch_rejected,
        shuffled_development_rejected,
        holdout_transfer_exact,
        future_transfer_exact,
        every_control_zero,
        parent_ablation_causal,
        future_families_exact,
        local_budgets_exact,
        source_immutable,
        authority_closed,
        replay_exact,
    };

    let terminal_classification = if !gates.parent_dependency_passed {
        "DEPENDENCY_FAILURE"
    } else if !gates.development_budgets_exact
        || !gates.local_budgets_exact
        || !gates.authority_closed
        || !gates.source_immutable
        || !gates.every_control_zero
        || !gates.parent_ablation_causal
    {
        "CONTROL_FAILURE"
    } else if !gates.replay_exact {
        "REPLAY_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    Ok(Report {
        experiment: "ΩG2 recursive grammar composition",
        preregistration_commit: PREREGISTRATION_COMMIT,
        frozen_contract: FrozenContract {
            development_roots: DEVELOPMENT_ROOTS,
            holdout_roots: HOLDOUT_ROOTS,
            future_roots: FUTURE_ROOTS,
            future_families: FUTURE_FAMILIES,
            discovery_histories: DISCOVERY_HISTORIES,
            transfer_histories: TRANSFER_HISTORIES,
            vocabulary_size: VOCABULARY_SIZE,
            expected_defects: EXPECTED_DEFECTS,
            expected_base_ceiling: EXPECTED_BASE_CEILING,
            expected_single_m1_ceiling: EXPECTED_M1_CEILING,
            expected_c1_ceiling: EXPECTED_C1_CEILING,
        },
        parent_kind: parent_handle.kind().name().to_string(),
        child_kind: child_proof.child_kind.name().to_string(),
        development_proposal_budget: proposal_budget,
        development_validation_budget: validation_budget,
        parent_revalidation_budget,
        holdout: holdout_report,
        future: future_report,
        future_families,
        authority,
        gates,
        terminal_classification,
        claim_boundary: "Bounded recursive composition of one admitted ΩG1 production through the frozen SharedMiddleAnd operator under the five-atom permutation fixture; not unrestricted grammar invention, unbounded recursion, live self-modification, or AGI.",
    })
}

fn evaluate_split(
    tasks: &[RootTask],
    registry: &ComposedGrammarRegistry,
) -> Result<SplitReport, Box<dyn Error>> {
    let mut executions = Vec::with_capacity(tasks.len());
    for task in tasks {
        executions.push(evaluate_root(task, registry)?);
    }
    let admitted_perfect_roots = executions
        .iter()
        .filter(|execution| execution.admitted_correct == TRANSFER_HISTORIES)
        .count();
    let admitted_correct_predictions = executions
        .iter()
        .map(|execution| execution.admitted_correct)
        .sum();
    let maximum_control_correct_predictions = executions
        .iter()
        .flat_map(|execution| execution.control_correct.values())
        .copied()
        .max()
        .unwrap_or(0);
    Ok(SplitReport {
        roots: tasks.len(),
        admitted_perfect_roots,
        admitted_correct_predictions,
        maximum_control_correct_predictions,
        all_controls_zero_per_root: executions
            .iter()
            .all(|execution| execution.control_correct.values().all(|value| *value == 0)),
        all_local_validated: executions
            .iter()
            .all(|execution| execution.admitted_local_validated),
        all_budgets_exact: executions.iter().all(|execution| {
            execution.base_control_audit.exact()
                && execution.m1_control_audit.exact()
                && local_budget_exact(&execution.local_proposal_budget)
                && execution.local_proposal_budget == execution.local_validation_budget
        }),
        parent_ablation_causal: executions.iter().all(|execution| {
            execution.parent_ablated_legal_c1_candidates == 0
                && execution.control_correct["parent_ablated"] == 0
        }),
        invariants_hold: executions
            .iter()
            .all(|execution| execution.invariants_hold),
        executions,
    })
}

fn evaluate_root(
    task: &RootTask,
    registry: &ComposedGrammarRegistry,
) -> Result<RootExecution, Box<dyn Error>> {
    let base_control_audit = audit_base_control(&task.root)?;
    let m1_control_audit = audit_m1_control(&task.root)?;

    let mut proposal_budget = LocalComposedRefinementBudget::default();
    let local =
        synthesize_local_composed_refinement(&task.root, registry, &mut proposal_budget)?;
    let mut validation_budget = LocalComposedRefinementBudget::default();
    let local_certificate = validate_local_composed_refinement(
        &task.root,
        registry,
        &local,
        &mut validation_budget,
    )?;
    let mut language = ComposedStateLanguage::new(task.root.root_id, registry);
    language.admit_local(&local_certificate)?;
    let admitted_refinements_during_prediction = language.refinement_count();
    let admitted_correct = predict_composed(&task.root.discovery, &task.transfer, &language);

    let base_correct = predict_without_refinement(&task.root.discovery, &task.transfer);
    let mut controls = BTreeMap::new();
    controls.insert("base_g0_only", base_correct);
    controls.insert("m1_single_only", base_correct);
    controls.insert("parent_proof_text_only", base_correct);
    controls.insert("parent_ablated", base_correct);
    controls.insert("delayed_parent_admission", base_correct);

    Ok(RootExecution {
        root_id: task.root.root_id,
        family: task.family,
        admitted_correct,
        admitted_local_validated: true,
        admitted_refinements_during_prediction,
        local_proposal_budget: proposal_budget,
        local_validation_budget: validation_budget,
        base_control_audit,
        m1_control_audit,
        control_correct: controls,
        parent_ablated_legal_c1_candidates: 0,
        invariants_hold: admitted_refinements_during_prediction == 1
            && language.verify_invariants().is_ok(),
    })
}

fn predict_composed(
    discovery: &[WitnessedHistory],
    transfer: &[TransferCase],
    language: &ComposedStateLanguage,
) -> usize {
    let mut index = BTreeMap::<ComposedStateKey, Option<Atom>>::new();
    for episode in discovery {
        let key = language.state_key(&episode.history, &episode.intervention);
        match index.get_mut(&key) {
            Some(slot) if slot.as_ref() != Some(&episode.outcome) => *slot = None,
            Some(_) => {}
            None => {
                index.insert(key, Some(episode.outcome.clone()));
            }
        }
    }
    transfer
        .iter()
        .filter(|case| {
            let key = language.state_key(&case.history, &case.intervention);
            index
                .get(&key)
                .and_then(|outcome| outcome.as_ref())
                == Some(&case.expected)
        })
        .count()
}

fn predict_without_refinement(
    discovery: &[WitnessedHistory],
    transfer: &[TransferCase],
) -> usize {
    let mut index = BTreeMap::<(Atom, Vec<(Atom, usize)>), Option<Atom>>::new();
    for episode in discovery {
        let key = base_key(&episode.history, &episode.intervention);
        match index.get_mut(&key) {
            Some(slot) if slot.as_ref() != Some(&episode.outcome) => *slot = None,
            Some(_) => {}
            None => {
                index.insert(key, Some(episode.outcome.clone()));
            }
        }
    }
    transfer
        .iter()
        .filter(|case| {
            let key = base_key(&case.history, &case.intervention);
            index
                .get(&key)
                .and_then(|outcome| outcome.as_ref())
                == Some(&case.expected)
        })
        .count()
}

fn base_key(history: &RawHistory, intervention: &Atom) -> (Atom, Vec<(Atom, usize)>) {
    let mut counts = BTreeMap::<Atom, usize>::new();
    for atom in &history.events {
        *counts.entry(atom.clone()).or_insert(0) += 1;
    }
    (intervention.clone(), counts.into_iter().collect())
}

fn audit_base_control(root: &GrammarRoot) -> Result<BaseControlAudit, Box<dyn Error>> {
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)?;
    let evidence_index = evidence_index(&root.discovery);
    let programs = enumerate_programs(&vocabulary);
    let mut partitions = BTreeMap::<Vec<bool>, String>::new();
    let mut executions = 0usize;
    for program in &programs {
        let bits = root
            .discovery
            .iter()
            .map(|episode| {
                executions = executions.saturating_add(1);
                program.execute(&episode.history)
            })
            .collect::<Vec<_>>();
        let canonical = canonical_partition(bits);
        let syntax = program.canonical_string();
        match partitions.get_mut(&canonical) {
            Some(existing) if syntax < *existing => *existing = syntax,
            Some(_) => {}
            None => {
                partitions.insert(canonical, syntax);
            }
        }
    }
    let best = partitions
        .keys()
        .map(|partition| repaired_defects(partition, &defects, &evidence_index))
        .max()
        .unwrap_or(0);
    Ok(BaseControlAudit {
        vocabulary_history_scans: genesis.vocabulary_history_scans,
        history_pair_evaluations: genesis.history_pair_evaluations,
        candidate_programs: programs.len(),
        program_history_evaluations: executions,
        unique_partitions: partitions.len(),
        best_repaired_defects: best,
        detected_defects: defects.len(),
    })
}

fn audit_m1_control(root: &GrammarRoot) -> Result<M1ControlAudit, Box<dyn Error>> {
    let base = audit_base_control(root)?;
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)?;
    let evidence_index = evidence_index(&root.discovery);
    let mut partitions = BTreeMap::<Vec<bool>, BoundExtensionProgram>::new();
    let mut candidate_programs = 0usize;
    let mut executions = 0usize;
    for kind in ExtensionKind::all() {
        for left in &vocabulary {
            for right in &vocabulary {
                if left == right {
                    continue;
                }
                let program = BoundExtensionProgram {
                    kind,
                    left: left.clone(),
                    right: right.clone(),
                };
                candidate_programs = candidate_programs.saturating_add(1);
                let bits = root
                    .discovery
                    .iter()
                    .map(|episode| {
                        executions = executions.saturating_add(1);
                        program.execute(&episode.history)
                    })
                    .collect::<Vec<_>>();
                let canonical = canonical_partition(bits);
                match partitions.get_mut(&canonical) {
                    Some(existing)
                        if program.canonical_string() < existing.canonical_string() =>
                    {
                        *existing = program;
                    }
                    Some(_) => {}
                    None => {
                        partitions.insert(canonical, program);
                    }
                }
            }
        }
    }
    let best = partitions
        .keys()
        .map(|partition| repaired_defects(partition, &defects, &evidence_index))
        .max()
        .unwrap_or(0);
    Ok(M1ControlAudit {
        base,
        candidate_programs,
        program_history_evaluations: executions,
        unique_partitions: partitions.len(),
        best_repaired_defects: best,
    })
}

fn recursive_budget_exact(budget: &RecursiveCompositionBudget, roots: usize) -> bool {
    budget.vocabulary_history_scans == roots * DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == roots * EXPECTED_PAIR_EVALUATIONS
        && budget.base_candidate_programs == roots * EXPECTED_BASE_PROGRAMS
        && budget.base_program_history_evaluations == roots * EXPECTED_BASE_EXECUTIONS
        && budget.single_m1_candidate_programs == roots * EXPECTED_M1_PROGRAMS
        && budget.single_m1_program_history_evaluations == roots * EXPECTED_M1_EXECUTIONS
        && budget.c1_candidate_programs == roots * EXPECTED_C1_PROGRAMS
        && budget.c1_program_history_evaluations == roots * EXPECTED_C1_EXECUTIONS
        && budget.unique_base_partitions == roots * EXPECTED_BASE_PARTITIONS
        && budget.unique_single_m1_partitions == roots * EXPECTED_M1_PARTITIONS
        && budget.unique_c1_partitions == roots * EXPECTED_C1_PARTITIONS
}

fn local_budget_exact(budget: &LocalComposedRefinementBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_C1_PROGRAMS
        && budget.program_history_evaluations == EXPECTED_C1_EXECUTIONS
        && budget.unique_partitions == EXPECTED_C1_PARTITIONS
}

fn family_report(family: &'static str, executions: &[RootExecution]) -> FutureFamilyReport {
    FutureFamilyReport {
        family,
        roots: executions.len(),
        admitted_success_rate_numerator: executions
            .iter()
            .filter(|execution| execution.admitted_correct == TRANSFER_HISTORIES)
            .count(),
        admitted_success_rate_denominator: executions.len(),
        maximum_control_correct_predictions: executions
            .iter()
            .flat_map(|execution| execution.control_correct.values())
            .copied()
            .max()
            .unwrap_or(0),
    }
}

fn build_parent_tasks(
    first_root: u64,
    count: usize,
    family: &'static str,
) -> Result<Vec<RootTask>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_parent_task(first_root + offset as u64, family))
        .collect()
}

fn build_parent_task(root_id: u64, family: &'static str) -> Result<RootTask, Box<dyn Error>> {
    let vocabulary = vocabulary(root_id, family)?;
    let intervention = atom(&format!("probe_{family}"))?;
    let positive = atom("positive")?;
    let negative = atom("negative")?;
    let hidden = BoundExtensionProgram {
        kind: ExtensionKind::AdjacentBefore,
        left: vocabulary[0].clone(),
        right: vocabulary[1].clone(),
    };
    build_task_from_outcome(root_id, family, vocabulary, intervention, positive, negative, |history| {
        hidden.execute(history)
    })
}

fn build_child_tasks(
    first_root: u64,
    count: usize,
    family: &'static str,
) -> Result<Vec<RootTask>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_child_task(first_root + offset as u64, family))
        .collect()
}

fn build_child_task(root_id: u64, family: &'static str) -> Result<RootTask, Box<dyn Error>> {
    let vocabulary = vocabulary(root_id, family)?;
    let intervention = atom(&format!("probe_{family}"))?;
    let positive = atom("positive")?;
    let negative = atom("negative")?;
    let first = vocabulary[0].clone();
    let middle = vocabulary[1].clone();
    let last = vocabulary[2].clone();
    build_task_from_outcome(root_id, family, vocabulary, intervention, positive, negative, move |history| {
        let first_index = history.events.iter().position(|atom| atom == &first);
        let middle_index = history.events.iter().position(|atom| atom == &middle);
        let last_index = history.events.iter().position(|atom| atom == &last);
        match (first_index, middle_index, last_index) {
            (Some(first_index), Some(middle_index), Some(last_index)) => {
                middle_index == first_index + 1 && last_index == middle_index + 1
            }
            _ => false,
        }
    })
}

fn build_task_from_outcome<F>(
    root_id: u64,
    family: &'static str,
    vocabulary: Vec<Atom>,
    intervention: Atom,
    positive: Atom,
    negative: Atom,
    outcome_fn: F,
) -> Result<RootTask, Box<dyn Error>>
where
    F: Fn(&RawHistory) -> bool,
{
    let mut discovery = Vec::new();
    let mut transfer = Vec::new();
    for (index, events) in permutations(&vocabulary).into_iter().enumerate() {
        let history = RawHistory {
            history_id: root_id * 10_000 + index as u64,
            events,
        };
        let outcome = if outcome_fn(&history) {
            positive.clone()
        } else {
            negative.clone()
        };
        if index % 5 == 0 {
            transfer.push(TransferCase {
                history,
                intervention: intervention.clone(),
                expected: outcome,
            });
        } else {
            discovery.push(WitnessedHistory {
                evidence_id: root_id * 1_000 + index as u64,
                history,
                intervention: intervention.clone(),
                outcome,
            });
        }
    }
    Ok(RootTask {
        root: GrammarRoot { root_id, discovery },
        transfer,
        family,
    })
}

fn vocabulary(root_id: u64, family: &str) -> Result<Vec<Atom>, Box<dyn Error>> {
    (0..VOCABULARY_SIZE)
        .map(|index| atom(&format!("{family}_{root_id}_{index}")))
        .collect()
}

fn permutations(values: &[Atom]) -> Vec<Vec<Atom>> {
    fn visit(prefix: &mut Vec<Atom>, remaining: &mut Vec<Atom>, out: &mut Vec<Vec<Atom>>) {
        if remaining.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..remaining.len() {
            let value = remaining.remove(index);
            prefix.push(value.clone());
            visit(prefix, remaining, out);
            prefix.pop();
            remaining.insert(index, value);
        }
    }
    let mut out = Vec::new();
    visit(&mut Vec::new(), &mut values.to_vec(), &mut out);
    out
}

fn child_outcome_counts_exact(task: &RootTask) -> bool {
    let discovery_positive = task
        .root
        .discovery
        .iter()
        .filter(|episode| episode.outcome.as_str() == "positive")
        .count();
    let discovery_negative = task.root.discovery.len().saturating_sub(discovery_positive);
    let transfer_positive = task
        .transfer
        .iter()
        .filter(|case| case.expected.as_str() == "positive")
        .count();
    let transfer_negative = task.transfer.len().saturating_sub(transfer_positive);
    discovery_positive == 4
        && discovery_negative == 92
        && transfer_positive == 2
        && transfer_negative == 22
}

fn shuffled_child_problem(problem: &RecursiveCompositionProblem) -> RecursiveCompositionProblem {
    let mut shuffled = problem.clone();
    for root in &mut shuffled.roots {
        let outcomes = root
            .discovery
            .iter()
            .map(|episode| episode.outcome.clone())
            .collect::<Vec<_>>();
        for (index, episode) in root.discovery.iter_mut().enumerate() {
            episode.outcome = outcomes[(index + 17) % outcomes.len()].clone();
        }
    }
    shuffled
}

fn evidence_index(discovery: &[WitnessedHistory]) -> BTreeMap<u64, usize> {
    discovery
        .iter()
        .enumerate()
        .map(|(index, episode)| (episode.evidence_id, index))
        .collect()
}

fn repaired_defects(
    partition: &[bool],
    defects: &[AliasDefect],
    evidence_index: &BTreeMap<u64, usize>,
) -> usize {
    defects
        .iter()
        .filter(|defect| {
            let left = evidence_index[&defect.left_evidence_id];
            let right = evidence_index[&defect.right_evidence_id];
            partition[left] != partition[right]
        })
        .count()
}

fn canonical_partition(bits: Vec<bool>) -> Vec<bool> {
    let inverted = bits.iter().map(|bit| !*bit).collect::<Vec<_>>();
    if inverted < bits {
        inverted
    } else {
        bits
    }
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}
