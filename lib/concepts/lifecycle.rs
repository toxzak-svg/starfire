//! Lifecycle stages for concepts

use serde::{Deserialize, Serialize};

/// Lifecycle stage of a concept
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Newly introduced, not yet tested
    Birth,
    /// Heavily revised, lots of pain and contradictions
    Adolescence,
    /// Stable usage, frequent successful deployment
    Maturity,
    /// Rarely used, often out-of-date or misleading
    Senescence,
    /// Retired, replaced by descendants
    Death,
}

impl LifecycleStage {
    /// Determine stage from usage metrics
    pub fn from_metrics(
        usage_count: usize,
        pain_count: usize,
        contradiction_count: usize,
        last_used: i64,
    ) -> Self {
        let now = crate::now_timestamp();
        let idle_time = now - last_used;

        if usage_count == 0 {
            return LifecycleStage::Birth;
        }

        if contradiction_count > 2 || pain_count > 5 {
            return LifecycleStage::Adolescence;
        }

        if usage_count > 10 && pain_count < 2 && contradiction_count < 2 {
            if idle_time > 86400 * 30 {
                return LifecycleStage::Senescence;
            }
            return LifecycleStage::Maturity;
        }

        if idle_time > 86400 * 60 && usage_count < 5 {
            return LifecycleStage::Senescence;
        }

        if idle_time > 86400 * 180 {
            return LifecycleStage::Death;
        }

        LifecycleStage::Adolescence
    }

    pub fn label(&self) -> &'static str {
        match self {
            LifecycleStage::Birth => "birth",
            LifecycleStage::Adolescence => "adolescence",
            LifecycleStage::Maturity => "maturity",
            LifecycleStage::Senescence => "senescence",
            LifecycleStage::Death => "death",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LifecycleStage::Birth => "newly introduced",
            LifecycleStage::Adolescence => "under revision",
            LifecycleStage::Maturity => "stable and reliable",
            LifecycleStage::Senescence => "declining usage",
            LifecycleStage::Death => "retired",
        }
    }
}

/// An event in a concept's lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub concept_id: super::concept::ConceptId,
    pub from: LifecycleStage,
    pub to: LifecycleStage,
    pub reason: String,
    pub at: i64,
}

impl LifecycleEvent {
    pub fn new(
        concept_id: super::concept::ConceptId,
        from: LifecycleStage,
        to: LifecycleStage,
        reason: &str,
    ) -> Self {
        Self {
            concept_id,
            from,
            to,
            reason: reason.to_string(),
            at: crate::now_timestamp(),
        }
    }
}