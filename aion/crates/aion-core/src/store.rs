//! Store — SQLite persistence layer using tokio::sync::Mutex + spawn_blocking.

use rusqlite::params;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::{AionError, AionResult, MindStatus, MindInfo, MindId, MindKind};
use crate::thought::{Thought, ThoughtKind};

pub struct Store { conn: Arc<Mutex<rusqlite::Connection>> }

impl Store {
    pub async fn new<P: AsRef<Path>>(path: P) -> AionResult<Self> {
        let conn = rusqlite::Connection::open(path.as_ref())
            .map_err(|e| AionError::Database(e.to_string()))?;
        let store = Store { conn: Arc::new(Mutex::new(conn)) };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> AionResult<()> {
        let conn = self.conn.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute_batch("CREATE TABLE IF NOT EXISTS mind_kinds (name TEXT PRIMARY KEY, description TEXT, version INTEGER NOT NULL DEFAULT 1, created_at INTEGER NOT NULL)")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS minds (id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT, status TEXT NOT NULL DEFAULT 'created', checkpoint TEXT, version INTEGER NOT NULL DEFAULT 1, channel TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, ended_at INTEGER)")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS thoughts (id TEXT PRIMARY KEY, mind_id TEXT NOT NULL, kind TEXT NOT NULL, input TEXT, outcome TEXT, started_at INTEGER NOT NULL, completed_at INTEGER, duration_ms INTEGER, retry_count INTEGER NOT NULL DEFAULT 0)")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS fragments (id TEXT PRIMARY KEY, parent_id TEXT NOT NULL, kind TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'running', input TEXT, result TEXT, created_at INTEGER NOT NULL, completed_at INTEGER)")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS channels (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL)")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS channel_subscriptions (channel_id TEXT NOT NULL, mind_id TEXT NOT NULL, PRIMARY KEY (channel_id, mind_id))")?;
            c.execute_batch("CREATE TABLE IF NOT EXISTS events (id INTEGER PRIMARY KEY AUTOINCREMENT, mind_id TEXT NOT NULL, kind TEXT NOT NULL, payload TEXT, timestamp INTEGER NOT NULL)")?;
            c.execute_batch("CREATE INDEX IF NOT EXISTS idx_minds_kind ON minds(kind)")?;
            c.execute_batch("CREATE INDEX IF NOT EXISTS idx_minds_status ON minds(status)")?;
            c.execute_batch("CREATE INDEX IF NOT EXISTS idx_thoughts_mind ON thoughts(mind_id)")?;
            c.execute_batch("CREATE INDEX IF NOT EXISTS idx_events_mind ON events(mind_id)")?;
            Ok::<(), AionError>(())
        }).await?;
        Ok(())
    }

    pub async fn register_kind(&self, name: &str) -> AionResult<()> {
        let conn = self.conn.clone();
        let name_owned = name.to_string();
        let now = chrono::Utc::now().timestamp();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "INSERT OR IGNORE INTO mind_kinds (name, created_at) VALUES (?1, ?2)",
                params![name_owned, now],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn list_kinds(&self) -> AionResult<Vec<MindKind>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare("SELECT name, description, version FROM mind_kinds ORDER BY name")?;
            let rows = stmt.query_map([], |row| {
                Ok(MindKind { name: row.get(0)?, description: row.get(1)?, version: row.get(2)? })
            })?;
            let mut kinds = Vec::new();
            for r in rows { kinds.push(r?); }
            Ok::<Vec<MindKind>, AionError>(kinds)
        }).await?
    }

    pub async fn create_mind(&self, id: Uuid, kind: &str, name: Option<&str>, checkpoint: &str, channel: Option<&str>) -> AionResult<MindId> {
        self.register_kind(kind).await?;
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let id_str = id.to_string();
        let kind_owned = kind.to_string();
        let name_owned = name.map(|s| s.to_string());
        let checkpoint_owned = checkpoint.to_string();
        let channel_owned = channel.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "INSERT INTO minds (id, kind, name, status, checkpoint, created_at, updated_at, channel) VALUES (?1, ?2, ?3, 'running', ?4, ?5, ?6, ?7)",
                params![id_str, kind_owned, name_owned, checkpoint_owned, now, now, channel_owned],
            )?;
            Ok::<(), AionError>(())
        }).await?;
        Ok(MindId::from_uuid(id))
    }

    pub async fn update_checkpoint(&self, id: Uuid, checkpoint: &str) -> AionResult<()> {
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let cp_owned = checkpoint.to_string();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "UPDATE minds SET checkpoint = ?1, updated_at = ?2, status = 'running' WHERE id = ?3",
                params![cp_owned, now, id_owned],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn update_status(&self, id: Uuid, status: MindStatus) -> AionResult<()> {
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "UPDATE minds SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![status.to_string(), now, id_owned],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn terminate_mind(&self, id: Uuid, checkpoint: &str) -> AionResult<()> {
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let cp_owned = checkpoint.to_string();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "UPDATE minds SET status = 'terminated', checkpoint = ?1, updated_at = ?2, ended_at = ?2 WHERE id = ?3",
                params![cp_owned, now, id_owned],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn get_mind_state(&self, id: Uuid) -> AionResult<(String, String)> {
        let conn = self.conn.clone();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare("SELECT kind, checkpoint FROM minds WHERE id = ?1")?;
            stmt.query_row([id_owned], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows => AionError::MindNotFound(id.to_string()), _ => AionError::Database(e.to_string()) })
        }).await?
    }

    pub async fn get_checkpoint(&self, id: Uuid) -> AionResult<Option<String>> {
        let conn = self.conn.clone();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            match c.query_row("SELECT checkpoint FROM minds WHERE id = ?1", [&id_owned], |row| row.get::<_, String>(0)) {
                Ok(s) => Ok(Some(s)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(AionError::Database(e.to_string())),
            }
        }).await?
    }

    pub async fn get_mind_status(&self, id: Uuid) -> AionResult<MindStatus> {
        let conn = self.conn.clone();
        let id_owned = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            match c.query_row("SELECT status FROM minds WHERE id = ?1", [&id_owned], |row| row.get::<_, String>(0)) {
                Ok(s) => match s.as_str() {
                    "created" => Ok(MindStatus::Created), "running" => Ok(MindStatus::Running),
                    "checkpointed" => Ok(MindStatus::Checkpointed), "terminated" => Ok(MindStatus::Terminated),
                    "failed" => Ok(MindStatus::Failed), _ => Err(AionError::InvalidState(s)),
                },
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(MindStatus::Unknown),
                Err(e) => Err(AionError::Database(e.to_string())),
            }
        }).await?
    }

    pub async fn list_minds(&self, _kind: Option<&str>, _status: Option<MindStatus>) -> AionResult<Vec<MindInfo>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare("SELECT id, kind, name, status, created_at, updated_at, ended_at FROM minds ORDER BY updated_at DESC")?;
            let rows = stmt.query_map([], |row| {
                let id_str: String = row.get(0)?;
                let status_str: String = row.get(3)?;
                let status = match status_str.as_str() {
                    "created" => MindStatus::Created, "running" => MindStatus::Running,
                    "checkpointed" => MindStatus::Checkpointed, "terminated" => MindStatus::Terminated,
                    "failed" => MindStatus::Failed, _ => MindStatus::Unknown,
                };
                Ok(MindInfo { id: MindId::from_uuid(Uuid::parse_str(&id_str).unwrap_or_default()), kind: row.get(1)?, name: row.get(2)?, status, created_at: row.get(4)?, updated_at: row.get(5)?, ended_at: row.get(6)? })
            })?;
            let mut minds = Vec::new();
            for r in rows { minds.push(r?); }
            Ok::<Vec<MindInfo>, AionError>(minds)
        }).await?
    }

    pub async fn delete_mind(&self, id: Uuid) -> AionResult<()> {
        let conn = self.conn.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute("DELETE FROM events WHERE mind_id = ?1", [&id_str])?;
            c.execute("DELETE FROM thoughts WHERE mind_id = ?1", [&id_str])?;
            c.execute("DELETE FROM fragments WHERE parent_id = ?1", [&id_str])?;
            c.execute("DELETE FROM channel_subscriptions WHERE mind_id = ?1", [&id_str])?;
            c.execute("DELETE FROM minds WHERE id = ?1", [&id_str])?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn record_thought(&self, thought: Thought) -> AionResult<()> {
        let conn = self.conn.clone();
        let outcome_str = match &thought.outcome {
            crate::thought::ThoughtOutcome::Complete { .. } => "complete",
            crate::thought::ThoughtOutcome::Failed { .. } => "failed",
            crate::thought::ThoughtOutcome::Interrupted => "interrupted",
            crate::thought::ThoughtOutcome::Cancelled => "cancelled",
        };
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "INSERT INTO thoughts (id, mind_id, kind, input, outcome, started_at, completed_at, duration_ms, retry_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![thought.id.to_string(), thought.mind_id.to_string(), thought.kind.to_string(), thought.input, outcome_str, thought.started_at, thought.completed_at, thought.duration_ms, thought.retry_count],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn get_thoughts(&self, mind_id: Uuid, limit: usize) -> AionResult<Vec<Thought>> {
        let conn = self.conn.clone();
        let mind_id_owned = mind_id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare(
                "SELECT id, mind_id, kind, input, outcome, started_at, completed_at, duration_ms, retry_count FROM thoughts WHERE mind_id = ?1 ORDER BY started_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![mind_id_owned, limit as i64], |row| {
                let outcome_str: String = row.get(4)?;
                let outcome = match outcome_str.as_str() {
                    "complete" => crate::thought::ThoughtOutcome::Complete { result: serde_json::json!({}) },
                    "failed" => crate::thought::ThoughtOutcome::Failed { error: "recorded".to_string() },
                    "interrupted" => crate::thought::ThoughtOutcome::Interrupted,
                    _ => crate::thought::ThoughtOutcome::Cancelled,
                };
                Ok(Thought {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                    mind_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                    kind: ThoughtKind::from(row.get::<_, String>(2)?.as_str()),
                    input: row.get(3)?,
                    outcome,
                    started_at: row.get(5)?,
                    completed_at: row.get(6)?,
                    duration_ms: row.get(7)?,
                    retry_count: row.get(8)?,
                })
            })?;
            let mut thoughts = Vec::new();
            for r in rows { thoughts.push(r?); }
            Ok::<Vec<Thought>, AionError>(thoughts)
        }).await?
    }

    pub async fn subscribe(&self, channel_name: &str, mind_id: Uuid) -> AionResult<()> {
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let ch_name = channel_name.to_string();
        let mind_id_owned = mind_id.to_string();
        tokio::task::spawn_blocking(move || {
            let channel_id = Uuid::new_v4().to_string();
            let c = conn.blocking_lock();
            c.execute(
                "INSERT OR IGNORE INTO channels (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![channel_id, ch_name, now],
            )?;
            let channel_id: String = c.query_row(
                "SELECT id FROM channels WHERE name = ?1", [&ch_name], |r| r.get(0)
            )?;
            c.execute(
                "INSERT OR IGNORE INTO channel_subscriptions (channel_id, mind_id) VALUES (?1, ?2)",
                params![channel_id, mind_id_owned],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn list_channels(&self) -> AionResult<Vec<(String, usize)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare(
                "SELECT c.name, COUNT(cs.mind_id) FROM channels c LEFT JOIN channel_subscriptions cs ON c.id = cs.channel_id GROUP BY c.name ORDER BY c.name"
            )?;
            let rows = stmt.query_map([], |row| {
                let name: String = row.get(0)?;
                let cnt: i64 = row.get(1)?;
                Ok((name, cnt as usize))
            })?;
            let mut channels = Vec::new();
            for r in rows { channels.push(r?); }
            Ok::<Vec<(String, usize)>, AionError>(channels)
        }).await?
    }

    pub async fn log_event(&self, mind_id: Uuid, kind: &str, payload: Option<&str>) -> AionResult<()> {
        let conn = self.conn.clone();
        let now = chrono::Utc::now().timestamp();
        let kind_owned = kind.to_string();
        let payload_owned = payload.map(|s| s.to_string());
        let mind_id_owned = mind_id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            c.execute(
                "INSERT INTO events (mind_id, kind, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![mind_id_owned, kind_owned, payload_owned, now],
            )?;
            Ok::<(), AionError>(())
        }).await?
    }

    pub async fn get_events(&self, mind_id: Uuid, limit: usize) -> AionResult<Vec<(String, Option<String>, i64)>> {
        let conn = self.conn.clone();
        let mind_id_owned = mind_id.to_string();
        tokio::task::spawn_blocking(move || {
            let c = conn.blocking_lock();
            let mut stmt = c.prepare(
                "SELECT kind, payload, timestamp FROM events WHERE mind_id = ?1 ORDER BY timestamp DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![mind_id_owned, limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
            let mut events = Vec::new();
            for r in rows { events.push(r?); }
            Ok::<Vec<(String, Option<String>, i64)>, AionError>(events)
        }).await?
    }
}
