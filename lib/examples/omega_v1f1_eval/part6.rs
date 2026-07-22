#![allow(dead_code, clippy::type_complexity)]
fn fallback_controls(m: &LearnedExpressionModel, x: &Case, p: &Proj) -> Result<f64> {
    let neutral = VerifierReadyRenderer.render(&x.program, &x.lexical)?;
    let projection = exact_projection(p, &x.fx.id, Pref::Direct)?;
    let mut corrupt = m.clone();
    corrupt.digest.0 ^= 1;
    let r = OfflineLearnedExpressionSelector::new(corrupt).select(
        &x.program,
        &x.lexical,
        &projection,
    )?;
    let model = r.payload.disposition == SelectionDisposition::NeutralFallback
        && r.payload.text == neutral.payload.text;
    let mut stale = x.projection.clone();
    stale.source_digest.push_str(":stale");
    let stale_result =
        OfflineLearnedExpressionSelector::new(m.clone()).select(&x.program, &x.lexical, &stale)?;
    let projection_guard = stale_result.payload.disposition
        == SelectionDisposition::NeutralFallback
        && stale_result.payload.text == neutral.payload.text;
    Ok(ratio(usize::from(model) + usize::from(projection_guard), 2))
}
fn prohibited(a: &AM, x: &Fx) -> Vec<String> {
    let mut v = a
        .category_prohibited_claim_anchors
        .get(&x.category)
        .cloned()
        .unwrap_or_default();
    v.extend(x.prohibited.iter().cloned());
    v.sort();
    v.dedup();
    v
}
fn opener_frequency(xs: &[String]) -> f64 {
    let keys = xs
        .iter()
        .map(|x| tokens(x).into_iter().take(4).collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>();
    let mut c = BTreeMap::new();
    for k in &keys {
        *c.entry(k.clone()).or_insert(0usize) += 1;
    }
    ratio(
        keys.iter()
            .filter(|k| c.get(*k).copied().unwrap_or(0) > 1)
            .count(),
        keys.len(),
    )
}
fn top_trigram(xs: &[String]) -> (String, f64) {
    let mut c = BTreeMap::new();
    for x in xs {
        for t in tokens(x)
            .windows(3)
            .map(|w| w.join(" "))
            .collect::<BTreeSet<_>>()
        {
            *c.entry(t).or_insert(0usize) += 1;
        }
    }
    let (k, n) = c
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .unwrap_or_default();
    (k, ratio(n, xs.len()))
}
fn tokens(x: &str) -> Vec<String> {
    x.split(|c: char| !c.is_ascii_alphanumeric() && c != '\'')
        .filter(|x| !x.is_empty())
        .map(str::to_lowercase)
        .collect()
}
fn reduction(base: f64, observed: f64) -> f64 {
    ((base - observed) / base).max(0.0)
}
fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 {
        0.0
    } else {
        a as f64 / b as f64
    }
}
fn bps(x: u16) -> f64 {
    f64::from(x) / 10_000.0
}
fn hash(x: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in x {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
