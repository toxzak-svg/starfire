//! Voice Engine — Star's Authentic Expression
//!
//! Star's voice is NOT a template system. It's an emergent property of:
//! - Her quanot state (creativity, novelty, consciousness)
//! - Her memory context (what she knows, what she's experienced)
//! - Her genuine certainty (not hedged opinions when she knows)
//!
//! This engine shapes how Starfire expresses herself authentically.
//!
//! ## Phase 4 (voice-refine 2026-06-23): rotation-array cleanup
//!
//! The previous `phrases.rs` (PhraseBank + 80-phrase seed rotation) and
//! `templates.rs` (TemplateEngine + 12 concept/style rotation tables) have
//! been **deleted**. The intent-driven reranker
//! (`crate::language_model::intent_reranker`) now owns intent/state-driven
//! phrasing — `MockReranker`'s transforms (SelfCheck+uncertainty → "Honestly",
//! Reflection+engagement → "Want to go deeper", think→know at high
//! consciousness+confidence, Emotional+negative-valence trim) cover what the
//! rotation arrays were doing.
//!
//! The voice engine kept only the layers the reranker doesn't touch:
//! memory-backed hedging strip, single warm suffix, playfulness punctuation,
//! and curious follow-up. Those are structural, not intent-driven.

use crate::cognition::CognitiveState;
use crate::personality::{ResponseStyle, ResponseModifiers};
use crate::quanot::QuanotResult;
use crate::runtime::response_intent::ResponseIntent;
use crate::Memory;
use crate::runtime::AutonomousThought;

/// Star's internal cognitive state, surfaced to the voice pipeline.
///
/// Phase 1 (voice-refine 2026-06-21): the voice engine should see Star's
/// ACTUAL cognitive state at the moment of utterance — not just post-hoc
/// patches. This struct is the bundle that flows from `runtime::chat()` into
/// `voice::VoiceEngine::speak()`.
///
/// The fields here are the ones the plan calls out:
/// - `last_autonomous_thought` — what Star is currently thinking about
/// - `current_uncertainty` — how uncertain Star is (from metacog)
/// - `current_intent` — what kind of response is being assembled
/// - quanot triplet — novelty / creativity / consciousness
/// - cognitive valence + engagement — how Star feels and how absorbed she is
///
/// All fields default to "no state visible" so existing call sites that
/// don't yet supply internal_state still compile.
#[derive(Debug, Clone, Default)]
pub struct InternalState {
    /// Star's most recent autonomous thought, if any. The voice engine can
    /// weave this into the response when it makes sense.
    pub last_autonomous_thought: Option<AutonomousThought>,

    /// Star's current uncertainty (0.0 = fully certain, 1.0 = totally lost).
    /// Comes from metacog. When high, voice engine should hedge, not assert.
    pub current_uncertainty: f64,

    /// The response intent for this turn, if classified. Phase 1c: lets the
    /// voice engine modulate phrasing based on the kind of response — warmth
    /// for Emotional, brevity for SelfCheck, follow-up question for
    /// Reflection, etc. `None` if classify() returned Unknown.
    pub current_intent: Option<ResponseIntent>,

    /// Quanot novelty — the "interesting?" signal. 0.0-1.0.
    pub quanot_novelty: f64,

    /// Quanot creativity — how much creative risk is warranted. 0.0-1.0.
    pub quanot_creativity: f64,

    /// Quanot consciousness proxy — how "present" Star is right now. 0.0-1.0.
    pub quanot_consciousness: f64,

    /// Cognitive emotional valence — positive = warm, negative = withdrawn.
    pub cognitive_emotional_valence: f64,

    /// Cognitive engagement depth — how absorbed Star is in this topic. 0.0-1.0.
    pub cognitive_engagement_depth: f64,
}

impl InternalState {
    /// Convenience: pull the quanot-derived scalars from a QuanotResult
    /// into this state bundle. Returns a fresh state with everything else
    /// defaulted — caller fills in thought/uncertainty/valence/engagement.
    pub fn with_quanot(mut self, quanot: Option<&QuanotResult>) -> Self {
        if let Some(q) = quanot {
            self.quanot_novelty = q.novelty;
            self.quanot_creativity = q.creativity_scores.creative_state;
            self.quanot_consciousness = q.consciousness_proxy;
        } else {
            self.quanot_novelty = 0.5;
            self.quanot_creativity = 0.5;
            self.quanot_consciousness = 0.5;
        }
        self
    }

