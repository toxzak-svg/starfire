//! Dream engine — generates and manages synthetic episodes

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A synthetic dream episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEpisode {
    pub id: DreamId,
    /// The theme or scenario of this dream
    pub theme: String,
    /// What was hypothesized in this dream
    pub hypothesis: String,
    /// Whether this dream was validated against reality
    pub validated: bool,
    /// If validated, whether the hypothesis was supported
    pub support_level: f64,
    /// When this dream was generated
    pub generated_at: i64,
    /// If validated, when
    pub validated_at: Option<i64>,
    /// Tokens of this dream (for decay tracking)
    pub tokens: usize,
}

/// Unique identifier for a dream
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DreamId(u64);

impl DreamId {
    pub fn new() -> Self {
        DreamId(rand_id())
    }
}

impl Default for DreamId {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    now.wrapping_mul(0x517cc1b727220a95)
}

/// Theme for dream generation
#[derive(Debug, Clone)]
pub enum DreamTheme {
    /// Zachary holding a specific stance
    UserStance(String),
    /// A hypothetical scenario
    Scenario(String),
    /// Exploration of an abstract concept
    ConceptExploration(String),
    /// What if analysis
    Counterfactual(String),
}

impl DreamTheme {
    pub fn label(&self) -> &str {
        match self {
            DreamTheme::UserStance(_) => "user_stance",
            DreamTheme::Scenario(_) => "scenario",
            DreamTheme::ConceptExploration(_) => "concept",
            DreamTheme::Counterfactual(_) => "counterfactual",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            DreamTheme::UserStance(s) => s,
            DreamTheme::Scenario(s) => s,
            DreamTheme::ConceptExploration(s) => s,
            DreamTheme::Counterfactual(s) => s,
        }
    }
}

/// Dream engine state
#[derive(Debug, Clone)]
pub struct DreamEngine {
    episodes: VecDeque<DreamEpisode>,
    max_episodes: usize,
    current_theme: Option<DreamTheme>,
    validation_history: Vec<bool>,
}


impl DreamEngine {
    pub fn new() -> Self {
        Self {
            episodes: VecDeque::new(),
            max_episodes: 100,
            current_theme: None,
            validation_history: Vec::new(),
        }
    }

    /// Start a dream session with a theme
    pub fn start_dream(&mut self, theme: DreamTheme) {
        self.current_theme = Some(theme);
    }

    /// Record a dream episode
    pub fn record_episode(&mut self, hypothesis: &str, tokens: usize) {
        let theme_label = self.current_theme.as_ref()
            .map(|t| t.label())
            .unwrap_or("unknown");

        let episode = DreamEpisode {
            id: DreamId::new(),
            theme: theme_label.to_string(),
            hypothesis: hypothesis.to_string(),
            validated: false,
            support_level: 0.0,
            generated_at: crate::now_timestamp(),
            validated_at: None,
            tokens,
        };

        if self.episodes.len() >= self.max_episodes {
            self.episodes.pop_front();
        }
        self.episodes.push_back(episode);
    }

    /// End the current dream session
    pub fn end_dream(&mut self) {
        self.current_theme = None;
    }

    /// Get unvalidated dream episodes
    pub fn unvalidated_episodes(&self) -> Vec<&DreamEpisode> {
        self.episodes.iter()
            .filter(|e| !e.validated)
            .collect()
    }

    /// Mark a dream as validated
    pub fn mark_validated(&mut self, dream_id: DreamId, supported: bool, support_level: f64) {
        if let Some(episode) = self.episodes.iter_mut().find(|e| e.id == dream_id) {
            episode.validated = true;
            episode.support_level = support_level;
            episode.validated_at = Some(crate::now_timestamp());

            self.validation_history.push(supported);
        }
    }

    /// Compute predictive value of dreams
    /// How often do dream-driven hypotheses get supported?
    pub fn dream_predictive_value(&self) -> f64 {
        if self.validation_history.is_empty() {
            return 0.5;
        }

        let supported = self.validation_history.iter()
            .filter(|&&supported| supported)
            .count();

        supported as f64 / self.validation_history.len() as f64
    }

    /// Should we allocate more or less time to dreaming?
    pub fn dreaming_recommendation(&self) -> &'static str {
        let pv = self.dream_predictive_value();

        if pv > 0.6 {
            "increase"
        } else if pv < 0.3 {
            "decrease"
        } else {
            "maintain"
        }
    }

    /// Get recent dream episodes
    pub fn recent_episodes(&self, n: usize) -> Vec<&DreamEpisode> {
        self.episodes.iter().rev().take(n).collect()
    }

    /// Get hypothesis success rate by theme
    pub fn theme_success_rate(&self, theme: &str) -> f64 {
        let theme_validations: Vec<_> = self.episodes.iter()
            .filter(|e| e.theme == theme && e.validated)
            .collect();

        if theme_validations.is_empty() {
            return 0.5;
        }

        let supported = theme_validations.iter()
            .filter(|e| e.support_level > 0.5)
            .count();

        supported as f64 / theme_validations.len() as f64
    }
}

impl Default for DreamEngine {
    fn default() -> Self {
        Self::new()
    }
}