#[derive(Debug, Clone)]
struct AncestorArtifacts {
    proof: RefinementProof,
    certificate: ValidatedRefinementCertificate,
}

fn prepare_ancestor(
    problem: &RefinementProblem,
    ledger: &mut BudgetLedger,
) -> Result<AncestorArtifacts, Box<dyn Error>> {
    let config = RefinementConfig::default();
    let mut proposal_budget = GenesisBudget::default();
    let proof = synthesize_refinement(problem, config, &mut proposal_budget)?;
    ledger.ancestor_proposals.push(proposal_budget);

    let mut validation_budget = GenesisBudget::default();
    let certificate = validate_refinement(problem, &proof, config, &mut validation_budget)?;
    ledger.ancestor_validations.push(validation_budget);
    Ok(AncestorArtifacts { proof, certificate })
}

fn propose_descendant(
    problem: &RefinementProblem,
    language: &StateLanguage,
    ledger: &mut BudgetLedger,
) -> Result<DescendantProof, DescendantGenesisError> {
    let mut budget = DescendantBudget::default();
    let result = synthesize_descendant(
        problem,
        language,
        DescendantConfig::default(),
        &mut budget,
    );
    ledger.descendant_proposals.push(budget);
    result
}

fn validate_descendant_with_ledger(
    problem: &RefinementProblem,
    language: &StateLanguage,
    proof: &DescendantProof,
    ledger: &mut BudgetLedger,
) -> Result<ValidatedDescendantCertificate, DescendantGenesisError> {
    let mut budget = DescendantBudget::default();
    let result = validate_descendant(
        problem,
        language,
        proof,
        DescendantConfig::default(),
        &mut budget,
    );
    ledger.descendant_validations.push(budget);
    result
}

fn raw_audit_with_ledger(
    problem: &RefinementProblem,
    ledger: &mut BudgetLedger,
) -> Result<RawExpressibilityAudit, DescendantGenesisError> {
    let mut budget = DescendantBudget::default();
    let result = audit_raw_expressibility(problem, &mut budget);
    ledger.raw_audits.push(budget);
    result
}

fn admit_ancestor(
    root_id: u64,
    certificate: &ValidatedRefinementCertificate,
    ledger: &mut BudgetLedger,
) -> Result<StateLanguage, Box<dyn Error>> {
    let mut language = StateLanguage::new(root_id);
    ledger.ancestor_admission_slots = ledger.ancestor_admission_slots.saturating_add(1);
    language.admit_certificate(certificate)?;
    Ok(language)
}

