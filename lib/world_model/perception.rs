//! Perception — Quanot to Starfire bridge
//!
//! Defines the interface for receiving Quanot reservoir states and
//! converting them into structured input for the World Model.

use serde::{Deserialize, Serialize};

/// Quanot perception input — raw output from Quanot's reservoir and consciousness systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuanotPerception {
    /// The reservoir state vector (N-dimensional, typically N=1000)
    pub reservoir_state: Vec<f64>,
    /// Consciousness proxy ψ (0-1) from IIT-inspired metrics
    pub consciousness_proxy: f64,
    /// Novelty score (0-1) — how different this state is from history
    pub novelty: f64,
    /// Full creativity output from Quanot
    pub creativity_scores: CreativityOutput,
}

impl QuanotPerception {
    /// Create a new perception from Quanot outputs
    pub fn new(
        reservoir_state: Vec<f64>,
        consciousness_proxy: f64,
        novelty: f64,
        creativity_scores: CreativityOutput,
    ) -> Self {
        Self {
            reservoir_state,
            consciousness_proxy: consciousness_proxy.clamp(0.0, 1.0),
            novelty: novelty.clamp(0.0, 1.0),
            creativity_scores,
        }
    }

    /// Get the dimensionality of the reservoir state
    pub fn dimensionality(&self) -> usize {
        self.reservoir_state.len()
    }

    /// Compute the norm of the reservoir state
    pub fn state_norm(&self) -> f64 {
        self.reservoir_state
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
    }

    /// Get the mean activation of the reservoir
    pub fn mean_activation(&self) -> f64 {
        if self.reservoir_state.is_empty() {
            return 0.0;
        }
        self.reservoir_state.iter().sum::<f64>() / self.reservoir_state.len() as f64
    }

    /// Check if this represents a high-consciousness event
    pub fn is_high_consciousness(&self) -> bool {
        self.consciousness_proxy > 0.7
    }

    /// Check if this represents a novel state
    pub fn is_novel(&self) -> bool {
        self.novelty > 0.4
    }
}

impl Default for QuanotPerception {
    fn default() -> Self {
        Self {
            reservoir_state: Vec::new(),
            consciousness_proxy: 0.0,
            novelty: 0.0,
            creativity_scores: CreativityOutput::default(),
        }
    }
}

/// Creativity output from Quanot's creative oscillation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativityOutput {
    /// Current creative state (0-1)
    pub creative_state: f64,
    /// Divergence metric — how different from typical attractor basin
    pub divergence_metric: f64,
    /// Diversity index — spread of activation across reservoir dimensions
    pub diversity_index: f64,
    /// Originality score — how unique this state's pattern is
    pub originality_score: f64,
    /// Oscillation phase (radians)
    pub oscillation_phase: f64,
}

impl Default for CreativityOutput {
    fn default() -> Self {
        Self {
            creative_state: 0.0,
            divergence_metric: 0.0,
            diversity_index: 0.0,
            originality_score: 0.0,
            oscillation_phase: 0.0,
        }
    }
}

impl CreativityOutput {
    pub fn new(
        creative_state: f64,
        divergence_metric: f64,
        diversity_index: f64,
        originality_score: f64,
        oscillation_phase: f64,
    ) -> Self {
        Self {
            creative_state: creative_state.clamp(0.0, 1.0),
            divergence_metric: divergence_metric.clamp(0.0, 1.0),
            diversity_index: diversity_index.clamp(0.0, 1.0),
            originality_score: originality_score.clamp(0.0, 1.0),
            oscillation_phase,
        }
    }

    /// Compute a combined creativity score
    pub fn combined_score(&self) -> f64 {
        (self.creative_state + self.divergence_metric + self.diversity_index + self.originality_score) / 4.0
    }
}

/// Processed perception features extracted from raw Quanot output
#[derive(Debug, Clone)]
pub struct ProcessedPerception {
    /// Extracted features from reservoir state
    pub features: ReservoirFeatures,
    /// Consciousness level category
    pub consciousness_level: ConsciousnessLevel,
    /// Creativity assessment
    pub creativity: CreativityAssessment,
    /// Recommendations for Starfire processing
    pub recommendations: Vec<PerceptionRecommendation>,
}

/// Features extracted from the reservoir state vector
#[derive(Debug, Clone)]
pub struct ReservoirFeatures {
    /// Mean activation
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Maximum activation
    pub max: f64,
    /// Minimum activation
    pub min: f64,
    /// Sparsity (% near-zero activations)
    pub sparsity: f64,
    /// Skewness of activation distribution
    pub skewness: f64,
}

impl ReservoirFeatures {
    pub fn from_state(state: &[f64]) -> Self {
        if state.is_empty() {
            return Self {
                mean: 0.0,
                std_dev: 0.0,
                max: 0.0,
                min: 0.0,
                sparsity: 0.0,
                skewness: 0.0,
            };
        }

        let n = state.len() as f64;
        let mean = state.iter().sum::<f64>() / n;
        let variance = state.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let max = state.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = state.iter().cloned().fold(f64::INFINITY, f64::min);
        let near_zero = state.iter().filter(|&&x| x.abs() < 0.01).count() as f64;
        let sparsity = near_zero / n;

        // Simple skewness approximation
        let skewness = if std_dev > 0.0 {
            state.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n
        } else {
            0.0
        };

        Self {
            mean,
            std_dev,
            max,
            min,
            sparsity,
            skewness,
        }
    }
}

/// Consciousness level categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ConsciousnessLevel {
    /// ψ < 0.3 — minimal awareness
    Minimal,
    /// ψ 0.3-0.5 — low consciousness
    Low,
    /// ψ 0.5-0.7 — moderate consciousness
    Moderate,
    /// ψ 0.7-0.85 — high consciousness
    High,
    /// ψ > 0.85 — peak consciousness
    Peak,
}

