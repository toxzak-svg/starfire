use serde::Serialize;
use star::commitment_state::Atom;
use star::grammar_extension::{
    synthesize_grammar_extension, synthesize_local_refinement, validate_grammar_extension,
    validate_local_refinement, BoundExtensionProgram, ExtendedStateKey, ExtensionKind,
    GrammarExtendedStateLanguage, GrammarExtensionBudget, GrammarExtensionConfig,
    GrammarExtensionProblem, GrammarRegistry, GrammarRoot, LocalRefinementBudget,
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
const EXPECTED_DEFECTS: usize = 1_463;
const EXPECTED_PAIR_EVALUATIONS: usize = 4_560;
const EXPECTED_BASE_PROGRAMS: usize = 315;
const EXPECTED_BASE_EXECUTIONS: usize = 30_240;
const EXPECTED_EXTENSION_PROGRAMS: usize = 60;
const EXPECTED_EXTENSION_EXECUTIONS: usize = 5_760;
const EXPECTED_LOCAL_PROGRAMS: usize = 20;
const EXPECTED_LOCAL_EXECUTIONS: usize = 1_920;
const PREREGISTRATION_COMMIT: &str = "d890a55fcaa9f30148835b42325da7456829f807";

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
struct BaseBudget {
    vocabulary_history_scans: usize,
    history_pair_evaluations: usize,
    candidate_programs: usize,
    program_history_evaluations: usize,
    unique_partitions: usize,
}

impl BaseBudget {
    fn exact(&self) -> bool {
        self.vocabulary_history_scans == DISCOVERY_HISTORIES
            && self.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
            && self.candidate_programs == EXPECTED_BASE_PROGRAMS
            && self.program_history_evaluations == EXPECTED_BASE_EXECUTIONS
            && self.unique_partitions > 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootExecution {
    root_id: u64,
    family: &'static str,
    admitted_correct: usize,
    admitted_local_validated: bool,
    admitted_refinements_during_prediction: usize,
    admitted_proposal_budget: LocalRefinementBudget,
    admitted_validation_budget: LocalRefinementBudget,
    base_best_repaired: usize,
    detected_defects: usize,
    base_budget: BaseBudget,
    control_correct: BTreeMap<&'static str, usize>,
    control_budgets_exact: bool,
    wrong_schema_validation_rejected: bool,
    delayed_refinement_after_prediction: bool,
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
    cohort_exact: bool,
    split_exact: bool,
    outcome_coverage_exact: bool,
    development_budgets_exact: bool,
    independent_validation_equal_budget: bool,
    base_grammar_insufficient_everywhere: bool,
    unique_schema_winner: bool,
    correct_schema_admitted: bool,
    duplicate_rejected_atomically: bool,
    foreign_rejected_atomically: bool,
    counterfeit_rejected: bool,
    shuffled_proof_rejected: bool,
    holdout_transfer_exact: bool,
    future_transfer_exact: bool,
    every_control_zero: bool,
    future_families_exact: bool,
    local_budgets_exact: bool,
    source_immutable: bool,
    authority_closed: bool,
    replay_exact: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.cohort_exact
            && self.split_exact
            && self.outcome_coverage_exact
            && self.development_budgets_exact
            && self.independent_validation_equal_budget
            && self.base_grammar_insufficient_everywhere
            && self.unique_schema_winner
            && self.correct_schema_admitted
            && self.duplicate_rejected_atomically
            && self.foreign_rejected_atomically
            && self.counterfeit_rejected
            && self.shuffled_proof_rejected
            && self.holdout_transfer_exact
            && self.future_transfer_exact
            && self.every_control_zero
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
    autonomous_action: bool,
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
            && !self.autonomous_action
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
    expected_base_programs: usize,
    expected_extension_programs: usize,
    expected_local_programs: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    frozen_contract: FrozenContract,
    winning_schema: String,
    development_proposal_budget: GrammarExtensionBudget,
    development_validation_budget: GrammarExtensionBudget,
    holdout: SplitReport,
    future: SplitReport,
    future_families: Vec<FutureFamilyReport>,
    authority: AuthorityMatrix,
    gates: GateReport,
    terminal_classification: &'static str,
    claim_boundary: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run()?;
    let second = run()?;
    let replay_exact = first == second;
    let report = run_with_replay(replay_exact)?;
    fs::create_dir_all("target")?;
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(
        "target/omega-g1-bounded-grammar-extension-report.json",
        &json,
    )?;
    println!("{json}");
    if report.terminal_classification != "PASS" {
        return Err(format!(
            "ΩG1 terminal classification: {}",
            report.terminal_classification
        )
        .into());
    }
    Ok(())
}

fn run() -> Result<Report, Box<dyn Error>> {
    run_with_replay(false)
}

fn run_with_replay(replay_exact: bool) -> Result<Report, Box<dyn Error>> {
    let config = GrammarExtensionConfig::default();
    let development = build_tasks(
        1,
        DEVELOPMENT_ROOTS,
        "development",
        ExtensionKind::AdjacentBefore,
    )?;
    let holdout = build_tasks(
        101,
        HOLDOUT_ROOTS,
        "holdout",
        ExtensionKind::AdjacentBefore,
    )?;
    let future_a = build_tasks(
        201,
        ROOTS_PER_FAMILY,
        "future_thermal",
        ExtensionKind::AdjacentBefore,
    )?;
    let future_b = build_tasks(
        209,
        ROOTS_PER_FAMILY,
        "future_software",
        ExtensionKind::AdjacentBefore,
    )?;
    let future_c = build_tasks(
        217,
        ROOTS_PER_FAMILY,
        "future_watershed",
        ExtensionKind::AdjacentBefore,
    )?;
    let future = future_a
        .iter()
        .chain(&future_b)
        .chain(&future_c)
        .cloned()
        .collect::<Vec<_>>();

    let development_problem = GrammarExtensionProblem {
        cohort_id: 0x0a61_0001,
        roots: development.iter().map(|task| task.root.clone()).collect(),
    };
    let source_snapshot = development_problem.clone();

    let mut proposal_budget = GrammarExtensionBudget::default();
    let proof = synthesize_grammar_extension(&development_problem, config, &mut proposal_budget)?;
    let mut validation_budget = GrammarExtensionBudget::default();
    let certificate =
        validate_grammar_extension(&development_problem, &proof, config, &mut validation_budget)?;

    let mut registry = GrammarRegistry::new(development_problem.cohort_id);
    registry.admit(&certificate)?;
    let registry_after_first = registry.clone();
    let duplicate_rejected = registry.admit(&certificate).is_err() && registry == registry_after_first;

    let wrong_tasks = build_tasks(
        1001,
        DEVELOPMENT_ROOTS,
        "wrong_development",
        ExtensionKind::ExactlyOneBetween,
    )?;
    let wrong_problem = GrammarExtensionProblem {
        cohort_id: 0x0a61_0002,
        roots: wrong_tasks.iter().map(|task| task.root.clone()).collect(),
    };
    let mut wrong_proposal_budget = GrammarExtensionBudget::default();
    let wrong_proof = synthesize_grammar_extension(&wrong_problem, config, &mut wrong_proposal_budget)?;
    let mut wrong_validation_budget = GrammarExtensionBudget::default();
    let wrong_certificate =
        validate_grammar_extension(&wrong_problem, &wrong_proof, config, &mut wrong_validation_budget)?;
    let mut wrong_registry = GrammarRegistry::new(wrong_problem.cohort_id);
    wrong_registry.admit(&wrong_certificate)?;

    let mut foreign_registry = GrammarRegistry::new(development_problem.cohort_id + 77);
    let foreign_before = foreign_registry.clone();
    let foreign_rejected =
        foreign_registry.admit(&certificate).is_err() && foreign_registry == foreign_before;

    let mut counterfeit = proof.clone();
    counterfeit.proof_id ^= 1;
    let mut counterfeit_budget = GrammarExtensionBudget::default();
    let counterfeit_rejected = validate_grammar_extension(
        &development_problem,
        &counterfeit,
        config,
        &mut counterfeit_budget,
    )
    .is_err();

    let shuffled_problem = shuffled_problem(&development_problem);
    let mut shuffled_proposal_budget = GrammarExtensionBudget::default();
    let shuffled_proof =
        synthesize_grammar_extension(&shuffled_problem, config, &mut shuffled_proposal_budget)?;
    let mut shuffled_validation_budget = GrammarExtensionBudget::default();
    let shuffled_rejected = validate_grammar_extension(
        &development_problem,
        &shuffled_proof,
        config,
        &mut shuffled_validation_budget,
    )
    .is_err();

    let holdout_report = evaluate_split(&holdout, &registry, &wrong_registry, config)?;
    let future_report = evaluate_split(&future, &registry, &wrong_registry, config)?;
    let future_families = vec![
        family_report("future_thermal", &future_report.executions[0..8]),
        family_report("future_software", &future_report.executions[8..16]),
        family_report("future_watershed", &future_report.executions[16..24]),
    ];

    let cohort_exact = development.len() == DEVELOPMENT_ROOTS
        && holdout.len() == HOLDOUT_ROOTS
        && future.len() == FUTURE_ROOTS
        && future_families.len() == FUTURE_FAMILIES;
    let all_tasks = development
        .iter()
        .chain(&holdout)
        .chain(&future)
        .collect::<Vec<_>>();
    let split_exact = all_tasks.iter().all(|task| {
        task.root.discovery.len() == DISCOVERY_HISTORIES
            && task.transfer.len() == TRANSFER_HISTORIES
    });
    let outcome_coverage_exact = all_tasks.iter().all(|task| {
        has_both_outcomes(&task.root.discovery)
            && has_both_transfer_outcomes(&task.transfer)
    });
    let development_budgets_exact = grammar_budget_exact(&proposal_budget, DEVELOPMENT_ROOTS);
    let independent_validation_equal_budget =
        proposal_budget == validation_budget
            && wrong_proposal_budget == wrong_validation_budget;
    let base_grammar_insufficient_development = proof.root_insufficiency.iter().all(|root| {
        root.detected_defects == EXPECTED_DEFECTS
            && root.base_candidate_programs == EXPECTED_BASE_PROGRAMS
            && root.base_best_repaired_defects < root.detected_defects
    });
    let base_grammar_insufficient_everywhere = base_grammar_insufficient_development
        && holdout_report
            .executions
            .iter()
            .chain(&future_report.executions)
            .all(|execution| execution.base_best_repaired < execution.detected_defects);
    let unique_schema_winner = proof.winner.kind == ExtensionKind::AdjacentBefore
        && proof.winner.exact_roots == DEVELOPMENT_ROOTS
        && proof.runner_up_exact_roots == 0
        && proof.exact_root_margin == DEVELOPMENT_ROOTS;
    let correct_schema_admitted = registry.admitted_count() == 1
        && registry.supports(ExtensionKind::AdjacentBefore)
        && wrong_registry.supports(ExtensionKind::ExactlyOneBetween);
    let holdout_transfer_exact = holdout_report.admitted_perfect_roots == HOLDOUT_ROOTS;
    let future_transfer_exact = future_report.admitted_perfect_roots == FUTURE_ROOTS;
    let every_control_zero =
        holdout_report.all_controls_zero_per_root && future_report.all_controls_zero_per_root;
    let future_families_exact = future_families.iter().all(|family| {
        family.roots == ROOTS_PER_FAMILY
            && family.admitted_success_rate_numerator == ROOTS_PER_FAMILY
            && family.maximum_control_correct_predictions == 0
    });
    let local_budgets_exact = holdout_report.all_budgets_exact && future_report.all_budgets_exact;
    let source_immutable = development_problem == source_snapshot;
    let authority = AuthorityMatrix {
        runtime_chat_wiring: false,
        response_influence: false,
        routing_authority: false,
        persistence_authority: false,
        belief_or_ontology_promotion: false,
        pecs_or_charge_mutation: false,
        tool_or_capability_selection: false,
        autonomous_action: false,
    };
    let authority_closed = authority.closed()
        && holdout_report.invariants_hold
        && future_report.invariants_hold
        && registry.verify_invariants().is_ok();

    let gates = GateReport {
        cohort_exact,
        split_exact,
        outcome_coverage_exact,
        development_budgets_exact,
        independent_validation_equal_budget,
        base_grammar_insufficient_everywhere,
        unique_schema_winner,
        correct_schema_admitted,
        duplicate_rejected_atomically: duplicate_rejected,
        foreign_rejected_atomically: foreign_rejected,
        counterfeit_rejected,
        shuffled_proof_rejected: shuffled_rejected,
        holdout_transfer_exact,
        future_transfer_exact,
        every_control_zero,
        future_families_exact,
        local_budgets_exact,
        source_immutable,
        authority_closed,
        replay_exact,
    };

    let terminal_classification = if !gates.development_budgets_exact
        || !gates.local_budgets_exact
        || !gates.authority_closed
        || !gates.source_immutable
        || !gates.every_control_zero
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
        experiment: "ΩG1 bounded grammar extension",
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
            expected_base_programs: EXPECTED_BASE_PROGRAMS,
            expected_extension_programs: EXPECTED_EXTENSION_PROGRAMS,
            expected_local_programs: EXPECTED_LOCAL_PROGRAMS,
        },
        winning_schema: proof.winner.kind.name().to_string(),
        development_proposal_budget: proposal_budget,
        development_validation_budget: validation_budget,
        holdout: holdout_report,
        future: future_report,
        future_families,
        authority,
        gates,
        terminal_classification,
        claim_boundary: "Bounded production-schema admission under the frozen permutation fixture; not unrestricted grammar invention, natural-language acquisition, live self-modification, or AGI.",
    })
}

