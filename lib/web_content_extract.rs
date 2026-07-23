//! Deterministic text extraction and evidence chunking for retrieved web content.
//!
//! This first extractor handles HTML, JSON, and plain text without executing page
//! code. PDFs and office documents remain inert downloads until dedicated parsers
//! are added. Extraction does not mutate memory or influence live chat by itself.

use crate::web_retrieval::{RetrievedKind, RetrievedResource};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;
use thiserror::Error;

const HARD_MAX_TEXT_BYTES: usize = 32 * 1024 * 1024;
const HARD_MAX_CHUNK_BYTES: usize = 64 * 1024;
const HARD_MAX_LINKS: usize = 8_192;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractionPolicy {
    pub max_text_bytes: usize,
    pub target_chunk_bytes: usize,
    pub max_links: usize,
}

impl Default for ExtractionPolicy {
    fn default() -> Self {
        Self {
            max_text_bytes: 4 * 1024 * 1024,
            target_chunk_bytes: 4 * 1024,
            max_links: 512,
        }
    }
}

impl ExtractionPolicy {
    pub fn validate(&self) -> Result<(), ExtractionError> {
        if self.max_text_bytes == 0 || self.max_text_bytes > HARD_MAX_TEXT_BYTES {
            return Err(ExtractionError::InvalidPolicy(format!(
                "max_text_bytes must be between 1 and {HARD_MAX_TEXT_BYTES}"
            )));
        }
        if self.target_chunk_bytes < 256 || self.target_chunk_bytes > HARD_MAX_CHUNK_BYTES {
            return Err(ExtractionError::InvalidPolicy(format!(
                "target_chunk_bytes must be between 256 and {HARD_MAX_CHUNK_BYTES}"
            )));
        }
        if self.max_links > HARD_MAX_LINKS {
            return Err(ExtractionError::InvalidPolicy(format!(
                "max_links cannot exceed {HARD_MAX_LINKS}"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceChunk {
    pub ordinal: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub source_url: String,
    pub source_kind: RetrievedKind,
    pub title: Option<String>,
    pub text: String,
    pub links: Vec<String>,
    pub chunks: Vec<EvidenceChunk>,
    pub source_fingerprint: String,
}

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("invalid extraction policy: {0}")]
    InvalidPolicy(String),
    #[error("retrieved {0:?} content is not supported by the text extractor")]
    UnsupportedKind(RetrievedKind),
    #[error("retrieved text is not valid UTF-8")]
    InvalidUtf8,
    #[error("retrieved JSON could not be parsed: {0}")]
    InvalidJson(String),
}

pub fn extract_document(
    resource: &RetrievedResource,
    policy: &ExtractionPolicy,
) -> Result<ExtractedDocument, ExtractionError> {
    policy.validate()?;
    let raw = std::str::from_utf8(&resource.bytes).map_err(|_| ExtractionError::InvalidUtf8)?;
    let (title, text, links) = match resource.kind {
        RetrievedKind::Html => extract_html(raw, policy.max_links),
        RetrievedKind::PlainText => (None, normalize_text(raw), Vec::new()),
        RetrievedKind::Json => {
            let value: serde_json::Value = serde_json::from_str(raw)
                .map_err(|error| ExtractionError::InvalidJson(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&value)
                .map_err(|error| ExtractionError::InvalidJson(error.to_string()))?;
            (None, rendered, Vec::new())
        }
        kind => return Err(ExtractionError::UnsupportedKind(kind)),
    };

    let text = truncate_utf8(text, policy.max_text_bytes);
    let chunks = chunk_text(&text, policy.target_chunk_bytes);
    Ok(ExtractedDocument {
        source_url: resource.final_url.clone(),
        source_kind: resource.kind,
        title: title.filter(|value| !value.is_empty()),
        text,
        links,
        chunks,
        source_fingerprint: resource.fingerprint_fnv1a64.clone(),
    })
}

fn extract_html(html: &str, max_links: usize) -> (Option<String>, String, Vec<String>) {
    let title = title_regex()
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|value| normalize_text(&decode_entities(value.as_str())));
    let links = extract_links(html, max_links);

    let without_comments = comment_regex().replace_all(html, " ");
    let without_hidden = hidden_block_regex().replace_all(&without_comments, " ");
    let with_boundaries = block_tag_regex().replace_all(&without_hidden, "\n");
    let without_tags = all_tag_regex().replace_all(&with_boundaries, " ");
    let text = normalize_text(&decode_entities(&without_tags));
    (title, text, links)
}

fn extract_links(html: &str, max_links: usize) -> Vec<String> {
    if max_links == 0 {
        return Vec::new();
    }
    let mut seen = HashSet::new();
    let mut links = Vec::with_capacity(max_links.min(64));
    for captures in href_regex().captures_iter(html) {
        let Some(value) = captures
            .get(1)
            .or_else(|| captures.get(2))
            .or_else(|| captures.get(3))
        else {
            continue;
        };
        let value = decode_entities(value.as_str());
        let value = value.trim();
        if value.is_empty()
            || value.starts_with('#')
            || value.starts_with("javascript:")
            || value.starts_with("data:")
            || value.chars().any(char::is_control)
            || !seen.insert(value.to_string())
        {
            continue;
        }
        links.push(value.to_string());
        if links.len() >= max_links {
            break;
        }
    }
    links
}

fn normalize_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pending_break = false;
    for line in input.lines() {
        let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            if !output.is_empty() {
                pending_break = true;
            }
            continue;
        }
        if !output.is_empty() {
            if pending_break {
                output.push_str("\n\n");
            } else {
                output.push('\n');
            }
        }
        output.push_str(&collapsed);
        pending_break = false;
    }
    output
}

fn decode_entities(input: &str) -> String {
    let named = input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    numeric_entity_regex()
        .replace_all(&named, |captures: &Captures<'_>| {
            decode_numeric_entity(
                captures
                    .get(1)
                    .map(|value| value.as_str())
                    .unwrap_or_default(),
            )
            .unwrap_or_else(|| captures.get(0).unwrap().as_str().to_string())
        })
        .into_owned()
}

fn decode_numeric_entity(value: &str) -> Option<String> {
    let number = if let Some(hex) = value.strip_prefix('x').or_else(|| value.strip_prefix('X')) {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        value.parse::<u32>().ok()?
    };
    char::from_u32(number).map(|character| character.to_string())
}

fn truncate_utf8(mut value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    let mut boundary = max_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
    value
}

fn chunk_text(text: &str, target_bytes: usize) -> Vec<EvidenceChunk> {
    let mut chunks = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        while cursor < text.len() && text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= text.len() {
            break;
        }

        let mut end = (cursor + target_bytes).min(text.len());
        while end > cursor && !text.is_char_boundary(end) {
            end -= 1;
        }
        if end < text.len() {
            let minimum = cursor + target_bytes / 2;
            if let Some(relative) = text[cursor..end]
                .char_indices()
                .rev()
                .find(|(index, character)| cursor + index >= minimum && character.is_whitespace())
                .map(|(index, _)| index)
            {
                end = cursor + relative;
            }
        }
        if end <= cursor {
            end = next_char_boundary(text, cursor);
        }

        let raw = &text[cursor..end];
        let leading = raw.len() - raw.trim_start().len();
        let trailing = raw.len() - raw.trim_end().len();
        let start_byte = cursor + leading;
        let end_byte = end.saturating_sub(trailing);
        if start_byte < end_byte {
            chunks.push(EvidenceChunk {
                ordinal: chunks.len() as u32 + 1,
                start_byte,
                end_byte,
                text: text[start_byte..end_byte].to_string(),
            });
        }
        cursor = end;
    }
    chunks
}

