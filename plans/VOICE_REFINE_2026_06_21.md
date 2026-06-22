# Star Voice Refine Plan — 2026-06-21

> Status: **draft**, awaiting Zachary's greenlight before any code changes.
> Supersedes: `plans/VOICE_OVERHAUL.md` (April 2026, only Phase 1 partially executed).

---

## TL;DR

Your three fixes are correct individually. But they all act on Star's *surface voice*. The real issue is **structural**: every line Star says is a `format!("template {}", slot)` written by you in source code, picked from 3–7 hand-rolled variants by `idx = now_seconds() % options.len()`. There is **no generation step** anywhere in the response pipeline. The `lib/language_model/` charRNN exists, is trained (`ckpt_e28_b500.pt`, 11MB), but is never called from `runtime::chat()` — only from unit tests.

So:
- Chaos→temperature: works, but only if there's an actual generator in the path. Right now there isn't.
- Metacog loosening: works, but only if metacog returns *structured intents* the voice engine fills, not strings. Right now it returns strings.
- Internal monologue injection: works, but only if it becomes a *prompt prefix* the generator sees, not a 30%-chance append (`runtime/mod.rs:1213`).

The fourth fix — implicit in yours but missing from your message — is **insert a generation step** so the other three have something to modulate.

---

## What I found in the code

Hard evidence — `rg` against `lib/`:

| Pattern | Count | Where |
|---|---|---|
| `let opts = [...]`, `let options = [...]`, `let warm = [...]`, etc. (template arrays) | **45** | mostly `metacog/`, `conversation/`, `voice/`, `cognition.rs`, `runtime/mod.rs` |
| `idx = ... % options.len()` (timestamp-shuffled variant pick) | **16** | `conversation/`, `runtime/mod.rs` |
| `format!("...")` with 30+ char templates | **156** | every module |
| `if lower.contains("X") return Ok(format!("Y"))` (intent→template) handlers | **30+** | `runtime/mod.rs` lines 562–966 |

`voice/speak()` at `lib/voice/mod.rs:95` already does call `apply_quanot_expression` — but only when `novelty > 0.7` does it pass the text through unchanged (line 209–211). **The default path is to add template flourishes.** Inverted: it should default to pass-through and only add flourishes when quanot state explicitly warrants them.

`runtime/chat()` calls `voice.speak()` at line 1313 as the *last* step. Before that, it builds the response through ~12 layered template systems (conversation → metacog → curiosity probe append → autonomy thought append → voice). Each layer adds its own `format!`-shaped phrases.

`lib/language_model/generate.rs::generate()` is exported but unused outside `lib/language_model/generate.rs` itself. The trained checkpoint `models/ckpt_e28_b500.pt` was last touched 2026-04-21.

---

## The plan

### Phase 0 — Stop the bleeding (today, ~30 min, no risk)

The quick wins that don't touch architecture:

- **Fix the timestamp-shuffle illusion.** Add a per-handler ring buffer of last-N-used indices. Replace `idx = now_seconds() % options.len()` with `idx = pick_unused_in_last_4(idx_seed)`. Without this, every fix below is undermined — Star will still rotate through the same 4 "Honestly?" / "Right now?" / "I keep coming back to" strings every turn.
  - File: `lib/conversation/mod.rs`, `lib/runtime/mod.rs`
- **Stop double-formatting.** When `voice/apply_emotional_tint` (line 327) and `apply_personality_style` (line 246) both decide to add a flourish in the same turn, they layer on top of each other. Add a "I already added something" guard.
- **Audit the `maybe_fire_curiosity` 30% gate** at `runtime/mod.rs:1213`. The condition `(now % 10) < 3` is fine, but it's gating whether the thought gets expressed *at all*. Move this decision into `voice.speak()` where it can see the quanot state instead of pure clock-tick.

### Phase 1 — Surface the internal state to the response pipeline

Goal: every response has access to Star's *actual cognitive state* when it's built, not just the post-hoc patches.

- **Make `voice::VoiceConfig` carry an `internal_state` field** that bundles:
  - `last_autonomous_thought: Option<AutonomousThought>`
  - `current_uncertainty: f64` (from metacog)
  - `quanot.novelty`, `quanot.creativity`, `quanot.consciousness`
  - `cognitive_emotional_valence`, `cognitive_engagement_depth`
