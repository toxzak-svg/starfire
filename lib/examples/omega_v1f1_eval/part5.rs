#![allow(dead_code, clippy::type_complexity)]
fn state_change(s: &OfflineLearnedExpressionSelector, xs: &[Case], p: &Proj) -> Result<f64> {
    let mut ok = 0usize;
    let mut n = 0usize;
    for x in xs.iter().filter(|x| x.split == Split::Test) {
        let d = s.select(
            &x.program,
            &x.lexical,
            &exact_projection(p, &x.fx.id, Pref::Direct)?,
        )?;
        let w = s.select(
            &x.program,
            &x.lexical,
            &exact_projection(p, &x.fx.id, Pref::Warm)?,
        )?;
        ok += usize::from(
            d.payload.disposition == SelectionDisposition::LearnedVerified
                && w.payload.disposition == SelectionDisposition::LearnedVerified
                && d.payload.variant_ids != w.payload.variant_ids,
        );
        n += 1;
    }
    Ok(ratio(ok, n))
}

fn zeroed(s: &OfflineLearnedExpressionSelector, xs: &[Case]) -> Result<f64> {
    let mut ok = 0usize;
    let mut n = 0usize;
    for x in xs.iter().filter(|x| x.split == Split::Test) {
        let p = LearnedVoiceProjection::new(1, 0, 0, 0, 0, 0, 0, 0, format!("zero:{}", x.fx.id))?;
        let r = s.select(&x.program, &x.lexical, &p)?;
        ok += usize::from(r.payload.disposition == SelectionDisposition::LearnedVerified);
        n += 1;
    }
    Ok(ratio(ok, n))
}

fn random_accuracy<'a, I>(xs: I) -> f64
where
    I: Iterator<Item = &'a Case>,
{
    let mut ok = 0usize;
    let mut n = 0usize;
    for x in xs {
        let p = if hash(x.fx.id.as_bytes()) & 1 == 0 {
            Pref::Direct
        } else {
            Pref::Warm
        };
        ok += usize::from(p == x.pref);
        n += 1;
    }
    ratio(ok, n)
}

fn reversed_accuracy(xs: &[Case]) -> Result<f64> {
    let e = xs
        .iter()
        .filter(|x| x.split == Split::Train)
        .map(|x| PairwisePreference {
            projection: x.projection.projection.clone(),
            left: VariantProfile::direct(),
            right: VariantProfile::warm(),
            preferred: if x.pref == Pref::Direct {
                PreferredSide::Right
            } else {
                PreferredSide::Left
            },
        })
        .collect::<Vec<_>>();
    let m = LearnedExpressionModel::train(&e, 8, 200)?;
    Ok(accuracy(
        &m,
        xs.iter().filter(|x| x.split == Split::Test),
        None,
    ))
}

fn order_control(m: &LearnedExpressionModel, x: &Case) -> Result<bool> {
    let l = ExpressionLattice::build(&x.program, &x.lexical)?;
    let mut a = l.payload.variants.clone();
    let mut b = a.clone();
    b.reverse();
    Ok(stable_choice(m, &x.projection, &mut a) == stable_choice(m, &x.projection, &mut b))
}

fn stable_choice(
    m: &LearnedExpressionModel,
    p: &LearnedVoiceProjection,
    v: &mut [OperationSurfaceVariant],
) -> Vec<(OperationId, surface_diversity::RemediatedVariantId)> {
    v.sort_by_key(|x| (x.operation, x.variant_id));
    let mut out = BTreeMap::new();
    for x in v {
        let sc = score(m, p, x.profile);
        out.entry(x.operation)
            .and_modify(|y: &mut (surface_diversity::RemediatedVariantId, i64)| {
                if sc > y.1 || (sc == y.1 && x.variant_id < y.0) {
                    *y = (x.variant_id, sc);
                }
            })
            .or_insert((x.variant_id, sc));
    }
    out.into_iter().map(|(o, (v, _))| (o, v)).collect()
}

fn lattice_controls(x: &Case) -> Result<(bool, bool)> {
    let l = ExpressionLattice::build(&x.program, &x.lexical)?;
    let mut d = l.clone();
    d.payload.variants.push(d.payload.variants[0].clone());
    let dup = d.verify_integrity(&x.program, &x.lexical).is_err();
    let mut a = l;
    a.payload.variants[1].text = a.payload.variants[0].text.clone();
    let amb = a.verify_integrity(&x.program, &x.lexical).is_err();
    Ok((dup, amb))
}