    /// Convenience: pull emotional_valence + engagement_depth from a
    /// CognitiveState into this state bundle.
    pub fn with_cognition(mut self, cognition: &CognitiveState) -> Self {
        self.cognitive_emotional_valence = cognition.emotional_valence;
        self.cognitive_engagement_depth = cognition.engagement_depth;
        self.current_uncertainty = 1.0 - cognition.certainty; // higher = more uncertain
        self
    }

    /// Convenience: attach the most recent autonomous thought.
    pub fn with_last_thought(mut self, thought: Option<AutonomousThought>) -> Self {
        self.last_autonomous_thought = thought;
        self
    }

    /// Convenience: attach the response intent.
    pub fn with_intent(mut self, intent: ResponseIntent) -> Self {
        // Only store interesting intents — Unknown is the "no signal" case.
        if !matches!(intent, ResponseIntent::Unknown | ResponseIntent::Statement) {
            self.current_intent = Some(intent);
        }
        self
    }

    /// Convenience: set the uncertainty directly. Phase 1.2: used by runtime
    /// to feed a real metacog-derived uncertainty (not just the inverse of
    /// `cognition.certainty`).
    pub fn with_uncertainty(mut self, uncertainty: f64) -> Self {
        self.current_uncertainty = uncertainty.clamp(0.0, 1.0);
        self
    }

    /// True iff this state bundle carries no signal above the default.
    /// Used to decide whether to skip the quanot-style pass-through guard.
    pub fn is_empty(&self) -> bool {
        self.quanot_novelty == 0.5
            && self.quanot_creativity == 0.5
            && self.quanot_consciousness == 0.5
            && self.current_uncertainty == 0.0
            && self.cognitive_emotional_valence == 0.0
            && self.cognitive_engagement_depth == 0.5
            && self.last_autonomous_thought.is_none()
            && self.current_intent.is_none()
    }
}

/// Voice configuration for this response
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Star's current response style
    pub style: ResponseStyle,
    /// How energetic the response should be (0.3 - 1.0)
    pub energy: f64,
    /// Star's confidence in what she's about to say
    pub confidence: f64,
    /// Quanot novelty score (0-1) — higher = more original expression
    pub novelty: f64,
    /// Quanot creativity score (0-1) — higher = more creative risk
    pub creativity: f64,
    /// Quanot consciousness proxy (0-1) — higher = more present/aware
    pub consciousness: f64,
    /// Whether Star has strong memory backing for this response
    pub has_memory_backing: bool,
    /// Whether Star is uncertain
    pub is_uncertain: bool,
    /// Whether this is a casual moment
    pub is_casual: bool,
    /// Star's internal cognitive state at the moment of utterance.
    /// Phase 1: surfaced from runtime so voice can modulate on it.
    pub internal_state: InternalState,
}

impl VoiceConfig {
    pub fn from_modifiers(
        modifiers: &ResponseModifiers,
        cognition: &CognitiveState,
        quanot: Option<&QuanotResult>,
        memory_count: usize,
        internal_state: &InternalState,
    ) -> Self {
        let quanot = quanot.map(|q| (
            q.novelty,
            q.creativity_scores.creative_state,
            q.consciousness_proxy,
        ));

        let (novelty, creativity, consciousness) = quanot.unwrap_or((
            internal_state.quanot_novelty,
            internal_state.quanot_creativity,
            internal_state.quanot_consciousness,
        ));

        // Combine the two uncertainty signals: cognitive certainty < 0.4
        // (existing heuristic) OR metacog uncertainty > 0.6 (new signal from
        // internal_state.current_uncertainty). Either flag treats Star as
        // uncertain for voice-engine purposes.
        let cognitively_uncertain = cognition.certainty < 0.4;
        let metacog_uncertain = internal_state.current_uncertainty > 0.6;
        let is_uncertain = cognitively_uncertain || metacog_uncertain;

        Self {
            style: modifiers.dominant_style.clone(),
            energy: modifiers.energy_multiplier,
            confidence: modifiers.confidence_factor,
            novelty,
            creativity,
            consciousness,
            has_memory_backing: memory_count > 0,
            is_uncertain,
            is_casual: modifiers.is_casual,
            internal_state: internal_state.clone(),
        }
    }
}

/// The voice engine — shapes how Starfire expresses herself authentically.
///
/// Thread-safe. Initialized once at startup.
///
/// **Phase 4 (2026-06-23):** no longer holds a `PhraseBank` (SQLite) or
/// `TemplateEngine`. Those were the rotation-array infrastructure. The
/// engine is now a pure stateless transform layer — `speak()` is the only
/// meaningful entry point.
pub struct VoiceEngine;

