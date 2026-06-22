# Star Curious Research Subagents — 2026-06-22

> Status: **draft**, awaiting Zachary's greenlight before any code changes.
> Companion to: `plans/VOICE_REFINE_2026_06_21.md` (in progress, Phase 0+4 landed).

---

## TL;DR

Curious can already web-search — `lib/runtime/curious.rs::run_probe()` already calls DuckDuckGo and stores findings as memories. What's missing is **targeted sub-question decomposition**: today one probe = one raw search of the probe topic. The result is shallow (whatever DuckDuckGo returns for the literal topic string).

The fix is a small "research subagent" — a unit that takes one targeted angle of a probe, runs one focused search, and returns one finding. Each probe fires 2–3 subagents on complementary angles in parallel (with timeouts), and a synthesizer combines the findings into one coherent answer.

Concretely:

- **Probe = "What is emergence in complex systems?"** becomes 3 subagents:
  1. `"emergence complex systems definition"` → Wikipedia summary
  2. `"examples of emergence in biology physics"` → concrete instances
  3. `"emergence vs reductionism philosophy"` → contrast / debate
- The 3 findings are stored as separate memories (`Research on emergence (definitional): ...`, `Research on emergence (examples): ...`, `Research on emergence (vs reductionism): ...`) and synthesized into a single answer like *"Emergence is when system-level patterns arise that aren't reducible to parts. Examples include... It contrasts with reductionism, which..."*

Net effect: Star's autonomous curiosity produces **real research** instead of one DuckDuckGo hit, without changing the chat-turn budget.

---

## What I found in the code

Hard evidence — what's already there and what's missing:

| Component | Status | Where |
|---|---|---|
| Gap-driven probe generation | ✅ exists | `lib/runtime/curious.rs::maybe_fire()` (lines 62–127) — detects reasoning gaps from store, generates probe questions |
| Inline web search | ✅ exists | `lib/runtime/curious.rs::web_search()` (lines 251–305) — DuckDuckGo instant-answer API, returns first hit |
| Sub-question decomposition | ❌ missing | `generate_probe_question()` only generates ONE question per gap |
| Parallel/sequential fan-out | ❌ missing | `run_probe()` calls `web_search` exactly once per probe |
| Synthesis of multiple findings | ❌ missing | Single result is returned directly to caller; no combining |
| Budget / timeout per subagent | ❌ missing | `ureq::get(&url).call()` has no timeout — can hang the turn |
| Sub-question generation templates | ✅ partial | `lib/curiosity/probes.rs::QuestionTemplates` has 14 patterns across 8 depths — can be reused for decomposition |
| Old `CuriousEngine` (crate::curiosity) | 🪦 dead code | `lib/curiosity/mod.rs` — no imports anywhere; superseded by `lib/runtime/curious.rs` |
| `ResearchWalkabout` | separate concern | `lib/research/mod.rs` — used by `conversation::handle_question()` for in-conversation research (different trigger, can stay parallel) |

The `lib/runtime/curious.rs::CuriousEngine` is the one that gets called from `runtime::maybe_fire_curiosity()` (line 1607). That's the path: idle → gap detection → probe → `run_probe` → result becomes an `AutonomousThought`. We extend THIS path.

The 17 handcrafted analogies in `lib/curiosity/connection.rs` are nice but bypass real knowledge — useful as "what if" prompts, not research targets.

The voice-refine Phase 0 changes (this commit) actually fix one curiosity-adjacent bug: the `lib/curiosity/connection.rs::generate_question()` uses pure timestamp modulo on analogy pools. But that file is dead code now, so the fix is moot.

---

## The plan

### Phase 0 — Extract `ResearchSubagent` (today, ~30 min, low risk)

Single concrete unit that does one thing well. No behavior change yet — just structure.

```rust
// New file: lib/runtime/research_subagent.rs

pub struct ResearchSubagent {
    /// DuckDuckGo client (shared)
    client: DuckDuckGoClient,
    /// Per-subagent timeout (1.5s default)
    timeout: Duration,
    /// Per-subagent result cap (500 chars)
    result_cap_chars: usize,
}

pub struct SubagentResult {
    /// The angle this subagent investigated (e.g., "definitional", "examples")
    pub angle: SubagentAngle,
    /// The original sub-question
    pub question: String,
    /// What we found (truncated to result_cap_chars)
    pub finding: Option<String>,
    /// Source URL if available
    pub source: Option<String>,
    /// Elapsed time
    pub elapsed: Duration,
    /// Why this failed (timeout / no-result / error)
    pub failure: Option<SubagentFailure>,
}

impl ResearchSubagent {
    /// Run one focused search for `question` from the angle `angle`.
    /// Bounded by `timeout`. Never panics, never blocks longer than the budget.
    pub fn run(&self, question: &str, angle: SubagentAngle) -> SubagentResult { ... }
}
```

