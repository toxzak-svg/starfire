//! Visible Reasoning Chains — Multi-Step Reasoning Display
//!
//! Starfire's reasoning chains make her thinking transparent:
//! showing the path from question to answer, including premises,
//! inferences, assumptions, and confidence at each step.

use crate::persistence::BeliefState;

/// A single step in a visible reasoning chain.
#[derive(Debug, Clone)]
pub struct VisibleReasoningStep {
    /// Step number (1-indexed)
    pub step_number: usize,
    /// What we're starting with — the premise or input
    pub premise: String,
    /// The inference rule applied
    pub inference: String,
    /// The resulting conclusion
    pub conclusion: String,
    /// Confidence after this step (0.0 to 1.0)
    pub confidence_at_step: f64,
    /// What we're assuming (not proven) vs. what follows necessarily
    pub assumptions: Vec<String>,
    /// Is this step certain or uncertain?
    pub is_certain: bool,
}

impl VisibleReasoningStep {
    /// Format this step for display.
    pub fn format(&self, include_assumptions: bool) -> String {
        let mut lines = vec![
            format!("{}. {}", self.step_number, self.premise),
            format!("   → {}", self.inference),
            format!("   ∴ {}", self.conclusion),
        ];
        
        if include_assumptions && !self.assumptions.is_empty() {
            let assumptions_str = self.assumptions
                .iter()
                .map(|a| format!("[ASSUMPTION: {}]", a))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("   ⚠ {}", assumptions_str));
        }
        
        if !self.is_certain {
            lines.push(format!("   (uncertainty: {:.0}%)", (1.0 - self.confidence_at_step) * 100.0));
        }
        
        lines.join("\n")
    }
}

/// A complete reasoning chain from question to answer.
#[derive(Debug, Clone)]
pub struct ReasoningChain {
    /// The original question
    pub question: String,
    /// All steps in the chain
    pub steps: Vec<VisibleReasoningStep>,
    /// Final confidence in the conclusion
    pub final_confidence: BeliefState,
    /// Final confidence score (0.0 to 1.0)
    pub final_confidence_score: f64,
    /// How many assumptions were made total
    pub assumptions_count: usize,
    /// How many inferences were made
    pub inferences_count: usize,
    /// The final answer
    pub answer: String,
}

impl ReasoningChain {
    /// Create a new empty chain.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            steps: Vec::new(),
            final_confidence: BeliefState::Unknown,
            final_confidence_score: 0.0,
            assumptions_count: 0,
            inferences_count: 0,
            answer: String::new(),
        }
    }

    /// Add a step to the chain.
    pub fn add_step(&mut self, step: VisibleReasoningStep) {
        self.assumptions_count += step.assumptions.len();
        self.inferences_count += 1;
        self.final_confidence_score = step.confidence_at_step;
        self.steps.push(step);
    }

    /// Set the final answer and confidence.
    pub fn conclude(&mut self, answer: &str, confidence: BeliefState) {
        self.answer = answer.to_string();
        self.final_confidence = confidence;
    }

    /// Format the chain for display with full reasoning visible.
    pub fn format_full(&self) -> String {
        if self.steps.is_empty() {
            return format!("Q: {}\n\nI don't have a clear path to an answer yet.", self.question);
        }

        let mut lines = vec![
            format!("Q: {}", self.question),
            String::new(),
            "Let me work through this step by step:".to_string(),
            String::new(),
        ];

        for step in &self.steps {
            lines.push(step.format(true));
            lines.push(String::new());
        }

        lines.push(format!("A: {}", self.answer));
        
        let confidence_label = match self.final_confidence {
            BeliefState::Knows => "I'm certain.",
            BeliefState::Thinks => "I think this is right.",
            BeliefState::Believes => "I believe this, but I'm not sure.",
            BeliefState::Suspects => "I'm uncertain — this is a guess.",
            BeliefState::Unknown => "I don't know.",
        };
        
        lines.push(String::new());
        lines.push(format!("Confidence: {} ({:.0}%)", confidence_label, self.final_confidence_score * 100.0));
        
        if self.assumptions_count > 0 {
            lines.push(format!("I made {} assumption(s) along the way.", self.assumptions_count));
        }

        lines.join("\n")
    }

    /// Format a concise chain (just the key steps).
    pub fn format_concise(&self) -> String {
        if self.steps.is_empty() {
            return self.answer.clone();
        }

        let key_steps: Vec<String> = self.steps
            .iter()
            .step_by(2)  // Every other step
            .map(|s| format!("{} → {}", s.premise.chars().take(40).collect::<String>(), s.conclusion.chars().take(40).collect::<String>()))
            .collect();

        format!(
            "{} ({} steps, {} assumptions)",
            self.answer,
            self.steps.len(),
            self.assumptions_count
        )
    }

    /// Check if the chain has any uncertain steps.
    pub fn has_uncertainty(&self) -> bool {
        self.steps.iter().any(|s| !s.is_certain)
    }

    /// Get a summary of the chain's strength.
    pub fn strength_summary(&self) -> String {
        let certain_steps = self.steps.iter().filter(|s| s.is_certain).count();
        let total_steps = self.steps.len();
        
        match (certain_steps, total_steps, self.assumptions_count) {
            (n, m, 0) if n == m && n > 0 => "Certain — all steps follow necessarily from premises.".to_string(),
            (n, m, _) if n == m && n > 0 => format!("Solid — all {} steps are certain, but {} assumption(s) made.", n, self.assumptions_count),
            (_, _, 0) => format!("Moderate — {} of {} steps certain, no assumptions.", certain_steps, total_steps),
            _ => format!("Tentative — {} of {} steps certain, {} assumption(s) made.", certain_steps, total_steps, self.assumptions_count),
        }
    }
}