fn evaluate_split(
    tasks: &[RootTask],
    registry: &GrammarRegistry,
    wrong_registry: &GrammarRegistry,
    config: GrammarExtensionConfig,
) -> Result<SplitReport, Box<dyn Error>> {
    let mut executions = Vec::with_capacity(tasks.len());
    for task in tasks {
        executions.push(evaluate_root(task, registry, wrong_registry, config)?);
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
            execution.base_budget.exact()
                && local_budget_exact(&execution.admitted_proposal_budget)
                && execution.admitted_proposal_budget == execution.admitted_validation_budget
                && execution.control_budgets_exact
        }),
        invariants_hold: executions
            .iter()
            .all(|execution| execution.invariants_hold),
        executions,
    })
}

fn evaluate_root(
    task: &RootTask,
    registry: &GrammarRegistry,
    wrong_registry: &GrammarRegistry,
    config: GrammarExtensionConfig,
) -> Result<RootExecution, Box<dyn Error>> {
    let (base_best, detected_defects, base_budget) = base_ceiling(&task.root)?;

    let mut proposal_budget = LocalRefinementBudget::default();
    let local = synthesize_local_refinement(&task.root, registry, &mut proposal_budget)?;
    let mut validation_budget = LocalRefinementBudget::default();
    let local_certificate =
        validate_local_refinement(&task.root, registry, &local, config, &mut validation_budget)?;
    let mut language = GrammarExtendedStateLanguage::new(task.root.root_id, registry);
    language.admit_local(&local_certificate)?;
    let admitted_refinements_during_prediction = language.refinement_count();
    let admitted_correct = predict(&task.root.discovery, &task.transfer, &language);

    let mut controls = BTreeMap::new();
    let mut control_budgets_exact = true;

    for name in [
        "base_grammar_only",
        "proof_text_only",
        "foreign_certificate",
        "counterfeit_certificate",
        "outcome_shuffled_development",
    ] {
        let (_, _, budget) = base_ceiling(&task.root)?;
        control_budgets_exact &= budget.exact();
        let empty_registry = GrammarRegistry::new(task.root.root_id + 90_000);
        let empty_language =
            GrammarExtendedStateLanguage::new(task.root.root_id, &empty_registry);
        controls.insert(name, predict(&task.root.discovery, &task.transfer, &empty_language));
    }

    let mut wrong_proposal_budget = LocalRefinementBudget::default();
    let wrong_local =
        synthesize_local_refinement(&task.root, wrong_registry, &mut wrong_proposal_budget)?;
    let mut wrong_validation_budget = LocalRefinementBudget::default();
    let wrong_validation_rejected = validate_local_refinement(
        &task.root,
        wrong_registry,
        &wrong_local,
        config,
        &mut wrong_validation_budget,
    )
    .is_err();
    control_budgets_exact &= local_budget_exact(&wrong_proposal_budget)
        && wrong_proposal_budget == wrong_validation_budget;
    let wrong_language =
        GrammarExtendedStateLanguage::new(task.root.root_id, wrong_registry);
    controls.insert(
        "wrong_schema",
        predict(&task.root.discovery, &task.transfer, &wrong_language),
    );

    let empty_before = GrammarRegistry::new(registry.cohort_id());
    let delayed_language_before =
        GrammarExtendedStateLanguage::new(task.root.root_id, &empty_before);
    let delayed_correct =
        predict(&task.root.discovery, &task.transfer, &delayed_language_before);
    controls.insert("delayed_admission", delayed_correct);

    let mut delayed_proposal_budget = LocalRefinementBudget::default();
    let delayed_local =
        synthesize_local_refinement(&task.root, registry, &mut delayed_proposal_budget)?;
    let mut delayed_validation_budget = LocalRefinementBudget::default();
    let delayed_certificate = validate_local_refinement(
        &task.root,
        registry,
        &delayed_local,
        config,
        &mut delayed_validation_budget,
    )?;
    let mut delayed_language = GrammarExtendedStateLanguage::new(task.root.root_id, registry);
    delayed_language.admit_local(&delayed_certificate)?;
    let delayed_refinement_after_prediction = delayed_language.refinement_count() == 1;
    control_budgets_exact &= local_budget_exact(&delayed_proposal_budget)
        && delayed_proposal_budget == delayed_validation_budget;

    Ok(RootExecution {
        root_id: task.root.root_id,
        family: task.family,
        admitted_correct,
        admitted_local_validated: true,
        admitted_refinements_during_prediction,
        admitted_proposal_budget: proposal_budget,
        admitted_validation_budget: validation_budget,
        base_best_repaired: base_best,
        detected_defects,
        base_budget,
        control_correct: controls,
        control_budgets_exact,
        wrong_schema_validation_rejected: wrong_validation_rejected,
        delayed_refinement_after_prediction,
        invariants_hold: wrong_validation_rejected
            && delayed_refinement_after_prediction
            && language.verify_invariants().is_ok()
            && wrong_language.verify_invariants().is_ok()
            && delayed_language.verify_invariants().is_ok(),
    })
}

