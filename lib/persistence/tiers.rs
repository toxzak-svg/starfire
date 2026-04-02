//! Memory Tiers — working, short-term, long-term memory management

use super::memory::Memory;

/// Memory tier levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    Working,   // Immediate — active reasoning
    ShortTerm, // Recent — within current session
    LongTerm,  // Persistent — survives sessions
}

/// Working memory buffer.
#[derive(Debug, Default)]
pub struct WorkingMemory {
    items: Vec<Memory>,
    capacity: usize,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self { items: Vec::new(), capacity }
    }

    pub fn push(&mut self, mem: Memory) {
        if self.items.len() >= self.capacity {
            self.items.remove(0);
        }
        self.items.push(mem);
    }
}

/// Consolidation manager — promotes memories between tiers.
#[derive(Debug)]
pub struct ConsolidationManager {
    threshold: f64,
}

impl ConsolidationManager {
    pub fn new() -> Self {
        Self { threshold: 0.7 }
    }

    pub fn should_promote(&self, mem: &Memory) -> bool {
        mem.importance >= self.threshold && mem.access_count >= 3
    }
}

impl Default for ConsolidationManager {
    fn default() -> Self {
        Self::new()
    }
}
