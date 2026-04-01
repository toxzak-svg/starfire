//! Scheduler — tokio-based timer loop.

use std::collections::HashMap;
use tokio::time::{interval, Duration};
use crate::{Impulse, MindId};

const DEFAULT_TICK_MS: u64 = 1000;

#[derive(Debug, Clone)]
pub struct ScheduledTimer {
    pub id: String,
    pub mind_id: MindId,
    pub fires_at: i64,
}

/// The Aion scheduler — manages timer dispatch and Mind wake-ups.
pub struct Scheduler {
    tick_ms: u64,
    timers: HashMap<String, ScheduledTimer>,
}

impl Scheduler {
    pub fn new() -> Self { Self::with_tick(DEFAULT_TICK_MS) }
    pub fn with_tick(tick_ms: u64) -> Self {
        Scheduler { tick_ms, timers: HashMap::new() }
    }

    pub fn schedule_timer(&mut self, id: String, mind_id: MindId, delay_ms: u64) {
        let fires_at = chrono::Utc::now().timestamp_millis() + delay_ms as i64;
        self.timers.insert(id.clone(), ScheduledTimer { id, mind_id, fires_at });
    }

    pub fn cancel_timer(&mut self, id: &str) {
        self.timers.remove(id);
    }

    pub fn active_timers(&self) -> usize { self.timers.len() }

    /// Take all timers that have fired, returning them and removing from scheduler.
    pub fn take_fired(&mut self, now: i64) -> Vec<ScheduledTimer> {
        let fired_ids: Vec<String> = self.timers.iter()
            .filter(|(_, t)| t.fires_at <= now)
            .map(|(id, _)| id.clone())
            .collect();
        fired_ids.into_iter()
            .filter_map(|id| self.timers.remove(&id))
            .collect()
    }

    /// Run the scheduler loop.
    pub async fn run<F>(&mut self, mut on_fire: F)
    where
        F: FnMut(MindId, Impulse) + Send + 'static,
    {
        let mut ticker = interval(Duration::from_millis(self.tick_ms));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let now = chrono::Utc::now().timestamp_millis();
                    let fired = self.take_fired(now);
                    for timer in fired {
                        let impulse = Impulse::timer(timer.id.clone());
                        on_fire(timer.mind_id, impulse);
                    }
                }
            }
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}
