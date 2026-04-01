//! Impulse — the Aion analogue of Temporal signals.

use serde::{Deserialize, Serialize};

/// An external or internal event that wakes a sleeping Mind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impulse {
    /// Direct message — e.g. from Telegram, user input
    Message(String),

    /// Internal timer fired
    Timer(TimerData),

    /// Child Fragment completed
    ChildComplete(ChildData),

    /// Arbitrary external signal
    External(ExternalData),

    /// Priority interrupt — preempts current thinking
    Priority(PriorityData),

    /// Query request — read state without modifying
    Query(QueryData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerData {
    pub id: String,
    pub fired_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildData {
    pub fragment_id: String,
    pub outcome: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalData {
    pub source: String,
    pub kind: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityData {
    pub reason: String,
    pub payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryData {
    pub query_id: String,
    pub query_type: String,
    pub payload: Option<String>,
}

impl Impulse {
    /// Human-readable description.
    pub fn describe(&self) -> String {
        match self {
            Impulse::Message(t) => format!("Message: \"{}\"", if t.len() > 40 { format!("{}...", &t[..40]) } else { t.clone() }),
            Impulse::Timer(t) => format!("Timer: {}", t.id),
            Impulse::ChildComplete(c) => format!("ChildComplete: {} -> {}", c.fragment_id, c.outcome),
            Impulse::External(e) => format!("External: {} from {}", e.kind, e.source),
            Impulse::Priority(p) => format!("Priority: {}", p.reason),
            Impulse::Query(q) => format!("Query: {} [{}]", q.query_type, q.query_id),
        }
    }

    pub fn is_wakeful(&self) -> bool {
        matches!(self, Impulse::Message(_) | Impulse::ChildComplete(_) | Impulse::External(_) | Impulse::Priority(_))
    }

    pub fn is_interrupt(&self) -> bool {
        matches!(self, Impulse::Priority(_) | Impulse::Query(_))
    }
}

impl Impulse {
    pub fn message<S: Into<String>>(text: S) -> Self { Impulse::Message(text.into()) }
    pub fn timer<S: Into<String>>(id: S) -> Self {
        Impulse::Timer(TimerData { id: id.into(), fired_at: chrono::Utc::now().timestamp() })
    }
    pub fn child_complete<S: Into<String>>(fragment_id: S, outcome: &str) -> Self {
        Impulse::ChildComplete(ChildData { fragment_id: fragment_id.into(), outcome: outcome.to_string(), result: None })
    }
    pub fn external<S: Into<String>>(source: S, kind: S, payload: S) -> Self {
        Impulse::External(ExternalData { source: source.into(), kind: kind.into(), payload: payload.into() })
    }
    pub fn priority<S: Into<String>>(reason: S) -> Self {
        Impulse::Priority(PriorityData { reason: reason.into(), payload: None })
    }
}
