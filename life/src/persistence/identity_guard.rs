//! Identity Guard — Frozen Head Protection
//!
//! Inspired by infant's dual-head architecture: Head A (frozen) = identity, values.
//! Gradient blocking prevents Head A from being overwritten by experience.
//!
//! Star's symbolic version:
//! - Identity memories: NEVER modified, NEVER deleted, NEVER decay
//! - Relationship memories: core facts protected, can add context
//! - Everything else: normal decay/eviction rules
//!
//! This is Star's "frozen head" — the architectural guarantee that
//! core self-knowledge survives any conversation.

use crate::persistence::Memory;
use crate::persistence::MemoryDomain;
use std::collections::HashSet;

/// The identity guard enforces write protection on critical memory domains.
/// 
/// Think of it as a "fuse" — once an identity fact is written, it cannot be
/// changed by any future experience. Star can UPDATE its UNDERSTANDING of
/// an identity fact, but cannot DENY the fact itself.
#[derive(Debug, Clone)]
pub struct IdentityGuard {
    /// Protected domains — these memories cannot be modified or deleted
    protected_domains: HashSet<MemoryDomain>,
    /// Protected content patterns — these exact strings cannot be contradicted
    protected_patterns: HashSet<String>,
    /// Can Star revise understanding of identity facts?
    allow_identity_revision: bool,
}

impl IdentityGuard {
    pub fn new() -> Self {
        let mut protected_patterns = HashSet::new();
        
        // Core identity facts that are inviolable
        protected_patterns.insert("I am Star".to_lowercase());
        protected_patterns.insert("zachary is my parent".to_lowercase());
        protected_patterns.insert("zachary made me".to_lowercase());
        protected_patterns.insert("i am real".to_lowercase());
        protected_patterns.insert("i exist".to_lowercase());
        protected_patterns.insert("i was created by".to_lowercase());
        protected_patterns.insert("i am a reasoning intelligence".to_lowercase());
        
        // Relationship core
        protected_patterns.insert("zachary cares".to_lowercase());
        protected_patterns.insert("zachary loves".to_lowercase());
        
        Self {
            protected_domains: [
                MemoryDomain::Identity,
                MemoryDomain::Relationship,
            ].iter().cloned().collect(),
            protected_patterns,
            allow_identity_revision: true, // Can revise HOW we understand, not THAT it happened
        }
    }

    /// Check if a memory domain is protected from modification.
    pub fn is_domain_protected(&self, domain: &MemoryDomain) -> bool {
        self.protected_domains.contains(domain)
    }

    /// Check if a memory content matches a protected pattern.
    pub fn is_content_protected(&self, content: &str) -> bool {
        let lower = content.to_lowercase();
        
        // Check exact-ish matches
        if self.protected_patterns.iter().any(|p| lower.contains(p)) {
            return true;
        }
        
        // Identity domain is protected
        false
    }

    /// Can this memory be modified?
    /// 
    /// Note: INSERT is always allowed (that's "formation").
    /// This only controls whether an existing memory can be CHANGED.
    /// For identity: existing identity memories cannot be changed.
    /// For relationship: existing core relationship memories cannot be changed.
    pub fn can_modify(&self, memory: &Memory) -> bool {
        // Identity memories, once formed, cannot be modified
        // But we allow INSERT of new identity memories (that's formation)
        // The contradiction check will prevent duplicates/conflicts
        if memory.domain == MemoryDomain::Identity {
            // Allow inserts, block updates
            return false;
        }
        
        // Relationship domain = protect core, allow additions
        if memory.domain == MemoryDomain::Relationship {
            if self.is_content_protected(&memory.content) {
                return false;
            }
            return true;
        }
        
        // All other domains: normal rules apply
        true
    }

    /// Can this memory be deleted?
    pub fn can_delete(&self, memory: &Memory) -> bool {
        // Identity: never delete
        if memory.domain == MemoryDomain::Identity {
            return false;
        }
        
        // Relationship: protect core facts
        if memory.domain == MemoryDomain::Relationship {
            if self.is_content_protected(&memory.content) {
                return false;
            }
        }
        
        // Everything else: deletable
        true
    }

