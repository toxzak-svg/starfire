fn evaluate_split(
    anchors: &[AnchorObservation],
    split_salt: u64,
    state: &ProbeState,
    judge: &RelativeImprovementJudge,
) -> Result<SplitEvaluation, Box<dyn Error>> {
    let mut intermediates = BTreeMap::<(u64, Candidate), IntermediateState>::new();
    let mut eligibility_resolver_calls = 0_usize;
    let mut eligibility_objective_evaluations = 0_usize;

    for anchor in anchors {
        for candidate in [Candidate::Reasoning, Candidate::Causal] {
            let first = perform_first_step(anchor, candidate, state, judge)?;
            eligibility_resolver_calls += 1;
            eligibility_objective_evaluations += 1;
            if let Some(runtime) = first {
                intermediates.insert(
                    (anchor.fixed_charge.id, candidate),
                    runtime.intermediate,
                );
            }
        }
    }

    let eligible_ids: Vec<u64> = anchors
        .iter()
        .map(|anchor| anchor.fixed_charge.id)
        .filter(|anchor_id| {
            intermediates.contains_key(&(*anchor_id, Candidate::Reasoning))
                && intermediates.contains_key(&(*anchor_id, Candidate::Causal))
        })
        .collect();
    let eligible_set: BTreeSet<u64> = eligible_ids.iter().copied().collect();

    let mut donor_maps = BTreeMap::<Word, BTreeMap<u64, IntermediateState>>::new();
    for word in Word::all() {
        donor_maps.insert(
            word,
            rewired_donors(&eligible_ids, &intermediates, word, split_salt)?,
        );
    }

    let mut rows = BTreeMap::<Word, Vec<PathRow>>::new();
    let mut composite_resolver_calls = 0_usize;
    let mut composite_objective_evaluations = 0_usize;

    for word in Word::all() {
        let donors = donor_maps
            .get(&word)
            .ok_or("missing rewired donor map")?;
        let mut word_rows = Vec::with_capacity(eligible_ids.len());
        for anchor in anchors
            .iter()
            .filter(|anchor| eligible_set.contains(&anchor.fixed_charge.id))
        {
            let donor = donors
                .get(&anchor.fixed_charge.id)
                .ok_or("missing rewired donor")?;
            let stateful = execute_word(
                anchor,
                word,
                ExecutionMode::Stateful,
                None,
                state,
                judge,
            )?;
            let blind = execute_word(
                anchor,
                word,
                ExecutionMode::Blind,
                None,
                state,
                judge,
            )?;
            let scalar = execute_word(
                anchor,
                word,
                ExecutionMode::ScalarState,
                None,
                state,
                judge,
            )?;
            let rewired = execute_word(
                anchor,
                word,
                ExecutionMode::Rewired,
                Some(donor),
                state,
                judge,
            )?;

            for execution in [&stateful, &blind, &scalar, &rewired] {
                if execution.resolver_calls != 2 || execution.objective_evaluations != 2 {
                    return Err("H8 matched path did not execute exactly two calls/evaluations".into());
                }
                composite_resolver_calls =
                    composite_resolver_calls.saturating_add(execution.resolver_calls);
                composite_objective_evaluations = composite_objective_evaluations
                    .saturating_add(execution.objective_evaluations);
            }

            word_rows.push(PathRow {
                anchor_id: anchor.fixed_charge.id,
                stateful_score: stateful.terminal_score,
                blind_score: blind.terminal_score,
                scalar_score: scalar.terminal_score,
                rewired_score: rewired.terminal_score,
                stateful_prompt_bytes: stateful.second_prompt_bytes,
                scalar_prompt_bytes: scalar.second_prompt_bytes,
                rewired_prompt_bytes: rewired.second_prompt_bytes,
            });
        }
        rows.insert(word, word_rows);
    }

    let expected_eligibility = 2 * anchors.len();
    let expected_composite = 16 * eligible_ids.len();
    if eligibility_resolver_calls != expected_eligibility
        || eligibility_objective_evaluations != expected_eligibility
        || composite_resolver_calls != expected_composite
        || composite_objective_evaluations != expected_composite
    {
        return Err(format!(
            "H8 budget mismatch within split: retained={} eligible={} eligibility_calls={} eligibility_evals={} composite_calls={} composite_evals={}",
            anchors.len(),
            eligible_ids.len(),
            eligibility_resolver_calls,
            eligibility_objective_evaluations,
            composite_resolver_calls,
            composite_objective_evaluations
        )
        .into());
    }

    Ok(SplitEvaluation {
        retained: anchors.len(),
        eligible: eligible_ids.len(),
        rows,
        eligibility_resolver_calls,
        eligibility_objective_evaluations,
        composite_resolver_calls,
        composite_objective_evaluations,
    })
}

