//! PSE — Proactive Staging Engine (Layer 4)
//!
//! Takes AIH's top predictions and pre-stages resources proportional to confidence.
//! All staging is reversible and zero-side-effect until the user confirms
//! or the causal event fires.
//!
//! "Predict-then-prepare" — aligns with the DILLO paradigm of semantic foresight
//! without expensive full-state simulation (~14× speedup).

use serde::{Deserialize, Serialize};
use super::aih::IntentPrediction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagedStatus {
    Pending,
    Confirmed,
    Cancelled,
    Expired,
}

impl StagedStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StagedStatus::Pending => "pending",
            StagedStatus::Confirmed => "confirmed",
            StagedStatus::Cancelled => "cancelled",
            StagedStatus::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StagedActionType {
    /// Pre-fetch file contents or repo state
    PreFetch { path: String },
    /// Pre-load model weights or cache embeddings
    PreLoad { resource: String },
    /// Draft a suggested next action in natural language
    Draft { text: String },
}

impl StagedActionType {
    pub fn is_reversible(&self) -> bool {
        // All PSE actions are designed to be zero-side-effect until confirmed
        true
    }

    pub fn description(&self) -> String {
        match self {
            StagedActionType::PreFetch { path } => format!("Pre-fetch: {}", path),
            StagedActionType::PreLoad { resource } => format!("Pre-load: {}", resource),
            StagedActionType::Draft { text } => format!("Draft: {}", text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StagedId(u64);

impl StagedId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for StagedId {
    fn default() -> Self {
        Self::new()
    }
}

/// A reversible pre-staged action awaiting user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedAction {
    pub id: StagedId,
    pub prediction: IntentPrediction,
    pub action_type: StagedActionType,
    /// Whether this action can be safely undone
    pub reversible: bool,
    pub status: StagedStatus,
    pub created_at: i64,
    /// Confidence at time of staging (for comparison)
    pub staged_probability: f64,
}

impl StagedAction {
    pub fn new(prediction: IntentPrediction, action_type: StagedActionType) -> Self {
        Self {
            id: StagedId::new(),
            prediction: prediction.clone(),
            action_type,
            reversible: true,
            status: StagedStatus::Pending,
            created_at: crate::now_timestamp(),
            staged_probability: prediction.probability,
        }
    }

    /// Check if this staged action has expired
    pub fn is_expired(&self, max_age_secs: i64) -> bool {
        crate::now_timestamp() - self.created_at >= max_age_secs
    }

    /// Confirm — user approved or causal event fired
    pub fn confirm(&mut self) {
        self.status = StagedStatus::Confirmed;
    }

    /// Cancel — user rejected or prediction invalidated
    pub fn cancel(&mut self) {
        self.status = StagedStatus::Cancelled;
    }

    /// Expire — timed out without action
    pub fn expire(&mut self) {
        self.status = StagedStatus::Expired;
    }
}

/// PSE — Proactive Staging Engine
#[derive(Debug, Clone)]
pub struct PSE {
    /// Minimum probability to stage an action
    threshold: f64,
    /// Max age for staged actions before expiry
    max_age_secs: i64,
    /// Pre-staged actions
    staged: Vec<StagedAction>,
    /// Cache of pre-fetched content
    prefetch_cache: std::collections::HashMap<String, String>,
    /// Cache of pre-loaded resources
    preload_cache: std::collections::HashMap<String, Vec<u8>>,
}

impl Default for PSE {
    fn default() -> Self {
        Self::new(0.5)
    }
}

impl PSE {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            max_age_secs: 300, // 5 minutes default
            staged: Vec::new(),
            prefetch_cache: std::collections::HashMap::new(),
            preload_cache: std::collections::HashMap::new(),
        }
    }

    /// Stage a high-confidence prediction
    pub fn stage(&mut self, prediction: IntentPrediction, action_type: StagedActionType) -> StagedAction {
        let staged = StagedAction::new(prediction.clone(), action_type);
        self.staged.push(staged.clone());

        // Execute the pre-stage (zero-side-effect until confirmed)
        match &staged.action_type {
            StagedActionType::PreFetch { path } => {
                self.do_prefetch(path);
            }
            StagedActionType::PreLoad { resource } => {
                self.do_preload(resource);
            }
            StagedActionType::Draft { .. } => {
                // Drafts are text — no pre-execution needed
            }
        }

        staged
    }

    /// Stage from prediction's natural action type
    pub fn stage_from_prediction(&mut self, prediction: &IntentPrediction) -> Option<StagedAction> {
        if prediction.probability < self.threshold {
            return None;
        }

        let action_type = self.classify_action(&prediction.action);
        Some(self.stage(prediction.clone(), action_type))
    }

    /// Classify an action string into a StagedActionType
    fn classify_action(&self, action: &str) -> StagedActionType {
        let lower = action.to_lowercase();

        if lower.contains("open") || lower.contains("read") || lower.contains("file") {
            StagedActionType::PreFetch {
                path: self.infer_path(action),
            }
        } else if lower.contains("load") || lower.contains("model") || lower.contains("weight") {
            StagedActionType::PreLoad {
                resource: action.to_string(),
            }
        } else {
            StagedActionType::Draft {
                text: format!("Suggestion: {}", action),
            }
        }
    }

    fn infer_path(&self, action: &str) -> String {
        // Try to extract a path from the action string
        let words: Vec<_> = action.split_whitespace().collect();
        for word in &words {
            if word.contains('/') || word.contains('\\') || word.ends_with(".rs") || word.ends_with(".py") {
                return word.to_string();
            }
        }
        action.to_string()
    }

    /// Pre-fetch a file (placeholder — actual I/O would be in Runtime)
    fn do_prefetch(&mut self, path: &str) {
        // In the real implementation, this would read the file into prefetch_cache.
        // For now, we just track the intent.
        self.prefetch_cache.insert(path.to_string(), format!("[prefetched: {}]", path));
    }

    /// Pre-load a resource (placeholder)
    fn do_preload(&mut self, resource: &str) {
        self.preload_cache.insert(resource.to_string(), Vec::new());
    }

    /// Get pending staged actions sorted by probability
    pub fn pending(&self) -> Vec<&StagedAction> {
        let mut pending: Vec<_> = self.staged.iter()
            .filter(|s| s.status == StagedStatus::Pending && !s.is_expired(self.max_age_secs))
            .collect();
        pending.sort_by(|a, b| b.staged_probability.partial_cmp(&a.staged_probability).unwrap());
        pending
    }

    /// Confirm a staged action by ID
    pub fn confirm(&mut self, id: StagedId) -> bool {
        if let Some(s) = self.staged.iter_mut().find(|s| s.id == id) {
            s.confirm();
            return true;
        }
        false
    }

    /// Cancel a staged action by ID
    pub fn cancel(&mut self, id: StagedId) -> bool {
        if let Some(s) = self.staged.iter_mut().find(|s| s.id == id) {
            s.cancel();
            return true;
        }
        false
    }

    /// Expire old pending actions
    pub fn expire_stale(&mut self) {
        for s in &mut self.staged {
            if s.status == StagedStatus::Pending && s.is_expired(self.max_age_secs) {
                s.expire();
            }
        }
    }

    /// Get top suggestion for UI
    pub fn top_suggestion(&self) -> Option<String> {
        self.pending().first().map(|s| {
            match &s.action_type {
                StagedActionType::Draft { text } => text.clone(),
                other => other.description(),
            }
        })
    }

    /// Clear all cancelled/expired
    pub fn cleanup(&mut self) {
        self.staged.retain(|s| s.status == StagedStatus::Pending);
    }

    pub fn staged_count(&self) -> usize {
        self.staged.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prediction(prob: f64) -> IntentPrediction {
        IntentPrediction::new("test action", prob, 1)
    }

    #[test]
    fn test_threshold_gating() {
        let mut pse = PSE::new(0.7);

        let pred = make_prediction(0.9);
        let result = pse.stage_from_prediction(&pred);
        assert!(result.is_some());

        let pred_low = make_prediction(0.3);
        let result_low = pse.stage_from_prediction(&pred_low);
        assert!(result_low.is_none());
    }

    #[test]
    fn test_confirm_cancel() {
        let mut pse = PSE::new(0.5);

        let pred = make_prediction(0.9);
        let staged = pse.stage_from_prediction(&pred).unwrap();
        let id = staged.id;

        assert!(pse.confirm(id));
        // Confirm changes status to Confirmed; cancel() still finds the item
        // (vector isn't modified) and flips it to Cancelled. Both return true.
        assert!(pse.cancel(id));
    }

    #[test]
    fn test_classify_action() {
        let pse = PSE::default();

        let fetch = pse.classify_action("open file /path/to/file.rs");
        assert!(matches!(fetch, StagedActionType::PreFetch { .. }));

        let load = pse.classify_action("load model weights");
        assert!(matches!(load, StagedActionType::PreLoad { .. }));

        let draft = pse.classify_action("write code");
        assert!(matches!(draft, StagedActionType::Draft { .. }));
    }

    #[test]
    fn test_expire_stale() {
        let mut pse = PSE::new(0.5);
        pse.max_age_secs = 0; // instant expiry for testing

        let pred = make_prediction(0.9);
        pse.stage_from_prediction(&pred);

        pse.expire_stale();
        // expire_stale() sets status=Expired; pending() filters by status==Pending
        // so expired items are excluded → count is 0
        assert_eq!(pse.pending_count(), 0);
    }

    #[test]
    fn test_top_suggestion() {
        let mut pse = PSE::new(0.3);

        let pred1 = IntentPrediction::new("action1", 0.6, 1);
        let pred2 = IntentPrediction::new("action2", 0.9, 1);

        pse.stage_from_prediction(&pred1);
        pse.stage_from_prediction(&pred2);

        let top = pse.top_suggestion();
        assert!(top.is_some());
        // Higher probability should be first
        assert!(top.unwrap().contains("action2"));
    }

    #[test]
    fn test_staged_action_is_reversible() {
        let pred = make_prediction(0.8);
        let staged = StagedAction::new(
            pred,
            StagedActionType::PreFetch { path: "/test".to_string() },
        );
        assert!(staged.reversible);
    }

    #[test]
    fn test_cleanup() {
        let mut pse = PSE::new(0.5);
        let pred = make_prediction(0.9);
        let staged = pse.stage_from_prediction(&pred).unwrap();

        pse.cancel(staged.id);
        pse.cleanup();
        assert_eq!(pse.staged_count(), 0);
    }
}