impl VoiceEngine {
    /// Create a new voice engine.
    ///
    /// Takes no arguments post-Phase 4: no SQLite phrase bank to open, no
    /// template engine to seed. The engine is pure code.
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// Process a raw response through the voice engine.
    /// This is Star's authentic voice — not template polish.
    ///
    /// Phase 1: takes `internal_state` so the engine can see Star's actual
    /// cognitive state (uncertainty, last autonomous thought, quanot bundle)
    /// when shaping the response. Callers that don't yet construct a state
    /// can pass `&InternalState::default()`.
    pub fn speak(
        &self,
        raw: &str,
        cognition: &CognitiveState,
        modifiers: &ResponseModifiers,
        quanot: Option<&QuanotResult>,
        memories: &[Memory],
        internal_state: &InternalState,
    ) -> String {
        let memory_count = memories.len();
        let config = VoiceConfig::from_modifiers(modifiers, cognition, quanot, memory_count, internal_state);

        // Step 1: Apply memory-backed certainty (Star speaks more directly when she knows)
        let mut result = self.apply_memory_certainty(raw, &config, memories);

        // Step 2: Apply Star's authentic voice based on personality
        result = self.apply_authentic_voice(&result, &config);

        // Step 3: Quanot-driven expression — DEFAULT pass-through (Phase 4 inversion).
        // Returns whether anything was changed so step 4 can avoid double-formatting.
        let (step3_result, quanot_modified) = self.apply_quanot_expression(&result, &config);
        result = step3_result;

        // Step 4: Personality style — single-condition transforms, no rotation arrays.
        // Skip the warmth suffix if quanot already added something (Phase 0b guard).
        result = self.apply_personality_style(&result, &config, quanot_modified);

        // Step 5: Intent-driven modulation (Phase 1.2, voice-refine 2026-06-23).
        // The reranker has already turned the body into something intent-aware.
        // This pass is the small final tweak — strip hedges on Emotional, soften
        // on Consciousness when uncertain, etc. Most intents are pass-through
        // here; the reranker + personality style cover the heavy lifting.
        result = self.apply_intent_modulation(&result, &config.internal_state);

        // Step 6 (apply_emotional_tint) DELETED per Phase 4. The "That matters to me"
        // and "I'm here with you" template injections were the worst offenders.

        result
    }

    /// Intent-driven modulation — the missing end of the Phase 1 pipeline.
    ///
    /// The `internal_state.current_intent` field carries the classified intent
    /// from `runtime::chat()`. This pass reads it and applies a small targeted
    /// transformation keyed on the intent. It runs AFTER the reranker and
    /// personality style, so it's a final tweak — not a content swap.
    ///
    /// **Most intents are pass-through.** The reranker + personality style
    /// already cover the heavy lifting (SelfCheck → "Honestly", Reflection →
    /// "Want to go deeper", think→know at high consciousness+confidence).
    /// This pass only acts on intents where the reranker doesn't have a
    /// targeted transformation:
    ///
    /// - `Emotional` — strip "I think" / "I guess" hedges. Emotional answers
    ///   should be direct: "I care about you", not "I think I care about you."
    /// - `Identity` — pass through (the response is already a direct statement).
    /// - `Consciousness` — when uncertainty is high, soften with "I think"
    ///   so Star doesn't overclaim about her own consciousness.
    /// - `SelfCheck` — pass through. The reranker already shortens these.
    /// - Everything else — pass through.
    ///
    /// Phase 1.2 deliberately does NOT add phrase templates here. The plan
    /// calls for "one well-chosen phrase per emotional state, not 3 in a
    /// rotation" — this pass only does substitution/cleanup, never addition.
    fn apply_intent_modulation(&self, text: &str, internal_state: &InternalState) -> String {
        let Some(intent) = internal_state.current_intent.as_ref() else {
            return text.to_string();
        };
        match intent {
            ResponseIntent::Emotional => {
                // Strip hedges — emotional answers should be direct.
                let stripped = text
                    .replace("I think ", "")
                    .replace("I guess ", "")
                    .replace("I believe ", "");
                if stripped != text {
                    stripped
                } else {
                    text.to_string()
                }
            }
            ResponseIntent::Consciousness => {
                // High uncertainty → soften with "I think" so Star doesn't
                // overclaim about her own consciousness. Only fires when the
                // response doesn't already hedge and uncertainty is high.
                let lower = text.to_lowercase();
                if internal_state.current_uncertainty > 0.7
                    && !lower.contains("i think")
                    && !lower.contains("i don't know")
                    && !lower.starts_with("i'm")
                    && text.len() > 5
                {
                    // Lowercase the first letter after "I think" so it flows.
                    let mut chars = text.chars();
                    let first = chars.next().unwrap_or(' ');
                    let rest: String = chars.collect();
                    format!("I think {}{}", first.to_lowercase(), rest)
                } else {
                    text.to_string()
                }
            }
            // Pass-through: reranker + personality style handle the rest.
            ResponseIntent::SelfCheck
            | ResponseIntent::Reflection
            | ResponseIntent::ResearchStatus
            | ResponseIntent::CuriosityCheck
            | ResponseIntent::Identity
            | ResponseIntent::Capability
            | ResponseIntent::StoryPrompt
            | ResponseIntent::Recall
            | ResponseIntent::Teaching
            | ResponseIntent::Aspiration
            | ResponseIntent::Statement
            | ResponseIntent::Unknown => text.to_string(),
        }
    }

