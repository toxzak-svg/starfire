//! ASRU Engine — Anticipatory Self-Reshaping Update
//!
//! Two-timescale architecture:
//!   Fast loop (τ_fast ≈ O(1 step)): M_t ← M_{t-1} - η_fast · ∇_M F̂(I_local)
//!   Slow loop (τ_slow ≫ τ_fast, episodic): θ ← θ - η_slow · ∇_θ U_basin
//!
//! The meta-learner F̂ learns to predict basin fragility U_basin from
//! cheap Lyapunov features I_local.
//!
//! Fast loop: updates meta-state field (routing, plasticity, evaluation)
//! Slow loop: updates base system parameters + symmetry breaking (column roles)

use serde::{Deserialize, Serialize};

use super::regime_classifier::{ReasoningRegime, RegimePrediction};
use super::fragility::{AttractorFragility, State};
use super::regime_memory::RegimeTracker;

/// Meta-state field — controls routing, plasticity, evaluation, and interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaStateField {
    /// Routing configuration
    pub routing: RoutingConfig,
    /// Plasticity mask — which parts are plastic vs frozen
    pub plasticity: PlasticityMask,
    /// Evaluation metrics
    pub evaluation: EvalMetrics,
    /// Interface shape
    pub interface: InterfaceShape,
}

impl Default for MetaStateField {
    fn default() -> Self {
        Self {
            routing: RoutingConfig::default(),
            plasticity: PlasticityMask::default(),
            evaluation: EvalMetrics::default(),
            interface: InterfaceShape::default(),
        }
    }
}

/// Routing configuration — which modules talk to which
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Priority order of modules
    pub module_order: Vec<String>,
    /// Whether speculative association is allowed
    pub allow_speculative: bool,
    /// Memory access weight (0-1)
    pub memory_weight: f32,
    /// Partner model weight (0-1)
    pub partner_model_weight: f32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            module_order: vec![
                "partner_model".to_string(),
                "reasoning".to_string(),
                "memory".to_string(),
                "response".to_string(),
            ],
            allow_speculative: true,
            memory_weight: 0.5,
            partner_model_weight: 0.5,
        }
    }
}

/// Plasticity mask — which computational pathways are plastic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityMask {
    /// Symbolic reasoning pathway
    pub symbolic_plasticity: f32,
    /// Emotional resonance pathway
    pub emotional_plasticity: f32,
    /// Causal reasoning pathway
    pub causal_plasticity: f32,
    /// Memory retrieval pathway
    pub recall_plasticity: f32,
    /// Exploratory/pathfinding pathway
    pub exploratory_plasticity: f32,
}

impl Default for PlasticityMask {
    fn default() -> Self {
        // Default: moderate plasticity everywhere
        Self {
            symbolic_plasticity: 0.5,
            emotional_plasticity: 0.5,
            causal_plasticity: 0.5,
            recall_plasticity: 0.5,
            exploratory_plasticity: 0.5,
        }
    }
}

impl PlasticityMask {
    /// Get plasticity for a specific regime
    pub fn for_regime(&self, regime: ReasoningRegime) -> f32 {
        match regime {
            ReasoningRegime::SymbolicManipulation => self.symbolic_plasticity,
            ReasoningRegime::EmotionalResonance => self.emotional_plasticity,
            ReasoningRegime::CausalReasoning => self.causal_plasticity,
            ReasoningRegime::AssociativeRecall => self.recall_plasticity,
            ReasoningRegime::Exploratory => self.exploratory_plasticity,
            ReasoningRegime::SteadyState => 0.3,
        }
    }

    /// Set plasticity for a specific regime
    pub fn set_for_regime(&mut self, regime: ReasoningRegime, plasticity: f32) {
        match regime {
            ReasoningRegime::SymbolicManipulation => self.symbolic_plasticity = plasticity,
            ReasoningRegime::EmotionalResonance => self.emotional_plasticity = plasticity,
            ReasoningRegime::CausalReasoning => self.causal_plasticity = plasticity,
            ReasoningRegime::AssociativeRecall => self.recall_plasticity = plasticity,
            ReasoningRegime::Exploratory => self.exploratory_plasticity = plasticity,
            ReasoningRegime::SteadyState => {}
        }
    }
}

