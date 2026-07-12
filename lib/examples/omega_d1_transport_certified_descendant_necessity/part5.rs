fn metrics_from_entries(entries: &[(Execution, bool)]) -> PathMetrics {
    let roots = entries.len();
    let full_successes = entries
        .iter()
        .filter(|(execution, _)| execution.full_success)
        .count();
    PathMetrics {
        roots,
        full_successes,
        success_rate: full_successes as f64 / roots.max(1) as f64,
        total_correct_predictions: entries
            .iter()
            .map(|(execution, _)| execution.correct_predictions)
            .sum(),
        correct_ancestor_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.correct_ancestor_admitted)
            .count(),
        wrong_ancestor_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.wrong_ancestor_admitted)
            .count(),
        stationary_ancestor_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.stationary_ancestor_admitted)
            .count(),
        descendant_proposal_successes: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_proposal_succeeded)
            .count(),
        descendant_validation_successes: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_validation_succeeded)
            .count(),
        descendant_validation_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_validation_rejected)
            .count(),
        descendant_admissions_during_window: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_admitted_during_window)
            .count(),
        descendant_final_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_admitted_final)
            .count(),
        no_ancestor_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.no_ancestor_rejection)
            .count(),
        raw_audit_complete_repairs: entries
            .iter()
            .filter(|(execution, _)| execution.raw_audit_complete_repair)
            .count(),
        ancestor_payload_preservations: entries
            .iter()
            .filter(|(execution, _)| execution.ancestor_payload_preserved)
            .count(),
        descendant_payload_preservations: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_payload_preserved)
            .count(),
        stable_correct_ancestor_selections: entries
            .iter()
            .filter(|(execution, _)| {
                execution
                    .selected_ancestor_program
                    .as_deref()
                    .map(is_stable_correct_ancestor_program)
                    .unwrap_or(false)
            })
            .count(),
        stable_wrong_ancestor_selections: entries
            .iter()
            .filter(|(execution, _)| {
                execution
                    .selected_ancestor_program
                    .as_deref()
                    .map(is_stable_wrong_ancestor_program)
                    .unwrap_or(false)
            })
            .count(),
        stationary_accidental_ancestor_selections: entries
            .iter()
            .filter(|(execution, _)| {
                execution
                    .selected_ancestor_program
                    .as_deref()
                    .map(is_stationary_accidental_program)
                    .unwrap_or(false)
            })
            .count(),
        budgets_exact: entries
            .iter()
            .all(|(execution, _)| execution.budget_exact),
        replay_exact: entries.iter().all(|(_, replay)| *replay),
        invariants_hold: entries
            .iter()
            .all(|(execution, _)| execution.invariants_hold),
    }
}

fn is_pair_program(program: &str, left_suffix: &str, right_suffix: &str) -> bool {
    let Some(rest) = program.strip_prefix("first(") else {
        return false;
    };
    let Some((left, right_with_suffix)) = rest.split_once(")<first(") else {
        return false;
    };
    let Some(right) = right_with_suffix.strip_suffix(')') else {
        return false;
    };
    left.ends_with(left_suffix)
        && right.ends_with(right_suffix)
        && left.strip_suffix(left_suffix) == right.strip_suffix(right_suffix)
}

fn is_stable_correct_ancestor_program(program: &str) -> bool {
    is_pair_program(program, "_a", "_b")
}

fn is_stable_wrong_ancestor_program(program: &str) -> bool {
    is_pair_program(program, "_g", "_h")
}

fn is_stationary_accidental_program(program: &str) -> bool {
    let Some(rest) = program.strip_prefix("first(") else {
        return false;
    };
    let Some((left, right_with_suffix)) = rest.split_once(")<count(") else {
        return false;
    };
    let Some(right) = right_with_suffix.strip_suffix(')') else {
        return false;
    };
    left == right && left.ends_with("_a")
}

