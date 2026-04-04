//! SQLite Storage Backend
//!
//! Single-file storage. No server. Human-readable schema.
//! Transactional for safety.

use crate::persistence::{Memory, MemoryDomain, Belief, BeliefState, IdentityGuard};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

/// A reasoning event — a record of Star's reasoning process at a point in time.
/// Used for self-probing, gap detection, and curiosity generation.
#[derive(Debug, Clone)]
pub struct ReasoningEvent {
    pub id: i64,
    pub query: String,
    pub conclusion: String,
    pub chain: Vec<String>,
    pub confidence_state: BeliefState,
    pub confidence_score: Option<f64>,
    pub emotional_valence: f64,
    pub engagement_depth: f64,
    pub topic: Option<String>,
    pub was_uncertain: bool,
    pub hedge_count: i32,
    pub timestamp: i64,
}

/// A gap in Star's reasoning — something worth probing.
#[derive(Debug, Clone)]
pub struct ReasoningGap {
    pub event_id: i64,
    pub query: String,
    pub conclusion: String,
    pub topic: String,
    pub salience: f64,
    pub emotional_valence: f64,
    pub why_it_matters: String,
}

/// The persistent store — SQLite backend for all Star's memory.
/// 
/// Thread-safe via Mutex. Each operation is transactional.
pub struct Store {
    conn: Mutex<Connection>,
    guard: Mutex<IdentityGuard>,
}

