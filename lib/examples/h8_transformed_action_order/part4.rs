fn word_metrics(evaluation: &SplitEvaluation, word: Word) -> WordMetrics {
    let rows = evaluation.rows.get(&word).map(Vec::as_slice).unwrap_or(&[]);
    let mean_stateful_score = mean(rows.iter().map(|row| row.stateful_score));
    let mean_blind_score = mean(rows.iter().map(|row| row.blind_score));
    let mean_scalar_score = mean(rows.iter().map(|row| row.scalar_score));
    let mean_rewired_score = mean(rows.iter().map(|row| row.rewired_score));
    let positive_fraction = rows
        .iter()
        .filter(|row| row.stateful_score > row.blind_score + 1e-12)
        .count() as f64
        / rows.len().max(1) as f64;
    let right_absorption_rate = rows
        .iter()
        .filter(|row| row.stateful_score.to_bits() == row.blind_score.to_bits())
        .count() as f64
        / rows.len().max(1) as f64;

    WordMetrics {
        word: word.name(),
        eligible: rows.len(),
        mean_stateful_score,
        mean_blind_score,
        composition_gain: mean_stateful_score - mean_blind_score,
        mean_scalar_score,
        scalar_margin: mean_stateful_score - mean_scalar_score,
        mean_rewired_score,
        rewired_margin: mean_stateful_score - mean_rewired_score,
        positive_fraction,
        right_absorption_rate,
        mean_stateful_prompt_bytes: mean(
            rows.iter()
                .map(|row| row.stateful_prompt_bytes as f64),
        ),
        mean_scalar_prompt_bytes: mean(rows.iter().map(|row| row.scalar_prompt_bytes as f64)),
        mean_rewired_prompt_bytes: mean(
            rows.iter()
                .map(|row| row.rewired_prompt_bytes as f64),
        ),
    }
}

fn split_report(evaluation: &SplitEvaluation, candidate: Word) -> SplitMetricsReport {
    let candidate_metrics = word_metrics(evaluation, candidate);
    let reverse_metrics = word_metrics(evaluation, candidate.reverse());
    SplitMetricsReport {
        retained: evaluation.retained,
        eligible: evaluation.eligible,
        candidate_order_advantage: candidate_metrics.mean_stateful_score
            - reverse_metrics.mean_stateful_score,
        words: Word::all()
            .into_iter()
            .map(|word| word_metrics(evaluation, word))
            .collect(),
    }
}

fn combine_evaluations(evaluations: &[SplitEvaluation]) -> SplitEvaluation {
    let mut rows = BTreeMap::<Word, Vec<PathRow>>::new();
    for word in Word::all() {
        let mut combined = Vec::new();
        for evaluation in evaluations {
            if let Some(word_rows) = evaluation.rows.get(&word) {
                combined.extend(word_rows.iter().cloned());
            }
        }
        rows.insert(word, combined);
    }
    SplitEvaluation {
        retained: evaluations.iter().map(|evaluation| evaluation.retained).sum(),
        eligible: evaluations.iter().map(|evaluation| evaluation.eligible).sum(),
        rows,
        eligibility_resolver_calls: evaluations
            .iter()
            .map(|evaluation| evaluation.eligibility_resolver_calls)
            .sum(),
        eligibility_objective_evaluations: evaluations
            .iter()
            .map(|evaluation| evaluation.eligibility_objective_evaluations)
            .sum(),
        composite_resolver_calls: evaluations
            .iter()
            .map(|evaluation| evaluation.composite_resolver_calls)
            .sum(),
        composite_objective_evaluations: evaluations
            .iter()
            .map(|evaluation| evaluation.composite_objective_evaluations)
            .sum(),
    }
}

fn aggregate_budget(
    train: &SplitEvaluation,
    holdout: &SplitEvaluation,
    future: &SplitEvaluation,
) -> BudgetReport {
    let retained = train.retained + holdout.retained + future.retained;
    let eligible = train.eligible + holdout.eligible + future.eligible;
    let eligibility_resolver_calls = train.eligibility_resolver_calls
        + holdout.eligibility_resolver_calls
        + future.eligibility_resolver_calls;
    let eligibility_objective_evaluations = train.eligibility_objective_evaluations
        + holdout.eligibility_objective_evaluations
        + future.eligibility_objective_evaluations;
    let composite_resolver_calls = train.composite_resolver_calls
        + holdout.composite_resolver_calls
        + future.composite_resolver_calls;
    let composite_objective_evaluations = train.composite_objective_evaluations
        + holdout.composite_objective_evaluations
        + future.composite_objective_evaluations;
    let expected_eligibility_resolver_calls = 2 * retained;
    let expected_eligibility_objective_evaluations = 2 * retained;
    let expected_composite_resolver_calls = 16 * eligible;
    let expected_composite_objective_evaluations = 16 * eligible;
    let budget_exact = eligibility_resolver_calls == expected_eligibility_resolver_calls
        && eligibility_objective_evaluations == expected_eligibility_objective_evaluations
        && composite_resolver_calls == expected_composite_resolver_calls
        && composite_objective_evaluations == expected_composite_objective_evaluations;

    BudgetReport {
        eligibility_resolver_calls,
        eligibility_objective_evaluations,
        composite_resolver_calls,
        composite_objective_evaluations,
        expected_eligibility_resolver_calls,
        expected_eligibility_objective_evaluations,
        expected_composite_resolver_calls,
        expected_composite_objective_evaluations,
        budget_exact,
    }
}

