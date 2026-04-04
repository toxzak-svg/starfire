//! Mathematics Module — Symbolic Algebra and Logic
//!
//! Starfire's mathematical reasoning engine. Handles:
//! - Algebraic expression simplification and solving
//! - Propositional and predicate logic
//! - Proof strategies (induction, contradiction, construction)

pub mod symbolic;
pub mod logic;
pub mod proof;

use symbolic::SolvedResult;
use logic::TruthValue;

/// Result of a math operation.
#[derive(Debug, Clone)]
pub enum MathResult {
    /// A symbolic expression result
    Expression(String),
    /// A solved equation result
    Solved(SolvedResult),
    /// A logical result
    Logical { proposition: String, truth: TruthValue },
    /// Proof attempt result
    Proof {
        proved: bool,
        steps: Vec<String>,
        conclusion: String,
    },
    /// Error
    Error(String),
}

impl MathResult {
    /// Format for display.
    pub fn display(&self) -> String {
        match self {
            MathResult::Expression(s) => s.clone(),
            MathResult::Solved(r) => r.display(),
            MathResult::Logical { proposition, truth } => {
                format!("{}: {}", proposition, truth)
            }
            MathResult::Proof { proved, steps, conclusion } => {
                let mut lines = vec![];
                if *proved {
                    lines.push("Proof:".to_string());
                } else {
                    lines.push("Proof attempt (failed):".to_string());
                }
                for step in steps {
                    lines.push(format!("  {}", step));
                }
                lines.push(format!("Conclusion: {}", conclusion));
                lines.join("\n")
            }
            MathResult::Error(s) => format!("Error: {}", s),
        }
    }

    /// Get the answer string.
    pub fn answer(&self) -> String {
        match self {
            MathResult::Expression(s) => s.clone(),
            MathResult::Solved(r) => r.display(),
            MathResult::Logical { proposition, .. } => proposition.clone(),
            MathResult::Proof { conclusion, .. } => conclusion.clone(),
            MathResult::Error(s) => s.clone(),
        }
    }
}

/// The math engine — routes math queries to appropriate sub-modules.
pub struct MathEngine {
    symbolic: symbolic::AlgebraEngine,
    logic: logic::LogicEngine,
    proof: proof::ProofEngine,
}

impl MathEngine {
    pub fn new() -> Self {
        Self {
            symbolic: symbolic::AlgebraEngine::new(),
            logic: logic::LogicEngine::new(),
            proof: proof::ProofEngine::new(),
        }
    }

    /// Attempt to solve or simplify a math expression.
    pub fn solve(&mut self, input: &str) -> MathResult {
        // First, check if it's an equation (contains '=')
        if input.contains('=') {
            if let Some(result) = self.symbolic.solve_equation(input) {
                return result;
            }
        }

        // Try simplifying
        if let Some(expr) = self.symbolic.simplify(input) {
            return MathResult::Expression(expr);
        }

        // Try evaluating a numeric expression
        if let Some(result) = self.symbolic.evaluate(input) {
            return MathResult::Expression(result);
        }

        MathResult::Error(format!("Could not parse or solve: {}", input))
    }

    /// Evaluate a logical proposition.
    pub fn evaluate_logic(&mut self, proposition: &str) -> MathResult {
        match self.logic.evaluate(proposition) {
            Ok((prop_str, truth)) => {
                MathResult::Logical {
                    proposition: prop_str,
                    truth,
                }
            }
            Err(e) => MathResult::Error(e),
        }
    }

    /// Attempt a proof.
    pub fn prove(&mut self, theorem: &str, strategy: &str) -> MathResult {
        match self.proof.prove(theorem, strategy) {
            Ok((proved, steps, conclusion)) => MathResult::Proof {
                proved,
                steps,
                conclusion,
            },
            Err(e) => MathResult::Error(e),
        }
    }

    /// Get a reasoning chain explanation for a math result.
    pub fn explain(&self, result: &MathResult) -> String {
        match result {
            MathResult::Solved(r) => {
                if r.steps.is_empty() {
                    format!("x = {}", r.solution)
                } else {
                    let steps_str: Vec<String> = r.steps
                        .iter()
                        .enumerate()
                        .map(|(i, s)| format!("{}. {}", i + 1, s))
                        .collect();
                    format!(
                        "Solving for {}:\n{}\n\n∴ x = {}",
                        r.variable,
                        steps_str.join("\n"),
                        r.solution
                    )
                }
            }
            _ => result.display(),
        }
    }
}

impl Default for MathEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_solve() {
        let mut engine = MathEngine::new();
        let result = engine.solve("2 * x + 3 = 7");
        let answer = result.answer();
        assert!(answer.contains("2") || answer.contains("x = 2"));
    }

    #[test]
    fn test_simplify() {
        let mut engine = MathEngine::new();
        let result = engine.solve("2 + 2 + 2");
        assert!(result.answer().contains("6") || result.answer().contains("2 + 2 + 2"));
    }
}