fn perform_first_step(
    anchor: &AnchorObservation,
    candidate: Candidate,
    state: &ProbeState,
    judge: &RelativeImprovementJudge,
) -> Result<Option<FirstStepRuntime>, Box<dyn Error>> {
    let mut environment = TargetVerifierEnvironment::new(&anchor.task);
    let _ = environment.reset(common_root_seed(anchor.fixed_charge.id));
    let mut cycle = CognitiveCycleState::new();
    if !cycle.admit_charge(anchor.fixed_charge.clone()) {
        return Err("H8 first-step charge rejected".into());
    }

    let current_charge = cycle.pending()[0].clone();
    let output = resolve_component(candidate, &anchor.task.prompt, state);
    let before = environment.objective_feedback();
    let step = environment.act(&output);
    let after = environment.objective_feedback();
    let compute_cost = step.action_cost.max(1);
    let resolution = Resolution {
        discharged: current_charge.magnitude,
        emitted: vec![],
        permitted_decay: 0.0,
        compute_cost,
    };
    let witness = OutcomeWitness::new(
        "objective_progress",
        before.progress,
        after.progress,
        ImprovementDirection::HigherIsBetter,
        after.evidence,
    );
    let motion = directed_normalized_motion(&witness).ok_or("non-finite H8 first-step witness")?;
    let judged = judge.evaluate(&current_charge, &resolution, &witness);
    cycle
        .apply_judgment(0, &judged)
        .ok_or("H8 first-step judgment application failed")?;

    if cycle.pending().is_empty() {
        return Ok(None);
    }

    let remaining = cycle.pending()[0].magnitude;
    let magnitude = anchor.fixed_charge.magnitude;
    let accepted_fraction = if magnitude > 0.0 {
        judged.accepted as f64 / magnitude as f64
    } else {
        0.0
    };
    let unresolved_fraction = if magnitude > 0.0 {
        remaining as f64 / magnitude as f64
    } else {
        0.0
    };

    Ok(Some(FirstStepRuntime {
        environment,
        cycle,
        intermediate: IntermediateState {
            anchor_id: anchor.fixed_charge.id,
            first_output: output,
            signed_objective_motion: motion,
            accepted_fraction,
            unresolved_fraction,
        },
        compute_cost,
    }))
}

fn execute_word(
    anchor: &AnchorObservation,
    word: Word,
    mode: ExecutionMode,
    donor: Option<&IntermediateState>,
    state: &ProbeState,
    judge: &RelativeImprovementJudge,
) -> Result<Execution, Box<dyn Error>> {
    let first = perform_first_step(anchor, word.first(), state, judge)?
        .ok_or("H8 paired eligibility drifted during matched execution")?;
    let FirstStepRuntime {
        mut environment,
        mut cycle,
        intermediate,
        compute_cost: first_cost,
    } = first;

    let second_prompt = match mode {
        ExecutionMode::Stateful => continuation_prompt(&anchor.task, &intermediate, true),
        ExecutionMode::Blind => anchor.task.prompt.clone(),
        ExecutionMode::ScalarState => continuation_prompt(&anchor.task, &intermediate, false),
        ExecutionMode::Rewired => continuation_prompt(
            &anchor.task,
            donor.ok_or("rewired H8 path missing donor intermediate")?,
            true,
        ),
    };
    let second_prompt_bytes = second_prompt.len();
    let second_output = resolve_component(word.second(), &second_prompt, state);

    let current_charge = cycle.pending()[0].clone();
    let before = environment.objective_feedback();
    let step = environment.act(&second_output);
    let after = environment.objective_feedback();
    let terminal_score = after.progress;
    let second_cost = step.action_cost.max(1);
    let resolution = Resolution {
        discharged: current_charge.magnitude,
        emitted: vec![],
        permitted_decay: 0.0,
        compute_cost: second_cost,
    };
    let witness = OutcomeWitness::new(
        "objective_progress",
        before.progress,
        after.progress,
        ImprovementDirection::HigherIsBetter,
        after.evidence,
    );
    let judged = judge.evaluate(&current_charge, &resolution, &witness);
    cycle
        .apply_judgment(0, &judged)
        .ok_or("H8 second-step judgment application failed")?;

    Ok(Execution {
        terminal_score,
        compute_cost: first_cost.saturating_add(second_cost),
        resolver_calls: 2,
        objective_evaluations: 2,
        second_prompt_bytes,
    })
}

fn continuation_prompt(
    task: &ProbeTask,
    intermediate: &IntermediateState,
    include_output: bool,
) -> String {
    let first_output = if include_output {
        intermediate.first_output.as_str()
    } else {
        ""
    };
    format!(
        "{}\n\nHistorical computation on this same unresolved state:\nprior_output: {}\nprior_objective_motion: {:.6}\nprior_accepted_fraction: {:.6}\ncurrent_unresolved_fraction: {:.6}\nContinue resolving the original question from this historical state.",
        task.prompt,
        first_output,
        intermediate.signed_objective_motion,
        intermediate.accepted_fraction,
        intermediate.unresolved_fraction,
    )
}

fn rewired_donors(
    eligible_ids: &[u64],
    intermediates: &BTreeMap<(u64, Candidate), IntermediateState>,
    word: Word,
    split_salt: u64,
) -> Result<BTreeMap<u64, IntermediateState>, Box<dyn Error>> {
    let mut ids = eligible_ids.to_vec();
    ids.sort_unstable();
    if ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let shift = if ids.len() < 2 {
        0
    } else {
        ((SEED ^ split_salt ^ word.salt()) as usize % (ids.len() - 1)) + 1
    };
    let mut donors = BTreeMap::new();
    for (index, anchor_id) in ids.iter().enumerate() {
        let donor_id = ids[(index + shift) % ids.len()];
        let donor = intermediates
            .get(&(donor_id, word.first()))
            .ok_or("rewired donor intermediate missing")?
            .clone();
        donors.insert(*anchor_id, donor);
    }
    Ok(donors)
}
