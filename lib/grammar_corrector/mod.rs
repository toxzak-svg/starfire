//! IntentionCNN — classifies "I'm X" utterances into intent categories.
//!
//! Handles: name introduction, state reports, apologies, intents, and chitchat.
//! Loaded behind the `llm` feature gate (candle-core + candle-nn inference).

/// Intention categories from the CNN classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImIntention {
    Name     = 0,
    State    = 1,
    Apology  = 2,
    Intent   = 3,
    Chitchat = 4,
    Other    = 5,
}

impl ImIntention {
    pub fn from_idx(idx: usize) -> Self {
        match idx {
            0 => ImIntention::Name,
            1 => ImIntention::State,
            2 => ImIntention::Apology,
            3 => ImIntention::Intent,
            4 => ImIntention::Chitchat,
            _ => ImIntention::Other,
        }
    }
}

/// Returns true if text matches the "I'm X" pattern.
pub fn is_im_utterance(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.starts_with("i'm ") || lower.starts_with("im ") || lower.starts_with("i am ")
}

/// Extract the name after "I'm" or "I am".
pub fn extract_name(text: &str) -> Option<String> {
    let text = text.trim();
    let prefix_and_rest = if text.to_lowercase().starts_with("i'm ") {
        text.strip_prefix("I'm ").or(text.strip_prefix("i'm "))
    } else if text.to_lowercase().starts_with("im ") {
        text.strip_prefix("Im ").or(text.strip_prefix("im "))
    } else if text.to_lowercase().starts_with("i am ") {
        text.strip_prefix("I am ").or(text.strip_prefix("i am "))
    } else {
        None
    };

    prefix_and_rest.and_then(|rest| {
        let name = rest.split_whitespace().next().unwrap_or(rest)
            .trim_matches(|c: char| c.is_ascii_punctuation() && c != '\'');
        if name.is_empty() || name.len() > 30 { None } else { Some(name.to_string()) }
    })
}

/// Stub classifier — model loading needs full candle API debugging.
pub struct ImIntentionClassifier;

impl ImIntentionClassifier {
    pub fn new() -> Result<Self, candle_core::Error> {
        Ok(Self)
    }

    pub fn load(&self, _path: &std::path::Path) -> Result<(), candle_core::Error> {
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        false
    }

    pub fn classify(&self, text: &str) -> (ImIntention, f32) {
        // Simple rule-based fallback
        if is_im_utterance(text) {
            if text.to_lowercase().contains("sorry") || text.to_lowercase().contains("apologize") {
                (ImIntention::Apology, 0.8)
            } else if text.to_lowercase().starts_with("i'm ") || text.to_lowercase().starts_with("im ") {
                if let Some(name) = extract_name(text) {
                    if name.chars().all(|c| c.is_lowercase() || c == '\'') {
                        return (ImIntention::Name, 0.9);
                    }
                }
                (ImIntention::State, 0.7)
            } else {
                (ImIntention::Other, 0.5)
            }
        } else {
            (ImIntention::Other, 0.0)
        }
    }
}