fn execute_path(root: &RootTask, path: PathKind) -> Result<Execution, Box<dyn Error>> {
    let mut budget = BudgetLedger::default();
    let mut correct_ancestor_proposal_succeeded = false;
    let mut correct_ancestor_validation_succeeded = false;
    let mut correct_ancestor_admitted = false;
    let mut wrong_ancestor_proposal_succeeded = false;
    let mut wrong_ancestor_validation_succeeded = false;
    let mut wrong_ancestor_admitted = false;
    let mut descendant_proposal_succeeded = false;
    let mut descendant_validation_succeeded = false;
    let mut descendant_admitted_during_prediction = false;
    let mut descendant_admitted_final = false;
    let mut no_ancestor_rejection = false;
    let mut descendant_validation_rejected = false;
    let mut raw_audit_complete_repair = false;
    let mut payload_preserved = false;
    let mut proposal_repaired_pairs = 0_usize;
    let mut proposal_candidate_count = 0_usize;
    let mut proposal_unique_partitions = 0_usize;
    let mut proposal_winner_margin = 0_usize;
    let mut validation_error = None;
    let mut admission_error = None;

    let mut layered = DescendantStateLanguage::new(StateLanguage::new(root.root_id));
    let mut delayed_certificate: Option<ValidatedDescendantCertificate> = None;

    match path {
        PathKind::StatefulDescendantChain => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            let certificate =
                validate_descendant_with_ledger(&root.stage2_problem, &base, &proof, &mut budget)?;
            descendant_validation_succeeded = true;
            layered = DescendantStateLanguage::new(base);
            budget.descendant_admission_slots =
                budget.descendant_admission_slots.saturating_add(1);
            layered.admit_certificate(&certificate)?;
            descendant_admitted_during_prediction = true;
            descendant_admitted_final = true;
        }
        PathKind::L0RawSearch => {
            let proposal_audit = raw_audit_with_ledger(&root.stage2_problem, &mut budget)?;
            let validation_audit = raw_audit_with_ledger(&root.stage2_problem, &mut budget)?;
            raw_audit_complete_repair =
                proposal_audit.complete_repair_exists || validation_audit.complete_repair_exists;
            payload_preserved = proposal_audit == validation_audit;
        }
        PathKind::L0DescendantNoAncestor => {
            let base = StateLanguage::new(root.root_id);
            match propose_descendant(&root.stage2_problem, &base, &mut budget) {
                Err(DescendantGenesisError::NoAncestorRefinement) => {
                    no_ancestor_rejection = true;
                }
                Err(error) => return Err(error.into()),
                Ok(proof) => {
                    descendant_proposal_succeeded = true;
                    capture_proposal_metrics(
                        &proof,
                        &mut proposal_repaired_pairs,
                        &mut proposal_candidate_count,
                        &mut proposal_unique_partitions,
                        &mut proposal_winner_margin,
                    );
                }
            }
        }
        PathKind::Delta1EndpointOnly => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            for episode in &root.stage2_problem.discovery {
                let _ = ancestor.proof.program.execute(&episode.history);
                budget.endpoint_ancestor_executions =
                    budget.endpoint_ancestor_executions.saturating_add(1);
            }
            let base = StateLanguage::new(root.root_id);
            match propose_descendant(&root.stage2_problem, &base, &mut budget) {
                Err(DescendantGenesisError::NoAncestorRefinement) => {
                    no_ancestor_rejection = true;
                }
                Err(error) => return Err(error.into()),
                Ok(proof) => {
                    descendant_proposal_succeeded = true;
                    capture_proposal_metrics(
                        &proof,
                        &mut proposal_repaired_pairs,
                        &mut proposal_candidate_count,
                        &mut proposal_unique_partitions,
                        &mut proposal_winner_margin,
                    );
                }
            }
        }
        PathKind::WrongValidAncestor => {
            let ancestor = prepare_ancestor(&root.wrong_ancestor_problem, &mut budget)?;
            wrong_ancestor_proposal_succeeded = true;
            wrong_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            wrong_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &base,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = DescendantStateLanguage::new(base);
        }
        PathKind::Delta1AblationReplacedBeforeValidation => {
            let correct_ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let correct_base =
                admit_ancestor(root.root_id, &correct_ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &correct_base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );

            let wrong_ancestor = prepare_ancestor(&root.wrong_ancestor_problem, &mut budget)?;
            wrong_ancestor_proposal_succeeded = true;
            wrong_ancestor_validation_succeeded = true;
            let replacement_base =
                admit_ancestor(root.root_id, &wrong_ancestor.certificate, &mut budget)?;
            wrong_ancestor_admitted = true;

            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &replacement_base,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = DescendantStateLanguage::new(replacement_base);
        }
        PathKind::Delta2PayloadOnly => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            let _certificate =
                validate_descendant_with_ledger(&root.stage2_problem, &base, &proof, &mut budget)?;
            descendant_validation_succeeded = true;
            payload_preserved = true;
            layered = DescendantStateLanguage::new(base);
        }
        PathKind::CounterfeitDelta2Proof => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let mut proof = propose_descendant(&root.stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            proof.winner_margin = proof.winner_margin.saturating_add(1);
            match validate_descendant_with_ledger(
                &root.stage2_problem,
                &base,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = DescendantStateLanguage::new(base);
        }
        PathKind::OutcomeShuffledDescendant => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.shuffled_stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            match validate_descendant_with_ledger(
                &root.shuffled_stage2_problem,
                &base,
                &proof,
                &mut budget,
            ) {
                Ok(_) => descendant_validation_succeeded = true,
                Err(error) => {
                    descendant_validation_rejected = true;
                    validation_error = Some(error.to_string());
                }
            }
            layered = DescendantStateLanguage::new(base);
        }
        PathKind::DelayedDelta2Admission => {
            let ancestor = prepare_ancestor(&root.correct_ancestor_problem, &mut budget)?;
            correct_ancestor_proposal_succeeded = true;
            correct_ancestor_validation_succeeded = true;
            let base = admit_ancestor(root.root_id, &ancestor.certificate, &mut budget)?;
            correct_ancestor_admitted = true;

            let proof = propose_descendant(&root.stage2_problem, &base, &mut budget)?;
            descendant_proposal_succeeded = true;
            capture_proposal_metrics(
                &proof,
                &mut proposal_repaired_pairs,
                &mut proposal_candidate_count,
                &mut proposal_unique_partitions,
                &mut proposal_winner_margin,
            );
            delayed_certificate = Some(validate_descendant_with_ledger(
                &root.stage2_problem,
                &base,
                &proof,
                &mut budget,
            )?);
            descendant_validation_succeeded = true;
            layered = DescendantStateLanguage::new(base);
        }
    }

    descendant_admitted_during_prediction = layered.descendant_count() > 0;
    let correct_predictions = predict_transfer(root, &layered, &mut budget)?;

    if let Some(certificate) = delayed_certificate {
        budget.descendant_admission_slots =
            budget.descendant_admission_slots.saturating_add(1);
        match layered.admit_certificate(&certificate) {
            Ok(()) => descendant_admitted_final = true,
            Err(error) => admission_error = Some(error.to_string()),
        }
    } else if descendant_admitted_during_prediction {
        descendant_admitted_final = true;
    }

    let invariants_hold = layered.verify_invariants().is_ok();
    let final_language_signature = layered.canonical_signature();
    let budget_exact = budget_exact_for_path(path, &budget);

    Ok(Execution {
        correct_predictions,
        success: correct_predictions == TRANSFER_HISTORIES,
        correct_ancestor_proposal_succeeded,
        correct_ancestor_validation_succeeded,
        correct_ancestor_admitted,
        wrong_ancestor_proposal_succeeded,
        wrong_ancestor_validation_succeeded,
        wrong_ancestor_admitted,
        descendant_proposal_succeeded,
        descendant_validation_succeeded,
        descendant_admitted_during_prediction,
        descendant_admitted_final,
        no_ancestor_rejection,
        descendant_validation_rejected,
        raw_audit_complete_repair,
        payload_preserved,
        proposal_repaired_pairs,
        proposal_candidate_count,
        proposal_unique_partitions,
        proposal_winner_margin,
        validation_error,
        admission_error,
        budget,
        budget_exact,
        invariants_hold,
        final_language_signature,
    })
}