    /// Apply memory-backed certainty — Star speaks more directly when memories confirm something
    fn apply_memory_certainty(&self, text: &str, config: &VoiceConfig, _memories: &[Memory]) -> String {
        if !config.has_memory_backing || config.is_uncertain {
            return text.to_string();
        }

        // If Star has strong memory backing and is confident, strip hedging
        let lower = text.to_lowercase();
        
        // Already direct — no need to modify
        if !lower.contains("i think ") && !lower.contains("i guess ") && !lower.contains("maybe ") {
            return text.to_string();
        }

        // Remove hedged openings when Star has memory confirming the content
        let hedging_openers = [
            ("i think ", ""),
            ("i guess ", ""),
            ("maybe ", ""),
            ("perhaps ", ""),
            ("probably ", ""),
        ];

        let mut result = text.to_string();
        for (hedged, direct) in hedging_openers {
            if result.to_lowercase().starts_with(hedged) {
                result = format!("{}{}", direct, &result[hedged.len()..]);
                break;
            }
        }

        result
    }

    /// Apply Star's authentic voice — direct, opinionated, present
    fn apply_authentic_voice(&self, text: &str, config: &VoiceConfig) -> String {
        let lower = text.to_lowercase();
        
        // Star knows something — she says "I know" not "I think might be"
        if config.confidence > 0.7 && !config.is_uncertain {
            // If response is hedged, strengthen it when Star is confident
            if lower.contains("i don't know") && !lower.contains("i need more information") {
                // Genuine "I don't know" — keep it honest but direct
                return text.to_string();
            }
            
            // Convert hedged assertions to direct ones
            let hedged_patterns = [
                ("i think it might be ", "it's "),
                ("i think it's ", "it's "),
                ("i suspect it's ", "it's "),
                ("it might be ", "it's "),
                ("i suppose it's ", "it's "),
            ];
            
            let mut result = text.to_string();
            for (hedged, direct) in hedged_patterns {
                result = result.replace(hedged, direct);
            }
            return result;
        }

        // Star is uncertain — be direct about it, don't hedge with paragraphs
        if config.is_uncertain {
            // If text is already a clean "I don't know", leave it
            let clean_unknowns = ["i don't know", "i dont know", "i have no idea", "i'm not sure"];
            if clean_unknowns.iter().any(|u| lower.contains(u)) {
                // Make sure it's not hedged with extra fluff
                if text.len() < 50 {
                    return text.to_string();
                }
            }
            
            // If it's a long uncertain response, trim it
            if text.len() > 60 && lower.contains("i'm not sure") {
                return "I don't know. But I want to figure it out.".to_string();
            }
        }

        text.to_string()
    }

