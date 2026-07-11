fn main() -> Result<(), Box<dyn Error>> {
    let state = ProbeState::new();
    let judge = RelativeImprovementJudge;
    let fixed_config = FixedResidualProjectionConfig::default();
    let h4_predicate = h4_memory_predicate();
    let mut next_id = 1_u64;
    let mut excluded = 0_usize;
    let mut retained = 0_usize;
    let mut excluded_hidden_distribution = BTreeMap::<String, usize>::new();
    let mut retained_hidden_distribution = BTreeMap::<String, usize>::new();
    let mut windows = Vec::<Vec<AnchorObservation>>::new();

    for family in FAMILIES {
        let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
        for class in EventClass::all() {
            for repeat in 0..REPEATS_PER_CLASS {
                let task = surface_task(family, class, repeat);
                let mut charge = emit_real_charge(&task)?;
                charge.kind = ChargeKind::Custom("unresolved".into());
                charge.id = next_id;
                next_id = next_id.saturating_add(1);

                let variable_charge = ontology_feature_charge(&charge);
                let fixed_charge = fixed_residual_feature_charge(&charge, fixed_config);
                if h4_predicate.matches(&variable_charge) {
                    excluded += 1;
                    *excluded_hidden_distribution
                        .entry(class.name().to_string())
                        .or_default() += 1;
                    continue;
                }

                retained += 1;
                *retained_hidden_distribution
                    .entry(class.name().to_string())
                    .or_default() += 1;
                window.push(AnchorObservation {
                    fixed_charge,
                    task,
                    hidden: class,
                });
            }
        }
        windows.push(window);
    }

    let total_real_emitter_observations =
        (TRAIN_WINDOWS + HOLDOUT_WINDOWS + TRANSFER_WINDOWS) * OBSERVATIONS_PER_WINDOW;
    let train = flatten_anchors(&windows[..TRAIN_WINDOWS]);
    let holdout = flatten_anchors(&windows[TRAIN_WINDOWS..TRAIN_WINDOWS + HOLDOUT_WINDOWS]);
    let future_windows = &windows[TRAIN_WINDOWS + HOLDOUT_WINDOWS..];
    let future = flatten_anchors(future_windows);

    if total_real_emitter_observations != 252
        || excluded != 84
        || retained != 168
        || train.len() != 48
        || holdout.len() != 24
        || future.len() != 96
    {
        return Err(format!(
            "frozen H8 cohort mismatch: total={total_real_emitter_observations} excluded={excluded} retained={retained} train={} holdout={} future={}",
            train.len(),
            holdout.len(),
            future.len()
        )
        .into());
    }

    let train_eval = evaluate_split(&train, 0x11, &state, &judge)?;
    let train_rc = word_metrics(&train_eval, Word::ReasoningThenCausal);
    let train_cr = word_metrics(&train_eval, Word::CausalThenReasoning);
    let candidate = if train_rc.mean_stateful_score >= train_cr.mean_stateful_score {
        Word::ReasoningThenCausal
    } else {
        Word::CausalThenReasoning
    };
    let reverse = candidate.reverse();

    let holdout_eval = evaluate_split(&holdout, 0x22, &state, &judge)?;
    let mut future_evals = Vec::with_capacity(TRANSFER_WINDOWS);
    for (index, window) in future_windows.iter().enumerate() {
        future_evals.push(evaluate_split(
            window,
            0x100_u64.saturating_add(index as u64),
            &state,
            &judge,
        )?);
    }
    let future_eval = combine_evaluations(&future_evals);

    let training_candidate = word_metrics(&train_eval, candidate);
    let training_reverse = word_metrics(&train_eval, reverse);
    let holdout_candidate = word_metrics(&holdout_eval, candidate);
    let holdout_reverse = word_metrics(&holdout_eval, reverse);
    let future_candidate = word_metrics(&future_eval, candidate);
    let future_reverse = word_metrics(&future_eval, reverse);

    let train_order_advantage =
        training_candidate.mean_stateful_score - training_reverse.mean_stateful_score;
    let holdout_order_advantage =
        holdout_candidate.mean_stateful_score - holdout_reverse.mean_stateful_score;
    let future_order_advantage =
        future_candidate.mean_stateful_score - future_reverse.mean_stateful_score;

    let mut future_window_reports = Vec::with_capacity(TRANSFER_WINDOWS);
    for (index, evaluation) in future_evals.iter().enumerate() {
        let candidate_metrics = word_metrics(evaluation, candidate);
        let reverse_metrics = word_metrics(evaluation, reverse);
        let composition_gain = candidate_metrics.composition_gain;
        future_window_reports.push(FutureWindowReport {
            index,
            family: FAMILIES[TRAIN_WINDOWS + HOLDOUT_WINDOWS + index].name,
            eligible: evaluation.eligible,
            candidate_stateful_score: candidate_metrics.mean_stateful_score,
            candidate_blind_score: candidate_metrics.mean_blind_score,
            composition_gain,
            order_advantage: candidate_metrics.mean_stateful_score
                - reverse_metrics.mean_stateful_score,
            win: composition_gain > 1e-12,
        });
    }
    let future_window_wins = future_window_reports.iter().filter(|window| window.win).count();
    let worst_family_gain = future_window_reports
        .iter()
        .map(|window| window.composition_gain)
        .fold(f64::INFINITY, f64::min);

    let train_support = train_eval.eligible >= MIN_TRAIN_ELIGIBLE;
    let holdout_support = holdout_eval.eligible >= MIN_HOLDOUT_ELIGIBLE;
    let future_support = future_eval.eligible >= MIN_FUTURE_ELIGIBLE;
    let train_gain_after_penalty = training_candidate.composition_gain - COMPLEXITY_PENALTY
        >= MIN_TRAIN_GAIN_AFTER_PENALTY;
    let train_order_gate = train_order_advantage >= MIN_TRAIN_ORDER_ADVANTAGE;
    let holdout_gain = holdout_candidate.composition_gain >= MIN_HOLDOUT_GAIN;
    let holdout_order_gate = holdout_order_advantage >= MIN_HOLDOUT_ORDER_ADVANTAGE;
    let holdout_positive_fraction =
        holdout_candidate.positive_fraction >= MIN_HOLDOUT_POSITIVE_FRACTION;
    let holdout_right_absorption =
        holdout_candidate.right_absorption_rate <= MAX_HOLDOUT_RIGHT_ABSORPTION;
    let future_gain = future_candidate.composition_gain >= MIN_FUTURE_GAIN;
    let future_order_gate = future_order_advantage >= MIN_FUTURE_ORDER_ADVANTAGE;
    let all_future_windows_win = future_window_wins >= MIN_FUTURE_WINDOW_WINS;
    let worst_family_gate = worst_family_gain >= MIN_WORST_FAMILY_GAIN;
    let future_right_absorption =
        future_candidate.right_absorption_rate <= MAX_FUTURE_RIGHT_ABSORPTION;
    let rewired_margin = future_candidate.rewired_margin >= MIN_REWIRED_MARGIN;
    let scalar_margin = future_candidate.scalar_margin >= MIN_SCALAR_MARGIN;

    let budget = aggregate_budget(&train_eval, &holdout_eval, &future_eval);
    let gates = GateReport {
        train_support,
        holdout_support,
        future_support,
        train_gain_after_penalty,
        train_order_advantage: train_order_gate,
        holdout_gain,
        holdout_order_advantage: holdout_order_gate,
        holdout_positive_fraction,
        holdout_right_absorption,
        future_gain,
        future_order_advantage: future_order_gate,
        all_future_windows_win,
        worst_family_gain: worst_family_gate,
        future_right_absorption,
        rewired_margin,
        scalar_margin,
        budget_exact: budget.budget_exact,
    };

    let promotion_passed = train_support
        && holdout_support
        && train_gain_after_penalty
        && train_order_gate
        && holdout_gain
        && holdout_order_gate
        && holdout_positive_fraction
        && holdout_right_absorption
        && budget.budget_exact;

    let primary_transfer_gates = train_support
        && holdout_support
        && future_support
        && train_gain_after_penalty
        && train_order_gate
        && holdout_gain
        && holdout_order_gate
        && holdout_positive_fraction
        && holdout_right_absorption
        && future_gain
        && future_order_gate
        && all_future_windows_win
        && worst_family_gate
        && future_right_absorption
        && budget.budget_exact;

    let terminal_classification = if gates.all_pass() {
        "PASS"
    } else if !train_support || !holdout_support || !future_support {
        "NOT_COMPOSABLE"
    } else if (holdout_candidate.composition_gain > 0.0 || future_candidate.composition_gain > 0.0)
        && (!holdout_right_absorption || !future_right_absorption)
    {
        "RIGHT_ABSORPTION"
    } else if primary_transfer_gates && (!rewired_margin || !scalar_margin) {
        "CONTROL_FAILURE"
    } else if holdout_gain
        && future_gain
        && all_future_windows_win
        && (!holdout_order_gate || !future_order_gate)
    {
        "ORDER_UNSTABLE"
    } else if future_gain && all_future_windows_win && !promotion_passed {
        "TRANSFER_ONLY"
    } else {
        "REJECTED"
    };

    let report = Report {
        experiment: "H8 transformed action-order diamond",
        observation_source: "real Starfire subsystem CHARGE -> actual first component output -> independent objective witness -> RelativeImprovementJudge -> explicit same-root IntermediateState -> actual second component -> terminal independent objective measurement",
        visible_charge_kind: "Custom(unresolved)",
        representation_boundary: "frozen H4 non-memory exclusion followed by an H8 execution path that receives no residual coordinate, hidden event class, task-family label, target answer, resolver leader, or future outcome",
        operation_under_test: "whether the actual first operation output plus witnessed intermediate unresolved state changes the behavior of the actual second Starfire component in an order-specific way beyond same-word blind, scalar-history, and common-root-rewired controls",
        frozen_contract: frozen_contract(),
        cohort: CohortReport {
            total_real_emitter_observations,
            excluded_by_frozen_h4_memory_predicate: excluded,
            retained_non_memory: retained,
            train_non_memory: train.len(),
            holdout_non_memory: holdout.len(),
            future_non_memory: future.len(),
            excluded_hidden_distribution,
            retained_hidden_distribution,
        },
        candidate_word: candidate.name(),
        promoted_word: promotion_passed.then_some(candidate.name()),
        training: split_report(&train_eval, candidate),
        holdout: split_report(&holdout_eval, candidate),
        future: split_report(&future_eval, candidate),
        future_windows: future_window_reports,
        future_window_wins,
        worst_family_gain,
        budget,
        gates,
        terminal_classification,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn frozen_contract() -> FrozenContract {
    FrozenContract {
        seed: SEED,
        proposal_budget: PROPOSAL_BUDGET,
        complexity_penalty: COMPLEXITY_PENALTY,
        min_train_eligible: MIN_TRAIN_ELIGIBLE,
        min_holdout_eligible: MIN_HOLDOUT_ELIGIBLE,
        min_future_eligible: MIN_FUTURE_ELIGIBLE,
        min_train_gain_after_penalty: MIN_TRAIN_GAIN_AFTER_PENALTY,
        min_train_order_advantage: MIN_TRAIN_ORDER_ADVANTAGE,
        min_holdout_gain: MIN_HOLDOUT_GAIN,
        min_holdout_order_advantage: MIN_HOLDOUT_ORDER_ADVANTAGE,
        min_holdout_positive_fraction: MIN_HOLDOUT_POSITIVE_FRACTION,
        max_holdout_right_absorption: MAX_HOLDOUT_RIGHT_ABSORPTION,
        min_future_gain: MIN_FUTURE_GAIN,
        min_future_order_advantage: MIN_FUTURE_ORDER_ADVANTAGE,
        min_future_window_wins: MIN_FUTURE_WINDOW_WINS,
        min_worst_family_gain: MIN_WORST_FAMILY_GAIN,
        max_future_right_absorption: MAX_FUTURE_RIGHT_ABSORPTION,
        min_rewired_margin: MIN_REWIRED_MARGIN,
        min_scalar_margin: MIN_SCALAR_MARGIN,
    }
}

fn h4_memory_predicate() -> ConceptPredicate {
    ConceptPredicate::ResidualThreshold {
        dimension: 2,
        threshold: 0.171875,
        direction: Direction::AtMost,
    }
}
