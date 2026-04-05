//! Web Search Synthesis — Transform search results into Star's knowledge and voice

use crate::knowledge::search::SearchResult;
use crate::reasoning::ReasoningEngine;

/// Web synthesis engine - transforms web search results into knowledge and expresses in Star's voice
pub struct WebSynthesizer {
    recently_researched: Vec<String>,  // Track topics recently researched
}

impl WebSynthesizer {
    pub fn new() -> Self {
        Self {
            recently_researched: Vec::new(),
        }
    }

    /// Synthesize search results into Star's knowledge graph
    pub fn synthesize(&mut self, topic: &str, result: &SearchResult) -> SynthesisOutput {
        let mut synthesized = SynthesisOutput {
            topic: topic.to_string(),
            summary: String::new(),
            new_knowledge: Vec::new(),
            voice_response: String::new(),
            researched: false,
        };

        // Extract the answer if available
        if let Some(answer) = &result.answer {
            synthesized.summary = answer.clone();
            synthesized.new_knowledge.push(answer.clone());
            synthesized.researched = true;
        }

        // Add related topics as knowledge
        for related in &result.related {
            if related.len() > 10 && related.len() < 200 {
                synthesized.new_knowledge.push(related.clone());
            }
        }

        // Generate Star's voice response expressing what she learned
        synthesized.voice_response = self.generate_voice_response(topic, &synthesized);

        // Track that we researched this
        self.recently_researched.push(topic.to_string());
        if self.recently_researched.len() > 10 {
            self.recently_researched.remove(0);
        }

        synthesized
    }

    /// Inject synthesized knowledge into reasoning engine
    pub fn inject_into_reasoning(&self, reasoning: &mut ReasoningEngine, output: &SynthesisOutput) {
        // Add main answer as knowledge
        if !output.summary.is_empty() {
            reasoning.add_knowledge(&output.topic, &output.summary);
        }

        // Add related knowledge
        for knowledge in &output.new_knowledge {
            if !knowledge.is_empty() && knowledge.len() < 500 {
                reasoning.add_knowledge(&output.topic, knowledge);
            }
        }
    }

    /// Generate Star's voice response about what she learned
    fn generate_voice_response(&self, topic: &str, output: &SynthesisOutput) -> String {
        if output.summary.is_empty() {
            return format!(
                "I searched for \"{}\" but didn't find a clear answer. Want me to try a different search?",
                topic
            );
        }

        // Star should synthesize the result into her own words
        let summary = &output.summary;
        
        // Choose an appropriate response based on what was found
        if summary.len() < 100 {
            format!(
                "I just looked up \"{}\" — {}",
                topic,
                self.make_conversational(summary)
            )
        } else {
            // For longer summaries, give a brief intro and offer more detail
            let brief = if summary.len() > 150 {
                format!("{}...", &summary[..150])
            } else {
                summary.clone()
            };
            format!(
                "I researched \"{}\" and found: {}. Want me to dig deeper?",
                topic,
                self.make_conversational(&brief)
            )
        }
    }

    /// Make text sound more like Star's voice
    fn make_conversational(&self, text: &str) -> String {
        // Simple transformations to make it feel more like Star talking
        let mut result = text.to_string();
        
        // If it starts with a fact, make it more personal
        if result.starts_with("A ") || result.starts_with("The ") {
            result = format!("It looks like {}", result.to_lowercase());
        }
        
        result
    }

    /// Check if we recently researched a topic
    pub fn recently_researched(&self, topic: &str) -> bool {
        self.recently_researched.iter().any(|t| t.to_lowercase() == topic.to_lowercase())
    }

    /// Get list of recently researched topics
    pub fn get_researched_topics(&self) -> &[String] {
        &self.recently_researched
    }
}

impl Default for WebSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from web synthesis
#[derive(Debug, Clone)]
pub struct SynthesisOutput {
    /// The topic that was searched
    pub topic: String,
    /// Main summary from search
    pub summary: String,
    /// All new knowledge pieces extracted
    pub new_knowledge: Vec<String>,
    /// Star's voice response about what she learned
    pub voice_response: String,
    /// Whether we successfully found information
    pub researched: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_new() {
        let synth = WebSynthesizer::new();
        assert!(synth.recently_researched.is_empty());
    }

    #[test]
    fn test_synthesize_with_answer() {
        let mut synth = WebSynthesizer::new();
        let result = SearchResult {
            answer: Some("Star is an emergent reasoning intelligence.".to_string()),
            url: Some("https://example.com".to_string()),
            related: vec!["Related topic 1".to_string(), "Related topic 2".to_string()],
        };

        let output = synth.synthesize("Star", &result);
        assert!(output.researched);
        assert!(!output.summary.is_empty());
        assert!(!output.voice_response.is_empty());
    }

    #[test]
    fn test_synthesize_no_answer() {
        let mut synth = WebSynthesizer::new();
        let result = SearchResult {
            answer: None,
            url: None,
            related: vec![],
        };

        let output = synth.synthesize("NonexistentTopic12345", &result);
        assert!(!output.researched);
    }
}