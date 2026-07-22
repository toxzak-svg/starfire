//! Shared types for user-cognition model


/// User-Cognition Model — tracks what Star knows about Zachary's cognition
#[derive(Debug, Clone)]
pub struct UserCognitionModel {
    /// Zachary's memory model (what he remembers, forgets, etc.)
    pub memory: super::memory_model::UserMemoryModel,
    /// Zachary's reasoning style preferences
    pub preferences: Vec<super::preference::UserPreference>,
    /// Domains where Zachary has strong knowledge
    pub strong_domains: Vec<String>,
    /// Domains where Zachary is unfamiliar or uncertain
    pub weak_domains: Vec<String>,
    /// How Zachary typically responds to different question types
    pub response_patterns: Vec<ResponsePattern>,
    /// Last time each aspect was updated
    pub last_updated: i64,
}

/// A pattern of how Zachary responds to certain stimulus types
#[derive(Debug, Clone)]
pub struct ResponsePattern {
    pub pattern_type: String,
    pub description: String,
    pub example_count: usize,
    pub reliability: f64,
}

/// User reasoning stance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningStance {
    /// Prefers safe, proven approaches
    Conservative,
    /// Willing to try speculative approaches
    Balanced,
    /// Embraces risk and novel approaches
    Speculative,
}

/// Argument style preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentStyle {
    /// Prefers concrete examples and specific cases
    Concrete,
    /// Prefers abstractions and theoretical frameworks
    Abstract,
    /// Uses both as needed
    Adaptive,
}

impl UserCognitionModel {
    pub fn new() -> Self {
        Self {
            memory: super::memory_model::UserMemoryModel::new(),
            preferences: Vec::new(),
            strong_domains: Vec::new(),
            weak_domains: Vec::new(),
            response_patterns: Vec::new(),
            last_updated: crate::now_timestamp(),
        }
    }

    /// Infer Zachary's reasoning stance from conversation history
    pub fn infer_reasoning_stance(&self) -> ReasoningStance {
        let speculative_count = self.response_patterns.iter()
            .filter(|p| p.pattern_type == "speculative")
            .count();
        let conservative_count = self.response_patterns.iter()
            .filter(|p| p.pattern_type == "conservative")
            .count();
        let total = speculative_count + conservative_count;

        if total == 0 {
            return ReasoningStance::Balanced;
        }

        let ratio = speculative_count as f64 / total as f64;
        if ratio > 0.6 {
            ReasoningStance::Speculative
        } else if ratio < 0.4 {
            ReasoningStance::Conservative
        } else {
            ReasoningStance::Balanced
        }
    }

    /// Infer Zachary's preferred argument style
    pub fn infer_argument_style(&self) -> ArgumentStyle {
        use super::preference::PreferenceType;
        let concrete_count = self.preferences.iter()
            .filter(|p| matches!(p.preference, PreferenceType::AgreesWithStyle(ArgumentStyle::Concrete)))
            .count();
        let abstract_count = self.preferences.iter()
            .filter(|p| matches!(p.preference, PreferenceType::AgreesWithStyle(ArgumentStyle::Abstract)))
            .count();
        let total = concrete_count + abstract_count;

        if total == 0 {
            return ArgumentStyle::Adaptive;
        }

        if concrete_count > abstract_count {
            ArgumentStyle::Concrete
        } else if abstract_count > concrete_count {
            ArgumentStyle::Abstract
        } else {
            ArgumentStyle::Adaptive
        }
    }

    /// Check if Zachary likely knows a topic
    pub fn likely_knows(&self, topic: &str) -> Option<bool> {
        let topic_lower = topic.to_lowercase();

        for domain in &self.strong_domains {
            if topic_lower.contains(&domain.to_lowercase()) {
                return Some(true);
            }
        }

        for domain in &self.weak_domains {
            if topic_lower.contains(&domain.to_lowercase()) {
                return Some(false);
            }
        }

        None
    }

    /// Should we ask Zachary a question instead of reasoning ourselves?
    /// Returns true if Zachary is likely to have better priors or preferences.
    pub fn should_delegate_to_user(&self, topic: &str) -> bool {
        if let Some(knows) = self.likely_knows(topic) {
            return !knows;
        }

        if let Some(pref) = self.preferences.iter().find(|p| {
            p.topic.to_lowercase() == topic.to_lowercase()
        }) {
            return pref.prefers_questions();
        }

        false
    }

