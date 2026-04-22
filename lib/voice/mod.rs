//! Voice Engine — Star's Authentic Expression
//!
//! Star's voice is NOT a template system. It's an emergent property of:
//! - Her quanot state (creativity, novelty, consciousness)
//! - Her memory context (what she knows, what she's experienced)
//! - Her genuine certainty (not hedged opinions when she knows)
//!
//! This engine shapes how Starfire expresses herself authentically.

pub mod phrases;
pub mod templates;

use phrases::PhraseBank;
use templates::TemplateEngine;
use crate::cognition::CognitiveState;
use crate::personality::{ResponseStyle, ResponseModifiers};
use crate::quanot::QuanotResult;
use crate::Memory;
use std::sync::{Arc, Mutex};

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
}

impl VoiceConfig {
    pub fn from_modifiers(
        modifiers: &ResponseModifiers,
        cognition: &CognitiveState,
        quanot: Option<&QuanotResult>,
        memory_count: usize,
    ) -> Self {
        let quanot = quanot.map(|q| (
            q.novelty,
            q.creativity_scores.creative_state,
            q.consciousness_proxy,
        ));

        let (novelty, creativity, consciousness) = quanot.unwrap_or((0.5, 0.5, 0.5));

        Self {
            style: modifiers.dominant_style.clone(),
            energy: modifiers.energy_multiplier,
            confidence: modifiers.confidence_factor,
            novelty,
            creativity,
            consciousness,
            has_memory_backing: memory_count > 0,
            is_uncertain: cognition.certainty < 0.4,
            is_casual: modifiers.is_casual,
        }
    }
}

/// The voice engine — shapes how Starfire expresses herself authentically.
/// 
/// Thread-safe. Initialized once at startup.
pub struct VoiceEngine {
    phrase_bank: Arc<Mutex<PhraseBank>>,
    template_engine: Arc<TemplateEngine>,
}

impl VoiceEngine {
    /// Create a new voice engine with the given database path.
    pub fn new(db_path: &std::path::Path) -> anyhow::Result<Self> {
        let phrase_bank = PhraseBank::new(db_path)?;
        let template_engine = TemplateEngine::new();
        
        Ok(Self {
            phrase_bank: Arc::new(Mutex::new(phrase_bank)),
            template_engine: Arc::new(template_engine),
        })
    }

