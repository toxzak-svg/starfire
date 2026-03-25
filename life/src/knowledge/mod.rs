//! Knowledge Layer — R&D-F: World Knowledge
//!
//! Gives Star the ability to read, understand, and store world knowledge.
//! 
//! Components:
//! - **Reader**: reads files, URLs, documents
//! - **FactExtractor**: parses text into structured facts
//! - **SeedKnowledge**: essential facts Star is born knowing
//! - **WebSearch**: optional lightweight web search for verification
//!
//! Facts are stored in the knowledge graph and surfaced via reasoning.

pub mod reader;
pub mod facts;
pub mod seed;
pub mod search;

pub use reader::Reader;
pub use facts::FactExtractor;
pub use seed::inject_seed_knowledge;
pub use search::WebSearcher;

/// A structured fact extracted from text.
#[derive(Debug, Clone)]
pub struct Fact {
    /// The subject (e.g., "fire")
    pub subject: String,
    /// The predicate/relation (e.g., "is", "burns", "requires")
    pub predicate: String,
    /// The object (e.g., "hot", "at 1000C", "oxygen")
    pub object: Option<String>,
    /// Confidence in this fact (0-1)
    pub confidence: f64,
    /// Source of this fact
    pub source: String,
}

impl Fact {
    pub fn to_string(&self) -> String {
        if let Some(ref obj) = self.object {
            format!("{} {} {}", self.subject, self.predicate, obj)
        } else {
            format!("{} {}", self.subject, self.predicate)
        }
    }
}

/// A claim that needs verification.
#[derive(Debug, Clone)]
pub struct UnverifiedClaim {
    pub fact: Fact,
    pub verification_status: VerificationStatus,
    pub asked_user: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
    Unverifiable,
}
