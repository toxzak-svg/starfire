//! TCMW-A: Temporal Causal Memory Weaving + Anticipation
//!
//! Five interlocking layers giving Starfire a learned model of Zach's
//! behavioral grammar so it can pre-stage resources and propose actions
//! *before being asked*.
//!
//! Pure Rust ML: Markov chains, k-means clustering, EMA updates.

pub mod cef;
pub mod bge;
pub mod aih;
pub mod pse;
pub mod oafl;

pub use cef::Outcome;
pub use aih::IntentPrediction;
pub use pse::{StagedAction, StagedActionType};

use serde::{Deserialize, Serialize};
use cef::{CEF, CausalEvent};
use bge::{BGE, SessionArchetype, ArchetypeId};
use aih::AIH;
use pse::{PSE, StagedStatus};
use oafl::{OAFL, PredictionDelta, MatchQuality};

/// TCMW-A configuration
#[derive(Debug, Clone)]
pub struct TCMWConfig {
    pub lambda: f64,
    pub cone_depth: usize,
    pub staging_threshold: f64,
    pub half_life_secs: i64,
    pub max_archetypes: usize,
    pub k_means_k: usize,
    pub ema_alpha: f64,
    pub revision_threshold: f64,
}

impl Default for TCMWConfig {
    fn default() -> Self {
        Self {
            lambda: 0.15,
            cone_depth: 5,
            staging_threshold: 0.5,
            half_life_secs: 3600,
            max_archetypes: 20,
            k_means_k: 5,
            ema_alpha: 0.3,
            revision_threshold: 0.4,
        }
    }
}

/// TCMW-A Engine — orchestrates all five layers
#[derive(Debug, Clone)]
pub struct TCMWEngine {
    config: TCMWConfig,
    cef: CEF,
    bge: BGE,
    aih: AIH,
    pse: PSE,
    oafl: OAFL,
    staged_actions: Vec<StagedAction>,
    event_history: Vec<CausalEvent>,
}

impl Default for TCMWEngine {
    fn default() -> Self {
        Self::new(TCMWConfig::default())
    }
}

impl TCMWEngine {
    pub fn new(config: TCMWConfig) -> Self {
        Self {
            config: config.clone(),
            cef: CEF::new(config.half_life_secs),
            bge: BGE::new(config.max_archetypes, config.k_means_k),
            aih: AIH::new(config.lambda, config.cone_depth),
            pse: PSE::new(config.staging_threshold),
            oafl: OAFL::new(config.ema_alpha, config.revision_threshold),
            staged_actions: Vec::new(),
            event_history: Vec::new(),
        }
    }

    pub fn on_user_action(&mut self, action: &str, parent_action: Option<&str>, outcome: Outcome) {
        let event = self.cef.record(action, parent_action, outcome);
        self.event_history.push(event.clone());

        let archetype = self.bge.observe_action(&action, &self.event_history);

        // Update Markov transition
        let prev_arch = self.event_history.len().saturating_sub(2);
        if prev_arch < self.event_history.len() {
            if let Some(prev_arch_id) = self.event_history[prev_arch].archetype_id {
                if let Some(curr_arch) = archetype.as_ref() {
                    self.bge.markov_mut().update(prev_arch_id, curr_arch.id);
                }
            }
        }

        let predictions = self.aih.build_cone(&self.bge, &self.cef, &self.event_history);

        self.staged_actions.clear();
        for pred in &predictions {
            if pred.probability >= self.config.staging_threshold {
                let staged = self.pse.stage(pred.clone(), StagedActionType::Draft {
                    text: format!("Based on your pattern, you might want to: {}", pred.action),
                });
                self.staged_actions.push(staged);
            }
        }
    }

    pub fn on_action_confirmed(&mut self, actual_action: &str) {
        if let Some(top_pred) = self.aih.top_predictions(1).first() {
            let delta = self.oafl.compute_delta(top_pred, actual_action);
            self.oafl.record(delta.clone());

            if matches!(delta.quality, MatchQuality::Miss) {
                self.bge.trigger_revision();
            }

            if let Some(arch_id) = &top_pred.archetype_id {
                self.aih.adapt_lambda(arch_id, &delta, &mut self.bge);
            }
        }
    }

    pub fn get_predictions(&self) -> Vec<IntentPrediction> {
        self.aih.top_predictions(self.config.cone_depth)
    }

    pub fn get_staged_actions(&self) -> &[StagedAction] {
        &self.staged_actions
    }

    pub fn cancel_staged(&mut self, id: pse::StagedId) {
        self.staged_actions.retain(|a| a.id != id);
    }

    pub fn current_archetype_label(&self) -> Option<String> {
        self.bge.current_archetype().map(|a| a.label.clone())
    }

    pub fn stats(&self) -> TCMWStats {
        TCMWStats {
            events_recorded: self.event_history.len(),
            archetypes_tracked: self.bge.archetype_count(),
            pending_predictions: self.aih.predictions_len(),
            staged_actions: self.staged_actions.len(),
            oafl_miss_rate: self.oafl.miss_rate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TCMWStats {
    pub events_recorded: usize,
    pub archetypes_tracked: usize,
    pub pending_predictions: usize,
    pub staged_actions: usize,
    pub oafl_miss_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcmw_record_action() {
        let mut engine = TCMWEngine::default();
        engine.on_user_action("open VSCode", None, Outcome::Success);
        engine.on_user_action("write code", Some("open VSCode"), Outcome::Success);
        engine.on_user_action("run tests", Some("write code"), Outcome::Success);
        assert!(engine.event_history.len() >= 3);
        assert!(engine.stats().archetypes_tracked > 0);
    }

    #[test]
    fn test_staging_threshold() {
        let mut config = TCMWConfig::default();
        config.staging_threshold = 0.9;
        let mut engine = TCMWEngine::new(config);
        engine.on_user_action("open VSCode", None, Outcome::Success);
        engine.on_user_action("write code", Some("open VSCode"), Outcome::Success);
        assert!(engine.get_staged_actions().is_empty());
    }

    #[test]
    fn test_feedback_loop() {
        let mut engine = TCMWEngine::default();
        engine.on_user_action("open VSCode", None, Outcome::Success);
        engine.on_user_action("write code", Some("open VSCode"), Outcome::Success);
        engine.on_user_action("run tests", Some("write code"), Outcome::Success);
        engine.on_action_confirmed("run tests");
        assert!(!engine.oafl.recent_deltas().is_empty());
    }

    #[test]
    fn test_stats() {
        let mut engine = TCMWEngine::default();
        engine.on_user_action("open VSCode", None, Outcome::Success);
        engine.on_user_action("write code", Some("open VSCode"), Outcome::Success);
        let stats = engine.stats();
        assert!(stats.events_recorded >= 2);
    }
}
