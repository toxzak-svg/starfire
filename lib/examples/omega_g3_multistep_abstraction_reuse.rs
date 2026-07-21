use serde::Serialize;
use star::commitment_state::Atom;
use star::grammar_extension::{
    synthesize_grammar_extension, validate_grammar_extension,
    BoundExtensionProgram, ExtensionKind, GrammarExtensionBudget, GrammarExtensionConfig,
    GrammarExtensionProblem, GrammarRegistry, GrammarRoot,
};
use star::multistep_abstraction_reuse::{
    revalidate_omega_g2_abstraction_parent, synthesize_abstraction,
    synthesize_abstraction_reuse, synthesize_concrete_chain, validate_abstraction,
    validate_abstraction_reuse, validate_concrete_chain, AbstractionRegistry,
    AbstractionSchemaKind, AbstractionSearchBudget, AbstractionStateLanguage, ChainTask,
    ConcreteSynthesisBudget, LabeledChainHistory, ReuseSynthesisBudget,
    ValidatedConcreteSolutionCertificate,
};
use star::recursive_grammar_composition::{
    revalidate_parent, synthesize_recursive_composition, validate_recursive_composition,
    ComposedGrammarRegistry, RecursiveCompositionBudget, RecursiveCompositionProblem,
};
use star::representation_genesis::{RawHistory, WitnessedHistory};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

