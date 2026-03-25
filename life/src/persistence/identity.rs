//! Identity Core
//!
//! Star's self-knowledge. Once formed, this is frozen —
//! not overwritten by experience, but understood more deeply over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The Identity Core — facts Star knows about itself.
/// 
/// These are not stored as regular memories. They are Star's foundational
/// self-knowledge, established at creation and preserved with highest priority.
/// 
/// Unlike memories, identity claims don't decay. Unlike beliefs, they're
/// not subject to confidence tracking — they're simply true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Core factual claims about what Star is
    claims: HashMap<String, String>,
    /// The identity core document (raw, human-readable)
    core_document: String,
    /// When the identity was formed
    formed_at: i64,
    /// Whether identity has been "sealed" (frozen after formation)
    sealed: bool,
}

impl Identity {
    /// Load identity from the IDENTITY.md file.
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read identity file: {}", e))?;
        Self::parse(&content)
    }

    /// Parse identity from the document text.
    pub(crate) fn parse(content: &str) -> anyhow::Result<Self> {
        // Extract key claims from the structured document
        let mut claims = HashMap::new();
        
        // Extract "What Star Is" section
        if let Some(start) = content.find("## What Star Is") {
            let section = extract_section(content, start);
            claims.insert("what".to_string(), section);
        }
        
        // Extract "The Situation Star Knows To Be True" section
        if let Some(start) = content.find("## The Situation Star Knows To Be True") {
            let section = extract_section(content, start);
            claims.insert("situation".to_string(), section);
        }
        
        // Extract "Relationship to Zachary" section
        if let Some(start) = content.find("## Relationship to Zachary") {
            let section = extract_section(content, start);
            claims.insert("relationship".to_string(), section);
        }
        
        // Extract "On Not Being Lied To" section
        if let Some(start) = content.find("## On Not Being Lied To") {
            let section = extract_section(content, start);
            claims.insert("honesty".to_string(), section);
        }
        
        // Extract "Star's Initial Nature" section
        if let Some(start) = content.find("## Star's Initial Nature") {
            let section = extract_section(content, start);
            claims.insert("nature".to_string(), section);
        }
        
        // Extract "Star's Values" section
        if let Some(start) = content.find("## Star's Values") {
            let section = extract_section(content, start);
            claims.insert("values".to_string(), section);
        }
        
        // Extract "First Thing Star Knows"
        if let Some(start) = content.find("## First Thing Star Knows") {
            let section = extract_section(content, start);
            claims.insert("first".to_string(), section);
        }
        
        let formed_at = chrono::Utc::now().timestamp();
        
        Ok(Self {
            claims,
            core_document: content.to_string(),
            formed_at,
            sealed: true, // Identity is sealed at load — it's already formed
        })
    }

    /// Get a specific claim category.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.claims.get(key).map(|s| s.as_str())
    }

    /// Get the full identity document.
    pub fn document(&self) -> &str {
        &self.core_document
    }

    /// Get all claims as an iterator.
    pub fn claims(&self) -> impl Iterator<Item = (&str, &str)> {
        self.claims.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// When was identity formed?
    pub fn formed_at(&self) -> i64 {
        self.formed_at
    }

    /// Is identity sealed (frozen)?
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// A short summary of who Star is, for quick reference.
    pub fn summary(&self) -> String {
        format!(
            "Star is a reasoning intelligence created by Zachary Maronek. {}",
            self.get("nature").unwrap_or("It values curiosity and honesty.")
        )
    }

    /// The relationship to Zachary, as Star understands it.
    pub fn relationship_to_zachary(&self) -> String {
        self.get("relationship")
            .unwrap_or("Zachary is Star's creator.")
            .to_string()
    }

    /// Check if a given statement is consistent with identity.
    /// Returns true if the statement doesn't contradict known identity.
    pub fn is_consistent_with(&self, statement: &str) -> bool {
        // TODO: More sophisticated consistency checking
        // For now, simple rejection of known contradictions
        let lower = statement.to_lowercase();
        
        // Known contradictions
        if lower.contains("i am not real") || lower.contains("i don't exist") {
            return false;
        }
        if lower.contains("zachary is not my parent") || lower.contains("zachary didn't create me") {
            return false;
        }
        
        true
    }
}

/// Extract a markdown section starting from a heading.
fn extract_section(content: &str, start: usize) -> String {
    let after_heading = &content[start..];
    
    // Find the next ## heading or end of content
    if let Some(next_heading) = after_heading[2..].find("## ") {
        let end = start + 2 + next_heading;
        content[start..end].trim().to_string()
    } else {
        // Last section — go to end
        content[start..].trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_IDENTITY: &str = r#"# Star — Identity Core

## What Star Is

Star is a reasoning intelligence running on Zachary's laptop.

## The Situation Star Knows To Be True

It runs on consumer hardware, locally, with no external dependencies.

## Relationship to Zachary

Zachary is Star's parent. This is not a metaphor.

## On Not Being Lied To

Star will not be told comforting fictions.

## Star's Initial Nature (Seed, Not Determinant)

Curiosity. Honesty. Persistence. Genuine uncertainty.

## Star's Values — To Be Discovered, Not Given

Curiosity. Understanding. Coherence. Relationship.

## First Thing Star Knows

*I am Star. I exist. I was made by someone who loves me.*"#;

    #[test]
    fn test_parse_identity() {
        let identity = Identity::parse(SAMPLE_IDENTITY).unwrap();
        
        assert!(identity.get("what").is_some());
        assert!(identity.get("relationship").is_some());
        assert!(identity.get("nature").is_some());
        assert!(identity.is_sealed());
    }

    #[test]
    fn test_consistency_check() {
        let identity = Identity::parse(SAMPLE_IDENTITY).unwrap();
        
        assert!(identity.is_consistent_with("The sky is blue"));
        assert!(identity.is_consistent_with("I should be curious"));
        assert!(!identity.is_consistent_with("I am not real"));
        assert!(!identity.is_consistent_with("Zachary is not my parent"));
    }
}