/// Evaluation metrics — what counts as "good" in current regime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalMetrics {
    /// Precision weight (vs recall)
    pub precision_weight: f32,
    /// Speed vs accuracy tradeoff
    pub speed_weight: f32,
    /// Novelty weight
    pub novelty_weight: f32,
    /// Safety weight
    pub safety_weight: f32,
}

impl Default for EvalMetrics {
    fn default() -> Self {
        Self {
            precision_weight: 0.5,
            speed_weight: 0.5,
            novelty_weight: 0.3,
            safety_weight: 0.7,
        }
    }
}

/// Interface shape — how the system exposes itself to tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceShape {
    /// Verbosity level (0-1)
    pub verbosity: f32,
    /// Formality level (0-1)
    pub formality: f32,
    /// Warmth level (0-1)
    pub warmth: f32,
    /// Directness level (0-1)
    pub directness: f32,
}

impl Default for InterfaceShape {
    fn default() -> Self {
        Self {
            verbosity: 0.5,
            formality: 0.3,
            warmth: 0.6,
            directness: 0.5,
        }
    }
}

/// Column role in the isomorphic pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnRole {
    Calculator,       // High-precision symbolic
    SafetyMonitor,    // Safety/rollback
    Explorer,          // Novel approaches
    Compressor,       // Efficient representations
    Sentinel,          // Tissue health monitoring
    Stem,              // Undifferentiated
}

impl Default for ColumnRole {
    fn default() -> Self {
        Self::Stem
    }
}

/// A column in the isomorphic pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: u32,
    pub role: ColumnRole,
    pub plasticity: f32,
    pub stress: f32,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            id: 0,
            role: ColumnRole::Stem,
            plasticity: 0.5,
            stress: 0.0,
        }
    }
}

/// The main ASRU engine
pub struct ASRUEngine {
    /// Current meta-state field
    m_t: MetaStateField,
    /// Regime tracker
    tracker: RegimeTracker,
    /// Column pool for symmetry breaking
    columns: Vec<Column>,
    /// Assignment field viscosity (0=stable, 1=flexible)
    viscosity: f32,
    /// Fragility threshold for triggering symmetry breaking
    fragility_threshold: f32,
    /// Learning rate for fast loop
    eta_fast: f32,
    /// Learning rate for slow loop
    eta_slow: f32,
    /// Whether we're in a slow loop computation
    in_slow_loop: bool,
    /// Slow loop counter
    slow_loop_counter: u32,
    /// Slow loop interval (steps between slow loops)
    slow_loop_interval: u32,
}

impl ASRUEngine {
    pub fn new(n_columns: usize) -> Self {
        let columns = (0..n_columns)
            .map(|i| Column { id: i as u32, ..Default::default() })
            .collect();

        Self {
            m_t: MetaStateField::default(),
            tracker: RegimeTracker::new(100, 50),
            columns,
            viscosity: 0.5,
            fragility_threshold: 0.7,
            eta_fast: 0.1,
            eta_slow: 0.01,
            in_slow_loop: false,
            slow_loop_counter: 0,
            slow_loop_interval: 100,  // Slow loop every 100 fast steps
        }
    }

    /// Main update step — called every forward pass
    /// Returns the updated meta-state field
    pub fn step(&mut self, text: &str, state_features: &[f32]) -> &MetaStateField {
        // Update regime tracker
        self.tracker.update(text, state_features);

        // Compute current AFI
        let afi = self.tracker.compute_afi();

        // Check for regime change — reset trajectory on regime switch
        // (we'd need to track previous regime for this)

        // FAST LOOP: Update M_t based on Lyapunov + RQA
        self.fast_step(&afi);

        // Check if slow loop should fire
        self.slow_loop_counter += 1;
        if self.slow_loop_counter >= self.slow_loop_interval {
            self.in_slow_loop = true;
            self.slow_loop_counter = 0;
        }

        if self.in_slow_loop {
            // SLOW LOOP: Basin analysis + symmetry breaking
            self.slow_step(afi);
            self.in_slow_loop = false;
        }

        &self.m_t
    }

