//! Context Layer — Symbolic Ring Attractor
//!
//! Provides persistent conversational state across turns.
//! Inspired by SRMoE's ring attractor: a continuous state space that influences
//! every processing step and maintains phase across time.
//!
//! Unlike a neural ring attractor (continuous, gradient-based), this is a
//! *symbolic ring* — discrete states connected by reasoning transitions.
//! The "ring" is a structured context record that persists across conversation.
//!
//! Key insight from SRMoE: routing decisions depend on BOTH current input
//! AND ring phase. Star's symbolic ring: reasoning mode depends on BOTH query
//! AND conversation context.

pub mod ring;
pub mod modes;
pub mod fusion;

pub use ring::{RingState, TopicPhase, OpenQuestion};
pub use modes::ReasoningMode;
pub use fusion::ContextFuser;

/// The complete contextual state for a Star reasoning turn.
/// 
/// Produced by the ring attractor and consumed by the reasoning engine
/// and conversation layer.
#[derive(Debug, Clone)]
pub struct ContextState {
    /// The ring state (persistent across conversation)
    pub ring: RingState,
    /// Current reasoning mode (discrete, like a VQ codebook)
    pub mode: ReasoningMode,
    /// Fusion weight: how much to trust the ring vs. the query
    /// (0.0 = ignore context, 1.0 = heavily context-dependent)
    pub context_weight: f64,
    /// Current working memory (last N conversation turns)
    pub working_memory: Vec<WorkingTurn>,
    /// What Star was most recently curious about
    pub curiosity_cursor: Option<String>,
    /// How many turns since the topic last changed
    pub turns_on_topic: usize,
}

impl ContextState {
    /// Create initial context state for a new session.
    pub fn fresh() -> Self {
        Self {
            ring: RingState::new(),
            mode: ReasoningMode::EXPLORING,
            context_weight: 0.3,
            working_memory: Vec::with_capacity(10),
            curiosity_cursor: None,
            turns_on_topic: 0,
        }
    }

    /// Record a conversation turn in working memory.
    pub fn record_turn(&mut self, user: &str, star: &str) {
        if self.working_memory.len() >= 10 {
            self.working_memory.remove(0);
        }
        self.working_memory.push(WorkingTurn {
            user: user.to_string(),
            star: star.to_string(),
            ring_phase_at_time: self.ring.topic_phase.clone(),
        });
    }

    /// Check if a question from the user is already open.
    pub fn is_question_open(&self, question: &str) -> bool {
        let q_lower = question.to_lowercase();
        self.ring.open_questions.iter().any(|q| {
            q.topic.to_lowercase() == q_lower
        })
    }

    /// Get a summary of what Star was last thinking about.
    pub fn last_focus(&self) -> Option<&str> {
        self.ring.topic_history.last().map(|s| s.as_str())
    }
}

/// A single conversation turn in working memory.
#[derive(Debug, Clone)]
pub struct WorkingTurn {
    pub user: String,
    pub star: String,
    /// The ring phase when this turn occurred
    ring_phase_at_time: TopicPhase,
}
