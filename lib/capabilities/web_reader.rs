//! Web Reader Capability
//!
//! Star can retrieve information from the internet and websites
//! to satisfy her curiosity at will.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Result of a web fetch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResult {
    pub url: String,
    pub content: String,
    pub success: bool,
    pub error: Option<String>,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub fetch_time_ms: u64,
}

/// Result of a web search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<SearchItem>,
    pub success: bool,
    pub error: Option<String>,
}

/// Individual search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Web reader for Star - enables internet and website retrieval.
pub struct WebReader {
    client: reqwest::Client,
    timeout_secs: u64,
    max_content_size: usize,
    user_agent: String,
}

impl WebReader {
    /// Create a new web reader.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            timeout_secs: 30,
            max_content_size: 2_000_000, // 2MB max
            user_agent: "Star/1.0 (curious AI assistant)".to_string(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(timeout_secs: u64, max_size: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            timeout_secs,
            max_content_size: max_size,
            user_agent: "Star/1.0 (curious AI assistant)".to_string(),
        }
    }

    /// Fetch a URL and return its content.
    pub async fn fetch(&self, url: &str) -> WebResult {
        let start = std::time::Instant::now();
        
        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return WebResult {
                url: url.to_string(),
                content: String::new(),
                success: false,
                error: Some("URL must start with http:// or https://".to_string()),
                status_code: None,
                content_type: None,
                fetch_time_ms: 0,
            };
        }

        info!("Fetching URL: {}", url);

        match self.client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                if !response.status().is_success() {
                    return WebResult {
                        url: url.to_string(),
                        content: String::new(),
                        success: false,
                        error: Some(format!("HTTP error: {}", status)),
                        status_code: Some(status),
                        content_type,
                        fetch_time_ms: start.elapsed().as_millis() as u64,
                    };
                }

