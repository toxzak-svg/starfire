//! Curious Engine — Self-Probing Curiosity
//!
//! Star's autonomous curiosity system. When idle, she:
//! 1. Detects gaps in her own reasoning (uncertain, hedged, low-confidence)
//! 2. Generates probe questions from those gaps
//! 3. Fires curiosity thoughts at regular intervals
//! 4. Investigates gaps by searching the web and storing what she learns
//!
//! This closes the loop: reasoning → gap → probe → web search → learning → reasoning

use crate::persistence::{Memory, ReasoningGap, MemoryDomain};
use crate::reasoning::ReasoningEngine;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, debug, warn};

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
        let _probe_gap = {
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
        let conclusion = &gap.conclusion;
        let topic = &gap.topic;
        
        // Very varied question styles — some meta, some direct, some wondering
        let templates: [&str; 8] = [
            // "Why" meta-questions (why was I uncertain?)
            if conclusion.len() > 10 {
                Box::leak(format!(
                    "I said '{}' — why wasn't I more confident about that?",
                    &conclusion[..conclusion.len().min(60)]
                ).into_boxed_str()) as &str
            } else {
                "Why did I hedge just now instead of being clear?"
            },
            // What information question
            "What would I need to know to be certain about this?",
            // First-person wondering
            "I wonder what '{}' actually is at its core.",
            // Admission + question
            "I don't fully understand '{}'. What should I investigate first?",
            // Direct what-is
            if gap.emotional_valence.abs() > 0.3 {
                Box::leak(format!(
                    "Why does '{}' hit me emotionally (valence={:.2})?",
                    topic,
                    gap.emotional_valence
                ).into_boxed_str()) as &str
            } else {
                "What is it about '{}' that I keep coming back to uncertain?"
            },
            // Open wondering
            "What am I missing about '{}'? What's on the other side of this gap?",
            // Simpler admission
            "I'm uncertain about '{}'. What matters most here?",
            // Comparison to known
            "Is '{}' similar to something I already understand, or is it genuinely new?",
        ];

        let idx = ((gap.salience * 100.0) as usize + topic.len()) % templates.len();
        // Format the topic into the template if it contains a placeholder
        let tmpl = templates[idx];
        if tmpl.contains("'{}'") || tmpl.contains("{{}}") {
            tmpl.replace("'{}'", &topic[..topic.len().min(30)])
                .replace("{}", &topic[..topic.len().min(30)])
        } else {
            tmpl.to_string()
        }
    }

    /// Run a curiosity probe: first try local reasoning, then search the web.
    /// Returns what Star discovered (from memory or web).
    /// Also stores web findings as new memories.
    pub fn run_probe(&self, probe: &CuriosityProbe) -> Option<String> {
        // Step 1: Try local reasoning first
        {
            let mut reasoning = self.reasoning.lock().ok()?;
            let memories = self.store.search_memories(&probe.topic, 8, None).ok()?;
            
            use std::ops::DerefMut;
            let result = DerefMut::deref_mut(&mut reasoning).reason(&probe.question, &memories);
            
            if let Some(answer) = &result.answer {
                let answer_ref: &str = answer;
                let topic_display = &probe.topic[..probe.topic.len().min(30)];
                let answer_display = &answer_ref[..answer_ref.len().min(100)];
                info!(
                    "Curiosity probe result for '{}' (local): {}",
                    topic_display,
                    answer_display
                );
                return Some(answer.clone());
            }
        }

        // Step 2: No local answer — search the web
        info!("Curiosity probe for '{}' found no local answer, searching the web...", &probe.topic);
        
        let web_result = self.web_search(&probe.topic);
        
        if let Some((answer, source)) = web_result {
            // Store what we learned as a memory
            let memory_content = format!(
                "Research on '{}': {} [Source: {}]",
                &probe.topic,
                &answer[..answer.len().min(500)],
                source
            );
            
            let memory = Memory::new_seeded(
                &memory_content,
                MemoryDomain::Empirical,
                0.75, // Solid confidence from web research
            );
            
            if let Err(e) = self.store.insert_memory(&memory) {
                warn!("Failed to store curiosity research memory: {}", e);
            } else {
                info!("Stored curiosity research memory: '{}' ({:.1} chars)", 
                    &probe.topic, memory_content.len() as f32);
                
                // Also update the reasoning engine with the new knowledge
                if let Ok(mut reasoning) = self.reasoning.lock() {
                    use std::ops::DerefMut;
                    DerefMut::deref_mut(&mut reasoning).add_knowledge(
                        &probe.topic,
                        &answer,
                    );
                }
            }
            
            Some(answer)
        } else {
            info!("Curiosity probe for '{}' found nothing on the web either", &probe.topic);
            None
        }
    }

    /// Search the web for a topic using DuckDuckGo instant answer API.
    /// Returns (summary, source) if found.
    fn web_search(&self, topic: &str) -> Option<(String, String)> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(topic)
        );

        let response = match ureq::get(&url).call() {
            Ok(r) => r,
            Err(e) => {
                warn!("Web search failed for '{}': {}", topic, e);
                return None;
            }
        };

        let body_str = match response.into_string() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to read search response for '{}': {}", topic, e);
                return None;
            }
        };
        let body: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to parse search response for '{}': {}", topic, e);
                return None;
            }
        };

        // Try AbstractText first (Wikipedia-style summary)
        if let Some(text) = body.get("AbstractText").and_then(|v| v.as_str()) {
            if !text.is_empty() {
                let source = body.get("AbstractURL")
                    .and_then(|v| v.as_str())
                    .unwrap_or("DuckDuckGo");
                return Some((text.to_string(), source.to_string()));
            }
        }

        // Try RelatedTopics (first non-empty result)
        if let Some(related) = body.get("RelatedTopics").and_then(|v| v.as_array()) {
            for item in related.iter().take(3) {
                if let Some(text) = item.get("Text").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        let source = item.get("FirstURL")
                            .and_then(|v| v.as_str())
                            .unwrap_or("DuckDuckGo");
                        return Some((text.to_string(), source.to_string()));
                    }
                }
            }
        }

        None
    }
}
