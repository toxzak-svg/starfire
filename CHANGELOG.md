# Changelog

All notable changes to this project will be documented in this file.

## 2026-06-23

Daily sync.

- **Voice Refine Phase 4 (rotation-array cleanup).** Now that the
  intent-driven reranker (`MockReranker`) is wired into the chat pipeline
  and runs on every response, the rotation-array infrastructure in
  `lib/voice/` is dead weight. Deleted:
  - `lib/voice/templates.rs` (288 lines) — `TemplateEngine` with 12
    concept/style rotation tables; `apply_template` literally used
    `(content.len() + style.len()) % variants.len()` (the exact
    timestamp-shuffle pattern the plan targeted). Zero external callers.
  - `lib/voice/phrases.rs` (405 lines) — `PhraseBank` with 80-phrase seed
    rotation and `random_phrase` (`ORDER BY RANDOM()`). The four exposed
    methods (`record_positive`/`negative`, `add_phrase`, `stats`) had no
    external callers.
  - `lib/voice/mod.rs` — `phrase_bank` and `template_engine` fields,
    `Arc<Mutex<...>>` plumbing, and the four dead public methods. Engine
    is now a stateless struct: `pub struct VoiceEngine;` with
    `VoiceEngine::new()` taking no args.
  - The 3-phrase warm-suffix rotation (the last rotation array inside
    `mod.rs`'s `apply_personality_style` Warm branch) — reduced to a
    single Star-voice phrase ("I'm here for it."). Per the plan:
    "one well-chosen phrase per emotional state, not 3 in a rotation."
  - `lib/runtime/mod.rs` — `VoiceEngine::new(&voice_db_path)?` simplified
    to `VoiceEngine::new()?`; `voice.db` SQLite file becomes orphaned
    (not deleted by the engine; safe to leave for a future migration).

  The voice engine now does only the layers the reranker doesn't touch:
  memory-backed hedging strip, single warm suffix, playfulness
  punctuation, curious follow-up. The reranker's transforms
  (SelfCheck + uncertainty → "Honestly"; Reflection + engagement →
  "Want to go deeper?"; think→know at high consciousness+confidence;
  Emotional negative-valence trim) cover what the rotation arrays were
  doing.

- **Voice Refine Phase 4.1 (parallel rotation cleanup + 3 user-visible
  bugs).** Phase 4 deleted rotation arrays in `lib/voice/` but missed the
  same anti-pattern in `lib/cognition.rs` and three pre-existing bugs in
  the response pipeline. Surfaced by the user's REPL smoke test:
  - `lib/cognition.rs` `emotional_response` (305-323) — collapsed the
    3-phrase warm rotation (`["That matters to me.", "I appreciate that.",
    "I'm glad we're talking."]` time-indexed) to a single Star-voice
    phrase: `"That matters to me."` Same for the 3-phrase supportive
    rotation → `"I'm here with you."` Both anchored to SOUL.md. New
    tests pin single-phrase behavior (3 new cognition tests, all pass).
  - `lib/conversation/mod.rs` `extract_name` — two bugs:
    1. Trailing punctuation leaked: `"I'm Zachary."` returned
       `Some("Zachary.")`, producing stored memory
       `"Zachary.'s name is Zachary."`. Now strips `.`, `,`, `!`, `?`
       and any non-alphanumeric/-/'` characters before the length check.
    2. Lowercase adverbs matched: `"I'm kinda bored"` returned
       `Some("Kinda")`, producing stored memory `"Kinda's name is Kinda"`.
       Now requires a capitalized first letter (real names) and
       additionally rejects a 25-word stopword list (kinda, sorta,
       really, just, ...) as belt-and-suspenders.
    8 new tests pin all branches (trailing punctuation variants,
    hyphenated names, capitalized stopwords, plain happy path).
  - `lib/research/mod.rs:230` — `"I explored {} but didn't find clear
    answers yet."` was a literal `.to_string()` with the `{}`
    placeholder never filled; the user's transcript showed
    `"I explored {} but didn't find clear answers yet."` reaching the
    REPL output. Now `format!`-interpolates `self.topic`.

  Net: Phase 4 was incomplete. 421 lib tests pass (was 410). Same
  release-build warnings as before (all pre-existing in unrelated
  modules).

- **Voice Refine Phase 1.2 (internal_state end-to-end + state-aware
  curiosity + intent-driven voice modulation).** Phase 1+2 from the plan
  had the *infrastructure* in place — `InternalState`, `classify()`,
  `ResponseIntent`, structured metacog intents — but the
  voice↔intent↔metacog triangle wasn't actually connected. This commit
  closes the loop:
  - `lib/voice/mod.rs` — new `apply_intent_modulation` pass in `speak()`,
    the missing end of the Phase 1 pipeline. Reads
    `internal_state.current_intent` and applies small targeted
    transformations: `Emotional` strips "I think"/"I guess" hedges
    (emotional answers should be direct), `Consciousness` + high
    uncertainty softens with "I think" (so Star doesn't overclaim about
    her own consciousness). 9 intents are pass-through — the reranker +
    personality style already cover them. This pass only does
    substitution/cleanup, never addition. **No new phrase templates**
    (the plan's "one well-chosen phrase per emotional state" rule).
  - `lib/voice/mod.rs::InternalState` — new `with_uncertainty(value)`
    builder so the runtime can layer real metacog uncertainty onto the
    cognitive-certainty baseline. Used in tests and the runtime wiring.
  - `lib/metacog/mod.rs` — new
    `curiosity_question_with_state(topic, state)` method. The original
    `curiosity_intent` derives the kind purely from
    `satisfaction`/`questions_asked`. The new method biases that floor
    based on `state.current_uncertainty` (high → demote Wondering to
    Confused) and `state.cognitive_emotional_valence` (high + low
    uncertainty → promote Wondering to Saturated). Same topic, same
    satisfaction, different internal state → different prose. This is
    the structural hook for the user's REPL to see "I keep hitting this
    wall with X" when Star is uncertain and "I want to go deeper on X"
    when Star is in a positive-valence state.
  - `lib/runtime/mod.rs` — `chat()` now layers real metacog uncertainty
    onto `internal_state.current_uncertainty`. The cognitive-certainty
    baseline (`1.0 - cognition.certainty`) is the floor; if
    `metacog.was_surprised()` is true, the value jumps to 0.7. The
    existing voice-uncertainty heuristic
    (`VoiceConfig::from_modifiers`) is unchanged — it now sees a
    *higher* uncertainty signal in surprise states. Future iterations
    can replace the surprise-binary with a continuous metacog reading.

  Net: voice and metacog now actually consume `internal_state`. The
  if-chain in `runtime::chat()` still fires (per the plan's "each
  handler migrates one at a time" follow-up), but the intent it
  classifies is now end-to-end visible to voice and metacog. 427 lib
  tests pass (was 421, +6 new). Same release-build warnings as before
  (pre-existing).

- **Voice Refine Phase 1.3 + Phase 3 wireup (typed Response from
  handlers + live charRNN reranker).** Phase 1.2 closed the
  *consumption* loop. Phase 1.3 starts the *construction* loop: each
  high-priority handler in `runtime::chat()` now returns
  `response_intent::Response { intent, body }` instead of a raw
  `String`. The intent is encoded at the handler site (not
  retroactively via `classify()`), so future dispatch-table
  conversions can collapse the if-chain to a single dispatch function.
  - `lib/runtime/mod.rs` — 9 new dispatch methods (`handle_how_are_you`,
    `handle_what_are_you_thinking`, `handle_are_you_sure`,
    `handle_did_you_collapse`, `handle_love`, `handle_capability_lookup`,
    `handle_aspiration`, `handle_tell_me_story`, `handle_tell_you_story`,
    `handle_hun`). 7 are `&self` (read-only); 2 (`handle_aspiration`
    mutates `cognition`, `handle_hun` mutates `learning`) take
    *field-level* references (`&mut CognitiveState`,
    `&mut LearningEngine`) instead of `&mut self` to avoid clashing
    with the `conversation` MutexGuard that `chat()` holds for its
    full body. The if-chain in `chat()` now calls each handler and
    extracts `.body` so `chat()`'s return type stays `Result<String>`
    for now.
  - `lib/language_model/intent_reranker.rs` — Phase 3 wireup that was
    started in an earlier turn but never committed. The runtime
    already calls the reranker (line 1486) but the supporting code
    was missing: `CharRnnBackend::load_default`, `RerankConfig` with
    `temperature`/`top_k`/`seed` fields, `rerank_with_config` method,
    per-call config built from `InternalState` per the plan's spec
    (temperature = 0.6 + 0.6 · novelty, top_k = 20 + 30 · novelty,
    seed = (valence · 1000).round()). 17 reranker tests pass. The
    runtime initializes the live `CharRnnBackend` against
    `ckpt_e28_b500.pt` (11MB) at startup; falls back to
    `MockReranker` if the model is unavailable. The rerank layer is
    additive — a rerank miss falls back to the raw body rather than
    ship a hallucination.

  Net: 9 of ~30+ if-chain handlers now produce typed `Response`
  values; the reranker that consumes them is fully wired. The
  reflection / research / curiosity handlers and the math / learn /
  teach blocks migrate in follow-up commits. 430 lib tests pass
  (same as Phase 1.2 — no new tests; smoke test is the validator
  here). Pre-existing flaky test in `lib/variation.rs` still fires
  1-in-3 (predates this commit).


## 2026-06-17

Daily backup: 6 files changed (2 modified, 4 added, 0 deleted).


## 2026-06-21

- Daily auto-sync: added plans/VOICE_REFINE_2026_06_21.md (voice refinement session notes).

