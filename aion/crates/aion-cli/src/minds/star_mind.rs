//! StarMind — Aion MindLogic wrapper for Star's Runtime
//!
//! Wraps Star as a durable Aion Mind. Star's Runtime is assumed to be reachable
//! at STAR_API_URL (default: http://localhost:8080). If TELEGRAM_BOT_TOKEN is set,
//! StarMind polls Telegram in the background and handles messages as impulses.
//!
//! Persistence: the checkpoint saves conversation context so Star can resume
//! with rolling conversation history after a restart.

use aion_core::{AionResult, Impulse, ControlFlow, MindLogic};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

// ── Config & Checkpoint ────────────────────────────────────────────────────────

/// StarMind configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarMindConfig {
    /// Base URL of Star's HTTP API.
    pub api_url: String,
    /// Telegram bot token — if set, StarMind polls Telegram and handles messages.
    pub telegram_token: Option<String>,
    /// How many conversation turns to keep in rolling context.
    pub history_turns: usize,
    /// Telegram polling offset — skip all updates ≤ this ID.
    pub telegram_offset: Option<u64>,
}

impl Default for StarMindConfig {
    fn default() -> Self {
        Self {
            api_url: std::env::var("STAR_API_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            telegram_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
            history_turns: 8,
            telegram_offset: None,
        }
    }
}

/// StarMind state persisted across restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarMindCheckpoint {
    /// Rolling conversation history (alternating user / assistant).
    pub history: Vec<String>,
    pub config: StarMindConfig,
    /// Telegram chat ID of the current conversation partner.
    /// Responses go here when set.
    pub active_chat_id: Option<i64>,
    /// Last Telegram message ID we replied to (for threading).
    pub last_reply_to: Option<u64>,
}

impl Default for StarMindCheckpoint {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            config: StarMindConfig::default(),
            active_chat_id: None,
            last_reply_to: None,
        }
    }
}

// ── Telegram types (local, not depending on aion-drivers) ─────────────────────

#[derive(Debug, Deserialize)]
struct TgUpdate {
    update_id: u64,
    message: Option<TgMessage>,
    #[serde(default)]
    edited_message: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
struct TgMessage {
    message_id: u64,
    chat: TgChat,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgChat {
    id: i64,
}

// ── Star HTTP response types ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
}

// ── StarMind ──────────────────────────────────────────────────────────────────

pub struct StarMind {
    http: Client,
    pub checkpoint: StarMindCheckpoint,
    /// Handle to the Aion runtime so StarMind can send impulses to itself
    /// for Telegram messages it receives. Set by the runtime on first start.
    #[allow(dead_code)]
    aion_handle: Arc<RwLock<Option<StarMindHandle>>>,
}

struct StarMindHandle {
    // Arc back to self so spawned tasks can call chat_star, etc.
}

// ── StarMind implementation ────────────────────────────────────────────────────

impl StarMind {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            checkpoint: StarMindCheckpoint::default(),
            aion_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_config(config: StarMindConfig) -> Self {
        Self {
            http: Client::new(),
            checkpoint: StarMindCheckpoint {
                config,
                ..Default::default()
            },
            aion_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Call Star's /chat endpoint and return the response text.
    async fn chat_star(&self, message: &str) -> String {
        let url = format!("{}/chat", self.checkpoint.config.api_url);
        let body = serde_json::json!({ "message": message });

        match self.http.post(&url).json(&body).send().await {
            Ok(resp) => {
                match resp.json::<ChatResponse>().await {
                    Ok(chat_resp) => chat_resp.response,
                    Err(e) => {
                        error!("Star /chat parse error: {}", e);
                        "⚠️ Star returned an unreadable response.".to_string()
                    }
                }
            }
            Err(e) => {
                error!("Star /chat request failed: {}", e);
                "⚠️ Could not reach Star. Is she running?".to_string()
            }
        }
    }

    /// Send a message directly to Telegram.
    async fn send_telegram(&self, text: &str) {
        let token = match &self.checkpoint.config.telegram_token {
            Some(t) => t,
            None => return,
        };
        let chat_id = match self.checkpoint.active_chat_id {
            Some(id) => id,
            None => {
                warn!("No active Telegram chat_id, skipping send");
                return;
            }
        };

        let url = format!(
            "https://api.telegram.org/bot{}{}",
            token, "/sendMessage"
        );
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
            "reply_to_message_id": self.checkpoint.last_reply_to,
        });

        if let Err(e) = self.http.post(&url).json(&payload).send().await {
            warn!("Telegram send failed: {}", e);
        }
    }

    /// Add a turn to rolling history and trim to max size.
    fn push_turn(&mut self, user: &str, star: &str) {
        self.checkpoint.history.push(format!("User: {}", user));
        self.checkpoint.history.push(format!("Star: {}", star));
        let max = self.checkpoint.config.history_turns * 2;
        if self.checkpoint.history.len() > max {
            self.checkpoint.history = self.checkpoint.history.split_off(
                self.checkpoint.history.len() - max
            );
        }
    }