`SubagentAngle` is an enum: `Definitional | Examples | Contrast | Mechanism | History` (5 angles, see Phase 1 for why 5).

**Why first:** today `run_probe` has 50 lines mixing gap detection, question generation, search, parsing, memory storage, and reasoning updates. Extracting `ResearchSubagent` is the surgical move that makes Phase 1–4 readable.

**Reversibility:** zero behavior change — `run_probe` still does exactly one search, just routed through `ResearchSubagent::run` instead of inline.

### Phase 1 — Sub-question decomposition (1 hour, low risk)

Given a probe's question, generate 2–3 targeted sub-questions, each tagged with an angle.

```rust
// In lib/runtime/curious.rs

fn decompose_probe(probe: &CuriosityProbe) -> Vec<(SubagentAngle, String)> {
    let topic = &probe.topic;
    
    // Pick 2-3 angles based on probe shape.
    // Heuristic: definitional + contrast is the minimum. Add examples/mechanism
    // when the topic looks concrete (has a noun phrase, not a question word).
    
    let mut angles = vec![
        (SubagentAngle::Definitional, format!("{} definition", topic)),
    ];
    
    if topic.split_whitespace().count() >= 2 && !looks_like_meta_question(topic) {
        angles.push((SubagentAngle::Examples, format!("examples of {}", topic)));
    }
    
    angles.push((SubagentAngle::Contrast, format!("{} vs", topic)));
    
    angles
}
```

The 5 angles and their search-query templates:

| Angle | Template | When used |
|---|---|---|
| Definitional | `"{topic} definition"` | Always (probe has to define what it asked about) |
| Examples | `"examples of {topic}"` | When topic is a noun-phrase (not "what" or "why") |
| Contrast | `"{topic} vs"` | Always (gives the synthesizer the "X is not Y" anchor) |
| Mechanism | `"how does {topic} work"` | When topic implies a process |
| History | `"history of {topic}"` | When topic has temporal implications |

The template table lives in `lib/runtime/research_subagent.rs` — one place, easy to extend.

**Why not LLM-based decomposition:** we don't have a working generator yet. Phase 3 of voice-refine wires the charRNN; until then, template decomposition is faster to ship, deterministic, and testable. The synthesizer doesn't need LLM-grade coherence for 2–3 findings.

**Reversibility:** the decomposition is pure-function over the probe. If template-based feels wrong, swap for LLM-based in Phase 4 of voice-refine.

### Phase 2 — Sequential with timeouts (1 hour, low–medium risk)

Run the 2–3 subagents sequentially with per-subagent timeout. **NOT parallel threads.**

```rust
fn run_subagents(&self, angles: Vec<(SubagentAngle, String)>) -> Vec<SubagentResult> {
    let mut results = Vec::with_capacity(angles.len());
    for (angle, question) in angles {
        let result = self.subagent.run(&question, angle);
        results.push(result);
    }
    results
}
```

**Why sequential, not parallel:**
- Each subagent is one DuckDuckGo call (~300–800ms). 3 sequential = ~1.5–2.5s worst case. Parallel saves ~1s.
- Parallel requires either `tokio::spawn` (async restructure of `chat()`) or `thread::spawn` (thread-pool management).
- `chat()` is currently synchronous. Parallel would force it async, which is a much bigger change.
- A hung subagent in parallel can still block the join — same timeout story, more threads.
- Easier to reason about, easier to test, easier to revert.

**Worst-case budget:** 3 subagents × 1.5s timeout = 4.5s per probe. Plus the existing local reasoning path (~50ms). That's higher than today's path (one search, ~1s). If 4.5s feels too long, lower the per-subagent timeout to 1.0s = 3s total, or cap subagent count at 2.

**Timeout implementation:** wrap `ureq::get(...).timeout(Duration::from_millis(1500)).call()`. ureq supports `.timeout()` natively. No async needed.

**Reversibility:** yes — single function, single call site. Easy to swap for parallel later if 4.5s proves too slow.

