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


## 2026-06-17

Daily backup: 6 files changed (2 modified, 4 added, 0 deleted).


## 2026-06-21

- Daily auto-sync: added plans/VOICE_REFINE_2026_06_21.md (voice refinement session notes).

