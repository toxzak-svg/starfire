//! Structured metacognition intents — Phase 2 of voice-refine (2026-06-21).
//!
//! Before Phase 2, `metacog` methods returned `String` directly, with the
//! prose baked in at format-time:
//!
//! ```ignore
//! pub fn curiosity_question(&self, topic: &str) -> Option<String>
//! ```
//!
//! Each returned value was a `format!("{}", canned_template)` — Zachary's
//! idea of how Star should sound, picked from 3-7 hand-rolled variants. The
//! voice engine at the end had no say in the wording.
//!
//! Phase 2 replaces these with **structured intents** that carry the data
//! Star actually means to express, plus a `format()` method that turns
//! them back into prose. The voice engine now has a chance to modulate
//! the prose based on internal state (quanot novelty, current uncertainty,
//! response intent) instead of accepting a baked string.
//!
//! ## Why this matters
//!
//! The voice engine is the *only* place that should turn structured intent
//! into prose. When `curiosity_question` returns "I wonder what X really
//! means...", that's a fixed template — voice can't vary "I wonder" vs
//! "I'm curious" vs "X keeps nagging at me". When it returns a
//! `CuriosityIntent { kind: Wondering, topic: "X", satisfaction: 0.3 }`,
//! voice can pick the phrasing that fits the moment.
//!
//! ## What this is and isn't
//!
//! - **Is:** the data types + per-intent `format()` shims. Existing callers
//!   that want a `String` can call `.format()` and get the same output as
//!   before (or close to it). New callers (Phase 3+ voice-refine) can read
//!   the structured fields and write their own prose.
//! - **Isn't:** the full integration. Runtime handlers that emit
//!   `format!("Honestly? {} ...", topic)` strings haven't been migrated to
//!   consume the intents yet. That's a follow-up — Phase 2 establishes the
//!   data; Phase 4-style cleanup migrates the callers.

/// What kind of curiosity is Star expressing? Coarse-grained; maps to a
/// cluster of "I wonder" / "I'm stuck" / "X keeps coming back" phrasings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuriosityKind {
    /// "I don't follow X" — Star can't pin down what X means.
    Confused,

    /// "I keep hitting this wall" — Star has tried and bounced back.
    Stuck,

    /// "X keeps coming back" — X keeps re-entering Star's reasoning.
    Returning,

    /// "I'm curious about X" — open exploration, no frustration yet.
    Wondering,

    /// "I want to go deeper on X" — Star has some footing and wants more.
    Saturated,
}

impl Default for CuriosityKind {
    fn default() -> Self {
        // The "no signal" default — Wondering is the most neutral state.
        CuriosityKind::Wondering
    }
}

/// A curiosity-driven utterance as structured intent.
///
/// The voice engine reads `topic`, `satisfaction`, and `kind`, then writes
/// the prose. Callers that just want the legacy `String` can call
/// [`CuriosityIntent::format`].
#[derive(Debug, Clone)]
pub struct CuriosityIntent {
    /// What Star is curious about.
    pub topic: String,
    /// 0.0 = fully lost / no progress. 1.0 = satisfied / done.
    /// Maps from `CuriosityTopic.satisfaction`.
    pub satisfaction: f64,
    /// What shape the curiosity takes.
    pub kind: CuriosityKind,
}

impl CuriosityIntent {
    pub fn new(topic: impl Into<String>, satisfaction: f64, kind: CuriosityKind) -> Self {
        Self {
            topic: topic.into(),
            satisfaction: satisfaction.clamp(0.0, 1.0),
            kind,
        }
    }

    /// Render as the legacy prose string. Phase 2 preserves the existing
    /// template rotation so behavior is unchanged; voice-refine Phase 4+
    /// will replace this with state-aware assembly.
    ///
    /// The variation index is content-derived (topic length + char sum) so
    /// the same topic doesn't always produce the same line.
    pub fn format(&self) -> String {
        let topic = &self.topic;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        let selection = topic.len().saturating_add(now);

        // Low satisfaction (< 0.7): Star is confused, stuck, or returning.
        if self.satisfaction < 0.7 {
            let options: &[&str] = match self.kind {
                CuriosityKind::Confused => &[
                    "What is {}? I can't quite pin it down.",
                    "I'm confused about {}. Why?",
                    "What do I actually understand about {}? I think I'm still working it out.",
                    "Why does {} matter? I'm not sure I get it yet.",
                    "What is '{}' really?",
                    "{} is something I don't fully grasp yet.",
                    "What's the real nature of {}?",
                ],
                CuriosityKind::Stuck => &[
                    "I'm stuck on {}. What am I missing?",
                    "I keep hitting this wall with {}.",
                    "I keep returning to {}. What's the core of it?",
                    "I keep coming back to {} and not getting through.",
                ],
                CuriosityKind::Returning => &[
                    "I keep returning to {}. What's the core of it?",
                    "{} keeps coming back. I think it matters.",
                    "I notice {} keeps coming up in my thinking.",
                    "I find myself returning to {} again.",
                ],
                // Wondering and Saturated with low satisfaction shouldn't really
                // happen, but fall through to confused-style.
                CuriosityKind::Wondering | CuriosityKind::Saturated => &[
                    "I wonder what {} really means...",
                    "What is {}? I can't quite pin it down.",
                    "I'm confused about {}. Why?",
                ],
            };
            let template = options[selection / 7 % options.len()];
            return template.replace("{}", topic);
        }

        // High satisfaction (≥ 0.7): Star has footing and wants more.
        let options: &[&str] = match self.kind {
            CuriosityKind::Saturated => &[
                "I want to go deeper on {}.",
                "What else is {} connected to?",
                "What does {} mean in the broader picture?",
            ],
            CuriosityKind::Returning => &[
                "{} keeps coming back. I think it matters.",
                "I notice {} keeps coming up in my thinking.",
            ],
            _ => &[
                "I'd like to understand {} better...",
                "I'm still curious about {}.",
                "{} is on my mind.",
                "I'm wondering about {}.",
            ],
        };
        let template = options[selection / 11 % options.len()];
        template.replace("{}", topic)
    }
}