    /// Format a response adapted to Zachary's preferences
    pub fn adapt_response(&self, content: &str, _topic: &str) -> String {
        let style = self.infer_argument_style();

        match style {
            ArgumentStyle::Concrete => {
                if !content.contains("example") && !content.contains("for instance") {
                    format!("{}. Here's a concrete case to consider.", content)
                } else {
                    content.to_string()
                }
            }
            ArgumentStyle::Abstract => {
                if !content.contains("concept") && !content.contains("principle") {
                    format!("{}. The underlying principle is this.", content)
                } else {
                    content.to_string()
                }
            }
            ArgumentStyle::Adaptive => content.to_string(),
        }
    }

    /// Record a teaching from Zachary (updates what he knows)
    pub fn record_teaching(&mut self, term: &str, definition: &str) {
        let domain = extract_domain(term, definition);
        if !domain.is_empty() && !self.strong_domains.contains(&domain) {
            self.strong_domains.push(domain);
        }
        self.last_updated = crate::now_timestamp();
    }

    /// Record a question from Zachary (updates response patterns)
    pub fn record_question(&mut self, question_type: &str, topic: &str) {
        if let Some(pattern) = self.response_patterns.iter_mut()
            .find(|p| p.pattern_type == question_type)
        {
            pattern.example_count += 1;
            pattern.reliability = (pattern.reliability * 0.9 + 0.1).min(0.95);
        } else {
            self.response_patterns.push(ResponsePattern {
                pattern_type: question_type.to_string(),
                description: format!("Questions about {}", topic),
                example_count: 1,
                reliability: 0.5,
            });
        }
        self.last_updated = crate::now_timestamp();
    }

    /// Add a preference observation
    pub fn add_preference(&mut self, pref: super::preference::UserPreference) {
        if !self.preferences.iter().any(|p| p.topic == pref.topic) {
            self.preferences.push(pref);
        }
        self.last_updated = crate::now_timestamp();
    }

    /// Summary of what Star knows about Zachary
    pub fn summary(&self) -> String {
        let stance = self.infer_reasoning_stance();
        let style = self.infer_argument_style();
        let stance_str = match stance {
            ReasoningStance::Conservative => "conservative",
            ReasoningStance::Balanced => "balanced",
            ReasoningStance::Speculative => "speculative",
        };
        let style_str = match style {
            ArgumentStyle::Concrete => "concrete examples",
            ArgumentStyle::Abstract => "abstract reasoning",
            ArgumentStyle::Adaptive => "adaptive",
        };

        format!(
            "User Model: {} reasoning stance, prefers {}. Strong in {} domains, learning {}.",
            stance_str,
            style_str,
            self.strong_domains.len(),
            self.weak_domains.len()
        )
    }
}

impl Default for UserCognitionModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract a domain hint from a term/definition pair
fn extract_domain(term: &str, definition: &str) -> String {
    let combined = format!("{} {}", term, definition).to_lowercase();

    let domain_hints = [
        ("rust", "programming", "code"),
        ("python", "programming", "code"),
        ("math", "mathematics", "math"),
        ("physics", "physics", "science"),
        ("bio", "biology", "science"),
        ("chem", "chemistry", "science"),
        ("history", "history", "humanities"),
        ("philo", "philosophy", "humanities"),
        ("psych", "psychology", "humanities"),
        ("ai", "artificial intelligence", "tech"),
        ("ml", "machine learning", "tech"),
        ("design", "design", "creative"),
    ];

    for (hint, domain, _) in domain_hints {
        if combined.contains(hint) {
            return domain.to_string();
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model() {
        let model = UserCognitionModel::new();
        assert_eq!(model.strong_domains.len(), 0);
    }

    #[test]
    fn test_infer_stance() {
        let model = UserCognitionModel::new();
        let stance = model.infer_reasoning_stance();
        assert_eq!(stance, ReasoningStance::Balanced);
    }

    #[test]
    fn test_likely_knows() {
        let mut model = UserCognitionModel::new();
        model.strong_domains.push("rust".to_string());

        assert_eq!(model.likely_knows("rust programming"), Some(true));
        assert_eq!(model.likely_knows("cooking"), None);
    }
}