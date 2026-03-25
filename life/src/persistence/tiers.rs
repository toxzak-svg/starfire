//! Memory Tiering — R&D-C
//!
//! Inspired by infant's memory architecture: working → episodic → semantic → identity.
//! Each tier has different decay characteristics and capacity limits.
//!
//! | Tier       | Capacity | Decay    | Notes                              |
//! |------------|----------|----------|-------------------------------------|
//! | Working    | 10       | None     | Current conversation, in-memory     |
//! | Episodic   | 100      | Fast     | Recent experiences, conversation    |
//! | Semantic   | 500      | Slow     | Distilled facts, accumulated knowledge|
//! | Identity   | ∞        | None     | Core self-knowledge, protected     |
//!
//! The tier a memory lives in determines:
//! - How quickly it decays
//! - How it's retrieved
//! - What triggers consolidation

use crate::persistence::{Memory, MemoryDomain};
use std::collections::VecDeque;

/// Memory tier — determines decay and eviction policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    /// Current conversation — no persistence between sessions
    Working,
    /// Recent experiences — decays fast unless accessed
    Episodic,
    /// Distilled facts — slow decay, reinforced by access
    Semantic,
    /// Core identity — no decay, protected
    Identity,
}

impl MemoryTier {
    /// Maximum items in this tier (None = unlimited).
    pub fn capacity(&self) -> Option<usize> {
        match self {
            MemoryTier::Working => Some(10),
            MemoryTier::Episodic => Some(100),
            MemoryTier::Semantic => Some(500),
            MemoryTier::Identity => None,
        }
    }

    /// Base decay rate per day (as multiplier).
    pub fn base_decay_rate(&self) -> f64 {
        match self {
            MemoryTier::Working => 0.0,      // No decay — flushed each session
            MemoryTier::Episodic => 0.15,    // 85% lost per day without access
            MemoryTier::Semantic => 0.03,    // Slow decay — 3% per day
            MemoryTier::Identity => 0.0,     // No decay — protected
        }
    }

    /// Determine the tier for a new memory.
    pub fn for_new_memory(memory: &Memory) -> Self {
        match memory.domain {
            MemoryDomain::Identity => MemoryTier::Identity,
            MemoryDomain::Relationship => MemoryTier::Semantic, // Stable, slow decay
            MemoryDomain::Procedural => MemoryTier::Semantic,  // Skills, reinforced by use
            MemoryDomain::Empirical => {
                // High importance → Semantic (distill into facts)
                // Low importance → Episodic (transient experience)
                if memory.importance >= 0.7 {
                    MemoryTier::Semantic
                } else {
                    MemoryTier::Episodic
                }
            }
            MemoryDomain::Episodic => MemoryTier::Episodic,
        }
    }
}

/// Working memory — in-memory, session-scoped.
///
/// This is NOT persisted. It lives in Runtime's memory and is reconstructed
/// each session from conversation context.
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    /// Recent conversation turns (user → Star)
    turns: VecDeque<ConversationTurn>,
    /// Current reasoning context
    context_stack: Vec<String>,
    /// Recently accessed memories (IDs)
    recent_accesses: Vec<i64>,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            turns: VecDeque::with_capacity(10),
            context_stack: Vec::with_capacity(5),
            recent_accesses: Vec::with_capacity(20),
        }
    }

    /// Add a conversation turn.
    pub fn add_turn(&mut self, user: &str, star: &str) {
        if self.turns.len() >= 10 {
            self.turns.pop_front();
        }
        self.turns.push_back(ConversationTurn {
            user: user.to_string(),
            star: star.to_string(),
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// Get recent turns.
    pub fn recent_turns(&self, n: usize) -> Vec<&ConversationTurn> {
        self.turns.iter().rev().take(n).collect()
    }

    /// Push a reasoning context.
    pub fn push_context(&mut self, context: &str) {
        if self.context_stack.len() >= 5 {
            self.context_stack.pop();
        }
        self.context_stack.push(context.to_string());
    }

    /// Record memory access.
    pub fn record_access(&mut self, memory_id: i64) {
        // Remove if already exists
        self.recent_accesses.retain(|&id| id != memory_id);
        self.recent_accesses.push(memory_id);
        if self.recent_accesses.len() > 20 {
            self.recent_accesses.remove(0);
        }
    }

    /// Get recently accessed memory IDs.
    pub fn recent_accesses(&self) -> &[i64] {
        &self.recent_accesses
    }

    /// Get conversation history as strings.
    pub fn history_strings(&self) -> Vec<(String, String)> {
        self.turns.iter().map(|t| (t.user.clone(), t.star.clone())).collect()
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// A single conversation turn.
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub user: String,
    pub star: String,
    pub timestamp: std::time::SystemTime,
}

/// Consolidation manager — handles memory promotion and demotion.
///
/// When working memory fills up, older working memories are consolidated
/// into episodic. Episodic memories that haven't been accessed decay toward
/// semantic. High-importance semantic memories stay semantic.
pub struct ConsolidationManager {
    /// Memories pending promotion (working → episodic)
    promotion_queue: Vec<Memory>,
    /// Last consolidation time
    last_consolidation: i64,
}

impl ConsolidationManager {
    pub fn new() -> Self {
        Self {
            promotion_queue: Vec::new(),
            last_consolidation: chrono::Utc::now().timestamp(),
        }
    }

    /// Queue a memory for promotion to the next tier.
    pub fn queue_promotion(&mut self, memory: Memory) {
        self.promotion_queue.push(memory);
    }

    /// Check if it's time for consolidation.
    pub fn should_consolidate(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        let elapsed = now - self.last_consolidation;
        elapsed > 3600 // Every hour
    }

    /// Get memories ready for promotion.
    pub fn get_promotions(&mut self) -> Vec<Memory> {
        self.last_consolidation = chrono::Utc::now().timestamp();
        std::mem::take(&mut self.promotion_queue)
    }

    /// Determine if a memory should be promoted or demoted.
    pub fn should_promote(&self, memory: &Memory, access_count: usize) -> bool {
        // High access + high importance = promote toward semantic
        if memory.importance >= 0.8 && access_count >= 3 {
            return true;
        }
        // Episodic with repeated access → semantic
        if memory.domain == MemoryDomain::Episodic && access_count >= 5 {
            return true;
        }
        false
    }

    pub fn should_demote(&self, memory: &Memory, age_hours: i64) -> bool {
        // Old episodic with low importance → evict
        if memory.domain == MemoryDomain::Episodic {
            let importance_threshold = 0.3 + (age_hours as f64 / 240.0).min(0.4);
            if memory.importance < importance_threshold && age_hours > 72 {
                return true;
            }
        }
        false
    }
}

impl Default for ConsolidationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_capacities() {
        assert_eq!(MemoryTier::Working.capacity(), Some(10));
        assert_eq!(MemoryTier::Episodic.capacity(), Some(100));
        assert_eq!(MemoryTier::Semantic.capacity(), Some(500));
        assert_eq!(MemoryTier::Identity.capacity(), None);
    }

    #[test]
    fn test_decay_rates() {
        assert_eq!(MemoryTier::Identity.base_decay_rate(), 0.0);
        assert!(MemoryTier::Semantic.base_decay_rate() < MemoryTier::Episodic.base_decay_rate());
    }

    #[test]
    fn test_working_memory_add_turn() {
        let mut wm = WorkingMemory::new();
        wm.add_turn("hello", "hi there");
        assert_eq!(wm.recent_turns(1).len(), 1);
        assert_eq!(wm.recent_turns(1)[0].user, "hello");
    }
}
