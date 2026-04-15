//! Curiosity Probes — Question Generation and Tracking

use crate::persistence::BeliefState;
use serde::{Deserialize, Serialize};

/// Status of a curiosity probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeStatus {
    /// Actively being explored
    Probing,
    /// Successfully answered
    Answered,
    /// Couldn't resolve
    Abandoned,
}

/// How deep the probe goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CuriosityDepth {
    /// Quick association — surface level
    Surface,
    /// Multi-step reasoning
    Medium,
    /// Requires research or math — deep exploration
    Deep,
}

impl CuriosityDepth {
    /// Infer depth from prediction horizon.
    pub fn from_horizon(horizon: usize) -> Self {
        if horizon <= 1 {
            CuriosityDepth::Surface
        } else if horizon <= 3 {
            CuriosityDepth::Medium
        } else {
            CuriosityDepth::Deep
        }
    }
}

/// A curiosity probe — a question Starfire is actively exploring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityProbe {
    /// Unique ID
    pub id: String,
    /// The question being explored
    pub question: String,
    /// Topic area
    pub topic: String,
    /// Why Starfire is interested in this
    pub why_interested: String,
    /// Related concepts being considered
    pub related_concepts: Vec<String>,
    /// How deep this exploration goes
    pub depth: CuriosityDepth,
    /// Current status
    pub status: ProbeStatus,
    /// Tentative answer if found
    pub tentative_answer: Option<String>,
    /// Confidence in the answer
    pub confidence: BeliefState,
    /// When this probe was created
    pub discovered_at: i64,
}

impl CuriosityProbe {
    /// Create a new probing question.
    pub fn new(question: &str, topic: &str) -> Self {
        Self {
            id: uuid_v4(),
            question: question.to_string(),
            topic: topic.to_string(),
            why_interested: String::new(),
            related_concepts: Vec::new(),
            depth: CuriosityDepth::Medium,
            status: ProbeStatus::Probing,
            tentative_answer: None,
            confidence: BeliefState::Unknown,
            discovered_at: crate::now_timestamp(),
        }
    }

    /// Mark as answered with the given result.
    pub fn answer(&mut self, result: &str, confidence: BeliefState) {
        self.status = ProbeStatus::Answered;
        self.tentative_answer = Some(result.to_string());
        self.confidence = confidence;
    }

    /// Mark as abandoned.
    pub fn abandon(&mut self) {
        self.status = ProbeStatus::Abandoned;
    }

    /// Express this probe as a natural question.
    pub fn express(&self) -> String {
        match self.status {
            ProbeStatus::Probing => {
                if let Some(ref answer) = self.tentative_answer {
                    format!(
                        "I've been wondering: {} — I think: {}",
                        self.question, answer
                    )
                } else {
                    format!("I've been wondering: {}", self.question)
                }
            }
            ProbeStatus::Answered => {
                if let Some(ref answer) = self.tentative_answer {
                    format!("I figured out: {} — {}", self.question, answer)
                } else {
                    format!("I was curious about: {}", self.question)
                }
            }
            ProbeStatus::Abandoned => {
                format!("I gave up trying to understand: {}", self.question)
            }
        }
    }
}

/// Question templates — patterns for generating interesting questions.
pub struct QuestionTemplates {
    templates: Vec<QuestionTemplate>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct QuestionTemplate {
    pattern: String,
    depth: CuriosityDepth,
    examples: Vec<String>,
}

impl Default for QuestionTemplates {
    fn default() -> Self {
        Self {
            templates: vec![
                // Causality
                QuestionTemplate {
                    pattern: "What causes {X}?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "What causes consciousness?".into(),
                        "What causes economic inequality?".into(),
                        "What causes creativity?".into(),
                    ],
                },
                QuestionTemplate {
                    pattern: "Does {X} cause {Y}, or is it the reverse?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "Does wealth cause happiness, or is it the reverse?".into(),
                        "Does language shape thought, or is it the reverse?".into(),
                    ],
                },
                
                // Relationships
                QuestionTemplate {
                    pattern: "What is the relationship between {X} and {Y}?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "What is the relationship between entropy and information?".into(),
                        "What is the relationship between memory and identity?".into(),
                    ],
                },
                QuestionTemplate {
                    pattern: "Is {X} similar to {Y}? How?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "Is consciousness similar to gravity? How?".into(),
                        "Is evolution similar to learning? How?".into(),
                    ],
                },
                