### Phase 3 — Synthesis (1 hour, medium risk)

Combine 2–3 `SubagentResult` entries into one coherent answer for `run_probe` to return.

```rust
fn synthesize_findings(probe: &CuriosityProbe, results: &[SubagentResult]) -> Option<String> {
    let good: Vec<&SubagentResult> = results.iter()
        .filter(|r| r.finding.is_some() && r.failure.is_none())
        .collect();
    
    if good.is_empty() {
        return None;
    }
    
    if good.len() == 1 {
        return good[0].finding.clone();
    }
    
    // Multi-finding synthesis: lead with definitional, then examples/contrast.
    let lead = good.iter().find(|r| r.angle == SubagentAngle::Definitional)
        .or_else(|| good.first());
    
    let mut parts: Vec<String> = vec![lead.unwrap().finding.clone().unwrap()];
    
    for r in &good {
        if std::ptr::eq(*r, lead.unwrap()) { continue; }
        if let Some(f) = &r.finding {
            let connector = match r.angle {
                SubagentAngle::Examples => "For example:",
                SubagentAngle::Contrast => "In contrast:",
                SubagentAngle::Mechanism => "Mechanistically:",
                SubagentAngle::History => "Historically:",
                _ => "Also:",
            };
            parts.push(format!("{} {}", connector, f));
        }
    }
    
    Some(parts.join(" "))
}
```

**Why this shape:**
- Single-finding case = passthrough (same as today)
- Multi-finding case = lead with the definitional answer (or first one), then add context with angle-aware connectors
- No LLM, no template engine — just sequence and label

**Risk:** the connectors are hardcoded English. For some topic/angle combos, "In contrast:" might not fit. Mitigation: keep the connectors short ("For example:", "Also:") and let truncation handle the rest. Defer LLM synthesis to voice-refine Phase 3.

**Test surface:** synthesizer is a pure function — easy to test with canned `SubagentResult` arrays.

### Phase 4 — Integration into `run_probe` (30 min, low risk)

Replace the inline `web_search` call with the new pipeline.

Before (current `run_probe` lines 184–247):
```rust
// Step 2: No local answer — search the web
info!("...searching the web...");
let web_result = self.web_search(&probe.topic);
if let Some((answer, source)) = web_result {
    // store as memory, return Some(answer)
}
```

After:
```rust
// Step 2: No local answer — fan out to subagents
info!("...firing research subagents on '{}'...", &probe.topic);
let angles = decompose_probe(&probe);
let results = self.run_subagents(angles);
let findings = synthesize_findings(&probe, &results);

// Store each non-failed subagent result as a separate memory
for r in &results {
    if let Some(finding) = &r.finding {
        let mem = Memory::new_seeded(
            &format!("Research on '{}' ({}): {} [Source: {}]",
                &probe.topic, angle_label(r.angle), finding, r.source.as_deref().unwrap_or("none")),
            MemoryDomain::Empirical,
            0.7,
        );
        let _ = self.store.insert_memory(&mem);
    }
}

findings  // synthesized answer (or None if all failed)
```

Memory quality changes too — instead of one "Research on X: ..." memory per probe, we get 2–3 angle-tagged memories. That's actually MORE useful: future probes can search for specific angles.

### Phase 5 — Test & measure (ongoing, ~1 hour initial)

How do we know subagents are firing well?

