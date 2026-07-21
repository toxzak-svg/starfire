//! Bounded, local-first web search and retrieval primitives.
//!
//! The feature is disabled by default. It can validate search contracts, fetch an
//! authorized HTTP(S) URL, follow validated redirects, classify and retain exact
//! bytes, and atomically save a caller-approved download. It cannot execute files,
//! render JavaScript, mutate memory, enter `Runtime::chat()`, or authorize actions.

use crate::now_timestamp;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;
use url::Url;

const HARD_MAX_BYTES: usize = 128 * 1024 * 1024;
const HARD_MAX_REDIRECTS: usize = 10;
const HARD_MAX_URL_BYTES: usize = 8 * 1024;
const HARD_MAX_PURPOSE_BYTES: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebSearchRequest {
    pub query: String,
    pub max_results: u8,
}

impl WebSearchRequest {
    pub fn validate(&self) -> Result<(), RetrievalError> {
        let query = self.query.trim();
        if query.is_empty() {
            return Err(RetrievalError::InvalidSearchQuery);
        }
        if query.len() > 1_024 {
            return Err(RetrievalError::SearchQueryTooLong { bytes: query.len() });
        }
        if !(1..=50).contains(&self.max_results) {
            return Err(RetrievalError::InvalidSearchResultLimit {
                requested: self.max_results,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebSearchHit {
    pub rank: u16,
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
}

/// Provider-neutral boundary for Brave, Bing, Exa, Tavily, or a self-hosted index.
pub trait SearchProvider: Send + Sync {
    fn search(&self, request: &WebSearchRequest) -> Result<Vec<WebSearchHit>, RetrievalError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebFetchRequest {
    pub url: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalPolicy {
    pub max_bytes: usize,
    pub timeout_ms: u64,
    pub max_redirects: usize,
    pub allow_http: bool,
    pub allow_private_network: bool,
    pub user_agent: String,
}

impl Default for RetrievalPolicy {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            timeout_ms: 20_000,
            max_redirects: 5,
            allow_http: false,
            allow_private_network: false,
            user_agent: "Starfire-Retrieval/0.1".to_string(),
        }
    }
}

impl RetrievalPolicy {
    pub fn validate(&self) -> Result<(), RetrievalError> {
        if self.max_bytes == 0 || self.max_bytes > HARD_MAX_BYTES {
            return Err(RetrievalError::InvalidByteLimit {
                requested: self.max_bytes,
                hard_max: HARD_MAX_BYTES,
            });
        }
        if self.timeout_ms == 0 || self.timeout_ms > 120_000 {
            return Err(RetrievalError::InvalidTimeout {
                requested_ms: self.timeout_ms,
            });
        }
        if self.max_redirects > HARD_MAX_REDIRECTS {
            return Err(RetrievalError::InvalidRedirectLimit {
                requested: self.max_redirects,
                hard_max: HARD_MAX_REDIRECTS,
            });
        }
        if self.user_agent.trim().is_empty() || self.user_agent.len() > 256 {
            return Err(RetrievalError::InvalidUserAgent);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievedKind {
    Html,
    Json,
    PlainText,
    Pdf,
    Image,
    Archive,
    Audio,
    Video,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievedResource {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub content_length_header: Option<u64>,
    pub kind: RetrievedKind,
    pub suggested_filename: String,
    pub fingerprint_fnv1a64: String,
    pub retrieved_at_unix: i64,
    pub redirect_count: usize,
    pub bytes: Vec<u8>,
}

impl RetrievedResource {
    pub fn as_text(&self) -> Result<&str, RetrievalError> {
        match self.kind {
            RetrievedKind::Html | RetrievedKind::Json | RetrievedKind::PlainText => {
                std::str::from_utf8(&self.bytes).map_err(|_| RetrievalError::InvalidUtf8)
            }
            _ => Err(RetrievalError::NotTextual { kind: self.kind }),
        }
    }

    /// Saves without overwriting. The temporary file is created beside the target
    /// and renamed only after all bytes have been flushed.
    pub fn save_atomic(&self, target: impl AsRef<Path>) -> Result<PathBuf, RetrievalError> {
        let target = target.as_ref();
        if target.exists() {
            return Err(RetrievalError::TargetExists {
                path: target.display().to_string(),
            });
        }
        let parent = target.parent().ok_or_else(|| RetrievalError::InvalidTargetPath {
            path: target.display().to_string(),
        })?;
        fs::create_dir_all(parent).map_err(|source| RetrievalError::Io {
            operation: "create target directory",
            source,
        })?;
        let name = target
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| RetrievalError::InvalidTargetPath {
                path: target.display().to_string(),
            })?;
        let temporary = parent.join(format!(".{name}.{}.part", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| RetrievalError::Io {
                operation: "create temporary download",
                source,
            })?;
        let result = (|| {
            file.write_all(&self.bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, target)?;
            Ok::<(), std::io::Error>(())
        })();
        if let Err(source) = result {
            let _ = fs::remove_file(&temporary);
            return Err(RetrievalError::Io {
                operation: "commit download",
                source,
            });
        }
        Ok(target.to_path_buf())
    }
}

#[derive(Debug, Error)]
pub enum RetrievalError {
    #[error("search query cannot be empty")]
    InvalidSearchQuery,
    #[error("search query is too long: {bytes} bytes")]
    SearchQueryTooLong { bytes: usize },
    #[error("search result limit must be between 1 and 50, got {requested}")]
    InvalidSearchResultLimit { requested: u8 },
    #[error("retrieval byte limit {requested} is outside the supported range; hard max is {hard_max}")]
    InvalidByteLimit { requested: usize, hard_max: usize },
    #[error("retrieval timeout must be between 1 and 120000 ms, got {requested_ms}")]
    InvalidTimeout { requested_ms: u64 },
    #[error("redirect limit {requested} exceeds hard max {hard_max}")]
    InvalidRedirectLimit { requested: usize, hard_max: usize },
    #[error("user agent must contain between 1 and 256 bytes")]
    InvalidUserAgent,
    #[error("URL is empty or exceeds {HARD_MAX_URL_BYTES} bytes")]
    InvalidUrlLength,
    #[error("retrieval purpose is empty or exceeds {HARD_MAX_PURPOSE_BYTES} bytes")]
    InvalidPurpose,
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("URL scheme is not allowed: {0}")]
    UnsupportedScheme(String),
    #[error("URL credentials are not allowed")]
    EmbeddedCredentials,
    #[error("URL has no host")]
    MissingHost,
    #[error("private or local network destination is blocked: {0}")]
    PrivateNetworkBlocked(String),
    #[error("DNS resolution failed for {host}: {message}")]
    DnsResolution { host: String, message: String },
    #[error("transport failed for {url}: {message}")]
    Transport { url: String, message: String },
    #[error("HTTP request failed with status {status} for {url}")]
    HttpStatus { url: String, status: u16 },
    #[error("redirect response from {url} did not contain a Location header")]
    RedirectMissingLocation { url: String },
    #[error("redirect limit exceeded after {limit} hops")]
    RedirectLimitExceeded { limit: usize },
    #[error("response declares {declared} bytes, exceeding limit {limit}")]
    DeclaredBodyTooLarge { declared: u64, limit: usize },
    #[error("response exceeded byte limit {limit}")]
    BodyTooLarge { limit: usize },
    #[error("response body is not valid UTF-8")]
    InvalidUtf8,
    #[error("resource kind {kind:?} is not textual")]
    NotTextual { kind: RetrievedKind },
    #[error("target path already exists: {path}")]
    TargetExists { path: String },
    #[error("invalid target path: {path}")]
    InvalidTargetPath { path: String },
    #[error("I/O failed while attempting to {operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("search provider failed: {0}")]
    SearchProvider(String),
}

#[derive(Clone)]
pub struct BoundedRetriever {
    policy: RetrievalPolicy,
    agent: ureq::Agent,
}

impl BoundedRetriever {
    pub fn new(policy: RetrievalPolicy) -> Result<Self, RetrievalError> {
        policy.validate()?;
        let timeout = Duration::from_millis(policy.timeout_ms);
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(timeout)
            .timeout_read(timeout)
            .timeout_write(timeout)
            .redirects(0)
            .build();
        Ok(Self { policy, agent })
    }

    pub fn policy(&self) -> &RetrievalPolicy {
        &self.policy
    }

    pub fn fetch(&self, request: &WebFetchRequest) -> Result<RetrievedResource, RetrievalError> {
        validate_request(request)?;
        let requested_url = request.url.clone();
        let mut current = parse_url(&request.url, &self.policy)?;
        let mut redirects = 0usize;

        loop {
            validate_destination(&current, &self.policy)?;
            let response = match self
                .agent
                .get(current.as_str())
                .set("User-Agent", &self.policy.user_agent)
                .set(
                    "Accept",
                    "text/html,application/xhtml+xml,application/json,text/plain,application/pdf,*/*;q=0.5",
                )
                .call()
            {
                Ok(response) => response,
                Err(ureq::Error::Status(status, _)) => {
                    return Err(RetrievalError::HttpStatus {
                        url: current.to_string(),
                        status,
                    });
                }
                Err(ureq::Error::Transport(error)) => {
                    return Err(RetrievalError::Transport {
                        url: current.to_string(),
                        message: error.to_string(),
                    });
                }
            };

            let status = response.status();
            if is_redirect(status) {
                if redirects >= self.policy.max_redirects {
                    return Err(RetrievalError::RedirectLimitExceeded {
                        limit: self.policy.max_redirects,
                    });
                }
                let location = response
                    .header("Location")
                    .ok_or_else(|| RetrievalError::RedirectMissingLocation {
                        url: current.to_string(),
                    })?;
                current = current
                    .join(location)
                    .map_err(|error| RetrievalError::InvalidUrl(error.to_string()))?;
                redirects += 1;
                continue;
            }

            let content_type = response.header("Content-Type").map(str::to_string);
            let content_length_header = response
                .header("Content-Length")
                .and_then(|value| value.parse::<u64>().ok());
            if let Some(declared) = content_length_header {
                if declared > self.policy.max_bytes as u64 {
                    return Err(RetrievalError::DeclaredBodyTooLarge {
                        declared,
                        limit: self.policy.max_bytes,
                    });
                }
            }

            let mut bytes = Vec::with_capacity(
                content_length_header
                    .unwrap_or(0)
                    .min(self.policy.max_bytes as u64) as usize,
            );
            response
                .into_reader()
                .take(self.policy.max_bytes as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|source| RetrievalError::Io {
                    operation: "read response body",
                    source,
                })?;
            if bytes.len() > self.policy.max_bytes {
                return Err(RetrievalError::BodyTooLarge {
                    limit: self.policy.max_bytes,
                });
            }

            let kind = classify_content(content_type.as_deref(), &bytes);
            return Ok(RetrievedResource {
                requested_url,
                final_url: current.to_string(),
                status,
                content_type: content_type.clone(),
                content_length_header,
                kind,
                suggested_filename: suggested_filename(&current, content_type.as_deref(), kind),
                fingerprint_fnv1a64: fnv1a64_hex(&bytes),
                retrieved_at_unix: now_timestamp(),
                redirect_count: redirects,
                bytes,
            });
        }
    }
}

fn validate_request(request: &WebFetchRequest) -> Result<(), RetrievalError> {
    if request.url.trim().is_empty() || request.url.len() > HARD_MAX_URL_BYTES {
        return Err(RetrievalError::InvalidUrlLength);
    }
    if request.purpose.trim().is_empty() || request.purpose.len() > HARD_MAX_PURPOSE_BYTES {
        return Err(RetrievalError::InvalidPurpose);
    }
    Ok(())
}

fn parse_url(raw: &str, policy: &RetrievalPolicy) -> Result<Url, RetrievalError> {
    let url = Url::parse(raw).map_err(|error| RetrievalError::InvalidUrl(error.to_string()))?;
    match url.scheme() {
        "https" => {}
        "http" if policy.allow_http => {}
        other => return Err(RetrievalError::UnsupportedScheme(other.to_string())),
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(RetrievalError::EmbeddedCredentials);
    }
    if url.host_str().is_none() {
        return Err(RetrievalError::MissingHost);
    }
    Ok(url)
}

fn validate_destination(url: &Url, policy: &RetrievalPolicy) -> Result<(), RetrievalError> {
    let checked = parse_url(url.as_str(), policy)?;
    if policy.allow_private_network {
        return Ok(());
    }
    let host = checked.host_str().ok_or(RetrievalError::MissingHost)?;
    let normalized = host.trim_end_matches('.').to_ascii_lowercase();
    if normalized == "localhost"
        || normalized.ends_with(".localhost")
        || normalized.ends_with(".local")
        || normalized.ends_with(".internal")
    {
        return Err(RetrievalError::PrivateNetworkBlocked(host.to_string()));
    }
    if let Ok(ip) = IpAddr::from_str(host) {
        return reject_non_public_ip(ip, host);
    }

    let port = checked.port_or_known_default().ok_or_else(|| {
        RetrievalError::InvalidUrl("URL does not resolve to a known port".to_string())
    })?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| RetrievalError::DnsResolution {
            host: host.to_string(),
            message: error.to_string(),
        })?;
    let mut resolved_any = false;
    for address in addresses {
        resolved_any = true;
        reject_non_public_ip(address.ip(), host)?;
    }
    if !resolved_any {
        return Err(RetrievalError::DnsResolution {
            host: host.to_string(),
            message: "no addresses returned".to_string(),
        });
    }
    Ok(())
}

fn reject_non_public_ip(ip: IpAddr, display: &str) -> Result<(), RetrievalError> {
    let blocked = match ip {
        IpAddr::V4(value) => is_blocked_ipv4(value),
        IpAddr::V6(value) => is_blocked_ipv6(value),
    };
    if blocked {
        Err(RetrievalError::PrivateNetworkBlocked(display.to_string()))
    } else {
        Ok(())
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, d] = ip.octets();
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || a == 0
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 88 && c == 99)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 255 && b == 255 && c == 255 && d == 255)
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    let first = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (first & 0xfe00) == 0xfc00
        || (first & 0xffc0) == 0xfe80
        || ip.to_ipv4().map(is_blocked_ipv4).unwrap_or(false)
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

pub fn classify_content(content_type: Option<&str>, bytes: &[u8]) -> RetrievedKind {
    if bytes.starts_with(b"%PDF-") {
        return RetrievedKind::Pdf;
    }
    if bytes.starts_with(b"PK\x03\x04")
        || bytes.starts_with(b"\x1f\x8b")
        || bytes.starts_with(b"Rar!\x1a\x07")
        || bytes.starts_with(b"7z\xbc\xaf\x27\x1c")
    {
        return RetrievedKind::Archive;
    }
    let webp = bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(&b"WEBP"[..]);
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"\xff\xd8\xff")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
        || webp
    {
        return RetrievedKind::Image;
    }

    let mime = normalized_mime(content_type);
    match mime.as_str() {
        "text/html" | "application/xhtml+xml" => RetrievedKind::Html,
        "application/json" | "application/ld+json" | "application/geo+json" => {
            RetrievedKind::Json
        }
        value if value.starts_with("text/") => RetrievedKind::PlainText,
        "application/pdf" => RetrievedKind::Pdf,
        value if value.starts_with("image/") => RetrievedKind::Image,
        value if value.starts_with("audio/") => RetrievedKind::Audio,
        value if value.starts_with("video/") => RetrievedKind::Video,
        "application/zip"
        | "application/x-zip-compressed"
        | "application/gzip"
        | "application/x-gzip"
        | "application/x-7z-compressed"
        | "application/vnd.rar" => RetrievedKind::Archive,
        _ => classify_unlabelled(bytes),
    }
}

fn classify_unlabelled(bytes: &[u8]) -> RetrievedKind {
    let trimmed = bytes
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .take(64)
        .collect::<Vec<_>>();
    if trimmed.starts_with(b"<!DOCTYPE html")
        || trimmed.starts_with(b"<!doctype html")
        || trimmed.starts_with(b"<html")
    {
        RetrievedKind::Html
    } else if trimmed.starts_with(b"{") || trimmed.starts_with(b"[") {
        RetrievedKind::Json
    } else if std::str::from_utf8(bytes).is_ok() {
        RetrievedKind::PlainText
    } else {
        RetrievedKind::Binary
    }
}

fn normalized_mime(content_type: Option<&str>) -> String {
    content_type
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn suggested_filename(url: &Url, content_type: Option<&str>, kind: RetrievedKind) -> String {
    if let Some(name) = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|segment| !segment.is_empty())
        .map(sanitize_filename)
        .filter(|name| !name.is_empty())
    {
        return name;
    }
    format!(
        "download-{}.{}",
        now_timestamp(),
        extension_for(content_type, kind)
    )
}

fn extension_for(content_type: Option<&str>, kind: RetrievedKind) -> &'static str {
    match normalized_mime(content_type).as_str() {
        "text/html" | "application/xhtml+xml" => "html",
        "application/json" | "application/ld+json" | "application/geo+json" => "json",
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "application/zip" | "application/x-zip-compressed" => "zip",
        "application/gzip" | "application/x-gzip" => "gz",
        _ => match kind {
            RetrievedKind::Html => "html",
            RetrievedKind::Json => "json",
            RetrievedKind::PlainText => "txt",
            RetrievedKind::Pdf => "pdf",
            RetrievedKind::Image => "img",
            RetrievedKind::Archive => "archive",
            RetrievedKind::Audio => "audio",
            RetrievedKind::Video => "video",
            RetrievedKind::Binary => "bin",
        },
    }
}

fn sanitize_filename(input: &str) -> String {
    let mut output = String::with_capacity(input.len().min(128));
    for character in input.chars().take(128) {
        if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | ' ') {
            output.push(character);
        } else {
            output.push('_');
        }
    }
    output
        .trim_matches(|character: char| character == '.' || character == ' ')
        .to_string()
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn local_policy(max_bytes: usize) -> RetrievalPolicy {
        RetrievalPolicy {
            max_bytes,
            timeout_ms: 2_000,
            max_redirects: 2,
            allow_http: true,
            allow_private_network: true,
            ..RetrievalPolicy::default()
        }
    }

    fn spawn_server(responses: Vec<String>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("server address");
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut request = [0u8; 2_048];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).expect("write response");
            }
        });
        format!("http://{address}")
    }

    #[test]
    fn search_contract_is_bounded() {
        assert!(WebSearchRequest {
            query: "public records".to_string(),
            max_results: 10,
        }
        .validate()
        .is_ok());
        assert!(matches!(
            WebSearchRequest {
                query: " ".to_string(),
                max_results: 10,
            }
            .validate(),
            Err(RetrievalError::InvalidSearchQuery)
        ));
    }

    #[test]
    fn private_network_is_blocked_by_default() {
        let retriever = BoundedRetriever::new(RetrievalPolicy {
            allow_http: true,
            ..RetrievalPolicy::default()
        })
        .expect("valid policy");
        let result = retriever.fetch(&WebFetchRequest {
            url: "http://127.0.0.1/private".to_string(),
            purpose: "test private destination protection".to_string(),
        });
        assert!(matches!(
            result,
            Err(RetrievalError::PrivateNetworkBlocked(_))
        ));
    }

    #[test]
    fn redirect_fetches_and_classifies_html() {
        let body = "<html><body>Starfire</body></html>";
        let base = spawn_server(vec![
            "HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_string(),
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            ),
        ]);
        let result = BoundedRetriever::new(local_policy(1_024))
            .expect("valid policy")
            .fetch(&WebFetchRequest {
                url: format!("{base}/start"),
                purpose: "retrieve public test page".to_string(),
            })
            .expect("fetch succeeds");
        assert_eq!(result.redirect_count, 1);
        assert_eq!(result.kind, RetrievedKind::Html);
        assert!(result.as_text().expect("text").contains("Starfire"));
    }

    #[test]
    fn body_limit_is_enforced_without_content_length() {
        let body = "x".repeat(65);
        let base = spawn_server(vec![format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n{body}"
        )]);
        let result = BoundedRetriever::new(local_policy(64))
            .expect("valid policy")
            .fetch(&WebFetchRequest {
                url: format!("{base}/large.bin"),
                purpose: "test bounded body".to_string(),
            });
        assert!(matches!(
            result,
            Err(RetrievalError::BodyTooLarge { limit: 64 })
        ));
    }

    #[test]
    fn magic_bytes_override_incorrect_mime() {
        assert_eq!(
            classify_content(Some("text/plain"), b"%PDF-1.7\n"),
            RetrievedKind::Pdf
        );
    }

    #[test]
    fn atomic_save_refuses_overwrite() {
        let directory = std::env::temp_dir().join(format!(
            "starfire-web-retrieval-{}-{}",
            std::process::id(),
            now_timestamp()
        ));
        let path = directory.join("document.txt");
        let resource = RetrievedResource {
            requested_url: "https://example.com/document.txt".to_string(),
            final_url: "https://example.com/document.txt".to_string(),
            status: 200,
            content_type: Some("text/plain".to_string()),
            content_length_header: Some(4),
            kind: RetrievedKind::PlainText,
            suggested_filename: "document.txt".to_string(),
            fingerprint_fnv1a64: fnv1a64_hex(b"test"),
            retrieved_at_unix: now_timestamp(),
            redirect_count: 0,
            bytes: b"test".to_vec(),
        };
        resource.save_atomic(&path).expect("first save succeeds");
        assert!(matches!(
            resource.save_atomic(&path),
            Err(RetrievalError::TargetExists { .. })
        ));
        let _ = fs::remove_dir_all(directory);
    }
}
