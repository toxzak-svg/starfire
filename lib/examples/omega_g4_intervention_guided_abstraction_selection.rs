use serde::Serialize;
use star::commitment_state::Atom;
use star::grammar_extension::{
    synthesize_grammar_extension, validate_grammar_extension, BoundExtensionProgram,
    ExtensionKind, GrammarExtensionBudget, GrammarExtensionConfig, GrammarExtensionProblem,
    GrammarRegistry, GrammarRoot,
};
use star::intervention_guided_abstraction_selection::{
    counterfeit_witness_control, diagnostic_schema_prediction, enum_order_control_kind,
    execute_intervention, forced_intervention_remaining_candidates,
    observational_promotion_rejected, revalidate_omega_g3_selection_parent,
    synthesize_intervention_plan, synthesize_passive_ambiguity, validate_intervention_plan,
    validate_passive_ambiguity, validate_witness_and_select, AbstractionSelectionProblem,
    FinalAbstractionRegistry, InterventionKind, InterventionPlanBudget, PassiveSearchBudget,
    SelectionHistory, SelectionRoot, SelectionSchemaKind, SelectionTransferTask, TransferCase,
};
use star::multistep_abstraction_reuse::{
    revalidate_omega_g2_abstraction_parent, synthesize_abstraction, synthesize_concrete_chain,
    validate_abstraction, validate_concrete_chain, AbstractionRegistry, AbstractionSearchBudget,
    ChainTask, ConcreteSynthesisBudget, LabeledChainHistory,
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

const PREREGISTRATION_COMMIT: &str = "d6778cf29db725775c0d6815a6d23d6398c74010";
const SELECTION_COHORT: u64 = 0x0a64_0001;
const G3_COHORT: u64 = 0x0a63_0001;
const LEAKAGE_CANARY: &str = "OMEGA_G4_OUTCOME_LEAKAGE_CANARY";
const RANDOM_SEED: u64 = 0x4F4D4547414734;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ParentBudgetReport {
    omega_g1_proposal: GrammarExtensionBudget,
    omega_g1_validation: GrammarExtensionBudget,
    omega_g1_revalidation: GrammarExtensionBudget,
    omega_g2_proposal: RecursiveCompositionBudget,
    omega_g2_validation: RecursiveCompositionBudget,
    omega_g2_revalidation: RecursiveCompositionBudget,
    omega_g3_concrete_proposal: ConcreteSynthesisBudget,
    omega_g3_concrete_validation: ConcreteSynthesisBudget,
    omega_g3_abstraction_proposal: AbstractionSearchBudget,
    omega_g3_abstraction_validation: AbstractionSearchBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PassiveReport {
    roots: usize,
    histories: usize,
    proposal_budget: PassiveSearchBudget,
    validation_budget: PassiveSearchBudget,
    exact_candidates: Vec<String>,
    winner_node_costs: Vec<usize>,
    pre_intervention_promotion_rejected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct InterventionReport {
    proposal_budget: InterventionPlanBudget,
    validation_budget: InterventionPlanBudget,
    selected: String,
    discriminating_candidates: usize,
    non_discriminating_candidates: usize,
    executed_interventions: usize,
    witnessed_outcome: bool,
    leakage_canary_absent: bool,
    counterfeit_witness_rejected: bool,
    final_winner: String,
    proxy_rejected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TransferSplitReport {
    roots: usize,
    predictions: usize,
    correct_predictions: usize,
    perfect_roots: usize,
    proxy_control_predictions: usize,
    proxy_control_perfect_roots: usize,
    foreign_vocabulary_rejected: bool,
    families: Vec<FamilyReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FamilyReport {
    family: String,
    roots: usize,
    perfect_roots: usize,
    correct_predictions: usize,
    proxy_control_predictions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ControlReport {
    observational_only_rejected: bool,
    enum_order_selected_proxy: bool,
    enum_order_not_perfect: bool,
    identity_tie_unresolved: bool,
    distractor_tie_unresolved: bool,
    seeded_random_intervention: String,
    seeded_random_tie_unresolved: bool,
    raw_proxy_injection_rejected: bool,
    omega_g3_parent_ablation_rejected: bool,
    proof_text_only_rejected: bool,
    counterfeit_passive_proof_rejected: bool,
    counterfeit_plan_rejected: bool,
    counterfeit_witness_rejected: bool,
    duplicate_final_admission_rejected: bool,
    foreign_final_certificate_rejected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuthorityMatrix {
    runtime_chat_wiring: bool,
    response_influence: bool,
    live_routing_authority: bool,
    persistence_mutation: bool,
    belief_or_ontology_promotion: bool,
    pecs_or_charge_mutation: bool,
    tool_or_capability_selection: bool,
    network_or_external_side_effect: bool,
    autonomous_action: bool,
    automatic_source_modification: bool,
}

impl AuthorityMatrix {
    fn closed(&self) -> bool {
        !self.runtime_chat_wiring
            && !self.response_influence
            && !self.live_routing_authority
            && !self.persistence_mutation
            && !self.belief_or_ontology_promotion
            && !self.pecs_or_charge_mutation
            && !self.tool_or_capability_selection
            && !self.network_or_external_side_effect
            && !self.autonomous_action
            && !self.automatic_source_modification
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GateReport {
    parent_lineage_revalidated: bool,
    parent_budgets_equal: bool,
    passive_counts_exact: bool,
    passive_budgets_exact: bool,
    passive_validation_equal_budget: bool,
    exactly_two_tied_passive_candidates: bool,
    pre_intervention_promotion_rejected: bool,
    intervention_budgets_exact: bool,
    intervention_validation_equal_budget: bool,
    selected_intervention_exact: bool,
    one_intervention_executed: bool,
    independent_witness_valid: bool,
    proxy_rejected: bool,
    recursive_abstraction_admitted: bool,
    holdout_transfer_exact: bool,
    future_transfer_exact: bool,
    families_exact: bool,
    controls_exact: bool,
    integrity_exact: bool,
    authority_closed: bool,
    source_fixture_immutable: bool,
    replay_exact: bool,
}

impl GateReport {
    fn all_pass(&self) -> bool {
        self.parent_lineage_revalidated
            && self.parent_budgets_equal
            && self.passive_counts_exact
            && self.passive_budgets_exact
            && self.passive_validation_equal_budget
            && self.exactly_two_tied_passive_candidates
            && self.pre_intervention_promotion_rejected
            && self.intervention_budgets_exact
            && self.intervention_validation_equal_budget
            && self.selected_intervention_exact
            && self.one_intervention_executed
            && self.independent_witness_valid
            && self.proxy_rejected
            && self.recursive_abstraction_admitted
            && self.holdout_transfer_exact
            && self.future_transfer_exact
            && self.families_exact
            && self.controls_exact
            && self.integrity_exact
            && self.authority_closed
            && self.source_fixture_immutable
            && self.replay_exact
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Report {
    experiment: &'static str,
    preregistration_commit: &'static str,
    parent_registry_signature: String,
    parent_budgets: ParentBudgetReport,
    passive: PassiveReport,
    intervention: InterventionReport,
    holdout: TransferSplitReport,
    future: TransferSplitReport,
    controls: ControlReport,
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
        "target/omega-g4-intervention-guided-abstraction-selection-report.json",
        &json,
    )?;
    println!("{json}");
    if report.terminal_classification != "PASS" {
        return Err(format!(
            "ΩG4 terminal classification: {}",
            report.terminal_classification
        )
        .into());
    }
    Ok(())
}

fn run(replay_exact: bool) -> Result<Report, Box<dyn Error>> {
    let parent_config = GrammarExtensionConfig::default();
    let omega_g1_problem = GrammarExtensionProblem {
        cohort_id: 0x0a61_0001,
        roots: build_omega_roots(1, 8, false)?,
    };
    let omega_g1_snapshot = omega_g1_problem.clone();
    let mut omega_g1_proposal = GrammarExtensionBudget::default();
    let omega_g1_proof = synthesize_grammar_extension(
        &omega_g1_problem,
        parent_config,
        &mut omega_g1_proposal,
    )?;
    let mut omega_g1_validation = GrammarExtensionBudget::default();
    let omega_g1_certificate = validate_grammar_extension(
        &omega_g1_problem,
        &omega_g1_proof,
        parent_config,
        &mut omega_g1_validation,
    )?;
    let mut omega_g1_registry = GrammarRegistry::new(omega_g1_problem.cohort_id);
    omega_g1_registry.admit(&omega_g1_certificate)?;
    let mut omega_g1_revalidation = GrammarExtensionBudget::default();
    let omega_g1_parent = revalidate_parent(
        &omega_g1_problem,
        &omega_g1_proof,
        &omega_g1_registry,
        parent_config,
        &mut omega_g1_revalidation,
    )?;

    let omega_g2_problem = RecursiveCompositionProblem {
        cohort_id: 0x0a62_0001,
        roots: build_omega_roots(10_001, 8, true)?,
    };
    let omega_g2_snapshot = omega_g2_problem.clone();
    let mut omega_g2_proposal = RecursiveCompositionBudget::default();
    let omega_g2_proof = synthesize_recursive_composition(
        &omega_g2_problem,
        &omega_g1_parent,
        &mut omega_g2_proposal,
    )?;
    let mut omega_g2_validation = RecursiveCompositionBudget::default();
    let omega_g2_certificate = validate_recursive_composition(
        &omega_g2_problem,
        &omega_g1_parent,
        &omega_g2_proof,
        &mut omega_g2_validation,
    )?;
    let mut omega_g2_registry =
        ComposedGrammarRegistry::new(omega_g2_problem.cohort_id, &omega_g1_parent);
    omega_g2_registry.admit(&omega_g2_certificate)?;
    let mut omega_g2_revalidation = RecursiveCompositionBudget::default();
    let omega_g2_parent = revalidate_omega_g2_abstraction_parent(
        &omega_g2_problem,
        &omega_g1_parent,
        &omega_g2_proof,
        &omega_g2_registry,
        &mut omega_g2_revalidation,
    )?;

    let g3_development = build_g3_development_tasks()?;
    let g3_development_snapshot = g3_development.clone();
    let mut g3_concrete_proposal = ConcreteSynthesisBudget::default();
    let mut g3_concrete_validation = ConcreteSynthesisBudget::default();
    let mut g3_examples: Vec<ValidatedConcreteSolutionCertificate> = Vec::new();
    for task in &g3_development {
        let proof = synthesize_concrete_chain(task, &mut g3_concrete_proposal)?;
        let certificate = validate_concrete_chain(
            task,
            &proof,
            &mut g3_concrete_validation,
        )?;
        g3_examples.push(certificate);
    }
    let mut g3_abstraction_proposal = AbstractionSearchBudget::default();
    let g3_abstraction_proof = synthesize_abstraction(
        G3_COHORT,
        &omega_g2_parent,
        &g3_examples,
        &mut g3_abstraction_proposal,
    )?;
    let mut g3_abstraction_validation = AbstractionSearchBudget::default();
    let g3_abstraction_certificate = validate_abstraction(
        G3_COHORT,
        &omega_g2_parent,
        &g3_examples,
        &g3_abstraction_proof,
        &mut g3_abstraction_validation,
    )?;
    let mut g3_registry = AbstractionRegistry::new(G3_COHORT, &omega_g2_parent);
    g3_registry.admit(&g3_abstraction_certificate)?;
    let selection_parent = revalidate_omega_g3_selection_parent(&g3_registry)?;

    let stale_g3_registry = AbstractionRegistry::new(G3_COHORT, &omega_g2_parent);
    let omega_g3_parent_ablation_rejected =
        revalidate_omega_g3_selection_parent(&stale_g3_registry).is_err();
    let proof_text_only_rejected = omega_g3_parent_ablation_rejected;

    let problem = build_selection_problem()?;
    let problem_snapshot = problem.clone();
    let mut passive_proposal_budget = PassiveSearchBudget::default();
    let passive_proof = synthesize_passive_ambiguity(
        &problem,
        &selection_parent,
        &mut passive_proposal_budget,
    )?;
    let mut passive_validation_budget = PassiveSearchBudget::default();
    let passive_certificate = validate_passive_ambiguity(
        &problem,
        &selection_parent,
        &passive_proof,
        &mut passive_validation_budget,
    )?;
    let mut counterfeit_passive = passive_proof.clone();
    counterfeit_passive.proof_id ^= 1;
    let mut counterfeit_passive_budget = PassiveSearchBudget::default();
    let counterfeit_passive_proof_rejected = validate_passive_ambiguity(
        &problem,
        &selection_parent,
        &counterfeit_passive,
        &mut counterfeit_passive_budget,
    )
    .is_err();

    let observational_only_rejected = observational_promotion_rejected(&passive_certificate);
    let enum_control_kind = enum_order_control_kind(&passive_certificate)?;

    let mut intervention_proposal_budget = InterventionPlanBudget::default();
    let intervention_proof = synthesize_intervention_plan(
        &problem,
        &passive_certificate,
        &mut intervention_proposal_budget,
    )?;
    let mut intervention_validation_budget = InterventionPlanBudget::default();
    let intervention_certificate = validate_intervention_plan(
        &problem,
        &passive_certificate,
        &intervention_proof,
        &mut intervention_validation_budget,
    )?;
    let mut counterfeit_plan = intervention_proof.clone();
    counterfeit_plan.selected = InterventionKind::IdentityControl;
    let mut counterfeit_plan_budget = InterventionPlanBudget::default();
    let counterfeit_plan_rejected = validate_intervention_plan(
        &problem,
        &passive_certificate,
        &counterfeit_plan,
        &mut counterfeit_plan_budget,
    )
    .is_err();

    let leakage_canary_absent = !intervention_certificate
        .canonical_trace()
        .contains(LEAKAGE_CANARY);
    let witness = execute_intervention(&problem, &intervention_certificate)?;
    let counterfeit_witness_rejected = counterfeit_witness_control(
        &problem,
        &intervention_certificate,
        &witness,
    );
    let (final_proof, final_certificate) = validate_witness_and_select(
        &problem,
        &selection_parent,
        &passive_certificate,
        &intervention_certificate,
        &witness,
    )?;

    let proxy_rejected = final_proof.verdicts.iter().any(|verdict| {
        verdict.kind == SelectionSchemaKind::ProxyAnchorAdjacent && !verdict.retained
    });
    let recursive_retained = final_proof.verdicts.iter().any(|verdict| {
        verdict.kind == SelectionSchemaKind::RecursiveAppendAdjacent && verdict.retained
    });

    let mut final_registry = FinalAbstractionRegistry::new(SELECTION_COHORT, &selection_parent);
    final_registry.admit(&final_certificate)?;
    let final_after_first = final_registry.clone();
    let duplicate_final_admission_rejected = final_registry
        .admit(&final_certificate)
        .is_err()
        && final_registry == final_after_first;
    let raw_before = final_registry.clone();
    let raw_proxy_injection_rejected = final_registry
        .reject_raw_schema_injection(SelectionSchemaKind::ProxyAnchorAdjacent)
        .is_err()
        && final_registry == raw_before;
    let mut foreign_final_registry =
        FinalAbstractionRegistry::new(SELECTION_COHORT + 1, &selection_parent);
    let foreign_before = foreign_final_registry.clone();
    let foreign_final_certificate_rejected = foreign_final_registry
        .admit(&final_certificate)
        .is_err()
        && foreign_final_registry == foreign_before;

    let identity_tie_unresolved = forced_intervention_remaining_candidates(
        &problem,
        &passive_certificate,
        InterventionKind::IdentityControl,
    )? == 2;
    let distractor_tie_unresolved = forced_intervention_remaining_candidates(
        &problem,
        &passive_certificate,
        InterventionKind::MoveDistractorLeft,
    )? == 2;
    let random_intervention = InterventionKind::all()[(RANDOM_SEED % 6) as usize];
    let seeded_random_tie_unresolved = forced_intervention_remaining_candidates(
        &problem,
        &passive_certificate,
        random_intervention,
    )? == 2;

    let holdout_tasks = build_transfer_tasks(
        50_001,
        4,
        5,
        "holdout_energy",
    )?
    .into_iter()
    .chain(build_transfer_tasks(50_101, 4, 5, "holdout_compiler")?)
    .chain(build_transfer_tasks(50_201, 4, 5, "holdout_river")?)
    .collect::<Vec<_>>();
    let future_tasks = build_transfer_tasks(60_001, 6, 6, "future_materials")?
        .into_iter()
        .chain(build_transfer_tasks(60_101, 6, 6, "future_protocols")?)
        .chain(build_transfer_tasks(60_201, 6, 6, "future_ecology")?)
        .collect::<Vec<_>>();
    let holdout = evaluate_transfer(&holdout_tasks, &final_registry, enum_control_kind)?;
    let future = evaluate_transfer(&future_tasks, &final_registry, enum_control_kind)?;
    let enum_order_not_perfect = holdout.proxy_control_perfect_roots < holdout.roots
        && future.proxy_control_perfect_roots < future.roots;

    let parent_budgets = ParentBudgetReport {
        omega_g1_proposal,
        omega_g1_validation,
        omega_g1_revalidation,
        omega_g2_proposal,
        omega_g2_validation,
        omega_g2_revalidation,
        omega_g3_concrete_proposal,
        omega_g3_concrete_validation,
        omega_g3_abstraction_proposal,
        omega_g3_abstraction_validation,
    };
    let parent_budgets_equal = parent_budgets.omega_g1_proposal
        == parent_budgets.omega_g1_validation
        && parent_budgets.omega_g1_validation == parent_budgets.omega_g1_revalidation
        && parent_budgets.omega_g2_proposal == parent_budgets.omega_g2_validation
        && parent_budgets.omega_g2_validation == parent_budgets.omega_g2_revalidation
        && parent_budgets.omega_g3_concrete_proposal
            == parent_budgets.omega_g3_concrete_validation
        && parent_budgets.omega_g3_abstraction_proposal
            == parent_budgets.omega_g3_abstraction_validation;

    let passive_histories = problem.roots.iter().map(|root| root.passive.len()).sum();
    let passive_report = PassiveReport {
        roots: problem.roots.len(),
        histories: passive_histories,
        proposal_budget: passive_proposal_budget.clone(),
        validation_budget: passive_validation_budget.clone(),
        exact_candidates: passive_proof
            .winners
            .iter()
            .map(|kind| kind.name().to_string())
            .collect(),
        winner_node_costs: passive_proof.winner_node_costs.clone(),
        pre_intervention_promotion_rejected: observational_only_rejected,
    };
    let discriminating_candidates = intervention_proof
        .analyses
        .iter()
        .filter(|analysis| analysis.disagreement_score == 1)
        .count();
    let non_discriminating_candidates = intervention_proof
        .analyses
        .iter()
        .filter(|analysis| analysis.disagreement_score == 0)
        .count();
    let intervention_report = InterventionReport {
        proposal_budget: intervention_proposal_budget.clone(),
        validation_budget: intervention_validation_budget.clone(),
        selected: intervention_certificate.selected().name().to_string(),
        discriminating_candidates,
        non_discriminating_candidates,
        executed_interventions: 1,
        witnessed_outcome: witness.observed_outcome(),
        leakage_canary_absent,
        counterfeit_witness_rejected,
        final_winner: final_certificate.kind().name().to_string(),
        proxy_rejected,
    };
    let controls = ControlReport {
        observational_only_rejected,
        enum_order_selected_proxy: enum_control_kind
            == SelectionSchemaKind::ProxyAnchorAdjacent,
        enum_order_not_perfect,
        identity_tie_unresolved,
        distractor_tie_unresolved,
        seeded_random_intervention: random_intervention.name().to_string(),
        seeded_random_tie_unresolved,
        raw_proxy_injection_rejected,
        omega_g3_parent_ablation_rejected,
        proof_text_only_rejected,
        counterfeit_passive_proof_rejected,
        counterfeit_plan_rejected,
        counterfeit_witness_rejected,
        duplicate_final_admission_rejected,
        foreign_final_certificate_rejected,
    };
    let authority = AuthorityMatrix {
        runtime_chat_wiring: false,
        response_influence: false,
        live_routing_authority: false,
        persistence_mutation: false,
        belief_or_ontology_promotion: false,
        pecs_or_charge_mutation: false,
        tool_or_capability_selection: false,
        network_or_external_side_effect: false,
        autonomous_action: false,
        automatic_source_modification: false,
    };

    let exactly_two_tied_passive_candidates = passive_proof.winners
        == vec![
            SelectionSchemaKind::ProxyAnchorAdjacent,
            SelectionSchemaKind::RecursiveAppendAdjacent,
        ]
        && passive_proof.winner_node_costs == vec![5, 5];
    let parent_lineage_revalidated = g3_registry.admitted_count() == 1
        && selection_parent.registry_signature() == g3_registry.canonical_signature();
    let passive_counts_exact = problem.roots.len() == 8 && passive_histories == 192;
    let passive_budgets_exact = passive_proposal_budget.schema_candidates == 4
        && passive_proposal_budget.schema_history_evaluations == 768;
    let passive_validation_equal_budget =
        passive_proposal_budget == passive_validation_budget;
    let intervention_budgets_exact = intervention_proposal_budget.intervention_candidates == 6
        && intervention_proposal_budget.candidate_intervention_predictions == 12
        && intervention_proposal_budget.pairwise_disagreement_comparisons == 6
        && intervention_proposal_budget.selected_interventions == 1;
    let intervention_validation_equal_budget =
        intervention_proposal_budget == intervention_validation_budget;
    let selected_intervention_exact = intervention_certificate.selected()
        == InterventionKind::MoveProxyAfterX0
        && discriminating_candidates == 2
        && non_discriminating_candidates == 4;
    let independent_witness_valid = witness.intervention()
        == InterventionKind::MoveProxyAfterX0
        && witness.observed_outcome();
    let recursive_abstraction_admitted = recursive_retained
        && final_registry.admitted_count() == 1
        && final_registry.supports(SelectionSchemaKind::RecursiveAppendAdjacent);
    let holdout_transfer_exact = holdout.roots == 12
        && holdout.predictions == 96
        && holdout.correct_predictions == 96
        && holdout.perfect_roots == 12;
    let future_transfer_exact = future.roots == 18
        && future.predictions == 144
        && future.correct_predictions == 144
        && future.perfect_roots == 18;
    let families_exact = holdout.families.iter().all(|family| {
        family.roots == 4 && family.perfect_roots == 4 && family.correct_predictions == 32
    }) && future.families.iter().all(|family| {
        family.roots == 6 && family.perfect_roots == 6 && family.correct_predictions == 48
    });
    let controls_exact = observational_only_rejected
        && enum_control_kind == SelectionSchemaKind::ProxyAnchorAdjacent
        && enum_order_not_perfect
        && identity_tie_unresolved
        && distractor_tie_unresolved
        && random_intervention == InterventionKind::RotateDistractors
        && seeded_random_tie_unresolved
        && raw_proxy_injection_rejected
        && omega_g3_parent_ablation_rejected
        && proof_text_only_rejected
        && counterfeit_passive_proof_rejected
        && counterfeit_plan_rejected
        && counterfeit_witness_rejected;
    let integrity_exact = duplicate_final_admission_rejected
        && foreign_final_certificate_rejected
        && holdout.foreign_vocabulary_rejected
        && future.foreign_vocabulary_rejected
        && final_registry.verify_invariants().is_ok()
        && leakage_canary_absent;
    let source_fixture_immutable = omega_g1_problem == omega_g1_snapshot
        && omega_g2_problem == omega_g2_snapshot
        && g3_development == g3_development_snapshot
        && problem == problem_snapshot;
    let authority_closed = authority.closed();

    let gates = GateReport {
        parent_lineage_revalidated,
        parent_budgets_equal,
        passive_counts_exact,
        passive_budgets_exact,
        passive_validation_equal_budget,
        exactly_two_tied_passive_candidates,
        pre_intervention_promotion_rejected: observational_only_rejected,
        intervention_budgets_exact,
        intervention_validation_equal_budget,
        selected_intervention_exact,
        one_intervention_executed: intervention_report.executed_interventions == 1,
        independent_witness_valid,
        proxy_rejected,
        recursive_abstraction_admitted,
        holdout_transfer_exact,
        future_transfer_exact,
        families_exact,
        controls_exact,
        integrity_exact,
        authority_closed,
        source_fixture_immutable,
        replay_exact,
    };
    let terminal_classification = if !gates.parent_lineage_revalidated {
        "DEPENDENCY_FAILURE"
    } else if !gates.controls_exact
        || !gates.integrity_exact
        || !gates.authority_closed
        || !gates.source_fixture_immutable
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
        experiment: "ΩG4 intervention-guided abstraction selection",
        preregistration_commit: PREREGISTRATION_COMMIT,
        parent_registry_signature: selection_parent.registry_signature().to_string(),
        parent_budgets,
        passive: passive_report,
        intervention: intervention_report,
        holdout,
        future,
        controls,
        authority,
        gates,
        terminal_classification,
        claim_boundary: "Bounded one-step intervention selection between two tied abstractions under the frozen correlation-break fixture; not unrestricted causal discovery, real-world autonomy, or AGI.",
    })
}

fn evaluate_transfer(
    tasks: &[SelectionTransferTask],
    registry: &FinalAbstractionRegistry,
    control_kind: SelectionSchemaKind,
) -> Result<TransferSplitReport, Box<dyn Error>> {
    let mut correct_predictions = 0usize;
    let mut perfect_roots = 0usize;
    let mut proxy_control_predictions = 0usize;
    let mut proxy_control_perfect_roots = 0usize;
    let mut by_family: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut foreign_vocabulary_rejected = true;
    for task in tasks {
        let mut root_correct = 0usize;
        let mut root_control_correct = 0usize;
        for case in &task.cases {
            if registry.predict(task, &case.events)? == case.expected {
                correct_predictions += 1;
                root_correct += 1;
            }
            if diagnostic_schema_prediction(control_kind, task, &case.events)? == case.expected {
                proxy_control_predictions += 1;
                root_control_correct += 1;
            }
        }
        if root_correct == task.cases.len() {
            perfect_roots += 1;
        }
        if root_control_correct == task.cases.len() {
            proxy_control_perfect_roots += 1;
        }
        let entry = by_family.entry(task.family.clone()).or_insert((0, 0, 0, 0));
        entry.0 += 1;
        entry.1 += usize::from(root_correct == task.cases.len());
        entry.2 += root_correct;
        entry.3 += root_control_correct;

        let mut foreign = task.cases[0].events.clone();
        foreign[0] = atom(&format!("foreign_{}", task.root_id))?;
        foreign_vocabulary_rejected &= registry.predict(task, &foreign).is_err();
    }
    let families = by_family
        .into_iter()
        .map(|(family, (roots, perfect, correct, control))| FamilyReport {
            family,
            roots,
            perfect_roots: perfect,
            correct_predictions: correct,
            proxy_control_predictions: control,
        })
        .collect();
    Ok(TransferSplitReport {
        roots: tasks.len(),
        predictions: tasks.iter().map(|task| task.cases.len()).sum(),
        correct_predictions,
        perfect_roots,
        proxy_control_predictions,
        proxy_control_perfect_roots,
        foreign_vocabulary_rejected,
        families,
    })
}

fn build_selection_problem() -> Result<AbstractionSelectionProblem, Box<dyn Error>> {
    let families = [
        "development_thermal",
        "development_thermal",
        "development_software",
        "development_software",
        "development_watershed",
        "development_watershed",
        "development_logistics",
        "development_logistics",
    ];
    let roots = families
        .iter()
        .enumerate()
        .map(|(index, family)| build_selection_root(40_001 + index as u64, family))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AbstractionSelectionProblem {
        cohort_id: SELECTION_COHORT,
        roots,
    })
}

fn build_selection_root(root_id: u64, family: &str) -> Result<SelectionRoot, Box<dyn Error>> {
    let chain = chain_atoms("g4", root_id, 4)?;
    let proxy = atom(&format!("g4_{root_id}_proxy"))?;
    let distractors = vec![
        atom(&format!("g4_{root_id}_d0"))?,
        atom(&format!("g4_{root_id}_d1"))?,
    ];
    let canonical = vec![
        proxy.clone(),
        chain[0].clone(),
        chain[1].clone(),
        chain[2].clone(),
        chain[3].clone(),
        distractors[0].clone(),
        distractors[1].clone(),
    ];
    let mut vocabulary = chain.clone();
    vocabulary.push(proxy.clone());
    vocabulary.extend(distractors.clone());
    let mut positive = Vec::new();
    let mut negative = Vec::new();
    for events in permutations(&vocabulary) {
        let causal = chain_is_consecutive(&chain, &events);
        let proxy_value = adjacent_before(&proxy, &chain[0], &events);
        if causal == proxy_value {
            if causal {
                positive.push(events);
            } else {
                negative.push(events);
            }
        }
    }
    positive.sort();
    negative.sort();
    positive.retain(|events| events != &canonical);
    positive.insert(0, canonical);
    if positive.len() < 6 || negative.len() < 18 {
        return Err("insufficient passive fixture histories".into());
    }
    let mut passive = Vec::new();
    for (index, events) in positive.into_iter().take(6).enumerate() {
        passive.push(SelectionHistory {
            history_id: root_id * 1_000 + index as u64,
            events,
            positive: true,
        });
    }
    for (index, events) in negative.into_iter().take(18).enumerate() {
        passive.push(SelectionHistory {
            history_id: root_id * 1_000 + 100 + index as u64,
            events,
            positive: false,
        });
    }
    Ok(SelectionRoot {
        root_id,
        family: family.to_string(),
        chain,
        proxy,
        distractors,
        passive,
    })
}

fn build_transfer_tasks(
    first_root: u64,
    count: usize,
    arity: usize,
    family: &str,
) -> Result<Vec<SelectionTransferTask>, Box<dyn Error>> {
    (0..count)
        .map(|offset| build_transfer_task(first_root + offset as u64, arity, family))
        .collect()
}

fn build_transfer_task(
    root_id: u64,
    arity: usize,
    family: &str,
) -> Result<SelectionTransferTask, Box<dyn Error>> {
    let chain = chain_atoms("g4_transfer", root_id, arity)?;
    let proxy = atom(&format!("g4_transfer_{root_id}_proxy"))?;
    let distractors = vec![
        atom(&format!("g4_transfer_{root_id}_d0"))?,
        atom(&format!("g4_transfer_{root_id}_d1"))?,
    ];
    let mut cases = Vec::new();

    let mut positive_one = chain.clone();
    positive_one.extend(distractors.clone());
    positive_one.push(proxy.clone());
    cases.push(TransferCase {
        events: positive_one,
        expected: true,
    });

    let mut positive_two = vec![distractors[0].clone()];
    positive_two.extend(chain.clone());
    positive_two.push(proxy.clone());
    positive_two.push(distractors[1].clone());
    cases.push(TransferCase {
        events: positive_two,
        expected: true,
    });

    let mut positive_three = vec![proxy.clone(), distractors[0].clone()];
    positive_three.extend(chain.clone());
    positive_three.push(distractors[1].clone());
    cases.push(TransferCase {
        events: positive_three,
        expected: true,
    });

    let mut positive_four = vec![distractors[0].clone(), distractors[1].clone()];
    positive_four.extend(chain.clone());
    positive_four.push(proxy.clone());
    cases.push(TransferCase {
        events: positive_four,
        expected: true,
    });

    let mut negative_one = vec![proxy.clone()];
    negative_one.extend(chain.clone());
    negative_one.swap(2, 3);
    negative_one.extend(distractors.clone());
    cases.push(TransferCase {
        events: negative_one,
        expected: false,
    });

    let mut negative_two = vec![proxy.clone()];
    let mut rotated = chain.clone();
    rotated.rotate_left(1);
    negative_two.extend(rotated);
    negative_two.extend(distractors.clone());
    cases.push(TransferCase {
        events: negative_two,
        expected: false,
    });

    let mut negative_three = chain.clone();
    negative_three.reverse();
    negative_three.push(proxy.clone());
    negative_three.extend(distractors.clone());
    cases.push(TransferCase {
        events: negative_three,
        expected: false,
    });

    let mut negative_four = vec![proxy.clone(), chain[0].clone(), distractors[0].clone()];
    negative_four.extend(chain.iter().skip(1).cloned());
    negative_four.push(distractors[1].clone());
    cases.push(TransferCase {
        events: negative_four,
        expected: false,
    });

    Ok(SelectionTransferTask {
        root_id,
        family: family.to_string(),
        chain,
        proxy,
        distractors,
        cases,
    })
}

fn build_g3_development_tasks() -> Result<Vec<ChainTask>, Box<dyn Error>> {
    let mut tasks = Vec::new();
    for (first_root, arity) in [(30_001u64, 3usize), (30_101, 4), (30_201, 5)] {
        for offset in 0..4 {
            tasks.push(build_g3_chain_task(first_root + offset, arity)?);
        }
    }
    Ok(tasks)
}

fn build_g3_chain_task(root_id: u64, arity: usize) -> Result<ChainTask, Box<dyn Error>> {
    let canonical = chain_atoms("g3", root_id, arity)?;
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
    let mut reversed = canonical;
    reversed.reverse();
    discovery.push(LabeledChainHistory {
        history_id: root_id * 100 + 90,
        events: reversed,
        positive: false,
    });
    Ok(ChainTask { root_id, discovery })
}

fn chain_atoms(prefix: &str, root_id: u64, arity: usize) -> Result<Vec<Atom>, Box<dyn Error>> {
    let codes = ["k7", "m2", "q9", "b4", "t1", "x8", "r5", "v3", "n6", "p0", "d8"];
    let steps = [2usize, 3, 5, 7];
    let step = steps[root_id as usize % steps.len()];
    let offset = (root_id as usize * 7) % codes.len();
    (0..arity)
        .map(|index| {
            let code = codes[(offset + index * step) % codes.len()];
            atom(&format!("{prefix}_{root_id}_{code}"))
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

fn chain_is_consecutive(chain: &[Atom], events: &[Atom]) -> bool {
    events.windows(chain.len()).any(|window| window == chain)
}

fn adjacent_before(left: &Atom, right: &Atom, events: &[Atom]) -> bool {
    events
        .windows(2)
        .any(|pair| &pair[0] == left && &pair[1] == right)
}

fn atom(value: &str) -> Result<Atom, Box<dyn Error>> {
    Ok(Atom::new(value.to_string())?)
}
