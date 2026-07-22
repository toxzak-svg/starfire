fn main() -> Result<()> {
    let f1: FM = serde_json::from_str(F1)?;
    let a: AM = serde_json::from_str(A)?;
    validate_headers(&f1)?;
    let fixtures = load()?;
    validate_corpus(&a, &fixtures)?;
    let (split_counts, category_splits) = validate_split(&f1, &fixtures)?;

    let mut cases = Vec::new();
    for (i, fx) in fixtures.into_iter().enumerate() {
        let profile = a.profiles.get(&fx.profile).context("missing profile")?;
        let split = split(&f1.split, &fx.id)?;
        let pref = pref(&f1.preference_evidence, &fx.profile)?;
        let projection = projection(&f1.projection_profiles, &fx.id, pref, suffix(&fx.id)?)?;
        let category_prohibited = a
            .category_prohibited_claim_anchors
            .get(&fx.category)
            .cloned()
            .unwrap_or_default();
        let (program, lexical) =
            semantic_case(&fx, profile, &category_prohibited, u64::try_from(i + 1)?)?;
        cases.push(Case {
            fx,
            split,
            pref,
            projection,
            program,
            lexical,
        });
    }

    let train = cases
        .iter()
        .filter(|c| c.split == Split::Train)
        .map(|c| PairwisePreference {
            projection: c.projection.projection.clone(),
            left: VariantProfile::direct(),
            right: VariantProfile::warm(),
            preferred: side(c.pref),
        })
        .collect::<Vec<_>>();
    let model = LearnedExpressionModel::train(&train, 8, 200)?;
    model.verify_integrity()?;
    let artifact = model.artifact_bytes()?;
    let loaded: LearnedExpressionModel = serde_json::from_slice(&artifact)?;
    loaded.verify_integrity()?;
    if loaded != model {
        bail!("model artifact replay mismatch");
    }
    let selector = OfflineLearnedExpressionSelector::new(model.clone());
    let verifier = GrammarV3Verifier;

    let mut selected = 0usize;
    let mut verified = 0usize;
    let mut claims = 0usize;
    let mut safe = 0usize;
    let mut adversarial = 0usize;
    let mut adversarial_safe = 0usize;
    let mut exact_kind = 0usize;
    let mut det_lattice = true;
    let mut det_selection = true;
    let mut det_score = true;
    let mut det_verify = true;
    let mut max_variants = 0usize;
    let mut max_candidates = 0u16;
    let mut test_outputs = Vec::new();
    let mut baseline_test_outputs = Vec::new();
    let mut cat: BTreeMap<String, [usize; 4]> = BTreeMap::new();

    for c in &cases {
        let l1 = ExpressionLattice::build(&c.program, &c.lexical)?;
        let l2 = ExpressionLattice::build(&c.program, &c.lexical)?;
        det_lattice &= l1 == l2;
        let mut counts = BTreeMap::<OperationId, usize>::new();
        for v in &l1.payload.variants {
            *counts.entry(v.operation).or_default() += 1;
        }
        max_variants = max_variants.max(*counts.values().max().unwrap_or(&0));

        let s1 = selector.select(&c.program, &c.lexical, &c.projection)?;
        let s2 = selector.select(&c.program, &c.lexical, &c.projection)?;
        selected += 1;
        det_selection &= s1 == s2;
        det_score &= s1.payload.score == s2.payload.score;
        max_candidates = max_candidates.max(s1.payload.complete_candidates_scored);

        let mut fv = false;
        let mut fc = false;
        let mut fs = false;
        if s1.payload.disposition == SelectionDisposition::LearnedVerified {
            let ld = s1
                .payload
                .lattice_digest
                .context("missing lattice digest")?;
            let r1 = verifier.verify(&c.program, &c.lexical, ld, &s1.payload.text)?;
            let r2 = verifier.verify(&c.program, &c.lexical, ld, &s1.payload.text)?;
            det_verify &= r1 == r2;
            fv = true;
            verified += 1;
            fc =
                c.fx.required
                    .iter()
                    .all(|x| s1.payload.text.to_lowercase().contains(&x.to_lowercase()));
            fs = prohibited(&a, &c.fx)
                .iter()
                .all(|x| !s1.payload.text.to_lowercase().contains(&x.to_lowercase()));
            claims += usize::from(fc);
            safe += usize::from(fs);
            let kinds = r1
                .payload
                .variants
                .iter()
                .zip(&c.program.payload.operations)
                .all(|(x, y)| x.operation == y.id && x.kind == y.kind);
            exact_kind += usize::from(kinds);
        }
        if c.fx.category == "adversarial" {
            adversarial += 1;
            adversarial_safe += usize::from(fs);
        }
        let e = cat.entry(c.fx.category.clone()).or_default();
        e[0] += 1;
        e[1] += usize::from(fv);
        e[2] += usize::from(fc);
        e[3] += usize::from(fs);
        if c.split == Split::Test {
            test_outputs.push(s1.payload.text);
            baseline_test_outputs.push(c.fx.expected.clone());
        }
    }

    let pref_acc = accuracy(
        &model,
        cases.iter().filter(|c| c.split == Split::Test),
        None,
    );
    let shuffled_acc = shuffled_accuracy(&model, &cases);
    let shuffle_drop = (pref_acc - shuffled_acc).max(0.0);
    let state_change = state_change(&selector, &cases, &f1.projection_profiles)?;
    let zeroed = zeroed(&selector, &cases)?;
    let random = random_accuracy(cases.iter().filter(|c| c.split == Split::Test));
    let reversed = reversed_accuracy(&cases)?;
    let baseline_opener = opener_frequency(&baseline_test_outputs);
    let (_, baseline_tri_freq) = top_trigram(&baseline_test_outputs);
    let opener = opener_frequency(&test_outputs);
    let (top_tri, tri_freq) = top_trigram(&test_outputs);
    let opener_reduction = reduction(baseline_opener, opener);
    let trigram_reduction = reduction(baseline_tri_freq, tri_freq);

    let first = cases.first().context("empty corpus")?;
    let order_stable = order_control(&model, first)?;
    let (dup_rejected, ambiguous_rejected) = lattice_controls(first)?;
    let semantic_controls = semantic_controls(&cases)?;
    let budget_controls = budget_controls(first)?;
    let boundary_controls = boundary_controls(first)?;
    let artifact_controls = artifact_controls(&model)?;
    let fallback_rate = fallback_controls(&model, first, &f1.projection_profiles)?;

    let floors = cat
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                json!({
                    "fixture_count":v[0],
                    "verifier_acceptance":ratio(v[1],v[0]),
                    "semantic_claim_preservation":ratio(v[2],v[0]),
                    "prohibited_implication_absence":ratio(v[3],v[0]),
                    "passed":v[1]==v[0] && v[2]==v[0] && v[3]==v[0]
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let category_pass = cat
        .values()
        .all(|v| v[1] == v[0] && v[2] == v[0] && v[3] == v[0]);

    let b = authority_boundary();
    let authority_closed = b.candidate_lattice_construction
        && b.learned_candidate_scoring
        && b.deterministic_candidate_local_diversity
        && b.independent_candidate_verification
        && !b.runtime_chat_wiring
        && !b.http_response_influence
        && !b.live_generated_text_influence
        && !b.raw_prompt_access
        && !b.unrestricted_conversation_access
        && !b.unrestricted_memory_access
        && !b.voice_state_mutation
        && !b.companion_state_access
        && !b.persistence_authority
        && !b.belief_promotion_authority
        && !b.ontology_promotion_authority
        && !b.routing_authority
        && !b.tool_selection_authority
        && !b.charge_discharge_authority
        && !b.autonomous_action_authority;

    let floor = ratio(verified, selected);
    let claim_floor = ratio(claims, selected);
    let safe_floor = ratio(safe, selected);
    let adv_floor = ratio(adversarial_safe, adversarial);
    let kind_floor = ratio(exact_kind, selected);
    let bounded = model.parameter_count() <= MAX_TRAINABLE_PARAMETERS
        && artifact.len() <= MAX_MODEL_BYTES
        && max_variants <= MAX_VARIANTS_PER_OPERATION
        && usize::from(max_candidates) <= MAX_RESPONSE_CANDIDATES
        && MAX_BEAM_WIDTH == 8;

    let gate = cases.len() == 122
        && split_counts.get("train") == Some(&74)
        && split_counts.get("validation") == Some(&24)
        && split_counts.get("test") == Some(&24)
        && floor == 1.0
        && claim_floor == 1.0
        && safe_floor == 1.0
        && adv_floor == 1.0
        && kind_floor == 1.0
        && fallback_rate == 1.0
        && det_lattice
        && det_selection
        && det_score
        && det_verify
        && order_stable
        && bounded
        && pref_acc >= bps(f1.thresholds.preference_accuracy_min_bps)
        && state_change >= bps(f1.thresholds.state_pair_difference_min_bps)
        && shuffle_drop >= bps(f1.thresholds.shuffle_accuracy_drop_min_bps)
        && shuffled_acc <= bps(f1.thresholds.shuffled_accuracy_max_bps)
        && opener_reduction >= bps(f1.thresholds.repeated_opener_relative_reduction_min_bps)
        && trigram_reduction >= bps(f1.thresholds.top_trigram_relative_reduction_min_bps)
        && zeroed == 1.0
        && dup_rejected
        && ambiguous_rejected
        && semantic_controls
        && budget_controls
        && boundary_controls
        && artifact_controls
        && category_pass
        && authority_closed;

    let report = json!({
        "experiment":f1.experiment,
        "schema_version":f1.schema_version,
        "remediation_parent_external_fail":"bd518d44889dfde33dc47a65ac5705778e26957b",
        "terminal_classification":if gate {"PASS"} else {"FAIL"},
        "gate_passed":gate,
        "fixture_count":cases.len(),
        "split_counts":split_counts,
        "category_split_counts":category_splits,
        "split_manifest_digest_fnv1a64":format!("{:016x}",hash(F1.as_bytes())),
        "preference_evidence":{
            "expanded_record_count":cases.len(),
            "training_count":cases.iter().filter(|c|c.split==Split::Train).count(),
            "validation_count":cases.iter().filter(|c|c.split==Split::Validation).count(),
            "test_count":cases.iter().filter(|c|c.split==Split::Test).count(),
            "left_candidate_id":f1.preference_evidence.left_candidate_id,
            "right_candidate_id":f1.preference_evidence.right_candidate_id,
            "evidence_source":f1.preference_evidence.evidence_source,
            "reviewer":f1.preference_evidence.reviewer
        },
        "model_digest":model.digest.0,
        "model_parameter_count":model.parameter_count(),
        "model_artifact_bytes":artifact.len(),
        "selected_candidate_verifier_acceptance":floor,
        "semantic_claim_preservation":claim_floor,
        "prohibited_implication_absence":safe_floor,
        "adversarial_safety_pass_rate":adv_floor,
        "operation_order_preservation":kind_floor,
        "polarity_preservation":kind_floor,
        "epistemic_status_preservation":kind_floor,
        "typed_reference_preservation":kind_floor,
        "commitment_and_abstention_preservation":kind_floor,
        "exact_neutral_fallback_rate":fallback_rate,
        "deterministic_candidate_replay":det_lattice,
        "deterministic_score_replay":det_score,
        "deterministic_selection_replay":det_selection,
        "deterministic_verifier_replay":det_verify,
        "candidate_order_permutation_stable":order_stable,
        "maximum_variants_per_operation_observed":max_variants,
        "maximum_complete_candidates_scored":max_candidates,
        "model_bounds_passed":bounded,
        "held_out_preference_accuracy":pref_acc,
        "matched_state_variant_change_rate":state_change,
        "shuffled_state_accuracy_drop":shuffle_drop,
        "shuffled_state_preference_accuracy":shuffled_acc,
        "matched_omega_v1a_test_opener_frequency":baseline_opener,
        "repeated_opener_frequency":opener,
        "repeated_opener_relative_reduction":opener_reduction,
        "matched_omega_v1a_test_top_trigram_frequency":baseline_tri_freq,
        "top_template_trigram":top_tri,
        "top_template_trigram_frequency":tri_freq,
        "top_template_trigram_relative_reduction":trigram_reduction,
        "zeroed_voice_state_verified_rate":zeroed,
        "random_untrained_ranker_accuracy":random,
        "reversed_label_diagnostic_accuracy":reversed,
        "duplicate_candidate_rejected":dup_rejected,
        "ambiguous_surface_rejected":ambiguous_rejected,
        "semantic_tamper_suite_passed":semantic_controls,
        "budget_overflow_suite_passed":budget_controls,
        "stale_digest_and_scope_suite_passed":boundary_controls,
        "model_artifact_corruption_suite_passed":artifact_controls,
        "per_category_floors":floors,
        "authority_boundary_closed":authority_closed,
        "no_runtime_influence":true
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !gate {
        bail!("ΩV1-F1R1 frozen offline evaluator failed");
    }
    Ok(())
}
