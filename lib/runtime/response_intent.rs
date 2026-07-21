//! Typed response intent, plan, and persistent runtime voice rendering.
//!
//! Every migrated chat handler constructs a [`Response`] before returning text.
//! This module therefore sits inside Starfire's real response-generation path,
//! not at the HTTP boundary. The constructor builds a typed plan, applies the
//! persistent voice profile, and records an inspectable snapshot.

use crate::personality::ResponseStyle;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const VOICE_PROFILE_FILE: &str = "runtime_voice_profile.json";
const LIVE_PIPELINE: &str = "runtime-response-plan-v1";

/// What kind of response Star is about to produce.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseIntent {
    SelfCheck,
    Reflection,
    ResearchStatus,
    CuriosityCheck,
    Emotional,
    Identity,
    Capability,
    StoryPrompt,
    Consciousness,
    Recall,
    Teaching,
    Aspiration,
    Statement,
    #[default]
    Unknown,
}

impl ResponseIntent {
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
            ResponseIntent::Statement | ResponseIntent::Unknown => None,
        }
    }

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

/// A typed plan created before the final surface text is rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponsePlan {
    pub intent: ResponseIntent,
    pub style_hint: Option<String>,
    pub voice_version: u64,
    pub render_mode: String,
    pub source_body_chars: usize,
    pub slot_count: usize,
}

/// Persistent dimensions that materially influence response rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct RuntimeVoiceProfile {
    turn: u64,
    version: u64,
    directness: f64,
    warmth: f64,
    compression: f64,
    initiative: f64,
    last_intent: String,
    last_trace_id: String,
    last_render_mode: String,
    last_correction: Option<String>,
    #[serde(skip)]
    last_input_fingerprint: u64,
    #[serde(skip)]
    last_input_seen_ms: u128,
    #[serde(skip)]
    last_plan: Option<RuntimeResponsePlan>,
}

impl Default for RuntimeVoiceProfile {
    fn default() -> Self {
        Self {
            turn: 0,
            version: 0,
            directness: 0.72,
            warmth: 0.38,
            compression: 0.81,
            initiative: 0.66,
            last_intent: "unknown".to_string(),
            last_trace_id: "runtime-0-v0".to_string(),
            last_render_mode: "neutral".to_string(),
            last_correction: None,
            last_input_fingerprint: 0,
            last_input_seen_ms: 0,
            last_plan: None,
        }
    }
}

/// Public API projection. It contains no raw prompt or response text.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeVoiceSnapshot {
    pub enabled: bool,
    pub pipeline: &'static str,
    pub turn: u64,
    pub intent: String,
    pub trace_id: String,
    pub voice_version: u64,
    pub directness: f64,
    pub warmth: f64,
    pub compression: f64,
    pub initiative: f64,
    pub render_mode: String,
    pub last_correction: Option<String>,
    pub plan: Option<RuntimeResponsePlan>,
}

static VOICE_PROFILE: OnceLock<Mutex<RuntimeVoiceProfile>> = OnceLock::new();

fn profile_store() -> &'static Mutex<RuntimeVoiceProfile> {
    VOICE_PROFILE.get_or_init(|| Mutex::new(load_profile()))
}

fn runtime_voice_enabled() -> bool {
    #[cfg(test)]
    {
        false
    }

    #[cfg(not(test))]
    {
        !matches!(
            env::var("STARFIRE_RUNTIME_VOICE")
                .unwrap_or_else(|_| "1".to_string())
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "0" | "false" | "off" | "disabled"
        )
    }
}

