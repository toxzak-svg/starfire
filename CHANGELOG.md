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


## 2026-06-17

Daily backup: 6 files changed (2 modified, 4 added, 0 deleted).


## 2026-06-21

- Daily auto-sync: added plans/VOICE_REFINE_2026_06_21.md (voice refinement session notes).

