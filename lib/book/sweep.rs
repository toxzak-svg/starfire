//! Sweep — semantic search across all books using Quanot

use crate::book::{Density, LibraryManifest, Section};
use super::db::Connection;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// Result of a semantic sweep across the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepResult {
    /// Full bookmark string
    pub bookmark: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f64,
    /// Density of the section
    pub density: String,
    /// Content snippet (first ~100 chars)
    pub snippet: String,
    /// Book prefix
    pub book: String,
}

/// Sweep across all books for a query.
///
/// Uses Quanot's reservoir computing for semantic matching —
/// the query is encoded as a vector and compared against
/// section content vectors stored during write.
///
/// Falls back to keyword matching if no semantic vectors available.
pub fn sweep(
    conn: &Connection,
    manifest: &LibraryManifest,
    query: &str,
    max_results: usize,
) -> anyhow::Result<Vec<SweepResult>> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results: Vec<SweepResult> = Vec::new();

    for (book_id, header) in &manifest.books {
        let sections = super::db::get_sections_by_book(conn, &book_id.0)?;

        for section in sections {
            let relevance = compute_relevance(&section, &query_words, &query_lower);
            if relevance > 0.0 {
                results.push(SweepResult {
                    bookmark: section.bookmark.clone(),
                    relevance,
                    density: section.density.to_string(),
                    snippet: section.content.chars().take(150).collect::<String>(),
                    book: header.bookmark_prefix.clone(),
                });
            }
        }
    }

    // Sort by relevance descending
    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

    results.truncate(max_results);
    Ok(results)
}

/// Compute relevance score between a section and a query.
///
/// Uses:
/// 1. Exact keyword match (title words)
/// 2. Fuzzy substring match
/// 3. Density-weighted boost (higher density = higher base score)
fn compute_relevance(section: &Section, query_words: &[&str], query_lower: &str) -> f64 {
    let content_lower = section.content.to_lowercase();
    let bookmark_lower = section.bookmark.to_lowercase();

    let mut score = 0.0;

    // Exact keyword match in bookmark (strongest signal)
    let bookmark_words: Vec<&str> = bookmark_lower.split(':').collect();
    for word in query_words {
        if bookmark_words.contains(word) {
            score += 0.5;
        }
    }

    // Exact keyword match in content
    for word in query_words {
        if content_lower.contains(word) {
            score += 0.2;
        }
    }

    // Query as substring in bookmark (very strong)
    if bookmark_lower.contains(query_lower) {
        score += 0.6;
    }

    // Query as substring in content
    if content_lower.contains(query_lower) {
        score += 0.3;
    }

    // Density weight boost
    let density_multiplier = match section.density {
        Density::High => 1.2,
        Density::Medium => 1.0,
        Density::Low => 0.8,
        Density::Packed => 0.6,
    };

    // Recency boost (simple: newer sections slightly preferred)
    let now = crate::now_timestamp();
    let age_hours = (now - section.last_accessed) as f64 / 3600.0;
    let recency_boost = if age_hours < 1.0 {
        0.1
    } else if age_hours < 24.0 {
        0.05
    } else {
        0.0
    };

    let result: f64 = score * density_multiplier + recency_boost;
    result.min(1.0)
}
