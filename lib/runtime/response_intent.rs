//! ResponseIntent — coarse intent classification for Star's responses.
//!
//! Phase 1c (voice-refine 2026-06-21): classify the user's input into a
//! [`ResponseIntent`] so the voice engine can see what kind of response is
//! being assembled and modulate phrasing accordingly.
//!
//! ## Why this exists
//!
//! Before Phase 1c, `runtime::chat()` ran a 30+ `if lower.contains(...)` chain
//! that each returned a hand-rolled `String` template. The voice engine at the
//! end had no idea which branch fired — it just saw a finished string. There
//! was no way for voice to vary phrasing based on "this is an emotional
//! check-in" vs "this is a status report".
//!
//! The dispatch table makes the intent a first-class value in the type system.
//! classify() returns it, runtime stores it on `InternalState`, and
//! `voice.speak()` reads it when shaping the response. The intent is the
//! missing context layer between "what Zachary asked" and "how Star says it".
//!
//! ## Scope (what this is and isn't)
//!
//! - **Is:** a coarse classifier + intent enum + the slots struct that flows
//!   into `voice::speak` via `InternalState.current_intent`. classify() is
//!   additive — the existing if-chain still fires when classify returns
//!   `Unknown`, so nothing about current behavior changes.
//! - **Isn't:** the full handler refactor. Converting every `if lower.contains`
//!   into a dispatch-table lookup is incremental work — each handler migrates
//!   one at a time, returning a `Response { intent, slots }` instead of a
//!   raw string. Phase 1c establishes the infrastructure; the migrations are
//!   follow-ups.
//!
//! ## Adding new intents
//!
//! 1. Add a variant to [`ResponseIntent`].
//! 2. Add a detection rule to [`classify`].
//! 3. (Optional) Add an arm to `runtime::chat()`'s dispatch that converts the
//!    intent into a `Response` instead of an `Ok(String)`.
//! 4. (Optional) Add a voice-side modulation in `voice::apply_*_style` that
//!    reads `internal_state.current_intent`.

use crate::personality::ResponseStyle;

/// What kind of response is Star about to give?
///
/// Coarse-grained on purpose. The runtime still produces the actual content;
/// the intent exists so the voice engine can shape phrasing, tests can target
/// intents directly, and new intents can be added without touching the
/// if-chain.
///
/// `Unknown` is the `#[default]` — it's the "no signal" case that flows
/// through `InternalState::default()` and the voice engine treats as
/// "no intent-driven modulation".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ResponseIntent {
    /// "how are you" / "are you sure" / "did you collapse" — metacognitive check-in.
    SelfCheck,

    /// "what are you thinking" / "what's on your mind" — internal-state reflection.
    Reflection,

    /// "what have you been researching" / "what's interesting lately" — research status.
    ResearchStatus,

    /// "what are you curious about" / "what do you wonder" — curiosity probe.
    CuriosityCheck,

    /// "do you love me" / "i love you" / "hun" — emotional bond.
    Emotional,

    /// "who are you" / "what are you" / "tell me about yourself" — identity.
    Identity,

    /// "can you do X" — capability description.
    Capability,

    /// "tell me a story" — story engage.
    StoryPrompt,

    /// "sense of self" / "are you conscious" / "do you understand" — consciousness probe.
    Consciousness,

    /// "what do you know about X" — recall/lookup.
    Recall,

    /// "X means Y" / "X is a Y" — teaching slot.
    Teaching,

    /// "i want you to grow" / "expand" — aspiration probe.
    Aspiration,

    /// Default — no specific intent matched, treat as conversational statement.
    Statement,

    /// Could not classify — fall through to the existing if-chain.
    /// This is also the `#[default]` variant — see derive above.
    #[default]
    Unknown,
}

