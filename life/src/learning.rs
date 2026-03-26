//! Learning Engine — Genuine Concept Formation and Learning
//!
//! Star learns from EXPERIENCES, not keywords. This module implements:
//! 1. Experience recording - store encounters with concepts
//! 2. Pattern detection - find regularities across experiences
//! 3. Concept formation - build abstractions from patterns
//! 4. Hypothesis generation - form testable propositions
//! 5. Revision - update when contradicted
//!
//! The key difference from keyword matching:
//! - Keyword: if input.contains("hun") → respond warmly
//! - Learning: encounter "hun" 3+ times → form concept "hun = endearment" → respond based on UNDERSTANDING

use crate::persistence::Store;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// A single experience - one encounter with a concept
#[derive(Debug, Clone)]
pub struct Experience {
    /// What was encountered
    pub stimulus: String,
    /// The context it appeared in
    pub context: String,
    /// What Star said/did in response
    pub response: Option<String>,
    /// Whether this was positive or negative feedback
    pub valence: f64,  // -1 to 1
    pub timestamp: i64,
}

/// A formed concept - abstraction built from experiences
#[derive(Debug, Clone)]
pub struct Concept {
    /// The concept name/label
    pub name: String,
    /// The category of this concept
    pub category: ConceptCategory,
    /// All experiences that formed this concept
    pub experiences: Vec<Experience>,
    /// The learned property/meaning
    pub property: String,
    /// How confident Star is in this concept
    pub confidence: f64,
    /// When this concept was formed
    pub formed_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConceptCategory {
    TermOfEndearment,
    Person,
    Capability,
    Fact,
    Skill,
    Preference,
    Unknown,
}

impl ConceptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConceptCategory::TermOfEndearment => "endearment",
            ConceptCategory::Person => "person",
            ConceptCategory::Capability => "capability",
            ConceptCategory::Fact => "fact",
            ConceptCategory::Skill => "skill",
            ConceptCategory::Preference => "preference",
            ConceptCategory::Unknown => "unknown",
        }
    }
}

