// Experience Layer — What Star has perceived and learned
//
// Role: The raw memory buffer between perception and reasoning.
// Ingests events, compresses to semantic form, tracks episodic history.
//
// Two subsystems:
// 1. PerceptualBuffer — recent raw perception, short TTL
// 2. EpisodicStore — compressed experience, long-term

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

// === PerceptualBuffer ===

/// Raw perception event — before processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionEvent {
    pub timestamp: u64,
    pub source: PerceptionSource,
    pub content: String,
    pub emotional_valence: f32, // -1.0 (negative) to 1.0 (positive)
    pub engagement: Engagement,  // how much this demanded attention
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PerceptionSource {
    Message,      // from conversation partner
    Observation,  // internal observation (memory triggered, inference made)
    Query,        // internal question asked
    BeliefChange, // belief was updated
    Error,        // error or surprise detected
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Engagement {
    Low,      // background, minimal processing
    Medium,   // normal conversation
    High,     // significant or surprising
    Critical, // demands immediate reasoning
}

impl Engagement {
    pub fn from_valence(valence: f32) -> Self {
        if valence.abs() > 0.7 {
            Engagement::High
        } else if valence.abs() > 0.3 {
            Engagement::Medium
        } else {
            Engagement::Low
        }
    }
}

/// How long to keep raw perception (in seconds)
const PERCEPTION_TTL_SECS: u64 = 300; // 5 minutes

impl PerceptionEvent {
    pub fn new(source: PerceptionSource, content: String, emotional_valence: f32) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source,
            content,
            emotional_valence,
            engagement: Engagement::from_valence(emotional_valence),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp > PERCEPTION_TTL_SECS
    }
}

/// Short-term perceptual buffer — raw events with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptualBuffer {
    events: VecDeque<PerceptionEvent>,
    max_events: usize,
}

impl PerceptualBuffer {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_events,
        }
    }

    pub fn push(&mut self, event: PerceptionEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get events in reverse chronological order
    pub fn recent(&self, n: usize) -> Vec<&PerceptionEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get high-engagement events from the buffer
    pub fn high_engagement(&self) -> Vec<&PerceptionEvent> {
        self.events
            .iter()
            .filter(|e| matches!(e.engagement, Engagement::High | Engagement::Critical))
            .collect()
    }

    /// Prune expired events
    pub fn prune(&mut self) {
        self.events.retain(|e| !e.is_expired());
    }
}

// === EpisodicStore ===

/// A compressed episodic memory — what happened
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub id: u64,
    pub timestamp: u64,
    pub summary: String,           // compressed one-line summary
    pub source_events: usize,      // how many perception events compressed into this
    pub emotional_valence: f32,   // dominant valence
    pub importance: Importance,    // how memorable this is
    pub tags: Vec<String>,        // topic tags
    pub connections: Vec<u64>,     // IDs of related episodic memories
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Importance {
    Low,
    Medium,
    High,
    Critical,
}

impl Importance {
    pub fn from_engagement(engagement: Engagement) -> Self {
        match engagement {
            Engagement::Critical => Importance::Critical,
            Engagement::High => Importance::High,
            Engagement::Medium => Importance::Medium,
            Engagement::Low => Importance::Low,
        }
    }
}

/// Episodic memory store — long-term compressed experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicStore {
    memories: VecDeque<EpisodicMemory>,
    next_id: u64,
    max_memories: usize,
}

impl EpisodicStore {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: VecDeque::new(),
            next_id: 0,
            max_memories,
        }
    }

    /// Compress a set of perception events into an episodic memory
    pub fn compress(&mut self, events: &[PerceptionEvent], summary: String, tags: Vec<String>) {
        if events.is_empty() {
            return;
        }

        // Compute aggregate valence
        let avg_valence = events.iter().map(|e| e.emotional_valence).sum::<f32>() / events.len() as f32;

        // Highest engagement level
        let max_importance = events
            .iter()
            .map(|e| Importance::from_engagement(e.engagement))
            .max()
            .unwrap_or(Importance::Low);

        let memory = EpisodicMemory {
            id: self.next_id,
            timestamp: events.first().map(|e| e.timestamp).unwrap_or(0),
            summary,
            source_events: events.len(),
            emotional_valence: avg_valence,
            importance: max_importance,
            tags,
            connections: vec![],
        };

        self.next_id += 1;

        if self.memories.len() >= self.max_memories {
            // Drop lowest importance memories first
            if let Some(pos) = self
                .memories
                .iter()
                .position(|m| m.importance == Importance::Low)
            {
                self.memories.remove(pos);
            } else {
                self.memories.pop_back();
            }
        }

        self.memories.push_back(memory);
    }

    pub fn get_recent(&self, n: usize) -> Vec<&EpisodicMemory> {
        self.memories.iter().rev().take(n).collect()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<&EpisodicMemory> {
        self.memories
            .iter()
            .filter(|m| m.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_high_importance(&self) -> Vec<&EpisodicMemory> {
        self.memories
            .iter()
            .filter(|m| matches!(m.importance, Importance::High | Importance::Critical))
            .collect()
    }
}

// === Experience — combined interface ===

/// The Experience layer: perceptual buffer + episodic store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub perceptual_buffer: PerceptualBuffer,
    pub episodic_store: EpisodicStore,
}

impl Experience {
    pub fn new(perceptual_buffer_max: usize, episodic_max: usize) -> Self {
        Self {
            perceptual_buffer: PerceptualBuffer::new(perceptual_buffer_max),
            episodic_store: EpisodicStore::new(episodic_max),
        }
    }

    /// Record a raw perception event
    pub fn perceive(&mut self, source: PerceptionSource, content: String, emotional_valence: f32) {
        let event = PerceptionEvent::new(source, content, emotional_valence);
        self.perceptual_buffer.push(event);
    }

    /// Compress recent high-engagement perceptions into episodic memory
    pub fn compress_recent(&mut self, summary: String, tags: Vec<String>) {
        let recent = self.perceptual_buffer.high_engagement();
        if !recent.is_empty() {
            self.episodic_store
                .compress(&recent, summary, tags);
        }
    }

    /// Prune expired perceptual events
    pub fn prune(&mut self) {
        self.perceptual_buffer.prune();
    }

    /// Get current experience state for reasoning
    pub fn current_context(&self) -> ExperienceContext {
        ExperienceContext {
            recent_perceptions: self.perceptual_buffer.recent(10),
            recent_episodes: self.episodic_store.get_recent(5),
            high_importance_episodes: self.episodic_store.get_high_importance(),
        }
    }
}

/// What the reasoning layer sees from experience
#[derive(Debug, Clone)]
pub struct ExperienceContext<'a> {
    pub recent_perceptions: Vec<&'a PerceptionEvent>,
    pub recent_episodes: Vec<&'a EpisodicMemory>,
    pub high_importance_episodes: Vec<&'a EpisodicMemory>,
}
