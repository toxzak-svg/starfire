//! Web Searcher — DuckDuckGo Instant Answer API client

use std::io::Read;

/// Search result from web search.
#[derive(Debug)]
pub struct SearchResult {
    pub answer: Option<String>,
    pub url: Option<String>,
    pub related: Vec<String>,
}

/// Web searcher for Star's curiosity engine.
pub struct WebSearcher {
    _priv: (),
}

impl WebSearcher {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Search the web for a topic and return search result.
    pub fn search(&self, topic: &str) -> anyhow::Result<SearchResult> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(topic)
        );

        let response = ureq::get(&url).call()?;
        let body_str = response.into_string()?;
        let body: serde_json::Value = serde_json::from_str(&body_str)?;

        let mut result = SearchResult {
            answer: None,
            url: None,
            related: Vec::new(),
        };

        // Try AbstractText first (Wikipedia-style summary)
        if let Some(text) = body.get("AbstractText").and_then(|v| v.as_str()) {
            if !text.is_empty() {
                result.answer = Some(text.to_string());
                result.url = body.get("AbstractURL").and_then(|v| v.as_str()).map(String::from);
            }
        }

        // Collect related topics
        if let Some(related) = body.get("RelatedTopics").and_then(|v| v.as_array()) {
            for item in related.iter().take(5) {
                if let Some(text) = item.get("Text").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        result.related.push(text.to_string());
                    }
                }
            }
        }

        Ok(result)
    }
}

impl Default for WebSearcher {
    fn default() -> Self {
        Self::new()
    }
}