fn resolve_component(candidate: Candidate, prompt: &str, state: &ProbeState) -> String {
    match candidate {
        Candidate::Reasoning => {
            let mut engine = ReasoningEngine::new();
            let result = engine.reason(prompt, &state.reasoning_memories);
            let mut output = result.answer.unwrap_or_default();
            if !result.reasoning_chain.is_empty() {
                output.push(' ');
                output.push_str(&result.reasoning_chain.join(" "));
            }
            output
        }
        Candidate::Causal => resolve_causal(prompt),
    }
}

fn resolve_causal(prompt: &str) -> String {
    let mut engine = CausalEngine::new();
    for family in FAMILIES {
        engine.add_edge(
            family.trajectory_topic,
            family.trajectory_effect,
            0.9,
            Some(1),
        );
    }
    let prompt_tokens = token_set(prompt);
    let mut ranked: Vec<(usize, String)> = engine
        .edges()
        .values()
        .map(|edge| {
            let text = format!("{} causes {}.", edge.cause, edge.effect);
            let overlap = token_set(&text).intersection(&prompt_tokens).count();
            (overlap, text)
        })
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    ranked
        .into_iter()
        .filter(|(overlap, _)| *overlap > 0)
        .take(2)
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn surface_task(family: TaskFamily, class: EventClass, repeat: usize) -> ProbeTask {
    let prefix = SURFACE_PREFIXES[repeat];
    let suffix = SURFACE_SUFFIXES[repeat];
    let decorate = |text: &str| format!("{prefix} {text} {suffix}");
    match class {
        EventClass::KnowledgeGap => ProbeTask {
            class,
            topic: family.gap_topic.to_string(),
            prompt: decorate(family.gap_prompt),
            target: family.gap_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::PredictionContradiction => ProbeTask {
            class,
            topic: family.contradiction_topic.to_string(),
            prompt: decorate(family.contradiction_prompt),
            target: family.contradiction_target.to_string(),
            lead_a: String::new(),
            lead_b: String::new(),
        },
        EventClass::QuanotTrajectory => ProbeTask {
            class,
            topic: family.trajectory_topic.to_string(),
            prompt: decorate(family.trajectory_prompt),
            target: family.trajectory_target.to_string(),
            lead_a: format!("{prefix} {}", family.lead_a),
            lead_b: format!("{} {suffix}", family.lead_b),
        },
    }
}

fn emit_real_charge(task: &ProbeTask) -> Result<Charge, Box<dyn Error>> {
    match task.class {
        EventClass::KnowledgeGap => {
            let mut metacog = MetaCognition::new();
            metacog.note_gap(KnowledgeGap::new(&task.topic, 0.85));
            let gap = metacog
                .top_gap()
                .ok_or("metacognition did not retain gap")?;
            knowledge_gap_charge(gap).ok_or_else(|| "gap emitter returned no charge".into())
        }
        EventClass::PredictionContradiction => {
            let mut quanot = Quanot::new(32, 64);
            let state = quanot.process(&task.prompt);
            let mut center = PredictionCenter::new();
            let context = conversation_context(task, Some(state.reservoir_state));
            let predictions = center.generate(&context);
            let prediction = predictions
                .first()
                .ok_or("prediction center emitted no prediction")?;
            let charge = prediction_contradiction_charge(
                prediction,
                PredictionOutcome::Refuted,
                &task.target,
            )
            .ok_or("contradiction emitter returned no charge")?;
            center.update_with_evidence(&Evidence {
                outcome: PredictionOutcome::Refuted,
                prediction_id: prediction.id,
            });
            Ok(charge)
        }
        EventClass::QuanotTrajectory => {
            let mut quanot = Quanot::new(32, 64);
            let mut emitter = QuanotTrajectoryEmitter::new();
            let first = quanot.process(&task.lead_a);
            let second = quanot.process(&task.lead_b);
            let third = quanot.process(&task.prompt);
            let _ = emitter.observe(&first);
            let _ = emitter.observe(&second);
            emitter
                .observe(&third)
                .ok_or_else(|| "Quanot trajectory emitter returned no charge".into())
        }
    }
}

fn conversation_context(task: &ProbeTask, quanot_state: Option<Vec<f64>>) -> ConversationContext {
    let mut context = ConversationContext::new(task.topic.clone(), 2, quanot_state, Some(0.5));
    context.recent_text = vec![task.prompt.clone()];
    context.discussed_entities = token_set(&task.prompt).into_iter().take(8).collect();
    context
}

fn token_set(text: &str) -> BTreeSet<String> {
    const STOPWORDS: [&str; 25] = [
        "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "for", "from", "in", "is",
        "it", "of", "on", "only", "the", "to", "used", "what", "which", "why", "with",
    ];
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '\'')
        .map(|token| token.trim_matches('\'').to_ascii_lowercase())
        .filter(|token| token.len() > 1 && !STOPWORDS.contains(&token.as_str()))
        .collect()
}

fn flatten_anchors(windows: &[Vec<AnchorObservation>]) -> Vec<AnchorObservation> {
    windows
        .iter()
        .flat_map(|window| window.iter().cloned())
        .collect()
}

fn common_root_seed(anchor_id: u64) -> u64 {
    SEED ^ anchor_id
}

fn mean<I>(values: I) -> f64
where
    I: IntoIterator<Item = f64>,
{
    let mut total = 0.0;
    let mut count = 0_usize;
    for value in values {
        total += value;
        count += 1;
    }
    total / count.max(1) as f64
}
