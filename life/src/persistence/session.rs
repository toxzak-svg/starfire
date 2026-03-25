//! Session Management
//!
//! Tracks conversations. A session is a unit of interaction —
//! a single conversation from start to end.

use crate::persistence::store::Session as SessionRecord;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A session — a single conversation with Zachary.
/// 
/// Sessions are the unit of continuity. When Star has a conversation,
/// it happens within a session. The session tracks what's discussed,
/// what Star learns, and how the conversation evolves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Database ID
    pub id: i64,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session ended (None = still active)
    pub ended_at: Option<DateTime<Utc>>,
    /// Brief summary of what was discussed
    pub summary: Option<String>,
    /// What was talked about in this session
    pub topics: Vec<String>,
    /// What Star learned in this session
    pub learnings: Vec<String>,
    /// Key beliefs that formed or changed during this session
    pub belief_changes: Vec<BeliefChange>,
    /// How the session ended
    pub end_reason: Option<EndReason>,
}

impl Session {
    /// Create a new session (in-memory, before persisting).
    pub fn new(id: i64) -> Self {
        Self {
            id,
            started_at: Utc::now(),
            ended_at: None,
            summary: None,
            topics: Vec::new(),
            learnings: Vec::new(),
            belief_changes: Vec::new(),
            end_reason: None,
        }
    }

    /// Record a topic that came up.
    pub fn add_topic(&mut self, topic: impl Into<String>) {
        let topic = topic.into();
        if !self.topics.contains(&topic) {
            self.topics.push(topic);
        }
    }

    /// Record something Star learned.
    pub fn add_learning(&mut self, learning: impl Into<String>) {
        self.learnings.push(learning.into());
    }

    /// Record a belief that was formed or changed.
    pub fn add_belief_change(&mut self, change: BeliefChange) {
        self.belief_changes.push(change);
    }

    /// Mark the session as ended.
    pub fn end(&mut self, reason: EndReason, summary: impl Into<String>) {
        self.ended_at = Some(Utc::now());
        self.end_reason = Some(reason);
        self.summary = Some(summary.into());
    }

    /// Is the session still active?
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    /// How long did the session last?
    pub fn duration_minutes(&self) -> Option<i64> {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        let duration = end - self.started_at;
        Some(duration.num_minutes())
    }

    /// Generate a summary of the session for memory consolidation.
    pub fn consolidation_summary(&self) -> String {
        let mut parts = Vec::new();
        
        if !self.topics.is_empty() {
            parts.push(format!("Topics: {}", self.topics.join(", ")));
        }
        if !self.learnings.is_empty() {
            parts.push(format!("Learned: {}", self.learnings.join("; ")));
        }
        if !self.belief_changes.is_empty() {
            let changes: Vec<String> = self.belief_changes
                .iter()
                .map(|c| c.description())
                .collect();
            parts.push(format!("Beliefs: {}", changes.join("; ")));
        }
        
        parts.join(" | ")
    }

    /// Convert from a stored session record.
    pub fn from_record(record: SessionRecord) -> Self {
        Self {
            id: record.id,
            started_at: DateTime::from_timestamp(record.started_at, 0)
                .unwrap_or_else(Utc::now),
            ended_at: record.ended_at.and_then(|t| DateTime::from_timestamp(t, 0)),
            summary: record.summary,
            topics: Vec::new(),
            learnings: Vec::new(),
            belief_changes: Vec::new(),
            end_reason: None,
        }
    }
}

/// A belief that was formed or changed during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefChange {
    /// What the belief was before
    pub before: Option<String>,
    /// What it became
    pub after: String,
    /// Why it changed
    pub reason: String,
    /// When
    pub at: DateTime<Utc>,
}

impl BeliefChange {
    pub fn new(before: Option<&str>, after: &str, reason: &str) -> Self {
        Self {
            before: before.map(String::from),
            after: after.to_string(),
            reason: reason.to_string(),
            at: Utc::now(),
        }
    }

    pub fn formed(belief: &str, reason: &str) -> Self {
        Self::new(None, belief, reason)
    }

    pub fn revised(before: &str, after: &str, reason: &str) -> Self {
        Self::new(Some(before), after, reason)
    }

    pub fn description(&self) -> String {
        match &self.before {
            Some(before) => format!("{} → {} ({})", before, self.after, self.reason),
            None => format!("New belief: {} ({})", self.after, self.reason),
        }
    }
}

/// Why a session ended.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndReason {
    /// User ended the conversation
    UserEnded,
    /// Session timed out due to inactivity
    Timeout,
    /// Session was explicitly paused (can resume)
    Paused,
    /// An error occurred
    Error,
}

impl EndReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserEnded => "User ended the conversation",
            Self::Timeout => "Session timed out",
            Self::Paused => "Session paused",
            Self::Error => "Session ended due to error",
        }
    }
}
