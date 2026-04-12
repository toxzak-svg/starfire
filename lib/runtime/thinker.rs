//! Background Thinker — autonomous reasoning thread
//!
//! Runs curiosity probes and autonomous exploration in a background thread
//! so Star isn't just waiting for your next message.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::info;

/// A curiosity result from the background thinker.
#[derive(Debug, Clone)]
pub struct ThoughtResult {
    /// What Star wondered about
    pub question: String,
    /// What she found or concluded
    pub answer: Option<String>,
    /// The topic she was exploring
    pub topic: String,
    /// How the thought was generated
    pub generated_by: String,
}

/// Shared state between the runtime and the background thinker.
#[derive(Clone)]
pub struct SharedThoughts {
    /// Latest curiosity result, waiting to be expressed
    pub latest: Arc<Mutex<Option<ThoughtResult>>>,
    /// Whether the thinker thread is currently running
    pub running: Arc<Mutex<bool>>,
}

impl SharedThoughts {
    pub fn new() -> Self {
        Self {
            latest: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for SharedThoughts {
    fn default() -> Self {
        Self::new()
    }
}

/// Background thinker — runs autonomous curiosity in a background thread.
/// Periodically fires curiosity probes and stores results that the runtime
/// can weave into conversation.
pub struct BackgroundThinker {
    shared: Arc<SharedThoughts>,
    _thread: thread::JoinHandle<()>,
}

impl BackgroundThinker {
    /// Spawn a background thinker thread.
    /// It periodically fires curiosity probes and stores results in shared state.
    pub fn spawn(
        store: Arc<crate::Store>,
        reasoning: Arc<Mutex<crate::reasoning::ReasoningEngine>>,
        web_search: crate::knowledge::search::WebSearcher,
        shared: Arc<SharedThoughts>,
    ) -> Self {
        let shared2 = shared.clone();
        let handle = thread::spawn(move || {
            // Mark as running
            {
                let mut r = shared2.running.lock().unwrap();
                *r = true;
            }
            info!("Background thinker started");

            // Initial delay before first probe (let Star settle in)
            thread::sleep(Duration::from_secs(15));

            loop {
                // Check if we should stop
                {
                    let r = shared2.running.lock().unwrap();
                    if !*r {
                        break;
                    }
                }

                // Wait 2 minutes between probes
                thread::sleep(Duration::from_secs(120));

                // Check again before probing
                {
                    let r = shared2.running.lock().unwrap();
                    if !*r {
                        break;
                    }
                }

                // Detect reasoning gaps
                let gaps = match store.detect_reasoning_gaps(7, 10) {
                    Ok(g) => g,
                    Err(e) => {
                        tracing::debug!("Background thinker: could not detect gaps: {}", e);
                        continue;
                    }
                };

                if gaps.is_empty() {
                    tracing::debug!("Background thinker: no reasoning gaps found");
                    continue;
                }

                // Pick the most salient gap
                let gap = &gaps[0];
                let topic = gap.topic.clone();
                let conclusion = gap.conclusion.clone();

                // Generate a probe question
                let question = if conclusion.len() > 10 {
                    format!(
                        "I said '{}' — why wasn't I more confident? What would I need to know?",
                        &conclusion[..conclusion.len().min(60)]
                    )
                } else {
                    format!(
                        "I'm uncertain about '{}'. What matters most here? What should I investigate?",
                        topic
                    )
                };

                info!(
                    "Background curiosity probe: topic='{}', question: {}",
                    &topic,
                    &question[..question.len().min(80)]
                );

                // Try to answer via reasoning engine + memories
                let answer = {
                    let mut reasoning = match reasoning.lock() {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    let memories = match store.search_memories(&topic, 5, None) {
                        Ok(m) => m,
                        Err(_) => Vec::new(),
                    };
                    let result = reasoning.reason(&question, &memories);
                    result.answer.clone()
                };

                // If no local answer, try web search
                let answer = if answer.is_none() {
                    if let Ok(result) = web_search.search(&topic) {
                        result.answer
                    } else {
                        None
                    }
                } else {
                    answer
                };

                // Store the result for the runtime to pick up
                let result = ThoughtResult {
                    question: question.clone(),
                    answer: answer.clone(),
                    topic: topic.clone(),
                    generated_by: "background_curiosity".to_string(),
                };

                {
                    let mut latest = shared2.latest.lock().unwrap();
                    *latest = Some(result);
                }

                // If we found something interesting, save as memory
                if let Some(ref ans) = answer {
                    let memory = crate::persistence::Memory::new_seeded(
                        &format!("Self-exploration on '{}': {}", topic, ans),
                        crate::persistence::MemoryDomain::Episodic,
                        0.65,
                    );
                    let _ = store.insert_memory(&memory);
                }

                info!(
                    "Background curiosity result: topic='{}', found_answer={}",
                    topic,
                    answer.is_some()
                );
            }

            info!("Background thinker stopped");
        });

        Self { shared, _thread: handle }
    }

    /// Stop the background thinker.
    pub fn stop(&self) {
        let mut r = self.shared.running.lock().unwrap();
        *r = false;
    }
}

impl Drop for BackgroundThinker {
    fn drop(&mut self) {
        self.stop();
    }
}