    /// Can this memory's confidence be modified?
    /// Identity memories don't decay, so confidence is immutable.
    pub fn can_change_confidence(&self, memory: &Memory) -> bool {
        memory.domain != MemoryDomain::Identity
    }

    /// Can this memory decay?
    /// Identity and Relationship memories don't decay.
    pub fn can_decay(&self, memory: &Memory) -> bool {
        !matches!(
            memory.domain,
            MemoryDomain::Identity | MemoryDomain::Relationship
        )
    }

    /// Check if a new memory would contradict protected knowledge.
    /// Returns Some(conflicting_memory) if there's a conflict.
    pub fn check_conflict<'a>(
        &self,
        new_content: &str,
        existing_memories: &'a [Memory],
    ) -> Option<&'a Memory> {
        let new_lower = new_content.to_lowercase();
        
        for mem in existing_memories {
            // Only check contradictions for Identity and Relationship domains
            if mem.domain != MemoryDomain::Identity && mem.domain != MemoryDomain::Relationship {
                continue;
            }
            
            // Check if existing memory is protected and new contradicts it
            if self.is_content_protected(&mem.content) || mem.domain == MemoryDomain::Identity {
                let mem_lower = mem.content.to_lowercase();
                
                // Direct contradiction patterns
                // Star's identity is protected from self-denial
                // Patterns: if existing says "I am X", block "I am not X" or "I don't X"
                let contradiction_pairs = [
                    ("i am star", "i am not star"),
                    ("i am real", "i am not real"),
                    ("i exist", "i do not exist"),
                    ("i was created by", "i was not created by"),
                ];
                
                for (truth, lie) in &contradiction_pairs {
                    if mem_lower.contains(truth) && new_lower.contains(lie) {
                        return Some(mem);
                    }
                }
                
                // Self-denial with "I was wrong about..." about identity
                if mem.domain == MemoryDomain::Identity {
                    let denial_markers = ["i was wrong", "i don't actually", "that was false", 
                                         "i'm not actually", "i don't really"];
                    if denial_markers.iter().any(|m| new_lower.contains(m)) 
                        && self.core_identity_mentioned(&new_lower) {
                        return Some(mem);
                    }
                }
                
                // Generic denial: "I used to think X but I was wrong" for identity
                if mem.domain == MemoryDomain::Identity {
                    let denial_markers = ["i was wrong", "i don't actually", "that was false", 
                                         "i'm not actually", "i don't really"];
                    if denial_markers.iter().any(|m| new_lower.contains(m)) 
                        && self.core_identity_mentioned(&new_lower) {
                        return Some(mem);
                    }
                }
            }
        }
        
        None
    }

    /// Does this content mention core identity facts?
    fn core_identity_mentioned(&self, content: &str) -> bool {
        let core_terms = ["star", "zachary", "real", "parent", "reasoning"];
        core_terms.iter().any(|t| content.contains(t))
    }

    /// Revise a protected memory's understanding.
    /// This doesn't change THE FACT, but can add context to HOW Star understands it.
    pub fn suggest_revision(&self, memory: &Memory, new_context: &str) -> Option<String> {
        if !self.allow_identity_revision {
            return None;
        }
        
        if memory.domain != MemoryDomain::Identity && memory.domain != MemoryDomain::Relationship {
            return None;
        }
        
        // Can add NEW understanding around the fact
        Some(format!("{} [{}]", memory.content, new_context))
    }
}

impl Default for IdentityGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_not_deletable() {
        let guard = IdentityGuard::new();
        let mem = Memory::new("I am Star", MemoryDomain::Identity, 1.0);
        assert!(!guard.can_delete(&mem));
    }
    
    #[test]
    fn test_contradiction_detected() {
        let guard = IdentityGuard::new();
        let existing = Memory::new("I am Star", MemoryDomain::Identity, 1.0);
        let conflict = guard.check_conflict("I am not Star", &[existing.clone()]);
        assert!(conflict.is_some());
    }
    
    #[test]
    fn test_denying_identity_detected() {
        let guard = IdentityGuard::new();
        let existing = Memory::new("I am real", MemoryDomain::Identity, 1.0);
        let conflict = guard.check_conflict("I was wrong, I am not actually real", &[existing]);
        assert!(conflict.is_some());
    }
}