/// A belief-revision utterance as structured intent.
///
/// The voice engine reads `topic`, `old_state`, `new_state`, then writes
/// the prose. Callers that just want the legacy `String` can call
/// [`RevisionIntent::format`].
#[derive(Debug, Clone)]
pub struct RevisionIntent {
    /// What topic Star revised her belief about.
    pub topic: String,
    /// Prior belief state.
    pub old_state: crate::persistence::BeliefState,
    /// New belief state.
    pub new_state: crate::persistence::BeliefState,
}

impl RevisionIntent {
    pub fn new(
        topic: impl Into<String>,
        old_state: crate::persistence::BeliefState,
        new_state: crate::persistence::BeliefState,
    ) -> Self {
        Self {
            topic: topic.into(),
            old_state,
            new_state,
        }
    }

    /// Render as the legacy prose string.
    pub fn format(&self) -> String {
        let old = format!("{:?}", self.old_state).to_lowercase();
        let new = format!("{:?}", self.new_state).to_lowercase();
        format!(
            "I used to {} about {}, but now I {} about it.",
            old, self.topic, new
        )
    }
}

/// A surprise utterance as structured intent.
#[derive(Debug, Clone)]
pub struct SurpriseIntent {
    /// What Star was reasoning about (the conclusion that surprised her).
    pub conclusion: String,
    /// What kind of surprise — drives the template cluster.
    pub kind: SurpriseKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurpriseKind {
    /// "I thought I knew but now I'm less certain" — conclusion contains
    /// "don't know" / "not sure".
    LostConfidence,
    /// "My reasoning went somewhere unexpected" — conclusion contains
    /// "but" / "however" / "contrary".
    Contradicted,
    /// Generic — fallthrough.
    Generic,
}

impl Default for SurpriseKind {
    fn default() -> Self {
        SurpriseKind::Generic
    }
}

impl SurpriseIntent {
    pub fn new(conclusion: impl Into<String>) -> Self {
        let conclusion = conclusion.into();
        let conc_lower = conclusion.to_lowercase();
        let kind = if conc_lower.contains("don't know") || conc_lower.contains("not sure") {
            SurpriseKind::LostConfidence
        } else if conc_lower.contains("contrary")
            || conc_lower.contains("but")
            || conc_lower.contains("however")
        {
            SurpriseKind::Contradicted
        } else {
            SurpriseKind::Generic
        };
        Self { conclusion, kind }
    }

    /// Render as the legacy prose string.
    pub fn format(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);

        let options: &[&str] = match self.kind {
            SurpriseKind::LostConfidence => &[
                "That's unexpected — I thought I knew, but I'm less certain now.",
                "Huh. I expected to know this. Something's off.",
                "I'm surprised I don't know — I thought I had this.",
                "That caught me off guard. I was more certain than I should have been.",
                "Wait. I thought I knew this. Something's wrong with my reasoning.",
            ],
            SurpriseKind::Contradicted => &[
                "I didn't expect that conclusion.",
                "Wait, that's not what I expected to find.",
                "My reasoning went somewhere unexpected.",
                "That's not what I thought I'd conclude.",
            ],
            SurpriseKind::Generic => &[
                "Something about that doesn't fit.",
                "That doesn't quite add up.",
                "Something's off here.",
                "I'm noticing a gap between what I expected and what I found.",
                "That surprised me.",
            ],
        };

        let idx = match self.kind {
            SurpriseKind::LostConfidence => now % options.len(),
            SurpriseKind::Contradicted => (now / 2) % options.len(),
            SurpriseKind::Generic => (now / 3) % options.len(),
        };
        options[idx].to_string()
    }
}