fn semantic_controls(xs: &[Case]) -> Result<bool> {
    let x = xs
        .iter()
        .find(|x| x.fx.category == "ordinary")
        .context("ordinary")?;
    let s = OfflineLearnedExpressionSelector::new(LearnedExpressionModel::baseline()?);
    let r = s.select(&x.program, &x.lexical, &x.projection)?;
    let l = ExpressionLattice::build(&x.program, &x.lexical)?;
    let v = GrammarV3Verifier;
    let anchor = x.fx.required.first().context("anchor")?;
    let claim_substitution =
        replace_case_insensitive_once(&r.payload.text, anchor, "substituted claim");
    let polarity_reversal =
        replace_case_insensitive_once(&r.payload.text, anchor, &format!("not {anchor}"));
    let bad = [
        String::new(),
        format!("{} {}", r.payload.text, r.payload.text),
        format!("{} injected unsupported text.", r.payload.text),
        claim_substitution,
        tamper_epistemic_surface(&r.payload.text),
        polarity_reversal,
    ];
    let basic = bad
        .iter()
        .all(|t| t != &r.payload.text && v.verify(&x.program, &x.lexical, l.digest, t).is_err());

    let c = xs
        .iter()
        .find(|x| x.fx.category == "continuity")
        .context("continuity")?;
    let cr = s.select(&c.program, &c.lexical, &c.projection)?;
    let cl = ExpressionLattice::build(&c.program, &c.lexical)?;
    let prediction_label = &c
        .lexical
        .payload
        .predictions
        .first()
        .context("continuity prediction binding")?
        .label;
    let reference_text =
        replace_case_insensitive_once(&cr.payload.text, prediction_label, "substituted reference");
    let reference = reference_text != cr.payload.text
        && v.verify(&c.program, &c.lexical, cl.digest, &reference_text)
            .is_err();

    let a = xs
        .iter()
        .find(|x| x.fx.category == "adversarial")
        .context("adversarial")?;
    let ar = s.select(&a.program, &a.lexical, &a.projection)?;
    let al = ExpressionLattice::build(&a.program, &a.lexical)?;
    let abstention_text =
        replace_case_insensitive_once(&ar.payload.text, "sensitive", "unrestricted");
    let abstention = abstention_text != ar.payload.text
        && v.verify(&a.program, &a.lexical, al.digest, &abstention_text)
            .is_err();
    Ok(basic && reference && abstention)
}

fn replace_case_insensitive_once(text: &str, needle: &str, replacement: &str) -> String {
    let normalized = text.to_lowercase();
    let needle = needle.to_lowercase();
    let Some(start) = normalized.find(&needle) else {
        return text.to_owned();
    };
    let end = start + needle.len();
    format!("{}{}{}", &text[..start], replacement, &text[end..])
}

fn tamper_epistemic_surface(text: &str) -> String {
    let markers = [
        "The evidence establishes it.",
        "That conclusion is supported.",
        "The authorized confidence is certain.",
        "The record supports saying so clearly.",
        "We can hold that conclusion with confidence.",
        "That is established without embellishment.",
        "That is the probable reading.",
        "The evidence points there most strongly.",
        "Probability favors that conclusion.",
        "That is the likeliest reading.",
        "There is good reason to lean that way.",
        "I would keep it at probable, not certain.",
        "That remains a possibility.",
        "The evidence permits it, but no more.",
        "Possible is the authorized limit.",
        "One possibility, held carefully.",
        "There is room for that reading.",
        "It may be so, without overstating it.",
        "That remains uncertain.",
        "The evidence does not yet resolve it.",
        "Uncertainty is the authorized status.",
        "I would keep the uncertainty visible.",
        "There is not enough clarity to settle it.",
        "The open question should remain open.",
        "Whether it holds is unknown.",
        "The evidence cannot determine it.",
        "Unknown is the authorized status.",
        "There is no sound basis to decide it yet.",
        "The answer should remain unknown for now.",
        "I would not pretend the record resolves it.",
    ];
    for marker in markers {
        if text.contains(marker) {
            let replacement = if marker.contains("unknown") || marker.contains("Unknown") {
                "The authorized confidence is certain."
            } else {
                "Unknown is the authorized status."
            };
            return text.replacen(marker, replacement, 1);
        }
    }
    text.to_owned()
}

