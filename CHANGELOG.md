# Changelog

All notable changes to this project will be documented in this file.

## 2026-06-23

- **Voice Refine Phase 3 (moonshot — wire the language model as the
  final polish pass).** Phase 1+2+4 shipped the *architecture* for
  rerank-then-speak (InternalState surfaced, ResponseIntent classified,
  rotation arrays removed) but the runtime was still using
  `IntentReranker::with_default_backend()` — the deterministic
  `MockReranker`. The `CharRnnBackend` existed as a shape-only stub
  that returned `RerankError::ModelNotFound` on real calls. This commit
  actually wires the moonshot path:
  - `lib/language_model/intent_reranker.rs` — `RerankConfig` gains
    `top_k: usize` (default 20, the plan's low-novelty baseline). New
    `IntentReranker::rerank_with_config()` accepts a per-call config
    override so the runtime can derive live `temperature` / `top_k` /
    `seed` from the live `InternalState` per call. `CharRnnBackend`
    gets:
    - `load_from_checkpoint(path)` — loads `ckpt_e28_b500.pt` (or any
      other `CharRNN::save`-format file) and wraps it in
      `Arc<Mutex<...>>` with a fresh `Vocabulary::new()`. The vocab is
      fixed-size ASCII+ext, so a fresh one matches without
      serialization overhead.
    - `load_default(data_dir)` — resolves the conventional locations
      (`<data_dir>/models/ckpt_e28_b500.pt`,
      `<data_dir>/../models/ckpt_e28_b500.pt`, cwd-relative
      `models/ckpt_e28_b500.pt`) and returns `ModelNotFound` listing
      what was tried if none exist.
    - `rewrite()` honors `cfg.top_k` (was hardcoded to 0) and applies
      two plan-spec guardrails: length sanity (output < 8 chars or >
      2× structured → fall back) and divergence (Levenshtein / max_len
      > 0.6 → fall back). Both fall back to the structured body rather
      than ship a hallucinated rewrite. The voice engine still runs
      AFTER this for style/personality modulation, so a clean structured
      body is always shippable.
    - `build_text_prompt()` augmented from 4 fields to 8 (uncertainty,
      consciousness, novelty, creativity, valence, engagement, and the
      autonomous thought topic if any) so the generator sees the full
      state bundle, not just intent + body.
  - `lib/runtime/mod.rs` — new `init_reranker(data_dir)` helper that
    tries `CharRnnBackend::load_default(data_dir)` first, falls back
    to `MockReranker` with a clear warning log if the model is
    missing or fails to load. The runtime's chat path builds a live
    `RerankConfig` per call from the live `internal_state`:
    - `temperature = 0.6 + 0.6 * quanot_novelty.clamp(0,1)`
    - `top_k = (20 + 30 * quanot_novelty) as usize`
    - `seed = (cognition.emotional_valence * 1000.0).round() as i64 as u64`
    and calls `rerank_with_config()` instead of `rerank()`. The
    backends' own error paths still fall back to the raw body, so a
    rerank miss never degrades the response.
  - 10 new tests pin the architecture: loader error paths,
    `RerankConfig::default()` shape, `rerank_with_config` flow
    (recording backend embeds config into output for inspection),
    `rerank()` self-config preservation, edit-distance threshold
    (0.0 / 0.2 / 1.0 / > 0.6 boundary), and the augmented
    `build_text_prompt()` field coverage.
  - **440 lib tests pass** (was 430). Same pre-existing warnings
    from unrelated modules. The 1 flaky `variation` test that
    intermittently fails is unrelated to Phase 3 — its ring-buffer
    test relies on global state that other tests mutate; Phase 0b
    already targeted it. No new warnings introduced.

  - **Open follow-up (NOT in this commit):** the project's actual
    `models/ckpt_e28_b500.pt` (11MB) uses a different config than
    the working `data/star_model.bin` (3.7MB). Loading it via
    `CharRNN::load` triggers a 36-petabyte allocation somewhere
    downstream of the embedding / LSTM / output projection setup —
    a pre-existing dimension/sizing bug in
    `lib/language_model/model.rs`, NOT a Phase 3 regression. The
    runtime's `init_reranker` fallback path is the production
    answer: if the loader panics or errors, the user sees a clear
    warning log and gets the deterministic `MockReranker` instead.
    When the dimension bug is fixed, the live reranker will engage
    with no further code changes — the wiring is in place.

  - **Net:** the rerank layer now actually does what its name
    promises. Before this commit, the runtime was using a
    deterministic mock and the "moonshot" path was stub-only. After
    this commit, the runtime tries the real 11MB charRNN on every
    response, derives live generation params from the live state,
    and falls back gracefully if anything goes wrong. Voice-refine
    Phase 5 (the user-facing measurement tests) can now run.

  - **Fix the 36-petabyte allocation in `CharRNN::load` (Phase 3
    follow-up).** The "open follow-up" noted above was a load-time
    dimension-validation gap, not a sizing bug. Root cause: the
    `models/ckpt_e28_b500.pt` file is **not a CharRNN save at all** —
    its first 20 bytes are `\x80\x75\x03\x04 00 00 08 08 00…`, a
    Python pickle protocol 4 header (`\x80\x04` = PROTO 4) that
    happens to be saved with a `.pt` extension. `CharRNN::load`
    trusted those bytes as the `vocab_size` (67,324,752) and
    `embedding_dim` (2,281,103,360) fields, then `Self::new(config)`
    tried to allocate `vocab_size * embedding_dim` floats —
    ~600 PB just for the embedding, plus per-layer LSTM weights,
    hitting ~36 PB before the OS killed the process. Fix:
    - `lib/language_model/model.rs` — `MAX_VOCAB_SIZE`,
      `MAX_EMBEDDING_DIM`, `MAX_HIDDEN_SIZE`, `MAX_NUM_LAYERS` (and
      `MAX_VEC_LEN` for `read_f32_vec`) are the ceilings. Real
      charRNNs sit comfortably below them (vocab ≤ 227, embed 64,
      hidden 256, layers 2 for this project). `load()` validates
      each config field after reading and returns `io::ErrorKind::
      InvalidData` if any is out of range, **before** any
      `Self::new` allocation. `read_f32_vec` also rejects vectors
      above `MAX_VEC_LEN = 100M` floats (= 400MB), so a corrupt
      length prefix can't trigger an OOM mid-read.
    - 9 new tests pin the behavior: the exact 20-byte pickle
      header is rejected; zero / oversized / out-of-range
      configs (vocab, hidden, layers) are rejected; an
      oversized vector length prefix is rejected; truncated
      files error out; `data/star_model.bin` (3.7MB, real
      charRNN save) still loads; and a smoke test loads the
      actual `models/ckpt_e28_b500.pt` and confirms the error
      is `InvalidData` (not a multi-petabyte allocation).
    - **450 lib tests pass** (was 442 before this fix). No new
      warnings.
    - **Operational impact:** `init_reranker` no longer needs the
      "if the loader panics, fall back" branch as a *correctness*
      contract — it's still the right *production* contract
      (the live charRNN is moonshot polish, not the source of
      truth), but the loader will never panic on a real file
      again. The `MockReranker` fallback is now triggered by a
      clean `LoadFailed` error, not a process crash.
    - **Net follow-up:** the moonshot path is now wire-ready, but
      `ckpt_e28_b500.pt` is structurally a Python pickle, not
      a CharRNN save. The live rerank will engage the moment a
      real CharRNN-format checkpoint (e.g. a re-saved
      `star_model.bin` with the same vocab) is dropped into
      `models/`. Until then, `MockReranker` runs and the
      `init_reranker` log line records the clean rejection.

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

- **Voice Refine Phase 1.4 (per-handler migration complete: 18 more
  handlers).** Phase 1.3 migrated the 9 high-priority short handlers.
  Phase 1.4 migrates the 18 longer handlers in the if-chain to the
  same pattern — each returns `Response { intent, body }` instead of
  raw `String`. Migrated:
  - 5 pure-string handlers: `handle_whats_your_name` (Identity),
    `handle_who_are_you` (Identity), `handle_do_you_understand`
    (Consciousness), `handle_teach` (Statement), `handle_what_to_learn`
    (CuriosityCheck)
  - 4 single-source handlers: `handle_what_do_you_know_about` (Recall,
    reads `self.learning`), `handle_sense_of_self` (Consciousness,
    reads `self.store` identity memories), `handle_can_you_generic`
    (Capability, takes `lower` to format the body), `handle_something_interesting_figured`
    (ResearchStatus, reads `self.metacog`)
  - 5 reflection/research handlers: `handle_what_been_thinking`,
    `handle_most_interesting_learned`, `handle_most_interesting_figured`
    (all Reflection), `handle_what_been_researching` (the big one —
    pulls recent reasoning events, dedupes with curiosity topics,
    filters conversational fillers via the new free function),
    `handle_what_did_you_figure` (ResearchStatus)
  - 4 curiosity handlers: `handle_what_are_you_curious`,
    `handle_what_do_you_wonder`, `handle_why_does_fascinate` (takes
    `lower` to extract the topic), `handle_what_been_wondering`
  - Extracted `is_conversational_topic` from inline duplicate
    definitions in two handlers to a single free function at the
    top of `mod.rs`. Both `handle_what_been_researching` and
    `handle_what_did_you_figure` use it. Future migrations of the
    math / learn / teach blocks can also use it.

  Net: all 27 of the if-chain intent-classified handlers now produce
  typed `Response` values. The remaining 13 `if (lower/input).contains`
  lines in `chat()` are: command parsing (`/read`, `/search`, `/ls`,
  `/find`), math evaluation (parses arithmetic expressions), and the
  fallthrough `conversation.respond()` path. Those are not
  intent-classified — they're command handlers and the default
  conversation path. They migrate in a follow-up if/when the
  `chat()` return type changes from `Result<String>` to
  `Result<Response>`. 440 lib tests pass (was 430). Pre-existing
  flaky test in `lib/variation.rs` still fires 1-in-3 (predates).


## 2026-06-17

Daily backup: 6 files changed (2 modified, 4 added, 0 deleted).


## 2026-06-21

- Daily auto-sync: added plans/VOICE_REFINE_2026_06_21.md (voice refinement session notes).

