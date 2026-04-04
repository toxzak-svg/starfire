//! Learning Engine — instant teaching, experience logging, and understanding tracking

use std::collections::HashMap;

/// An instant teaching — user directly tells Star something.
#[derive(Clone)]
pub struct InstantTeach {
    pub term: String,
    pub definition: String,
    pub confidence: f64,
}

/// A learning experience — observed from conversation.
#[derive(Clone)]
pub struct Experience {
    pub input: String,
    pub output: Option<String>,
    pub confidence: f64,
    pub timestamp: i64,
}

/// Understanding of a concept.
#[derive(Clone)]
pub struct Understanding {
    pub term: String,
    pub definition: Option<String>,
    pub confidence: f64,
    pub taught_instantly: bool,
}

/// Learning engine — tracks instant teachings, experiences, and understanding.
pub struct LearningEngine {
    instant_teachings: Vec<InstantTeach>,
    experiences: Vec<Experience>,
    understandings: HashMap<String, Understanding>,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            instant_teachings: Vec::new(),
            experiences: Vec::new(),
            understandings: HashMap::new(),
        }
    }

    /// Record an instant teaching (user directly provides a fact).
    pub fn teach_instant(&mut self, term: &str, definition: &str, confidence: f64) {
        let teaching = InstantTeach {
            term: term.to_string(),
            definition: definition.to_string(),
            confidence,
        };
        self.instant_teachings.push(teaching.clone());
        self.understandings.insert(term.to_string(), Understanding {
            term: term.to_string(),
            definition: Some(definition.to_string()),
            confidence,
            taught_instantly: true,
        });
    }

    /// Record a learning experience from conversation.
    pub fn experience(&mut self, _term: &str, input: &str, output: Option<&str>, confidence: f64) {
        self.experiences.push(Experience {
            input: input.to_string(),
            output: output.map(|s| s.to_string()),
            confidence,
            timestamp: crate::now_timestamp(),
        });
    }

    /// Get Star's current understanding of a term.
    pub fn get_understanding(&self, term: &str) -> Option<String> {
        self.understandings.get(term).map(|u| {
            u.definition.clone().unwrap_or_else(|| u.term.clone())
        })
    }

    /// Get a summary of all learning.
    pub fn summary(&self) -> String {
        let taught = self.instant_teachings.len();
        let exp = self.experiences.len();
        let understood = self.understandings.len();
        format!(
            "Learning: {} instant teachings, {} experiences, {} understood concepts",
            taught, exp, understood
        )
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}
