//! Thought — a discrete unit of AI thinking.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of thought — what kind of processing happened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThoughtKind {
    Reason, Remember, Wonder, Reflect, Research, Converse, Plan, Execute, Introspect, Custom(String),
}
impl std::fmt::Display for ThoughtKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThoughtKind::Reason => write!(f, "reason"),
            ThoughtKind::Remember => write!(f, "remember"),
            ThoughtKind::Wonder => write!(f, "wonder"),
            ThoughtKind::Reflect => write!(f, "reflect"),
            ThoughtKind::Research => write!(f, "research"),
            ThoughtKind::Converse => write!(f, "converse"),
            ThoughtKind::Plan => write!(f, "plan"),
            ThoughtKind::Execute => write!(f, "execute"),
            ThoughtKind::Introspect => write!(f, "introspect"),
            ThoughtKind::Custom(s) => write!(f, "{}", s),
        }
    }
}
impl From<&str> for ThoughtKind {
    fn from(s: &str) -> Self {
        match s {
            "reason" => ThoughtKind::Reason,
            "remember" => ThoughtKind::Remember,
            "wonder" => ThoughtKind::Wonder,
            "reflect" => ThoughtKind::Reflect,
            "research" => ThoughtKind::Research,
            "converse" => ThoughtKind::Converse,
            "plan" => ThoughtKind::Plan,
            "execute" => ThoughtKind::Execute,
            "introspect" => ThoughtKind::Introspect,
            other => ThoughtKind::Custom(other.to_string()),
        }
    }
}

/// Outcome of a completed thought.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThoughtOutcome {
    Complete { result: serde_json::Value },
    Failed { error: String },
    Interrupted,
    Cancelled,
}
impl ThoughtOutcome {
    pub fn is_success(&self) -> bool { matches!(self, ThoughtOutcome::Complete { .. }) }
    pub fn result(&self) -> Option<&serde_json::Value> {
        match self { ThoughtOutcome::Complete { result } => Some(result), _ => None }
    }
}

/// A record of a completed thought.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub id: Uuid,
    pub mind_id: Uuid,
    pub kind: ThoughtKind,
    pub input: Option<String>,
    pub outcome: ThoughtOutcome,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub retry_count: u32,
}
impl Thought {
    pub fn new(mind_id: Uuid, kind: ThoughtKind) -> Self {
        Self { id: Uuid::new_v4(), mind_id, kind, input: None,
            outcome: ThoughtOutcome::Complete { result: serde_json::json!({}) },
            started_at: chrono::Utc::now().timestamp_millis(), completed_at: None,
            duration_ms: None, retry_count: 0 }
    }
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = Some(serde_json::to_string(&input).unwrap_or_default()); self
    }
    pub fn complete(mut self, result: serde_json::Value) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        self.completed_at = Some(now); self.duration_ms = Some(now - self.started_at);
        self.outcome = ThoughtOutcome::Complete { result }; self
    }
    pub fn fail(mut self, error: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        self.completed_at = Some(now); self.duration_ms = Some(now - self.started_at);
        self.outcome = ThoughtOutcome::Failed { error }; self
    }
}
