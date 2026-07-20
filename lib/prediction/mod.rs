//! Prediction Center — Unified prediction API
//!
//! Four distinct prediction engines, each leveraging a different aspect of Star's architecture:
//! - Question Gravity: predicts which curiosity questions will fire
//! - Belief Revision: projects reservoir dynamics to forecast conclusions
//! - Basin: constraint satisfaction as prediction
//! - Meta Prediction: confidence calibration

use std::fmt;

#[cfg(feature = "omega-g2-shadow")]
use crate::omega_g2_shadow::{OmegaG2ShadowObserver, OmegaG2ShadowSnapshot};

pub mod types;
pub mod question_gravity;
pub mod belief_revision;
pub mod basin;
pub mod meta_prediction;
pub mod counterfactual;

pub use types::*;
pub use question_gravity::QuestionGravityEngine;
pub use belief_revision::BeliefRevisionEngine;
pub use basin::BasinEngine;
pub use meta_prediction::MetaPredictionEngine;
pub use counterfactual::CounterfactualEngine;

/// Unified prediction center — queries all engines, returns ranked predictions
pub struct PredictionCenter {
    /// Question Gravity engine
    question_gravity: QuestionGravityEngine,
    /// Belief Revision engine
    belief_revision: BeliefRevisionEngine,
    /// Basin engine
    basin: BasinEngine,
    /// Meta Prediction engine
    meta: MetaPredictionEngine,
    /// Counterfactual engine
    counterfactual: CounterfactualEngine,
    /// Prediction history
    history: Vec<Prediction>,
    /// Maximum history size
    max_history: usize,
    /// ΩG2-S0 receives copies of typed ranking traces and settled outcomes only.
    /// It is absent from default builds and cannot influence this center.
    #[cfg(feature = "omega-g2-shadow")]
    omega_g2_shadow: OmegaG2ShadowObserver,
}

impl PredictionCenter {
    /// Create a new prediction center
    pub fn new() -> Self {
        PredictionCenter {
            question_gravity: QuestionGravityEngine::new(),
            belief_revision: BeliefRevisionEngine::new(),
            basin: BasinEngine::new(),
            meta: MetaPredictionEngine::new(),
            counterfactual: CounterfactualEngine::new(),
            history: Vec::new(),
            max_history: 100,
            #[cfg(feature = "omega-g2-shadow")]
            omega_g2_shadow: OmegaG2ShadowObserver::default(),
        }
    }

    /// Generate all predictions given current context
    /// Called after each conversation exchange
    pub fn generate(&mut self, context: &ConversationContext) -> Vec<Prediction> {
        let mut predictions = Vec::new();

        // 1. Question Gravity predictions
        let gaps = self.question_gravity.analyze_gaps(context);
        let question_preds = self.question_gravity.predict_questions(&gaps, context.depth.max(1));
        predictions.extend(question_preds);

        // 2. Belief Revision predictions (if we have reservoir state)
        if let Some(ref state) = context.quanot_state {
            self.belief_revision.record_state(
                state.clone(),
                context.depth,
                &context.current_topic,
                context.consciousness_proxy,
                context.creativity_phase,
            );

            let belief_preds = self.belief_revision.project_conclusion(3);
            predictions.extend(belief_preds);
        }

        // 3. Basin predictions
        // Add current topic as a node
        if !context.current_topic.is_empty() && context.current_topic != "general" {
            self.basin.add_node(
                &context.current_topic,
                PropertyValue::String(context.current_topic.clone()),
                0.7,
            );
        }

        // Add discussed entities
        for entity in &context.discussed_entities {
            self.basin.add_node(
                entity,
                PropertyValue::String(entity.clone()),
                0.5,
            );
        }

        let basin_preds = self.basin.predict_equilibrium();
        predictions.extend(basin_preds);

        // 4. Apply meta-prediction calibration to all predictions
        for pred in &mut predictions {
            let calibrated = self.meta.calibrate(pred);
            pred.confidence = calibrated;
        }

        // 5. Add to history. Preserve the pre-existing history behavior.
        for pred in &predictions {
            self.add_to_history(pred.clone());
        }

        // Sort by confidence
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // ΩG2-S0 observes the already-final ranking. Its return value is ignored,
        // and no value from the observer can flow back into prediction behavior.
        #[cfg(feature = "omega-g2-shadow")]
        {
            let _ = self.omega_g2_shadow.observe_prediction_batch(&predictions);
        }

        predictions
    }

