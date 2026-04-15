//! CEF — Causal Event Fabric (Layer 1)
//!
//! The foundation of TCMW-A. Records *why* actions happened, not just *what* happened.
//! Each event carries:
//! - A causal parent pointer (what triggered it)
//! - An outcome tag (success / partial / failure / abandoned)
//! - A half-life weighted confidence that decays over time
//!
//! Emits events to the BGE on each record.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Partial,
    Failed,
    Abandoned,
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Outcome::Success => "success",
            Outcome::Partial => "partial",
            Outcome::Failed => "failed",
            Outcome::Abandoned => "abandoned",
        }
    }
}

/// A causally-labeled event in the TCMW-A system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    pub id: EventId,
    pub parent: Option<EventId>,
    pub action: String,
    pub outcome: Outcome,
    /// Half-life weighted confidence. Decays via: weight₀ × 0.5^(t / half_life)
    pub weight: f64,
    pub timestamp: i64,
    pub archetype_id: Option<super::bge::ArchetypeId>,
}

impl CausalEvent {
    /// Compute decayed weight at time `t`
    pub fn decayed_weight(&self, t: i64, half_life_secs: i64) -> f64 {
        if half_life_secs <= 0 {
            return self.weight;
        }
        let elapsed = (t - self.timestamp) as f64;
        let half_life = half_life_secs as f64;
        self.weight * 0.5_f64.powf(elapsed / half_life)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(u64);

impl EventId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// CEF — records events with causal parent pointers and half-life decay
#[derive(Debug, Clone)]
pub struct CEF {
    events: Vec<CausalEvent>,
    half_life_secs: i64,
    event_index: std::collections::HashMap<String, Vec<EventId>>,
}

impl Default for CEF {
    fn default() -> Self {
        Self::new(3600)
    }
}

impl CEF {
    pub fn new(half_life_secs: i64) -> Self {
        Self {
            events: Vec::new(),
            half_life_secs,
            event_index: std::collections::HashMap::new(),
        }
    }

    /// Record a new causal event
    pub fn record(&mut self, action: &str, parent_action: Option<&str>, outcome: Outcome) -> CausalEvent {
        let id = EventId::new();
        let parent = parent_action.and_then(|pa| {
            self.event_index
                .get(pa)
                .and_then(|ids| ids.last())
                .copied()
        });

        let event = CausalEvent {
            id,
            parent,
            action: action.to_string(),
            outcome,
            weight: 1.0,
            timestamp: crate::now_timestamp(),
            archetype_id: None,
        };

        // Index by action name for parent lookup
        self.event_index
            .entry(action.to_string())
            .or_default()
            .push(id);

        self.events.push(event.clone());
        event
    }

    /// Get the causal parent of an event, if any
    pub fn causal_parent(&self, event_id: &EventId) -> Option<&CausalEvent> {
        let event = self.events.iter().find(|e| e.effective_id() == *event_id)?;
        event.parent.and_then(|pid| self.events.iter().find(|e| e.id == pid))
    }

    /// Get all events in the last `window_secs` with decayed weights
    pub fn recent_events(&self, window_secs: i64) -> Vec<(CausalEvent, f64)> {
        let now = crate::now_timestamp();
        let cutoff = now - window_secs;
        self.events
            .iter()
            .filter(|e| e.timestamp >= cutoff)
            .map(|e| {
                let decayed = e.decayed_weight(now, self.half_life_secs);
                (e.clone(), decayed)
            })
            .collect()
    }

    /// Get cumulative CEF weight for an action pattern
    pub fn causal_weight(&self, action_prefix: &str) -> f64 {
        let now = crate::now_timestamp();
        self.events
            .iter()
            .filter(|e| e.action.starts_with(action_prefix))
            .map(|e| e.decayed_weight(now, self.half_life_secs))
            .sum()
    }

    /// Total events recorded
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl CausalEvent {
    /// Alias for EventId accessor for compatibility
    pub fn effective_id(&self) -> EventId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_event() {
        let mut cef = CEF::new(3600);
        let e = cef.record("open VSCode", None, Outcome::Success);
        assert_eq!(cef.len(), 1);
        assert!(e.parent.is_none());
    }

    #[test]
    fn test_casual_parent_chain() {
        let mut cef = CEF::new(3600);
        let e1 = cef.record("open VSCode", None, Outcome::Success);
        let e2 = cef.record("write code", Some("open VSCode"), Outcome::Success);

        assert!(e2.parent.is_some());
        assert_eq!(e2.parent.map(|p| p), Some(e1.id));
    }

    #[test]
    fn test_half_life_decay() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut cef = CEF::new(10); // 10 second half-life for test

        let mut e = cef.record("test action", None, Outcome::Success);
        // Pretend 10 seconds have passed
        e.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 - 10;

        let decayed = e.decayed_weight(crate::now_timestamp(), 10);
        assert!(decayed < 1.0);
        assert!(decayed > 0.4); // ~0.5 with some tolerance
    }

    #[test]
    fn test_outcome_as_str() {
        assert_eq!(Outcome::Success.as_str(), "success");
        assert_eq!(Outcome::Failed.as_str(), "failed");
    }

    #[test]
    fn test_event_id_unique() {
        let mut cef = CEF::default();
        let ids: Vec<_> = (0..100)
            .map(|_| {
                let e = cef.record("action", None, Outcome::Success);
                e.id
            })
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 100);
    }

    #[test]
    fn test_recent_events() {
        let mut cef = CEF::new(3600);
        cef.record("action1", None, Outcome::Success);
        cef.record("action2", None, Outcome::Success);
        let recent = cef.recent_events(3600);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_casual_weight() {
        let mut cef = CEF::new(3600);
        cef.record("open VSCode", None, Outcome::Success);
        cef.record("open terminal", None, Outcome::Success);
        cef.record("write code", None, Outcome::Success);

        let weight = cef.causal_weight("open ");
        assert!(weight > 0.0);
    }
}