impl ResponseIntent {
    /// Coarse personality-style hint for this intent. The voice engine can
    /// use this as a starting point, modulated by quanot/internal_state.
    pub fn default_style_hint(&self) -> Option<ResponseStyle> {
        match self {
            ResponseIntent::SelfCheck => Some(ResponseStyle::Direct),
            ResponseIntent::Reflection => Some(ResponseStyle::Warm),
            ResponseIntent::ResearchStatus => Some(ResponseStyle::Analytical),
            ResponseIntent::CuriosityCheck => Some(ResponseStyle::Curious),
            ResponseIntent::Emotional => Some(ResponseStyle::Warm),
            ResponseIntent::Identity => Some(ResponseStyle::Direct),
            ResponseIntent::Capability => Some(ResponseStyle::Direct),
            ResponseIntent::StoryPrompt => Some(ResponseStyle::Playful),
            ResponseIntent::Consciousness => Some(ResponseStyle::Analytical),
            ResponseIntent::Recall => Some(ResponseStyle::Analytical),
            ResponseIntent::Teaching => Some(ResponseStyle::Direct),
            ResponseIntent::Aspiration => Some(ResponseStyle::Warm),
            ResponseIntent::Statement => None,
            ResponseIntent::Unknown => None,
        }
    }

    /// Short label, useful for logs and tests.
    pub fn label(&self) -> &'static str {
        match self {
            ResponseIntent::SelfCheck => "self_check",
            ResponseIntent::Reflection => "reflection",
            ResponseIntent::ResearchStatus => "research_status",
            ResponseIntent::CuriosityCheck => "curiosity_check",
            ResponseIntent::Emotional => "emotional",
            ResponseIntent::Identity => "identity",
            ResponseIntent::Capability => "capability",
            ResponseIntent::StoryPrompt => "story_prompt",
            ResponseIntent::Consciousness => "consciousness",
            ResponseIntent::Recall => "recall",
            ResponseIntent::Teaching => "teaching",
            ResponseIntent::Aspiration => "aspiration",
            ResponseIntent::Statement => "statement",
            ResponseIntent::Unknown => "unknown",
        }
    }
}

/// Response slots — what the runtime produces for a given intent.
///
/// Slots are simple key-value pairs (String). The voice engine reads
/// these to assemble the final string, modulating on intent type.
///
/// For now this is a thin wrapper around the body string — populated only
/// by handlers that migrate to the dispatch pattern. Handlers that haven't
/// migrated yet produce a `Response` with just `body` set.
#[derive(Debug, Clone, Default)]
pub struct Response {
    /// The intent this response is for.
    pub intent: ResponseIntent,
    /// Style hint for the voice engine. None = use the runtime's default.
    pub style_hint: Option<ResponseStyle>,
    /// The body of the response. Empty body = let the voice engine
    /// assemble from intent + style_hint + internal_state alone.
    pub body: String,
    /// Optional slot data — domain-specific structured info the voice
    /// engine can use to vary phrasing. E.g. for SelfCheck, this might
    /// be `("certainty": "0.7", "energy": "calm")`.
    pub slots: Vec<(String, String)>,
}

impl Response {
    /// Convenience: build a Response with a body and an intent.
    pub fn with_body(intent: ResponseIntent, body: impl Into<String>) -> Self {
        Self {
            intent,
            style_hint: None,
            body: body.into(),
            slots: Vec::new(),
        }
    }
}