fn perfect_stateful(report: &SplitReport) -> bool {
    let metrics = path_metrics(report, PathKind::TransportDescendantChain);
    metrics.full_successes == metrics.roots
        && metrics.total_correct_predictions == metrics.roots * PREDICTIONS_PER_ROOT
        && metrics.correct_ancestor_admissions == metrics.roots
        && metrics.descendant_validation_successes == metrics.roots
        && metrics.descendant_admissions_during_window == metrics.roots
        && metrics.stable_correct_ancestor_selections == metrics.roots
}

fn audit_roots(roots: &[RootTask]) -> Result<RootAudit, Box<dyn Error>> {
    let mut audit = RootAudit {
        roots: roots.len(),
        correct_ancestor_alias_defects_exact: 0,
        wrong_ancestor_alias_defects_exact: 0,
        correct_transport_frontier_exact: 0,
        wrong_transport_frontier_exact: 0,
        stationary_transport_frontier_exact: 0,
        l0_raw_audit_exact: 0,
        l0_descendant_empty_exact: 0,
        correct_descendant_frontier_exact: 0,
    };

    for root in roots {
        let l0 = StateLanguage::new(root.root_id);
        let mut correct_defect_budget = star::representation_genesis::GenesisBudget::default();
        if detect_alias_defects(
            &root.correct_ancestor_problem,
            &l0,
            &mut correct_defect_budget,
        )
        .map(|defects| {
            defects.len() == EXPECTED_ALIAS_DEFECTS
                && correct_defect_budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        })
        .unwrap_or(false)
        {
            audit.correct_ancestor_alias_defects_exact += 1;
        }

        let mut wrong_defect_budget = star::representation_genesis::GenesisBudget::default();
        if detect_alias_defects(
            &root.wrong_ancestor_problem,
            &l0,
            &mut wrong_defect_budget,
        )
        .map(|defects| {
            defects.len() == EXPECTED_ALIAS_DEFECTS
                && wrong_defect_budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        })
        .unwrap_or(false)
        {
            audit.wrong_ancestor_alias_defects_exact += 1;
        }

        let (correct_proof, correct_certificate, correct_budgets_exact) =
            audit_transport_frontier(&root.correct_ancestor_problem, &correct_suite())?;
        if correct_budgets_exact
            && transport_proof_common_exact(&correct_proof)
            && correct_proof.zero_violation_representatives == EXPECTED_CORRECT_ZERO_VIOLATION
            && correct_proof.minimum_transport_violations == 0
            && correct_proof.selected_transport_violations == 0
            && is_stable_correct_ancestor_program(&correct_proof.program.canonical_string())
        {
            audit.correct_transport_frontier_exact += 1;
        }

        let (wrong_proof, _wrong_certificate, wrong_budgets_exact) =
            audit_transport_frontier(&root.wrong_ancestor_problem, &wrong_suite())?;
        if wrong_budgets_exact
            && transport_proof_common_exact(&wrong_proof)
            && wrong_proof.zero_violation_representatives == EXPECTED_WRONG_ZERO_VIOLATION
            && wrong_proof.minimum_transport_violations == 0
            && wrong_proof.selected_transport_violations == 0
            && is_stable_wrong_ancestor_program(&wrong_proof.program.canonical_string())
        {
            audit.wrong_transport_frontier_exact += 1;
        }

        let (stationary_proof, _stationary_certificate, stationary_budgets_exact) =
            audit_transport_frontier(&root.correct_ancestor_problem, &stationary_suite())?;
        if stationary_budgets_exact
            && transport_proof_common_exact(&stationary_proof)
            && stationary_proof.zero_violation_representatives
                == EXPECTED_STATIONARY_ZERO_VIOLATION
            && stationary_proof.minimum_transport_violations == 0
            && stationary_proof.selected_transport_violations == 0
            && is_stationary_accidental_program(&stationary_proof.program.canonical_string())
        {
            audit.stationary_transport_frontier_exact += 1;
        }

        let mut raw_budget = TransportDescendantBudget::default();
        let raw_audit = audit_raw_expressibility(&root.stage2_problem, &mut raw_budget)?;
        if raw_audit_exact(&raw_audit, &raw_budget) {
            audit.l0_raw_audit_exact += 1;
        }

        let empty_ancestor = TransportStateLanguage::new(root.root_id);
        let mut empty_budget = TransportDescendantBudget::default();
        let empty_result = synthesize_transport_descendant(
            &root.stage2_problem,
            &empty_ancestor,
            TransportDescendantConfig::default(),
            &mut empty_budget,
        );
        if matches!(empty_result, Err(TransportDescendantError::NoAncestorRefinement))
            && no_ancestor_budget_exact(&empty_budget)
        {
            audit.l0_descendant_empty_exact += 1;
        }

        let mut correct_ancestor = TransportStateLanguage::new(root.root_id);
        correct_ancestor.admit_certificate(&correct_certificate)?;
        let mut descendant_budget = TransportDescendantBudget::default();
        let descendant_proof = synthesize_transport_descendant(
            &root.stage2_problem,
            &correct_ancestor,
            TransportDescendantConfig::default(),
            &mut descendant_budget,
        )?;
        if descendant_proof_exact(&descendant_proof)
            && descendant_budget_exact(&descendant_budget)
        {
            audit.correct_descendant_frontier_exact += 1;
        }
    }

    Ok(audit)
}

