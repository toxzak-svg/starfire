//! Regime Classifier — identifies which metastable reasoning mode the system is in
//!
//! NOTE: "Attractors" here are METASTABLE REASONING MODES — not asymptotic fixed points.
//! These are regions of state space where trajectories "dwell" disproportionately long,
//! with well-defined escape statistics. This matches the cortical computation literature on
//! metastable attractors (Rabinovich et al., 2008; Stevens, 2013).
//!
//! Reasoning modes (metastable attractors):
//! - SymbolicManipulation: precise logical/algebraic reasoning
//! - EmotionalResonance: empathy, social inference, emotional state tracking
//! - CausalReasoning: cause-effect chains, intervention prediction
//! - AssociativeRecall: memory retrieval, pattern matching
//! - Exploratory: open-ended curiosity, hypothesis generation
//! - SteadyState: idle, waiting, no active reasoning mode

use serde::{Deserialize, Serialize};

/// The six reasoning mode attractors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReasoningRegime {
    SymbolicManipulation = 0,
    EmotionalResonance   = 1,
    CausalReasoning      = 2,
    AssociativeRecall    = 3,
    Exploratory          = 4,
    SteadyState         = 5,
}

impl ReasoningRegime {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SymbolicManipulation => "symbolic_manipulation",
            Self::EmotionalResonance   => "emotional_resonance",
            Self::CausalReasoning      => "causal_reasoning",
            Self::AssociativeRecall    => "associative_recall",
            Self::Exploratory          => "exploratory",
            Self::SteadyState          => "steady_state",
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Self::SymbolicManipulation,
            1 => Self::EmotionalResonance,
            2 => Self::CausalReasoning,
            3 => Self::AssociativeRecall,
            4 => Self::Exploratory,
            _ => Self::SteadyState,
        }
    }

    pub fn num_regimes() -> usize { 6 }
}

/// Input features for regime classification
#[derive(Debug, Clone, Default)]
pub struct RegimeFeatures {
    /// Ratio of symbolic/mathematical tokens
    pub symbolic_ratio: f32,
    /// Ratio of emotional/social words
    pub emotional_ratio: f32,
    /// Ratio of causal connectives (because, therefore, hence, causes, leads to)
    pub causal_ratio: f32,
    /// Ratio of memory/retrieval indicators (remember, recall, previously, earlier)
    pub recall_ratio: f32,
    /// Ratio of uncertainty/curiosity words (wonder, explore, maybe, perhaps)
    pub exploratory_ratio: f32,
    /// Average token length (longer = more deliberate reasoning)
    pub avg_token_len: f32,
    /// Number of questions in input
    pub question_density: f32,
    /// Presence of personal pronouns (first/second person = social/emotional)
    pub social_density: f32,
}

impl RegimeFeatures {
    /// Extract features from input text
    pub fn from_text(text: &str) -> Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let n = words.len().max(1) as f32;

        let symbolic_markers = ["math", "calculate", "proof", "equation", "solve", "=", "+", "-", "×", "÷", "prove", "theorem", "algebra", "geometry", "logic", "if and only if", "therefore", "thus", "hence", "implies", "∀", "∃", "∈", "⊂", "∪", "∩"];
        let emotional_markers = ["feel", "feeling", "emotion", "happy", "sad", "angry", "scared", "love", "hate", "fear", "anxiety", "worried", "hope", "hope", "miss", "lonely", "upset", "frustrated", "excited", "depressed", "anxious", "stressed", "overwhelmed", "cherish", "grateful", "appreciate", "sorry", "apologize", "sad", "hurt"];
        let causal_markers = ["because", "causes", "leads to", "result", "effect", "impact", "consequence", "reason", "since", "due to", "so that", "why", "how", "affects", "influences", "produces", "yields", "entails"];
        let recall_markers = ["i remember", "i think i", "back when", "mentioned", "told me"];
        let exploratory_markers = ["wonder", "maybe", "perhaps", "might", "could be", "what if", "explore", "curious", "not sure", "unsure", "guess", "hypothesize", "speculate", "imagine", "suppose", "assume", "theorize", "probably", "possibly", "seems like"];

        let lower = text.to_lowercase();

        let symbolic_ratio = symbolic_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;
        let emotional_ratio = emotional_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;
        let causal_ratio = causal_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;
        let recall_ratio = recall_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;
        let exploratory_ratio = exploratory_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;

        let question_density = text.matches(['?', '¿']).count() as f32 / n;

        let social_markers = ["i ", "you ", "we ", "my ", "your ", "me ", "us ", "our ", "i'm ", "you're ", "i'll ", "you'll ", "i've ", "you've ", "i'd ", "you'd "];
        let social_density = social_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f32 / n;

