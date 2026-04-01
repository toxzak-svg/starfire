//! Rule Engine — if-then rules with forward/backward chaining

use std::collections::VecDeque;

/// A single if-then rule.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: i64,
    pub if_parts: Vec<String>,
    pub then_part: String,
    pub confidence: f64,
    pub source: Option<String>,
}

/// Rule engine — manages if-then rules and executes chaining.
#[derive(Default, Clone)]
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Parse a rule from text (simple "if X then Y" pattern).
    pub fn parse_rule(&self, text: &str) -> Option<Rule> {
        let lower = text.to_lowercase();
        let if_pos = lower.find(" if ")?;
        let then_pos = lower.find(" then ")?;
        if then_pos <= if_pos { return None; }
        let if_part = text[if_pos + 4..then_pos].trim().to_string();
        let then_part = text[then_pos + 7..].trim().to_string();
        if if_part.is_empty() || then_part.is_empty() { return None; }
        Some(Rule {
            id: 0,
            if_parts: vec![if_part],
            then_part,
            confidence: 0.8,
            source: Some(text.to_string()),
        })
    }

    /// Forward chain — apply rules whose conditions are satisfied.
    pub fn forward_chain(&self, facts: &[String]) -> Vec<String> {
        let mut results = Vec::new();
        for rule in &self.rules {
            for fact in facts {
                if rule.if_parts.iter().any(|p| fact.to_lowercase().contains(&p.to_lowercase())) {
                    results.push(rule.then_part.clone());
                }
            }
        }
        results
    }
}