                // Check content length
                if let Some(len) = response.content_length() {
                    if len as usize > self.max_content_size {
                        return WebResult {
                            url: url.to_string(),
                            content: String::new(),
                            success: false,
                            error: Some(format!(
                                "Content too large: {} bytes (max: {})",
                                len, self.max_content_size
                            )),
                            status_code: Some(status),
                            content_type,
                            fetch_time_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }

                // Read content
                match response.text().await {
                    Ok(text) => {
                        let actual_size = text.len();
                        info!(
                            "Successfully fetched {} ({} bytes in {}ms)",
                            url,
                            actual_size,
                            start.elapsed().as_millis()
                        );
                        WebResult {
                            url: url.to_string(),
                            content: text,
                            success: true,
                            error: None,
                            status_code: Some(status),
                            content_type,
                            fetch_time_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                    Err(e) => WebResult {
                        url: url.to_string(),
                        content: String::new(),
                        success: false,
                        error: Some(format!("Failed to read response: {}", e)),
                        status_code: Some(status),
                        content_type,
                        fetch_time_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => {
                warn!("Failed to fetch {}: {}", url, e);
                WebResult {
                    url: url.to_string(),
                    content: String::new(),
                    success: false,
                    error: Some(format!("Connection error: {}", e)),
                    status_code: None,
                    content_type: None,
                    fetch_time_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
    }

    /// Fetch multiple URLs concurrently.
    pub async fn fetch_many(&self, urls: &[String]) -> Vec<WebResult> {
        let mut handles = Vec::new();
        
        for url in urls {
            let url = url.clone();
            let client = self.client.clone();
            let user_agent = self.user_agent.clone();
            let max_size = self.max_content_size;
            
            handles.push(tokio::spawn(async move {
                let start = std::time::Instant::now();
                
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return WebResult {
                        url,
                        content: String::new(),
                        success: false,
                        error: Some("URL must start with http:// or https://".to_string()),
                        status_code: None,
                        content_type: None,
                        fetch_time_ms: 0,
                    };
                }

                match client
                    .get(&url)
                    .header("User-Agent", &user_agent)
                    .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                    .send()
                    .await
                {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        let content_type = response
                            .headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());

                        if !response.status().is_success() {
                            return WebResult {
                                url,
                                content: String::new(),
                                success: false,
                                error: Some(format!("HTTP error: {}", status)),
                                status_code: Some(status),
                                content_type,
                                fetch_time_ms: start.elapsed().as_millis() as u64,
                            };
                        }

                        match response.text().await {
                            Ok(text) if text.len() <= max_size => WebResult {
                                url,
                                content: text,
                                success: true,
                                error: None,
                                status_code: Some(status),
                                content_type,
                                fetch_time_ms: start.elapsed().as_millis() as u64,
                            },
                            Ok(_) => WebResult {
                                url,
                                content: String::new(),
                                success: false,
                                error: Some("Content too large".to_string()),
                                status_code: Some(status),
                                content_type,
                                fetch_time_ms: start.elapsed().as_millis() as u64,
                            },
                            Err(e) => WebResult {
                                url,
                                content: String::new(),
                                success: false,
                                error: Some(format!("Read error: {}", e)),
                                status_code: Some(status),
                                content_type,
                                fetch_time_ms: start.elapsed().as_millis() as u64,
                            },
                        }
                    }
                    Err(e) => WebResult {
                        url,
                        content: String::new(),
                        success: false,
                        error: Some(format!("Connection error: {}", e)),
                        status_code: None,
                        content_type: None,
                        fetch_time_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap_or_else(|_| WebResult {
                url: String::new(),
                content: String::new(),
                success: false,
                error: Some("Task panicked".to_string()),
                status_code: None,
                content_type: None,
                fetch_time_ms: 0,
            }));
        }
        results
    }

    /// Search the web using DuckDuckGo (no API key required).
    pub async fn search(&self, query: &str) -> SearchResult {
        // Using DuckDuckGo HTML search (no API key needed)
        let search_url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        info!("Searching for: {}", query);

        let result = self.fetch(&search_url).await;

        if !result.success {
            return SearchResult {
                query: query.to_string(),
                results: vec![],
                success: false,
                error: result.error,
            };
        }

        // Parse HTML results
        let results = self.parse_search_results(&result.content);
        
        SearchResult {
            query: query.to_string(),
            results,
            success: true,
            error: None,
        }
    }

    /// Parse DuckDuckGo HTML search results.
    fn parse_search_results(&self, html: &str) -> Vec<SearchItem> {
        let mut results = Vec::new();
        
        // Simple regex-based parsing for DuckDuckGo results
        // Each result is in a <a> tag with class "result__a"
        let re = regex::Regex::new(r#"<a[^>]+class="result__a"[^>]+href="([^"]+)"[^>]*>([^<]+)</a>"#).ok();
        
        if let Some(re) = re {
            for cap in re.captures_iter(html) {
                if let (Some(url), Some(title)) = (cap.get(1), cap.get(2)) {
                    // Extract actual URL from DuckDuckGo redirect
                    let url_str = url.as_str();
                    let actual_url = if url_str.contains("uddg=") {
                        url_str
                            .split("uddg=")
                            .nth(1)
                            .unwrap_or(url_str)
                            .split('&')
                            .next()
                            .unwrap_or(url_str)
                            .to_string()
                    } else {
                        url_str.to_string()
                    };

                    results.push(SearchItem {
                        title: title.as_str().to_string(),
                        url: actual_url,
                        snippet: String::new(), // Would need more parsing for snippets
                    });
                }
            }
        }

        // Limit results
        results.truncate(10);
        results
    }

    /// Extract readable text from HTML (simple strip).
    pub fn extract_text(&self, html: &str) -> String {
        // Simple HTML tag removal
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(html, " ");
        
        // Clean up whitespace
        let re = regex::Regex::new(r"\s+").unwrap();
        re.replace_all(&text, " ").trim().to_string()
    }

    /// Get a summary of a URL (title + first paragraph).
    pub async fn summarize(&self, url: &str) -> WebResult {
        let result = self.fetch(url).await;
        
        if !result.success {
            return result;
        }

        // Extract title
        let title_re = regex::Regex::new(r"<title>([^<]+)</title>").ok();
        let title = title_re
            .and_then(|re| re.captures(&result.content))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| url.to_string());

        // Extract first few paragraphs
        let text = self.extract_text(&result.content);
        let summary = text.chars().take(500).collect::<String>();

        WebResult {
            url: result.url,
            content: format!("Title: {}\n\nSummary: {}...", title, summary),
            success: true,
            error: None,
            status_code: result.status_code,
            content_type: result.content_type,
            fetch_time_ms: result.fetch_time_ms,
        }
    }
}

impl Default for WebReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WebReader {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            timeout_secs: self.timeout_secs,
            max_content_size: self.max_content_size,
            user_agent: self.user_agent.clone(),
        }
    }
}