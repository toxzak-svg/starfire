//! Curious Engine — Self-Probing Curiosity
//!
//! Star's autonomous curiosity system. When idle, she:
//! 1. Detects gaps in her own reasoning (uncertain, hedged, low-confidence)
//! 2. Generates probe questions from those gaps
//! 3. Fires curiosity thoughts at regular intervals
//!
//! This is what closes the loop: reasoning → gap → probe → learning → reasoning

use crate::persistence::ReasoningGap;
use crate::reasoning::ReasoningEngine;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, debug};

/// Curiosity engine — runs self-probing curiosity in idle time.
pub struct CuriousEngine {
    /// When we last fired a curiosity probe
    last_probe: Mutex<Option<Instant>>,
    /// Minimum interval between probes (1 minute)
    probe_interval: Duration,
    /// Cooldown: don't fire if we just had a conversation turn recently
    last_activity: Mutex<Instant>,
    /// Activity threshold: fire curiosity if idle for at least this long
    idle_threshold: Duration,
    /// Store for reasoning event queries
    store: Arc<crate::Store>,
    /// Reasoning engine for probe generation
    reasoning: Arc<std::sync::Mutex<ReasoningEngine>>,
}

#[derive(Debug, Clone)]
pub struct CuriosityProbe {
    pub gap: ReasoningGap,
    pub question: String,
    pub topic: String,
}

impl CuriousEngine {
    pub fn new(store: Arc<crate::Store>, reasoning: Arc<std::sync::Mutex<ReasoningEngine>>) -> Self {
        Self {
            last_probe: Mutex::new(None),
            probe_interval: Duration::from_secs(60), // 1 minute
            last_activity: Mutex::new(Instant::now()),
            idle_threshold: Duration::from_secs(30), // Fire if 30s idle
            store,
            reasoning,
        }
    }

    /// Notify the engine that Star just had a conversation turn.
    /// Resets the idle timer.
    pub fn note_activity(&self) {
        let mut last = self.last_activity.lock().unwrap();
        *last = Instant::now();
    }

    /// Check if we should fire a curiosity probe.
    /// Call this periodically (e.g., on every conversation turn).
    /// Returns the probe if it's time to fire, otherwise None.
    pub fn maybe_fire(&self) -> Option<CuriosityProbe> {
        let idle_for = {
            let last = self.last_activity.lock().unwrap();
            last.elapsed()
        };

        // Not idle enough yet
        if idle_for < self.idle_threshold {
            return None;
        }

        // Check if we've fired recently
        let probe_gap = {
            let last_probe = self.last_probe.lock().unwrap();
            if let Some(last) = *last_probe {
                let elapsed = last.elapsed();
                if elapsed < self.probe_interval {
                    return None;
                }
                elapsed
            } else {
                // Never fired, no gap
                Duration::ZERO
            }
        };

        // Time to probe
        let gaps = match self.store.detect_reasoning_gaps(7, 10) {
            Ok(g) => g,
            Err(e) => {
                debug!("Could not detect reasoning gaps: {}", e);
                return None;
            }
        };

        if gaps.is_empty() {
            debug!("No reasoning gaps found for curiosity probe");
            return None;
        }

        // Pick the highest-salience gap
        let gap = &gaps[0];
        
        // Generate probe question
        let question = self.generate_probe_question(gap);
        let topic = gap.topic.clone();

        // Record that we fired
        {
            let mut last_probe = self.last_probe.lock().unwrap();
            *last_probe = Some(Instant::now());
        }

        info!(
            "Curiosity probe fired: gap='{}' (salience={:.3}), question: {}",
            &gap.topic,
            gap.salience,
            &question[..question.len().min(80)]
        );

        Some(CuriosityProbe {
            gap: gap.clone(),
            question,
            topic,
        })
    }

    /// Generate a self-probing question from a reasoning gap.
    /// The question is NOT "what is X?" — it's "why was I uncertain?"
    fn generate_probe_question(&self, gap: &ReasoningGap) -> String {
        // The key insight: we're not asking about the TOPIC.
        // We're asking about the GAP IN OUR REASONING.
        // "Why did I conclude X with low confidence?"
        
        let conclusion = &gap.conclusion;
        let topic = &gap.topic;
        
        // Templates that get at the reasoning gap, not just the topic
        let templates = [
            // Direct uncertainty probe
            format!(
                "I said '{}' but wasn't sure — why did I lack confidence there?",
                &conclusion[..conclusion.len().min(60)]
            ),
            // Missing information probe
            format!(
                "What information would change my conclusion about '{}'?",
                topic
            ),
            // Structural uncertainty probe
            format!(
                "When I think about '{}', I concluded '{}' — but what am I missing?",
                topic,
                &conclusion[..conclusion.len().min(40)]
            ),
            // Hedge analysis
            format!(
                "I hedged about '{}' — was I right to be uncertain, or should I know this?",
                topic
            ),
            // Emotional salience probe (why did it bother me?)
            if gap.emotional_valence.abs() > 0.3 {
                format!(
                    "I found thinking about '{}' emotionally charged (valence={:.2}) — why does it bother me?",
                    topic,
                    gap.emotional_valence
                )
            } else {
                format!(
                    "Why does '{}' remain uncertain for me after all this time?",
                    topic
                )
            },
        ];

        // Use gap salience + topic length as selection key for variety
        let idx = ((gap.salience * 100.0) as usize + topic.len()) % templates.len();
        templates[idx].clone()
    }

    /// Run a curiosity probe: use the reasoning engine to actually investigate the gap.
    /// Returns what Star discovered.
    pub fn run_probe(&self, probe: &CuriosityProbe) -> Option<String> {
        let mut reasoning = self.reasoning.lock().ok()?;
        
        // Load relevant memories for the topic
        let memories = self.store.search_memories(&probe.topic, 8, None).ok()?;
        
        // Reason about the probe question — use deref_mut to get &mut ReasoningEngine
        use std::ops::DerefMut;
        let result = DerefMut::deref_mut(&mut reasoning).reason(&probe.question, &memories);
        
        if let Some(answer) = result.answer {
            let answer_ref: &str = &answer;
            let topic_display = &probe.topic[..probe.topic.len().min(30)];
            let answer_display = &answer_ref[..answer_ref.len().min(100)];
            info!(
                "Curiosity probe result for '{}': {}",
                topic_display,
                answer_display
            );
            Some(answer)
        } else {
            info!("Curiosity probe for '{}' found no answer", &probe.topic);
            None
        }
    }
}