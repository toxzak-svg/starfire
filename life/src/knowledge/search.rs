//! Web Search — Optional Knowledge Verification
//!
//! Lightweight web search for verifying facts and filling knowledge gaps.
//! This is NOT used for training — only for fact-checking and verification.
//!
//! Uses DuckDuckGo's instant answer API (no API key needed).

use anyhow::{Context, Result};
use serde::Deserialize;

/// Lightweight web search for fact verification.
pub struct WebSearcher {
    /// User agent to use
    user_agent: String,
}

impl WebSearcher {
    pub fn new() -> Self {
        Self {
            user_agent: "Star/1.0 (knowledge verification)".to_string(),
        }
    }

    /// Search for a topic and get a snippet.
    pub fn search(&self, query: &str) -> Result<SearchResult> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_redirect=1",
            urlencoding::encode(query)
        );

        let response = ureq::get(&url)
            .set("User-Agent", &self.user_agent)
            .call()
            .context("Failed to search")?;

        let text = response.into_string()
            .context("Failed to read response")?;

        let parsed: DuckDuckGoResponse = serde_json::from_str(&text)
            .context("Failed to parse response")?;

        Ok(SearchResult {
            query: query.to_string(),
            answer: parsed.Answer.clone().or(parsed.AbstractText.clone()),
            url: parsed.AbstractURL.clone().or(parsed.AnswerURL.clone()),
            related: parsed.RelatedTopics.iter().take(5)
                .filter_map(|t| t.Text.clone())
                .collect(),
        })
    }

    /// Verify a fact by searching for it.
    pub fn verify(&self, statement: &str) -> Result<VerificationResult> {
        let result = self.search(statement)?;
        
        let confirmed = result.answer.as_ref()
            .map(|a| a.to_lowercase().contains(&statement.to_lowercase()))
            .unwrap_or(false);
        
        Ok(VerificationResult {
            statement: statement.to_string(),
            verified: confirmed,
            answer: result.answer,
            source: result.url,
        })
    }
}

impl Default for WebSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub query: String,
    pub answer: Option<String>,
    pub url: Option<String>,
    pub related: Vec<String>,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub statement: String,
    pub verified: bool,
    pub answer: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "Answer")]
    Answer: Option<String>,
    #[serde(rename = "AbstractText")]
    AbstractText: Option<String>,
    #[serde(rename = "AbstractURL")]
    AbstractURL: Option<String>,
    #[serde(rename = "AnswerURL")]
    AnswerURL: Option<String>,
    #[serde(rename = "RelatedTopics")]
    RelatedTopics: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    Text: Option<String>,
}