    /// Add a prediction to history
    fn add_to_history(&mut self, prediction: Prediction) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(prediction);
    }

    /// Query predictions by filter
    pub fn query(&self, filter: PredictionFilter) -> Vec<&Prediction> {
        self.history.iter().filter(|p| filter.matches(p)).collect()
    }

    /// Update prediction statuses based on new evidence
    pub fn update_with_evidence(&mut self, evidence: &Evidence) {
        for pred in &mut self.history {
            if pred.id == evidence.prediction_id {
                // Copy the independently supplied outcome into the inert shadow
                // ledger before applying the unchanged live status transition.
                #[cfg(feature = "omega-g2-shadow")]
                {
                    let _ = self
                        .omega_g2_shadow
                        .resolve_prediction_evidence(pred, evidence);
                }

                match evidence.outcome {
                    PredictionOutcome::Confirmed => pred.mark_confirmed(),
                    PredictionOutcome::Refuted => pred.mark_refuted(),
                    PredictionOutcome::Surprised => pred.mark_surprised(),
                    PredictionOutcome::Uncertain => pred.mark_uncertain(),
                }

                // Update meta-prediction calibration
                let correct = matches!(
                    evidence.outcome,
                    PredictionOutcome::Confirmed | PredictionOutcome::Surprised
                );
                self.meta.update_engine_accuracy(pred.engine, correct);
                self.meta.update_kind_accuracy(pred.kind, correct);
                self.meta
                    .record_outcome_with_confidence(pred.confidence, evidence.outcome);
            }
        }
    }

    /// Read-only ΩG2-S0 diagnostics. This method exists only in opt-in builds.
    #[cfg(feature = "omega-g2-shadow")]
    #[must_use]
    pub fn omega_g2_shadow_snapshot(&self) -> OmegaG2ShadowSnapshot {
        self.omega_g2_shadow.snapshot()
    }

    /// Get the top N most confident pending predictions
    pub fn top_predictions(&self, n: usize) -> Vec<&Prediction> {
        let mut pending: Vec<&Prediction> = self
            .history
            .iter()
            .filter(|p| p.status == PredictionStatus::Pending && !p.is_expired())
            .collect();

        pending.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pending.into_iter().take(n).collect()
    }

    /// Get prediction accuracy statistics per engine
    pub fn engine_accuracy(&self) -> Vec<EngineAccuracy> {
        let mut accuracies = Vec::new();

        for engine in [
            PredictionEngine::QuestionGravity,
            PredictionEngine::BeliefRevision,
            PredictionEngine::Basin,
            PredictionEngine::Meta,
        ] {
            let mut acc = EngineAccuracy::new(engine);
            for pred in &self.history {
                if pred.engine == engine {
                    acc.update(pred.status);
                }
            }
            accuracies.push(acc);
        }

        accuracies
    }

    /// Project a "what if" counterfactual question
    pub fn project_counterfactual(&mut self, assumption: &str) -> CounterfactualResult {
        self.counterfactual
            .project_counterfactual(assumption, &self.basin)
    }

    /// Get current state for display/debugging
    pub fn summary(&self) -> PredictionCenterSummary {
        PredictionCenterSummary {
            total_predictions: self.history.len(),
            pending_count: self
                .history
                .iter()
                .filter(|p| p.status == PredictionStatus::Pending)
                .count(),
            confirmed_count: self
                .history
                .iter()
                .filter(|p| p.status == PredictionStatus::Confirmed)
                .count(),
            refuted_count: self
                .history
                .iter()
                .filter(|p| p.status == PredictionStatus::Refuted)
                .count(),
            question_gravity_gaps: self.question_gravity.current_gaps().len(),
            belief_revision_trajectory: self.belief_revision.trajectory_length(),
            basin_nodes: self.basin.node_count(),
            basin_constraints: self.basin.constraint_count(),
            overall_accuracy: self.meta.overall_accuracy(),
            engine_weights: self.meta.engine_weights(),
        }
    }

    /// Record an exchange - update all engines with new context
    pub fn record_exchange(&mut self, context: &ConversationContext) {
        // Update question gravity
        self.question_gravity
            .note_topic(&context.current_topic, context.depth);

        // Update belief revision with reservoir state
        if let Some(ref state) = context.quanot_state {
            self.belief_revision.record_state(
                state.clone(),
                context.depth,
                &context.current_topic,
                context.consciousness_proxy,
                context.creativity_phase,
            );
        }

        // Update basin with new nodes
        if !context.current_topic.is_empty() && context.current_topic != "general" {
            self.basin.add_node(
                &context.current_topic,
                PropertyValue::String(context.current_topic.clone()),
                0.7,
            );
        }
    }

    /// Get recent predictions for display
    pub fn recent_predictions(&self, n: usize) -> Vec<String> {
        self.history
            .iter()
            .rev()
            .take(n)
            .map(|p| format!("[{}] {} (conf: {:.2})", p.engine, p.description, p.confidence))
            .collect()
    }
}