impl Store {
    /// Open or create the database at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create data directory")?;
        }

        let conn = Connection::open(path)
            .context("Failed to open database")?;
        
        // Configure for Railway's ephemeral filesystem
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 30000;
             PRAGMA locking_mode = NORMAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -64000;"
        )?;
        
        // Enable foreign keys and WAL mode for safety
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             BEGIN IMMEDIATE;"
        )?;

        let store = Self {
            conn: Mutex::new(conn),
            guard: Mutex::new(IdentityGuard::new()),
        };
        store.init_schema()?;
        Ok(store)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(r#"
            -- Identity core (frozen after formation)
            CREATE TABLE IF NOT EXISTS identity (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                formed_at INTEGER NOT NULL,
                updated_at INTEGER
            );

            -- Memory objects
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                domain TEXT NOT NULL,
                confidence REAL,
                importance REAL NOT NULL,
                formed_at INTEGER NOT NULL,
                access_count INTEGER DEFAULT 0,
                decay_rate REAL NOT NULL,
                last_accessed INTEGER,
                provenance TEXT,
                summary TEXT
            );

            -- Beliefs (meta-cognition)
            CREATE TABLE IF NOT EXISTS beliefs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                confidence_state TEXT NOT NULL,
                confidence_score REAL,
                based_on INTEGER,
                formed_at INTEGER NOT NULL,
                revised_from INTEGER,
                reasoning TEXT,
                FOREIGN KEY (based_on) REFERENCES memories(id),
                FOREIGN KEY (revised_from) REFERENCES beliefs(id)
            );

            -- Sessions
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                summary TEXT
            );

            -- Session memories (which memories were active in which session)
            CREATE TABLE IF NOT EXISTS session_memories (
                session_id INTEGER NOT NULL,
                memory_id INTEGER NOT NULL,
                access_order INTEGER NOT NULL,
                PRIMARY KEY (session_id, memory_id),
                FOREIGN KEY (session_id) REFERENCES sessions(id),
                FOREIGN KEY (memory_id) REFERENCES memories(id)
            );

            -- Indexes for fast retrieval
            CREATE INDEX IF NOT EXISTS idx_memories_domain ON memories(domain);
            CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);
            CREATE INDEX IF NOT EXISTS idx_memories_formed_at ON memories(formed_at);
            CREATE INDEX IF NOT EXISTS idx_beliefs_state ON beliefs(confidence_state);
            CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC);

            -- Reasoning events (self-probing foundation)
            CREATE TABLE IF NOT EXISTS reasoning_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                conclusion TEXT NOT NULL,
                chain TEXT NOT NULL,
                confidence_state TEXT NOT NULL,
                confidence_score REAL,
                emotional_valence REAL NOT NULL,
                engagement_depth REAL NOT NULL,
                topic TEXT,
                was_uncertain INTEGER NOT NULL,
                hedge_count INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_reasoning_timestamp ON reasoning_events(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_reasoning_uncertain ON reasoning_events(was_uncertain) WHERE was_uncertain = 1;

            -- Phrase bank for voice engine
            CREATE TABLE IF NOT EXISTS phrases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                phrase TEXT NOT NULL UNIQUE,
                context TEXT,
                positive_count INTEGER DEFAULT 0,
                negative_count INTEGER DEFAULT 0,
                last_used INTEGER,
                style_tags TEXT DEFAULT '[]'
            );

            -- Voice templates for expression variation
            CREATE TABLE IF NOT EXISTS voice_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                concept TEXT NOT NULL,
                style TEXT NOT NULL DEFAULT 'default',
                template TEXT NOT NULL,
                variants INTEGER DEFAULT 1,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_phrases_effectiveness ON phrases(positive_count DESC);
            CREATE INDEX IF NOT EXISTS idx_templates_concept_style ON voice_templates(concept, style);
        "#)?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────
    // Identity Operations
    // ─────────────────────────────────────────────────────────────────

    /// Store an identity claim (key-value pair).
    pub fn put_identity(&self, key: &str, value: &str, formed_at: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO identity (key, value, formed_at) VALUES (?1, ?2, ?3)",
            params![key, value, formed_at],
        )?;
        Ok(())
    }

    /// Get an identity claim.
    pub fn get_identity(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT value FROM identity WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Get all identity claims.
    pub fn get_all_identity(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM identity")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ─────────────────────────────────────────────────────────────────
    // Memory Operations
    // ─────────────────────────────────────────────────────────────────

    /// Insert a new memory. Returns the assigned ID.
    /// 
    /// Identity and Relationship memories are protected by the IdentityGuard.
    /// Contradictions of protected memories are blocked.
    pub fn insert_memory(&self, memory: &Memory) -> Result<i64> {
        // Check for conflicts with existing protected memories
        let guard = self.guard.lock().unwrap();
        
        // Get existing memories from this domain
        let existing: Vec<Memory> = self.get_memories_by_domain(memory.domain, None)?;
        if let Some(conflict) = guard.check_conflict(&memory.content, &existing) {
            anyhow::bail!(
                "Cannot insert memory: conflicts with protected memory: \"{}\"",
                conflict.content
            );
        }
        
        // Also check Identity domain for Star-related self-statements
        if memory.domain != MemoryDomain::Identity {
            let identity_memories: Vec<Memory> = self.get_memories_by_domain(MemoryDomain::Identity, None)?;
            if let Some(conflict) = guard.check_conflict(&memory.content, &identity_memories) {
                anyhow::bail!(
                    "Cannot insert memory: contradicts protected identity: \"{}\"",
                    conflict.content
                );
            }
        }
        drop(guard);
        
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO memories 
               (content, domain, confidence, importance, formed_at, access_count, decay_rate, last_accessed, provenance, summary)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                memory.content,
                format!("{:?}", memory.domain).to_lowercase(),
                memory.confidence,
                memory.importance,
                memory.formed_at,
                memory.access_count,
                memory.decay_rate,
                memory.last_accessed,
                memory.provenance,
                memory.summary,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Update a memory's access stats.
    pub fn record_memory_access(&self, memory_id: i64, now: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memories SET access_count = access_count + 1, last_accessed = ?1 WHERE id = ?2",
            params![now, memory_id],
        )?;
        Ok(())
    }

    /// Get a memory by ID.
    pub fn get_memory(&self, id: i64) -> Result<Option<Memory>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT * FROM memories WHERE id = ?1",
                params![id],
                row_to_memory,
            )
            .optional()?;
        Ok(result)
    }

    /// Search memories by relevance to a query.
    pub fn search_memories(&self, query: &str, limit: usize, domain: Option<MemoryDomain>) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        
        let domain_filter = domain.map(|d| format!("AND domain = '{}'", format!("{:?}", d).to_lowercase())).unwrap_or_default();
        let sql = format!(
            "SELECT * FROM memories WHERE content LIKE ?1 {} ORDER BY importance DESC, last_accessed DESC LIMIT ?2",
            domain_filter
        );
        
        // Use word boundaries to avoid matching "brain" when searching for "rain"
        // Match: "rain " or " rain" or " rain " at word boundaries
        let exact_pattern = format!("% {} %", query);
        let partial_pattern = format!("%{}%", query);
        
        // Try exact word match first (surrounded by spaces)
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![exact_pattern, limit as i64], row_to_memory)?;
        
        let mut results = Vec::new();
        for row in rows {
            let mut mem = row?;
            mem.record_access(now);
            results.push(mem);
        }
        
        // If no exact matches and query is long enough, try partial match
        if results.is_empty() && query.len() >= 4 {
            // Avoid matching substrings in common words
            let skip_substrings = ["the ", "and ", "for ", "brain", "drain", "train", "grain", "plain", "remain", "contain", "about", "which"];
            if !skip_substrings.contains(&query.to_lowercase().as_str()) {
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map(params![partial_pattern, limit as i64], row_to_memory)?;
                for row in rows {
                    let mut mem = row?;
                    mem.record_access(now);
                    results.push(mem);
                }
            }
        }
        
        Ok(results)
    }

    /// Get all memories of a given domain.
    pub fn get_memories_by_domain(&self, domain: MemoryDomain, limit: Option<usize>) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            "SELECT * FROM memories WHERE domain = ?1 ORDER BY importance DESC, formed_at DESC{}",
            limit_clause
        );
        let domain_str = format!("{:?}", domain).to_lowercase();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![domain_str], row_to_memory)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Delete a memory by ID.
    pub fn delete_memory(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get memories that have decayed below threshold (for cleanup).
    pub fn get_forgotten_memories(&self, now: i64, confidence_threshold: f64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        // This is approximate — full implementation would calculate decay per-memory
        let sql = "SELECT * FROM memories WHERE decay_rate > 0 AND importance < 0.3 AND access_count < 3";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], row_to_memory)?;
        let mut results = Vec::new();
        for row in rows {
            let mem = row?;
            if let Some(conf) = mem.current_confidence(now) {
                if conf < confidence_threshold {
                    results.push(mem);
                }
            }
        }
        Ok(results)
    }

    /// Get total memory count.
    pub fn memory_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;
        Ok(count)
    }

    // ─────────────────────────────────────────────────────────────────
    // Belief Operations
    // ─────────────────────────────────────────────────────────────────

    /// Insert a new belief.
    pub fn insert_belief(&self, belief: &Belief) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO beliefs 
               (content, confidence_state, confidence_score, based_on, formed_at, revised_from, reasoning)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                belief.content,
                format!("{:?}", belief.confidence_state).to_lowercase(),
                belief.confidence_score,
                belief.based_on,
                belief.formed_at,
                belief.revised_from,
                belief.reasoning,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all beliefs.
    pub fn get_all_beliefs(&self) -> Result<Vec<Belief>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM beliefs ORDER BY formed_at DESC")?;
        let rows = stmt.query_map([], row_to_belief)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get beliefs by confidence state.
    pub fn get_beliefs_by_state(&self, state: BeliefState) -> Result<Vec<Belief>> {
        let conn = self.conn.lock().unwrap();
        let sql = "SELECT * FROM beliefs WHERE confidence_state = ?1 ORDER BY formed_at DESC";
        let state_str = format!("{:?}", state).to_lowercase();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![state_str], row_to_belief)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get the most recent beliefs.
    pub fn get_recent_beliefs(&self, limit: usize) -> Result<Vec<Belief>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM beliefs ORDER BY formed_at DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], row_to_belief)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ─────────────────────────────────────────────────────────────────
    // Reasoning Event Operations (Self-Probing Foundation)
    // ─────────────────────────────────────────────────────────────────

    /// Record a reasoning event.
    pub fn record_reasoning_event(&self, event: &ReasoningEvent) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let chain_json = serde_json::to_string(&event.chain)
            .unwrap_or_else(|_| "[]".to_string());
        let state_str = format!("{:?}", event.confidence_state).to_lowercase();
        let was_uncertain = if event.was_uncertain { 1 } else { 0 };
        conn.execute(
            r#"INSERT INTO reasoning_events 
               (query, conclusion, chain, confidence_state, confidence_score,
                emotional_valence, engagement_depth, topic, was_uncertain, hedge_count, timestamp)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                event.query,
                event.conclusion,
                chain_json,
                state_str,
                event.confidence_score,
                event.emotional_valence,
                event.engagement_depth,
                event.topic,
                was_uncertain,
                event.hedge_count,
                event.timestamp,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Search reasoning events by query pattern.
    pub fn search_reasoning_events(&self, query_pattern: &str, limit: usize) -> Result<Vec<ReasoningEvent>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query_pattern);
        let mut stmt = conn.prepare(
            "SELECT * FROM reasoning_events WHERE query LIKE ?1 OR conclusion LIKE ?1 ORDER BY timestamp DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64], row_to_reasoning_event)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get recent reasoning events.
    pub fn get_recent_reasoning_events(&self, limit: usize) -> Result<Vec<ReasoningEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT * FROM reasoning_events ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_reasoning_event)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get reasoning events from the last N seconds.
    pub fn get_reasoning_events_since(&self, seconds_ago: i64) -> Result<Vec<ReasoningEvent>> {
        let conn = self.conn.lock().unwrap();
        let since = crate::now_timestamp() - seconds_ago;
        let mut stmt = conn.prepare(
            "SELECT * FROM reasoning_events WHERE timestamp >= ?1 ORDER BY timestamp DESC"
        )?;
        let rows = stmt.query_map(params![since], row_to_reasoning_event)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get uncertain reasoning events (gaps worth probing).
    pub fn get_uncertain_reasoning_events(&self, max_age_seconds: i64, limit: usize) -> Result<Vec<ReasoningEvent>> {
        let conn = self.conn.lock().unwrap();
        let since = crate::now_timestamp() - max_age_seconds;
        let mut stmt = conn.prepare(
            "SELECT * FROM reasoning_events 
             WHERE (was_uncertain = 1 OR confidence_score < 0.4 OR hedge_count > 0)
               AND timestamp >= ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![since, limit as i64], row_to_reasoning_event)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Detect reasoning gaps — uncertain events ranked by salience.
    /// Returns events that are worth self-probing.
    pub fn detect_reasoning_gaps(&self, max_age_days: i64, limit: usize) -> Result<Vec<ReasoningGap>> {
        let conn = self.conn.lock().unwrap();
        let since = crate::now_timestamp() - (max_age_days * 24 * 60 * 60);
        
        // Find uncertain events
        let mut stmt = conn.prepare(
            "SELECT * FROM reasoning_events 
             WHERE (was_uncertain = 1 OR confidence_score < 0.4 OR hedge_count > 0)
               AND timestamp >= ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;
        
        let rows = stmt.query_map(params![since, (limit * 2) as i64], row_to_reasoning_event)?;
        let mut gaps = Vec::new();
        let now = crate::now_timestamp();
        
        for row in rows {
            if let Ok(event) = row {
                // Compute salience: recency * uncertainty * emotional salience
                let age_hours = (now - event.timestamp) as f64 / 3600.0;
                let recency = 1.0 / (1.0 + age_hours / 24.0); // decays over days
                
                let uncertainty = if event.was_uncertain { 1.0 } else { 0.5 };
                let hedge_bonus = (event.hedge_count as f64).min(1.0) * 0.3;
                let score_uncertainty = event.confidence_score
                    .map(|s| if s < 0.4 { 1.0 } else { 0.5 })
                    .unwrap_or(0.7);
                
                let emotional = event.emotional_valence.abs(); // how charged was this?
                let salience = recency * (uncertainty + hedge_bonus) * score_uncertainty * (1.0 + emotional * 0.5);
                
                // Don't probe if salience is too low
                if salience < 0.05 {
                    continue;
                }
                
                let topic = event.topic.clone().unwrap_or_else(|| {
                    // Extract from query if no topic stored
                    event.query.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
                });
                
                gaps.push(ReasoningGap {
                    event_id: event.id,
                    query: event.query.clone(),
                    conclusion: event.conclusion.clone(),
                    topic: topic.clone(),
                    salience,
                    emotional_valence: event.emotional_valence,
                    why_it_matters: format!(
                        "I concluded '{}' with {} confidence — {}",
                        &event.conclusion[..event.conclusion.len().min(50)],
                        format!("{:?}", event.confidence_state).to_lowercase(),
                        if event.was_uncertain { "I wasn't sure" }
                        else if event.hedge_count > 0 { "I hedged" }
                        else { "my confidence was low" }
                    ),
                });
            }
            if gaps.len() >= limit {
                break;
            }
        }
        
        // Sort by salience descending
        gaps.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));
        gaps.truncate(limit);
        
        Ok(gaps)
    }

    /// Get reasoning event count.
    pub fn reasoning_event_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM reasoning_events", [], |row| row.get(0))?;
        Ok(count)
    }

    // ─────────────────────────────────────────────────────────────────
    // Session Operations
    // ─────────────────────────────────────────────────────────────────

    /// Start a new session. Returns the session ID.
    pub fn start_session(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "INSERT INTO sessions (started_at) VALUES (?1)",
            params![now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// End a session.
    pub fn end_session(&self, session_id: i64, summary: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = crate::now_timestamp();
        conn.execute(
            "UPDATE sessions SET ended_at = ?1, summary = ?2 WHERE id = ?3",
            params![now, summary, session_id],
        )?;
        Ok(())
    }

    /// Get session by ID.
    pub fn get_session(&self, id: i64) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT * FROM sessions WHERE id = ?1",
                params![id],
                row_to_session,
            )
            .optional()?;
        Ok(result)
    }

    /// Get the most recent session.
    pub fn get_last_session(&self) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT * FROM sessions ORDER BY started_at DESC LIMIT 1",
                [],
                row_to_session,
            )
            .optional()?;
        Ok(result)
    }

    /// Get recent sessions.
    pub fn get_recent_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM sessions ORDER BY started_at DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], row_to_session)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ─────────────────────────────────────────────────────────────────
    // Utility
    // ─────────────────────────────────────────────────────────────────

    /// Get a full snapshot of Star's memory state (for debugging/inspection).
    pub fn snapshot(&self) -> Result<MemorySnapshot> {
        Ok(MemorySnapshot {
            memory_count: self.memory_count()?,
            beliefs_count: {
                let conn = self.conn.lock().unwrap();
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM beliefs", [], |row| row.get(0))?;
                count
            },
            sessions_count: {
                let conn = self.conn.lock().unwrap();
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
                count
            },
            domain_breakdown: {
                let conn = self.conn.lock().unwrap();
                let mut stmt = conn.prepare(
                    "SELECT domain, COUNT(*) FROM memories GROUP BY domain"
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?;
                rows.into_iter().filter_map(|r| r.ok()).collect()
            },
        })
    }
}

