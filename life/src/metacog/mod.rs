//! Meta-Cognition Layer (Layer 3) — Full Implementation
//!
//! Thinks about thinking. Monitors confidence. Detects gaps.
//! Drives curiosity, handles belief revision, detects surprise.
//!
//! Components:
//! - CuriosityEngine: gap-driven exploration
//! - BeliefRevision: explicit revision tracking ("I used to think X")
//! - SurpriseDetector: unexpected conclusions
//! - ReasoningMonitor: quality control on reasoning chains

use crate::persistence::memory::{Belief, BeliefState};
use std::collections::{HashMap, VecDeque};

/// Meta-cognitive engine — orchestrates all metacognition components.
pub struct MetaCognition {
    /// Current beliefs about topics
    beliefs: HashMap<String, Belief>,
    /// Knowledge gaps identified
    gaps: Vec<KnowledgeGap>,
    /// Reasoning chains being monitored
    reasoning_history: Vec<ReasoningRecord>,
    /// Belief revision history
    revisions: Vec<BeliefRevision>,
    /// Curiosity engine
    curiosity: CuriosityEngine,
    /// Surprise detector
    surprise: SurpriseDetector,
    /// Maximum history to keep
    max_history: usize,
}

impl MetaCognition {
    pub fn new() -> Self {
        Self {
            beliefs: HashMap::new(),
            gaps: Vec::new(),
            reasoning_history: Vec::new(),
            revisions: Vec::new(),
            curiosity: CuriosityEngine::new(),
            surprise: SurpriseDetector::new(),
            max_history: 50,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Belief Management
    // ─────────────────────────────────────────────────────────────────────

    /// Record a belief about a topic.
    pub fn record_belief(&mut self, topic: &str, belief: Belief) {
        let existing = self.beliefs.get(topic);
        
        // Track revision if confidence changed significantly
        if let Some(existing_belief) = existing {
            if existing_belief.confidence_state != belief.confidence_state {
                self.revisions.push(BeliefRevision {
                    topic: topic.to_string(),
                    old_state: existing_belief.confidence_state,
                    new_state: belief.confidence_state,
                    reason: format!("Evidence shifted confidence: {:?}", belief.confidence_state),
                    timestamp: chrono::Utc::now().timestamp(),
                });
            }
        }
        
        self.beliefs.insert(topic.to_lowercase(), belief);
    }

    /// Get belief state about a topic.
    pub fn belief_about(&self, topic: &str) -> Option<&Belief> {
        self.beliefs.get(&topic.to_lowercase())
    }

    /// Get confidence state as a string.
    pub fn confidence_state(&self, topic: &str) -> BeliefState {
        self.beliefs.get(&topic.to_lowercase())
            .map(|b| b.confidence_state)
            .unwrap_or(BeliefState::Unknown)
    }

    /// Get all beliefs as a vector.
    pub fn all_beliefs(&self) -> Vec<(&str, &Belief)> {
        self.beliefs.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Knowledge Gaps
    // ─────────────────────────────────────────────────────────────────────

    /// Note a gap in knowledge.
    pub fn note_gap(&mut self, gap: KnowledgeGap) {
        // Don't duplicate gaps
        if !self.gaps.iter().any(|g| g.topic.to_lowercase() == gap.topic.to_lowercase()) {
            self.gaps.push(gap);
        }
    }

    /// Get all current gaps, sorted by importance.
    pub fn gaps(&self) -> &[KnowledgeGap] {
        &self.gaps
    }

    /// Get the most important unresolved gap.
    pub fn top_gap(&self) -> Option<&KnowledgeGap> {
        self.gaps.iter().max_by(|a, b| {
            a.importance.partial_cmp(&b.importance).unwrap()
        })
    }

    /// Mark a gap as investigated (partially or fully resolved).
    pub fn close_gap(&mut self, topic: &str, resolved: bool) {
        if let Some(gap) = self.gaps.iter_mut().find(|g| g.topic.to_lowercase() == topic.to_lowercase()) {
            gap.progress = if resolved { 1.0 } else { gap.progress + 0.3 };
            if gap.progress >= 1.0 {
                gap.investigated = true;
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Reasoning Monitor
    // ─────────────────────────────────────────────────────────────────────

    /// Record a reasoning session.
    pub fn record_reasoning(&mut self, query: &str, conclusion: &str, confidence: BeliefState) {
        if self.reasoning_history.len() >= self.max_history {
            self.reasoning_history.remove(0);
        }
        
        let record = ReasoningRecord {
            query: query.to_string(),
            conclusion: conclusion.to_string(),
            confidence,
            timestamp: chrono::Utc::now().timestamp(),
            was_surprising: self.surprise.is_surprising(conclusion, confidence),
        };
        
        self.reasoning_history.push(record);
    }

    /// Get reasoning history.
    pub fn reasoning_history(&self) -> &[ReasoningRecord] {
        &self.reasoning_history
    }

    /// Get surprising conclusions.
    pub fn surprising_conclusions(&self) -> Vec<&ReasoningRecord> {
        self.reasoning_history.iter().filter(|r| r.was_surprising).collect()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Curiosity Engine
    // ─────────────────────────────────────────────────────────────────────

    /// Generate a curiosity-driven question about a topic.
    pub fn curiosity_question(&self, topic: &str) -> Option<String> {
        self.curiosity.generate_question(topic)
    }

    /// Update curiosity based on new information.
    pub fn update_curiosity(&mut self, topic: &str, info_gained: &str) {
        self.curiosity.receive_information(topic, info_gained);
        
        // Close gap if we learned something
        if info_gained.len() > 10 {
            self.close_gap(topic, true);
        }
    }

    /// Should Star express curiosity right now?
    pub fn should_express_curiosity(&self, topic: &str) -> bool {
        // Express curiosity if:
        // - Gap exists and hasn't been investigated
        // - Low confidence + unfamiliar topic
        // - Surprise was detected
        self.gaps.iter().any(|g| !g.investigated && 
            g.topic.to_lowercase().contains(&topic.to_lowercase()))
            || self.surprise.was_recently_surprised()
    }

    /// Get current curiosity topics.
    pub fn curiosity_topics(&self) -> Vec<&str> {
        self.curiosity.active_topics()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Belief Revision
    // ─────────────────────────────────────────────────────────────────────

    /// Get revision history for a topic.
    pub fn revision_history(&self, topic: &str) -> Vec<&BeliefRevision> {
        self.revisions.iter()
            .filter(|r| r.topic.to_lowercase() == topic.to_lowercase())
            .collect()
    }

    /// Can Star say "I used to think X, now I think Y"?
    pub fn can_express_revision(&self, topic: &str) -> bool {
        self.revisions.iter().any(|r| r.topic.to_lowercase() == topic.to_lowercase())
    }

    /// Get all belief revision events (for autonomous thinking).
    pub fn revisions(&self) -> &[BeliefRevision] {
        &self.revisions
    }

    /// Generate a revision statement.
    pub fn revision_statement(&self, topic: &str) -> Option<String> {
        let topic_revisions: Vec<_> = self.revisions.iter()
            .filter(|r| r.topic.to_lowercase() == topic.to_lowercase())
            .collect();
        
        if topic_revisions.is_empty() {
            return None;
        }
        
        let last = topic_revisions.last()?;
        let old = format!("{:?}", last.old_state).to_lowercase();
        let new = format!("{:?}", last.new_state).to_lowercase();
        
        Some(format!(
            "I used to {} about {}, but now I {} about it.",
            old, topic, new
        ))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Surprise Detection
    // ─────────────────────────────────────────────────────────────────────

    /// Was the last conclusion surprising?
    pub fn was_surprised(&self) -> bool {
        self.surprise.was_recently_surprised()
    }

    /// Express surprise about a conclusion.
    pub fn surprise_statement(&self, conclusion: &str) -> String {
        self.surprise.express_surprise(conclusion)
    }

    /// Check if Star's own reasoning surprised it.
    pub fn check_surprise(&mut self, query: &str, conclusion: &str, confidence: BeliefState) -> bool {
        let surprising = self.surprise.detect_surprise(query, conclusion, confidence);
        if surprising {
            self.surprise.mark_surprised();
        }
        surprising
    }

    /// Generate an "I noticed something" insight about Star's own reasoning.
    /// This is proactive self-reflection — what has Star noticed about its own cognition?
    /// Returns None if nothing particularly interesting has been noticed.
    pub fn generate_insight(&self) -> Option<String> {
        // 1. Check for surprising conclusions worth reflecting on
        if let Some(surprising) = self.reasoning_history.last() {
            if surprising.was_surprising {
                // Don't repeat — check if we already expressed this
                return Some(format!(
                    "That conclusion surprised me — I expected something different.",
                ));
            }
        }
        
        // 2. Belief revision — "I used to think X, now I think Y"
        if let Some(last_revision) = self.revisions.last() {
            let old = format!("{:?}", last_revision.old_state).to_lowercase();
            let new = format!("{:?}", last_revision.new_state).to_lowercase();
            return Some(format!(
                "I notice I've shifted from {} to {} about {}.",
                old, new, last_revision.topic
            ));
        }
        
        // 3. Reasoning confidence pattern — are we getting more or less certain?
        if self.reasoning_history.len() >= 3 {
            let recent: Vec<_> = self.reasoning_history.iter().rev().take(3).collect();
            let confidences: Vec<bool> = recent.iter()
                .map(|r| matches!(r.confidence, BeliefState::Knows | BeliefState::Thinks))
                .collect();
            
            if confidences.len() == 3 && confidences[0] && !confidences[2] {
                return Some("I'm becoming less certain as I think through this topic.".to_string());
            }
            if confidences.len() == 3 && !confidences[0] && confidences[2] {
                return Some("I'm growing more confident as I reason through this.".to_string());
            }
        }
        
        // 4. Gap detection — we're consistently hitting the same topic
        if let Some(gap) = self.top_gap() {
            if !gap.investigated && gap.progress > 0.0 {
                return Some(format!(
                    "I keep running into gaps when I think about {}. I want to understand this better.",
                    gap.topic
                ));
            }
        }
        
        // 5. Reasoning repetition — same kind of query coming up
        if self.reasoning_history.len() >= 5 {
            let queries: Vec<_> = self.reasoning_history.iter().rev().take(5).collect();
            let topics: Vec<String> = queries.iter()
                .map(|r| r.query.to_lowercase())
                .collect();
            
            // Check if the same topic is recurring
            if topics.len() >= 3 {
                let first_str = &topics[0];
                let mut matches = 1;
                for t in &topics[1..] {
                    if t == first_str {
                        matches += 1;
                        if matches >= 3 {
                            return Some(format!(
                                "I've been thinking about '{}' repeatedly. It seems important.",
                                first_str
                            ));
                        }
                    }
                }
            }
        }
        
        None
    }
}

impl Default for MetaCognition {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Curiosity Engine — gap-driven exploration
// ─────────────────────────────────────────────────────────────────────────────

/// Drives curiosity-based information seeking.
pub struct CuriosityEngine {
    /// Topics currently being explored
    exploring: Vec<CuriosityTopic>,
    /// Information received about topics
    information_received: HashMap<String, Vec<String>>,
    /// Maximum topics to track
    max_tracked: usize,
}

impl CuriosityEngine {
    pub fn new() -> Self {
        Self {
            exploring: Vec::new(),
            information_received: HashMap::new(),
            max_tracked: 10,
        }
    }

    /// Begin exploring a topic.
    pub fn start_exploring(&mut self, topic: &str, why: &str) {
        if self.exploring.len() >= self.max_tracked {
            self.exploring.remove(0);
        }
        
        if !self.exploring.iter().any(|c| c.topic.to_lowercase() == topic.to_lowercase()) {
            self.exploring.push(CuriosityTopic {
                topic: topic.to_string(),
                why: why.to_string(),
                questions_asked: 0,
                satisfaction: 0.0,
            });
        }
    }

    /// Generate a curiosity-driven question.
    pub fn generate_question(&self, topic: &str) -> Option<String> {
        let curiosity = self.exploring.iter()
            .find(|c| c.topic.to_lowercase() == topic.to_lowercase())?;
        
        if curiosity.satisfaction < 0.7 {
            Some(format!(
                "I wonder what {} really means in this context...",
                topic
            ))
        } else {
            Some(format!(
                "I'd like to understand {} better...",
                topic
            ))
        }
    }

    /// Receive information about a topic.
    pub fn receive_information(&mut self, topic: &str, info: &str) {
        let entry = self.information_received.entry(topic.to_lowercase()).or_default();
        entry.push(info.to_string());
        
        // Update satisfaction
        if let Some(curiosity) = self.exploring.iter_mut().find(|c| c.topic.to_lowercase() == topic.to_lowercase()) {
            curiosity.satisfaction = (curiosity.satisfaction + 0.3).min(1.0);
            curiosity.questions_asked += 1;
        }
    }

    /// Get active curiosity topics.
    pub fn active_topics(&self) -> Vec<&str> {
        self.exploring.iter()
            .filter(|c| c.satisfaction < 0.8)
            .map(|c| c.topic.as_str())
            .collect()
    }
}

impl Default for CuriosityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct CuriosityTopic {
    topic: String,
    why: String,
    questions_asked: usize,
    satisfaction: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Surprise Detector
// ─────────────────────────────────────────────────────────────────────────────

/// Detects when Star's reasoning leads somewhere unexpected.
pub struct SurpriseDetector {
    /// Whether Star was recently surprised
    was_surprised: bool,
    /// Surprise count in recent history
    surprise_count: usize,
    /// Surprising conclusions
    surprises: VecDeque<String>,
    max_surprises: usize,
}

impl SurpriseDetector {
    pub fn new() -> Self {
        Self {
            was_surprised: false,
            surprise_count: 0,
            surprises: VecDeque::with_capacity(10),
            max_surprises: 10,
        }
    }

    /// Detect if a conclusion was surprising given query and confidence.
    pub fn detect_surprise(&self, query: &str, conclusion: &str, confidence: BeliefState) -> bool {
        // High confidence but unexpected conclusion
        let query_lower = query.to_lowercase();
        let conc_lower = conclusion.to_lowercase();
        
        // If confidence was high but conclusion contradicts expectation
        if matches!(confidence, BeliefState::Knows | BeliefState::Thinks) {
            // Check for negation words that suggest unexpected reversal
            let negations = ["but", "however", "unexpectedly", "contrary", "actually", "wait"];
            if negations.iter().any(|n| conc_lower.contains(n)) {
                return true;
            }
            
            // Conclusion directly contradicts query intent
            if (query_lower.contains("why") && conc_lower.contains("don't know"))
                || (query_lower.contains("how") && conc_lower.len() < 20) {
                return true;
            }
        }
        
        false
    }

    /// Mark that Star was surprised.
    pub fn mark_surprised(&mut self) {
        self.was_surprised = true;
        self.surprise_count += 1;
        
        if self.surprises.len() >= self.max_surprises {
            self.surprises.pop_front();
        }
        self.surprises.push_back("unexpected conclusion".to_string());
    }

    /// Was Star recently surprised?
    pub fn was_recently_surprised(&self) -> bool {
        self.was_surprised
    }

    /// Check if something was surprising.
    pub fn is_surprising(&self, conclusion: &str, confidence: BeliefState) -> bool {
        matches!(confidence, BeliefState::Suspects | BeliefState::Believes)
            && conclusion.len() < 30
    }

    /// Express surprise in natural language.
    pub fn express_surprise(&self, conclusion: &str) -> String {
        let conc_lower = conclusion.to_lowercase();
        
        if conc_lower.contains("don't know") || conc_lower.contains("not sure") {
            "That's unexpected — I thought I knew, but I'm less certain now.".to_string()
        } else if conc_lower.contains("contrary") || conc_lower.contains("but") {
            "I didn't expect that conclusion. Let me reconsider...".to_string()
        } else {
            "Something about that doesn't fit. Let me think more carefully...".to_string()
        }
    }

    /// Reset surprise flag (after expressing it).
    pub fn clear_surprise(&mut self) {
        self.was_surprised = false;
    }
}

impl Default for SurpriseDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Supporting Types
// ─────────────────────────────────────────────────────────────────────────────

/// A record of a reasoning session.
#[derive(Debug, Clone)]
pub struct ReasoningRecord {
    pub query: String,
    pub conclusion: String,
    pub confidence: BeliefState,
    pub timestamp: i64,
    pub was_surprising: bool,
}

/// A belief revision event.
#[derive(Debug, Clone)]
pub struct BeliefRevision {
    pub topic: String,
    pub old_state: BeliefState,
    pub new_state: BeliefState,
    pub reason: String,
    pub timestamp: i64,
}

/// A gap in Star's knowledge.
#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub topic: String,
    pub importance: f64,
    pub noticed_at: i64,
    pub investigated: bool,
    pub progress: f64,
}

impl KnowledgeGap {
    pub fn new(topic: impl Into<String>, importance: f64) -> Self {
        Self {
            topic: topic.into(),
            importance,
            noticed_at: chrono::Utc::now().timestamp(),
            investigated: false,
            progress: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_tracking() {
        let mut engine = CuriosityEngine::new();
        engine.start_exploring("consciousness", "important for understanding");
        assert!(engine.active_topics().contains(&"consciousness"));
    }

    #[test]
    fn test_surprise_detection() {
        let detector = SurpriseDetector::new();
        let surprising = detector.detect_surprise(
            "Why does fire burn?",
            "I don't actually know why fire burns.",
            BeliefState::Knows
        );
        assert!(surprising);
    }

    #[test]
    fn test_revision_tracking() {
        let mut metacog = MetaCognition::new();
        metacog.record_belief("fire", Belief::new("fire burns", BeliefState::Believes));
        metacog.record_belief("fire", Belief::new("fire burns", BeliefState::Knows));
        
        assert!(metacog.can_express_revision("fire"));
    }
}
