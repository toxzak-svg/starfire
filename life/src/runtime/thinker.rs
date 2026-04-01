//! Background Thinker — autonomous reasoning thread

use std::sync::{Arc, Mutex};
use std::thread;
use tracing::info;

/// Background thinker — runs autonomous reasoning in a background thread.
pub struct BackgroundThinker {
    // Thread handle for the background thinker thread
    _thread: thread::JoinHandle<()>,
}

impl BackgroundThinker {
    /// Spawn a new background thinker thread.
    pub fn spawn(store: Arc<crate::Store>, reasoning: Arc<Mutex<crate::reasoning::ReasoningEngine>>) -> Self {
        let handle = thread::spawn(move || {
            info!("Background thinker started");
            // Background thinking loop — runs curiosity probes and autonomous reasoning
            // The actual loop is managed by CuriousEngine in the runtime
        });
        Self { _thread: handle }
    }
}