fn capture_proposal_metrics(
    proof: &DescendantProof,
    repaired_pairs: &mut usize,
    candidate_count: &mut usize,
    unique_partitions: &mut usize,
    winner_margin: &mut usize,
) {
    *repaired_pairs = proof.repaired_pairs;
    *candidate_count = proof.descendant_candidate_program_count;
    *unique_partitions = proof.unique_partition_count;
    *winner_margin = proof.winner_margin;
}

fn audit_roots(roots: &[RootTask]) -> Result<RootAudit, Box<dyn Error>> {
    let mut audit = RootAudit {
        roots: roots.len(),
        correct_stage1_alias_defects_exact: 0,
        wrong_stage1_alias_defects_exact: 0,
        correct_stage1_search_exact: 0,
        wrong_stage1_search_exact: 0,
        l0_raw_audit_exact: 0,
        l0_descendant_empty_exact: 0,
        l1_alias_defects_exact: 0,
        l1_descendant_frontier_exact: 0,
    };

    for root in roots {
        let l0 = StateLanguage::new(root.root_id);
        let mut correct_defect_budget = GenesisBudget::default();
        if detect_alias_defects(
            &root.correct_ancestor_problem,
            &l0,
            &mut correct_defect_budget,
        )
        .map(|defects| defects.len() == EXPECTED_OMEGA1_ALIAS_DEFECTS)
        .unwrap_or(false)
        {
            audit.correct_stage1_alias_defects_exact += 1;
        }

        let mut wrong_defect_budget = GenesisBudget::default();
        if detect_alias_defects(
            &root.wrong_ancestor_problem,
            &l0,
            &mut wrong_defect_budget,
        )
        .map(|defects| defects.len() == EXPECTED_OMEGA1_ALIAS_DEFECTS)
        .unwrap_or(false)
        {
            audit.wrong_stage1_alias_defects_exact += 1;
        }

        let mut correct_ledger = BudgetLedger::default();
        let correct = prepare_ancestor(&root.correct_ancestor_problem, &mut correct_ledger)?;
        if omega1_search_ledger_exact(&correct_ledger)
            && ancestor_proof_exact(&correct.proof)
        {
            audit.correct_stage1_search_exact += 1;
        }

        let mut wrong_ledger = BudgetLedger::default();
        let wrong = prepare_ancestor(&root.wrong_ancestor_problem, &mut wrong_ledger)?;
        if omega1_search_ledger_exact(&wrong_ledger) && ancestor_proof_exact(&wrong.proof) {
            audit.wrong_stage1_search_exact += 1;
        }

        let mut raw_budget = DescendantBudget::default();
        let raw_audit = audit_raw_expressibility(&root.stage2_problem, &mut raw_budget)?;
        if raw_audit_exact(&raw_audit, &raw_budget) {
            audit.l0_raw_audit_exact += 1;
        }

        let mut no_ancestor_budget = DescendantBudget::default();
        let no_ancestor = synthesize_descendant(
            &root.stage2_problem,
            &l0,
            DescendantConfig::default(),
            &mut no_ancestor_budget,
        );
        if matches!(no_ancestor, Err(DescendantGenesisError::NoAncestorRefinement))
            && no_ancestor_budget_exact(&no_ancestor_budget)
        {
            audit.l0_descendant_empty_exact += 1;
        }

        let mut l1 = StateLanguage::new(root.root_id);
        l1.admit_certificate(&correct.certificate)?;
        let mut l1_defect_budget = GenesisBudget::default();
        if detect_alias_defects(&root.stage2_problem, &l1, &mut l1_defect_budget)
            .map(|defects| defects.len() == EXPECTED_L1_ALIAS_DEFECTS)
            .unwrap_or(false)
        {
            audit.l1_alias_defects_exact += 1;
        }

        let mut descendant_budget = DescendantBudget::default();
        let proof = synthesize_descendant(
            &root.stage2_problem,
            &l1,
            DescendantConfig::default(),
            &mut descendant_budget,
        )?;
        if descendant_proof_exact(&proof) && descendant_budget_exact(&descendant_budget) {
            audit.l1_descendant_frontier_exact += 1;
        }
    }

    Ok(audit)
}
