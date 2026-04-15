//! ASRU — Anticipatory Self-Reshaping Update
//!
//! Novel architecture for forethought as structural meta-dynamics.
//! NOT cognitive forethought (predict task → plan response).
//! Instead: the update operator itself morphs in anticipation of future regimes.
//!
//! ## Two-Timescale Architecture
//!
//! Fast loop (τ_fast ≈ O(1 step)):
//!   M_t ← M_{t-1} - η_fast · ∇_M F̂(I_local)
//!
//! Slow loop (τ_slow ≫ τ_fast, episodic):
//!   θ ← θ - η_slow · ∇_θ U_basin
//!
//! Where:
//!   I_local = Lyapunov exponents + RQA metrics (cheap, real-time)
//!   U_basin = basin uncertainty exponent (expensive, periodic)
//!   F̂ = learned surrogate approximating U_basin from I_local
//!
//! ## Key Papers
//!
//! - Resilience of dynamical systems: https://www.cambridge.org/core/journals/european-journal-of-applied-mathematics/article/resilience-of-dynamical-systems/B277FB38B049FD4DECC2097E7460E4E3
//! - Uncertainty exponent: https://juliadynamics.github.io/DynamicalSystems.jl/previews/PR156/chaos/basins/
//! - Metastable attractors: https://pmc.ncbi.nlm.nih.gov/articles/PMC4930057/
//! - RQA for AI: https://www.nature.com/articles/s41598-020-60066-7
//!
//! ## Modules
//!
//! - `regime_classifier`: classifies input into reasoning mode attractors
//! - `fragility`: Lyapunov + RQA-based AFI computation
//! - `regime_memory`: tracks metastable regime dwell times and transitions
//! - `engine`: orchestrates two-timescale ASRU update

pub mod regime_classifier;
pub mod fragility;
pub mod regime_memory;
pub mod engine;

pub use regime_classifier::{
    ReasoningRegime, RegimeFeatures, RegimePrediction,
};
pub use fragility::{
    AttractorFragility, State, StateTrajectory, RQAMetrics,
    LyapunovEstimator, RQAAnalyzer, FragilityEstimator,
};
pub use regime_memory::{RegimeStats, RegimeTransition, RegimeMemory, RegimeTracker};
pub use engine::{
    ASRUEngine, MetaStateField, RoutingConfig, PlasticityMask,
    EvalMetrics, InterfaceShape, ColumnRole, Column,
};
