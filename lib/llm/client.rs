//! HTTP client helpers for llama-server communication

use std::time::Duration;

/// Configuration for the HTTP client connection.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
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

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the completions URL.
    pub fn completions_url(&self) -> String {
        format!("{}/v1/completions", self.base_url)
    }

    /// Build the chat completions URL.
    pub fn chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    /// Build the health URL.
    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }
}
