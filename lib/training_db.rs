//! Training DB — persistent training data store

use std::path::Path;
use std::sync::Mutex;

/// A training database session.
#[derive(Debug)]
pub struct TrainingSession {
    pub id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub examples_seen: i32,
}

/// Training database — stores examples for Star's learning.
pub struct TrainingDB {
    conn: Mutex<rusqlite::Connection>,
}

impl TrainingDB {
    /// Open or create the training database.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = rusqlite::Connection::open(path)?;
        
        // Configure for Railway's ephemeral filesystem
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 30000;
             PRAGMA locking_mode = NORMAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -64000;"
        )?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS training_sessions (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 started_at INTEGER NOT NULL,
                 ended_at INTEGER,
                 examples_seen INTEGER DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS training_examples (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 session_id INTEGER,
                 input TEXT NOT NULL,
                 output TEXT,
                 confidence REAL,
                 timestamp INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_examples_session ON training_examples(session_id);"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Start a new training session.
    pub fn start_session(&self) -> anyhow::Result<TrainingSession> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "INSERT INTO training_sessions (started_at) VALUES (?1)",
            rusqlite::params![now],
        )?;
        let id = conn.last_insert_rowid();
        Ok(TrainingSession {
            id,
            started_at: now,
            ended_at: None,
            examples_seen: 0,
        })
    }

    /// Export all training data as JSON.
    pub fn export_json(&self) -> anyhow::Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT input, output, confidence FROM training_examples ORDER BY timestamp DESC LIMIT 1000")?;
        let rows: Vec<(String, Option<String>, Option<f64>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(serde_json::to_string(&rows)?)
    }

    /// Get training stats.
    pub fn stats(&self) -> anyhow::Result<(i64, i64, i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let convos: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT session_id) FROM training_examples", [], |r| r.get(0)
        )?;
        let turns: i64 = conn.query_row(
            "SELECT COUNT(*) FROM training_examples", [], |r| r.get(0)
        )?;
        let facts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM training_examples WHERE output IS NOT NULL", [], |r| r.get(0)
        )?;
        let corrections: i64 = 0; // Placeholder
        Ok((convos, turns, facts, corrections))
    }

    /// Record a conversation turn.
    pub fn record_turn(&self, session_id: i64, input: &str, output: &str, confidence: f64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "INSERT INTO training_examples (session_id, input, output, confidence, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![session_id, input, output, confidence, now],
        )?;
        Ok(())
    }

    /// Record a fact from Star's reasoning.
    pub fn record_fact(&self, session_id: i64, fact: &str, confidence: f64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "INSERT INTO training_examples (session_id, input, output, confidence, timestamp) VALUES (?1, ?2, NULL, ?3, ?4)",
            rusqlite::params![session_id, fact, confidence, now],
        )?;
        Ok(())
    }

    /// End a training session.
    pub fn end_session(&self, session_id: i64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "UPDATE training_sessions SET ended_at = ?1, examples_seen = examples_seen + 1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db_path() -> std::path::PathBuf {
        // Use a unique path each time to avoid conflicts
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("star_test_training_{}.db", unique));
        path
    }

    #[test]
    fn test_training_session_creation() {
        let path = temp_db_path();
        let _ = fs::remove_file(&path);
        let db = TrainingDB::open(&path).expect("Failed to open training DB");
        let session = db.start_session().expect("Failed to start session");
        assert!(session.id > 0);
        assert!(session.started_at > 0);
        db.end_session(session.id).expect("Failed to end session");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_turn() {
        let path = temp_db_path();
        let _ = fs::remove_file(&path);
        let db = TrainingDB::open(&path).expect("Failed to open training DB");
        let session = db.start_session().expect("Failed to start session");
        db.record_turn(session.id, "test input", "test output", 0.9).expect("Failed to record turn");
        db.end_session(session.id).expect("Failed to end session");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_fact() {
        let path = temp_db_path();
        let _ = fs::remove_file(&path);
        let db = TrainingDB::open(&path).expect("Failed to open training DB");
        let session = db.start_session().expect("Failed to start session");
        db.record_fact(session.id, "test fact", 0.7).expect("Failed to record fact");
        db.end_session(session.id).expect("Failed to end session");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_stats() {
        let path = temp_db_path();
        let _ = fs::remove_file(&path);
        let db = TrainingDB::open(&path).expect("Failed to open training DB");
        let session = db.start_session().expect("Failed to start session");
        db.record_turn(session.id, "input1", "output1", 0.5).expect("Failed to record turn");
        db.record_fact(session.id, "fact1", 0.8).expect("Failed to record fact");
        db.end_session(session.id).expect("Failed to end session");

        let (convos, turns, facts, corrections) = db.stats().expect("Failed to get stats");
        assert_eq!(convos, 1, "Expected 1 conversation, got {}", convos);
        assert_eq!(turns, 2, "Expected 2 training examples, got {}", turns);
        assert_eq!(facts, 1, "Expected 1 fact (turn with output), got {}", facts);
        assert_eq!(corrections, 0, "Expected 0 corrections");
        let _ = fs::remove_file(&path);
    }
}
