fn prepare_transport(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    ledger: &mut BudgetLedger,
) -> Result<(TransportProof, ValidatedTransportCertificate), Box<dyn Error>> {
    let mut proposal_budget = TransportBudget::default();
    let proof = synthesize_transport_refinement(
        problem,
        suite,
        TransportConfig::default(),
        &mut proposal_budget,
    )?;
    ledger.transport_proposals.push(proposal_budget);

    let mut validation_budget = TransportBudget::default();
    let certificate = validate_transport_refinement(
        problem,
        suite,
        &proof,
        TransportConfig::default(),
        &mut validation_budget,
    )?;
    ledger.transport_validations.push(validation_budget);
    Ok((proof, certificate))
}

fn propose_transport(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    ledger: &mut BudgetLedger,
) -> Result<TransportProof, TransportError> {
    let mut budget = TransportBudget::default();
    let result = synthesize_transport_refinement(
        problem,
        suite,
        TransportConfig::default(),
        &mut budget,
    );
    ledger.transport_proposals.push(budget);
    result
}

fn validate_transport_with_ledger(
    problem: &RefinementProblem,
    suite: &TransformationSuite,
    proof: &TransportProof,
    ledger: &mut BudgetLedger,
) -> Result<ValidatedTransportCertificate, TransportError> {
    let mut budget = TransportBudget::default();
    let result = validate_transport_refinement(
        problem,
        suite,
        proof,
        TransportConfig::default(),
        &mut budget,
    );
    ledger.transport_validations.push(budget);
    result
}

fn prepare_partition_only(
    problem: &RefinementProblem,
    ledger: &mut BudgetLedger,
) -> Result<(RefinementProof, StateLanguage), Box<dyn Error>> {
    let mut proposal_budget = GenesisBudget::default();
    let proof = synthesize_refinement(problem, RefinementConfig::default(), &mut proposal_budget)?;
    ledger.omega1_proposals.push(proposal_budget);

    let mut validation_budget = GenesisBudget::default();
    let certificate = validate_refinement(
        problem,
        &proof,
        RefinementConfig::default(),
        &mut validation_budget,
    )?;
    ledger.omega1_validations.push(validation_budget);

    let mut language = StateLanguage::new(problem.root_id);
    language.admit_certificate(&certificate)?;
    Ok((proof, language))
}

