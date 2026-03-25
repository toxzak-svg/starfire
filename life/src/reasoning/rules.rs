//! Rule Engine
//!
//! If-then rules with forward and backward chaining.
//! Allows Star to make logical inferences from known facts.

use std::collections::{HashMap, VecDeque, HashSet};
use serde::{Deserialize, Serialize};

/// A rule: IF conditions THEN conclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Human-readable description
    pub description: String,
    /// Conditions that must be met (AND)
    pub conditions: Vec<Condition>,
    /// Conclusion when conditions are met
    pub conclusion: Conclusion,
    /// Confidence in this rule (0.0 to 1.0)
    pub confidence: f64,
    /// Source of this rule (how learned)
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// A fact that must be true
    Fact(String),
    /// A fact that must NOT be true
    NotFact(String),
    /// A pattern that must match
    Pattern(String),
    /// A variable binding
    Binding { name: String, pattern: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conclusion {
    /// The conclusion to draw
    pub content: String,
    /// Type of conclusion
    pub kind: ConclusionKind,
    /// Optional: facts this conclusion depends on
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConclusionKind {
    /// A new fact to add to knowledge
    Infer,
    /// An action to take
    Action,
    /// A question to ask
    Question,
}

/// The rule engine — evaluates rules and performs chaining.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleEngine {
    rules: Vec<Rule>,
    /// Working memory of asserted facts
    facts: HashMap<String, f64>, // fact -> confidence
}

impl RuleEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Assert a fact (add to working memory).
    pub fn assert_fact(&mut self, fact: &str, confidence: f64) {
        self.facts.insert(fact.to_lowercase(), confidence);
    }

    /// Get a fact's confidence.
    pub fn get_fact(&self, fact: &str) -> Option<f64> {
        self.facts.get(&fact.to_lowercase()).copied()
    }

    /// Get all asserted facts.
    pub fn facts(&self) -> impl Iterator<Item = (&str, f64)> {
        self.facts.iter().map(|(k, v)| (k.as_str(), *v))
    }

    /// Forward chain: apply all rules whose conditions are met.
    pub fn forward_chain(&self) -> Vec<Conclusion> {
        let mut conclusions = Vec::new();
        
        for rule in &self.rules {
            if self.evaluate_conditions(&rule.conditions) {
                conclusions.push(rule.conclusion.clone());
            }
        }
        
        conclusions
    }

    /// Forward chain iteratively until no new facts.
    pub fn forward_chain_iter(&mut self) -> Vec<Conclusion> {
        let mut all_conclusions = Vec::new();
        let mut changed = true;
        let mut iterations = 0;
        let max_iterations = 100;
        
        while changed && iterations < max_iterations {
            changed = false;
            iterations += 1;
            
            // Collect which rules fire this iteration
            let mut new_facts: Vec<(String, f64)> = Vec::new();
            
            for rule in &self.rules {
                if !self.rule_was_applied(rule) && self.evaluate_conditions(&rule.conditions) {
                    all_conclusions.push(rule.conclusion.clone());
                    
                    if rule.conclusion.kind == ConclusionKind::Infer {
                        new_facts.push((rule.conclusion.content.clone(), rule.confidence));
                        changed = true;
                    }
                }
            }
            
            // Apply new facts after iteration
            for (fact, confidence) in new_facts {
                self.assert_fact(&fact, confidence);
            }
        }
        
        all_conclusions
    }

    /// Backward chain: try to prove a goal by finding supporting rules.
    pub fn backward_chain(&self, goal: &str) -> Option<Proof> {
        let goal_lower = goal.to_lowercase();
        
        // Check if goal is already known
        if let Some(&confidence) = self.facts.get(&goal_lower) {
            return Some(Proof {
                goal: goal.to_string(),
                confidence,
                steps: vec![],
                succeeded: true,
            });
        }
        
        // Find rules that could prove this goal
        for rule in &self.rules {
            if self.rule_concludes(rule, &goal_lower) {
                let sub_goals = self.extract_sub_goals(&rule.conditions);
                
                // Try to prove all sub-goals
                let mut all_proven = true;
                let mut sub_proofs = Vec::new();
                
                for sub_goal in sub_goals {
                    if let Some(proof) = self.backward_chain(&sub_goal) {
                        sub_proofs.push(proof);
                    } else {
                        all_proven = false;
                        break;
                    }
                }
                
                if all_proven {
                    let proof = Proof {
                        goal: goal.to_string(),
                        confidence: rule.confidence,
                        steps: sub_proofs,
                        succeeded: true,
                    };
                    return Some(proof);
                }
            }
        }
        
        None
    }

    /// Evaluate whether a rule has already been applied.
    fn rule_was_applied(&self, rule: &Rule) -> bool {
        if rule.conclusion.kind != ConclusionKind::Infer {
            return false;
        }
        self.facts.contains_key(&rule.conclusion.content.to_lowercase())
    }

    /// Evaluate whether conditions are met.
    fn evaluate_conditions(&self, conditions: &[Condition]) -> bool {
        for condition in conditions {
            match condition {
                Condition::Fact(fact) => {
                    if !self.facts.contains_key(&fact.to_lowercase()) {
                        return false;
                    }
                }
                Condition::NotFact(fact) => {
                    if self.facts.contains_key(&fact.to_lowercase()) {
                        return false;
                    }
                }
                Condition::Pattern(pattern) => {
                    // Check if any fact matches the pattern
                    let matches = self.facts.keys().any(|f| 
                        f.contains(&pattern.to_lowercase())
                    );
                    if !matches { return false; }
                }
                Condition::Binding { name: _, pattern: _ } => {
                    // Binding conditions are for variable unification
                    // Simplified: just check pattern
                }
            }
        }
        true
    }

    /// Check if a rule concludes the given fact.
    fn rule_concludes(&self, rule: &Rule, fact: &str) -> bool {
        rule.conclusion.kind == ConclusionKind::Infer &&
        rule.conclusion.content.to_lowercase().contains(fact)
    }

    /// Extract sub-goals from conditions.
    fn extract_sub_goals(&self, conditions: &[Condition]) -> Vec<String> {
        conditions.iter()
            .filter_map(|c| match c {
                Condition::Fact(fact) => Some(fact.clone()),
                Condition::Pattern(pattern) => Some(pattern.clone()),
                _ => None,
            })
            .collect()
    }

    /// Parse a natural language rule into a Rule.
    pub fn parse_rule(&self, text: &str) -> Option<Rule> {
        let lower = text.to_lowercase();
        
        // Pattern: "IF X THEN Y" or "if X, then Y"
        let if_then = lower.find(" if ").or_else(|| lower.find(" if "))?;
        let then_idx = lower[if_then..].find(" then ")? + if_then;
        
        let condition_text = &text[if_then + 4..then_idx];
        let conclusion_text = &text[then_idx + 6..];
        
        // Parse conditions (simplified: just split on " and ")
        let conditions: Vec<Condition> = condition_text
            .split(" and ")
            .map(|s| Condition::Fact(s.trim().to_string()))
            .collect();
        
        Some(Rule {
            description: text.to_string(),
            conditions,
            conclusion: Conclusion {
                content: conclusion_text.trim().to_string(),
                kind: ConclusionKind::Infer,
                depends_on: Vec::new(),
            },
            confidence: 0.7,
            source: Some("parsed from statement".to_string()),
        })
    }

    /// Get the total number of rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Clear all facts (but keep rules).
    pub fn clear_facts(&mut self) {
        self.facts.clear();
    }
}

