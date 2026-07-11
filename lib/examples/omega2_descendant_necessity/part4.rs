fn metrics_from_entries(entries: &[(Execution, bool)]) -> PathMetrics {
    let roots = entries.len();
    PathMetrics {
        roots,
        root_successes: entries.iter().filter(|(execution, _)| execution.success).count(),
        success_rate: entries.iter().filter(|(execution, _)| execution.success).count() as f64
            / roots.max(1) as f64,
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
        descendant_proposal_successes: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_proposal_succeeded)
            .count(),
        descendant_validation_successes: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_validation_succeeded)
            .count(),
        descendant_admissions_during_prediction: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_admitted_during_prediction)
            .count(),
        descendant_final_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_admitted_final)
            .count(),
        no_ancestor_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.no_ancestor_rejection)
            .count(),
        descendant_validation_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.descendant_validation_rejected)
            .count(),
        raw_audit_complete_repairs: entries
            .iter()
            .filter(|(execution, _)| execution.raw_audit_complete_repair)
            .count(),
        payload_preservations: entries
            .iter()
            .filter(|(execution, _)| execution.payload_preserved)
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

fn stateful_perfect(report: &SplitReport) -> bool {
    let metrics = path_metrics(report, PathKind::StatefulDescendantChain);
    metrics.root_successes == metrics.roots
        && metrics.total_correct_predictions == metrics.roots * TRANSFER_HISTORIES
        && metrics.descendant_validation_successes == metrics.roots
        && metrics.descendant_admissions_during_prediction == metrics.roots
}

fn controls_have_zero_predictions(report: &SplitReport) -> bool {
    PathKind::all()
        .into_iter()
        .filter(|path| *path != PathKind::StatefulDescendantChain)
        .all(|path| {
            let metrics = path_metrics(report, path);
            metrics.root_successes == 0 && metrics.total_correct_predictions == 0
        })
}

fn ancestor_proof_exact(proof: &RefinementProof) -> bool {
    proof.candidate_program_count == EXPECTED_RAW_CANDIDATES
        && proof.unique_partition_count == EXPECTED_RAW_UNIQUE_PARTITIONS
        && proof.detected_defects == EXPECTED_OMEGA1_ALIAS_DEFECTS
        && proof.repaired_defects == EXPECTED_OMEGA1_ALIAS_DEFECTS
        && proof.unrepaired_defects == 0
        && proof.partition_support_min == 4
        && proof.runner_up_repaired_defects == 8
        && proof.winner_margin == 8
}

fn descendant_proof_exact(proof: &DescendantProof) -> bool {
    proof.raw_candidate_program_count == EXPECTED_RAW_CANDIDATES
        && proof.descendant_candidate_program_count == EXPECTED_DESCENDANT_CANDIDATES
        && proof.unique_partition_count == EXPECTED_DESCENDANT_UNIQUE_PARTITIONS
        && proof.opposite_outcome_pairs == EXPECTED_STAGE2_OPPOSITION_PAIRS
        && proof.repaired_pairs == EXPECTED_DESCENDANT_REPAIRED
        && proof.unrepaired_pairs == 0
        && proof.partition_support_min == EXPECTED_DESCENDANT_SUPPORT
        && proof.runner_up_repaired_pairs == EXPECTED_DESCENDANT_RUNNER_UP
        && proof.winner_margin == EXPECTED_DESCENDANT_MARGIN
}

fn raw_audit_exact(audit: &RawExpressibilityAudit, budget: &DescendantBudget) -> bool {
    audit.candidate_program_count == EXPECTED_RAW_CANDIDATES
        && audit.unique_partition_count == EXPECTED_RAW_UNIQUE_PARTITIONS
        && audit.opposite_outcome_pairs == EXPECTED_STAGE2_OPPOSITION_PAIRS
        && audit.best_repaired_pairs == EXPECTED_L0_BEST_REPAIR
        && !audit.complete_repair_exists
        && raw_audit_budget_exact(budget)
}

fn omega1_budget_exact(budget: &GenesisBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_OMEGA1_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.program_history_evaluations == EXPECTED_RAW_EVALUATIONS
        && budget.unique_partitions == EXPECTED_RAW_UNIQUE_PARTITIONS
}

fn omega1_search_ledger_exact(ledger: &BudgetLedger) -> bool {
    ledger.ancestor_proposals.len() == 1
        && ledger.ancestor_validations.len() == 1
        && ledger.ancestor_proposals.iter().all(omega1_budget_exact)
        && ledger.ancestor_validations.iter().all(omega1_budget_exact)
}