/// A simplified session record.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub summary: Option<String>,
}

/// Snapshot of overall memory state.
#[derive(Debug, Default)]
pub struct MemorySnapshot {
    pub memory_count: i64,
    pub beliefs_count: i64,
    pub sessions_count: i64,
    pub domain_breakdown: std::collections::HashMap<String, i64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Row conversion helpers
// ─────────────────────────────────────────────────────────────────────────────

fn row_to_memory(row: &rusqlite::Row) -> rusqlite::Result<Memory> {
    let domain_str: String = row.get(2)?;
    let domain = match domain_str.as_str() {
        "identity" => MemoryDomain::Identity,
        "empirical" => MemoryDomain::Empirical,
        "procedural" => MemoryDomain::Procedural,
        "episodic" => MemoryDomain::Episodic,
        "relationship" => MemoryDomain::Relationship,
        _ => MemoryDomain::Episodic,
    };

    Ok(Memory {
        id: Some(row.get(0)?),
        content: row.get(1)?,
        domain,
        confidence: row.get(3)?,
        importance: row.get(4)?,
        formed_at: row.get(5)?,
        access_count: row.get(6)?,
        decay_rate: row.get(7)?,
        last_accessed: row.get(8)?,
        provenance: row.get(9)?,
        summary: row.get(10)?,
    })
}

fn row_to_belief(row: &rusqlite::Row) -> rusqlite::Result<Belief> {
    let state_str: String = row.get(2)?;
    let state = match state_str.as_str() {
        "knows" => BeliefState::Knows,
        "thinks" => BeliefState::Thinks,
        "believes" => BeliefState::Believes,
        "suspects" => BeliefState::Suspects,
        "unknown" => BeliefState::Unknown,
        _ => BeliefState::Unknown,
    };

    Ok(Belief {
        id: Some(row.get(0)?),
        content: row.get(1)?,
        confidence_state: state,
        confidence_score: row.get(3)?,
        based_on: row.get(4)?,
        formed_at: row.get(5)?,
        revised_from: row.get(6)?,
        reasoning: row.get(7)?,
    })
}

fn row_to_session(row: &rusqlite::Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        summary: row.get(3)?,
    })
}

