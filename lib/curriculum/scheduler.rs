//! Curriculum Scheduler — Schedules learning sessions

use super::{CurriculumEngine, GapId, LearningTask};

/// Scheduling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulePolicy {
    /// Process gaps as they are discovered
    Immediate,
    /// Batch and process at intervals
    Periodic { interval_secs: i64 },
    /// Process when idle
    OnIdle,
    /// Threshold-based
    Threshold { min_gaps: usize },
}

impl Default for SchedulePolicy {
    fn default() -> Self {
        Self::Threshold { min_gaps: 3 }
    }
}

/// A scheduled learning session
#[derive(Debug, Clone)]
pub struct LearningSession {
    pub task: LearningTask,
    pub scheduled_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub outcome: Option<String>,
}

/// Curriculum scheduler
pub struct CurriculumScheduler {
    policy: SchedulePolicy,
    sessions: Vec<LearningSession>,
    last_run: Option<i64>,
    pending_gaps: Vec<GapId>,
}

impl Default for CurriculumScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl CurriculumScheduler {
    pub fn new() -> Self {
        Self {
            policy: SchedulePolicy::default(),
            sessions: Vec::new(),
            last_run: None,
            pending_gaps: Vec::new(),
        }
    }

    pub fn with_policy(policy: SchedulePolicy) -> Self {
        Self {
            policy,
            sessions: Vec::new(),
            last_run: None,
            pending_gaps: Vec::new(),
        }
    }

    /// Check if a session should be triggered
    pub fn should_trigger(&self, engine: &CurriculumEngine) -> bool {
        match &self.policy {
            SchedulePolicy::Immediate => !engine.gaps().is_empty(),

            SchedulePolicy::Periodic { interval_secs } => {
                let now = crate::now_timestamp();
                if let Some(last) = self.last_run {
                    if now - last >= *interval_secs {
                        return !engine.gaps().is_empty();
                    }
                } else {
                    // Never run, trigger now
                    return !engine.gaps().is_empty();
                }
                false
            }

            SchedulePolicy::OnIdle => !engine.gaps().is_empty(), // Would need idle detection

            SchedulePolicy::Threshold { min_gaps } => engine.gap_count() >= *min_gaps,
        }
    }

    /// Schedule learning sessions for pending gaps
    pub fn schedule(&mut self, engine: &CurriculumEngine) -> Vec<LearningSession> {
        let now = crate::now_timestamp();
        let mut sessions = Vec::new();

        // Get top gaps
        let gaps = engine.top_gaps(5);

        for gap in gaps {
            if self.pending_gaps.contains(&gap.id) {
                continue; // Already scheduled
            }

            let task = engine.generate_task(gap);
            let session = LearningSession {
                task,
                scheduled_at: now,
                started_at: None,
                completed_at: None,
                outcome: None,
            };

            sessions.push(session.clone());
            self.pending_gaps.push(gap.id);
            self.sessions.push(session);
        }

        self.last_run = Some(now);
        sessions
    }

    /// Start a session
    pub fn start_session(&mut self, gap_id: &GapId) -> bool {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.task.gap.id == *gap_id) {
            session.started_at = Some(crate::now_timestamp());
            return true;
        }
        false
    }

    /// Complete a session
    pub fn complete_session(&mut self, gap_id: &GapId, outcome: impl Into<String>) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.task.gap.id == *gap_id) {
            session.completed_at = Some(crate::now_timestamp());
            session.outcome = Some(outcome.into());
            self.pending_gaps.retain(|id| id != gap_id);
        }
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[LearningSession] {
        &self.sessions
    }

    /// Get completed sessions
    pub fn completed_sessions(&self) -> Vec<&LearningSession> {
        self.sessions
            .iter()
            .filter(|s| s.completed_at.is_some())
            .collect()
    }

    /// Get pending sessions
    pub fn pending_sessions(&self) -> Vec<&LearningSession> {
        self.sessions
            .iter()
            .filter(|s| s.completed_at.is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_trigger_threshold() {
        let scheduler = CurriculumScheduler::with_policy(SchedulePolicy::Threshold { min_gaps: 3 });
        let mut engine = CurriculumEngine::new();

        // Add 2 gaps
        engine.add_gap(KnowledgeGap::new("A", GapType::Incomplete));
        engine.add_gap(KnowledgeGap::new("B", GapType::Incomplete));

        assert!(!scheduler.should_trigger(&engine));

        // Add 3rd gap
        engine.add_gap(KnowledgeGap::new("C", GapType::Incomplete));

        assert!(scheduler.should_trigger(&engine));
    }

    #[test]
    fn test_schedule() {
        let mut scheduler = CurriculumScheduler::new();
        let mut engine = CurriculumEngine::new();

        engine.add_gap(KnowledgeGap::new("Test", GapType::Incomplete).with_urgency(0.8));

        let sessions = scheduler.schedule(&engine);
        assert!(!sessions.is_empty());
    }
}