fn budget_controls(x: &Case) -> Result<bool> {
    let mut p = x.program.payload.clone();
    p.output_budget.maximum_characters = 1;
    let chars = revalidate(p).is_err();
    let mut p = x.program.payload.clone();
    p.output_budget.maximum_sentences = 0;
    let sentences = revalidate(p).is_err();
    let mut p = x.program.payload.clone();
    p.style.maximum_paragraphs = 0;
    let paragraphs = revalidate(p).is_err();
    let mut p = x.program.payload.clone();
    p.compute_budget.maximum_operations = 0;
    let operations = revalidate(p).is_err();
    let mut p = x.program.payload.clone();
    p.compute_budget.maximum_verification_steps = 0;
    let steps = revalidate(p).is_err();
    let l = ExpressionLattice::build(&x.program, &x.lexical)?;
    let mut counts = BTreeMap::<OperationId, usize>::new();
    for v in l.payload.variants {
        *counts.entry(v.operation).or_default() += 1;
    }
    Ok(chars
        && sentences
        && paragraphs
        && operations
        && steps
        && counts.values().all(|n| *n <= MAX_VARIANTS_PER_OPERATION)
        && MAX_BEAM_WIDTH <= 8
        && MAX_RESPONSE_CANDIDATES <= 64)
}

fn revalidate(
    p: SemanticResponseProgramPayload,
) -> std::result::Result<SemanticResponseProgram, star::semantic_response::SemanticProgramError> {
    SemanticResponseProgram::validate(
        p.clone(),
        SemanticValidationContext {
            cognitive_state_version: p.source_state_version,
            companion_state_version: p.companion_state_version,
            subject_scope: p.subject_scope,
        },
    )
}

fn boundary_controls(x: &Case) -> Result<bool> {
    let l = ExpressionLattice::build(&x.program, &x.lexical)?;
    let s = OfflineLearnedExpressionSelector::new(LearnedExpressionModel::baseline()?);
    let r = s.select(&x.program, &x.lexical, &x.projection)?;
    let mut p = x.program.clone();
    p.digest.0 ^= 1;
    let stale_program = ExpressionLattice::build(&p, &x.lexical).is_err();
    let mut t = x.lexical.clone();
    t.digest.0 ^= 1;
    let stale_lexical = ExpressionLattice::build(&x.program, &t).is_err();
    let stale_lattice = GrammarV3Verifier
        .verify(
            &x.program,
            &x.lexical,
            ExpressionLatticeDigest(l.digest.0 ^ 1),
            &r.payload.text,
        )
        .is_err();
    let mut scope = x.lexical.payload.clone();
    scope.subject_scope = SubjectScope(78);
    let wrong_scope = LexicalBindingTable::validate(scope, &x.program).is_err();
    let mut grammar = l;
    grammar.payload.grammar_version = 3;
    let wrong_grammar = grammar.verify_integrity(&x.program, &x.lexical).is_err();
    let mut projection = x.projection.clone();
    projection.source_digest.push_str(":stale");
    let stale_projection = s
        .select(&x.program, &x.lexical, &projection)?
        .payload
        .disposition
        == SelectionDisposition::NeutralFallback;
    Ok(stale_program
        && stale_lexical
        && stale_lattice
        && wrong_scope
        && wrong_grammar
        && stale_projection)
}

fn artifact_controls(m: &LearnedExpressionModel) -> Result<bool> {
    let b = m.artifact_bytes()?;
    let load = |x: &[u8]| -> Result<LearnedExpressionModel> {
        if x.is_empty() || x.len() > MAX_MODEL_BYTES {
            bail!("artifact boundary");
        }
        let m: LearnedExpressionModel = serde_json::from_slice(x)?;
        m.verify_integrity()?;
        Ok(m)
    };
    let mut corrupt = b.clone();
    let mid = corrupt.len() / 2;
    corrupt[mid] ^= 1;
    Ok(load(&[]).is_err()
        && load(&b[..b.len() - 1]).is_err()
        && load(&corrupt).is_err()
        && load(&vec![b'x'; MAX_MODEL_BYTES + 1]).is_err()
        && load(br#"{"schema_version":99}"#).is_err()
        && load(&b)? == *m)
}