- **Per-probe result count test:** fire 20 synthetic probes, assert at least 1 subagent returns a finding ≥80% of the time. (Some DuckDuckGo queries will return nothing — that's fine, but the FALLBACK rate should be <20%.)
- **Per-subagent timing test:** all subagents complete within their timeout (no hangs). Use `std::time::Instant` and assert `elapsed < timeout` for every result.
- **Synthesis coherence smoke test:** feed 3 known-good findings, assert the output contains all 3 (no drops) AND has angle-aware connectors in roughly the right positions.
- **Integration test (manual):** start `cargo run`, idle for 30s, watch the autonomous thought log. Each thought should now have richer content than the pre-Phase-4 output.
- **Memory count growth:** over 24h idle, count the new Empirical memories. Pre-Phase-4: 1 per probe. Post-Phase-4: 2–3 per probe. Should roughly triple (if probes fire at the same rate).

---

## Sequencing & ownership

| Phase | Effort | Risk | Reversible |
|---|---|---|---|
| 0 | 30 min | very low | yes (pure refactor, no behavior change) |
| 1 | 1 hour | low (pure function over probe) | yes |
| 2 | 1 hour | low–medium (timeout behavior, DuckDuckGo rate limits) | yes |
| 3 | 1 hour | medium (synthesis coherence on edge cases) | yes |
| 4 | 30 min | low (single call site) | yes |
| 5 | ongoing | — | — |

**Do Phase 0 + Phase 1 first.** They're independent, low-risk, and unblock everything else. You can land them and see the shape before committing to the synthesis design.

**Phase 2 is the riskiest phase** because it changes observable timing (per-turn latency). Land it with a DEBUG log of every subagent's elapsed time so you can spot the long-tail cases.

**Phase 3 + Phase 4 are coupled.** Phase 3 produces the synthesis, Phase 4 plugs it in. Land them together.

**Phase 5 is ongoing** — wire the metrics in early (during Phase 2) so you have baseline numbers when Phase 4 lands.

---

## What I am NOT proposing

- **NOT async/paralleI subagents.** The latency win (~1s) isn't worth restructuring `chat()` from sync to async. If we ever need it, Phase 2 can be swapped for parallel — design leaves that door open.
- **NOT LLM-based decomposition or synthesis.** The charRNN isn't wired in chat path yet (voice-refine Phase 3). Until it is, template-based is deterministic, testable, and fast.
- **NOT modifying `lib/curiosity/connection.rs`.** It's dead code, and the plan's voice changes already broke the timestamp-shuffle pattern there. Resurrecting it is a separate project.
- **NOT touching `lib/research/mod.rs::ResearchWalkabout`.** That's the in-conversation research path (triggered when Star doesn't know the answer to Zachary's question). Different trigger, different latency budget, different storage. Subagents are for AUTONOMOUS idle curiosity. Keep them separate.
- **NOT introducing new dependencies.** ureq is already in the deps; no `reqwest`, no `tokio` (for this), no `rayon`.
- **NOT a wholesale rewrite.** Each phase is independently shippable. Phase 0 alone is a strict refactor with zero behavior change.

---

## Open questions for Zachary before I start

1. **Subagent count: 2 or 3?** My default is 2-3 (Definitional + Contrast + maybe Examples). Lower = faster turn, shallower synthesis. Higher = slower turn, more comprehensive. Default: 3, with timeout 1.2s each = 3.6s worst case. ✅ **CONFIRMED 2026-06-22: default 3**
2. **Per-subagent timeout: 1.0s or 1.5s?** DuckDuckGo usually responds in 300-600ms. 1.5s is a comfortable margin. 1.0s is tighter but caps total budget at 3s. Default: 1.2s. ✅ **CONFIRMED 2026-06-22: default 1.2s**
3. **Should failed subagents (timeout / no result) still log to memory?** My default: NO. Failed = no useful knowledge. If you want even failures logged for debugging ("Star tried to research X and got nothing"), I can flip it. ✅ **CONFIRMED 2026-06-22: default NO**
4. **Do you want a "research budget" knob in `CuriousEngine`?** E.g. `max_subagents_per_probe: usize` and `subagent_timeout_ms: u64` as configurable fields. My default: yes, hardcoded for now, expose as fields later. Adds ~10 lines. **PENDING**
5. **Is `models/ckpt_e28_b500.pt` still the right charRNN?** Same question from voice-refine — if you've retrained since 2026-04-21, point me at the new checkpoint so I can wire synthesis-LLM later. **PENDING**
6. **Test mode vs. live DuckDuckGo:** tests can't reliably hit DuckDuckGo (network, rate limits). My plan: introduce a `SearchBackend` trait with `RealSearch` (production) and `MockSearch` (test) implementations. Default: real. Mock for unit tests. OK? **PENDING**

### Implementation defaults (locked)

- Subagent count: **3** (Definitional + Contrast + Examples when topic is concrete)
- Per-subagent timeout: **1.2s**
- Per-probe worst-case latency: **3.6s**
- Failed subagents: **do not** log to memory
- Per-subagent result cap: **500 chars** (assumed from inline `web_search` truncation behavior — confirm in Phase 0 if I find a different cap)

---

_This plan extends the curiosity infrastructure in `lib/runtime/curious.rs` that already closes the reasoning → gap → probe → web → memory loop. The change is depth, not breadth: one probe becomes 2-3 targeted subagents instead of one raw search._