fn row_to_reasoning_event(row: &rusqlite::Row) -> rusqlite::Result<ReasoningEvent> {
    let state_str: String = row.get(4)?;
    let state = match state_str.as_str() {
        "knows" => BeliefState::Knows,
        "thinks" => BeliefState::Thinks,
        "believes" => BeliefState::Believes,
        "suspects" => BeliefState::Suspects,
        "unknown" => BeliefState::Unknown,
        _ => BeliefState::Unknown,
    };

    let chain_str: String = row.get(2)?;
    let chain: Vec<String> = serde_json::from_str(&chain_str).unwrap_or_default();

    let was_uncertain_i64: i64 = row.get(9)?;
    let hedge_i64: i64 = row.get(10)?;

    Ok(ReasoningEvent {
        id: row.get(0)?,
        query: row.get(1)?,
        conclusion: row.get(2)?,
        chain,
        confidence_state: state,
        confidence_score: row.get(5)?,
        emotional_valence: row.get(6)?,
        engagement_depth: row.get(7)?,
        topic: row.get(8)?,
        was_uncertain: was_uncertain_i64 != 0,
        hedge_count: hedge_i64 as i32,
        timestamp: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> Store {
        let dir = std::env::temp_dir().join("star_test_store");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.db");
        Store::open(&path).unwrap()
    }

    #[test]
    fn test_identity_crud() {
        let store = test_store();
        let now = crate::now_timestamp();
        
        store.put_identity("name", "Star", now).unwrap();
        assert_eq!(store.get_identity("name").unwrap(), Some("Star".to_string()));
        
        store.put_identity("name", "Starfire", now, now).unwrap();
        assert_eq!(store.get_identity("name").unwrap(), Some("Starfire".to_string()));
    }

    #[test]
    fn test_memory_crud() {
        let store = test_store();
        
        let mem = Memory::new("Test memory", MemoryDomain::Empirical, 0.7)
            .with_confidence(0.9)
            .with_provenance("User told me");
        
        let id = store.insert_memory(&mem).unwrap();
        assert!(id > 0);
        
        let retrieved = store.get_memory(id).unwrap().unwrap();
        assert_eq!(retrieved.content, "Test memory");
        assert_eq!(retrieved.domain, MemoryDomain::Empirical);
    }

    #[test]
    fn test_search_memories() {
        let store = test_store();

        let mem1 = Memory::new("The sky is blue", MemoryDomain::Empirical, 0.8);
        let mem2 = Memory::new("The grass is green", MemoryDomain::Empirical, 0.6);
        let mem3 = Memory::new("Fish live in water", MemoryDomain::Empirical, 0.7);

        store.insert_memory(&mem1).unwrap();
        store.insert_memory(&mem2).unwrap();
        store.insert_memory(&mem3).unwrap();

        // Search for "sky" should match "The sky is blue"
        let results = store.search_memories("sky", 10, None).unwrap();
        assert!(!results.is_empty());

        // Search for "blue" should match "The sky is blue"
        let results2 = store.search_memories("blue", 10, None).unwrap();
        assert!(!results2.is_empty());
    }

    #[test]
    fn test_session_lifecycle() {
        let store = test_store();
        
        let session_id = store.start_session().unwrap();
        assert!(session_id > 0);
        
        store.end_session(session_id, Some("Test conversation")).unwrap();
        
        let session = store.get_session(session_id).unwrap().unwrap();
        assert!(session.ended_at.is_some());
        assert_eq!(session.summary, Some("Test conversation".to_string()));
    }

    #[test]
    fn test_snapshot() {
        let store = test_store();
        
        let mem = Memory::new("Test", MemoryDomain::Empirical, 0.5);
        store.insert_memory(&mem).unwrap();
        
        let snap = store.snapshot().unwrap();
        assert_eq!(snap.memory_count, 1);
        assert_eq!(snap.beliefs_count, 0);
    }
}