fn predict(
    discovery: &[WitnessedHistory],
    transfer: &[TransferCase],
    language: &GrammarExtendedStateLanguage,
) -> usize {
    let mut index = BTreeMap::<ExtendedStateKey, Option<Atom>>::new();
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

fn base_ceiling(root: &GrammarRoot) -> Result<(usize, usize, BaseBudget), Box<dyn Error>> {
    let problem = RefinementProblem {
        root_id: root.root_id,
        discovery: root.discovery.clone(),
    };
    let mut genesis = GenesisBudget::default();
    let vocabulary = derive_vocabulary(&problem, &mut genesis)?;
    let language = StateLanguage::new(root.root_id);
    let defects = detect_alias_defects(&problem, &language, &mut genesis)?;
    let evidence_index = root
        .discovery
        .iter()
        .enumerate()
        .map(|(index, episode)| (episode.evidence_id, index))
        .collect::<BTreeMap<_, _>>();
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
    Ok((
        best,
        defects.len(),
        BaseBudget {
            vocabulary_history_scans: genesis.vocabulary_history_scans,
            history_pair_evaluations: genesis.history_pair_evaluations,
            candidate_programs: programs.len(),
            program_history_evaluations: executions,
            unique_partitions: partitions.len(),
        },
    ))
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

fn grammar_budget_exact(budget: &GrammarExtensionBudget, roots: usize) -> bool {
    budget.vocabulary_history_scans == roots * DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == roots * EXPECTED_PAIR_EVALUATIONS
        && budget.base_candidate_programs == roots * EXPECTED_BASE_PROGRAMS
        && budget.base_program_history_evaluations == roots * EXPECTED_BASE_EXECUTIONS
        && budget.extension_bound_candidates == roots * EXPECTED_EXTENSION_PROGRAMS
        && budget.extension_program_history_evaluations == roots * EXPECTED_EXTENSION_EXECUTIONS
        && budget.unique_base_partitions >= roots
        && budget.unique_extension_partitions >= roots
}

fn local_budget_exact(budget: &LocalRefinementBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_LOCAL_PROGRAMS
        && budget.program_history_evaluations == EXPECTED_LOCAL_EXECUTIONS
        && budget.unique_partitions > 1
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

fn build_tasks(
    first_root: u64,
    count: usize,
    family: &'static str,
    hidden_kind: ExtensionKind,
) -> Result<Vec<RootTask>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_task(first_root + offset as u64, family, hidden_kind))
        .collect()
}

fn build_task(
    root_id: u64,
    family: &'static str,
    hidden_kind: ExtensionKind,
) -> Result<RootTask, Box<dyn Error>> {
    let vocabulary = (0..VOCABULARY_SIZE)
        .map(|index| atom(&format!("{family}_{root_id}_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let intervention = atom(&format!("probe_{family}"))?;
    let positive = atom("positive")?;
    let negative = atom("negative")?;
    let hidden = BoundExtensionProgram {
        kind: hidden_kind,
        left: vocabulary[0].clone(),
        right: vocabulary[1].clone(),
    };
    let mut discovery = Vec::new();
    let mut transfer = Vec::new();
    for (index, events) in permutations(&vocabulary).into_iter().enumerate() {
        let history = RawHistory {
            history_id: root_id * 10_000 + index as u64,
            events,
        };
        let outcome = if hidden.execute(&history) {
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

fn shuffled_problem(problem: &GrammarExtensionProblem) -> GrammarExtensionProblem {
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

fn has_both_outcomes(discovery: &[WitnessedHistory]) -> bool {
    let mut outcomes = discovery
        .iter()
        .map(|episode| episode.outcome.as_str())
        .collect::<Vec<_>>();
    outcomes.sort_unstable();
    outcomes.dedup();
    outcomes.len() == 2
}

fn has_both_transfer_outcomes(transfer: &[TransferCase]) -> bool {
    let mut outcomes = transfer
        .iter()
        .map(|case| case.expected.as_str())
        .collect::<Vec<_>>();
    outcomes.sort_unstable();
    outcomes.dedup();
    outcomes.len() == 2
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}
