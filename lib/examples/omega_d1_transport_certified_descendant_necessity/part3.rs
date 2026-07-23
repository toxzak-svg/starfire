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

fn admit_ancestor(
    root_id: u64,
    certificate: &ValidatedTransportCertificate,
    ledger: &mut BudgetLedger,
) -> Result<TransportStateLanguage, Box<dyn Error>> {
    let mut ancestor = TransportStateLanguage::new(root_id);
    ledger.ancestor_admission_slots = ledger.ancestor_admission_slots.saturating_add(1);
    ancestor.admit_certificate(certificate)?;
    Ok(ancestor)
}

fn propose_descendant(
    problem: &RefinementProblem,
    ancestor: &TransportStateLanguage,
    ledger: &mut BudgetLedger,
) -> Result<TransportDescendantProof, TransportDescendantError> {
    let mut budget = TransportDescendantBudget::default();
    let result = synthesize_transport_descendant(
        problem,
        ancestor,
        TransportDescendantConfig::default(),
        &mut budget,
    );
    ledger.descendant_proposals.push(budget);
    result
}

fn validate_descendant_with_ledger(
    problem: &RefinementProblem,
    ancestor: &TransportStateLanguage,
    proof: &TransportDescendantProof,
    ledger: &mut BudgetLedger,
) -> Result<ValidatedTransportDescendantCertificate, TransportDescendantError> {
    let mut budget = TransportDescendantBudget::default();
    let result = validate_transport_descendant(
        problem,
        ancestor,
        proof,
        TransportDescendantConfig::default(),
        &mut budget,
    );
    ledger.descendant_validations.push(budget);
    result
}

fn raw_audit_with_ledger(
    problem: &RefinementProblem,
    ledger: &mut BudgetLedger,
) -> Result<RawExpressibilityAudit, TransportDescendantError> {
    let mut budget = TransportDescendantBudget::default();
    let result = audit_raw_expressibility(problem, &mut budget);
    ledger.raw_audits.push(budget);
    result
}