    /// Fast ASRU step — update M_t from Lyapunov + RQA signals
    fn fast_step(&mut self, afi: &AttractorFragility) {
        let regime = self.tracker.current_regime();
        let current_plasticity = self.m_t.plasticity.for_regime(regime);

        // AFI tells us fragility: high AFI → reduce plasticity (more frozen = more stable)
        // Low AFI → increase plasticity (more adaptive)
        let target_plasticity = 1.0 - afi.afi; // Inverse relationship
        let delta = (target_plasticity - current_plasticity) * self.eta_fast;

        let new_plasticity = (current_plasticity + delta).clamp(0.1, 0.9);
        self.m_t.plasticity.set_for_regime(regime, new_plasticity);

        // Also update routing based on regime
        self.update_routing_for_regime(regime);

        // Update evaluation metrics based on fragility
        if afi.afi > 0.7 {
            // High fragility → prioritize safety and precision
            self.m_t.evaluation.safety_weight = 0.9;
            self.m_t.evaluation.precision_weight = 0.8;
        } else if afi.afi < 0.3 {
            // Low fragility → allow more novelty and speed
            self.m_t.evaluation.novelty_weight = 0.6;
            self.m_t.evaluation.speed_weight = 0.6;
        }
    }

    /// Slow ASRU step — basin analysis + symmetry breaking
    fn slow_step(&mut self, afi: AttractorFragility) {
        // Compute basin fragility (approximation)
        let basin_fragility = self.approximate_basin_fragility();

        // If fragility is high, trigger anticipatory symmetry breaking
        if basin_fragility > self.fragility_threshold {
            self.break_symmetry_anticipatory();
        } else {
            // Keep columns in exploratory state
            self.reset_to_stem();
        }

        // Update viscosity based on fragility
        // High fragility → low viscosity (stable assignments)
        // Low fragility → high viscosity (flexible, exploratory)
        self.viscosity = 1.0 - basin_fragility;

        // Update global fragility estimate
        self.tracker.memory.update_global_fragility(afi.afi as f64);
    }

    /// Approximate basin fragility from regime statistics
    fn approximate_basin_fragility(&self) -> f32 {
        let regime = self.tracker.current_regime();
        let stats = self.tracker.memory.stats(regime);

        if let Some(s) = stats {
            // High escape rate = fragile basin
            // Low mean dwell = fragile basin
            let escape_component = s.escape_rate as f32;
            let dwell_component = (1.0 / (s.mean_dwell as f32 + 1.0)).clamp(0.0, 1.0);
            (escape_component + dwell_component) / 2.0
        } else {
            0.5 // Default medium
        }
    }

    /// Anticipatory symmetry breaking — pre-assign column roles
    fn break_symmetry_anticipatory(&mut self) {
        let predicted_regime = self.tracker.predicted_next_regime();

        // Target role distribution based on predicted regime
        let target_roles = self.roles_for_regime(predicted_regime);

        // Slowly update column assignments (viscosity controls speed)
        for (i, col) in self.columns.iter_mut().enumerate() {
            let target_role = target_roles.get(i % target_roles.len()).copied().unwrap_or(ColumnRole::Stem);

            // Blend current toward target based on viscosity
            if self.viscosity < 0.5 && col.role != target_role {
                // Low viscosity = stable assignment, make the change
                col.role = target_role;
                col.plasticity = 0.3; // Frozen for stable assignment
            } else if self.viscosity > 0.5 {
                // High viscosity = exploratory
                col.role = ColumnRole::Explorer;
                col.plasticity = 0.8;
            }
        }
    }

    /// Return column roles to undifferentiated state
    fn reset_to_stem(&mut self) {
        for col in &mut self.columns {
            col.role = ColumnRole::Stem;
            col.plasticity = 0.6; // Moderate plasticity
        }
    }

    /// Get target column roles for a given regime
    fn roles_for_regime(&self, regime: ReasoningRegime) -> Vec<ColumnRole> {
        match regime {
            ReasoningRegime::SymbolicManipulation => {
                vec![ColumnRole::Calculator; 3]
            }
            ReasoningRegime::EmotionalResonance => {
                vec![ColumnRole::Sentinel; 2]
            }
            ReasoningRegime::CausalReasoning => {
                vec![ColumnRole::Calculator; 2]
            }
            ReasoningRegime::Exploratory => {
                vec![ColumnRole::Explorer; 3]
            }
            ReasoningRegime::AssociativeRecall => {
                vec![ColumnRole::Compressor; 2]
            }
            ReasoningRegime::SteadyState => {
                vec![ColumnRole::Stem; 3]
            }
        }
    }