/// A proof for backward chaining.
#[derive(Debug)]
pub struct Proof {
    pub goal: String,
    pub confidence: f64,
    pub steps: Vec<Proof>,
    pub succeeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_chaining() {
        let mut engine = RuleEngine::new();
        
        // If fire AND oxygen THEN heat
        engine.add_rule(Rule {
            description: "Fire produces heat".to_string(),
            conditions: vec![
                Condition::Fact("fire".to_string()),
                Condition::Fact("oxygen present".to_string()),
            ],
            conclusion: Conclusion {
                content: "heat produced".to_string(),
                kind: ConclusionKind::Infer,
                depends_on: vec![],
            },
            confidence: 0.9,
            source: None,
        });
        
        engine.assert_fact("fire", 1.0);
        engine.assert_fact("oxygen present", 1.0);
        
        let conclusions = engine.forward_chain();
        assert!(!conclusions.is_empty());
        assert_eq!(conclusions[0].content, "heat produced");
    }

    #[test]
    fn test_backward_chaining() {
        let mut engine = RuleEngine::new();
        
        engine.add_rule(Rule {
            description: "Birds can fly".to_string(),
            conditions: vec![Condition::Fact("is bird".to_string())],
            conclusion: Conclusion {
                content: "can fly".to_string(),
                kind: ConclusionKind::Infer,
                depends_on: vec![],
            },
            confidence: 0.8,
            source: None,
        });
        
        engine.assert_fact("is bird", 1.0);
        
        let proof = engine.backward_chain("can fly");
        assert!(proof.is_some());
        assert!(proof.unwrap().succeeded);
    }

    #[test]
    fn test_parse_rule() {
        let engine = RuleEngine::new();
        let rule = engine.parse_rule("If fire and oxygen then heat");
        
        assert!(rule.is_some());
        let rule = rule.unwrap();
        assert_eq!(rule.conditions.len(), 2);
        assert_eq!(rule.conclusion.content, "heat");
    }
}
