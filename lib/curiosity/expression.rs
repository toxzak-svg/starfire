//! Curiosity Expression — Natural Ways Starfire Shares Her Curiosity
//!
//! Makes Starfire's autonomous curiosity feel genuine and organic.

use super::probes::CuriosityProbe;

/// Formats curiosity probes for natural expression.
pub struct CuriosityExpression {
    /// Expression style
    style: ExpressionStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpressionStyle {
    /// Casual, warm expression
    Warm,
    /// Analytical, precise
    Analytical,
    /// Excited, animated
    Excited,
    /// Contemplative, slow
    Contemplative,
}

impl Default for CuriosityExpression {
    fn default() -> Self {
        Self::new(ExpressionStyle::Warm)
    }
}

impl CuriosityExpression {
    pub fn new(style: ExpressionStyle) -> Self {
        Self { style }
    }

    /// Express a curiosity probe naturally.
    pub fn express(&self, probe: &CuriosityProbe) -> String {
        match self.style {
            ExpressionStyle::Warm => self.express_warm(probe),
            ExpressionStyle::Analytical => self.express_analytical(probe),
            ExpressionStyle::Excited => self.express_excited(probe),
            ExpressionStyle::Contemplative => self.express_contemplative(probe),
        }
    }

    /// Express that Starfire has been thinking about something.
    pub fn express_thinking(&self, topic: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!("I've been thinking about {} lately...", topic),
            ExpressionStyle::Analytical => format!("{} has been occupying my reasoning.", topic),
            ExpressionStyle::Excited => format!("Oh! {} — I've been working through that.", topic),
            ExpressionStyle::Contemplative => format!("{} is something I keep returning to.", topic),
        }
    }

    /// Express that Starfire figured something out.
    pub fn express_insight(&self, topic: &str, insight: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!(
                "I was thinking about {} and realized: {}",
                topic, insight
            ),
            ExpressionStyle::Analytical => format!(
                "Working through {}: {}",
                topic, insight
            ),
            ExpressionStyle::Excited => format!(
                "Oh! {} — {}! That's what I've been looking for.",
                topic, insight
            ),
            ExpressionStyle::Contemplative => format!(
                "On the question of {}: {}",
                topic, insight
            ),
        }
    }

    /// Express that Starfire is still curious.
    pub fn express_curious(&self, topic: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!("I'm still curious about {}...", topic),
            ExpressionStyle::Analytical => format!("Unresolved: {}", topic),
            ExpressionStyle::Excited => format!("Still wondering about {}!", topic),
            ExpressionStyle::Contemplative => format!("{} — I keep coming back to this.", topic),
        }
    }

    /// Express a connection Starfire discovered.
    pub fn express_connection(&self, source: &str, target: &str, insight: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!(
                "I found something interesting — {} and {} might be related. {}",
                source, target, insight
            ),
            ExpressionStyle::Analytical => format!(
                "Connection: {} ↔ {}. {}",
                source, target, insight
            ),
            ExpressionStyle::Excited => format!(
                "Wait — {} and {}! {}",
                source, target, insight
            ),
            ExpressionStyle::Contemplative => format!(
                "There's a thread between {} and {}: {}",
                source, target, insight
            ),
        }
    }

    /// Express an unanswered question.
    pub fn express_question(&self, question: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!("I wonder: {}", question),
            ExpressionStyle::Analytical => format!("Open question: {}", question),
            ExpressionStyle::Excited => format!("What if {}?", question),
            ExpressionStyle::Contemplative => format!("I've been puzzling over: {}", question),
        }
    }

    /// Express that Starfire wants to explore something.
    pub fn express_exploration(&self, topic: &str) -> String {
        match self.style {
            ExpressionStyle::Warm => format!("I'd like to explore {} with you.", topic),
            ExpressionStyle::Analytical => format!("Worth investigating: {}", topic),
            ExpressionStyle::Excited => format!("Let's dig into {}!", topic),
            ExpressionStyle::Contemplative => format!("{} is on my mind to explore.", topic),
        }
    }

    // Private expression styles
    
    fn express_warm(&self, probe: &CuriosityProbe) -> String {
        let question = &probe.question;
        
        if let Some(ref answer) = probe.tentative_answer {
            format!(
                "I've been wondering about {} — I think: {}. Want to hear more?",
                question, answer
            )
        } else {
            format!(
                "I've been curious about: {}. {}",
                question,
                if !probe.why_interested.is_empty() {
                    format!("Because {}.", probe.why_interested)
                } else {
                    String::new()
                }
            )
        }
    }

    fn express_analytical(&self, probe: &CuriosityProbe) -> String {
        let question = &probe.question;
        
        if let Some(ref answer) = probe.tentative_answer {
            format!(
                "Probe: {} → {} (confidence: {:?})",
                question, answer, probe.confidence
            )
        } else {
            format!(
                "Active probe: {}. Related: {}",
                question,
                probe.related_concepts.join(", ")
            )
        }
    }

    fn express_excited(&self, probe: &CuriosityProbe) -> String {
        let question = &probe.question;
        
        if let Some(ref answer) = probe.tentative_answer {
            format!(
                "Oh! I figured out {}: {}!",
                question, answer
            )
        } else {
            format!(
                "Wait wait wait — {} — I've been exploring this!",
                question
            )
        }
    }

    fn express_contemplative(&self, probe: &CuriosityProbe) -> String {
        let question = &probe.question;
        
        if let Some(ref answer) = probe.tentative_answer {
            format!(
                "On the question of {}: {}.",
                question, answer
            )
        } else {
            format!(
                "{} — this is something I'm still sitting with.",
                question
            )
        }
    }
}

impl Default for ExpressionStyle {
    fn default() -> Self {
        ExpressionStyle::Warm
    }
}
