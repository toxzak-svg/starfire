fn metrics_from_entries(entries: &[(Execution, bool)]) -> PathMetrics {
    let roots = entries.len();
    let full_successes = entries
        .iter()
        .filter(|(execution, _)| execution.full_success)
        .count();
    let zero_values = entries
        .iter()
        .map(|(execution, _)| execution.zero_violation_representatives)
        .collect::<Vec<_>>();
    let violation_values = entries
        .iter()
        .map(|(execution, _)| execution.minimum_transport_violations)
        .collect::<Vec<_>>();

    PathMetrics {
        roots,
        full_successes,
        success_rate: full_successes as f64 / roots.max(1) as f64,
        total_correct_predictions: entries
            .iter()
            .map(|(execution, _)| execution.correct_predictions)
            .sum(),
        proposal_successes: entries
            .iter()
            .filter(|(execution, _)| execution.proposal_succeeded)
            .count(),
        validation_successes: entries
            .iter()
            .filter(|(execution, _)| execution.validation_succeeded)
            .count(),
        validation_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.validation_rejected)
            .count(),
        admissions_during_window: entries
            .iter()
            .filter(|(execution, _)| execution.admission_succeeded_during_window)
            .count(),
        final_admissions: entries
            .iter()
            .filter(|(execution, _)| execution.final_admission_succeeded)
            .count(),
        admission_rejections: entries
            .iter()
            .filter(|(execution, _)| execution.admission_rejected)
            .count(),
        payload_preservations: entries
            .iter()
            .filter(|(execution, _)| execution.payload_preserved)
            .count(),
        expected_stable_program_selections: entries
            .iter()
            .filter(|(execution, _)| {
                execution
                    .selected_program
                    .as_deref()
                    .map(is_stable_program_string)
                    .unwrap_or(false)
            })
            .count(),
        expected_accidental_program_selections: entries
            .iter()
            .filter(|(execution, _)| {
                execution
                    .selected_program
                    .as_deref()
                    .map(is_accidental_program_string)
                    .unwrap_or(false)
            })
            .count(),
        minimum_zero_violation_representatives: zero_values.iter().copied().min().unwrap_or(0),
        maximum_zero_violation_representatives: zero_values.iter().copied().max().unwrap_or(0),
        minimum_transport_violations: violation_values.iter().copied().min().unwrap_or(0),
        maximum_transport_violations: violation_values.iter().copied().max().unwrap_or(0),
        budgets_exact: entries
            .iter()
            .all(|(execution, _)| execution.budget_exact),
        replay_exact: entries.iter().all(|(_, replay)| *replay),
        invariants_hold: entries
            .iter()
            .all(|(execution, _)| execution.invariants_hold),
    }
}

fn is_stable_program_string(program: &str) -> bool {
    let Some(rest) = program.strip_prefix("first(") else {
        return false;
    };
    let Some((left, right_with_suffix)) = rest.split_once(")<first(") else {
        return false;
    };
    let Some(right) = right_with_suffix.strip_suffix(')') else {
        return false;
    };
    left.ends_with("_a")
        && right.ends_with("_b")
        && left.strip_suffix("_a") == right.strip_suffix("_b")
}

