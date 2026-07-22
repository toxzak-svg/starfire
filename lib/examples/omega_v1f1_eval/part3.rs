fn validate_headers(f: &FM) -> Result<()> {
    if f.schema_version != 2
        || f.experiment != "OMEGAV1F1R1_OFFLINE_LEARNED_SELECTOR"
        || f.source_corpus != "OMEGAV1A_VOICE_BASELINE"
        || f.preference_evidence.schema_version != 2
        || f.preference_evidence.left_candidate_id != "direct-family-v1"
        || f.preference_evidence.right_candidate_id != "warm-family-v1"
        || f.preference_evidence.evidence_source != "reviewed_project_voice_guidance"
        || f.preference_evidence.reviewer.is_empty()
        || f.split.modulus != 5
        || f.split.test_remainder != 0
        || f.split.validation_remainder != 4
    {
        bail!("frozen F1R1 manifest drift");
    }
    Ok(())
}
fn load() -> Result<Vec<Fx>> {
    let mut out = Vec::new();
    for (name, raw) in SHARDS {
        let mut shard: Vec<Fx> =
            serde_json::from_str(raw).with_context(|| format!("parse {name}"))?;
        out.append(&mut shard);
    }
    Ok(out)
}
fn validate_corpus(a: &AM, f: &[Fx]) -> Result<()> {
    let mut counts = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for x in f {
        *counts.entry(x.category.clone()).or_insert(0usize) += 1;
        if !ids.insert(x.id.clone())
            || x.prompt.is_empty()
            || x.raw.is_empty()
            || x.expected.is_empty()
            || !a.profiles.contains_key(&x.profile)
        {
            bail!("bad fixture {}", x.id);
        }
    }
    if f.len() != 122 || counts != a.category_requirements {
        bail!("corpus drift");
    }
    Ok(())
}
fn validate_split(
    f: &FM,
    xs: &[Fx],
) -> Result<(
    BTreeMap<String, usize>,
    BTreeMap<String, BTreeMap<String, usize>>,
)> {
    let mut totals = BTreeMap::new();
    let mut cats: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut suffixes: BTreeMap<String, BTreeSet<u16>> = BTreeMap::new();
    for x in xs {
        let n = suffix(&x.id)?;
        suffixes.entry(x.category.clone()).or_default().insert(n);
        let s = split(&f.split, &x.id)?;
        *totals.entry(s.s().to_owned()).or_insert(0usize) += 1;
        *cats
            .entry(x.category.clone())
            .or_default()
            .entry(s.s().to_owned())
            .or_insert(0usize) += 1;
    }
    for (k, v) in &f.split.expected_categories {
        let total = v.values().sum::<usize>();
        let expected = (1..=u16::try_from(total)?).collect::<BTreeSet<_>>();
        if suffixes.get(k) != Some(&expected) {
            bail!("suffix drift in {k}");
        }
    }
    if totals != f.split.expected_totals || cats != f.split.expected_categories {
        bail!("split drift");
    }
    Ok((totals, cats))
}
fn suffix(id: &str) -> Result<u16> {
    Ok(id.rsplit_once('-').context("bad id")?.1.parse()?)
}
fn split(p: &SP, id: &str) -> Result<Split> {
    let r = suffix(id)? % p.modulus;
    Ok(if r == p.test_remainder {
        Split::Test
    } else if r == p.validation_remainder {
        Split::Validation
    } else {
        Split::Train
    })
}
fn pref(p: &PP, profile: &str) -> Result<Pref> {
    match p
        .profile_preferences
        .get(profile)
        .map(String::as_str)
        .context("missing preference")?
    {
        "direct" => Ok(Pref::Direct),
        "warm" => Ok(Pref::Warm),
        x => bail!("bad preference {x}"),
    }
}
fn projection(p: &Proj, id: &str, pref: Pref, n: u16) -> Result<LearnedVoiceProjection> {
    let target = if pref == Pref::Direct {
        p.direct
    } else {
        p.warm
    };
    let v = if n.is_multiple_of(2) {
        target
    } else {
        mix(p.neutral, target, 1, 4)
    };
    LearnedVoiceProjection::new(
        u64::from(n),
        v[0],
        v[1],
        v[2],
        v[3],
        v[4],
        v[5],
        v[6],
        format!(
            "omega-v1f1r1-projection-v1:{id}:{}:{:016x}",
            if pref == Pref::Direct {
                "direct"
            } else {
                "warm"
            },
            hash(&v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
        ),
    )
    .map_err(Into::into)
}
fn exact_projection(p: &Proj, id: &str, pref: Pref) -> Result<LearnedVoiceProjection> {
    let v = if pref == Pref::Direct {
        p.direct
    } else {
        p.warm
    };
    LearnedVoiceProjection::new(
        1,
        v[0],
        v[1],
        v[2],
        v[3],
        v[4],
        v[5],
        v[6],
        format!(
            "omega-v1f1r1-exact:{id}:{}",
            if pref == Pref::Direct {
                "direct"
            } else {
                "warm"
            }
        ),
    )
    .map_err(Into::into)
}
fn mix(a: [u16; 7], b: [u16; 7], n: i32, d: i32) -> [u16; 7] {
    let mut v = [0; 7];
    for i in 0..7 {
        let x = i32::from(a[i]) + (i32::from(b[i]) - i32::from(a[i])) * n / d;
        v[i] = u16::try_from(x.clamp(0, 10_000)).unwrap_or(0);
    }
    v
}