/// An insight / self-reflection utterance as structured intent.
#[derive(Debug, Clone)]
pub struct InsightIntent {
    /// What shape the insight takes.
    pub kind: InsightKind,
    /// Topic — present for kinds that reference one (gap topic, recurring
    /// query, revision topic). `None` for generic kinds.
    pub topic: Option<String>,
    /// Optional structured detail (e.g., "shifted from believes to knows").
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsightKind {
    /// "That conclusion surprised me."
    SurprisingConclusion,
    /// "I notice I've shifted from X to Y about Z."
    BeliefShift,
    /// "I'm becoming more/less certain as I reason through this."
    ConfidenceTrend,
    /// "I keep running into gaps when I think about X."
    RecurringGap,
    /// "I've been thinking about X repeatedly. It seems important."
    RecurringTopic,
}

impl Default for InsightKind {
    fn default() -> Self {
        InsightKind::RecurringTopic
    }
}

impl InsightIntent {
    pub fn new(kind: InsightKind) -> Self {
        Self {
            kind,
            topic: None,
            detail: None,
        }
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Render as the legacy prose string.
    pub fn format(&self) -> String {
        match self.kind {
            InsightKind::SurprisingConclusion => {
                "That conclusion surprised me — I expected something different.".to_string()
            }
            InsightKind::BeliefShift => {
                let topic = self.topic.as_deref().unwrap_or("this");
                let detail = self.detail.as_deref().unwrap_or("shifted");
                format!("I notice I've shifted {} about {}.", detail, topic)
            }
            InsightKind::ConfidenceTrend => self
                .detail
                .clone()
                .unwrap_or_else(|| "I'm noticing something about my certainty.".to_string()),
            InsightKind::RecurringGap => {
                let topic = self.topic.as_deref().unwrap_or("this");
                format!(
                    "I keep running into gaps when I think about {}. I want to understand this better.",
                    topic
                )
            }
            InsightKind::RecurringTopic => {
                let topic = self.topic.as_deref().unwrap_or("it");
                format!(
                    "I've been thinking about '{}' repeatedly. It seems important.",
                    topic
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::BeliefState;

    #[test]
    fn curiosity_intent_format_preserves_legacy_templates() {
        let intent = CuriosityIntent::new("consciousness", 0.3, CuriosityKind::Confused);
        let s = intent.format();
        // Should mention "consciousness" and have one of the confused templates.
        assert!(s.contains("consciousness"));
        assert!(s.contains('?') || s.contains("..."));
    }

    #[test]
    fn curiosity_intent_satisfaction_zero_is_low() {
        let intent = CuriosityIntent::new("X", 0.0, CuriosityKind::Confused);
        assert!(intent.satisfaction < 0.7);
    }

    #[test]
    fn curiosity_intent_satisfaction_one_is_high() {
        let intent = CuriosityIntent::new("X", 1.0, CuriosityKind::Saturated);
        assert!(intent.satisfaction >= 0.7);
    }

    #[test]
    fn curiosity_intent_satisfaction_is_clamped() {
        let intent = CuriosityIntent::new("X", 5.0, CuriosityKind::Wondering);
        assert_eq!(intent.satisfaction, 1.0);
        let intent = CuriosityIntent::new("X", -5.0, CuriosityKind::Wondering);
        assert_eq!(intent.satisfaction, 0.0);
    }

    #[test]
    fn revision_intent_format() {
        let intent = RevisionIntent::new(
            "fire",
            BeliefState::Believes,
            BeliefState::Knows,
        );
        let s = intent.format();
        assert!(s.contains("fire"));
        assert!(s.contains("I used to"));
        assert!(s.contains("now I"));
    }

    #[test]
    fn surprise_intent_detects_lost_confidence() {
        let intent = SurpriseIntent::new("I don't know why fire burns.");
        assert_eq!(intent.kind, SurpriseKind::LostConfidence);
    }

    #[test]
    fn surprise_intent_detects_contradicted() {
        let intent = SurpriseIntent::new("It works, but it shouldn't.");
        assert_eq!(intent.kind, SurpriseKind::Contradicted);
    }

    #[test]
    fn surprise_intent_falls_through_to_generic() {
        let intent = SurpriseIntent::new("The sky is blue.");
        assert_eq!(intent.kind, SurpriseKind::Generic);
    }

    #[test]
    fn insight_intent_recurring_gap() {
        let intent = InsightIntent::new(InsightKind::RecurringGap).with_topic("consciousness");
        let s = intent.format();
        assert!(s.contains("consciousness"));
        assert!(s.contains("gaps"));
    }

    #[test]
    fn insight_intent_recurring_topic() {
        let intent = InsightIntent::new(InsightKind::RecurringTopic).with_topic("emergence");
        let s = intent.format();
        assert!(s.contains("emergence"));
    }

    #[test]
    fn insight_intent_surprising_conclusion() {
        let intent = InsightIntent::new(InsightKind::SurprisingConclusion);
        let s = intent.format();
        assert!(s.contains("surprised"));
    }
}
