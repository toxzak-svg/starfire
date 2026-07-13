fn main() -> Result<(), Box<dyn Error>> {
    let roots = build_roots()?;
    let train_end = TRAIN_FAMILIES * ROOTS_PER_FAMILY;
    let holdout_end = train_end + HOLDOUT_FAMILIES * ROOTS_PER_FAMILY;
    let train = &roots[..train_end];
    let holdout = &roots[train_end..holdout_end];
    let future = &roots[holdout_end..];

    let root_audit = audit_roots(&roots)?;
    let training = evaluate_split(train)?;
    let holdout_report = evaluate_split(holdout)?;
    let future_report = evaluate_split(future)?;

    let mut future_families = Vec::new();
    for family_index in 0..FUTURE_FAMILIES {
        let start = holdout_end + family_index * ROOTS_PER_FAMILY;
        let end = start + ROOTS_PER_FAMILY;
        let report = evaluate_split(&roots[start..end])?;
        let stateful = path_metrics(&report, PathKind::TransportDescendantChain);
        let stationary = path_metrics(&report, PathKind::StationaryAncestorDescendantChain);
        future_families.push(FutureFamilyReport {
            family: FAMILIES[TRAIN_FAMILIES + HOLDOUT_FAMILIES + family_index],
            roots: ROOTS_PER_FAMILY,
            stateful_success_rate: stateful.success_rate,
            stateful_correct_predictions: stateful.total_correct_predictions,
            stationary_success_rate: stationary.success_rate,
            stationary_correct_predictions: stationary.total_correct_predictions,
        });
    }

    let cohort_exact = train.len() == 16
        && holdout.len() == 8
        && future.len() == 32
        && future_families.len() == FUTURE_FAMILIES;
    let structural_audits_exact = root_audit.correct_ancestor_alias_defects_exact == roots.len()
        && root_audit.wrong_ancestor_alias_defects_exact == roots.len()
        && root_audit.correct_transport_frontier_exact == roots.len()
        && root_audit.wrong_transport_frontier_exact == roots.len()
        && root_audit.stationary_transport_frontier_exact == roots.len()
        && root_audit.l0_raw_audit_exact == roots.len()
        && root_audit.l0_descendant_empty_exact == roots.len()
        && root_audit.correct_descendant_frontier_exact == roots.len();
    let stateful_training = perfect_stateful(&training);
    let stateful_holdout = perfect_stateful(&holdout_report);
    let stateful_future = perfect_stateful(&future_report);
    let l0_raw_zero_predictions = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::L0RawSearch);
            metrics.raw_audit_complete_repairs == 0
                && metrics.total_correct_predictions == 0
                && metrics.full_successes == 0
        });
    let l0_descendant_empty_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::L0DescendantNoAncestor);
            metrics.no_ancestor_rejections == metrics.roots
                && metrics.descendant_proposal_successes == 0
                && metrics.total_correct_predictions == 0
        });
    let ancestor_payload_does_not_create_terminal = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::AncestorCertificatePayloadOnly);
            metrics.ancestor_payload_preservations == metrics.roots
                && metrics.no_ancestor_rejections == metrics.roots
                && metrics.correct_ancestor_admissions == 0
                && metrics.descendant_proposal_successes == 0
                && metrics.total_correct_predictions == 0
        });
    let wrong_transport_ancestor_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::WrongTransportCertifiedAncestor);
            metrics.wrong_ancestor_admissions == metrics.roots
                && metrics.descendant_proposal_successes == metrics.roots
                && metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let exact_ancestor_replacement_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(
                split,
                PathKind::ExactAncestorReplacedBeforeDescendantValidation,
            );
            metrics.correct_ancestor_admissions == metrics.roots
                && metrics.wrong_ancestor_admissions == metrics.roots
                && metrics.descendant_proposal_successes == metrics.roots
                && metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let stationary_chain_exact_half_transfer = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::StationaryAncestorDescendantChain);
            metrics.stationary_ancestor_admissions == metrics.roots
                && metrics.descendant_validation_successes == metrics.roots
                && metrics.descendant_admissions_during_window == metrics.roots
                && metrics.full_successes == 0
                && metrics.total_correct_predictions
                    == metrics.roots * EXPECTED_STATIONARY_CORRECT
        });
    let descendant_payload_only_inert_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::DescendantPayloadOnly);
            metrics.correct_ancestor_admissions == metrics.roots
                && metrics.descendant_validation_successes == metrics.roots
                && metrics.descendant_admissions_during_window == 0
                && metrics.descendant_final_admissions == 0
                && metrics.descendant_payload_preservations == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let counterfeit_descendant_rejected_everywhere = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::CounterfeitDescendantProof);
            metrics.correct_ancestor_admissions == metrics.roots
                && metrics.descendant_validation_successes == 0
                && metrics.descendant_validation_rejections == metrics.roots
                && metrics.total_correct_predictions == 0
        });
    let delayed_zero_during_window = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            path_metrics(split, PathKind::DelayedDescendantAdmission).total_correct_predictions == 0
        });
    let delayed_eventual_admission = [&training, &holdout_report, &future_report]
        .into_iter()
        .all(|split| {
            let metrics = path_metrics(split, PathKind::DelayedDescendantAdmission);
            metrics.descendant_admissions_during_window == 0
                && metrics.descendant_final_admissions == metrics.roots
        });
    let all_future_families_transfer = future_families.iter().all(|family| {
        family.stateful_success_rate == 1.0
            && family.stateful_correct_predictions == ROOTS_PER_FAMILY * PREDICTIONS_PER_ROOT
            && family.stationary_success_rate == 0.0
            && family.stationary_correct_predictions
                == ROOTS_PER_FAMILY * EXPECTED_STATIONARY_CORRECT
    });
    let budgets_exact = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.budgets_exact);
    let replay_exact = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.replay_exact);
    let invariants_hold = [&training, &holdout_report, &future_report]
        .into_iter()
        .flat_map(|split| split.paths.values())
        .all(|metrics| metrics.invariants_hold);

    let gates = GateReport {
        cohort_exact,
        structural_audits_exact,
        stateful_training,
        stateful_holdout,
        stateful_future,
        l0_raw_zero_predictions,
        l0_descendant_empty_everywhere,
        ancestor_payload_does_not_create_terminal,
        wrong_transport_ancestor_rejected_everywhere,
        exact_ancestor_replacement_rejected_everywhere,
        stationary_chain_exact_half_transfer,
        descendant_payload_only_inert_everywhere,
        counterfeit_descendant_rejected_everywhere,
        delayed_zero_during_window,
        delayed_eventual_admission,
        all_future_families_transfer,
        budgets_exact,
        replay_exact,
        invariants_hold,
    };

    let terminal_classification = if !budgets_exact
        || !invariants_hold
        || !exact_ancestor_replacement_rejected_everywhere
    {
        "CONTROL_FAILURE"
    } else if !replay_exact {
        "REPLAY_FAILURE"
    } else if gates.all_pass() {
        "PASS"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "ΩD1 transport-certified descendant necessity",
        mechanism: "an independently validated and admitted ΩR1 transport-stable ancestor creates the sole executable ancestor terminal for a bounded descendant grammar; a proof-carrying descendant certificate is bound to the exact transport-certified ancestor-state signature and must be admitted before held-out cumulative capability exists",
        claim_boundary: "a PASS supports bounded two-generation executable representation growth under the frozen symbolic pair-block regime with transport-certified ancestor identity; it does not establish automatic transformation discovery, learned grammar invention, arbitrary-depth recursion, AGI, consciousness, or human-level cognition",
        frozen_contract: FrozenContract {
            preregistration_commit: PREREGISTRATION_COMMIT,
            train_roots: 16,
            holdout_roots: 8,
            future_roots: 32,
            future_families: FUTURE_FAMILIES,
            raw_atoms: RAW_ATOMS,
            discovery_histories: DISCOVERY_HISTORIES,
            heldout_transformations: HELDOUT_TRANSFORMATIONS,
            predictions_per_root: PREDICTIONS_PER_ROOT,
            alias_defects: EXPECTED_ALIAS_DEFECTS,
            raw_candidates: EXPECTED_RAW_CANDIDATES,
            raw_evaluations: EXPECTED_RAW_EVALUATIONS,
            raw_unique_partitions: EXPECTED_RAW_UNIQUE_PARTITIONS,
            raw_best_repair: EXPECTED_RAW_BEST_REPAIR,
            ancestor_winner: EXPECTED_ANCESTOR_WINNER,
            ancestor_runner_up: EXPECTED_ANCESTOR_RUNNER_UP,
            ancestor_margin: EXPECTED_ANCESTOR_MARGIN,
            ancestor_support: EXPECTED_ANCESTOR_SUPPORT,
            ancestor_winning_class: EXPECTED_WINNING_CLASS,
            transport_evaluations: EXPECTED_TRANSPORT_EVALUATIONS,
            correct_zero_violation_representatives: EXPECTED_CORRECT_ZERO_VIOLATION,
            wrong_zero_violation_representatives: EXPECTED_WRONG_ZERO_VIOLATION,
            stationary_zero_violation_representatives: EXPECTED_STATIONARY_ZERO_VIOLATION,
            descendant_candidates: EXPECTED_DESCENDANT_CANDIDATES,
            descendant_evaluations: EXPECTED_DESCENDANT_EVALUATIONS,
            descendant_unique_partitions: EXPECTED_DESCENDANT_UNIQUE_PARTITIONS,
            descendant_winner: EXPECTED_DESCENDANT_WINNER,
            descendant_runner_up: EXPECTED_DESCENDANT_RUNNER_UP,
            descendant_margin: EXPECTED_DESCENDANT_MARGIN,
            descendant_support: EXPECTED_DESCENDANT_SUPPORT,
            paths_per_root: PathKind::all().len(),
        },
        root_audit,
        training,
        holdout: holdout_report,
        future: future_report,
        future_families,
        gates,
        terminal_classification,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal_classification != "PASS" {
        std::process::exit(1);
    }
    Ok(())
}
