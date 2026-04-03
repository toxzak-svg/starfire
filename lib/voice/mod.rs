//! Voice Engine — Phrase Banking & Expression Generation
//!
//! Starfire's expressive voice system. Manages phrase accumulation,
//! expression templates, and voice-aware response generation.

pub mod phrases;
pub mod templates;

use phrases::PhraseBank;
use templates::{VoiceTemplate, TemplateEngine};
use crate::cognition::CognitiveState;
use std::sync::{Arc, Mutex};

/// The voice engine — shapes how Starfire expresses herself.
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
    /// Applies phrase variations, template selection, and voice shaping.
    pub fn speak(&self, raw: &str, cognition: &CognitiveState) -> String {
        // Step 1: Check if we have good phrases for any part of this response
        let phrased = self.apply_phrase_variations(raw, cognition);
        
        // Step 2: Apply characteristic voice patterns
        let voiced = self.apply_voice_patterns(&phrased, cognition);
        
        // Step 3: Apply emotional tinting
        let final_text = cognition.emotional_response(&voiced);
        
        final_text
    }

    /// Apply accumulated phrase variations where appropriate.
    fn apply_phrase_variations(&self, text: &str, cognition: &CognitiveState) -> String {
        let bank = match self.phrase_bank.lock() {
            Ok(b) => b,
            Err(_) => return text.to_string(),
        };
        
        // Get phrases relevant to this response
        let relevant = bank.get_relevant_phrases(text, 3);
        
        if relevant.is_empty() {
            return text.to_string();
        }
        
        // For now, apply subtle phrase blending for positive phrases
        // Only modify if we have strong matching phrases
        let mut result = text.to_string();
        for phrase in relevant {
            if phrase.positive_count > phrase.negative_count + 2 {
                // This phrase has proven to land well
                // Try to blend its construction into our response
                result = self.blend_phrase(&result, &phrase.phrase);
            }
        }
        
        result
    }

    /// Blend a proven phrase's construction into existing text.
    fn blend_phrase(&self, text: &str, phrase: &str) -> String {
        // If the text is short and the phrase is expressive, sometimes substitute
        // the opening for something more characteristic
        if text.len() < 50 && phrase.len() > 20 {
            // Check if text starts with something generic
            let lower = text.to_lowercase();
            if lower.starts_with("i ") || lower.starts_with("i'm ") || lower.starts_with("i think ") {
                // Blend in the phrase's construction style
                return format!("{} — {}", phrase, text);
            }
        }
        text.to_string()
    }

    /// Apply characteristic voice patterns based on cognitive state.
    fn apply_voice_patterns(&self, text: &str, cognition: &CognitiveState) -> String {
        let engagement = cognition.engagement_depth;
        let certainty = cognition.certainty;
        
        // High engagement + high certainty = more assertive, direct
        // High engagement + low certainty = more exploratory, questioning
        // Low engagement = shorter, more clipped
        
        let template_key = match (engagement, certainty) {
            (e, c) if e > 0.7 && c > 0.6 => "assertive",
            (e, c) if e > 0.7 && c <= 0.6 => "exploratory",
            (e, _) if e < 0.3 => "minimal",
            _ => "balanced",
        };
        
        self.template_engine.apply_template(text, template_key)
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
        let mut bank = self.phrase_bank.lock()?;
        bank.add_phrase(phrase, context, tags)?;
        Ok(())
    }

    /// Get Starfire's current voice statistics.
    pub fn stats(&self) -> anyhow::Result<phrases::VoiceStats> {
        let bank = self.phrase_bank.lock()?;
        Ok(bank.stats())
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
