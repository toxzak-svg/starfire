//! HTTP LLM client — calls an external llm-server over REST.
//!
//! Used when the `llm` feature is disabled (star-core Docker build).
//! The llm-server URL is configured via the `LLM_ENDPOINT` env var
//! (defaults to `http://127.0.0.1:8081`).

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8081".to_string(),
            timeout: Duration::from_secs(60),
        }
    }
}

impl ClientConfig {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            ..Default::default()
        }
    }

    pub fn completions_url(&self) -> String {
        format!("{}/v1/completions", self.base_url)
    }

    pub fn chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }
}

/// HTTP LLM client for talking to a remote llm-server.
#[derive(Debug, Clone)]
pub struct HttpLlmClient {
    config: ClientConfig,
}

impl HttpLlmClient {
    /// Create pointing at a specific URL.
    pub fn new(base_url: &str) -> Self {
        Self { config: ClientConfig::new(base_url) }
    }

    /// Read from `LLM_ENDPOINT` env var, fallback to localhost:8081.
    pub fn from_env() -> Self {
        let endpoint = std::env::var("LLM_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
        Self::new(&endpoint)
    }

    /// Returns true if the remote server responds to /health.
    pub fn health_check(&self) -> bool {
        ureq::get(&self.config.health_url())
            .call()
            .map(|r| r.status() == 200)
            .unwrap_or(false)
    }

    /// Plain text completion.
    pub fn generate(&self, prompt: &str, max_tokens: Option<u32>) -> anyhow::Result<String> {
        #[derive(serde::Serialize)]
        struct Req {
            model: String,
            prompt: String,
            max_tokens: Option<u32>,
            temperature: Option<f32>,
            stream: Option<bool>,
        }

        let resp: CompletionResp = ureq::post(&self.config.completions_url())
            .timeout(self.config.timeout)
            .send_json(Req {
                model: "bonzai-8b".to_string(),
                prompt: prompt.to_string(),
                max_tokens: max_tokens.or(Some(256)),
                temperature: Some(0.7),
                stream: Some(false),
            })?
            .into_json()?;

        Ok(resp.choices.first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }

    /// Chat completion.
    pub fn chat(&self, messages: &[HttpChatMsg], max_tokens: Option<u32>) -> anyhow::Result<String> {
        #[derive(serde::Serialize)]
        struct ChatReq<'a> {
            model: &'a str,
            messages: Vec<HttpChatMsg>,
            max_tokens: Option<u32>,
            temperature: Option<f32>,
            stream: Option<bool>,
        }

        let resp: ChatResp = ureq::post(&self.config.chat_url())
            .timeout(self.config.timeout)
            .send_json(ChatReq {
                model: "bonzai-8b",
                messages: messages.to_vec(),
                max_tokens: max_tokens.or(Some(256)),
                temperature: Some(0.7),
                stream: Some(false),
            })?
            .into_json()?;

        Ok(resp.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    /// Polish rough text via remote LLM.
    pub fn polish(&self, rough_text: &str) -> anyhow::Result<String> {
        self.chat(&[
            HttpChatMsg { role: "system".to_string(), content: "You are a text polish engine. Rewrite the following text to be more fluent and natural-sounding while preserving all factual content. Be concise. Do not add new information.".to_string() },
            HttpChatMsg { role: "user".to_string(), content: rough_text.to_string() },
        ], Some(512))
    }
}

// ============================================================
// Types matching the llm-server OpenAI-compatible API
// ============================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpChatMsg {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
struct CompletionResp {
    choices: Vec<CompletionChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct CompletionChoice {
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct ChatResp {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct ChatChoice {
    message: ChatMsgOut,
}

#[derive(Debug, serde::Deserialize)]
struct ChatMsgOut {
    content: String,
}