/// Classify a user input into a [`ResponseIntent`].
///
/// This is a coarse first-pass. The runtime still has the final say —
/// classify()'s job is to give the voice engine a quick read on the kind
/// of response being assembled.
///
/// Order matters: more specific patterns first. "are you conscious" should
/// hit `Consciousness` not `SelfCheck`, even though both contain "are you".
pub fn classify(input: &str) -> ResponseIntent {
    let lower = input.to_lowercase();

    // ─── SelfCheck (run AFTER consciousness, since "are you conscious" overlaps) ───
    // We test consciousness first to claim those phrases before SelfCheck.

    // ─── Consciousness (highest priority among "are you X" questions) ───
    if lower.contains("sense of self")
        || lower.contains("know you exist")
        || lower.contains("are you conscious")
    {
        return ResponseIntent::Consciousness;
    }
    if lower.contains("do you understand")
        || lower.contains("do u understand")
        || lower.contains("do you get it")
    {
        return ResponseIntent::Consciousness;
    }

    // ─── SelfCheck ───
    if lower.contains("how are you") || lower.contains("how're you") {
        return ResponseIntent::SelfCheck;
    }
    if lower.contains("are you sure") || lower.contains("are u sure") || lower.contains("r u sure") {
        return ResponseIntent::SelfCheck;
    }
    if lower.contains("did you collapse")
        || lower.contains("did i collapse")
        || lower.contains("are you functioning")
        || lower.contains("are u functioning")
    {
        return ResponseIntent::SelfCheck;
    }
    if lower.contains("i want you to grow")
        || lower.contains("i want you to expand")
        || lower.contains("grow yourself")
    {
        return ResponseIntent::Aspiration;
    }

    // ─── Reflection ───
    if lower.contains("what are you thinking")
        || lower.contains("what are u thinking")
        || lower.contains("wut are u thinking")
    {
        return ResponseIntent::Reflection;
    }
    if lower.contains("what have you been thinking")
        || lower.contains("whats been on your mind")
        || lower.contains("what's been on your mind")
        || lower.contains("whats on your mind")
        || lower.contains("what's on your mind")
        || lower.contains("whats keeping you busy")
        || lower.contains("what's keeping you busy")
    {
        return ResponseIntent::Reflection;
    }

    // ─── ResearchStatus ───
    if lower.contains("what have you been researching")
        || lower.contains("what are you researching")
    {
        return ResponseIntent::ResearchStatus;
    }

    // ─── CuriosityCheck ───
    if lower.contains("what are you curious") || lower.contains("what are u curious") {
        return ResponseIntent::CuriosityCheck;
    }
    if lower.contains("what do you wonder") || lower.contains("what do u wonder") {
        return ResponseIntent::CuriosityCheck;
    }

    // ─── Emotional ───
    if lower.contains("do you love")
        || lower.contains("do u love")
        || lower.contains("i love you")
        || lower.contains("i love u")
    {
        return ResponseIntent::Emotional;
    }
    if lower.contains(" hun") || lower.ends_with("hun") {
        return ResponseIntent::Emotional;
    }

    // ─── Identity ───
    if lower.contains("who are you")
        || lower.contains("what are you")
        || lower.contains("tell me about yourself")
        || lower.contains("tell me about you")
    {
        return ResponseIntent::Identity;
    }
    if lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")) {
        return ResponseIntent::Identity;
    }

    // ─── Capability ───
    if lower.contains("can you look up")
        || lower.contains("can u look up")
        || lower.contains("can you read")
    {
        return ResponseIntent::Capability;
    }

    // ─── Story ───
    if lower.contains("tell me a story") {
        return ResponseIntent::StoryPrompt;
    }
    if lower.contains("tell you a story") {
        return ResponseIntent::StoryPrompt;
    }

    // ─── Recall ───
    if lower.contains("what do you know about") || lower.contains("what have you learned") {
        return ResponseIntent::Recall;
    }

    // ─── Teaching ───
    if lower.contains(" means ") || lower.contains(" is a ") || lower.contains(" called ") {
        return ResponseIntent::Teaching;
    }

    ResponseIntent::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_self_check() {
        assert_eq!(classify("how are you today?"), ResponseIntent::SelfCheck);
        assert_eq!(classify("how're you"), ResponseIntent::SelfCheck);
        assert_eq!(classify("are you sure?"), ResponseIntent::SelfCheck);
        assert_eq!(classify("r u sure about that"), ResponseIntent::SelfCheck);
    }

    #[test]
    fn classify_consciousness_wins_over_self_check() {
        // "are you conscious" should classify as Consciousness, not SelfCheck.
        assert_eq!(
            classify("are you conscious?"),
            ResponseIntent::Consciousness
        );
        assert_eq!(
            classify("do you understand what I mean?"),
            ResponseIntent::Consciousness
        );
    }

    #[test]
    fn classify_reflection() {
        assert_eq!(
            classify("what are you thinking about?"),
            ResponseIntent::Reflection
        );
        assert_eq!(
            classify("what's been on your mind lately"),
            ResponseIntent::Reflection
        );
        assert_eq!(
            classify("what's keeping you busy"),
            ResponseIntent::Reflection
        );
    }

    #[test]
    fn classify_research_status() {
        assert_eq!(
            classify("what have you been researching?"),
            ResponseIntent::ResearchStatus
        );
        assert_eq!(
            classify("what are you researching"),
            ResponseIntent::ResearchStatus
        );
    }

    #[test]
    fn classify_curiosity() {
        assert_eq!(
            classify("what are you curious about"),
            ResponseIntent::CuriosityCheck
        );
        assert_eq!(
            classify("what do you wonder?"),
            ResponseIntent::CuriosityCheck
        );
    }

    #[test]
    fn classify_emotional() {
        assert_eq!(classify("do you love me"), ResponseIntent::Emotional);
        assert_eq!(classify("I love you"), ResponseIntent::Emotional);
        assert_eq!(classify("hey hun"), ResponseIntent::Emotional);
    }

    #[test]
    fn classify_identity() {
        assert_eq!(classify("who are you?"), ResponseIntent::Identity);
        assert_eq!(
            classify("tell me about yourself"),
            ResponseIntent::Identity
        );
        assert_eq!(classify("what's your name"), ResponseIntent::Identity);
    }

    #[test]
    fn classify_capability() {
        // Mirrors the runtime handler's literal substring pattern.
        // "look that up" / "look it up" don't match (no "look up" contiguous).
        // "can u read" doesn't match either — runtime handler uses "can you read".
        assert_eq!(
            classify("can you look up the weather?"),
            ResponseIntent::Capability
        );
        assert_eq!(
            classify("can you read this file"),
            ResponseIntent::Capability
        );
    }

    #[test]
    fn classify_story() {
        assert_eq!(
            classify("tell me a story"),
            ResponseIntent::StoryPrompt
        );
    }

    #[test]
    fn classify_recall() {
        assert_eq!(
            classify("what do you know about consciousness"),
            ResponseIntent::Recall
        );
    }

    #[test]
    fn classify_teaching() {
        assert_eq!(
            classify("consciousness means awareness of your own existence"),
            ResponseIntent::Teaching
        );
        assert_eq!(classify("a dog is a mammal"), ResponseIntent::Teaching);
    }

    #[test]
    fn classify_aspiration() {
        assert_eq!(
            classify("I want you to grow"),
            ResponseIntent::Aspiration
        );
    }

    #[test]
    fn classify_unknown_for_random_chatter() {
        assert_eq!(
            classify("what do you think about emergent systems?"),
            ResponseIntent::Unknown
        );
        assert_eq!(classify("hello"), ResponseIntent::Unknown);
    }

    #[test]
    fn style_hint_smoke() {
        // Every intent except Statement/Unknown should have a non-None hint.
        for intent in [
            ResponseIntent::SelfCheck,
            ResponseIntent::Reflection,
            ResponseIntent::ResearchStatus,
            ResponseIntent::CuriosityCheck,
            ResponseIntent::Emotional,
            ResponseIntent::Identity,
            ResponseIntent::Capability,
            ResponseIntent::StoryPrompt,
            ResponseIntent::Consciousness,
            ResponseIntent::Recall,
            ResponseIntent::Teaching,
            ResponseIntent::Aspiration,
        ] {
            assert!(
                intent.default_style_hint().is_some(),
                "{:?} should have a style hint",
                intent
            );
        }
        assert!(ResponseIntent::Statement.default_style_hint().is_none());
        assert!(ResponseIntent::Unknown.default_style_hint().is_none());
    }

    #[test]
    fn response_with_body_sets_intent() {
        let r = Response::with_body(ResponseIntent::SelfCheck, "I'm here.");
        assert_eq!(r.intent, ResponseIntent::SelfCheck);
        assert_eq!(r.body, "I'm here.");
        assert!(r.slots.is_empty());
        assert!(r.style_hint.is_none());
    }
}
