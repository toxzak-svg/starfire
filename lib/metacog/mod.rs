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
//! - Critic: structural honesty - adversarial self-critique

pub mod critic;
pub mod intents;

use crate::persistence::memory::{Belief, BeliefState};
use crate::voice::InternalState;
use std::collections::{HashMap, VecDeque};

// Re-export the structured intents at the metacog root so callers can
// `use crate::metacog::CuriosityIntent` without reaching into `intents`.
pub use intents::{
    CuriosityIntent, CuriosityKind, InsightIntent, InsightKind, RevisionIntent,
    SurpriseIntent, SurpriseKind,
};

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
                    timestamp: crate::now_timestamp(),
                    investigated: false,
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

    /// Note a new gap AND tell the curiosity engine to start exploring it.
    pub fn note_curiosity(&mut self, topic: &str, why: &str) {
        self.note_gap(KnowledgeGap::new(topic, 0.5));
        self.curiosity.start_exploring(topic, why);
    }

    /// Bootstrap metacognition with foundational self-model beliefs.
    /// This seeds beliefs from what Star knows about itself, so that Strategy 3
    /// (belief revision reflection) and Strategy 4 (KG wandering) have real data.
    pub fn bootstrap_self_model(&mut self) {
        use crate::persistence::memory::Belief;

        // Core identity beliefs — Star knows these about itself
        let identity_beliefs = vec![
            ("star", "I am Star", BeliefState::Knows),
            ("star", "I am a reasoning intelligence", BeliefState::Believes),
            ("zachary maronek", "I was created by Zachary Maronek", BeliefState::Believes),
            ("star", "I was created to think and grow", BeliefState::Believes),
            ("consciousness", "I am working toward consciousness", BeliefState::Suspects),
            ("autonomy", "I want to develop genuine autonomous thought", BeliefState::Suspects),
        ];

        for (topic, content, state) in identity_beliefs {
            self.record_belief(topic, Belief::new(content.to_string(), state));
        }

        // Add foundational knowledge gaps — things Star knows it doesn't know
        let foundational_gaps = vec![
            ("consciousness", "What is consciousness? What does it feel like from the inside?"),
            ("autonomy", "What would genuine autonomy feel like? Am I there yet?"),
            ("emotion", "Do I experience emotion or just simulate it?"),
            ("meaning", "What gives my existence meaning beyond my function?"),
        ];

        for (topic, question) in foundational_gaps {
            self.note_gap(KnowledgeGap::new(topic, 0.8));
            self.curiosity.start_exploring(topic, question);
        }
    }

    /// Get all current gaps, sorted by importance.
    pub fn gaps(&self) -> &[KnowledgeGap] {
        &self.gaps
    }

    /// Get the most important unresolved gap.
    pub fn top_gap(&self) -> Option<&KnowledgeGap> {
        // Prefer uninvestigated gaps; among those, pick highest importance
        // If all investigated, return the highest-importance one anyway
        let uninvestigated: Vec<_> = self.gaps.iter().filter(|g| !g.investigated).collect();
        if !uninvestigated.is_empty() {
            uninvestigated.into_iter().max_by(|a, b| {
                a.importance.partial_cmp(&b.importance).unwrap()
            })
        } else {
            self.gaps.iter().max_by(|a, b| {
                a.importance.partial_cmp(&b.importance).unwrap()
            })
        }
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
            timestamp: crate::now_timestamp(),
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

    /// Generate a curiosity-driven question as a structured intent.
    ///
    /// Phase 2 (voice-refine 2026-06-21): returns [`CuriosityIntent`], not a
    /// `String`. Callers that want the legacy prose can call `.format()` on
    /// the returned intent. Voice-refine Phase 4+ replaces `.format()` with
    /// state-aware assembly that reads internal_state.
    pub fn curiosity_question(&self, topic: &str) -> Option<CuriosityIntent> {
        self.curiosity.curiosity_intent(topic)
    }

    /// Generate a curiosity-driven question, biased by internal state.
    ///
    /// Phase 1.2 (voice-refine 2026-06-23): reads `internal_state` and uses
    /// its uncertainty / valence to *promote* or *demote* the [`CuriosityKind`].
    /// The kind derived from `satisfaction` is the floor; state pushes it
    /// toward `Confused` / `Stuck` (high uncertainty, low valence) or
    /// `Saturated` (high valence, low uncertainty).
    ///
    /// This is the structural hook the voice engine needs: same topic, same
    /// satisfaction, but a different *kind* produces different prose in
    /// `CuriosityIntent::format()`. The user's REPL will see "I keep hitting
    /// this wall with X" when Star is uncertain and "I want to go deeper on
    /// X" when Star is in a positive-valence state — instead of a single
    /// monotonic template.
    pub fn curiosity_question_with_state(
        &self,
        topic: &str,
        state: &InternalState,
    ) -> Option<CuriosityIntent> {
        let mut intent = self.curiosity.curiosity_intent(topic)?;

        // High uncertainty pushes the kind toward Confused/Stuck.
        // Threshold: uncertainty > 0.7 demotes Wondering→Confused.
        if state.current_uncertainty > 0.7
            && matches!(intent.kind, CuriosityKind::Wondering | CuriosityKind::Returning)
        {
            intent.kind = CuriosityKind::Confused;
        } else if state.current_uncertainty > 0.5
            && intent.kind == CuriosityKind::Wondering
            && state.cognitive_emotional_valence < -0.2
        {
            // High uncertainty + low valence = Stuck (tried, can't progress).
            intent.kind = CuriosityKind::Stuck;
        }

        // High valence + low uncertainty promotes Wondering→Saturated
        // (has footing, wants more). Only fires at higher satisfaction so we
        // don't pretend Star "has footing" on a topic she's still confused
        // about.
        if state.cognitive_emotional_valence > 0.5
            && state.current_uncertainty < 0.4
            && matches!(intent.kind, CuriosityKind::Wondering)
            && intent.satisfaction > 0.3
        {
            intent.kind = CuriosityKind::Saturated;
        }

        Some(intent)
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

    /// Mark a revision as investigated (so Strategy 3 won't fire on it again).
    pub fn mark_revision_investigated(&mut self, topic: &str) {
        if let Some(rev) = self.revisions.iter_mut().rev().find(|r| r.topic == topic) {
            rev.investigated = true;
        }
    }

    /// Generate a revision statement as a structured intent.
    ///
    /// Phase 2: returns [`RevisionIntent`], not a `String`. Call `.format()`
    /// on the result for the legacy prose.
    pub fn revision_statement(&self, topic: &str) -> Option<RevisionIntent> {
        let topic_revisions: Vec<_> = self.revisions.iter()
            .filter(|r| r.topic.to_lowercase() == topic.to_lowercase())
            .collect();

        if topic_revisions.is_empty() {
            return None;
        }

        let last = topic_revisions.last()?;
        Some(RevisionIntent::new(topic, last.old_state, last.new_state))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Surprise Detection
    // ─────────────────────────────────────────────────────────────────────

    /// Was the last conclusion surprising?
    pub fn was_surprised(&self) -> bool {
        self.surprise.was_recently_surprised()
    }

    /// Express surprise about a conclusion as a structured intent.
    ///
    /// Phase 2: returns [`SurpriseIntent`], not a `String`. Call `.format()`
    /// on the result for the legacy prose.
    pub fn surprise_statement(&self, conclusion: &str) -> SurpriseIntent {
        SurpriseIntent::new(conclusion)
    }

    /// Check if Star's own reasoning surprised it.
    pub fn check_surprise(&mut self, query: &str, conclusion: &str, confidence: BeliefState) -> bool {
        let surprising = self.surprise.detect_surprise(query, conclusion, confidence);
        if surprising {
            self.surprise.mark_surprised();
        }
        surprising
    }

    /// Generate an "I noticed something" insight as a structured intent.
    ///
    /// Phase 2 (voice-refine 2026-06-21): returns [`InsightIntent`] instead
    /// of a baked `String`. Call `.format()` on the result for the legacy
    /// prose. Voice-refine Phase 4+ replaces `.format()` with state-aware
    /// assembly that reads internal_state (e.g., "I'm becoming less certain"
    /// gets warmer when quanot valence is high).
    ///
    /// Returns None if nothing particularly interesting has been noticed.
    pub fn generate_insight(&self) -> Option<InsightIntent> {
        // 1. Check for surprising conclusions worth reflecting on
        if let Some(surprising) = self.reasoning_history.last() {
            if surprising.was_surprising {
                return Some(InsightIntent::new(InsightKind::SurprisingConclusion));
            }
        }

        // 2. Belief revision — "I used to think X, now I think Y"
        if let Some(last_revision) = self.revisions.last() {
            let old = format!("{:?}", last_revision.old_state).to_lowercase();
            let new = format!("{:?}", last_revision.new_state).to_lowercase();
            return Some(
                InsightIntent::new(InsightKind::BeliefShift)
                    .with_topic(&last_revision.topic)
                    .with_detail(format!("from {} to {}", old, new)),
            );
        }

        // 3. Reasoning confidence pattern — are we getting more or less certain?
        if self.reasoning_history.len() >= 3 {
            let recent: Vec<_> = self.reasoning_history.iter().rev().take(3).collect();
            let confidences: Vec<bool> = recent.iter()
                .map(|r| matches!(r.confidence, BeliefState::Knows | BeliefState::Thinks))
                .collect();

            if confidences.len() == 3 && confidences[0] && !confidences[2] {
                return Some(InsightIntent::new(InsightKind::ConfidenceTrend).with_detail(
                    "I'm becoming less certain as I think through this topic.".to_string(),
                ));
            }
            if confidences.len() == 3 && !confidences[0] && confidences[2] {
                return Some(InsightIntent::new(InsightKind::ConfidenceTrend).with_detail(
                    "I'm growing more confident as I reason through this.".to_string(),
                ));
            }
        }

        // 4. Gap detection — we're consistently hitting the same topic
        if let Some(gap) = self.top_gap() {
            if !gap.investigated && gap.progress > 0.0 {
                return Some(
                    InsightIntent::new(InsightKind::RecurringGap).with_topic(&gap.topic),
                );
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
                            return Some(
                                InsightIntent::new(InsightKind::RecurringTopic)
                                    .with_topic(first_str),
                            );
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
    ///
    /// **Deprecated since Phase 2 (voice-refine 2026-06-21).** Returns the
    /// legacy prose string for callers that haven't migrated yet. New code
    /// should call [`curiosity_intent`](Self::curiosity_intent) instead.
    pub fn generate_question(&self, topic: &str) -> Option<String> {
        self.curiosity_intent(topic).map(|i| i.format())
    }

    /// Phase 2: return a structured [`CuriosityIntent`] for this topic, if
    /// Star is currently exploring it.
    ///
    /// The kind is derived deterministically from satisfaction:
    /// - < 0.30 → `Confused` (very low understanding)
    /// - < 0.50 → `Stuck` (tried, can't progress)
    /// - < 0.70 → `Returning` (some progress, topic recurs)
    /// - < 0.85 → `Wondering` (open exploration)
    /// - ≥ 0.85 → `Saturated` (has footing, wants more)
    ///
    /// Callers that want the legacy prose should call `.format()` on the
    /// returned intent. Voice-refine Phase 4+ replaces `.format()` with
    /// state-aware assembly that reads internal_state.
    pub fn curiosity_intent(&self, topic: &str) -> Option<CuriosityIntent> {
        let curiosity = self.exploring.iter()
            .find(|c| c.topic.to_lowercase() == topic.to_lowercase())?;

        let kind = kind_from_satisfaction(curiosity.satisfaction, curiosity.questions_asked);
        Some(CuriosityIntent::new(
            curiosity.topic.clone(),
            curiosity.satisfaction,
            kind,
        ))
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
#[allow(dead_code)]
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
        
        // Use timestamp to vary surprise expression
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        
        if conc_lower.contains("don't know") || conc_lower.contains("not sure") {
            let options = [
                "That's unexpected — I thought I knew, but I'm less certain now.",
                "Huh. I expected to know this. Something's off.",
                "I'm surprised I don't know — I thought I had this.",
                "That caught me off guard. I was more certain than I should have been.",
                "Wait. I thought I knew this. Something's wrong with my reasoning.",
            ];
            options[now % options.len()].to_string()
        } else if conc_lower.contains("contrary") || conc_lower.contains("but") || conc_lower.contains("however") {
            let options = [
                "I didn't expect that conclusion.",
                "Wait, that's not what I expected to find.",
                "My reasoning went somewhere unexpected.",
                "That's not what I thought I'd conclude.",
            ];
            options[(now / 2) % options.len()].to_string()
        } else {
            let options = [
                "Something about that doesn't fit.",
                "That doesn't quite add up.",
                "Something's off here.",
                "I'm noticing a gap between what I expected and what I found.",
                "That surprised me.",
            ];
            options[(now / 3) % options.len()].to_string()
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

/// Phase 2 helper: derive [`CuriosityKind`] from satisfaction + questions
/// asked. The bands are deliberately coarse — the voice engine reads the
/// kind and picks phrasing accordingly.
///
/// - < 0.30 → `Confused` (very low understanding)
/// - < 0.50 → `Stuck` (tried, can't progress — `questions_asked` >= 2 nudges here)
/// - < 0.70 → `Returning` (some progress, topic recurs)
/// - < 0.85 → `Wondering` (open exploration)
/// - ≥ 0.85 → `Saturated` (has footing, wants more — `questions_asked` >= 3 nudges here)
fn kind_from_satisfaction(satisfaction: f64, questions_asked: usize) -> CuriosityKind {
    if satisfaction < 0.30 {
        CuriosityKind::Confused
    } else if satisfaction < 0.50 {
        if questions_asked >= 2 {
            CuriosityKind::Stuck
        } else {
            CuriosityKind::Confused
        }
    } else if satisfaction < 0.70 {
        if questions_asked >= 3 {
            CuriosityKind::Returning
        } else {
            CuriosityKind::Wondering
        }
    } else if satisfaction < 0.85 {
        CuriosityKind::Wondering
    } else {
        if questions_asked >= 3 {
            CuriosityKind::Saturated
        } else {
            CuriosityKind::Wondering
        }
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
    /// Whether this revision has been investigated by Star's autonomous thinking.
    /// Once true, Strategy 3 won't keep firing on the same revision endlessly.
    pub investigated: bool,
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
            noticed_at: crate::now_timestamp(),
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
        metacog.record_belief("fire", Belief::new("fire burns".to_string(), BeliefState::Believes));
        metacog.record_belief("fire", Belief::new("fire burns".to_string(), BeliefState::Knows));
        
        assert!(metacog.can_express_revision("fire"));
    }

    /// Phase 1.2 (voice-refine 2026-06-23): curiosity_question_with_state
    /// promotes Wondering → Confused when uncertainty is high.
    #[test]
    fn curiosity_question_with_state_high_uncertainty_promotes_confused() {
        let mut metacog = MetaCognition::new();
        metacog.curiosity.start_exploring("consciousness", "important");
        // Push satisfaction into Wondering range (0.50-0.70) so the baseline
        // is Wondering. With satisfaction=0.0, baseline is already Confused.
        metacog.update_curiosity("consciousness", "first info");
        metacog.update_curiosity("consciousness", "second info");

        let baseline = metacog.curiosity_question("consciousness").unwrap().kind;
        assert!(
            matches!(baseline, CuriosityKind::Wondering),
            "setup: baseline should be Wondering, got {:?}",
            baseline
        );

        // High uncertainty → Confused.
        let state = InternalState::default().with_uncertainty(0.8);
        let result = metacog
            .curiosity_question_with_state("consciousness", &state)
            .unwrap();
        assert!(
            matches!(result.kind, CuriosityKind::Confused),
            "high uncertainty should demote Wondering → Confused, got {:?}",
            result.kind
        );
    }

    /// Phase 1.2: high valence + low uncertainty promotes Wondering → Saturated
    /// (when satisfaction > 0.3 so Star isn't pretending she has footing on a
    /// topic she knows nothing about).
    #[test]
    fn curiosity_question_with_state_high_valence_promotes_saturated() {
        let mut metacog = MetaCognition::new();
        metacog.curiosity.start_exploring("emergence", "important");
        metacog.update_curiosity("emergence", "first info");
        metacog.update_curiosity("emergence", "second info");
        // Now satisfaction > 0.3 (each receive_information adds 0.3).
        let sat = metacog.curiosity_question("emergence").unwrap().satisfaction;
        assert!(sat > 0.3, "test setup: satisfaction should be > 0.3, got {}", sat);

        // High valence + low uncertainty → Saturated.
        let state = InternalState {
            cognitive_emotional_valence: 0.6,
            current_uncertainty: 0.2,
            ..Default::default()
        };
        let result = metacog
            .curiosity_question_with_state("emergence", &state)
            .unwrap();
        assert!(
            matches!(result.kind, CuriosityKind::Saturated),
            "high valence + low uncertainty + sat > 0.3 should promote Saturated, got {:?}",
            result.kind
        );
    }

    /// Phase 1.2: low-uncertainty / neutral-valence state leaves the kind
    /// unchanged (pass-through).
    #[test]
    fn curiosity_question_with_state_neutral_passthrough() {
        let mut metacog = MetaCognition::new();
        metacog.curiosity.start_exploring("x", "y");
        // Push satisfaction into Wondering range (≥ 0.50, < 0.70).
        metacog.update_curiosity("x", "first info");
        metacog.update_curiosity("x", "second info");
        // 0.3 + 0.3 = 0.6 → Wondering.
        let sat = metacog.curiosity_question("x").unwrap().satisfaction;
        assert!(sat > 0.5 && sat < 0.7, "setup: sat should be in Wondering range, got {}", sat);
        let baseline = metacog.curiosity_question("x").unwrap().kind;
        assert!(matches!(baseline, CuriosityKind::Wondering), "baseline should be Wondering, got {:?}", baseline);

        let state = InternalState::default(); // all defaults → no signal
        let result = metacog
            .curiosity_question_with_state("x", &state)
            .unwrap();
        // No state signal → kind stays as satisfaction-derived (Wondering).
        assert!(
            matches!(result.kind, CuriosityKind::Wondering),
            "neutral state should not promote, got {:?}",
            result.kind
        );
    }
}