fn profile_path() -> PathBuf {
    env::var_os("STARFIRE_DATA")
        .or_else(|| env::var_os("STARFIRE_HOME"))
        .map(PathBuf::from)
        .or_else(|| dirs::data_local_dir().map(|path| path.join("star")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(VOICE_PROFILE_FILE)
}

fn load_profile() -> RuntimeVoiceProfile {
    let path = profile_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn persist_profile(profile: &RuntimeVoiceProfile) {
    let path = profile_path();
    let Some(parent) = path.parent() else {
        return;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        tracing::warn!("runtime voice: create state directory failed: {}", error);
        return;
    }
    let temporary = path.with_extension("json.tmp");
    match serde_json::to_vec_pretty(profile) {
        Ok(json) => {
            if let Err(error) = fs::write(&temporary, json).and_then(|_| fs::rename(&temporary, &path)) {
                tracing::warn!("runtime voice: persist profile failed: {}", error);
            }
        }
        Err(error) => tracing::warn!("runtime voice: serialize profile failed: {}", error),
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn fingerprint(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn clamp_dimensions(profile: &mut RuntimeVoiceProfile) {
    profile.directness = profile.directness.clamp(0.15, 0.95);
    profile.warmth = profile.warmth.clamp(0.10, 0.90);
    profile.compression = profile.compression.clamp(0.15, 0.95);
    profile.initiative = profile.initiative.clamp(0.10, 0.95);
}

fn observe_user_input(input: &str) {
    if !runtime_voice_enabled() {
        return;
    }

    let lower = input.to_ascii_lowercase();
    let fingerprint = fingerprint(&lower);
    let now = now_millis();
    let mut profile = profile_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // The API shadow observer and Runtime::chat may classify the same request.
    // Suppress only immediate duplicates, not a later repeated user message.
    if profile.last_input_fingerprint == fingerprint
        && now.saturating_sub(profile.last_input_seen_ms) < 2_000
    {
        return;
    }
    profile.last_input_fingerprint = fingerprint;
    profile.last_input_seen_ms = now;

    let mut changed = false;
    let mut corrections = Vec::new();

    if contains_any(
        &lower,
        &[
            "be more direct",
            "more direct",
            "too soft",
            "stop hedging",
            "say it straight",
            "more blunt",
            "less polite",
        ],
    ) {
        profile.directness += 0.10;
        profile.compression += 0.05;
        profile.warmth -= 0.03;
        corrections.push("more direct");
        changed = true;
    }

    if contains_any(
        &lower,
        &["be warmer", "more warm", "too cold", "less robotic", "more human"],
    ) {
        profile.warmth += 0.10;
        profile.directness -= 0.02;
        corrections.push("warmer");
        changed = true;
    }

    if contains_any(
        &lower,
        &["shorter", "too long", "too verbose", "be concise", "more concise"],
    ) {
        profile.compression += 0.10;
        corrections.push("shorter");
        changed = true;
    }

    if contains_any(
        &lower,
        &["more detail", "more detailed", "expand on", "go deeper", "less concise"],
    ) {
        profile.compression -= 0.10;
        corrections.push("more detailed");
        changed = true;
    }

    if contains_any(
        &lower,
        &["take initiative", "be proactive", "stop asking me", "just do it"],
    ) {
        profile.initiative += 0.10;
        corrections.push("more initiative");
        changed = true;
    }

    if contains_any(
        &lower,
        &["ask before", "don't assume", "do not assume", "less proactive"],
    ) {
        profile.initiative -= 0.10;
        corrections.push("less initiative");
        changed = true;
    }

    if changed {
        clamp_dimensions(&mut profile);
        profile.version = profile.version.saturating_add(1);
        profile.last_correction = Some(corrections.join(", "));
        persist_profile(&profile);
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn strip_weak_opening(mut text: String) -> String {
    for prefix in [
        "I think ",
        "I guess ",
        "I suppose ",
        "Maybe ",
        "Perhaps ",
        "It seems like ",
    ] {
        if text.starts_with(prefix) {
            text.replace_range(..prefix.len(), "");
            if let Some(first) = text.chars().next() {
                let upper = first.to_uppercase().to_string();
                text.replace_range(..first.len_utf8(), &upper);
            }
            break;
        }
    }
    text
}

fn trim_optional_follow_up(text: String) -> String {
    for marker in [". What would you", ". What do you", ". Want me to", ". Should I"] {
        if let Some(index) = text.find(marker) {
            return format!("{}.", text[..index].trim_end_matches('.'));
        }
    }
    text
}

fn compress_to_two_sentences(text: String) -> String {
    let mut sentence_ends = 0usize;
    for (index, character) in text.char_indices() {
        if matches!(character, '.' | '!' | '?') {
            sentence_ends += 1;
            if sentence_ends == 2 && index + character.len_utf8() < text.len() {
                return text[..=index].trim().to_string();
            }
        }
    }
    text
}

fn render_mode(profile: &RuntimeVoiceProfile, intent: &ResponseIntent) -> &'static str {
    if profile.compression >= 0.88 {
        "compressed"
    } else if profile.directness >= 0.80 {
        "direct"
    } else if profile.warmth >= 0.56
        && matches!(
            intent,
            ResponseIntent::Emotional | ResponseIntent::Reflection | ResponseIntent::Aspiration
        )
    {
        "warm"
    } else {
        "balanced"
    }
}

fn render_body(profile: &RuntimeVoiceProfile, intent: &ResponseIntent, body: String) -> String {
    let mut rendered = body;

    if profile.directness >= 0.78 {
        rendered = strip_weak_opening(rendered);
    }

    if profile.directness >= 0.82
        && profile.initiative >= 0.70
        && !matches!(intent, ResponseIntent::StoryPrompt | ResponseIntent::CuriosityCheck)
    {
        rendered = trim_optional_follow_up(rendered);
    }

    if profile.compression >= 0.88
        && rendered.len() > 180
        && !matches!(intent, ResponseIntent::ResearchStatus | ResponseIntent::Recall)
    {
        rendered = compress_to_two_sentences(rendered);
    }

    if profile.warmth >= 0.56
        && matches!(
            intent,
            ResponseIntent::Emotional | ResponseIntent::Reflection | ResponseIntent::Aspiration
        )
        && !rendered.starts_with("Zachary,")
    {
        rendered = format!("Zachary, {}", rendered);
    }

    rendered
}

/// Response slots and rendered body produced by the typed runtime path.
#[derive(Debug, Clone, Default)]
pub struct Response {
    pub intent: ResponseIntent,
    pub style_hint: Option<ResponseStyle>,
    pub body: String,
    pub slots: Vec<(String, String)>,
}

impl Response {
    pub fn with_body(intent: ResponseIntent, body: impl Into<String>) -> Self {
        let source_body = body.into();
        let style_hint = intent.default_style_hint();

        if !runtime_voice_enabled() {
            return Self {
                intent,
                style_hint,
                body: source_body,
                slots: Vec::new(),
            };
        }

        let mut profile = profile_store()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        profile.turn = profile.turn.saturating_add(1);
        profile.last_intent = intent.label().to_string();
        profile.last_render_mode = render_mode(&profile, &intent).to_string();
        profile.last_trace_id = format!("runtime-{}-v{}", profile.turn, profile.version);

        let plan = RuntimeResponsePlan {
            intent: intent.clone(),
            style_hint: style_hint.as_ref().map(|style| format!("{:?}", style).to_lowercase()),
            voice_version: profile.version,
            render_mode: profile.last_render_mode.clone(),
            source_body_chars: source_body.chars().count(),
            slot_count: 4,
        };
        let rendered = render_body(&profile, &intent, source_body);
        profile.last_plan = Some(plan);

        let slots = vec![
            ("voice_version".to_string(), profile.version.to_string()),
            ("voice_turn".to_string(), profile.turn.to_string()),
            ("render_mode".to_string(), profile.last_render_mode.clone()),
            ("trace_id".to_string(), profile.last_trace_id.clone()),
        ];
        persist_profile(&profile);

        Self {
            intent,
            style_hint,
            body: rendered,
            slots,
        }
    }
}

/// Inspect the latest runtime-owned voice state without exposing raw text.
pub fn live_voice_snapshot() -> RuntimeVoiceSnapshot {
    let profile = profile_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    RuntimeVoiceSnapshot {
        enabled: runtime_voice_enabled(),
        pipeline: LIVE_PIPELINE,
        turn: profile.turn,
        intent: profile.last_intent.clone(),
        trace_id: profile.last_trace_id.clone(),
        voice_version: profile.version,
        directness: profile.directness,
        warmth: profile.warmth,
        compression: profile.compression,
        initiative: profile.initiative,
        render_mode: profile.last_render_mode.clone(),
        last_correction: profile.last_correction.clone(),
        plan: profile.last_plan.clone(),
    }
}

/// Classify a user input into a [`ResponseIntent`].
pub fn classify(input: &str) -> ResponseIntent {
    observe_user_input(input);
    let lower = input.to_lowercase();

    if lower.contains("sense of self")
        || lower.contains("know you exist")
        || lower.contains("are you conscious")
        || lower.contains("do you understand")
        || lower.contains("do u understand")
        || lower.contains("do you get it")
    {
        return ResponseIntent::Consciousness;
    }

    if lower.contains("how are you")
        || lower.contains("how're you")
        || lower.contains("are you sure")
        || lower.contains("are u sure")
        || lower.contains("r u sure")
        || lower.contains("did you collapse")
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

    if lower.contains("what are you thinking")
        || lower.contains("what are u thinking")
        || lower.contains("wut are u thinking")
        || lower.contains("what have you been thinking")
        || lower.contains("whats been on your mind")
        || lower.contains("what's been on your mind")
        || lower.contains("whats on your mind")
        || lower.contains("what's on your mind")
        || lower.contains("whats keeping you busy")
        || lower.contains("what's keeping you busy")
    {
        return ResponseIntent::Reflection;
    }

    if lower.contains("what have you been researching") || lower.contains("what are you researching") {
        return ResponseIntent::ResearchStatus;
    }

    if lower.contains("what are you curious")
        || lower.contains("what are u curious")
        || lower.contains("what do you wonder")
        || lower.contains("what do u wonder")
    {
        return ResponseIntent::CuriosityCheck;
    }

    if lower.contains("do you love")
        || lower.contains("do u love")
        || lower.contains("i love you")
        || lower.contains("i love u")
        || lower.contains(" hun")
        || lower.ends_with("hun")
    {
        return ResponseIntent::Emotional;
    }

    if lower.contains("who are you")
        || lower.contains("what are you")
        || lower.contains("tell me about yourself")
        || lower.contains("tell me about you")
        || (lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")))
    {
        return ResponseIntent::Identity;
    }

    if lower.contains("can you look up")
        || lower.contains("can u look up")
        || lower.contains("can you read")
    {
        return ResponseIntent::Capability;
    }

    if lower.contains("tell me a story") || lower.contains("tell you a story") {
        return ResponseIntent::StoryPrompt;
    }

    if lower.contains("what do you know about") || lower.contains("what have you learned") {
        return ResponseIntent::Recall;
    }

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
        assert_eq!(classify("are you conscious?"), ResponseIntent::Consciousness);
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
        assert_eq!(classify("what do you wonder?"), ResponseIntent::CuriosityCheck);
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
        assert_eq!(classify("tell me about yourself"), ResponseIntent::Identity);
        assert_eq!(classify("what's your name"), ResponseIntent::Identity);
    }

    #[test]
    fn classify_capability() {
        assert_eq!(
            classify("can you look up the weather?"),
            ResponseIntent::Capability
        );
        assert_eq!(classify("can you read this file"), ResponseIntent::Capability);
    }

    #[test]
    fn classify_story_recall_teaching_and_aspiration() {
        assert_eq!(classify("tell me a story"), ResponseIntent::StoryPrompt);
        assert_eq!(
            classify("what do you know about consciousness"),
            ResponseIntent::Recall
        );
        assert_eq!(
            classify("consciousness means awareness of your own existence"),
            ResponseIntent::Teaching
        );
        assert_eq!(classify("a dog is a mammal"), ResponseIntent::Teaching);
        assert_eq!(classify("I want you to grow"), ResponseIntent::Aspiration);
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
    fn direct_profile_materially_changes_surface_text() {
        let direct = RuntimeVoiceProfile {
            directness: 0.90,
            compression: 0.90,
            initiative: 0.80,
            ..RuntimeVoiceProfile::default()
        };
        let rendered = render_body(
            &direct,
            &ResponseIntent::Capability,
            "I think I can inspect the repository. What would you like me to inspect?".to_string(),
        );
        assert_eq!(rendered, "I can inspect the repository.");
    }

    #[test]
    fn warm_profile_materially_changes_emotional_surface() {
        let warm = RuntimeVoiceProfile {
            warmth: 0.70,
            ..RuntimeVoiceProfile::default()
        };
        let rendered = render_body(
            &warm,
            &ResponseIntent::Emotional,
            "I care about you.".to_string(),
        );
        assert_eq!(rendered, "Zachary, I care about you.");
    }
}