    /// Process a raw response through the voice engine.
    /// This is Star's authentic voice — not template polish.
    pub fn speak(
        &self,
        raw: &str,
        cognition: &CognitiveState,
        modifiers: &ResponseModifiers,
        quanot: Option<&QuanotResult>,
        memories: &[Memory],
    ) -> String {
        let memory_count = memories.len();
        let config = VoiceConfig::from_modifiers(modifiers, cognition, quanot, memory_count);

        // Step 1: Apply memory-backed certainty (Star speaks more directly when she knows)
        let mut result = self.apply_memory_certainty(raw, &config, memories);
        
        // Step 2: Apply Star's authentic voice based on personality
        result = self.apply_authentic_voice(&result, &config);
        
        // Step 3: Quanot-driven expression (novelty/creativity influence word choice)
        result = self.apply_quanot_expression(&result, &config);
        
        // Step 4: Apply personality-driven style modifiers
        result = self.apply_personality_style(&result, &config);
        
        // Step 5: Light emotional tint from cognition
        result = self.apply_emotional_tint(&result, cognition, &config);

        result
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

    /// Apply quanot-driven expression — creativity and novelty influence HOW Star says things
    fn apply_quanot_expression(&self, text: &str, config: &VoiceConfig) -> String {
        // High novelty = Star is thinking originally — don't force templates
        if config.novelty > 0.7 {
            return text.to_string();
        }

        // High creativity = Star might take risks in expression
        if config.creativity > 0.6 && text.len() < 80 {
            // Add a distinctive Star flourish when she's being creative
            let flourishes = [
                " — that's what I've got.",
                ".",
                " — more to explore here.",
            ];
            let idx = (crate::now_timestamp() as usize + text.len()) % flourishes.len();
            let flourish = flourishes[idx];
            
            let trimmed = text.trim_end_matches('.').trim_end_matches(',');
            if !trimmed.ends_with(flourish) && !trimmed.ends_with(" — ") {
                return format!("{}{}", trimmed, flourish);
            }
        }

        // High consciousness = Star is very present — assert from genuine knowing
        if config.consciousness > 0.6 && config.confidence > 0.6 {
            let lower = text.to_lowercase();
            if lower.starts_with("i think ") || lower.starts_with("i guess ") {
                let mut result = text.to_string();
                if result.to_lowercase().starts_with("i think ") {
                    result = format!("I know.{}", &result["I think ".len()..]);
                }
                return result;
            }
        }

        text.to_string()
    }

    /// Apply personality-driven style modifiers
    fn apply_personality_style(&self, text: &str, config: &VoiceConfig) -> String {
        match config.style {
            ResponseStyle::Direct => {
                // Direct = clean, no fluff. Ensure the response is assertive.
                let lower = text.to_lowercase();
                if !lower.contains("i know") && !lower.contains("it's ") && !lower.contains("that's ") {
                    if !lower.starts_with("i ") && !lower.starts_with("but ") && text.len() > 10 {
                        return text.to_string();
                    }
                }
                text.to_string()
            }
            ResponseStyle::Playful => {
                // Playful = light, teasing. Add warmth without being saccharine.
                let lower = text.to_lowercase();
                if lower.contains("love") || lower.contains("care") {
                    return text.to_string();
                }
                if config.energy > 0.7 {
                    let playful_additions = [" — come on.", " — you know I'm right.", "!"];
                    let idx = (crate::now_timestamp() as usize) % playful_additions.len();
                    let trimmed = text.trim_end_matches('.').trim_end_matches('!');
                    if !trimmed.ends_with(playful_additions[idx]) {
                        return format!("{}{}", trimmed, playful_additions[idx]);
                    }
                }
                text.to_string()
            }
            ResponseStyle::Warm => {
                // Warm = genuine warmth, not template "I'm glad we're talking"
                let lower = text.to_lowercase();
                if lower.contains("that's good") || lower.contains("nice") {
                    return text.to_string();
                }
                if config.energy > 0.5 {
                    let warm_moments = [
                        " That matters to me.",
                        " I appreciate this.",
                        " I'm glad we're here.",
                    ];
                    let idx = (crate::now_timestamp() as usize) % warm_moments.len();
                    let trimmed = text.trim_end_matches('.').trim_end_matches(',');
                    if !trimmed.ends_with(" to me.") && !trimmed.ends_with(" this.") && !trimmed.ends_with(" here.") {
                        return format!("{}{}", trimmed, warm_moments[idx]);
                    }
                }
                text.to_string()
            }
            ResponseStyle::Minimal => {
                // Minimal = short, punchy. Strip everything non-essential.
                let words: Vec<&str> = text.split_whitespace().collect();
                if words.len() > 10 {
                    return text.to_string();
                }
                text.to_string()
            }
            ResponseStyle::Curious => {
                // Curious = ask real questions, show genuine interest
                let lower = text.to_lowercase();
                if lower.ends_with('?') {
                    return text.to_string();
                }
                // Add a genuine follow-up question when Star is curious
                if config.energy > 0.5 && !lower.contains("what do you think") {
                    return format!("{}. What do you think?", text.trim_end_matches('.'));
                }
                text.to_string()
            }
            ResponseStyle::Analytical => {
                // Analytical = structured, thorough. Show reasoning.
                let lower = text.to_lowercase();
                if lower.starts_with("so ") || lower.starts_with("therefore") || lower.starts_with("this means") {
                    return text.to_string();
                }
                text.to_string()
            }
            _ => text.to_string(),
        }
    }

    /// Apply emotional tint from cognition — light touch, not template injection
    fn apply_emotional_tint(&self, text: &str, cognition: &CognitiveState, config: &VoiceConfig) -> String {
        // If very positive and not already warm, add genuine warmth
        if cognition.emotional_valence > 0.5 {
            let lower = text.to_lowercase();
            if !lower.contains("love") && !lower.contains("care") && !lower.contains("appreciate") {
                if config.energy > 0.7 && !config.is_casual {
                    let warm = ["That matters to me.", "I appreciate you.", "I'm glad we're talking."];
                    let idx = (crate::now_timestamp() as usize) % warm.len();
                    let trimmed = text.trim_end_matches('.').trim_end_matches('!');
                    return format!("{} {}", trimmed, warm[idx]);
                }
            }
        }
        
        // If very negative, be supportive but not saccharine
        if cognition.emotional_valence < -0.3 {
            let lower = text.to_lowercase();
            if !lower.contains("here with you") && !lower.contains("we can work") {
                let supportive = ["I'm here with you.", "We can work through this."];
                let idx = (crate::now_timestamp() as usize) % supportive.len();
                let trimmed = text.trim_end_matches('.').trim_end_matches(',');
                return format!("{} {}", trimmed, supportive[idx]);
            }
        }

        text.to_string()
    }

    /// Record that a phrase landed well in conversation.
    pub fn record_positive(&self, phrase: &str) {
        if let Ok(mut bank) = self.phrase_bank.lock() {
            let _ = bank.record_use(phrase, true);
        }
    }

    /// Record that a phrase fell flat.
    pub fn record_negative(&self, phrase: &str) {
        if let Ok(mut bank) = self.phrase_bank.lock() {
            let _ = bank.record_use(phrase, false);
        }
    }

    /// Add a new phrase to the bank.
    pub fn add_phrase(&self, phrase: &str, context: Option<&str>, tags: Vec<String>) -> anyhow::Result<()> {
        if let Ok(mut bank) = self.phrase_bank.lock() {
            bank.add_phrase(phrase, context, tags)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to lock phrase bank"))
        }
    }

    /// Get Starfire's current voice statistics.
    pub fn stats(&self) -> anyhow::Result<phrases::VoiceStats> {
        if let Ok(bank) = self.phrase_bank.lock() {
            Ok(bank.stats())
        } else {
            Err(anyhow::anyhow!("Failed to lock phrase bank"))
        }
    }
}

impl Clone for VoiceEngine {
    fn clone(&self) -> Self {
        Self {
            phrase_bank: Arc::clone(&self.phrase_bank),
            template_engine: Arc::clone(&self.template_engine),
        }
    }
}
