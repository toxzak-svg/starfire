//! Training Database — Structured Conversation Storage
//!
//! Stores all Star conversations in a format suitable for:
//! - Analyzing Star's behavior and growth
//! - Training future neural components (like infant)
//! - Debugging and pattern analysis
//! - Evolving Star's knowledge base
//!
//! Schema:
//! - conversations: full conversation sessions
//! - turns: individual message turns
//! - facts: extracted facts from conversations
//! - corrections: when Zachary corrects Star

use rusqlite::{Connection, params};
use std::path::Path;

pub struct TrainingDB {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ConversationSession {
    pub id: i64,
    pub started_at: String,
    pub turn_count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Turn {
    pub speaker: String,
    pub content: String,
}

impl TrainingDB {
    /// Open or create the training database.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Initialize schema.
    fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                turn_count INTEGER DEFAULT 0,
                tags TEXT DEFAULT '',
                notes TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                turn_index INTEGER NOT NULL,
                speaker TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE IF NOT EXISTS facts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER,
                extracted_at TEXT NOT NULL,
                subject TEXT NOT NULL,
                predicate TEXT NOT NULL,
                object TEXT DEFAULT '',
                confidence REAL DEFAULT 0.5,
                verified INTEGER DEFAULT 0,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE IF NOT EXISTS corrections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER,
                turn_id INTEGER,
                incorrect TEXT NOT NULL,
                correction TEXT NOT NULL,
                explained TEXT DEFAULT '',
                timestamp TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id),
                FOREIGN KEY (turn_id) REFERENCES turns(id)
            );

            CREATE INDEX IF NOT EXISTS idx_turns_conversation ON turns(conversation_id);
            CREATE INDEX IF NOT EXISTS idx_facts_subject ON facts(subject);
        "#)?;
        Ok(())
    }

    /// Start a new conversation session.
    pub fn start_session(&self) -> rusqlite::Result<ConversationSession> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO conversations (started_at, turn_count) VALUES (?1, 0)",
            params![now],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(ConversationSession { id, started_at: now, turn_count: 0 })
    }

    /// End a conversation session.
    pub fn end_session(&self, id: i64) -> rusqlite::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conversations SET ended_at = ?1 WHERE id = ?2 AND ended_at IS NULL",
            params![now, id],
        )?;
        Ok(())
    }

    /// Record a turn and return its ID.
    pub fn record_turn(
        &self,
        conversation_id: i64,
        turn_index: i64,
        speaker: &str,
        content: &str,
    ) -> rusqlite::Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO turns (conversation_id, turn_index, speaker, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![conversation_id, turn_index, speaker, content, now],
        )?;
        
        // Update turn count
        self.conn.execute(
            "UPDATE conversations SET turn_count = turn_count + 1 WHERE id = ?1",
            params![conversation_id],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }

    /// Record an extracted fact.
    pub fn record_fact(
        &self,
        conversation_id: Option<i64>,
        subject: &str,
        predicate: &str,
        object: &str,
        confidence: f64,
    ) -> rusqlite::Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO facts (conversation_id, extracted_at, subject, predicate, object, confidence) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![conversation_id, now, subject, predicate, object, confidence],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Record a correction.
    pub fn record_correction(
        &self,
        conversation_id: Option<i64>,
        incorrect: &str,
        correction: &str,
    ) -> rusqlite::Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO corrections (conversation_id, incorrect, correction, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, incorrect, correction, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all facts about a subject.
    pub fn facts_about(&self, subject: &str) -> rusqlite::Result<Vec<(i64, String, String, String, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, subject, predicate, object, confidence FROM facts WHERE subject LIKE ?1 ORDER BY confidence DESC"
        )?;
        
        let facts = stmt.query_map(params![format!("%{}%", subject)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?.filter_map(|r| r.ok()).collect();
        
        Ok(facts)
    }

    /// Get recent conversations.
    pub fn recent_conversations(&self, limit: i64) -> rusqlite::Result<Vec<(i64, String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, turn_count FROM conversations WHERE ended_at IS NOT NULL ORDER BY started_at DESC LIMIT ?1"
        )?;
        
        let rows = stmt.query_map(params![limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.filter_map(|r| r.ok()).collect();
        
        Ok(rows)
    }

    /// Get conversation turns.
    pub fn get_turns(&self, conversation_id: i64) -> rusqlite::Result<Vec<Turn>> {
        let mut stmt = self.conn.prepare(
            "SELECT speaker, content FROM turns WHERE conversation_id = ?1 ORDER BY turn_index"
        )?;
        
        let turns = stmt.query_map(params![conversation_id], |row| {
            Ok(Turn {
                speaker: row.get(0)?,
                content: row.get(1)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        
        Ok(turns)
    }

    /// Export all training data as JSON.
    pub fn export_json(&self) -> rusqlite::Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, ended_at, turn_count FROM conversations WHERE ended_at IS NOT NULL ORDER BY started_at DESC"
        )?;
        
        let mut conversations = Vec::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, i64>(3)?))
        })?;
        
        for row in rows.filter_map(|r| r.ok()) {
            let (id, started_at, ended_at, turn_count) = row;
            let turns = self.get_turns(id)?;
            conversations.push(serde_json::json!({
                "id": id,
                "started_at": started_at,
                "ended_at": ended_at,
                "turn_count": turn_count,
                "turns": turns
            }));
        }
        
        Ok(serde_json::to_string_pretty(&conversations).unwrap_or_else(|_| "[]".to_string()))
    }

    /// Get database statistics.
    pub fn stats(&self) -> rusqlite::Result<(i64, i64, i64, i64)> {
        let conversations: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM conversations WHERE ended_at IS NOT NULL", [], |r| r.get(0)
        )?;
        let turns: i64 = self.conn.query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0)
        )?;
        let facts: i64 = self.conn.query_row("SELECT COUNT(*) FROM facts", [], |r| r.get(0)
        )?;
        let corrections: i64 = self.conn.query_row("SELECT COUNT(*) FROM corrections", [], |r| r.get(0)
        )?;
        Ok((conversations, turns, facts, corrections))
    }
}
