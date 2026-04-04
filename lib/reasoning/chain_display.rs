//! Chain Display — Natural Language Reasoning Chain Generation
//!
//! Converts ReasoningChain structures into natural, fluent text
//! that shows Starfire's thinking without being robotic.

use super::chain::{ReasoningChain, VisibleReasoningStep};

/// Natural language generator for reasoning chains.
pub struct ChainDisplay {
    /// Whether to use formal or casual language
    pub formality: Formality,
    /// Whether to show all steps or just key ones
    pub verbosity: Verbosity,
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum Formality {
    Formal,
    #[default]
    Casual,
    Warm,
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum Verbosity {
    Minimal,      // Just the answer
    #[default]
    Standard,    // Key steps + answer
    Full,        // All steps + assumptions + confidence
}

impl Default for ChainDisplay {
    fn default() -> Self {
        Self {
            formality: Formality::Casual,
            verbosity: Verbosity::Standard,
        }
    }
}

impl ChainDisplay {
    /// Create a new chain display with given settings.
    pub fn new(formality: Formality, verbosity: Verbosity) -> Self {
        Self { formality, verbosity }
    }

    /// Display a reasoning chain.
    pub fn display(&self, chain: &ReasoningChain) -> String {
        match self.verbosity {
            Verbosity::Minimal => self.display_minimal(chain),
            Verbosity::Standard => self.display_standard(chain),
            Verbosity::Full => self.display_full(chain),
        }
    }

    /// Minimal display: just the answer with a brief reasoning note.
    fn display_minimal(&self, chain: &ReasoningChain) -> String {
        if chain.steps.is_empty() {
            return chain.answer.clone();
        }

        let opening = match self.formality {
            Formality::Warm => "Here's what I think:",
            Formality::Casual => "My take:",
            Formality::Formal => "Conclusion:",
        };

        let reasoning_note = if chain.has_uncertainty() {
            if chain.assumptions_count > 0 {
                format!(" I'm working through {} step(s) but made {} assumption(s).", chain.steps.len(), chain.assumptions_count)
            } else {
                format!(" I worked through {} step(s).", chain.steps.len())
            }
        } else {
            format!(" I'm certain after {} step(s).", chain.steps.len())
        };

        format!("{} {}{}", opening, chain.answer, reasoning_note)
    }

    /// Standard display: key steps shown.
    fn display_standard(&self, chain: &ReasoningChain) -> String {
        if chain.steps.is_empty() {
            return chain.answer.clone();
        }

        let mut lines = Vec::new();

        // Opening
        let opening = match self.formality {
            Formality::Warm => "Here's how I'm thinking about this:",
            Formality::Casual => "Let me work through this:",
            Formality::Formal => "My reasoning proceeds as follows:",
        };
        lines.push(opening.to_string());
        lines.push(String::new());

        // Show every 2nd step for brevity
        let steps_to_show: Vec<_> = chain.steps
            .iter()
            .enumerate()
            .filter(|(i, _)| *i % 2 == 0 || *i == chain.steps.len() - 1)
            .map(|(_, s)| s)
            .collect();

        for step in steps_to_show {
            lines.push(self.format_step_compact(step));
        }

        lines.push(String::new());

        // Conclusion with confidence
        let conclusion_prefix = match self.formality {
            Formality::Warm => "So I'm thinking: ",
            Formality::Casual => "My answer: ",
            Formality::Formal => "Therefore: ",
        };
        lines.push(format!("{}{}", conclusion_prefix, chain.answer));

        // Confidence note
        let confidence_note = self.confidence_note(chain);
        if !confidence_note.is_empty() {
            lines.push(confidence_note);
        }

        lines.join("\n")
    }

    /// Full display: all steps, all assumptions, all details.
    fn display_full(&self, chain: &ReasoningChain) -> String {
        if chain.steps.is_empty() {
            return chain.answer.clone();
        }

        let mut lines = Vec::new();

        // Opening
        lines.push(self.opening_for(chain));
        lines.push(String::new());

        // Each step
        for step in &chain.steps {
            lines.push(self.format_step_full(step));
            lines.push(String::new());
        }

        // Conclusion
        lines.push(self.format_conclusion(chain));

        // Confidence and assumption summary
        lines.push(String::new());
        lines.push(self.confidence_note(chain));
        
        if chain.assumptions_count > 0 {
            lines.push(self.assumptions_note(chain));
        }

        lines.join("\n")
    }

    /// Format a single step compactly.
    fn format_step_compact(&self, step: &VisibleReasoningStep) -> String {
        let connector = match self.formality {
            Formality::Warm => "—",
            Formality::Casual => "→",
            Formality::Formal => "∴",
        };
        
        let rule_tag = if step.is_certain {
            String::new()
        } else {
            format!(" [{}]", step.inference)
        };
        
        format!("  {} {}{}", connector, step.conclusion, rule_tag)
    }

    /// Format a single step with full detail.
    fn format_step_full(&self, step: &VisibleReasoningStep) -> String {
        let mut lines = vec![
            format!("Step {}:", step.step_number),
        ];

        // What we're reasoning from
        lines.push(format!("    Starting point: \"{}\"", step.premise));
        
        // The rule
        let rule_text = match self.formality {
            Formality::Warm => format!("I'm using: {}", step.inference),
            Formality::Casual => format!("Applying: {}", step.inference),
            Formality::Formal => format!("Inference rule: {}", step.inference),
        };
        lines.push(format!("    {}", rule_text));
        
        // Conclusion
        lines.push(format!("    Therefore: \"{}\"", step.conclusion));
        
        // Confidence and assumptions
        if !step.is_certain {
            lines.push(format!("    Confidence: {:.0}%", step.confidence_at_step * 100.0));
        }
        
        if !step.assumptions.is_empty() {
            let assumptions_str = step.assumptions
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("    Assuming: {}", assumptions_str));
        }

        lines.join("\n")
    }

    /// Generate an appropriate opening based on chain characteristics.
    fn opening_for(&self, chain: &ReasoningChain) -> String {
        let first = |opts: &[&str]| opts[0].to_string();
        
        match (chain.steps.len(), chain.assumptions_count, chain.has_uncertainty()) {
            (1, 0, false) => first(&[
                "This is straightforward.",
                "There's a clear answer here.",
            ]),
            (n, 0, false) if n > 1 => first(&[
                &format!("I'm reasoning through {} steps, all certain.", n),
                &format!("{} steps of certain reasoning lead to this.", n),
            ]),
            (n, a, false) if a > 0 => first(&[
                &format!("Let me show my reasoning — {} step(s), {} assumption(s).", n, a),
                &format!("Working through {} step(s), but I'm assuming: {}", n, a),
            ]),
            (_, 0, true) => first(&[
                "Let me work through this step by step.",
                "Here's my reasoning, though I'm not entirely certain.",
                "I want to be transparent about my thinking here.",
            ]),
            _ => first(&[
                "Let me think through this out loud.",
                "Here's how I'm approaching this question.",
                "This is interesting — let me show my work.",
            ]),
        }
    }

    /// Format the final conclusion.
    fn format_conclusion(&self, chain: &ReasoningChain) -> String {
        let prefix = match self.formality {
            Formality::Warm => "So what I believe is: ",
            Formality::Casual => "My answer: ",
            Formality::Formal => "Therefore: ",
        };
        
        format!("{}{}", prefix, chain.answer)
    }

    /// Generate a confidence note.
    fn confidence_note(&self, chain: &ReasoningChain) -> String {
        let score = chain.final_confidence_score;
        
        let statement = match self.formality {
            Formality::Warm => match score {
                s if s > 0.9 => "I'm quite certain about this.",
                s if s > 0.7 => "I think this is right.",
                s if s > 0.5 => "I'm moderately confident.",
                _ => "I'm not very certain about this.",
            },
            Formality::Casual => match score {
                s if s > 0.9 => "Pretty sure on this one.",
                s if s > 0.7 => "Feels right to me.",
                s if s > 0.5 => "Could go either way.",
                _ => "This is more of a guess.",
            },
            Formality::Formal => match score {
                s if s > 0.9 => "High confidence in this conclusion.",
                s if s > 0.7 => "Moderate confidence.",
                s if s > 0.5 => "Provisional conclusion.",
                _ => "Low confidence — further verification recommended.",
            },
        };
        
        statement.to_string()
    }

    /// Generate an assumptions note.
    fn assumptions_note(&self, chain: &ReasoningChain) -> String {
        let count = chain.assumptions_count;
        
        match self.formality {
            Formality::Warm => format!("I should note I made {} assumption(s) along the way — those are the spots where I'm less certain.", count),
            Formality::Casual => format!("FYI, I made {} assumption(s) — those are the less solid parts.", count),
            Formality::Formal => format!("Note: {} assumption(s) were introduced during reasoning.", count),
        }
    }
}


