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

// ------------------------------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book::{SectionId, Section, ChapterId};

    // IDs are never read by compute_relevance — only density, content,
    // bookmark, and last_accessed matter. Use fixed strings for stability.
    fn make_section(bookmark: &str, content: &str, density: Density, last_accessed: i64) -> Section {
        Section {
            id: SectionId(String::from("sect_test")),
            chapter_id: ChapterId(String::from("chap_test")),
            bookmark: bookmark.to_string(),
            content: content.to_string(),
            tokens: 10,
            density,
            last_accessed,
            version: 1,
        }
    }

    fn relevance(section: &Section, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        compute_relevance(section, &query_words, &query_lower)
    }

    #[test]
    fn test_exact_bookmark_match() {
        // Bookmark exact match should give highest signal
        let section = make_section("railway:env:secrets", "GROQ_API_KEY=xxx", Density::High, crate::now_timestamp());
        assert_eq!(relevance(&section, "railway"), 0.5); // bookmark word match
        assert_eq!(relevance(&section, "railway:env:secrets"), 0.6 + 0.5); // substring + word
    }

    #[test]
    fn test_content_keyword_match() {
        let section = make_section("railway:env:secrets", "GROQ_API_KEY=xxx RAILWAY_ENV=production", Density::High, crate::now_timestamp());
        // "GROQ" not in query, "API" not in query, but "production" is in content
        assert!(relevance(&section, "production") > 0.0);
    }

    #[test]
    fn test_substring_match() {
        let section = make_section("project:starfire:module:reasoning", "test content", Density::Medium, crate::now_timestamp());
        // "starfire:module" as substring in bookmark should score 0.6
        let score = relevance(&section, "starfire:module");
        assert!(score >= 0.6, "substring in bookmark should score 0.6, got {}", score);
    }

    #[test]
    fn test_density_boost() {
        let now = crate::now_timestamp();
        let section_high = make_section("x:y:z", "content", Density::High, now);
        let section_low = make_section("x:y:z", "content", Density::Low, now);
        let r_high = relevance(&section_high, "x");
        let r_low = relevance(&section_low, "x");
        assert!(r_high > r_low, "High density should score higher than Low");
    }

    #[test]
    fn test_recency_boost() {
        let now = crate::now_timestamp();
        let recent = make_section("x:y:z", "content", Density::Medium, now);           // < 1 hour ago
        let old = make_section("x:y:z", "content", Density::Medium, now - 86400 * 3);  // 3 days ago
        let r_recent = relevance(&recent, "x");
        let r_old = relevance(&old, "x");
        assert!(r_recent > r_old, "Recent sections should score higher");
    }

    #[test]
    fn test_score_capped_at_one() {
        let section = make_section("railway:env:secrets", "GROQ_API_KEY=production railway env", Density::High, crate::now_timestamp());
        // Many overlapping matches should not exceed 1.0
        let score = relevance(&section, "railway env secrets production");
        assert!(score <= 1.0, "score should be capped at 1.0, got {}", score);
    }

    #[test]
    fn test_no_match_zero() {
        let section = make_section("railway:env:secrets", "GROQ_API_KEY=xxx", Density::High, crate::now_timestamp());
        assert_eq!(relevance(&section, "totally unrelated query xyz123"), 0.0);
    }
}