fn audit_transport_frontier(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
) -> Result<(TransportProof, ValidatedTransportCertificate, bool), Box<dyn Error>> {
    let mut proposal_budget = TransportBudget::default();
    let proof = synthesize_transport_refinement(
        problem,
        suite,
        TransportConfig::default(),
        &mut proposal_budget,
    )?;
    let mut validation_budget = TransportBudget::default();
    let certificate = validate_transport_refinement(
        problem,
        suite,
        &proof,
        TransportConfig::default(),
        &mut validation_budget,
    )?;
    Ok((
        proof,
        certificate,
        transport_budget_exact(&proposal_budget) && transport_budget_exact(&validation_budget),
    ))
}

fn transport_proof_common_exact(proof: &TransportProof) -> bool {
    proof.candidate_program_count == EXPECTED_RAW_CANDIDATES
        && proof.unique_partition_count == EXPECTED_RAW_UNIQUE_PARTITIONS
        && proof.opposite_outcome_pairs == EXPECTED_ALIAS_DEFECTS
        && proof.repaired_pairs == EXPECTED_ANCESTOR_WINNER
        && proof.runner_up_repaired_pairs == EXPECTED_ANCESTOR_RUNNER_UP
        && proof.winner_margin == EXPECTED_ANCESTOR_MARGIN
        && proof.partition_support_min == EXPECTED_ANCESTOR_SUPPORT
        && proof.winning_class_representatives == EXPECTED_WINNING_CLASS
}

fn raw_audit_exact(
    audit: &RawExpressibilityAudit,
    budget: &TransportDescendantBudget,
) -> bool {
    audit.candidate_program_count == EXPECTED_RAW_CANDIDATES
        && audit.unique_partition_count == EXPECTED_RAW_UNIQUE_PARTITIONS
        && audit.opposite_outcome_pairs == EXPECTED_ALIAS_DEFECTS
        && audit.best_repaired_pairs == EXPECTED_RAW_BEST_REPAIR
        && !audit.complete_repair_exists
        && raw_audit_budget_exact(budget)
}

fn descendant_proof_exact(proof: &TransportDescendantProof) -> bool {
    proof.raw_candidate_program_count == EXPECTED_RAW_CANDIDATES
        && proof.descendant_candidate_program_count == EXPECTED_DESCENDANT_CANDIDATES
        && proof.unique_partition_count == EXPECTED_DESCENDANT_UNIQUE_PARTITIONS
        && proof.opposite_outcome_pairs == EXPECTED_ALIAS_DEFECTS
        && proof.repaired_pairs == EXPECTED_DESCENDANT_WINNER
        && proof.unrepaired_pairs == 0
        && proof.runner_up_repaired_pairs == EXPECTED_DESCENDANT_RUNNER_UP
        && proof.winner_margin == EXPECTED_DESCENDANT_MARGIN
        && proof.partition_support_min == EXPECTED_DESCENDANT_SUPPORT
}