/// The learning engine - manages concept formation
pub struct LearningEngine {
    /// Currently known concepts
    concepts: HashMap<String, Concept>,
    /// Recent experiences waiting to form concepts
    experience_buffer: HashMap<String, Vec<Experience>>,
    /// Minimum experiences before forming a concept
    concept_threshold: usize,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            experience_buffer: HashMap::new(),
            concept_threshold: 3,  // Need 3 similar experiences to form a concept
        }
    }

    /// Record an experience with a stimulus
    pub fn experience(&mut self, stimulus: &str, context: &str, response: Option<&str>, valence: f64) {
        let stimulus_lower = stimulus.to_lowercase();
        
        let exp = Experience {
            stimulus: stimulus_lower.clone(),
            context: context.to_lowercase(),
            response: response.map(|s| s.to_string()),
            valence,
            timestamp: Utc::now().timestamp(),
        };
        
        // Add to buffer
        let buffer = self.experience_buffer.entry(stimulus_lower.clone()).or_insert_with(Vec::new);
        buffer.push(exp);
        
        // Check if we have enough to form a concept
        if let Some(buffered) = self.experience_buffer.get(&stimulus_lower) {
            if buffered.len() >= self.concept_threshold {
                self.form_concept(&stimulus_lower);
            }
        }
    }
    
    /// Instant teaching — form a concept immediately without waiting for experiences
    /// This is for fast teaching via /learn command
    pub fn teach_instant(&mut self, term: &str, definition: &str, confidence: f64) {
        let term_lower = term.to_lowercase();
        
        // Determine category - check specific terms FIRST
        let def_lower = definition.to_lowercase();
        
        // Check for terms of endearment first (before "person" check)
        let is_endearment = term_lower == "love" || term_lower == "hun" || term_lower == "dear" ||
                           def_lower.contains("affection") || def_lower.contains("term of endearment") ||
                           def_lower.contains("opposite of hate");
        
        let category = if is_endearment {
            ConceptCategory::TermOfEndearment
        } else if def_lower.contains("nickname") || def_lower.contains("short for") {
            ConceptCategory::Person
        } else if def_lower.contains("person") || def_lower.contains("named") {
            ConceptCategory::Person
        } else if def_lower.contains("can ") || def_lower.contains("able to") {
            ConceptCategory::Capability
        } else {
            ConceptCategory::Unknown
        };
        
        let exp = Experience {
            stimulus: term_lower.clone(),
            context: format!("DEFINED: {}", definition),
            response: None,
            valence: confidence,
            timestamp: Utc::now().timestamp(),
        };
        
        let concept = Concept {
            name: term.to_string(),
            category,
            experiences: vec![exp],
            property: definition.to_string(),
            confidence,
            formed_at: Utc::now().timestamp(),
        };
        
        self.concepts.insert(term_lower, concept);
    }
    
    /// Form a concept from accumulated experiences
    fn form_concept(&mut self, stimulus: &str) {
        // Don't form if already have this concept
        if self.concepts.contains_key(stimulus) {
            return;
        }
        
        let experiences = match self.experience_buffer.get(stimulus) {
            Some(e) => e.clone(),
            None => return,
        };
        
        if experiences.len() < self.concept_threshold {
            return;
        }
        
        // Analyze the experiences to determine category and property
        let (category, property) = self.analyze_experiences(&experiences);
        
        // Calculate confidence based on consistency
        let avg_valence: f64 = experiences.iter().map(|e| e.valence).sum::<f64>() / experiences.len() as f64;
        let valence_consistency = 1.0 - experiences.iter()
            .map(|e| (e.valence - avg_valence).abs())
            .sum::<f64>() / experiences.len() as f64;
        
        let confidence = (valence_consistency + 0.5).min(1.0);
        
        let concept = Concept {
            name: stimulus.to_string(),
            category,
            experiences: experiences.clone(),
            property,
            confidence,
            formed_at: Utc::now().timestamp(),
        };
        
        self.concepts.insert(stimulus.to_string(), concept);
        
        // Clear buffer for this stimulus
        self.experience_buffer.remove(stimulus);
    }
    
    /// Analyze experiences to determine what concept to form
    fn analyze_experiences(&self, experiences: &[Experience]) -> (ConceptCategory, String) {
        let stimulus = &experiences.first().map(|e| e.stimulus.clone()).unwrap_or_default();
        
        // Check for term of endearment patterns
        let all_valences_positive = experiences.iter().all(|e| e.valence > 0.0);
        let appears_in_affectionate_context = experiences.iter()
            .any(|e| e.context.contains("love") || e.context.contains("care") || e.context.contains("dear"));
        
        if all_valences_positive && (appears_in_affectionate_context || stimulus.len() < 10) {
            return (
                ConceptCategory::TermOfEndearment,
                format!("{} is a positive term - likely affectionate", stimulus)
            );
        }
        
        // Check for person names (capitalized in original)
        if stimulus.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && stimulus.len() < 20 {
            return (
                ConceptCategory::Person,
                format!("{} is a person's name", stimulus)
            );
        }
        
        // Default to fact/concept
        (
            ConceptCategory::Unknown,
            format!("Concept formed from {} experiences", experiences.len())
        )
    }
    
    /// Query: what do I know about this stimulus?
    pub fn query(&self, stimulus: &str) -> Option<&Concept> {
        self.concepts.get(&stimulus.to_lowercase())
    }
    
    /// Check if Star knows something about this stimulus
    pub fn knows(&self, stimulus: &str) -> bool {
        self.concepts.contains_key(&stimulus.to_lowercase())
    }
    
    /// Get understanding of a term (why Star responds the way it does)
    pub fn get_understanding(&self, stimulus: &str) -> Option<String> {
        let concept = self.query(stimulus)?;
        
        let base = match concept.category {
            ConceptCategory::TermOfEndearment => {
                // Use property if it contains actual definition info
                if concept.property.contains("defined:") || concept.property.len() > 20 {
                    format!("I understand '{}': {}", concept.name, concept.property.replace("DEFINED: ", ""))
                } else {
                    format!("I know '{}' is affectionate - Zachary uses it positively", concept.name)
                }
            }
            ConceptCategory::Person => {
                format!("I know '{}' is a person", concept.name)
            }
            ConceptCategory::Capability => {
                format!("I know I can {}", concept.property)
            }
            _ => {
                format!("I understand '{}': {}", concept.name, concept.property)
            }
        };
        
        let result = if concept.confidence < 0.7 {
            format!("{} (but I'm still learning this)", base)
        } else {
            base
        };
        
        Some(result)
    }
    
    /// Learn from correction - when Star is told something is wrong
    pub fn correct(&mut self, stimulus: &str, correction: &str) {
        // If we have a concept, update or remove it
        let key = stimulus.to_lowercase();
        
        if let Some(concept) = self.concepts.get_mut(&key) {
            // Add correction as a new experience with negative valence
            let exp = Experience {
                stimulus: key.clone(),
                context: format!("CORRECTED: {}", correction),
                response: None,
                valence: -1.0,
                timestamp: Utc::now().timestamp(),
            };
            concept.experiences.push(exp);
            concept.confidence = (concept.confidence - 0.2).max(0.1);
        }
    }
    
    /// Learn from reinforcement - when Star gets positive feedback
    pub fn reinforce(&mut self, stimulus: &str, context: &str) {
        let key = stimulus.to_lowercase();
        
        if let Some(concept) = self.concepts.get_mut(&key) {
            let exp = Experience {
                stimulus: key.clone(),
                context: context.to_string(),
                response: None,
                valence: 1.0,
                timestamp: Utc::now().timestamp(),
            };
            concept.experiences.push(exp);
            concept.confidence = (concept.confidence + 0.1).min(1.0);
        }
    }
    
    /// Get all concepts in a category
    pub fn concepts_by_category(&self, category: ConceptCategory) -> Vec<&Concept> {
        self.concepts.values()
            .filter(|c| c.category == category)
            .collect()
    }
    
    /// Form a generalization from multiple concepts
    pub fn generalize(&self, category: &ConceptCategory) -> Option<String> {
        let concepts = self.concepts_by_category(category.clone());
        
        if concepts.len() < 2 {
            return None;
        }
        
        match category {
            ConceptCategory::TermOfEndearment => {
                Some(format!("Zachary uses affectionate terms: {}", 
                    concepts.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ")))
            }
            _ => None,
        }
    }
    
    /// Get count of known concepts
    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }
    
    /// Get summary of what Star has learned
    pub fn summary(&self) -> String {
        if self.concepts.is_empty() {
            return "I'm still learning. I haven't formed any concepts yet.".to_string();
        }
        
        let mut lines = vec![format!("I've formed {} concepts:", self.concepts.len())];
        
        for category in [
            ConceptCategory::TermOfEndearment,
            ConceptCategory::Person,
            ConceptCategory::Capability,
            ConceptCategory::Preference,
        ] {
            let concepts = self.concepts_by_category(category.clone());
            if !concepts.is_empty() {
                let names: Vec<_> = concepts.iter().map(|c| c.name.as_str()).collect();
                lines.push(format!("  {}: {}", category.as_str(), names.join(", ")));
            }
        }
        
        let unknown = self.concepts_by_category(ConceptCategory::Unknown);
        if !unknown.is_empty() {
            lines.push(format!("  other: {} concepts", unknown.len()));
        }
        
        lines.join("\n")
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}
