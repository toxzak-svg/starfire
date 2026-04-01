//! Telegram polling helper for Aion drivers
//!
//! Provides a lightweight `TgPoller` that can be embedded in any MindLogic
//! to poll Telegram updates and convert them into `Impulse::Message` calls.

use aion_core::Impulse;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

// ── Telegram types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TgUpdate {
    pub update_id: u64,
    #[serde(default)]
    pub message: Option<TgMessage>,
    #[serde(default)]
    pub edited_message: Option<TgMessage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TgMessage {
    pub message_id: u64,
    pub chat: TgChat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TgChat {
    pub id: i64,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgResponse<T> {
    ok: bool,
    #[serde(default)]
    result: T,
    #[serde(default)]
    description: Option<String>,
}

// ── TgPoller ─────────────────────────────────────────────────────────────────

/// A Telegram polling helper. Embed this in a MindLogic struct to receive
/// Telegram messages as impulses.
///
/// Usage in your MindLogic:
/// ```ignore
/// struct MyMind {
///     tg: TgPoller,
/// }
///
/// impl MindLogic for MyMind {
///     async fn handle_impulse(&mut self, impulse: &Impulse) -> AionResult<ControlFlow> {
///         if let Impulse::External(ext) = impulse {
///             if ext.source == "telegram" {
///                 // handle Telegram message
///             }
///         }
///         Ok(ControlFlow::Continue)
///     }
/// }
/// ```
pub struct TgPoller {
    http: Client,
    bot_token: String,
    /// Last update_id we've processed — skip anything ≤ this.
    offset: Option<u64>,
    poll_interval: Duration,
}

impl TgPoller {
    /// Create a new poller.
    pub fn new(bot_token: String) -> Self {
        Self {
            http: Client::new(),
            bot_token,
            offset: None,
            poll_interval: Duration::from_secs(2),
        }
    }

    /// Set the polling interval. Default: 2 seconds.
    pub fn with_interval(mut self, dur: Duration) -> Self {
        self.poll_interval = dur;
        self
    }

    /// Start polling. Calls `on_message` for each new message.
    /// Returns when `shutdown` is set to true.
    pub async fn run(&mut self, shutdown: Arc<RwLock<bool>>, mut on_message: impl FnMut(i64, u64, String)) {
        info!(
            "Telegram poller started (token: {}…, interval: {:?})",
            &self.bot_token[..8.min(self.bot_token.len())],
            self.poll_interval
        );

        let mut poll = interval(self.poll_interval);

        loop {
            poll.tick().await;

            if *shutdown.read().await {
                info!("Telegram poller shutting down");
                break;
            }

            if let Err(e) = self.poll_once(&mut on_message).await {
                warn!("Telegram poll error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    /// One poll iteration.
    async fn poll_once(&mut self, on_message: &mut impl FnMut(i64, u64, String)) -> Result<(), String> {
        let mut params = vec![
            ("timeout".to_string(), "3".to_string()),
            ("limit".to_string(), "5".to_string()),
        ];
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }

        let url = format!(
            "https://api.telegram.org/bot{}{}",
            self.bot_token, "/getUpdates"
        );

        let resp = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        let tg_resp: TgResponse<Vec<TgUpdate>> = resp
            .json()
            .await
            .map_err(|e| format!("parse failed: {}", e))?;

        let updates = tg_resp.result;

        for update in updates {
            let update_id = update.update_id;

            let msg = update
                .message
                .as_ref()
                .or(update.edited_message.as_ref());

            let (chat_id, text, reply_to) = match msg {
                Some(m) => {
                    let chat_id = m.chat.id;
                    let reply_to = m.message_id;
                    let text = m.text.clone().unwrap_or_default();
                    (chat_id, text, reply_to)
                }
                None => continue,
            };

            if text.trim().is_empty() {
                continue;
            }

            info!(
                "📩 Telegram [{}] @{}: {}",
                chat_id,
                msg.as_ref().unwrap().chat.username.as_deref().unwrap_or("?"),
                if text.len() > 60 {
                    format!("{}…", &text[..60])
                } else {
                    text.clone()
                }
            );

            on_message(chat_id, reply_to, text);

            self.offset = Some(update_id.saturating_add(1));
        }

        Ok(())
    }
}

/// Send a message to a Telegram chat.
pub async fn send_telegram_message(
    http: &Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
    reply_to: Option<u64>,
) -> Result<(), String> {
    let url = format!(
        "https://api.telegram.org/bot{}{}",
        bot_token, "/sendMessage"
    );

    #[derive(serde::Serialize)]
    struct Payload<'a> {
        chat_id: i64,
        text: &'a str,
        parse_mode: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to_message_id: Option<u64>,
    }

    let payload = Payload {
        chat_id,
        text,
        parse_mode: "Markdown",
        reply_to_message_id: reply_to,
    };

    let resp = http
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("send failed: {}", e))?;

    let tg_resp: TgResponse<()> = resp
        .json()
        .await
        .map_err(|e| format!("parse response failed: {}", e))?;

    if !tg_resp.ok {
        return Err(tg_resp.description.unwrap_or_else(|| "unknown error".to_string()));
    }

    Ok(())
}