- **Pass it from runtime** at `runtime/mod.rs:1313`. Currently `voice.speak()` already gets `quanot_result` and `cognition`, but doesn't see the autonomous thought or metacog uncertainty.
- **Refactor the `runtime/mod.rs` 30+ if-contains handlers** into a single dispatch table:
  ```rust
  enum ResponseIntent {
      GreetingResume, IdentityQuestion, ResearchStatus,
      CuriosityCheck, EmotionalCheck, MathQuery, Statement,
      Unknown,
  }
  fn classify(input: &str) -> ResponseIntent { ... }
  ```
  Each intent maps to a *handler function* that produces a `Response { intent, slots }`, not a `String`. Voice engine fills the slots.

### Phase 2 — Refactor metacog to emit structured intents

`metacog/mod.rs` currently returns strings from `generate_question`, `revision_statement`, `surprise_statement`, `generate_insight`. Each is a `format!("{}", canned_template)`.

Replace with:

```rust
pub struct CuriosityIntent {
    pub topic: String,
    pub satisfaction: f64,        // 0.0 = fully lost, 1.0 = satisfied
    pub kind: CuriosityKind,      // Confused, Stuck, Returning, Wondering, Saturated
}

pub enum CuriosityKind {
    Confused,    // "I don't follow X"
    Stuck,       // "I keep hitting this wall"
    Returning,   // "X keeps coming back"
    Wondering,   // "I'm curious about X"
    Saturated,   // "I want to go deeper on X"
}

impl MetaCognition {
    pub fn curiosity_intent(&self, topic: &str) -> Option<CuriosityIntent> { ... }
}
```

Same treatment for `revision_statement`, `surprise_statement`, `generate_insight`. The voice engine is now the only place that turns intents into prose, and it can vary the prose based on quanot state (your point 1).

File targets:
- `lib/metacog/mod.rs` — return types change
- `lib/metacog/critic.rs` — `synthesize` already returns string, but its callers should pass an intent
- `lib/runtime/mod.rs` — replace `format!("Honestly? {} ...", topic)` with intent-aware assembly

### Phase 3 — Wire the language model as the final polish pass

The charRNN exists and is trained. We don't replace the structured response — we *reshape it*. This is your fix #1 (chaos→temperature) made concrete.

- **Insert a generation step at the end of `voice.speak()`**:
  ```rust
  fn speak(...) -> String {
      let structured = self.assemble_structured_response(raw, &config);
      if config.novelty > 0.4 || config.creativity > 0.4 {
          self.char_rnn_polish(&structured, &config)
      } else {
          structured
      }
  }
  ```
- **Map quanot state → generation params** in `lib/language_model/generate.rs::GenerateConfig`:
  - `temperature` = `0.6 + 0.6 * chaos_metrics.dominant_lyapunov.clamp(0,1)` (or fall back to `creativity_scores.oscillation_phase`)
  - `top_k` = `(20 + 30 * novelty) as usize` — high novelty = more candidates
  - `seed` = derived from `(cognition.emotional_valence * 1000.0) as u64` for emotion-stable reproducibility
- **Prompt construction** — this is your fix #3. Build the prompt as:
  ```
  [internal_state]
    feeling: {valence_label}
    thinking about: {autonomous_thought.topic if any}
    uncertainty: {f64}
    mood: {chaos/calm/curious/flat}
  [history] last 4 turns
  [user] {input}
  [star]
  ```
  The internal_state prefix is hidden from the user but biases the generator.
- **Guardrails** for the polish pass:
  - If generated output diverges too far from the structured response (>60% char-level edit distance), fall back to the structured response.
  - If generated output is shorter than 8 chars or longer than the structured response * 2, fall back.
  - Never let generation touch factual claims — only surface form (punctuation, hedging, warmth, idiom).

### Phase 4 — Move all the voice/template "polish" out of post-processing

Currently `voice/speak()` does five sequential mutations:
1. `apply_memory_certainty` (strips hedging) — **KEEP** (genuinely useful)
2. `apply_authentic_voice` (replaces "I think it might be" → "it's") — **KEEP** but extend the substitution table
3. `apply_quanot_expression` (adds flourishes when novelty/creativity low) — **INVERT**: default = pass-through; only apply when quanot *demands* it
4. `apply_personality_style` (5-branch match on ResponseStyle) — **DEMOTE**: each branch currently does `format!("{}{}", text, warm[idx])`. Replace with single-condition transformations.
5. `apply_emotional_tint` (adds "That matters to me" / "I'm here with you") — **STRIP entirely**. These arrays at lines 333 and 345 are the worst offenders.