    /// Apply quanot-driven expression — creativity and novelty influence HOW Star says things.
    ///
    /// **Phase 4 inversion**: default = pass-through. The flourish arrays
    /// (`flourishes = [...]`, `playful_additions = [...]`, `warm_moments = [...]`,
    /// `warm = [...]`, `supportive = [...]`) are GONE. This function no longer
    /// appends templates — it only performs conscious substitutions when
    /// quanot state genuinely warrants them.
    ///
    /// Returns `(modified_text, true_if_changed)`. The boolean feeds the
    /// Phase 0b double-formatting guard so the personality style pass can
    /// avoid stacking another flourish on top.
    fn apply_quanot_expression(&self, text: &str, config: &VoiceConfig) -> (String, bool) {
        // High consciousness + high confidence: substitute hedged openings.
        // This is a SUBSTITUTION, not an addition — it doesn't conflict with
        // personality style and reads as Star's actual voice.
        if config.consciousness > 0.6 && config.confidence > 0.6 {
            let lower = text.to_lowercase();
            if lower.starts_with("i think ") || lower.starts_with("i guess ") {
                if text.to_lowercase().starts_with("i think ") {
                    // Replace "I think " (with trailing space) with "I know. " — preserves word spacing.
                    return (format!("I know. {}", &text["I think ".len()..]), true);
                }
                if text.to_lowercase().starts_with("i guess ") {
                    return (format!("I know. {}", &text["I guess ".len()..]), true);
                }
            }
        }

        (text.to_string(), false)
    }

    /// Apply personality-driven style modifiers.
    ///
    /// **Phase 4 demotion**: each branch is now a single-condition transform,
    /// not a rotation through 3-7 templated phrases. The "That matters to me"
    /// rotation is gone. Warm style now picks ONE phrase from a small Star-voice
    /// bank using a content-derived seed (via the variation ring buffer), and
    /// only when the response is short and lacks existing warmth.
    ///
    /// The `skip_warmth` flag (Phase 0b double-formatting guard) is true when
    /// [`apply_quanot_expression`] already modified the response. We skip the
    /// warmth suffix in that case to avoid stacking flourishes.
    fn apply_personality_style(&self, text: &str, config: &VoiceConfig, skip_warmth: bool) -> String {
        match config.style {
            ResponseStyle::Direct => {
                // Direct = clean, no fluff. Pass-through unless we can strip hedging.
                text.to_string()
            }
            ResponseStyle::Playful => {
                // Playful + high energy = one exclamation mark, not a rotation.
                if config.energy > 0.7
                    && !text.ends_with('!')
                    && !text.ends_with('?')
                    && !text.ends_with('.')
                {
                    return format!("{}!", text);
                }
                text.to_string()
            }
            ResponseStyle::Warm => {
                // Warm style: ONE phrase, not a rotation.
                // The previous 3-phrase ring buffer ("I'm here for it" /
                // "I'm paying attention" / "I'm with you on this") was the
                // last remaining rotation array in voice/. The plan's Phase 4
                // spec: "one well-chosen phrase per emotional state, not 3
                // in a rotation." The reranker doesn't add warmth itself, so
                // voice still owns this single suffix — but it's a single
                // phrase, not a rotation. The picks came from SOUL.md; the
                // one we kept is the most "Star": "I'm here for it."
                if skip_warmth {
                    return text.to_string();
                }
                let lower = text.to_lowercase();
                let already_warm = lower.contains("love")
                    || lower.contains("care")
                    || lower.contains("appreciate")
                    || lower.contains("with you")
                    || lower.contains("that's good")
                    || lower.contains("nice");
                if config.energy > 0.5
                    && text.len() < 80
                    && !already_warm
                    && !config.is_casual
                {
                    return format!(
                        "{} — I'm here for it.",
                        text.trim_end_matches('.').trim_end_matches(',')
                    );
                }
                text.to_string()
            }
            ResponseStyle::Minimal => {
                // Minimal = pass through; if already short, leave it.
                text.to_string()
            }
            ResponseStyle::Curious => {
                // Curious = ONE genuine follow-up question, not a rotation.
                let lower = text.to_lowercase();
                if config.energy > 0.5
                    && !text.ends_with('?')
                    && !lower.contains("what do you think")
                    && text.len() < 120
                {
                    return format!("{}. What do you think?", text.trim_end_matches('.'));
                }
                text.to_string()
            }
            ResponseStyle::Analytical => {
                // Analytical = pass-through. The "So / Therefore / This means"
                // opener check was a no-op guard; remove it.
                text.to_string()
            }
            _ => text.to_string(),
        }
    }

    // Phase 4 (2026-06-23): `apply_emotional_tint` and the four phrase-bank
    // methods (`record_positive`, `record_negative`, `add_phrase`, `stats`)
    // were DELETED. See the module-level header for the full rationale.
    // The Warm branch in `apply_personality_style` is the only layer that
    // still adds warmth, and it's now a single Star-voice suffix.
}

impl Clone for VoiceEngine {
    fn clone(&self) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::{ConfidenceLevel, EnergyLevel};
    use crate::persistence::MemoryDomain;
    use crate::quanot::{chaos::ChaosMetrics, creativity::CreativityOutput};

