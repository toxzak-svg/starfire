//! SearXNG-compatible search provider for Starfire's bounded retrieval layer.
//!
//! This adapter only turns an explicit search request into typed result metadata.
//! It does not fetch result pages by itself, store credentials, mutate memory, enter
//! `Runtime::chat()`, or grant tool or autonomous-action authority.

use crate::web_retrieval::{
    BoundedRetriever, RetrievalError, RetrievalPolicy, SearchProvider, WebFetchRequest,
    WebSearchHit, WebSearchRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_ENDPOINT_BYTES: usize = 4 * 1024;
const MAX_SEARCH_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearxngConfig {
    /// Full SearXNG search endpoint, normally `https://host.example/search`.
    pub endpoint: String,
    /// SearXNG language value such as `en-US` or `all`.
    pub language: String,
    /// SearXNG safe-search level: 0 off, 1 moderate, 2 strict.
    pub safe_search: u8,
}

impl SearxngConfig {
    pub fn validate(&self) -> Result<(), RetrievalError> {
        let endpoint = self.endpoint.trim();
        if endpoint.is_empty() || endpoint.len() > MAX_ENDPOINT_BYTES {
            return Err(RetrievalError::SearchProvider(
                "SearXNG endpoint is empty or too long".to_string(),
            ));
        }
        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err(RetrievalError::SearchProvider(
                "SearXNG endpoint must use HTTP or HTTPS".to_string(),
            ));
        }
        if endpoint.contains('#')
            || endpoint.contains('\\')
            || endpoint.chars().any(char::is_whitespace)
        {
            return Err(RetrievalError::SearchProvider(
                "SearXNG endpoint contains unsupported characters".to_string(),
            ));
        }
        if self.language.trim().is_empty() || self.language.len() > 32 {
            return Err(RetrievalError::SearchProvider(
                "SearXNG language is empty or too long".to_string(),
            ));
        }
        if self.safe_search > 2 {
            return Err(RetrievalError::SearchProvider(
                "SearXNG safe_search must be 0, 1, or 2".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SearxngSearchProvider {
    config: SearxngConfig,
    retriever: BoundedRetriever,
}

impl SearxngSearchProvider {
    pub fn new(
        config: SearxngConfig,
        mut retrieval_policy: RetrievalPolicy,
    ) -> Result<Self, RetrievalError> {
        config.validate()?;
        retrieval_policy.validate()?;
        retrieval_policy.max_bytes = retrieval_policy.max_bytes.min(MAX_SEARCH_RESPONSE_BYTES);
        let retriever = BoundedRetriever::new(retrieval_policy)?;
        Ok(Self { config, retriever })
    }

    pub fn config(&self) -> &SearxngConfig {
        &self.config
    }

    pub fn build_search_url(&self, request: &WebSearchRequest) -> Result<String, RetrievalError> {
        request.validate()?;
        let endpoint = self
            .config
            .endpoint
            .trim_end_matches(|character| matches!(character, '?' | '&'));
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        Ok(format!(
            "{endpoint}{separator}q={query}&format=json&language={language}&safesearch={safe_search}",
            query = urlencoding::encode(request.query.trim()),
            language = urlencoding::encode(self.config.language.trim()),
            safe_search = self.config.safe_search,
        ))
    }
}

impl SearchProvider for SearxngSearchProvider {
    fn search(&self, request: &WebSearchRequest) -> Result<Vec<WebSearchHit>, RetrievalError> {
        let resource = self.retriever.fetch(&WebFetchRequest {
            url: self.build_search_url(request)?,
            purpose: "search the public web through the configured SearXNG provider".to_string(),
        })?;
        let body = std::str::from_utf8(&resource.bytes).map_err(|_| {
            RetrievalError::SearchProvider("SearXNG response is not UTF-8 JSON".to_string())
        })?;
        let response: SearxngResponse = serde_json::from_str(body).map_err(|error| {
            RetrievalError::SearchProvider(format!("invalid SearXNG JSON response: {error}"))
        })?;

        let mut seen = HashSet::new();
        let mut hits = Vec::with_capacity(request.max_results as usize);
        for result in response.results {
            let url = result.url.trim();
            if !is_supported_result_url(url) || !seen.insert(url.to_string()) {
                continue;
            }
            let title = if result.title.trim().is_empty() {
                url.to_string()
            } else {
                result.title.trim().to_string()
            };
            let snippet = result
                .content
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            hits.push(WebSearchHit {
                rank: hits.len() as u16 + 1,
                url: url.to_string(),
                title,
                snippet,
            });
            if hits.len() >= request.max_results as usize {
                break;
            }
        }
        Ok(hits)
    }
}

fn is_supported_result_url(value: &str) -> bool {
    value.len() <= 8 * 1024
        && !value.contains('\\')
        && !value.chars().any(char::is_whitespace)
        && (value.starts_with("https://") || value.starts_with("http://"))
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    #[serde(default)]
    results: Vec<SearxngResult>,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    #[serde(default)]
    url: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn local_policy() -> RetrievalPolicy {
        RetrievalPolicy {
            max_bytes: 64 * 1024,
            timeout_ms: 2_000,
            max_redirects: 1,
            allow_http: true,
            allow_private_network: true,
            ..RetrievalPolicy::default()
        }
    }

    fn spawn_search_server(body: String) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind search server");
        let address = listener.local_addr().expect("search server address");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept search request");
            let mut request = [0u8; 4_096];
            let count = stream.read(&mut request).expect("read search request");
            sender
                .send(String::from_utf8_lossy(&request[..count]).to_string())
                .expect("record search request");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write search response");
        });
        (format!("http://{address}/search"), receiver)
    }

    #[test]
    fn builds_encoded_search_request() {
        let provider = SearxngSearchProvider::new(
            SearxngConfig {
                endpoint: "https://search.example/search?categories=general".to_string(),
                language: "en-US".to_string(),
                safe_search: 1,
            },
            RetrievalPolicy::default(),
        )
        .expect("valid provider");
        let url = provider
            .build_search_url(&WebSearchRequest {
                query: "Parchment mill reports".to_string(),
                max_results: 5,
            })
            .expect("search URL");
        assert!(url.contains("categories=general&q=Parchment%20mill%20reports"));
        assert!(url.contains("format=json"));
        assert!(url.contains("language=en-US"));
    }

    #[test]
    fn parses_filters_deduplicates_and_limits_results() {
        let body = r#"{
            "results": [
                {"url":"https://example.com/a","title":"A","content":"first"},
                {"url":"https://example.com/a","title":"duplicate","content":"again"},
                {"url":"file:///private","title":"blocked","content":"no"},
                {"url":"https://example.com/b","title":"B","content":"second"},
                {"url":"https://example.com/c","title":"C","content":"third"}
            ]
        }"#
        .to_string();
        let (endpoint, request_capture) = spawn_search_server(body);
        let provider = SearxngSearchProvider::new(
            SearxngConfig {
                endpoint,
                language: "en-US".to_string(),
                safe_search: 1,
            },
            local_policy(),
        )
        .expect("valid provider");
        let hits = provider
            .search(&WebSearchRequest {
                query: "environmental report".to_string(),
                max_results: 2,
            })
            .expect("search succeeds");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].rank, 1);
        assert_eq!(hits[0].url, "https://example.com/a");
        assert_eq!(hits[1].rank, 2);
        assert_eq!(hits[1].url, "https://example.com/b");
        let request = request_capture.recv().expect("captured request");
        assert!(request.contains("q=environmental%20report"));
    }
}