fn is_accidental_program_string(program: &str) -> bool {
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

fn perfect_orbit(report: &SplitReport) -> bool {
    let metrics = path_metrics(report, PathKind::OrbitAwareStateful);
    metrics.full_successes == metrics.roots
        && metrics.total_correct_predictions == metrics.roots * PREDICTIONS_PER_ROOT
        && metrics.validation_successes == metrics.roots
        && metrics.admissions_during_window == metrics.roots
        && metrics.expected_stable_program_selections == metrics.roots
        && metrics.minimum_zero_violation_representatives == EXPECTED_PRIMARY_ZERO_VIOLATION
        && metrics.maximum_zero_violation_representatives == EXPECTED_PRIMARY_ZERO_VIOLATION
        && metrics.minimum_transport_violations == 0
        && metrics.maximum_transport_violations == 0
}

fn exact_half_transfer(report: &SplitReport, path: PathKind) -> bool {
    let metrics = path_metrics(report, path);
    metrics.full_successes == 0
        && metrics.total_correct_predictions == metrics.roots * EXPECTED_BASELINE_CORRECT
        && metrics.validation_successes == metrics.roots
        && metrics.admissions_during_window == metrics.roots
        && metrics.expected_accidental_program_selections == metrics.roots
}

fn audit_roots(roots: &[RootTask]) -> Result<RootAudit, Box<dyn Error>> {
    let mut audit = RootAudit {
        roots: roots.len(),
        alias_defects_exact: 0,
        omega1_partition_search_exact: 0,
        omega1_accidental_representative_exact: 0,
        primary_transport_frontier_exact: 0,
        stationary_transport_frontier_exact: 0,
        rewired_transport_frontier_exact: 0,
    };

    for root in roots {
        let l0 = StateLanguage::new(root.root_id);
        let mut defect_budget = GenesisBudget::default();
        if detect_alias_defects(&root.problem, &l0, &mut defect_budget)
            .map(|defects| {
                defects.len() == EXPECTED_ALIAS_DEFECTS
                    && defect_budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
            })
            .unwrap_or(false)
        {
            audit.alias_defects_exact += 1;
        }

        let mut omega1_proposal_budget = GenesisBudget::default();
        let omega1_proof = synthesize_refinement(
            &root.problem,
            RefinementConfig::default(),
            &mut omega1_proposal_budget,
        )?;
        let mut omega1_validation_budget = GenesisBudget::default();
        let _ = validate_refinement(
            &root.problem,
            &omega1_proof,
            RefinementConfig::default(),
            &mut omega1_validation_budget,
        )?;
        if omega1_proof_exact(&omega1_proof)
            && omega1_budget_exact(&omega1_proposal_budget)
            && omega1_budget_exact(&omega1_validation_budget)
        {
            audit.omega1_partition_search_exact += 1;
        }
        if is_accidental_program_string(&omega1_proof.program.canonical_string()) {
            audit.omega1_accidental_representative_exact += 1;
        }

        let mut primary_budget = TransportBudget::default();
        let primary = synthesize_transport_refinement(
            &root.problem,
            &moving_suite(),
            TransportConfig::default(),
            &mut primary_budget,
        )?;
        if primary_transport_proof_exact(&primary)
            && transport_budget_exact(&primary_budget)
            && is_stable_program_string(&primary.program.canonical_string())
        {
            audit.primary_transport_frontier_exact += 1;
        }

        let mut stationary_budget = TransportBudget::default();
        let stationary = synthesize_transport_refinement(
            &root.problem,
            &stationary_suite(),
            TransportConfig::default(),
            &mut stationary_budget,
        )?;
        if stationary_transport_proof_exact(&stationary)
            && transport_budget_exact(&stationary_budget)
            && is_accidental_program_string(&stationary.program.canonical_string())
        {
            audit.stationary_transport_frontier_exact += 1;
        }

        let mut rewired_budget = TransportBudget::default();
        let rewired = synthesize_transport_refinement(
            &root.problem,
            &rewired_suite(),
            TransportConfig::default(),
            &mut rewired_budget,
        )?;
        if rewired_transport_proof_exact(&rewired)
            && transport_budget_exact(&rewired_budget)
        {
            audit.rewired_transport_frontier_exact += 1;
        }
    }

    Ok(audit)
}

fn omega1_proof_exact(proof: &RefinementProof) -> bool {
    proof.candidate_program_count == EXPECTED_CANDIDATES
        && proof.unique_partition_count == EXPECTED_UNIQUE_PARTITIONS
        && proof.detected_defects == EXPECTED_ALIAS_DEFECTS
        && proof.repaired_defects == EXPECTED_WINNER_REPAIR
        && proof.unrepaired_defects == 0
        && proof.partition_support_min == EXPECTED_SUPPORT
        && proof.runner_up_repaired_defects == EXPECTED_RUNNER_UP
        && proof.winner_margin == EXPECTED_MARGIN
}

fn common_transport_proof_exact(proof: &TransportProof) -> bool {
    proof.candidate_program_count == EXPECTED_CANDIDATES
        && proof.unique_partition_count == EXPECTED_UNIQUE_PARTITIONS
        && proof.opposite_outcome_pairs == EXPECTED_ALIAS_DEFECTS
        && proof.repaired_pairs == EXPECTED_WINNER_REPAIR
        && proof.runner_up_repaired_pairs == EXPECTED_RUNNER_UP
        && proof.winner_margin == EXPECTED_MARGIN
        && proof.partition_support_min == EXPECTED_SUPPORT
        && proof.winning_class_representatives == EXPECTED_WINNING_CLASS
}

fn primary_transport_proof_exact(proof: &TransportProof) -> bool {
    common_transport_proof_exact(proof)
        && proof.zero_violation_representatives == EXPECTED_PRIMARY_ZERO_VIOLATION
        && proof.minimum_transport_violations == 0
        && proof.selected_transport_violations == 0
}

fn stationary_transport_proof_exact(proof: &TransportProof) -> bool {
    common_transport_proof_exact(proof)
        && proof.zero_violation_representatives == EXPECTED_STATIONARY_ZERO_VIOLATION
        && proof.minimum_transport_violations == 0
        && proof.selected_transport_violations == 0
}

fn rewired_transport_proof_exact(proof: &TransportProof) -> bool {
    common_transport_proof_exact(proof)
        && proof.zero_violation_representatives == EXPECTED_REWIRED_ZERO_VIOLATION
        && proof.minimum_transport_violations == EXPECTED_REWIRED_MIN_VIOLATIONS
        && proof.selected_transport_violations == EXPECTED_REWIRED_MIN_VIOLATIONS
}

fn omega1_budget_exact(budget: &GenesisBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_CANDIDATES
        && budget.program_history_evaluations == EXPECTED_DISCOVERY_EVALUATIONS
        && budget.unique_partitions == EXPECTED_UNIQUE_PARTITIONS
}

fn transport_budget_exact(budget: &TransportBudget) -> bool {
    budget.vocabulary_history_scans == DISCOVERY_HISTORIES
        && budget.history_pair_evaluations == EXPECTED_PAIR_EVALUATIONS
        && budget.candidate_programs == EXPECTED_CANDIDATES
        && budget.discovery_program_history_evaluations == EXPECTED_DISCOVERY_EVALUATIONS
        && budget.unique_partitions == EXPECTED_UNIQUE_PARTITIONS
        && budget.winning_class_representatives == EXPECTED_WINNING_CLASS
        && budget.calibration_transformations == EXPECTED_CALIBRATION_TRANSFORMS
        && budget.transport_program_history_evaluations == EXPECTED_TRANSPORT_EVALUATIONS
}

fn common_prediction_budget_exact(ledger: &BudgetLedger) -> bool {
    ledger.heldout_transformation_applications == PREDICTIONS_PER_ROOT
        && ledger.discovery_key_index_passes == 1
        && ledger.prediction_attempts == PREDICTIONS_PER_ROOT
        && ledger.objective_checks == PREDICTIONS_PER_ROOT
}

fn budget_exact_for_path(path: PathKind, ledger: &BudgetLedger) -> bool {
    if !common_prediction_budget_exact(ledger) {
        return false;
    }
    match path {
        PathKind::OrbitAwareStateful => {
            ledger.omega1_proposals.is_empty()
                && ledger.omega1_validations.is_empty()
                && ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.transport_proposals.iter().all(transport_budget_exact)
                && ledger.transport_validations.iter().all(transport_budget_exact)
                && ledger.transport_admission_slots == 1
        }
        PathKind::PartitionOnlyBaseline => {
            ledger.omega1_proposals.len() == 1
                && ledger.omega1_validations.len() == 1
                && ledger.omega1_proposals.iter().all(omega1_budget_exact)
                && ledger.omega1_validations.iter().all(omega1_budget_exact)
                && ledger.transport_proposals.is_empty()
                && ledger.transport_validations.is_empty()
                && ledger.transport_admission_slots == 0
        }
        PathKind::TargetStationaryMatchedCalibration
        | PathKind::TransportPayloadOnly
        | PathKind::CounterfeitTransportProof => {
            ledger.omega1_proposals.is_empty()
                && ledger.omega1_validations.is_empty()
                && ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.transport_proposals.iter().all(transport_budget_exact)
                && ledger.transport_validations.iter().all(transport_budget_exact)
                && ledger.transport_admission_slots
                    == usize::from(path == PathKind::TargetStationaryMatchedCalibration)
        }
        PathKind::RewiredCorrespondenceCalibration => {
            ledger.omega1_proposals.is_empty()
                && ledger.omega1_validations.is_empty()
                && ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.transport_proposals.iter().all(transport_budget_exact)
                && ledger.transport_validations.iter().all(transport_budget_exact)
                && ledger.transport_admission_slots == 0
        }
        PathKind::ForeignRootTransportCertificate | PathKind::DelayedTransportAdmission => {
            ledger.omega1_proposals.is_empty()
                && ledger.omega1_validations.is_empty()
                && ledger.transport_proposals.len() == 1
                && ledger.transport_validations.len() == 1
                && ledger.transport_proposals.iter().all(transport_budget_exact)
                && ledger.transport_validations.iter().all(transport_budget_exact)
                && ledger.transport_admission_slots == 1
        }
    }
}