fn execute_path(root: &RootTask, path: PathKind) -> Result<Execution, Box<dyn Error>> {
    let mut budget = BudgetLedger::default();
    let mut selected_ancestor_program = None;
    let mut correct_ancestor_proposal_succeeded = false;
    let mut correct_ancestor_validation_succeeded = false;
    let mut correct_ancestor_admitted = false;
    let mut wrong_ancestor_proposal_succeeded = false;
    let mut wrong_ancestor_validation_succeeded = false;
    let mut wrong_ancestor_admitted = false;
    let mut stationary_ancestor_proposal_succeeded = false;
    let mut stationary_ancestor_validation_succeeded = false;
    let mut stationary_ancestor_admitted = false;
    let mut descendant_proposal_succeeded = false;
    let mut descendant_validation_succeeded = false;
    let mut descendant_admitted_final = false;
    let mut no_ancestor_rejection = false;
    let mut descendant_validation_rejected = false;
    let mut raw_audit_complete_repair = false;
    let mut ancestor_payload_preserved = false;
    let mut descendant_payload_preserved = false;
    let mut descendant_repaired_pairs = 0_usize;
    let mut descendant_candidate_count = 0_usize;
    let mut descendant_unique_partitions = 0_usize;
    let mut descendant_winner_margin = 0_usize;
    let mut descendant_partition_support = 0_usize;
    let mut validation_error = None;
    let mut admission_error = None;
    let mut delayed_descendant_certificate: Option<ValidatedTransportDescendantCertificate> = None;

    let mut layered = TransportDescendantStateLanguage::new(TransportStateLanguage::new(root.root_id));

    match path {
        PathKind::TransportDescendantChain => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            let certificate = validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            )?;
            descendant_validation_succeeded = true;
            layered = TransportDescendantStateLanguage::new(ancestor);
            budget.descendant_admission_slots =
                budget.descendant_admission_slots.saturating_add(1);
            layered.admit_certificate(&certificate)?;
            descendant_admitted_final = true;
        }
        PathKind::L0RawSearch => {
            let first = raw_audit_with_ledger(&root.stage2_problem, &mut budget)?;
            let second = raw_audit_with_ledger(&root.stage2_problem, &mut budget)?;
            raw_audit_complete_repair = first.complete_repair_exists || second.complete_repair_exists;
        }
        PathKind::L0DescendantNoAncestor => {
            let ancestor = TransportStateLanguage::new(root.root_id);
            match propose_descendant(&root.stage2_problem, &ancestor, &mut budget) {
                Err(TransportDescendantError::NoAncestorRefinement) => {
                    no_ancestor_rejection = true;
                }
                Err(error) => return Err(error.into()),
                Ok(proof) => {
                    descendant_proposal_succeeded = true;
                    capture_descendant_metrics(
                        &proof,
                        &mut descendant_repaired_pairs,
                        &mut descendant_candidate_count,
                        &mut descendant_unique_partitions,
                        &mut descendant_winner_margin,
                        &mut descendant_partition_support,
                    );
                }
            }
        }
        PathKind::AncestorCertificatePayloadOnly => {
            let (ancestor_proof, _ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            ancestor_payload_preserved = true;
            let ancestor = TransportStateLanguage::new(root.root_id);
            match propose_descendant(&root.stage2_problem, &ancestor, &mut budget) {
                Err(TransportDescendantError::NoAncestorRefinement) => {
                    no_ancestor_rejection = true;
                }
                Err(error) => return Err(error.into()),
                Ok(proof) => {
                    descendant_proposal_succeeded = true;
                    capture_descendant_metrics(
                        &proof,
                        &mut descendant_repaired_pairs,
                        &mut descendant_candidate_count,
                        &mut descendant_unique_partitions,
                        &mut descendant_winner_margin,
                        &mut descendant_partition_support,
                    );
                }
            }
        }
        PathKind::WrongTransportCertifiedAncestor => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.wrong_ancestor_problem, &wrong_suite(), &mut budget)?;
            wrong_ancestor_proposal_succeeded = true;
            wrong_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            wrong_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = TransportDescendantStateLanguage::new(ancestor);
        }
        PathKind::ExactAncestorReplacedBeforeDescendantValidation => {
            let (correct_proof, correct_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(correct_proof.program.canonical_string());
            let correct_ancestor = admit_ancestor(root.root_id, &correct_certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let descendant_proof =
                propose_descendant(&root.stage2_problem, &correct_ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &descendant_proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );

            let (_wrong_proof, wrong_certificate) =
                prepare_transport(&root.wrong_ancestor_problem, &wrong_suite(), &mut budget)?;
            wrong_ancestor_proposal_succeeded = true;
            wrong_ancestor_validation_succeeded = true;
            let wrong_ancestor = admit_ancestor(root.root_id, &wrong_certificate, &mut budget)?;
            wrong_ancestor_admitted = true;

            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &wrong_ancestor,
                &descendant_proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = TransportDescendantStateLanguage::new(wrong_ancestor);
        }
        PathKind::StationaryAncestorDescendantChain => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &stationary_suite(), &mut budget)?;
            stationary_ancestor_proposal_succeeded = true;
            stationary_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            stationary_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            let certificate = validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            )?;
            descendant_validation_succeeded = true;
            layered = TransportDescendantStateLanguage::new(ancestor);
            budget.descendant_admission_slots =
                budget.descendant_admission_slots.saturating_add(1);
            layered.admit_certificate(&certificate)?;
            descendant_admitted_final = true;
        }
        PathKind::DescendantPayloadOnly => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            let _certificate = validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            )?;
            descendant_validation_succeeded = true;
            descendant_payload_preserved = true;
            layered = TransportDescendantStateLanguage::new(ancestor);
        }
        PathKind::CounterfeitDescendantProof => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let mut proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            proof.winner_margin = proof.winner_margin.saturating_add(1);
            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = TransportDescendantStateLanguage::new(ancestor);
        }
        PathKind::DelayedDescendantAdmission => {
            let (ancestor_proof, ancestor_certificate) =
                prepare_transport(&root.correct_ancestor_problem, &correct_suite(), &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            selected_ancestor_program = Some(ancestor_proof.program.canonical_string());
            let ancestor = admit_ancestor(root.root_id, &ancestor_certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &ancestor, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_descendant_metrics(
                &proof,
                &mut descendant_repaired_pairs,
                &mut descendant_candidate_count,
                &mut descendant_unique_partitions,
                &mut descendant_winner_margin,
                &mut descendant_partition_support,
            );
            delayed_descendant_certificate = Some(validate_descendant_with_ledger(
                &root.stage2_problem,
                &ancestor,
                &proof,
                &mut budget,
            )?);
            descendant_validation_succeeded = true;
            layered = TransportDescendantStateLanguage::new(ancestor);
        }
    }

    let descendant_admitted_during_window = layered.descendant_count() > 0;
    let correct_predictions = predict_transfer(root, &layered, &mut budget)?;

    if let Some(certificate) = delayed_descendant_certificate {
        budget.descendant_admission_slots =
            budget.descendant_admission_slots.saturating_add(1);
        match layered.admit_certificate(&certificate) {
            Ok(()) => descendant_admitted_final = true,
            Err(error) => admission_error = Some(error.to_string()),
        }
    } else if descendant_admitted_during_window {
        descendant_admitted_final = true;
    }

    let invariants_hold = layered.verify_invariants().is_ok();
    let final_language_signature = layered.canonical_signature();
    let budget_exact = budget_exact_for_path(path, &budget);

    Ok(Execution {
        correct_predictions,
        full_success: correct_predictions == PREDICTIONS_PER_ROOT,
        selected_ancestor_program,
        correct_ancestor_proposal_succeeded,
        correct_ancestor_validation_succeeded,
        correct_ancestor_admitted,
        wrong_ancestor_proposal_succeeded,
        wrong_ancestor_validation_succeeded,
        wrong_ancestor_admitted,
        stationary_ancestor_proposal_succeeded,
        stationary_ancestor_validation_succeeded,
        stationary_ancestor_admitted,
        descendant_proposal_succeeded,
        descendant_validation_succeeded,
        descendant_admitted_during_window,
        descendant_admitted_final,
        no_ancestor_rejection,
        descendant_validation_rejected,
        raw_audit_complete_repair,
        ancestor_payload_preserved,
        descendant_payload_preserved,
        descendant_repaired_pairs,
        descendant_candidate_count,
        descendant_unique_partitions,
        descendant_winner_margin,
        descendant_partition_support,
        validation_error,
        admission_error,
        budget,
        budget_exact,
        invariants_hold,
        final_language_signature,
    })
}

fn capture_descendant_metrics(
    proof: &TransportDescendantProof,
    repaired_pairs: &mut usize,
    candidate_count: &mut usize,
    unique_partitions: &mut usize,
    winner_margin: &mut usize,
    partition_support: &mut usize,
) {
    *repaired_pairs = proof.repaired_pairs;
    *candidate_count = proof.descendant_candidate_program_count;
    *unique_partitions = proof.unique_partition_count;
    *winner_margin = proof.winner_margin;
    *partition_support = proof.partition_support_min;
}