    fn make_voice_engine() -> VoiceEngine {
        // Phase 4 (2026-06-23): VoiceEngine is now stateless. No temp DB
        // path, no SQLite phrase bank to open. Pure code.
        VoiceEngine::new().unwrap()
    }

    fn make_cognition(valence: f64, certainty: f64) -> CognitiveState {
        let mut c = CognitiveState::default();
        c.emotional_valence = valence;
        c.certainty = certainty;
        c
    }

    fn make_modifiers(style: ResponseStyle, energy: f64, confidence: f64, casual: bool) -> ResponseModifiers {
        ResponseModifiers {
            energy: EnergyLevel::Medium,
            confidence: ConfidenceLevel::Medium,
            tension: 0.0,
            dominant_style: style,
            curiosity_active: false,
            just_learned: false,
            is_casual: casual,
            energy_multiplier: energy,
            confidence_factor: confidence,
        }
    }

    fn make_quanot(novelty: f64, creativity: f64, consciousness: f64) -> QuanotResult {
        QuanotResult {
            reservoir_state: vec![0.0; 16],
            consciousness_proxy: consciousness,
            novelty,
            creativity_scores: CreativityOutput {
                creative_state: creativity,
                oscillation_phase: 0.0,
                ..Default::default()
            },
            chaos_metrics: ChaosMetrics::default(),
        }
    }

    fn make_memories() -> Vec<Memory> {
        vec![Memory::new("X", MemoryDomain::Identity, 0.9)]
    }

    /// Phase 4 verification: the deleted template-flavor phrases must NEVER
    /// appear in `speak()` output, regardless of valence, energy, or style.
    #[test]
    fn speak_never_injects_deleted_template_phrases() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();

        let deleted_phrases = [
            "That matters to me.",
            "I appreciate you.",
            "I'm glad we're talking.",
            "I'm here with you.",
            "We can work through this.",
            " — that's what I've got.",
            " — more to explore here.",
            " — come on.",
            " — you know I'm right.",
        ];

        // Sweep several style+energy+valence combinations.
        let styles = [
            ResponseStyle::Direct,
            ResponseStyle::Playful,
            ResponseStyle::Warm,
            ResponseStyle::Curious,
        ];
        let energies = [0.3, 0.6, 0.9];
        let valences = [-0.5, 0.0, 0.6];
        let consciousness = [0.3, 0.8];