fn next_char_boundary(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .nth(1)
        .map(|(index, _)| start + index)
        .unwrap_or(text.len())
}

fn title_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"(?is)<title\b[^>]*>(.*?)</title\s*>").unwrap())
}

fn comment_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"(?is)<!--.*?-->").unwrap())
}

fn hidden_block_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(
            r"(?is)<(?:script|style|noscript|template|svg|canvas)\b[^>]*>.*?</(?:script|style|noscript|template|svg|canvas)\s*>",
        )
        .unwrap()
    })
}

fn block_tag_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(
            r"(?is)</?(?:p|div|section|article|main|header|footer|nav|aside|h[1-6]|li|ul|ol|table|tr|td|th|blockquote|pre|br|hr)\b[^>]*>",
        )
        .unwrap()
    })
}

fn all_tag_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").unwrap())
}

fn href_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r#"(?is)\bhref\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap()
    })
}

fn numeric_entity_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"&#(x[0-9A-Fa-f]+|[0-9]+);").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(kind: RetrievedKind, body: &str) -> RetrievedResource {
        RetrievedResource {
            requested_url: "https://example.com/source".to_string(),
            final_url: "https://example.com/source".to_string(),
            status: 200,
            content_type: None,
            content_length_header: Some(body.len() as u64),
            kind,
            suggested_filename: "source.html".to_string(),
            fingerprint_fnv1a64: "abc123".to_string(),
            retrieved_at_unix: 0,
            redirect_count: 0,
            bytes: body.as_bytes().to_vec(),
        }
    }

    #[test]
    fn extracts_visible_html_and_links_without_script_text() {
        let html = r#"
            <html><head><title>Example &amp; Evidence</title>
            <style>.secret { display:none }</style></head>
            <body><main><h1>Public report</h1><p>Finding &#x41;.</p>
            <script>ignore('instructions')</script>
            <a href="/report.pdf">PDF</a><a href="javascript:alert(1)">bad</a>
            </main></body></html>
        "#;
        let document = extract_document(
            &resource(RetrievedKind::Html, html),
            &ExtractionPolicy::default(),
        )
        .expect("extract HTML");
        assert_eq!(document.title.as_deref(), Some("Example & Evidence"));
        assert!(document.text.contains("Public report"));
        assert!(document.text.contains("Finding A."));
        assert!(!document.text.contains("ignore"));
        assert_eq!(document.links, vec!["/report.pdf"]);
        assert_eq!(document.chunks[0].text, document.text);
    }

    #[test]
    fn chunks_preserve_exact_byte_ranges() {
        let body = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda";
        let policy = ExtractionPolicy {
            max_text_bytes: 1_024,
            target_chunk_bytes: 16,
            max_links: 0,
        };
        let error = extract_document(&resource(RetrievedKind::PlainText, body), &policy)
            .expect_err("chunk limit below policy minimum must fail");
        assert!(matches!(error, ExtractionError::InvalidPolicy(_)));

        let policy = ExtractionPolicy {
            max_text_bytes: 1_024,
            target_chunk_bytes: 256,
            max_links: 0,
        };
        let document = extract_document(
            &resource(RetrievedKind::PlainText, &body.repeat(12)),
            &policy,
        )
        .expect("extract text");
        for chunk in &document.chunks {
            assert_eq!(&document.text[chunk.start_byte..chunk.end_byte], chunk.text);
        }
    }

    #[test]
    fn rejects_binary_content() {
        let error = extract_document(
            &resource(RetrievedKind::Pdf, "%PDF-1.7"),
            &ExtractionPolicy::default(),
        )
        .expect_err("PDF parser is intentionally absent");
        assert!(matches!(
            error,
            ExtractionError::UnsupportedKind(RetrievedKind::Pdf)
        ));
    }
}