impl From<f64> for ConsciousnessLevel {
    fn from(psi: f64) -> Self {
        if psi < 0.3 {
            ConsciousnessLevel::Minimal
        } else if psi < 0.5 {
            ConsciousnessLevel::Low
        } else if psi < 0.7 {
            ConsciousnessLevel::Moderate
        } else if psi < 0.85 {
            ConsciousnessLevel::High
        } else {
            ConsciousnessLevel::Peak
        }
    }
}

impl ConsciousnessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsciousnessLevel::Minimal => "minimal awareness",
            ConsciousnessLevel::Low => "low consciousness",
            ConsciousnessLevel::Moderate => "moderate consciousness",
            ConsciousnessLevel::High => "high consciousness",
            ConsciousnessLevel::Peak => "peak consciousness",
        }
    }
}

/// Creativity assessment from Quanot output
#[derive(Debug, Clone)]
pub struct CreativityAssessment {
    pub level: CreativityLevel,
    pub divergence: f64,
    pub diversity: f64,
    pub originality: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum CreativityLevel {
    /// Low creative state
    Reproducive,
    /// Moderate creative state
    Adaptive,
    /// High creative state
    Creative,
    /// Peak creative state
    HighlyCreative,
}

impl From<&CreativityOutput> for CreativityAssessment {
    fn from(co: &CreativityOutput) -> Self {
        let combined = co.combined_score();
        let level = if combined < 0.25 {
            CreativityLevel::Reproducive
        } else if combined < 0.5 {
            CreativityLevel::Adaptive
        } else if combined < 0.75 {
            CreativityLevel::Creative
        } else {
            CreativityLevel::HighlyCreative
        };

        Self {
            level,
            divergence: co.divergence_metric,
            diversity: co.diversity_index,
            originality: co.originality_score,
        }
    }
}

/// Recommendations for how Starfire should process this perception
#[derive(Debug, Clone, PartialEq)]
pub enum PerceptionRecommendation {
    /// Store as episodic memory
    StoreEpisodic,
    /// Trigger curiosity-driven exploration
    TriggerCuriosity,
    /// High novelty — investigate for causal patterns
    InvestigateCausal,
    /// High consciousness — important, high priority
    HighPriority,
    /// Normal processing
    Normal,
}

impl QuanotPerception {
    /// Process and analyze this perception into structured recommendations
    pub fn process(&self) -> ProcessedPerception {
        let features = ReservoirFeatures::from_state(&self.reservoir_state);
        let consciousness_level = ConsciousnessLevel::from(self.consciousness_proxy);
        let creativity = CreativityAssessment::from(&self.creativity_scores);

        let mut recommendations = vec![PerceptionRecommendation::Normal];

        if consciousness_level >= ConsciousnessLevel::High {
            recommendations.push(PerceptionRecommendation::HighPriority);
        }

        if self.novelty > 0.5 {
            recommendations.push(PerceptionRecommendation::InvestigateCausal);
        }

        if creativity.level >= CreativityLevel::Creative {
            recommendations.push(PerceptionRecommendation::TriggerCuriosity);
        }

        if features.sparsity > 0.7 {
            recommendations.push(PerceptionRecommendation::StoreEpisodic);
        }

        ProcessedPerception {
            features,
            consciousness_level,
            creativity,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creativity_output_default() {
        let co = CreativityOutput::default();
        assert_eq!(co.combined_score(), 0.0);
    }

    #[test]
    fn test_creativity_output_combined() {
        let co = CreativityOutput::new(0.8, 0.6, 0.7, 0.5, 0.0);
        assert!((co.combined_score() - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_consciousness_level_from_psi() {
        assert_eq!(ConsciousnessLevel::from(0.2), ConsciousnessLevel::Minimal);
        assert_eq!(ConsciousnessLevel::from(0.4), ConsciousnessLevel::Low);
        assert_eq!(ConsciousnessLevel::from(0.6), ConsciousnessLevel::Moderate);
        assert_eq!(ConsciousnessLevel::from(0.8), ConsciousnessLevel::High);
        assert_eq!(ConsciousnessLevel::from(0.95), ConsciousnessLevel::Peak);
    }

    #[test]
    fn test_reservoir_features() {
        let state = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let features = ReservoirFeatures::from_state(&state);

        assert!((features.mean - 0.3).abs() < 0.001);
        assert!(features.max > 0.4);
        assert!(features.min < 0.2);
    }

    #[test]
    fn test_quanot_perception_processing() {
        let perception = QuanotPerception {
            reservoir_state: vec![0.5; 100],
            consciousness_proxy: 0.8,
            novelty: 0.6,
            creativity_scores: CreativityOutput::new(0.7, 0.6, 0.5, 0.4, 0.0),
        };

        let processed = perception.process();

        assert_eq!(processed.consciousness_level, ConsciousnessLevel::High);
        assert!(processed.recommendations.contains(&PerceptionRecommendation::HighPriority));
        assert!(processed.recommendations.contains(&PerceptionRecommendation::InvestigateCausal));
    }

    #[test]
    fn test_perception_defaults() {
        let p = QuanotPerception::default();
        assert!(p.reservoir_state.is_empty());
        assert_eq!(p.consciousness_proxy, 0.0);
        assert_eq!(p.novelty, 0.0);
    }

    #[test]
    fn test_state_norm() {
        let perception = QuanotPerception::new(
            vec![3.0, 4.0], // 3-4-5 right triangle
            0.5,
            0.5,
            CreativityOutput::default(),
        );
        assert!((perception.state_norm() - 5.0).abs() < 0.001);
    }
}