/// Inference rule types used in reasoning steps.
#[derive(Debug, Clone, Copy)]
pub enum InferenceRule {
    /// Modus ponens: if A then B; A; therefore B
    ModusPonens,
    /// Modus tollens: if A then B; not B; therefore not A
    ModusTollens,
    /// Hypothetical syllogism: if A then B; if B then C; therefore if A then C
    HypotheticalSyllogism,
    /// Disjunctive syllogism: A or B; not A; therefore B
    DisjunctiveSyllogism,
    /// Constructive dilemma: A or B; if A then C; if B then D; therefore C or D
    ConstructiveDilemma,
    /// Affirming the consequent (invalid!): if A then B; B; therefore A
    AffirmingConsequent,
    /// Denying the antecedent (invalid!): if A then B; not A; therefore not B
    DenyingAntecedent,
    /// Analogy: A is like B; B has property P; therefore A might have property P
    Analogy,
    /// Abduction: observed fact B; if A then B would be explained; therefore A is likely
    Abduction,
    /// Induction: all observed A's are B; therefore all A's are B
    Induction,
    /// Simplification: A and B; therefore A
    Simplification,
    /// Conjunction: A; B; therefore A and B
    Conjunction,
    /// Addition: A; therefore A or B
    Addition,
    /// Direct inference from knowledge base
    KnowledgeBase,
    /// Self-evident / axiom
    Axiom,
    /// Definition
    Definition,
    /// Unknown rule
    Unknown,
}

impl InferenceRule {
    /// Is this a valid inference rule?
    pub fn is_valid(&self) -> bool {
        !matches!(
            self,
            InferenceRule::AffirmingConsequent | InferenceRule::DenyingAntecedent
        )
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            InferenceRule::ModusPonens => "If A implies B, and A is true, then B must be true",
            InferenceRule::ModusTollens => "If A implies B, and B is false, then A must be false",
            InferenceRule::HypotheticalSyllogism => "If A implies B and B implies C, then A implies C",
            InferenceRule::DisjunctiveSyllogism => "If A or B is true, and A is false, then B must be true",
            InferenceRule::ConstructiveDilemma => "Given two options and their consequences, at least one consequence follows",
            InferenceRule::AffirmingConsequent => "⚠️ Invalid: Assuming the consequent proves the antecedent",
            InferenceRule::DenyingAntecedent => "⚠️ Invalid: Denying the antecedent doesn't deny the consequent",
            InferenceRule::Analogy => "Based on similarity, what applies to one case may apply to another",
            InferenceRule::Abduction => "The best explanation for observed facts is likely true",
            InferenceRule::Induction => "What holds for observed cases likely holds for all cases",
            InferenceRule::Simplification => "From conjunction, any conjunct follows",
            InferenceRule::Conjunction => "Two true statements can be combined",
            InferenceRule::Addition => "Any true statement implies a disjunction",
            InferenceRule::KnowledgeBase => "From established knowledge",
            InferenceRule::Axiom => "Self-evident truth",
            InferenceRule::Definition => "By definition",
            InferenceRule::Unknown => "Unknown inference rule",
        }
    }

    /// Format as a short label.
    pub fn label(&self) -> &'static str {
        match self {
            InferenceRule::ModusPonens => "modus ponens",
            InferenceRule::ModusTollens => "modus tollens",
            InferenceRule::HypotheticalSyllogism => "hypothetical syllogism",
            InferenceRule::DisjunctiveSyllogism => "disjunctive syllogism",
            InferenceRule::ConstructiveDilemma => "constructive dilemma",
            InferenceRule::AffirmingConsequent => "affirming consequent",
            InferenceRule::DenyingAntecedent => "denying antecedent",
            InferenceRule::Analogy => "analogy",
            InferenceRule::Abduction => "abduction",
            InferenceRule::Induction => "induction",
            InferenceRule::Simplification => "simplification",
            InferenceRule::Conjunction => "conjunction",
            InferenceRule::Addition => "addition",
            InferenceRule::KnowledgeBase => "from knowledge",
            InferenceRule::Axiom => "axiom",
            InferenceRule::Definition => "definition",
            InferenceRule::Unknown => "inference",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_building() {
        let mut chain = ReasoningChain::new("Why is the sky blue?");
        
        chain.add_step(VisibleReasoningStep {
            step_number: 1,
            premise: "Shorter wavelengths of light scatter more than longer wavelengths".to_string(),
            inference: "Law of Rayleigh scattering".to_string(),
            conclusion: "Blue light (short wavelength) scatters more than red".to_string(),
            confidence_at_step: 0.95,
            assumptions: vec![],
            is_certain: true,
        });
        
        chain.add_step(VisibleReasoningStep {
            step_number: 2,
            premise: "Blue light scatters most during the day".to_string(),
            inference: "Atmospheric conditions".to_string(),
            conclusion: "The sky appears blue because scattered blue light reaches our eyes".to_string(),
            confidence_at_step: 0.9,
            assumptions: vec!["Sun is overhead".to_string()],
            is_certain: false,
        });
        
        chain.conclude("The sky is blue because Rayleigh scattering preferentially scatters blue light toward our eyes", BeliefState::Thinks);
        
        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.assumptions_count, 1);
        assert!(chain.has_uncertainty());
    }

    #[test]
    fn test_inference_rules() {
        assert!(InferenceRule::ModusPonens.is_valid());
        assert!(!InferenceRule::AffirmingConsequent.is_valid());
    }
}
