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
            } else if text.to_lowercase().contains("going to") {
                // "I'm going to [verb]" — future-oriented intent
                (ImIntention::Intent, 0.85)
            } else if text.to_lowercase().starts_with("i'm ") || text.to_lowercase().starts_with("im ") {
                if let Some(name) = extract_name(text) {
                    if name.chars().all(|c| c.is_lowercase() || c == '\'') {
                        return (ImIntention::Name, 0.9);
                    }
                }
                (ImIntention::State, 0.7)
            } else if text.to_lowercase().starts_with("i will ") || text.to_lowercase().starts_with("i'll ") {
                (ImIntention::Intent, 0.8)
            } else {
                (ImIntention::Other, 0.5)
            }
        } else {
            (ImIntention::Other, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        input: &'static str,
        expected: ImIntention,
    }

    fn run_cases(cases: &[TestCase]) {
        let classifier = ImIntentionClassifier;
        for tc in cases {
            let (result, _conf) = classifier.classify(tc.input);
            assert_eq!(
                result, tc.expected,
                "classify(\"{}\") = {:?}, expected {:?}",
                tc.input, result, tc.expected
            );
        }
    }

    #[test]
    fn test_im_utterance_detection() {
        assert!(is_im_utterance("I'm John"));
        assert!(is_im_utterance("im going"));
        assert!(is_im_utterance("I am tired"));
        assert!(!is_im_utterance("Hello there"));
        assert!(!is_im_utterance("What's up"));
    }

    #[test]
    fn test_name_extraction() {
        assert_eq!(extract_name("I'm John"), Some("John".to_string()));
        assert_eq!(extract_name("I'm Sarah O'Connor"), Some("Sarah".to_string()));
        assert_eq!(extract_name("i'm alex"), Some("alex".to_string()));
        assert_eq!(extract_name("I am tired"), Some("tired".to_string()));
        assert_eq!(extract_name("Hello"), None);
    }

    #[test]
    fn test_name_intention() {
        // Python model: capitalized name after "I'm" → Name
        let cases = &[
            TestCase { input: "I'm John",         expected: ImIntention::Name },
            TestCase { input: "I'm Sarah",        expected: ImIntention::Name },
            TestCase { input: "I'm O'Connor",     expected: ImIntention::Name },
            TestCase { input: "Im Zachary",       expected: ImIntention::Name },
            TestCase { input: "i am Max",          expected: ImIntention::Name },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_state_intention() {
        // Python model: lowercase adjective/verb after "I'm" → State
        let cases = &[
            TestCase { input: "I'm tired",        expected: ImIntention::State },
            TestCase { input: "I'm happy",         expected: ImIntention::State },
            TestCase { input: "I'm frustrated",    expected: ImIntention::State },
            TestCase { input: "I'm hungry",        expected: ImIntention::State },
            TestCase { input: "Im cold",           expected: ImIntention::State },
            TestCase { input: "i am busy",         expected: ImIntention::State },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_apology_intention() {
        // Python model: contains "sorry" or "apologize" → Apology
        let cases = &[
            TestCase { input: "I'm sorry",         expected: ImIntention::Apology },
            TestCase { input: "I'm so sorry",      expected: ImIntention::Apology },
            TestCase { input: "I apologize",       expected: ImIntention::Apology },
            TestCase { input: "I'm sorry about that", expected: ImIntention::Apology },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_intent_intention() {
        // Python model: "I'm going to [verb]" or future-oriented → Intent
        let cases = &[
            TestCase { input: "I'm going to the store",  expected: ImIntention::Intent },
            TestCase { input: "I'm going to try",        expected: ImIntention::Intent },
            TestCase { input: "I will go",               expected: ImIntention::Intent },
        ];
        run_cases(cases);
    }

    #[test]
    fn test_other_intention() {
        let cases = &[
            TestCase { input: "I'm just saying",    expected: ImIntention::Other },
            TestCase { input: "What's the plan",    expected: ImIntention::Other },
            TestCase { input: "Hello",              expected: ImIntention::Other },
            TestCase { input: "How are you",         expected: ImIntention::Other },
        ];
        run_cases(cases);
    }
}
