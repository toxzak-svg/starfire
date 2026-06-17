//! Pain tracker — records pain events and computes pain statistics

use std::collections::HashMap;
use super::{PainEvent, PainSource, ReasoningPattern};

/// Pain statistics for a concept
#[derive(Debug, Clone)]
pub struct ConceptPainStats {
    pub concept: String,
    pub total_pain: f64,
    pub event_count: usize,
    pub avg_intensity: f64,
    pub last_pain_at: i64,
    pub source_breakdown: HashMap<PainSource, usize>,
}

/// Pain tracker — tracks pain events and computes statistics
#[derive(Debug, Clone)]
pub struct PainTracker {
    events: Vec<PainEvent>,
    concept_stats: HashMap<String, ConceptPainStats>,
    max_events: usize,
}

impl PainTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            concept_stats: HashMap::new(),
            max_events: 1000,
        }
    }

    /// Record a pain event
    pub fn record(&mut self, event: PainEvent) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event.clone());

        for concept in &event.concepts_involved {
            let stats = self.concept_stats.entry(concept.clone()).or_insert_with(|| {
                ConceptPainStats {
                    concept: concept.clone(),
                    total_pain: 0.0,
                    event_count: 0,
                    avg_intensity: 0.0,
                    last_pain_at: 0,
                    source_breakdown: HashMap::new(),
                }
            });

            stats.total_pain += event.intensity;
            stats.event_count += 1;
            stats.avg_intensity = stats.total_pain / stats.event_count as f64;
            stats.last_pain_at = event.timestamp;

            *stats.source_breakdown.entry(event.source).or_insert(0) += 1;
        }
    }

    /// Record a redundant computation pain
    pub fn record_redundant(&mut self, concept: &str, topic: &str) {
        let event = PainEvent::new(
            PainSource::RedundantComputation,
            ReasoningPattern::RepeatedDerivation,
            vec![concept.to_string()],
            0.5,
            topic.to_string(),
        );
        self.record(event);
    }

    /// Record a contradiction pain
    pub fn record_contradiction(&mut self, concepts: Vec<String>, topic: &str) {
        let event = PainEvent::new(
            PainSource::Contradiction,
            ReasoningPattern::FailedCausalSearch,
            concepts,
            0.8,
            topic.to_string(),
        );
        self.record(event);
    }

    /// Record a wasted effort pain
    pub fn record_wasted(&mut self, concept: &str, topic: &str) {
        let event = PainEvent::new(
            PainSource::WastedEffort,
            ReasoningPattern::ExcessiveChaining,
            vec![concept.to_string()],
            0.3,
            topic.to_string(),
        );
        self.record(event);
    }

    /// Get pain score for a concept (0-1, higher = more painful)
    pub fn pain_score(&self, concept: &str) -> f64 {
        self.concept_stats.get(concept)
            .map(|s| (s.avg_intensity * s.event_count as f64 / 10.0).min(1.0))
            .unwrap_or(0.0)
    }

    /// Get top painful concepts
    pub fn top_painful_concepts(&self, n: usize) -> Vec<&str> {
        let mut concepts: Vec<_> = self.concept_stats.values()
            .collect();
        concepts.sort_by(|a, b| b.total_pain.partial_cmp(&a.total_pain).unwrap());
        concepts.into_iter()
            .take(n)
            .map(|s| s.concept.as_str())
            .collect()
    }

    /// Check if a concept is causing repeated pain (suggests structural issue)
    pub fn is_problematic(&self, concept: &str) -> bool {
        self.concept_stats.get(concept)
            .map(|s| s.event_count >= 3 && s.avg_intensity > 0.5)
            .unwrap_or(false)
    }

    /// Get recent pain events
    pub fn recent_events(&self, n: usize) -> Vec<&PainEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get pain history for a concept
    pub fn concept_history(&self, concept: &str) -> Vec<&PainEvent> {
        self.events.iter()
            .filter(|e| e.concepts_involved.contains(&concept.to_string()))
            .collect()
    }

    /// Suggest structural fixes based on pain patterns
    pub fn suggest_fixes(&self) -> Vec<PainFixSuggestion> {
        let mut suggestions = Vec::new();

        for (concept, stats) in &self.concept_stats {
            if stats.event_count >= 3 && stats.avg_intensity > 0.5 {
                let fix = if let Some(&redundant_count) = stats.source_breakdown.get(&PainSource::RedundantComputation) {
                    if redundant_count > 2 {
                        "Consider caching intermediate results".to_string()
                    } else if let Some(&contradiction_count) = stats.source_breakdown.get(&PainSource::Contradiction) {
                        if contradiction_count > 1 {
                            "Add intermediate abstraction to resolve contradictions".to_string()
                        } else {
                            "Review concept definition or decomposition".to_string()
                        }
                    } else {
                        "Review concept definition or decomposition".to_string()
                    }
                } else {
                    "Review concept definition or decomposition".to_string()
                };

                suggestions.push(PainFixSuggestion {
                    concept: concept.clone(),
                    problem: format!("{} pain events with {:.1} avg intensity",
                        stats.event_count, stats.avg_intensity),
                    suggested_fix: fix,
                });
            }
        }

        suggestions.sort_by(|a, b| b.suggested_fix.len().cmp(&a.suggested_fix.len()));
        suggestions
    }
}

impl Default for PainTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A suggested structural fix based on pain analysis
#[derive(Debug, Clone)]
pub struct PainFixSuggestion {
    pub concept: String,
    pub problem: String,
    pub suggested_fix: String,
}