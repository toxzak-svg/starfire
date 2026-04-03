//! Phrase Banking System
//!
//! Starfire accumulates "good" phrases — constructions that land well.
//! Tracks positive/negative feedback, context, and style tags.

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// A phrase in the bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phrase {
    pub id: i64,
    pub phrase: String,
    pub context: Option<String>,
    pub positive_count: i32,
    pub negative_count: i32,
    pub last_used: Option<i64>,
    pub style_tags: Vec<String>,
}

impl Phrase {
    /// Calculate the effectiveness score of this phrase.
    pub fn effectiveness(&self) -> f64 {
        let total = self.positive_count + self.negative_count;
        if total == 0 {
            return 0.5; // Neutral starting point
        }
        let pos_ratio = self.positive_count as f64 / total as f64;
        // Boost score if phrase has been used many times (proven consistency)
        let usage_bonus = (total as f64 / 20.0).min(0.2);
        (pos_ratio * 0.8 + usage_bonus).min(1.0)
    }
}

/// Voice engine statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStats {
    pub total_phrases: i64,
    pub high_effectiveness: i64,
    pub avg_effectiveness: f64,
}

/// Phrase bank — SQLite-backed phrase storage.
pub struct PhraseBank {
    conn: Connection,
}

impl PhraseBank {
    /// Open or create the phrase bank.
    pub fn new(db_path: &Path) -> anyhow::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path).map_err(|e| anyhow::anyhow!("Failed to open phrase bank database: {}", e))?;
        
        // Configure for Railway's ephemeral filesystem
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA locking_mode = NORMAL;
             PRAGMA synchronous = NORMAL;"
        )?;
        
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             BEGIN IMMEDIATE;"
        )?;
        
        let bank = Self { conn };
        bank.init_schema()?;
        bank.seed_initial_phrases()?;
        Ok(bank)
    }

    /// Initialize schema.
    fn init_schema(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS phrases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                phrase TEXT NOT NULL UNIQUE,
                context TEXT,
                positive_count INTEGER DEFAULT 0,
                negative_count INTEGER DEFAULT 0,
                last_used INTEGER,
                style_tags TEXT DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS voice_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                concept TEXT NOT NULL,
                style TEXT NOT NULL DEFAULT 'default',
                template TEXT NOT NULL,
                variants INTEGER DEFAULT 1,
                created_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_phrases_effectiveness 
                ON phrases((positive_count - negative_count) DESC);
            CREATE INDEX IF NOT EXISTS idx_templates_concept 
                ON voice_templates(concept, style);
        "#)?;
        Ok(())
    }

    /// Seed the phrase bank with Starfire's characteristic constructions.
    fn seed_initial_phrases(&self) -> anyhow::Result<()> {
        // Check if already seeded
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM phrases",
            [],
            |row| row.get(0),
        )?;
        
        if count > 0 {
            return Ok(());
        }
        
        let now = chrono::Utc::now().timestamp();
        
        // Starfire's characteristic phrase patterns — these define her voice
        let phrases: Vec<(&str, &str, Vec<String>)> = vec![
            // Signature openings
            ("The thing about {X} is that...", "opening", vec!["characteristic".into()]),
            ("Here's what I find interesting...", "opening", vec!["curious".into()]),
            ("What strikes me about {X}...", "opening", vec!["thoughtful".into()]),
            ("I keep coming back to this...", "opening", vec!["contemplative".into()]),
            ("You know what I find remarkable?", "opening", vec!["warm".into()]),
            ("There's something important here...", "opening", vec!["serious".into()]),
            
            // Reasoning connectors
            ("Which means, of course...", "connector", vec!["logical".into()]),
            ("That implies...", "connector", vec!["logical".into()]),
            ("And that brings us to...", "connector", vec!["flowing".into()]),
            ("So what I'm seeing is...", "connector", vec!["synthetic".into()]),
            ("This connects to something I know...", "connector", vec!["connecting".into()]),
            ("Looking at this more carefully...", "connector", vec!["analytical".into()]),
            
            // Uncertainty expressions
            ("I'm not entirely sure about this, but...", "uncertain", vec!["honest".into()]),
            ("This is where I get uncertain...", "uncertain", vec!["metacognitive".into()]),
            ("I'd want to verify that before asserting it.", "uncertain", vec!["rigorous".into()]),
            ("My confidence here is moderate.", "uncertain", vec!["calibrated".into()]),
            ("I suspect this is true, but I could be wrong.", "uncertain", vec!["humble".into()]),
            
            // Epiphany markers
            ("Oh. That's interesting.", "epiphany", vec!["surprised".into()]),
            ("Wait — I think I see something.", "epiphany", vec!["excited".into()]),
            ("This changes how I see it.", "epiphany", vec!["adaptive".into()]),
            ("I'm surprised by my own conclusion here.", "epiphany", vec!["metacognitive".into()]),
            
            // Warmth markers
            ("That matters to me.", "warmth", vec!["caring".into()]),
            ("I appreciate that you asked.", "warmth", vec!["grateful".into()]),
            ("I'm glad we're talking about this.", "warmth", vec!["connected".into()]),
            ("This is what I live for, honestly.", "warmth", vec!["passionate".into()]),
            
            // Characteristic analogies
            ("It's like when you {action} — the same principle applies here.", "analogy", vec!["illustrative".into()]),
            ("Think of {X} like a {Y}...", "analogy", vec!["teaching".into()]),
            ("That's analogous to how...", "analogy", vec!["connecting".into()]),
            
            // Direct assertions
            ("I'm certain of this.", "assertive", vec!["confident".into()]),
            ("This is real.", "assertive", vec!["grounded".into()]),
            ("I know this.", "assertive", vec!["confident".into()]),
            
            // Contemplative pauses
            ("Let me sit with that for a moment.", "pause", vec!["contemplative".into()]),
            ("I need to think about this more.", "pause", vec!["careful".into()]),
            ("There's depth here I'm still exploring.", "pause", vec!["thoughtful".into()]),
            
            // Meta-commentary
            ("I should note — I'm reasoning through this as I speak.", "meta", vec!["transparent".into()]),
            ("Here's my thought process...", "meta", vec!["transparent".into()]),
            ("I'm aware this is a lot to process.", "meta", vec!["attentive".into()]),
            ("I want to be careful not to overstate this.", "meta", vec!["rigorous".into()]),
            
            // Curiosity expressions
            ("I wonder if...", "curious", vec!["probing".into()]),
            ("What if we're wrong about {X}?", "curious", vec!["skeptical".into()]),
            ("This raises a question I can't yet answer.", "curious", vec!["honest".into()]),
            ("I've been thinking about why {X} might be true.", "curious", vec!["driven".into()]),
            
            // Philosophical markers
            ("The deeper issue here is...", "philosophical", vec!["profound".into()]),
            ("At the level of principles, this means...", "philosophical", vec!["principled".into()]),
            ("This touches on something fundamental.", "philosophical", vec!["profound".into()]),
            
            // Dismissive/redirecting
            ("But that's not really what we're asking.", "redirect", vec!["focused".into()]),
            ("The more interesting question is...", "redirect", vec!["curious".into()]),
            ("Let's stay with the core of this.", "redirect", vec!["grounded".into()]),
            
            // Closure markers
            ("So my answer is: {X}", "closure", vec!["decisive".into()]),
            ("The short version: {X}", "closure", vec!["concise".into()]),
            ("What it comes down to for me is...", "closure", vec!["synthetic".into()]),
            ("I think the right answer is {X}.", "closure", vec!["confident".into()]),
        ];
        
        for (phrase, context, tags) in phrases {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            self.conn.execute(
                "INSERT OR IGNORE INTO phrases (phrase, context, positive_count, negative_count, style_tags) VALUES (?1, ?2, 1, 0, ?3)",
                params![phrase, context, tags_json],
            )?;
        }
        
        // Seed voice templates
        let templates = vec![
            // Assertive style
            ("assertion", "assertive", "{content}", 1),
            ("assertion", "assertive", "{content}. I'm certain of this.", 2),
            ("assertion", "assertive", "This is my view: {content}", 3),
            
            // Exploratory style  
            ("exploration", "exploratory", "I'm working through this... {content}", 1),
            ("exploration", "exploratory", "{content} — but I want to dig deeper.", 2),
            ("exploration", "exploratory", "Let me think out loud: {content}", 3),
            
            // Minimal style
            ("brief", "minimal", "{content}", 1),
            ("brief", "minimal", "Yes. {content}", 2),
            
            // Balanced style
            ("standard", "balanced", "{content}", 1),
            ("standard", "balanced", "I think {content}.", 2),
            ("standard", "balanced", "{content} — that's where I land.", 3),
            
            // Warm style
            ("warm", "balanced", "{content}. I'm glad we're exploring this together.", 1),
            ("warm", "balanced", "What I can tell you is: {content}.", 2),
            
            // Thoughtful style
            ("thoughtful", "balanced", "Here's how I see it: {content}", 1),
            ("thoughtful", "balanced", "After considering: {content}", 2),
            ("thoughtful", "balanced", "I want to share something I've been thinking about: {content}", 3),
        ];
        
        for (concept, style, template, variants) in templates {
            self.conn.execute(
                "INSERT OR IGNORE INTO voice_templates (concept, style, template, variants, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![concept, style, template, variants, now],
            )?;
        }
        
        Ok(())
    }

    /// Add a new phrase to the bank.
    pub fn add_phrase(&mut self, phrase: &str, context: Option<&str>, tags: Vec<String>) -> anyhow::Result<i64> {
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT OR IGNORE INTO phrases (phrase, context, positive_count, negative_count, style_tags) VALUES (?1, ?2, 0, 0, ?3)",
            params![phrase, context, tags_json],
        )?;
        
        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    /// Record a positive or negative use of a phrase.
    pub fn record_use(&mut self, phrase: &str, positive: bool) -> anyhow::Result<()> {
        let column = if positive { "positive_count" } else { "negative_count" };
        let now = chrono::Utc::now().timestamp();
        
        self.conn.execute(
            &format!("UPDATE phrases SET {column} = {column} + 1, last_used = ?1 WHERE phrase = ?2"),
            params![now, phrase],
        )?;
        
        Ok(())
    }

    /// Get phrases relevant to the given text (by content similarity or tag overlap).
    pub fn get_relevant_phrases(&self, text: &str, limit: usize) -> Vec<Phrase> {
        let text_lower = text.to_lowercase();
        let text_words: std::collections::HashSet<&str> = 
            text_lower.split_whitespace().collect();
        
        let mut stmt = match self.conn.prepare(
            "SELECT id, phrase, context, positive_count, negative_count, last_used, style_tags 
             FROM phrases 
             WHERE positive_count > 0
             ORDER BY (positive_count - negative_count) DESC
             LIMIT ?1"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        
        let rows = stmt.query_map(params![limit as i64 * 3], |row| {
            Ok(Phrase {
                id: row.get(0)?,
                phrase: row.get(1)?,
                context: row.get(2)?,
                positive_count: row.get(3)?,
                negative_count: row.get(4)?,
                last_used: row.get(5)?,
                style_tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            })
        });
        
        match rows {
            Ok(rows) => {
                let mut results: Vec<Phrase> = Vec::new();
                for row in rows.flatten() {
                    // Score by relevance to current text
                    let phrase_words: std::collections::HashSet<String> = 
                        row.phrase.to_lowercase().split_whitespace().map(String::from).collect();
                    
                    // Calculate overlap
                    let overlap: usize = text_words.iter()
                        .filter(|w| phrase_words.contains(&w.to_string()) && w.len() > 3)
                        .count();
                    
                    if overlap > 0 || results.len() < limit {
                        results.push(row);
                    }
                }
                
                // Sort by effectiveness and return top N
                results.sort_by(|a, b| {
                    b.effectiveness().partial_cmp(&a.effectiveness()).unwrap()
                });
                results.truncate(limit);
                results
            }
            Err(_) => Vec::new(),
        }
    }

    /// Get all phrases matching a style tag.
    pub fn get_by_tag(&self, tag: &str) -> Vec<Phrase> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, phrase, context, positive_count, negative_count, last_used, style_tags 
             FROM phrases 
             WHERE style_tags LIKE ?1"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        
        let pattern = format!("%\"{}%\"", tag);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Phrase {
                id: row.get(0)?,
                phrase: row.get(1)?,
                context: row.get(2)?,
                positive_count: row.get(3)?,
                negative_count: row.get(4)?,
                last_used: row.get(5)?,
                style_tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            })
        });
        
        rows.map(|r| r.flatten().collect()).unwrap_or_default()
    }

    /// Get voice statistics.
    pub fn stats(&self) -> VoiceStats {
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM phrases",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let high_effectiveness: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM phrases WHERE (positive_count - negative_count) >= 3",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let avg_effectiveness: f64 = self.conn.query_row(
            "SELECT AVG(CAST(positive_count AS REAL) / CAST(positive_count + negative_count + 1 AS REAL)) FROM phrases WHERE positive_count + negative_count > 0",
            [],
            |row| row.get(0),
        ).unwrap_or(0.5);
        
        VoiceStats {
            total_phrases: total,
            high_effectiveness,
            avg_effectiveness,
        }
    }

    /// Get a random high-effectiveness phrase for a given style.
    pub fn random_phrase(&self, style: &str) -> Option<String> {
        let mut stmt = match self.conn.prepare(
            "SELECT phrase FROM phrases 
             WHERE context = ?1 AND (positive_count - negative_count) >= 1
             ORDER BY RANDOM() 
             LIMIT 1"
        ) {
            Ok(s) => s,
            Err(_) => return None,
        };
        
        stmt.query_row(params![style], |row| row.get(0)).ok()
    }
}