impl Default for PredictionCenter {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of the prediction center state
#[derive(Debug)]
pub struct PredictionCenterSummary {
    pub total_predictions: usize,
    pub pending_count: usize,
    pub confirmed_count: usize,
    pub refuted_count: usize,
    pub question_gravity_gaps: usize,
    pub belief_revision_trajectory: usize,
    pub basin_nodes: usize,
    pub basin_constraints: usize,
    pub overall_accuracy: f64,
    pub engine_weights: std::collections::HashMap<PredictionEngine, f64>,
}

impl fmt::Display for PredictionCenterSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Prediction Center Summary:")?;
        writeln!(f, "  Total predictions: {}", self.total_predictions)?;
        writeln!(
            f,
            "  Pending: {}, Confirmed: {}, Refuted: {}",
            self.pending_count, self.confirmed_count, self.refuted_count
        )?;
        writeln!(f, "  Question Gravity gaps: {}", self.question_gravity_gaps)?;
        writeln!(
            f,
            "  Belief Revision trajectory: {}",
            self.belief_revision_trajectory
        )?;
        writeln!(
            f,
            "  Basin nodes: {}, constraints: {}",
            self.basin_nodes, self.basin_constraints
        )?;
        writeln!(f, "  Overall accuracy: {:.1}%", self.overall_accuracy * 100.0)?;

        writeln!(f, "  Engine weights:")?;
        for (engine, weight) in &self.engine_weights {
            writeln!(f, "    {}: {:.2}", engine, weight)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_prediction_center() {
        let center = PredictionCenter::new();
        let summary = center.summary();

        assert_eq!(summary.total_predictions, 0);
        assert_eq!(summary.pending_count, 0);
    }

    #[test]
    fn test_generate_predictions() {
        let mut center = PredictionCenter::new();

        // Provide context with discussed entities so gaps can be found
        let mut context = ConversationContext::new(
            "consciousness".to_string(),
            1,
            Some(vec![0.1, 0.2, 0.3]),
            Some(0.5),
        );
        context.discussed_entities = vec!["AI".to_string(), "learning".to_string()];

        let _predictions = center.generate(&context);

        // Should have generated some predictions (depends on context having entities)
        // Just verify it doesn't panic
    }

    #[test]
    fn test_query_pending() {
        let mut center = PredictionCenter::new();

        // Provide context with discussed entities
        let mut context = ConversationContext::new(
            "test".to_string(),
            0,
            Some(vec![0.1]),
            Some(0.5),
        );
        context.discussed_entities = vec!["test".to_string()];

        center.generate(&context);

        let filter = PredictionFilter {
            status: Some(PredictionStatus::Pending),
            ..Default::default()
        };

        let _pending = center.query(filter);
        // May be empty depending on implementation
    }

    #[test]
    fn test_top_predictions() {
        let mut center = PredictionCenter::new();

        let context = ConversationContext::new("test".to_string(), 1, None, None);

        center.generate(&context);

        let top = center.top_predictions(3);
        assert!(top.len() <= 3);
    }

    #[test]
    fn test_engine_accuracy() {
        let mut center = PredictionCenter::new();

        let context = ConversationContext::default();
        center.generate(&context);

        let accuracies = center.engine_accuracy();

        // Should have accuracy for all 4 engines
        assert_eq!(accuracies.len(), 4);
    }

    #[test]
    fn test_prediction_center_summary() {
        let center = PredictionCenter::new();
        let summary = center.summary();

        assert_eq!(summary.basin_nodes, 0);
        assert_eq!(summary.basin_constraints, 0);
    }

    #[test]
    fn test_record_exchange() {
        let mut center = PredictionCenter::new();

        let context = ConversationContext::new(
            "test topic".to_string(),
            1,
            Some(vec![0.1, 0.2, 0.3]),
            Some(0.5),
        );

        center.record_exchange(&context);

        let summary = center.summary();
        assert_eq!(summary.belief_revision_trajectory, 1);
    }

    #[test]
    fn test_counterfactual_projection() {
        let mut center = PredictionCenter::new();

        let result = center.project_counterfactual("fire causes heat");

        assert!(!result.assumption.is_empty());
    }

    #[cfg(feature = "omega-g2-shadow")]
    #[test]
    fn omega_shadow_is_observational_only() {
        let mut center = PredictionCenter::new();
        let mut context = ConversationContext::new(
            "shadow test".to_string(),
            1,
            Some(vec![0.1, 0.2, 0.3]),
            Some(0.5),
        );
        context.discussed_entities = vec!["alpha".to_string(), "beta".to_string()];
        let predictions = center.generate(&context);
        let snapshot = center.omega_g2_shadow_snapshot();
        assert_eq!(snapshot.pending_traces, predictions.len());
        assert!(!snapshot.authority.prediction_generation_influence);
        assert!(!snapshot.authority.prediction_ranking_influence);
        assert!(!snapshot.authority.prediction_outcome_influence);
    }
}
