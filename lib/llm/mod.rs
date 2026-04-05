//! LLM — Qwen2-0.5B integration via llama-server HTTP API
//!
//! Starfire uses a tiny quantized LLM (Qwen2-0.5B Q4_K_M) for:
//! - Fluency: makes Starfire's output sound natural
//! - Gap filling: fills knowledge gaps that Quanot can't handle
//! - Text generation: polishes rough reasoning into readable prose
//!
//! The LLM runs as a local llama-server sidecar (HTTP on localhost:8080).
//! This keeps the integration clean and avoids CGO complexity.
//!
//! # Usage
//!
//! ```ignore
//! let llm = LlmEngine::new("http://127.0.0.1:8080", "qwen2-0.5b")?;
//!
//! // Check if server is running
//! if llm.health_check() {
//!     let response = llm.generate("What is 2+2?").await?;
//! }
//! ```

pub mod client;
pub mod chat;

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LLM inference engine — talks to a local llama-server HTTP endpoint.
pub struct LlmEngine {
    base_url: String,
    model: String,
    timeout: Duration,
}

impl LlmEngine {
    /// Create a new LLM engine pointing at a llama-server instance.
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: model.to_string(),
            timeout: Duration::from_secs(60),
        }
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if the llama-server is running and responsive.
    pub fn health_check(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        match ureq::get(&url).timeout(self.timeout).call() {
            Ok(resp) => resp.status() == 200,
            Err(_) => false,
        }
    }

    /// Generate a text completion for a prompt.
    pub fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let payload = chat::CompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            max_tokens: Some(256),
            temperature: Some(0.7),
            stream: Some(false),
            ..Default::default()
        };

        let url = format!("{}/v1/completions", self.base_url);
        let resp = ureq::post(&url)
            .timeout(self.timeout)
            .send_json(serde_json::to_value(&payload)?)?;

        let body: serde_json::Value = resp.into_json()?;
        let text = body["choices"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No text in completion response"))?;
        Ok(text.to_string())
    }

    /// Generate a chat completion from a list of messages.
    pub fn chat(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        let payload = chat::ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            max_tokens: Some(256),
            temperature: Some(0.7),
            stream: Some(false),
            ..Default::default()
        };

        let url = format!("{}/v1/chat/completions", self.base_url);
        let resp = ureq::post(&url)
            .timeout(self.timeout)
            .send_json(serde_json::to_value(&payload)?)?;

        let body: serde_json::Value = resp.into_json()?;
        let text = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No message content in chat response"))?;
        Ok(text.to_string())
    }

    /// Generate with a system prompt + user message.
    pub fn generate_with_system(&self, system: &str, user: &str) -> anyhow::Result<String> {
        self.chat(&[
            ChatMessage::system(system),
            ChatMessage::user(user),
        ])
    }

    /// Polish a rough Starfire output into fluent text.
    /// Uses a concise system prompt to keep the LLM's role narrow.
    pub fn polish(&self, rough_text: &str) -> anyhow::Result<String> {
        self.generate_with_system(
            "You are a text polish engine. Rewrite the following text to be more fluent \
             and natural-sounding while preserving all factual content. Be concise. \
             Do not add new information. Do not summarize.",
            rough_text,
        )
    }

    /// Base URL accessor (for testing / configuration).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "You are helpful");

        let usr = ChatMessage::user("Hello");
        assert_eq!(usr.role, "user");

        let ast = ChatMessage::assistant("Hi there");
        assert_eq!(ast.role, "assistant");
    }

    #[test]
    fn test_engine_creation() {
        let engine = LlmEngine::new("http://localhost:8080", "qwen2-0.5b");
        assert_eq!(engine.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_health_check_no_server() {
        // Without a server running, should return false
        let engine = LlmEngine::new("http://127.0.0.1:19999", "qwen2-0.5b");
        assert!(!engine.health_check());
    }
}
