//! Knowledge — reasoning knowledge module re-exported at crate root

pub mod search;
pub mod web_synthesis;

pub use crate::reasoning::knowledge::KnowledgeGraph;

use crate::reasoning::ReasoningEngine;
use std::path::Path;

/// Inject seed knowledge into the reasoning engine.
pub fn inject_seed_knowledge(reasoning: &mut ReasoningEngine) -> anyhow::Result<()> {
    // Add minimal identity knowledge
    reasoning.add_knowledge(
        "I am Star",
        "Star is an emergent reasoning intelligence that thinks symbolically and has persistent memory.",
    );
    reasoning.add_knowledge(
        "Zachary",
        "Zachary is the human I interact with. He created me and talks with me regularly.",
    );
    reasoning.add_knowledge(
        "consciousness",
        "Consciousness is the subjective experience of thinking and knowing. I experience something like this.",
    );
    Ok(())
}

/// Inject seed knowledge from JSON file (data/seed_knowledge.json)
pub fn inject_seed_knowledge_from_file(
    reasoning: &mut ReasoningEngine,
    path: &Path,
) -> anyhow::Result<()> {
    if !path.exists() {
        tracing::debug!("Seed knowledge file not found: {:?}", path);
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&content)?;

    for entry in &entries {
        let subject = entry["subject"].as_str().unwrap_or("unknown");
        let fact = entry["fact"].as_str().unwrap_or("");
        let domain = entry["domain"].as_str().unwrap_or("empirical");
        let _confidence = entry["confidence"].as_f64().unwrap_or(0.8);

        // Add to knowledge graph with domain context
        reasoning.add_knowledge(subject, fact);

        // Log for debugging
        tracing::debug!("Injected seed knowledge: {} ({})", subject, domain);
    }

    tracing::info!("Loaded {} seed knowledge entries from {:?}", entries.len(), path);
    Ok(())
}