fn transport_budget_exact(budget: &TransportBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.discovery_program_history_evaluations == EXPECTED_RAW_EVALUATIONS
        && budget.unique_partitions == EXPECTED_RAW_UNIQUE_PARTITIONS
        && budget.winning_class_representatives == EXPECTED_WINNING_CLASS
        && budget.calibration_transformations == 2
        && budget.transport_program_history_evaluations == EXPECTED_TRANSPORT_EVALUATIONS
}

fn raw_audit_budget_exact(budget: &TransportDescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == EXPECTED_RAW_EVALUATIONS
        && budget.ancestor_terminals == 0
        && budget.descendant_candidate_programs == 0
        && budget.descendant_program_history_evaluations == 0
        && budget.unique_partitions == EXPECTED_RAW_UNIQUE_PARTITIONS
}

fn no_ancestor_budget_exact(budget: &TransportDescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == 0
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == 0
        && budget.ancestor_terminals == 0
        && budget.descendant_candidate_programs == 0
        && budget.descendant_program_history_evaluations == 0
        && budget.unique_partitions == 0
}

fn descendant_budget_exact(budget: &TransportDescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == 0
        && budget.ancestor_terminals == 1
        && budget.descendant_candidate_programs == EXPECTED_DESCENDANT_CANDIDATES
        && budget.descendant_program_history_evaluations == EXPECTED_DESCENDANT_EVALUATIONS
        && budget.unique_partitions == EXPECTED_DESCENDANT_UNIQUE_PARTITIONS
}

fn common_prediction_budget_exact(ledger: &BudgetLedger) -> bool {
    ledger.heldout_transformation_applications == PREDICTIONS_PER_ROOT
        && ledger.discovery_key_index_passes == 1
        && ledger.prediction_attempts == PREDICTIONS_PER_ROOT
        && ledger.objective_checks == PREDICTIONS_PER_ROOT
}

fn all_transport_budgets_exact(ledger: &BudgetLedger) -> bool {
    ledger.transport_proposals.iter().all(transport_budget_exact)
        && ledger.transport_validations.iter().all(transport_budget_exact)
}

fn budget_exact_for_path(path: PathKind, ledger: &BudgetLedger) -> bool {
    if !common_prediction_budget_exact(ledger) || !all_transport_budgets_exact(ledger) {
        return false;
    }
    match path {
        PathKind::TransportDescendantChain | PathKind::StationaryAncestorDescendantChain => {
            ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 1
        }
        PathKind::L0RawSearch => {
            ledger.transport_proposals.is_empty()
                && ledger.transport_validations.is_empty()
                && ledger.raw_audits.len() == 2
                && ledger.raw_audits.iter().all(raw_audit_budget_exact)
                && ledger.descendant_proposals.is_empty()
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
        }
        PathKind::L0DescendantNoAncestor => {
            ledger.transport_proposals.is_empty()
                && ledger.transport_validations.is_empty()
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && no_ancestor_budget_exact(&ledger.descendant_proposals[0])
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
        }
        PathKind::AncestorCertificatePayloadOnly => {
            ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && no_ancestor_budget_exact(&ledger.descendant_proposals[0])
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
        }
        PathKind::WrongTransportCertifiedAncestor => {
            ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 0
        }
        PathKind::ExactAncestorReplacedBeforeDescendantValidation => {
            ledger.transport_proposals.len() == 2
                && ledger.transport_validations.len() == 2
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 2
                && ledger.descendant_admission_slots == 0
        }
        PathKind::DescendantPayloadOnly | PathKind::CounterfeitDescendantProof => {
            ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 0
        }
        PathKind::DelayedDescendantAdmission => {
            ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 1
        }
    }
}