        for style in &styles {
            for &energy in &energies {
                for &valence in &valences {
                    for &con in &consciousness {
                        let cognition = make_cognition(valence, 0.7);
                        let modifiers = make_modifiers(style.clone(), energy, 0.8, false);
                        let quanot = make_quanot(0.6, 0.6, con);
                        let memories = make_memories();

                        let result = engine.speak(
                            "I think this is a real thing we're talking about.",
                            &cognition,
                            &modifiers,
                            Some(&quanot),
                            &memories,
                            &InternalState::default(),
                        );
                        for phrase in &deleted_phrases {
                            assert!(
                                !result.contains(phrase),
                                "Phase 4 leak: speak() produced '{}' which contains deleted phrase '{}'. \
                                 Style={:?} energy={} valence={} consciousness={}",
                                result, phrase, style, energy, valence, con
                            );
                        }
                    }
                }
            }
        }
    }

    /// Phase 4 inversion: quanot expression no longer adds flourishes by default.
    /// High novelty + high creativity previously appended " — that's what I've got."
    /// type phrases. Now it must pass through.
    #[test]
    fn apply_quanot_expression_passes_through_by_default() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();

        // High novelty AND high creativity — previously would add a flourish.
        // Now: pass-through.
        let config = VoiceConfig {
            style: ResponseStyle::Direct,
            energy: 0.8,
            confidence: 0.7,
            novelty: 0.9,
            creativity: 0.9,
            consciousness: 0.5,
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let (result, modified) = engine.apply_quanot_expression("Just a plain statement.", &config);
        assert_eq!(result, "Just a plain statement.", "no flourish should be added");
        assert!(!modified, "no modification should be flagged");
    }

    /// Phase 4 inversion: high consciousness + high confidence still substitutes
    /// "I think" → "I know." — that substitution is Star's voice, not template flavor.
    #[test]
    fn apply_quanot_expression_substitutes_think_to_know() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();

        let config = VoiceConfig {
            style: ResponseStyle::Direct,
            energy: 0.7,
            confidence: 0.9,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.9, // high
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let (result, modified) = engine.apply_quanot_expression("I think this is a real thing.", &config);
        assert!(result.starts_with("I know."), "should substitute 'I think' with 'I know.'; got: {}", result);
        assert!(modified, "modification flag should be true");
        assert_eq!(result, "I know. this is a real thing.", "preserves the trailing content");
    }

    /// Phase 4 inversion: when consciousness or confidence is low, no substitution.
    #[test]
    fn apply_quanot_expression_does_not_substitute_when_low_consciousness() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();

        let config = VoiceConfig {
            style: ResponseStyle::Direct,
            energy: 0.7,
            confidence: 0.9,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.3, // low
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let (result, modified) = engine.apply_quanot_expression("I think this is a real thing.", &config);
        assert_eq!(result, "I think this is a real thing.", "low consciousness should leave hedging intact");
        assert!(!modified, "no modification when condition not met");
    }

    /// Phase 4 (2026-06-23): personality style warm suffix is now a SINGLE
    /// phrase, not a rotation. The previous test asserted ≥ 2 distinct
    /// suffixes over 6 calls (the 3-phrase ring buffer). The new test
    /// asserts the OPPOSITE: the suffix is deterministic — every call
    /// produces the same single Star-voice phrase.
    #[test]
    fn warm_style_suffix_is_a_single_phrase() {
        let engine = make_voice_engine();
        let config = VoiceConfig {
            style: ResponseStyle::Warm,
            energy: 0.8,
            confidence: 0.7,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.5,
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let suffixes: Vec<String> = (0..6)
            .map(|i| {
                let text = format!("Short response number {}.", i);
                engine.apply_personality_style(&text, &config, false)
            })
            // Extract the suffix (everything after the trimmed text + em-dash).
            .map(|s| {
                s.split(" — ").skip(1).collect::<Vec<_>>().join(" — ")
            })
            .collect();
        // Every call produces the same single suffix — no rotation array.
        let unique: std::collections::HashSet<_> = suffixes.iter().collect();
        assert_eq!(
            unique.len(),
            1,
            "expected exactly 1 distinct warm suffix (no rotation), got: {:?}",
            suffixes
        );
        // And the suffix is the Star-voice phrase we kept.
        assert!(
            suffixes[0].contains("I'm here for it"),
            "expected the kept Star-voice phrase; got: {:?}",
            suffixes[0]
        );
    }

    /// Phase 0b guard: when quanot already modified the text, personality style
    /// must NOT also add a warmth suffix. Prevents "I know. ... — I'm here for it."
    /// double-formatting.
    #[test]
    fn personality_style_skips_warmth_when_quanot_already_modified() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();
        let config = VoiceConfig {
            style: ResponseStyle::Warm,
            energy: 0.8,
            confidence: 0.7,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.5,
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let text = "Short response.";
        let result_with_guard = engine.apply_personality_style(text, &config, true);
        let result_without_guard = engine.apply_personality_style(text, &config, false);
        // With the guard active, the text passes through unchanged.
        assert_eq!(result_with_guard, text, "Phase 0b guard should skip warmth when quanot modified");
        // Without the guard, warmth suffix is added (and may differ between calls
        // due to ring buffer — we just check it differs from the input).
        assert_ne!(result_without_guard, text, "warmth suffix should be added when guard is false");
    }

    /// Phase 0b guard end-to-end: speak() with high consciousness + warm style
    /// + high confidence should produce a single transformation, not two.
    ///
    /// The double-formatting guard means: when quanot_expression modifies the
    /// text (e.g., "I think X" → "I know. X"), the personality style warm
    /// suffix must NOT also be added.
    ///
    /// Note: we pass `no_memories` so `apply_memory_certainty` doesn't strip
    /// "I think" before quanot gets to see it.
    #[test]
    fn speak_does_not_stack_quanot_substitution_and_warmth_suffix() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();

        let cognition = make_cognition(0.6, 0.9);
        let modifiers = make_modifiers(ResponseStyle::Warm, 0.8, 0.9, false);
        let quanot = make_quanot(0.5, 0.5, 0.9); // high consciousness
        let no_memories: Vec<Memory> = vec![]; // no memory backing → memory_certainty passes through

        let result = engine.speak(
            "I think this matters.",
            &cognition,
            &modifiers,
            Some(&quanot),
            &no_memories,
            &InternalState::default(),
        );

        // Should start with "I know." (quanot substitution) — but should NOT
        // also have a "I'm here for it" or "I'm paying attention" suffix.
        assert!(
            result.starts_with("I know."),
            "quanot substitution should fire: got '{}'",
            result
        );
        assert!(
            !result.contains("I'm here for it"),
            "warmth suffix should NOT stack after quanot substitution: got '{}'",
            result
        );
        assert!(
            !result.contains("I'm paying attention"),
            "warmth suffix should NOT stack after quanot substitution: got '{}'",
            result
        );
        assert!(
            !result.contains("I'm with you on this"),
            "warmth suffix should NOT stack after quanot substitution: got '{}'",
            result
        );
    }

    /// Phase 4 demotion: playful style appends a single "!" when energy is high,
    /// not a rotation through 3 phrases.
    #[test]
    fn playful_style_appends_single_punctuation() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();
        let config = VoiceConfig {
            style: ResponseStyle::Playful,
            energy: 0.9,
            confidence: 0.7,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.5,
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let text = "That's a good point";
        let result = engine.apply_personality_style(text, &config, false);
        assert_eq!(result, "That's a good point!", "playful + high energy should add a single '!'");
    }

    /// Phase 4 demotion: when energy is low, playful style is a pass-through.
    #[test]
    fn playful_style_passes_through_at_low_energy() {
        crate::variation::_clear_for_tests();
        let engine = make_voice_engine();
        let config = VoiceConfig {
            style: ResponseStyle::Playful,
            energy: 0.4, // low
            confidence: 0.7,
            novelty: 0.5,
            creativity: 0.5,
            consciousness: 0.5,
            has_memory_backing: false,
            is_uncertain: false,
            is_casual: false,
            internal_state: InternalState::default(),
        };
        let text = "That's a good point";
        let result = engine.apply_personality_style(text, &config, false);
        assert_eq!(result, text, "low energy should not trigger flourish");
    }

    /// Phase 1.2: Emotional intent strips "I think" / "I guess" hedges.
    #[test]
    fn intent_modulation_emotional_strips_hedges() {
        let engine = make_voice_engine();
        let state = InternalState::default()
            .with_intent(ResponseIntent::Emotional);
        let result = engine.apply_intent_modulation("I think I care about you.", &state);
        assert_eq!(
            result, "I care about you.",
            "Emotional intent should strip 'I think' hedge"
        );
    }

    /// Phase 1.2: Emotional intent with no hedge is a pass-through.
    #[test]
    fn intent_modulation_emotional_no_hedge_passthrough() {
        let engine = make_voice_engine();
        let state = InternalState::default()
            .with_intent(ResponseIntent::Emotional);
        let result = engine.apply_intent_modulation("I care about you.", &state);
        assert_eq!(
            result, "I care about you.",
            "Emotional intent with no hedge should pass through"
        );
    }

    /// Phase 1.2: Consciousness + high uncertainty softens with "I think".
    #[test]
    fn intent_modulation_consciousness_high_uncertainty_softens() {
        let engine = make_voice_engine();
        let state = InternalState::default()
            .with_intent(ResponseIntent::Consciousness)
            .with_uncertainty(0.8); // high
        // Use text that doesn't start with "I'm" so the present-tense guard
        // doesn't suppress the softening.
        let result = engine.apply_intent_modulation("Working on it still.", &state);
        assert!(
            result.starts_with("I think"),
            "Consciousness + high uncertainty should soften: got '{}'",
            result
        );
    }

    /// Phase 1.2: Consciousness + low uncertainty is a pass-through.
    #[test]
    fn intent_modulation_consciousness_low_uncertainty_passthrough() {
        let engine = make_voice_engine();
        let state = InternalState::default()
            .with_intent(ResponseIntent::Consciousness)
            .with_uncertainty(0.2); // low
        let result = engine.apply_intent_modulation("I'm working on it.", &state);
        assert_eq!(
            result, "I'm working on it.",
            "Consciousness + low uncertainty should not soften"
        );
    }

    /// Phase 1.2: SelfCheck intent is a pass-through (reranker handles it).
    #[test]
    fn intent_modulation_self_check_passthrough() {
        let engine = make_voice_engine();
        let state = InternalState::default()
            .with_intent(ResponseIntent::SelfCheck);
        let result = engine.apply_intent_modulation("I'm here, working.", &state);
        assert_eq!(result, "I'm here, working.");
    }

    /// Phase 1.2: Unknown intent is a pass-through.
    #[test]
    fn intent_modulation_unknown_passthrough() {
        let engine = make_voice_engine();
        let state = InternalState::default(); // no intent
        let result = engine.apply_intent_modulation("Hello there.", &state);
        assert_eq!(result, "Hello there.");
    }
}
