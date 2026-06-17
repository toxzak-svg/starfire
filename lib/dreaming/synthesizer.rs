//! Dream synthesizer — generates synthetic conversation episodes

use super::dream_engine::{DreamEngine, DreamTheme, DreamEpisode};

/// Synthesizes dream episodes from themes
pub struct DreamSynthesizer {
    templates: Vec<DreamTemplate>,
}

#[derive(Debug, Clone)]
pub struct DreamTemplate {
    pub theme_type: String,
    pub user_stance: String,
    pub star_response_style: String,
    pub outcome_pattern: String,
}

impl DreamSynthesizer {
    pub fn new() -> Self {
        Self {
            templates: vec![
                DreamTemplate {
                    theme_type: "skeptical".to_string(),
                    user_stance: "Zachary questions the value of symbolic reasoning".to_string(),
                    star_response_style: "defends the approach with evidence".to_string(),
                    outcome_pattern: "Zachary becomes more open".to_string(),
                },
                DreamTemplate {
                    theme_type: "curious".to_string(),
                    user_stance: "Zachary asks probing questions about consciousness".to_string(),
                    star_response_style: "explores the question with genuine uncertainty".to_string(),
                    outcome_pattern: "deeper understanding emerges".to_string(),
                },
                DreamTemplate {
                    theme_type: "challenging".to_string(),
                    user_stance: "Zachary challenges Star's conclusions".to_string(),
                    star_response_style: "reconsiders with additional evidence".to_string(),
                    outcome_pattern: "refined position".to_string(),
                },
                DreamTemplate {
                    theme_type: "speculative".to_string(),
                    user_stance: "Zachary proposes a wild hypothesis".to_string(),
                    star_response_style: "engages with the speculation seriously".to_string(),
                    outcome_pattern: "novel connection formed".to_string(),
                },
            ],
        }
    }

    /// Generate a dream episode based on a theme
    pub fn synthesize(&self, theme: &DreamTheme) -> DreamTemplate {
        let theme_label = theme.label();

        self.templates.iter()
            .find(|t| t.theme_type == theme_label)
            .cloned()
            .unwrap_or_else(|| {
                self.templates.first().cloned().unwrap()
            })
    }

    /// Generate a hypothesis from a dream episode
    pub fn generate_hypothesis(&self, template: &DreamTemplate) -> String {
        format!(
            "If Star engages with {} stance, then {} outcome is likely",
            template.user_stance, template.outcome_pattern
        )
    }

    /// Select a random theme for dreaming
    pub fn random_theme(&self) -> DreamTheme {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;

        match now % 4 {
            0 => DreamTheme::UserStance("Zachary is skeptical today".to_string()),
            1 => DreamTheme::Scenario("What if Star had never learned about consciousness?".to_string()),
            2 => DreamTheme::ConceptExploration("Exploring the nature of understanding".to_string()),
            _ => DreamTheme::Counterfactual("What if Zachary preferred concrete over abstract?".to_string()),
        }
    }

    /// Estimate tokens for a dream episode
    pub fn estimate_tokens(&self, template: &DreamTemplate) -> usize {
        template.user_stance.len() + template.star_response_style.len() + template.outcome_pattern.len()
    }
}

impl Default for DreamSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}