//! Session — Persistence Layer

/// A conversation session.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Option<i64>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub summary: Option<String>,
}