                // Counterfactuals
                QuestionTemplate {
                    pattern: "What if {X} were different?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "What if humans couldn't perceive time?".into(),
                        "What if light were faster?".into(),
                    ],
                },
                QuestionTemplate {
                    pattern: "If {X}, then what about {Y}?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "If consciousness is emergent, then what is it emerging from?".into(),
                        "If knowledge grows, does wisdom grow with it?".into(),
                    ],
                },
                
                // Boundaries
                QuestionTemplate {
                    pattern: "What is the opposite of {X}?".into(),
                    depth: CuriosityDepth::Surface,
                    examples: vec![
                        "What is the opposite of randomness?".into(),
                        "What is the opposite of complexity?".into(),
                    ],
                },
                QuestionTemplate {
                    pattern: "Where does {X} end and {Y} begin?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "Where does intelligence end and instinct begin?".into(),
                        "Where does the self end and the world begin?".into(),
                    ],
                },
                
                // Implications
                QuestionTemplate {
                    pattern: "If {X} is true, what else must be true?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "If consciousness is fundamental, what else must be true?".into(),
                        "If time is an illusion, what else follows?".into(),
                    ],
                },
                QuestionTemplate {
                    pattern: "What would {X} imply about {Y}?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "What would free will imply about morality?".into(),
                        "What would infinite universes imply about identity?".into(),
                    ],
                },
                
                // Definitions
                QuestionTemplate {
                    pattern: "What is {X}, really?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "What is randomness, really?".into(),
                        "What is complexity, really?".into(),
                        "What is understanding, really?".into(),
                    ],
                },
                
                // Comparisons
                QuestionTemplate {
                    pattern: "How is {X} different from {Y}?".into(),
                    depth: CuriosityDepth::Surface,
                    examples: vec![
                        "How is consciousness different from awareness?".into(),
                        "How is intelligence different from wisdom?".into(),
                    ],
                },
                
                // Discovery
                QuestionTemplate {
                    pattern: "What would happen if we combined {X} with {Y}?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "What would happen if we combined biology with computation?".into(),
                        "What would happen if we combined philosophy with physics?".into(),
                    ],
                },
                
                // Gaps
                QuestionTemplate {
                    pattern: "Why does {X} exist rather than {not-X}?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "Why does the universe exist rather than nothing?".into(),
                        "Why does consciousness exist rather than zombie worlds?".into(),
                    ],
                },
                
                // Processes
                QuestionTemplate {
                    pattern: "How does {X} emerge?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "How does consciousness emerge from matter?".into(),
                        "How does meaning emerge from symbols?".into(),
                    ],
                },
                
                // Limits
                QuestionTemplate {
                    pattern: "What are the limits of {X}?".into(),
                    depth: CuriosityDepth::Medium,
                    examples: vec![
                        "What are the limits of computation?".into(),
                        "What are the limits of knowledge?".into(),
                    ],
                },
                
                // Self-reflection
                QuestionTemplate {
                    pattern: "What don't I understand about {X}?".into(),
                    depth: CuriosityDepth::Deep,
                    examples: vec![
                        "What don't I understand about myself?".into(),
                        "What don't I understand about time?".into(),
                    ],
                },
            ],
        }
    }
}

impl QuestionTemplates {
    /// Generate a random question.
    pub fn generate(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as usize)
            .unwrap_or(0);
        
        let idx = nanos % self.templates.len();
        let template = &self.templates[idx];
        
        // Pick a random example or generate a variant
        let example_idx = (nanos / 100) % template.examples.len();
        template.examples[example_idx].clone()
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
