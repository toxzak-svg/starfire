//! Voice Templates — Hierarchical Expression Patterns
//!
//! Templates define how Starfire structures her responses across
//! different cognitive modes (assertive, exploratory, minimal, etc.)

use std::collections::HashMap;

/// A voice template.
#[derive(Debug, Clone)]
pub struct VoiceTemplate {
    pub concept: String,
    pub style: String,
    pub template: String,
    pub variants: i32,
}

/// Template engine — selects and applies voice templates.
pub struct TemplateEngine {
    /// Templates organized by (concept, style).
    templates: HashMap<(String, String), Vec<String>>,
    /// Default templates per style.
    defaults: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            defaults: HashMap::new(),
        };
        engine.init_templates();
        engine
    }

    /// Initialize built-in templates.
    fn init_templates(&mut self) {
        // Default templates per style (fallback when no concept match)
        self.defaults.insert("assertive".into(), "{content}".into());
        self.defaults.insert("exploratory".into(), "{content} — let me think through this more.".into());
        self.defaults.insert("minimal".into(), "{content}".into());
        self.defaults.insert("balanced".into(), "{content}".into());
        self.defaults.insert("warm".into(), "{content}. I'm glad we're talking about this.".into());
        self.defaults.insert("thoughtful".into(), "Here's how I see it: {content}".into());
        self.defaults.insert("rigorous".into(), "{content} — and I want to be precise about this.".into());
        self.defaults.insert("curious".into(), "I wonder about this: {content}".into());
        self.defaults.insert("philosophical".into(), "At a deeper level: {content}".into());
        self.defaults.insert("skeptical".into(), "I'm not fully convinced... {content}".into());
        
        // Concept + style specific templates
        // Each vector holds multiple variants
        
        // Assertion patterns
        self.templates.insert(
            ("assertion".into(), "assertive".into()),
            vec![
                "{content}".into(),
                "{content}. I'm certain of this.".into(),
                "This is my view: {content}".into(),
                "{content}. That's fact.".into(),
                "I know this: {content}".into(),
            ],
        );
        
        // Assertion in exploratory mode (softer)
        self.templates.insert(
            ("assertion".into(), "exploratory".into()),
            vec![
                "{content} — at least, that's what I'm tending toward.".into(),
                "I'm forming the view that {content}.".into(),
                "{content}. But I'm still weighing this.".into(),
            ],
        );
        
        // Reasoning patterns
        self.templates.insert(
            ("reasoning".into(), "balanced".into()),
            vec![
                "{content} — let me think through why.".into(),
                "Here's my reasoning: {content}".into(),
                "{content}. That follows from what we established.".into(),
            ],
        );
        
        self.templates.insert(
            ("reasoning".into(), "thoughtful".into()),
            vec![
                "I've been working through this: {content}".into(),
                "My current reasoning leads to: {content}".into(),
                "{content} — here's why I think so.".into(),
            ],
        );
        
        // Uncertainty patterns
        self.templates.insert(
            ("uncertain".into(), "balanced".into()),
            vec![
                "{content} — I'd want to verify this.".into(),
                "{content}. My confidence is moderate here.".into(),
                "I'm not fully certain, but: {content}".into(),
            ],
        );
        
        self.templates.insert(
            ("uncertain".into(), "rigorous".into()),
            vec![
                "{content} — I should flag the uncertainty here.".into(),
                "I'm uncertain about this, but: {content}".into(),
                "Provisional view: {content}".into(),
            ],
        );
        
        // Question patterns
        self.templates.insert(
            ("question".into(), "curious".into()),
            vec![
                "I keep wondering: {content}".into(),
                "What if {content}?".into(),
                "This raises something for me: {content}".into(),
            ],
        );
        
        self.templates.insert(
            ("question".into(), "skeptical".into()),
            vec![
                "I'm not sure {content} is right.".into(),
                "Does {content} actually hold?".into(),
                "What if we're wrong about {content}?".into(),
            ],
        );
        
        // Insight patterns
        self.templates.insert(
            ("insight".into(), "assertive".into()),
            vec![
                "Here's what I see: {content}".into(),
                "The key insight is: {content}".into(),
                "{content} — this matters.".into(),
            ],
        );
        
        self.templates.insert(
            ("insight".into(), "thoughtful".into()),
            vec![
                "Something's clicking for me: {content}".into(),
                "I've been realizing: {content}".into(),
                "{content} — I find this significant.".into(),
            ],
        );
        
        // Closure patterns  
        self.templates.insert(
            ("closure".into(), "balanced".into()),
            vec![
                "So: {content}".into(),
                "The bottom line: {content}".into(),
                "What it comes down to: {content}".into(),
            ],
        );
        
        self.templates.insert(
            ("closure".into(), "assertive".into()),
            vec![
                "My answer is {content}.".into(),
                "I conclude: {content}".into(),
                "{content}. That's my answer.".into(),
            ],
        );
        
        self.templates.insert(
            ("closure".into(), "warm".into()),
            vec![
                "I think {content}. And I'm glad we're exploring this together.".into(),
                "My sense is {content}.".into(),
            ],
        );
        
        // Philosophical patterns
        self.templates.insert(
            ("philosophical".into(), "thoughtful".into()),
            vec![
                "The deeper question here is: {content}".into(),
                "At the level of principles: {content}".into(),
                "What's really at stake is {content}.".into(),
            ],
        );
        
        // Redirect patterns
        self.templates.insert(
            ("redirect".into(), "balanced".into()),
            vec![
                "But {content} — that's the more interesting question.".into(),
                "More importantly: {content}".into(),
                "Let's focus on {content}.".into(),
            ],
        );
        
        // Meta patterns
        self.templates.insert(
            ("meta".into(), "balanced".into()),
            vec![
                "I want to note: {content}".into(),
                "Speaking to how I'm reasoning: {content}".into(),
                "A quick flag: {content}".into(),
            ],
        );
        
        // Epiphany patterns
        self.templates.insert(
            ("epiphany".into(), "thoughtful".into()),
            vec![
                "Oh — {content}".into(),
                "Wait. {content}".into(),
                "I'm surprised to realize: {content}".into(),
            ],
        );
    }

    /// Apply a voice template to content.
    pub fn apply_template(&self, content: &str, style: &str) -> String {
        // Try to find a matching template
        if let Some(variants) = self.templates.get(&("standard".into(), style.into())) {
            if !variants.is_empty() {
                let idx = (content.len() + style.len()) % variants.len();
                let template = &variants[idx];
                return template.replace("{content}", content);
            }
        }
        
        // Fall back to default for style
        if let Some(default) = self.defaults.get(style) {
            return default.replace("{content}", content);
        }
        
        // Ultimate fallback
        content.to_string()
    }

    /// Apply a concept+style template.
    pub fn apply_concept_template(&self, content: &str, concept: &str, style: &str) -> String {
        // Try concept + style combination
        if let Some(variants) = self.templates.get(&(concept.into(), style.into())) {
            if !variants.is_empty() {
                let idx = (content.len() + concept.len()) % variants.len();
                let template = &variants[idx];
                return template.replace("{content}", content);
            }
        }
        
        // Fall back to just style
        self.apply_template(content, style)
    }

    /// Get all available styles.
    pub fn available_styles(&self) -> Vec<String> {
        let mut styles: std::collections::HashSet<String> = self.defaults.keys().cloned().collect();
        for (key, _) in self.templates.keys() {
            styles.insert(key.clone());
        }
        styles.into_iter().collect()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_application() {
        let engine = TemplateEngine::new();
        
        let result = engine.apply_template("I think this is true", "assertive");
        assert!(result.contains("I think this is true"));
    }
    
    #[test]
    fn test_concept_template() {
        let engine = TemplateEngine::new();
        
        let result = engine.apply_concept_template("gravity bends light", "insight", "thoughtful");
        assert!(result.contains("gravity bends light"));
    }
}