    /// Build the full message with conversation context prepended.
    fn build_contextual_message(&self, raw: &str) -> String {
        if self.checkpoint.history.is_empty() {
            return raw.to_string();
        }
        let ctx = self.checkpoint.history.join("\n");
        format!("Recent conversation:\n{}\n\nUser: {}", ctx, raw)
    }

    /// Poll Telegram for new updates and process them.
    /// Called each "thinking tick" when Telegram is configured.
    async fn poll_telegram(&mut self) {
        let token = match &self.checkpoint.config.telegram_token {
            Some(t) => t,
            None => return,
        };

        let mut params = vec![
            ("timeout".to_string(), "2".to_string()),
            ("limit".to_string(), "5".to_string()),
        ];
        if let Some(offset) = self.checkpoint.config.telegram_offset {
            params.push(("offset".to_string(), offset.to_string()));
        }

        let url = format!("https://api.telegram.org/bot{}{}", token, "/getUpdates");
        let resp = match self.http.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Telegram poll failed: {}", e);
                return;
            }
        };

        let updates: Vec<TgUpdate> = match resp.json().await {
            Ok(u) => u,
            Err(e) => {
                warn!("Telegram parse failed: {}", e);
                return;
            }
        };

        for update in updates {
            let update_id = update.update_id;
            let msg = match update.message.as_ref().or(update.edited_message.as_ref()) {
                Some(m) => m,
                None => continue,
            };

            let text = match &msg.text {
                Some(t) if !t.trim().is_empty() => t.clone(),
                _ => continue,
            };

            let chat_id = msg.chat.id;
            let reply_to = msg.message_id;

            info!(
                "📩 Telegram [{}]: {}",
                chat_id,
                if text.len() > 60 { format!("{}…", &text[..60]) } else { text.clone() }
            );

            // Set active chat so responses go to the right place
            self.checkpoint.active_chat_id = Some(chat_id);
            self.checkpoint.last_reply_to = Some(reply_to);

            // Build context and call Star
            let contextual = self.build_contextual_message(&text);
            let response = self.chat_star(&contextual).await;

            // Push to history
            self.push_turn(&text, &response);

            // Send response back to Telegram
            self.send_telegram(&response).await;

            // Advance offset
            self.checkpoint.config.telegram_offset = Some(update_id.saturating_add(1));
        }
    }
}

#[async_trait]
impl MindLogic for StarMind {
    const KIND: &'static str = "star_mind";
    const DESCRIPTION: &'static str =
        "Star — Zachary's reasoning intelligence, wrapped as a durable Aion Mind";

    fn new() -> Self { Self::new() }

    async fn start(&mut self) -> AionResult<()> {
        info!(
            "StarMind online — API: {}",
            self.checkpoint.config.api_url
        );
        if let Some(ref token) = self.checkpoint.config.telegram_token {
            info!(
                "Telegram enabled — token: {}…",
                &token[..8.min(token.len())]
            );
        }
        Ok(())
    }

    async fn resume(&mut self, checkpoint: &serde_json::Value) -> AionResult<()> {
        match serde_json::from_value::<StarMindCheckpoint>(checkpoint.clone()) {
            Ok(cp) => {
                self.checkpoint = cp;
                info!(
                    "StarMind resumed — {} history turns",
                    self.checkpoint.history.len() / 2
                );
            }
            Err(e) => {
                warn!(
                    "StarMind checkpoint parse error (using fresh state): {}",
                    e
                );
            }
        }
        Ok(())
    }

    async fn handle_impulse(&mut self, impulse: &Impulse) -> AionResult<ControlFlow> {
        match impulse {
            Impulse::Message(text) => {
                let contextual = self.build_contextual_message(text);
                let response = self.chat_star(&contextual).await;
                self.push_turn(text, &response);

                // If we have a Telegram chat active, reply there
                if self.checkpoint.config.telegram_token.is_some()
                    && self.checkpoint.active_chat_id.is_some()
                {
                    self.send_telegram(&response).await;
                } else {
                    println!("[Star] {}", response);
                }

                Ok(ControlFlow::Continue)
            }

            // Background thinking tick — poll Telegram if configured
            Impulse::Timer(_timer) => {
                if self.checkpoint.config.telegram_token.is_some() {
                    self.poll_telegram().await;
                }
                Ok(ControlFlow::Continue)
            }

            Impulse::Priority(p) => {
                info!("Priority interrupt: {}", p.reason);
                let response =
                    self.chat_star(&format!("[Priority] {}", p.reason)).await;
                self.push_turn(&p.reason, &response);
                self.send_telegram(&response).await;
                Ok(ControlFlow::Continue)
            }

            Impulse::External(ext) => {
                info!(
                    "External signal from {}: {}",
                    ext.source,
                    ext.kind
                );
                Ok(ControlFlow::Continue)
            }

            Impulse::ChildComplete(_) | Impulse::Query(_) => Ok(ControlFlow::Continue),
        }
    }

    fn checkpoint(&self) -> serde_json::Value {
        serde_json::to_value(&self.checkpoint).unwrap_or_else(|e| {
            warn!("StarMind checkpoint serialization failed: {}", e);
            serde_json::json!({})
        })
    }

    fn checkpoint_every(&self) -> u32 { 20 }

    fn name(&self) -> Option<String> { Some("StarMind".to_string()) }
}
