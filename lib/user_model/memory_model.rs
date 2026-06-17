//! User memory model — what Star knows about Zachary's memory
//!
//! Tracks what Zachary remembers well, what he tends to forget,
//! and how his memory interacts with Star's self-model.

use std::collections::HashMap;

/// Memory strength for a topic/domain
#[derive(Debug, Clone)]
pub struct MemoryStrength {
    pub topic: String,
    /// How well Zachary remembers this (0-1)
    pub strength: f64,
    pub access_count: usize,
    pub last_accessed: i64,
}

/// User memory model
#[derive(Debug, Clone)]
pub struct UserMemoryModel {
    /// Zachary's memory strengths by topic/domain
    strengths: HashMap<String, MemoryStrength>,
    /// Topics Zachary has introduced and might remember
    introduced_topics: Vec<String>,
    /// Topics that faded or were not retained
    forgotten_topics: Vec<String>,
}

impl UserMemoryModel {
    pub fn new() -> Self {
        Self {
            strengths: HashMap::new(),
            introduced_topics: Vec::new(),
            forgotten_topics: Vec::new(),
        }
    }

    /// Record that Zachary introduced a topic
    pub fn record_introduction(&mut self, topic: &str) {
        if !self.introduced_topics.contains(&topic.to_lowercase()) {
            self.introduced_topics.push(topic.to_lowercase());
        }
    }

    /// Record that Zachary demonstrated memory of a topic
    pub fn record_memory_access(&mut self, topic: &str, success: bool) {
        let key = topic.to_lowercase();

        if success {
            if let Some(strength) = self.strengths.get_mut(&key) {
                strength.strength = (strength.strength * 0.9 + 0.1).min(1.0);
                strength.access_count += 1;
                strength.last_accessed = crate::now_timestamp();
            } else {
                self.strengths.insert(key, MemoryStrength {
                    topic: topic.to_string(),
                    strength: 0.6,
                    access_count: 1,
                    last_accessed: crate::now_timestamp(),
                });
            }
        } else {
            if let Some(strength) = self.strengths.get_mut(&key) {
                strength.strength *= 0.8;
            }
        }
    }

    /// Get memory strength for a topic
    pub fn get_strength(&self, topic: &str) -> Option<f64> {
        self.strengths.get(&topic.to_lowercase())
            .map(|s| s.strength)
    }

    /// Check if Zachary introduced this topic
    pub fn did_introduce(&self, topic: &str) -> bool {
        self.introduced_topics.contains(&topic.to_lowercase())
    }

    /// Topics that might have been forgotten
    pub fn potentially_forgotten(&self) -> Vec<&str> {
        self.strengths.iter()
            .filter(|(_, s)| s.strength < 0.3)
            .map(|(_, s)| s.topic.as_str())
            .collect()
    }

    /// Get all strong topics (where Zachary has good memory)
    pub fn strong_topics(&self) -> Vec<&str> {
        self.strengths.iter()
            .filter(|(_, s)| s.strength > 0.7)
            .map(|(_, s)| s.topic.as_str())
            .collect()
    }
}

impl Default for UserMemoryModel {
    fn default() -> Self {
        Self::new()
    }
}