fn execute_path(root: &RootTask, path: PathKind) -> Result<Execution, Box<dyn Error>> {
    let mut budget = BudgetLedger::default();
    let mut selected_program = None;
    let mut baseline_program = None;
    let proposal_succeeded = true;
    let mut validation_succeeded = false;
    let mut validation_rejected = false;
    let mut admission_succeeded_during_window = false;
    let mut final_admission_succeeded = false;
    let mut admission_rejected = false;
    let mut payload_preserved = false;
    let mut candidate_program_count = 0_usize;
    let mut unique_partition_count = 0_usize;
    let mut repaired_pairs = 0_usize;
    let mut runner_up_repaired_pairs = 0_usize;
    let mut winner_margin = 0_usize;
    let mut partition_support_min = 0_usize;
    let mut winning_class_representatives = 0_usize;
    let mut zero_violation_representatives = 0_usize;
    let mut minimum_transport_violations = 0_usize;
    let mut selected_transport_violations = 0_usize;
    let mut validation_error = None;
    let mut admission_error = None;

    let mut transport_language = TransportStateLanguage::new(root.root_id);
    let mut partition_language: Option<StateLanguage> = None;
    let mut delayed_certificate: Option<ValidatedTransportCertificate> = None;

    match path {
        PathKind::OrbitAwareStateful => {
            let (proof, certificate) = prepare_transport(&root.problem, &moving_suite(), &mut budget)?;
            validation_succeeded = true;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            budget.transport_admission_slots = budget.transport_admission_slots.saturating_add(1);
            transport_language.admit_certificate(&certificate)?;
            admission_succeeded_during_window = true;
            final_admission_succeeded = true;
        }
        PathKind::PartitionOnlyBaseline => {
            let (proof, language) = prepare_partition_only(&root.problem, &mut budget)?;
            validation_succeeded = true;
            baseline_program = Some(proof.program.canonical_string());
            selected_program = baseline_program.clone();
            candidate_program_count = proof.candidate_program_count;
            unique_partition_count = proof.unique_partition_count;
            repaired_pairs = proof.repaired_defects;
            runner_up_repaired_pairs = proof.runner_up_repaired_defects;
            winner_margin = proof.winner_margin;
            partition_support_min = proof.partition_support_min;
            partition_language = Some(language);
            admission_succeeded_during_window = true;
            final_admission_succeeded = true;
        }
        PathKind::TargetStationaryMatchedCalibration => {
            let (proof, certificate) =
                prepare_transport(&root.problem, &stationary_suite(), &mut budget)?;
            validation_succeeded = true;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            budget.transport_admission_slots = budget.transport_admission_slots.saturating_add(1);
            transport_language.admit_certificate(&certificate)?;
            admission_succeeded_during_window = true;
            final_admission_succeeded = true;
        }
        PathKind::RewiredCorrespondenceCalibration => {
            let suite = rewired_suite();
            let proof = propose_transport(&root.problem, &suite, &mut budget)?;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            match validate_transport_with_ledger(&root.problem, &suite, &proof, &mut budget) {
                Ok(_) => validation_succeeded = true,
                Err(error) => {
                    validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
        }
        PathKind::TransportPayloadOnly => {
            let (proof, _certificate) = prepare_transport(&root.problem, &moving_suite(), &mut budget)?;
            validation_succeeded = true;
            payload_preserved = true;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
        }
        PathKind::CounterfeitTransportProof => {
            let suite = moving_suite();
            let mut proof = propose_transport(&root.problem, &suite, &mut budget)?;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            proof.zero_violation_representatives = proof.zero_violation_representatives.saturating_add(1);
            match validate_transport_with_ledger(&root.problem, &suite, &proof, &mut budget) {
                Ok(_) => validation_succeeded = true,
                Err(error) => {
                    validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
        }
        PathKind::ForeignRootTransportCertificate => {
            let donor_problem = foreign_problem(&root.problem);
            let (proof, certificate) = prepare_transport(&donor_problem, &moving_suite(), &mut budget)?;
            validation_succeeded = true;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            budget.transport_admission_slots = budget.transport_admission_slots.saturating_add(1);
            match transport_language.admit_certificate(&certificate) {
                Ok(()) => {
                    admission_succeeded_during_window = true;
                    final_admission_succeeded = true;
                }
                Err(error) => {
                    admission_rejected = true;
                    admission_error = Some(error.to_string());
                }
            }
        }
        PathKind::DelayedTransportAdmission => {
            let (proof, certificate) = prepare_transport(&root.problem, &moving_suite(), &mut budget)?;
            validation_succeeded = true;
            capture_transport_metrics(
                &proof,
                &mut selected_program,
                &mut candidate_program_count,
                &mut unique_partition_count,
                &mut repaired_pairs,
                &mut runner_up_repaired_pairs,
                &mut winner_margin,
                &mut partition_support_min,
                &mut winning_class_representatives,
                &mut zero_violation_representatives,
                &mut minimum_transport_violations,
                &mut selected_transport_violations,
            );
            delayed_certificate = Some(certificate);
        }
    }

    let correct_predictions = if let Some(language) = partition_language.as_ref() {
        predict_partition_language(root, language, &mut budget)?
    } else {
        predict_transport_language(root, &transport_language, &mut budget)?
    };

    if let Some(certificate) = delayed_certificate {
        budget.transport_admission_slots = budget.transport_admission_slots.saturating_add(1);
        match transport_language.admit_certificate(&certificate) {
            Ok(()) => final_admission_succeeded = true,
            Err(error) => admission_error = Some(error.to_string()),
        }
    }

    let invariants_hold = transport_language.verify_invariants().is_ok()
        && partition_language
            .as_ref()
            .map(|language| language.verify_invariants().is_ok())
            .unwrap_or(true);
    let final_language_signature = if let Some(language) = partition_language.as_ref() {
        language.canonical_signature()
    } else {
        transport_language.canonical_signature()
    };
    let budget_exact = budget_exact_for_path(path, &budget);

    Ok(Execution {
        correct_predictions,
        full_success: correct_predictions == PREDICTIONS_PER_ROOT,
        selected_program,
        baseline_program,
        proposal_succeeded,
        validation_succeeded,
        validation_rejected,
        admission_succeeded_during_window,
        final_admission_succeeded,
        admission_rejected,
        payload_preserved,
        candidate_program_count,
        unique_partition_count,
        repaired_pairs,
        runner_up_repaired_pairs,
        winner_margin,
        partition_support_min,
        winning_class_representatives,
        zero_violation_representatives,
        minimum_transport_violations,
        selected_transport_violations,
        validation_error,
        admission_error,
        budget,
        budget_exact,
        invariants_hold,
        final_language_signature,
    })
}

#[allow(clippy::too_many_arguments)]
fn capture_transport_metrics(
    proof: &TransportProof,
    selected_program: &mut Option<String>,
    candidate_program_count: &mut usize,
    unique_partition_count: &mut usize,
    repaired_pairs: &mut usize,
    runner_up_repaired_pairs: &mut usize,
    winner_margin: &mut usize,
    partition_support_min: &mut usize,
    winning_class_representatives: &mut usize,
    zero_violation_representatives: &mut usize,
    minimum_transport_violations: &mut usize,
    selected_transport_violations: &mut usize,
) {
    *selected_program = Some(proof.program.canonical_string());
    *candidate_program_count = proof.candidate_program_count;
    *unique_partition_count = proof.unique_partition_count;
    *repaired_pairs = proof.repaired_pairs;
    *runner_up_repaired_pairs = proof.runner_up_repaired_pairs;
    *winner_margin = proof.winner_margin;
    *partition_support_min = proof.partition_support_min;
    *winning_class_representatives = proof.winning_class_representatives;
    *zero_violation_representatives = proof.zero_violation_representatives;
    *minimum_transport_violations = proof.minimum_transport_violations;
    *selected_transport_violations = proof.selected_transport_violations;
}

fn foreign_problem(problem: &RefinementProblem) -> RefinementProblem {
    let mut foreign = problem.clone();
    foreign.root_id = foreign.root_id.saturating_add(1_000_000);
    foreign
}