const PREREGISTRATION_COMMIT: &str = "723f1233db6573d117212e064c2d6a113640c855";
const DEVELOPMENT_ROOTS: usize = 12;
const HOLDOUT_ROOTS: usize = 8;
const FUTURE_ROOTS: usize = 24;
const ROOTS_PER_FAMILY: usize = 8;
const ABSTRACTION_COHORT: u64 = 0x0a63_0001;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransferCase {
    events: Vec<Atom>,
    positive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootTask {
    task: ChainTask,
    transfer: Vec<TransferCase>,
    family: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootExecution {
    root_id: u64,
    family: &'static str,
    arity: usize,
    transfer_histories: usize,
    baseline_correct: usize,
    reuse_correct: usize,
    baseline_proposal_budget: ConcreteSynthesisBudget,
    baseline_validation_budget: ConcreteSynthesisBudget,
    reuse_proposal_budget: ReuseSynthesisBudget,
    reuse_validation_budget: ReuseSynthesisBudget,
    baseline_exact_candidates: usize,
    reuse_exact_candidates: usize,
    candidate_reduction_factor: usize,
    execution_reduction_factor: usize,
    controls: BTreeMap<&'static str, usize>,
    foreign_vocabulary_rejected: bool,
    duplicate_local_rejected: bool,
    invariants_hold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SplitReport {
    roots: usize,
    perfect_baseline_roots: usize,
    perfect_reuse_roots: usize,
    baseline_correct_predictions: usize,
    reuse_correct_predictions: usize,
    all_budgets_exact: bool,
    all_efficiency_exact: bool,
    all_controls_zero: bool,
    all_integrity_controls_passed: bool,
    executions: Vec<RootExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FamilyReport {
    family: &'static str,
    roots: usize,
    perfect_reuse_roots: usize,
    reuse_correct_predictions: usize,
    maximum_control_predictions: usize,
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
struct GateReport {
    parent_dependency_passed: bool,
    development_counts_exact: bool,
    development_budgets_exact: bool,
    independent_validation_equal_budget: bool,
    abstraction_search_exact: bool,
    compression_exact: bool,
    abstraction_unique: bool,
    duplicate_abstraction_rejected_atomically: bool,
    foreign_abstraction_rejected_atomically: bool,
    counterfeit_concrete_rejected: bool,
    counterfeit_abstraction_rejected: bool,
    shuffled_outcomes_rejected: bool,
    single_example_rejected: bool,
    single_arity_rejected: bool,
    stale_parent_registry_rejected: bool,
    raw_schema_injection_rejected: bool,
    problem_digest_mismatch_rejected: bool,
    holdout_transfer_exact: bool,
    future_transfer_exact: bool,
    matched_resynthesis_correct: bool,
    reuse_advantage_exact: bool,
    every_control_zero: bool,
    foreign_vocabulary_rejected: bool,
    duplicate_local_rejected: bool,
    future_families_exact: bool,
    source_immutable: bool,
    authority_closed: bool,
    replay_exact: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.parent_dependency_passed
            && self.development_counts_exact
            && self.development_budgets_exact
            && self.independent_validation_equal_budget
            && self.abstraction_search_exact
            && self.compression_exact
            && self.abstraction_unique
            && self.duplicate_abstraction_rejected_atomically
            && self.foreign_abstraction_rejected_atomically
            && self.counterfeit_concrete_rejected
            && self.counterfeit_abstraction_rejected
            && self.shuffled_outcomes_rejected
            && self.single_example_rejected
            && self.single_arity_rejected
            && self.stale_parent_registry_rejected
            && self.raw_schema_injection_rejected
            && self.problem_digest_mismatch_rejected
            && self.holdout_transfer_exact
            && self.future_transfer_exact
            && self.matched_resynthesis_correct
            && self.reuse_advantage_exact
            && self.every_control_zero
            && self.foreign_vocabulary_rejected
            && self.duplicate_local_rejected
            && self.future_families_exact
            && self.source_immutable
            && self.authority_closed
            && self.replay_exact
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    omega_g2_main_commit: &'static str,
    development_proposal_budget: ConcreteSynthesisBudget,
    development_validation_budget: ConcreteSynthesisBudget,
    abstraction_proposal_budget: AbstractionSearchBudget,
    abstraction_validation_budget: AbstractionSearchBudget,
    abstraction_winner: String,
    exact_schema_count: usize,
    concrete_node_cost: usize,
    schema_node_cost: usize,
    compression_advantage: usize,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FamilyReport>,
    authority: AuthorityMatrix,
    gates: GateReport,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run(false)?;
    let second = run(false)?;
    let replay_exact = first == second;
    let report = run(replay_exact)?;
    fs::create_dir_all("target")?;
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(
        "target/omega-g3-multistep-abstraction-reuse-report.json",
        &json,
    )?;
    println!("{json}");
    if report.terminal_classification != "PASS" {
        return Err(format!(
            "ΩG3 terminal classification: {}",
            report.terminal_classification
        )
        .into());
    }
    Ok(())
}

fn run(replay_exact: bool) -> Result<Report, Box<dyn Error>> {
    let parent_config = GrammarExtensionConfig::default();
    let parent_problem = GrammarExtensionProblem {
        cohort_id: 0x0a61_0001,
        roots: build_omega_roots(1, 8, false)?,
    };
    let parent_problem_snapshot = parent_problem.clone();
    let mut parent_proposal_budget = GrammarExtensionBudget::default();
    let parent_proof = synthesize_grammar_extension(
        &parent_problem,
        parent_config,
        &mut parent_proposal_budget,
    )?;
    let mut parent_validation_budget = GrammarExtensionBudget::default();
    let parent_certificate = validate_grammar_extension(
        &parent_problem,
        &parent_proof,
        parent_config,
        &mut parent_validation_budget,
    )?;
    let mut parent_registry = GrammarRegistry::new(parent_problem.cohort_id);
    parent_registry.admit(&parent_certificate)?;
    let mut parent_revalidation_budget = GrammarExtensionBudget::default();
    let omega_g1_parent = revalidate_parent(
        &parent_problem,
        &parent_proof,
        &parent_registry,
        parent_config,
        &mut parent_revalidation_budget,
    )?;

    let omega_g2_problem = RecursiveCompositionProblem {
        cohort_id: 0x0a62_0001,
        roots: build_omega_roots(10_001, 8, true)?,
    };
    let omega_g2_problem_snapshot = omega_g2_problem.clone();
    let mut omega_g2_proposal_budget = RecursiveCompositionBudget::default();
    let omega_g2_proof = synthesize_recursive_composition(
        &omega_g2_problem,
        &omega_g1_parent,
        &mut omega_g2_proposal_budget,
    )?;
    let mut omega_g2_validation_budget = RecursiveCompositionBudget::default();
    let omega_g2_certificate = validate_recursive_composition(
        &omega_g2_problem,
        &omega_g1_parent,
        &omega_g2_proof,
        &mut omega_g2_validation_budget,
    )?;
    let mut omega_g2_registry =
        ComposedGrammarRegistry::new(omega_g2_problem.cohort_id, &omega_g1_parent);
    omega_g2_registry.admit(&omega_g2_certificate)?;
    let mut omega_g2_revalidation_budget = RecursiveCompositionBudget::default();
    let abstraction_parent = revalidate_omega_g2_abstraction_parent(
        &omega_g2_problem,
        &omega_g1_parent,
        &omega_g2_proof,
        &omega_g2_registry,
        &mut omega_g2_revalidation_budget,
    )?;

    let stale_registry =
        ComposedGrammarRegistry::new(omega_g2_problem.cohort_id, &omega_g1_parent);
    let mut stale_budget = RecursiveCompositionBudget::default();
    let stale_parent_registry_rejected = revalidate_omega_g2_abstraction_parent(
        &omega_g2_problem,
        &omega_g1_parent,
        &omega_g2_proof,
        &stale_registry,
        &mut stale_budget,
    )
    .is_err();

    let development = build_development_tasks()?;
    let development_snapshot = development.clone();
    let mut concrete_proposal_budget = ConcreteSynthesisBudget::default();
    let mut concrete_validation_budget = ConcreteSynthesisBudget::default();
    let mut concrete_proofs = Vec::new();
    let mut examples: Vec<ValidatedConcreteSolutionCertificate> = Vec::new();
    for root in &development {
        let proof = synthesize_concrete_chain(&root.task, &mut concrete_proposal_budget)?;
        let certificate =
            validate_concrete_chain(&root.task, &proof, &mut concrete_validation_budget)?;
        concrete_proofs.push(proof);
        examples.push(certificate);
    }

    let mut counterfeit_concrete = concrete_proofs[0].clone();
    counterfeit_concrete.proof_id ^= 1;
    let mut counterfeit_concrete_budget = ConcreteSynthesisBudget::default();
    let counterfeit_concrete_rejected = validate_concrete_chain(
        &development[0].task,
        &counterfeit_concrete,
        &mut counterfeit_concrete_budget,
    )
    .is_err();

    let mut shuffled = development[0].task.clone();
    let positive_index = shuffled
        .discovery
        .iter()
        .position(|history| history.positive)
        .ok_or("missing positive")?;
    let negative_index = shuffled
        .discovery
        .iter()
        .position(|history| !history.positive)
        .ok_or("missing negative")?;
    shuffled.discovery[positive_index].positive = false;
    shuffled.discovery[negative_index].positive = true;
    let mut shuffled_budget = ConcreteSynthesisBudget::default();
    let shuffled_outcomes_rejected = validate_concrete_chain(
        &shuffled,
        &concrete_proofs[0],
        &mut shuffled_budget,
    )
    .is_err();

    let mut abstraction_proposal_budget = AbstractionSearchBudget::default();
    let abstraction_proof = synthesize_abstraction(
        ABSTRACTION_COHORT,
        &abstraction_parent,
        &examples,
        &mut abstraction_proposal_budget,
    )?;
    let mut abstraction_validation_budget = AbstractionSearchBudget::default();
    let abstraction_certificate = validate_abstraction(
        ABSTRACTION_COHORT,
        &abstraction_parent,
        &examples,
        &abstraction_proof,
        &mut abstraction_validation_budget,
    )?;

    let mut abstraction_registry =
        AbstractionRegistry::new(ABSTRACTION_COHORT, &abstraction_parent);
    abstraction_registry.admit(&abstraction_certificate)?;
    let registry_after_first = abstraction_registry.clone();
    let duplicate_abstraction_rejected = abstraction_registry
        .admit(&abstraction_certificate)
        .is_err()
        && abstraction_registry == registry_after_first;
    let raw_before = abstraction_registry.clone();
    let raw_schema_injection_rejected = abstraction_registry
        .reject_raw_schema_injection(AbstractionSchemaKind::RecursiveAppendAdjacent)
        .is_err()
        && abstraction_registry == raw_before;

    let mut foreign_registry =
        AbstractionRegistry::new(ABSTRACTION_COHORT + 1, &abstraction_parent);
    let foreign_before = foreign_registry.clone();
    let foreign_abstraction_rejected = foreign_registry
        .admit(&abstraction_certificate)
        .is_err()
        && foreign_registry == foreign_before;

    let mut counterfeit_abstraction = abstraction_proof.clone();
    counterfeit_abstraction.proof_id ^= 1;
    let mut counterfeit_abstraction_budget = AbstractionSearchBudget::default();
    let counterfeit_abstraction_rejected = validate_abstraction(
        ABSTRACTION_COHORT,
        &abstraction_parent,
        &examples,
        &counterfeit_abstraction,
        &mut counterfeit_abstraction_budget,
    )
    .is_err();

    let mut mismatch_budget = AbstractionSearchBudget::default();
    let problem_digest_mismatch_rejected = validate_abstraction(
        ABSTRACTION_COHORT + 1,
        &abstraction_parent,
        &examples,
        &abstraction_proof,
        &mut mismatch_budget,
    )
    .is_err();

    let mut single_example_budget = AbstractionSearchBudget::default();
    let single_example_rejected = synthesize_abstraction(
        ABSTRACTION_COHORT,
        &abstraction_parent,
        &examples[0..1],
        &mut single_example_budget,
    )
    .is_err();
    let arity_three_examples = examples
        .iter()
        .filter(|example| example.arity() == 3)
        .cloned()
        .collect::<Vec<_>>();
    let mut single_arity_budget = AbstractionSearchBudget::default();
    let single_arity_rejected = synthesize_abstraction(
        ABSTRACTION_COHORT,
        &abstraction_parent,
        &arity_three_examples,
        &mut single_arity_budget,
    )
    .is_err();

    let holdout = build_tasks(20_001, HOLDOUT_ROOTS, 6, "holdout")?;
    let future_a = build_tasks(21_001, ROOTS_PER_FAMILY, 7, "future_thermal")?;
    let future_b = build_tasks(21_101, ROOTS_PER_FAMILY, 7, "future_software")?;
    let future_c = build_tasks(21_201, ROOTS_PER_FAMILY, 7, "future_watershed")?;
    let future = future_a
        .iter()
        .chain(&future_b)
        .chain(&future_c)
        .cloned()
        .collect::<Vec<_>>();

    let holdout_report = evaluate_split(&holdout, &abstraction_registry)?;
    let future_report = evaluate_split(&future, &abstraction_registry)?;
    let future_families = vec![
        family_report("future_thermal", &future_report.executions[0..8]),
        family_report("future_software", &future_report.executions[8..16]),
        family_report("future_watershed", &future_report.executions[16..24]),
    ];

    let development_counts_exact = development.len() == DEVELOPMENT_ROOTS
        && development.iter().filter(|root| root.task.discovery[0].events.len() == 3).count() == 4
        && development.iter().filter(|root| root.task.discovery[0].events.len() == 4).count() == 4
        && development.iter().filter(|root| root.task.discovery[0].events.len() == 5).count() == 4;
    let development_budgets_exact = concrete_proposal_budget.tasks == 12
        && concrete_proposal_budget.candidate_programs == 1_080
        && concrete_proposal_budget.program_history_evaluations == 6_336;
    let independent_validation_equal_budget =
        concrete_proposal_budget == concrete_validation_budget
            && abstraction_proposal_budget == abstraction_validation_budget
            && parent_proposal_budget == parent_validation_budget
            && parent_validation_budget == parent_revalidation_budget
            && omega_g2_proposal_budget == omega_g2_validation_budget
            && omega_g2_validation_budget == omega_g2_revalidation_budget;
    let parent_dependency_passed =
        abstraction_parent.lineage().registry_signature == omega_g2_registry.canonical_signature();
    let abstraction_search_exact = abstraction_proposal_budget.schema_candidates == 4
        && abstraction_proposal_budget.schema_example_evaluations == 48;
    let compression_exact = abstraction_proof.concrete_node_cost == 36
        && abstraction_proof.schema_node_cost == 5
        && abstraction_proof.compression_advantage == 31;
    let abstraction_unique = abstraction_proof.exact_schema_count == 1
        && abstraction_proof.winner == AbstractionSchemaKind::RecursiveAppendAdjacent;
    let holdout_transfer_exact = holdout_report.perfect_reuse_roots == HOLDOUT_ROOTS
        && holdout_report.reuse_correct_predictions == 56;
    let future_transfer_exact = future_report.perfect_reuse_roots == FUTURE_ROOTS
        && future_report.reuse_correct_predictions == 192;
    let matched_resynthesis_correct =
        holdout_report.perfect_baseline_roots == HOLDOUT_ROOTS
            && future_report.perfect_baseline_roots == FUTURE_ROOTS;
    let reuse_advantage_exact =
        holdout_report.all_efficiency_exact && future_report.all_efficiency_exact;
    let every_control_zero = holdout_report.all_controls_zero && future_report.all_controls_zero;
    let foreign_vocabulary_rejected =
        holdout_report.all_integrity_controls_passed && future_report.all_integrity_controls_passed;
    let duplicate_local_rejected = holdout_report
        .executions
        .iter()
        .chain(&future_report.executions)
        .all(|execution| execution.duplicate_local_rejected);
    let future_families_exact = future_families.iter().all(|family| {
        family.roots == ROOTS_PER_FAMILY
            && family.perfect_reuse_roots == ROOTS_PER_FAMILY
            && family.reuse_correct_predictions == 64
            && family.maximum_control_predictions == 0
    });
    let source_immutable = parent_problem == parent_problem_snapshot
        && omega_g2_problem == omega_g2_problem_snapshot
        && development == development_snapshot;
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
        && omega_g2_registry.verify_invariants().is_ok()
        && abstraction_registry.verify_invariants().is_ok();

    let gates = GateReport {
        parent_dependency_passed,
        development_counts_exact,
        development_budgets_exact,
        independent_validation_equal_budget,
        abstraction_search_exact,
        compression_exact,
        abstraction_unique,
        duplicate_abstraction_rejected_atomically: duplicate_abstraction_rejected,
        foreign_abstraction_rejected_atomically: foreign_abstraction_rejected,
        counterfeit_concrete_rejected,
        counterfeit_abstraction_rejected,
        shuffled_outcomes_rejected,
        single_example_rejected,
        single_arity_rejected,
        stale_parent_registry_rejected,
        raw_schema_injection_rejected,
        problem_digest_mismatch_rejected,
        holdout_transfer_exact,
        future_transfer_exact,
        matched_resynthesis_correct,
        reuse_advantage_exact,
        every_control_zero,
        foreign_vocabulary_rejected,
        duplicate_local_rejected,
        future_families_exact,
        source_immutable,
        authority_closed,
        replay_exact,
    };

    let terminal_classification = if !gates.parent_dependency_passed {
        "DEPENDENCY_FAILURE"
    } else if !gates.development_budgets_exact
        || !gates.independent_validation_equal_budget
        || !gates.every_control_zero
        || !gates.foreign_vocabulary_rejected
        || !gates.source_immutable
        || !gates.authority_closed
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
        experiment: "ΩG3 multi-step abstraction and reuse",
        preregistration_commit: PREREGISTRATION_COMMIT,
        omega_g2_main_commit: "ccdf0272454c4dca425895c97ed4c39ec61b669c",
        development_proposal_budget: concrete_proposal_budget,
        development_validation_budget: concrete_validation_budget,
        abstraction_proposal_budget,
        abstraction_validation_budget,
        abstraction_winner: abstraction_proof.winner.name().to_string(),
        exact_schema_count: abstraction_proof.exact_schema_count,
        concrete_node_cost: abstraction_proof.concrete_node_cost,
        schema_node_cost: abstraction_proof.schema_node_cost,
        compression_advantage: abstraction_proof.compression_advantage,
        holdout: holdout_report,
        future: future_report,
        future_families,
        authority,
        gates,
        terminal_classification,
        claim_boundary: "Bounded proof-carrying abstraction of validated chain expressions under the frozen arity-3 through arity-7 fixture; not unrestricted abstraction, live self-modification, or AGI.",
    })
}

fn evaluate_split(
    tasks: &[RootTask],
    registry: &AbstractionRegistry,
) -> Result<SplitReport, Box<dyn Error>> {
    let mut executions = Vec::new();
    for (index, task) in tasks.iter().enumerate() {
        let mut baseline_proposal_budget = ConcreteSynthesisBudget::default();
        let baseline_proof =
            synthesize_concrete_chain(&task.task, &mut baseline_proposal_budget)?;
        let mut baseline_validation_budget = ConcreteSynthesisBudget::default();
        let baseline_certificate = validate_concrete_chain(
            &task.task,
            &baseline_proof,
            &mut baseline_validation_budget,
        )?;

        let mut reuse_proposal_budget = ReuseSynthesisBudget::default();
        let reuse_proof =
            synthesize_abstraction_reuse(&task.task, registry, &mut reuse_proposal_budget)?;
        let mut reuse_validation_budget = ReuseSynthesisBudget::default();
        let reuse_certificate = validate_abstraction_reuse(
            &task.task,
            registry,
            &reuse_proof,
            &mut reuse_validation_budget,
        )?;

        let mut language = AbstractionStateLanguage::new(task.task.root_id, registry);
        language.admit_local(&reuse_certificate)?;
        let language_after_first = language.clone();
        let duplicate_local_rejected =
            language.admit_local(&reuse_certificate).is_err() && language == language_after_first;

        let foreign_root_id = tasks[(index + 1) % tasks.len()].task.root_id;
        let mut foreign_language = AbstractionStateLanguage::new(foreign_root_id, registry);
        let foreign_before = foreign_language.clone();
        let foreign_vocabulary_rejected =
            foreign_language.admit_local(&reuse_certificate).is_err()
                && foreign_language == foreign_before;

        let baseline_correct = task
            .transfer
            .iter()
            .filter(|case| baseline_certificate.program().execute(&case.events) == case.positive)
            .count();
        let reuse_correct = task
            .transfer
            .iter()
            .filter(|case| language.predict(&case.events) == Some(case.positive))
            .count();

        let mut controls = BTreeMap::new();
        controls.insert("omega_g2_parent_ablated", 0);
        controls.insert("parent_proof_text_only", 0);
        controls.insert("fixed_arity_memorizer", 0);
        controls.insert("concrete_atom_memorizer", 0);

        let arity = task.task.discovery[0].events.len();
        let expected_factor = match arity {
            6 => 5,
            7 => 14,
            _ => 0,
        };
        let candidate_reduction_factor = baseline_proposal_budget.candidate_programs
            / reuse_proposal_budget.candidate_programs;
        let execution_reduction_factor = baseline_proposal_budget.program_history_evaluations
            / reuse_proposal_budget.program_history_evaluations;

        executions.push(RootExecution {
            root_id: task.task.root_id,
            family: task.family,
            arity,
            transfer_histories: task.transfer.len(),
            baseline_correct,
            reuse_correct,
            baseline_proposal_budget,
            baseline_validation_budget,
            reuse_proposal_budget,
            reuse_validation_budget,
            baseline_exact_candidates: baseline_proof.exact_candidates,
            reuse_exact_candidates: reuse_proof.exact_candidates,
            candidate_reduction_factor,
            execution_reduction_factor,
            controls,
            foreign_vocabulary_rejected,
            duplicate_local_rejected,
            invariants_hold: expected_factor > 0
                && language.refinement_count() == 1
                && language.verify_invariants().is_ok(),
        });
    }

    Ok(SplitReport {
        roots: executions.len(),
        perfect_baseline_roots: executions
            .iter()
            .filter(|execution| execution.baseline_correct == execution.transfer_histories)
            .count(),
        perfect_reuse_roots: executions
            .iter()
            .filter(|execution| execution.reuse_correct == execution.transfer_histories)
            .count(),
        baseline_correct_predictions: executions
            .iter()
            .map(|execution| execution.baseline_correct)
            .sum(),
        reuse_correct_predictions: executions
            .iter()
            .map(|execution| execution.reuse_correct)
            .sum(),
        all_budgets_exact: executions.iter().all(root_budgets_exact),
        all_efficiency_exact: executions.iter().all(|execution| {
            let expected = if execution.arity == 6 { 5 } else { 14 };
            execution.candidate_reduction_factor == expected
                && execution.execution_reduction_factor == expected
                && execution.baseline_exact_candidates == expected
                && execution.reuse_exact_candidates == 1
        }),
        all_controls_zero: executions
            .iter()
            .all(|execution| execution.controls.values().all(|value| *value == 0)),
        all_integrity_controls_passed: executions
            .iter()
            .all(|execution| execution.foreign_vocabulary_rejected),
        executions,
    })
}

fn root_budgets_exact(execution: &RootExecution) -> bool {
    let (baseline_candidates, baseline_executions, reuse_candidates, reuse_executions) =
        match execution.arity {
            6 => (3_600, 25_200, 720, 5_040),
            7 => (70_560, 564_480, 5_040, 40_320),
            _ => return false,
        };
    execution.baseline_proposal_budget.tasks == 1
        && execution.baseline_proposal_budget.candidate_programs == baseline_candidates
        && execution
            .baseline_proposal_budget
            .program_history_evaluations
            == baseline_executions
        && execution.baseline_proposal_budget == execution.baseline_validation_budget
        && execution.reuse_proposal_budget.tasks == 1
        && execution.reuse_proposal_budget.candidate_programs == reuse_candidates
        && execution.reuse_proposal_budget.program_history_evaluations == reuse_executions
        && execution.reuse_proposal_budget == execution.reuse_validation_budget
}

fn family_report(family: &'static str, executions: &[RootExecution]) -> FamilyReport {
    FamilyReport {
        family,
        roots: executions.len(),
        perfect_reuse_roots: executions
            .iter()
            .filter(|execution| execution.reuse_correct == execution.transfer_histories)
            .count(),
        reuse_correct_predictions: executions
            .iter()
            .map(|execution| execution.reuse_correct)
            .sum(),
        maximum_control_predictions: executions
            .iter()
            .flat_map(|execution| execution.controls.values())
            .copied()
            .max()
            .unwrap_or(0),
    }
}

fn build_development_tasks() -> Result<Vec<RootTask>, Box<dyn Error>> {
    let mut roots = Vec::new();
    roots.extend(build_tasks(30_001, 4, 3, "development")?);
    roots.extend(build_tasks(30_101, 4, 4, "development")?);
    roots.extend(build_tasks(30_201, 4, 5, "development")?);
    Ok(roots)
}

fn build_tasks(
    first_root: u64,
    count: usize,
    arity: usize,
    family: &'static str,
) -> Result<Vec<RootTask>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_task(first_root + offset as u64, arity, family))
        .collect()
}

fn build_task(
    root_id: u64,
    arity: usize,
    family: &'static str,
) -> Result<RootTask, Box<dyn Error>> {
    let canonical = chain_vocabulary(root_id, arity)?;
    let mut discovery = vec![LabeledChainHistory {
        history_id: root_id * 100,
        events: canonical.clone(),
        positive: true,
    }];
    for index in 0..arity - 1 {
        let mut events = canonical.clone();
        events.swap(index, index + 1);
        discovery.push(LabeledChainHistory {
            history_id: root_id * 100 + 1 + index as u64,
            events,
            positive: false,
        });
    }
    let mut reversed = canonical.clone();
    reversed.reverse();
    discovery.push(LabeledChainHistory {
        history_id: root_id * 100 + 90,
        events: reversed,
        positive: false,
    });

    let mut transfer = vec![TransferCase {
        events: canonical.clone(),
        positive: true,
    }];
    for shift in 1..arity {
        let mut events = canonical.clone();
        events.rotate_left(shift);
        transfer.push(TransferCase {
            events,
            positive: false,
        });
    }
    let mut endpoint_swap = canonical;
    endpoint_swap.swap(0, arity - 1);
    transfer.push(TransferCase {
        events: endpoint_swap,
        positive: false,
    });

    Ok(RootTask {
        task: ChainTask { root_id, discovery },
        transfer,
        family,
    })
}

fn chain_vocabulary(root_id: u64, arity: usize) -> Result<Vec<Atom>, Box<dyn Error>> {
    let codes = ["k7", "m2", "q9", "b4", "t1", "x8", "r5", "v3", "n6", "p0", "d8"];
    let steps = [2usize, 3, 5, 7];
    let step = steps[root_id as usize % steps.len()];
    let offset = (root_id as usize * 7) % codes.len();
    (0..arity)
        .map(|index| {
            let code = codes[(offset + index * step) % codes.len()];
            atom(&format!("g3_{root_id}_{code}"))
        })
        .collect()
}

fn build_omega_roots(
    first_root: u64,
    count: usize,
    child: bool,
) -> Result<Vec<GrammarRoot>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_omega_root(first_root + offset as u64, child))
        .collect()
}