        let avg_token_len = words.iter()
            .map(|w| w.len() as f32)
            .sum::<f32>() / n;

        Self {
            symbolic_ratio,
            emotional_ratio,
            causal_ratio,
            recall_ratio,
            exploratory_ratio,
            avg_token_len,
            question_density,
            social_density,
        }
    }

    /// Classify into a reasoning regime using heuristic rules
    pub fn classify(&self) -> ReasoningRegime {
        let t = 0.06; // require meaningful signal

        // Emotional resonance: high emotional + social density
        if self.emotional_ratio > t && self.social_density > 0.1 {
            return ReasoningRegime::EmotionalResonance;
        }

        // Causal reasoning: high causal markers
        if self.causal_ratio > t {
            return ReasoningRegime::CausalReasoning;
        }

        // Symbolic manipulation: high symbolic ratio
        if self.symbolic_ratio > t {
            return ReasoningRegime::SymbolicManipulation;
        }

        // Associative recall: high recall markers
        if self.recall_ratio > t {
            return ReasoningRegime::AssociativeRecall;
        }

        // Exploratory: uncertainty markers and questions
        if self.exploratory_ratio > t || self.question_density > 0.1 {
            return ReasoningRegime::Exploratory;
        }

        // Default: steady state
        ReasoningRegime::SteadyState
    }

    /// Return all regime scores as a vector (for learned classifier)
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.symbolic_ratio,
            self.emotional_ratio,
            self.causal_ratio,
            self.recall_ratio,
            self.exploratory_ratio,
            self.avg_token_len / 20.0,  // normalize
            self.question_density,
            self.social_density,
        ]
    }
}

/// A scored regime prediction
#[derive(Debug, Clone)]
pub struct RegimePrediction {
    pub regime: ReasoningRegime,
    pub confidence: f32,
    pub features: RegimeFeatures,
}

impl RegimePrediction {
    pub fn classify(text: &str) -> Self {
        let features = RegimeFeatures::from_text(text);
        let regime = features.classify();
        let confidence = Self::confidence_from_features(&features, regime);
        Self { regime, confidence, features }
    }

    fn confidence_from_features(features: &RegimeFeatures, regime: ReasoningRegime) -> f32 {
        match regime {
            ReasoningRegime::SymbolicManipulation => (features.symbolic_ratio * 10.0).clamp(0.0, 1.0),
            ReasoningRegime::EmotionalResonance => (features.emotional_ratio * 10.0).clamp(0.0, 1.0),
            ReasoningRegime::CausalReasoning => (features.causal_ratio * 10.0).clamp(0.0, 1.0),
            ReasoningRegime::AssociativeRecall => (features.recall_ratio * 10.0).clamp(0.0, 1.0),
            ReasoningRegime::Exploratory => ((features.exploratory_ratio + features.question_density) * 5.0).clamp(0.0, 1.0),
            ReasoningRegime::SteadyState => {
                let total_signal = features.symbolic_ratio + features.emotional_ratio
                    + features.causal_ratio + features.recall_ratio + features.exploratory_ratio;
                (1.0 - total_signal * 5.0).clamp(0.3, 0.9)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_classification() {
        let emotional = "I feel really lonely and I'm scared about what happens next";
        let p = RegimePrediction::classify(emotional);
        assert_eq!(p.regime, ReasoningRegime::EmotionalResonance);

        let symbolic = "Prove that the sum of two even numbers is even using modular arithmetic";
        let p = RegimePrediction::classify(symbolic);
        assert_eq!(p.regime, ReasoningRegime::SymbolicManipulation);

        let causal = "Because the temperature rose, the pressure increased therefore the system became unstable";
        let p = RegimePrediction::classify(causal);
        assert_eq!(p.regime, ReasoningRegime::CausalReasoning);

        let recall = "I remember when we went to Paris, that was such a good trip";
        let p = RegimePrediction::classify(recall);
        assert_eq!(p.regime, ReasoningRegime::AssociativeRecall);

        let exploratory = "I wonder what would happen if we tried a different approach, maybe we could explore";
        let p = RegimePrediction::classify(exploratory);
        assert_eq!(p.regime, ReasoningRegime::Exploratory);

        let steady = "The meeting is at 3pm don't forget";
        let p = RegimePrediction::classify(steady);
        assert_eq!(p.regime, ReasoningRegime::SteadyState);
    }

    #[test]
    fn test_confidence() {
        let p = RegimePrediction::classify("I'm so frustrated and angry about this situation");
        assert!(p.confidence > 0.5);
    }
}