fn raw_audit_budget_exact(budget: &DescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.opposite_outcome_pair_checks == EXPECTED_STAGE2_OPPOSITION_PAIRS
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == EXPECTED_RAW_EVALUATIONS
        && budget.ancestor_terminals == 0
        && budget.descendant_candidate_programs == 0
        && budget.descendant_program_history_evaluations == 0
        && budget.unique_partitions == EXPECTED_RAW_UNIQUE_PARTITIONS
}

fn no_ancestor_budget_exact(budget: &DescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.opposite_outcome_pair_checks == 0
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == 0
        && budget.ancestor_terminals == 0
        && budget.descendant_candidate_programs == 0
        && budget.descendant_program_history_evaluations == 0
        && budget.unique_partitions == 0
}

fn descendant_budget_exact(budget: &DescendantBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.opposite_outcome_pair_checks == EXPECTED_STAGE2_OPPOSITION_PAIRS
        && budget.raw_candidate_programs == EXPECTED_RAW_CANDIDATES
        && budget.raw_program_history_evaluations == 0
        && budget.ancestor_terminals == 1
        && budget.descendant_candidate_programs == EXPECTED_DESCENDANT_CANDIDATES
        && budget.descendant_program_history_evaluations == EXPECTED_DESCENDANT_EVALUATIONS
        && budget.unique_partitions == EXPECTED_DESCENDANT_UNIQUE_PARTITIONS
}

fn common_prediction_budget_exact(ledger: &BudgetLedger) -> bool {
    ledger.downstream_key_index_passes == 1
        && ledger.transfer_predictions == TRANSFER_HISTORIES
        && ledger.objective_checks == TRANSFER_HISTORIES
}

fn all_omega1_budgets_exact(ledger: &BudgetLedger) -> bool {
    ledger.ancestor_proposals.iter().all(omega1_budget_exact)
        && ledger.ancestor_validations.iter().all(omega1_budget_exact)
}

fn budget_exact_for_path(path: PathKind, ledger: &BudgetLedger) -> bool {
    if !common_prediction_budget_exact(ledger) || !all_omega1_budgets_exact(ledger) {
        return false;
    }
    match path {
        PathKind::StatefulDescendantChain => {
            ledger.ancestor_proposals.len() == 1
                && ledger.ancestor_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 1
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::L0RawSearch => {
            ledger.ancestor_proposals.is_empty()
                && ledger.ancestor_validations.is_empty()
                && ledger.raw_audits.len() == 2
                && ledger.raw_audits.iter().all(raw_audit_budget_exact)
                && ledger.descendant_proposals.is_empty()
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::L0DescendantNoAncestor => {
            ledger.ancestor_proposals.is_empty()
                && ledger.ancestor_validations.is_empty()
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && no_ancestor_budget_exact(&ledger.descendant_proposals[0])
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::Delta1EndpointOnly => {
            ledger.ancestor_proposals.len() == 1
                && ledger.ancestor_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && no_ancestor_budget_exact(&ledger.descendant_proposals[0])
                && ledger.descendant_validations.is_empty()
                && ledger.ancestor_admission_slots == 0
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == DISCOVERY_HISTORIES
        }
        PathKind::WrongValidAncestor => {
            ledger.ancestor_proposals.len() == 1
                && ledger.ancestor_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::Delta1AblationReplacedBeforeValidation => {
            ledger.ancestor_proposals.len() == 2
                && ledger.ancestor_validations.len() == 2
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 2
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::Delta2PayloadOnly
        | PathKind::CounterfeitDelta2Proof
        | PathKind::OutcomeShuffledDescendant => {
            ledger.ancestor_proposals.len() == 1
                && ledger.ancestor_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 0
                && ledger.endpoint_ancestor_executions == 0
        }
        PathKind::DelayedDelta2Admission => {
            ledger.ancestor_proposals.len() == 1
                && ledger.ancestor_validations.len() == 1
                && ledger.raw_audits.is_empty()
                && ledger.descendant_proposals.len() == 1
                && ledger.descendant_validations.len() == 1
                && ledger.descendant_proposals.iter().all(descendant_budget_exact)
                && ledger.descendant_validations.iter().all(descendant_budget_exact)
                && ledger.ancestor_admission_slots == 1
                && ledger.descendant_admission_slots == 1
                && ledger.endpoint_ancestor_executions == 0
        }
    }
}