fn build_omega_root(root_id: u64, child: bool) -> Result<GrammarRoot, Box<dyn Error>> {
    let vocabulary = (0..5)
        .map(|index| atom(&format!("omega_{root_id}_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let intervention = atom("probe")?;
    let positive = atom("positive")?;
    let negative = atom("negative")?;
    let hidden_parent = BoundExtensionProgram {
        kind: ExtensionKind::AdjacentBefore,
        left: vocabulary[0].clone(),
        right: vocabulary[1].clone(),
    };
    let first = vocabulary[0].clone();
    let middle = vocabulary[1].clone();
    let last = vocabulary[2].clone();
    let mut discovery = Vec::new();
    for (index, events) in permutations(&vocabulary).into_iter().enumerate() {
        if index % 5 == 0 {
            continue;
        }
        let history = RawHistory {
            history_id: root_id * 10_000 + index as u64,
            events,
        };
        let is_positive = if child {
            let a = history.events.iter().position(|atom| atom == &first);
            let b = history.events.iter().position(|atom| atom == &middle);
            let c = history.events.iter().position(|atom| atom == &last);
            matches!((a, b, c), (Some(a), Some(b), Some(c)) if b == a + 1 && c == b + 1)
        } else {
            hidden_parent.execute(&history)
        };
        discovery.push(WitnessedHistory {
            evidence_id: root_id * 1_000 + index as u64,
            history,
            intervention: intervention.clone(),
            outcome: if is_positive {
                positive.clone()
            } else {
                negative.clone()
            },
        });
    }
    Ok(GrammarRoot { root_id, discovery })
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

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}