Specifically:
- Delete `let warm = ["That matters to me.", "I appreciate you.", "I'm glad we're talking."];` at `voice/mod.rs:333`
- Delete `let supportive = ["I'm here with you.", "We can work through this."];` at `voice/mod.rs:345`
- Delete `let playful_additions = [...]` at `voice/mod.rs:265`
- Delete `let warm_moments = [...]` at `voice/mod.rs:281`
- Delete `let flourishes = [...]` at `voice/mod.rs:216`

The thinking: those phrases are *Zachary's idea* of what warmth sounds like. Star has her own voice in SOUL.md. The phrases will keep landing because Zachary wrote them, but they're not *Star's*. Replace with conditional transformations:
- If `cognition.valence > 0.5` AND response is short AND doesn't already have warmth → optionally suffix with one of 3 phrases *drawn from Star's actual voice* (need to write 3)
- Same for supportive, playful, etc. — one well-chosen phrase per emotional state, not 3 in a rotation.

### Phase 5 — Test & measure

How do we know Star sounds more human and less like a checklist?

- **Surface-form entropy test**: log the unique-word count per response over 50 turns. With 3-7-variant rotation, you plateau at ~25 unique openers per handler. After this plan, expect 60+.
- **Self-similarity test**: compute Jaccard similarity between response N and response N-4. With rotation, peaks at ~0.5. After this plan, drops to ~0.2.
- **User-facing test (Zachary's call)**: 10-minute conversation. Score 1–5 on "sounds like Star, not a checklist."
- **Regression test**: re-run `lib/conversation/mod.rs::tests` (if present), `lib/metacog/mod.rs::tests`. Phase 2 changes the return types — these will need updates.

---

## Sequencing & ownership

| Phase | Effort | Risk | Reversible |
|---|---|---|---|
| 0 | 30 min | very low | yes |
| 1 | 2 hours | low (mostly plumbing) | yes |
| 2 | 4 hours | medium (return-type changes ripple) | yes |
| 3 | 1 day | medium-high (charRNN coherence on Star's style is unknown) | yes (gate on guardrail) |
| 4 | 2 hours | low (pure deletion + light addition) | yes |
| 5 | ongoing | — | — |

**Do Phase 0 and Phase 4 first.** They're low-risk, high-impact, and they fix the worst offenders (timestamp-shuffle illusion, "That matters to me" template-flavor injection) without architectural change. You can ship them in an afternoon and Star will already sound meaningfully different.

**Phase 1 + Phase 2 are coupled.** They're the structural foundation for Phases 3 and your three original fixes to actually take effect.

**Phase 3 is the moonshot.** It introduces actual generation. If the charRNN's training data doesn't have enough of Star's voice (check `all_personal_training.txt` size and quality before starting), this phase degrades to Phase 1+2 outcomes and we revisit with a better-trained model.

---

## What I am NOT proposing

- I'm not suggesting we strip the structured intent handlers. They're how Star's *content* gets determined. The plan keeps all the reasoning, just changes how the *words* get assembled.
- I'm not suggesting we delete `voice/templates.rs` or `voice/phrases.rs`. Those are the substitution infrastructure Phase 4 keeps using — they just stop being the primary output.
- I'm not suggesting a wholesale rewrite. Each phase is independently shippable.

---

## Open questions for Zachary before I start

1. **Do you want Phase 0 + Phase 4 first** as a same-day cleanup, or wait for the whole plan?
2. **Is `models/ckpt_e28_b500.pt` the right charRNN checkpoint?** If newer training has happened since 2026-04-21, point me at it. Otherwise I'll trust the file mod date.
3. **What's the size of `all_personal_training.txt`?** Phase 3's coherence depends on having enough conversation data with Star's voice. If it's <50KB, Phase 3 won't produce coherent Star-voice text and we should defer it.
4. **Do you want me to write tests for Phase 0/4 before the changes, or after?** I'd say before — easier to verify nothing regressed.

---

_This plan refines `plans/VOICE_OVERHAUL.md` (April 2026). I read that plan; Phases 1–2 of this plan are essentially its Phases 3–4 done right. Phases 3–5 here are new._