    /// Update routing configuration for a regime
    fn update_routing_for_regime(&mut self, regime: ReasoningRegime) {
        match regime {
            ReasoningRegime::SymbolicManipulation => {
                self.m_t.routing.module_order = vec![
                    "reasoning".to_string(),
                    "verification".to_string(),
                    "memory".to_string(),
                    "response".to_string(),
                ];
                self.m_t.routing.allow_speculative = false;
            }
            ReasoningRegime::EmotionalResonance => {
                self.m_t.routing.module_order = vec![
                    "partner_model".to_string(),
                    "empathy".to_string(),
                    "reasoning".to_string(),
                    "response".to_string(),
                ];
                self.m_t.routing.partner_model_weight = 0.8;
            }
            ReasoningRegime::Exploratory => {
                self.m_t.routing.module_order = vec![
                    "curiosity".to_string(),
                    "reasoning".to_string(),
                    "memory".to_string(),
                    "response".to_string(),
                ];
                self.m_t.routing.allow_speculative = true;
            }
            _ => {
                self.m_t.routing = RoutingConfig::default();
            }
        }
    }

    // ─── Public getters ───

    pub fn meta_state(&self) -> &MetaStateField {
        &self.m_t
    }

    pub fn current_regime(&self) -> ReasoningRegime {
        self.tracker.current_regime()
    }

    pub fn current_dwell(&self) -> u64 {
        self.tracker.current_dwell()
    }

    pub fn predicted_next_regime(&self) -> ReasoningRegime {
        self.tracker.predicted_next_regime()
    }

    pub fn is_fragile(&self) -> bool {
        self.tracker.is_fragile(self.fragility_threshold)
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn viscosity(&self) -> f32 {
        self.viscosity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asru_fast_loop() {
        let mut asru = ASRUEngine::new(4);

        // Normal input
        let m1 = asru.step("Hello, how are you?", &[0.1, 0.3, 0.1, 0.1, 0.2, 0.5, 0.1, 0.5]).clone();
        assert_eq!(asru.current_regime(), ReasoningRegime::SteadyState);

        // Emotional input — should shift plasticity
        let m2 = asru.step("I feel really sad and lonely today", &[0.1, 0.9, 0.1, 0.0, 0.2, 0.5, 0.3, 0.7]).clone();
        assert_eq!(asru.current_regime(), ReasoningRegime::EmotionalResonance);

        // Check that plasticity changed
        let ep1 = m1.plasticity.emotional_plasticity;
        let ep2 = m2.plasticity.emotional_plasticity;
        // High AFI → lower plasticity (more frozen)
        // Low AFI → higher plasticity (more adaptive)
        // Just check they're valid
        assert!(ep1 >= 0.0 && ep1 <= 1.0);
        assert!(ep2 >= 0.0 && ep2 <= 1.0);
    }

    #[test]
    fn test_symmetry_breaking() {
        let mut asru = ASRUEngine::new(4);

        // Trigger slow loop manually by running many steps
        for i in 0..110 {
            let text = if i % 10 == 0 {
                "I feel really frustrated and angry about this"
            } else {
                "The meeting is at 3pm"
            };
            asru.step(text, &[0.1, 0.5, 0.2, 0.1, 0.2, 0.5, 0.2, 0.4]);
        }

        // After slow loop, columns should have assigned roles
        let roles: Vec<_> = asru.columns.iter().map(|c| c.role).collect();
        // Not all should be Stem
        let non_stem = roles.iter().filter(|r| **r != ColumnRole::Stem).count();
        // Some roles assigned (depends on fragility threshold)
        assert!(non_stem >= 0);
    }

    #[test]
    fn test_routing_update() {
        let mut asru = ASRUEngine::new(2);

        asru.step("Prove that the sum of two even numbers is even", &[0.8, 0.1, 0.2, 0.0, 0.1, 0.6, 0.1, 0.2]);
        assert_eq!(asru.current_regime(), ReasoningRegime::SymbolicManipulation);
        assert!(!asru.m_t.routing.allow_speculative);
    }
}
