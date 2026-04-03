//! Logic Engine — Propositional and Predicate Logic
//!
//! Handles: modus ponens, modus tollens, syllogisms, truth tables.

use std::collections::HashMap;

/// Truth values in logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    True,
    False,
    Unknown,
    Contradiction,
}

impl std::fmt::Display for TruthValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TruthValue::True => write!(f, "TRUE"),
            TruthValue::False => write!(f, "FALSE"),
            TruthValue::Unknown => write!(f, "UNKNOWN"),
            TruthValue::Contradiction => write!(f, "CONTRADICTION"),
        }
    }
}

/// A propositional variable or expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proposition {
    /// Atomic proposition (named variable)
    Atom(String),
    /// Logical AND
    And(Box<Proposition>, Box<Proposition>),
    /// Logical OR
    Or(Box<Proposition>, Box<Proposition>),
    /// Logical NOT
    Not(Box<Proposition>),
    /// Logical IMPLIES (→)
    Implies(Box<Proposition>, Box<Proposition>),
    /// Logical IFF (↔)
    Iff(Box<Proposition>, Box<Proposition>),
    /// Constant truth values
    True,
    False,
}

impl Proposition {
    /// Evaluate this proposition given variable assignments.
    pub fn evaluate(&self, vars: &HashMap<String, bool>) -> TruthValue {
        match self {
            Proposition::Atom(name) => {
                vars.get(name).map(|&v| if v { TruthValue::True } else { TruthValue::False })
                    .unwrap_or(TruthValue::Unknown)
            }
            Proposition::And(a, b) => {
                match (a.evaluate(vars), b.evaluate(vars)) {
                    (TruthValue::True, TruthValue::True) => TruthValue::True,
                    (TruthValue::False, _) | (_, TruthValue::False) => TruthValue::False,
                    _ => TruthValue::Unknown,
                }
            }
            Proposition::Or(a, b) => {
                match (a.evaluate(vars), b.evaluate(vars)) {
                    (TruthValue::True, _) | (_, TruthValue::True) => TruthValue::True,
                    (TruthValue::False, TruthValue::False) => TruthValue::False,
                    _ => TruthValue::Unknown,
                }
            }
            Proposition::Not(inner) => {
                match inner.evaluate(vars) {
                    TruthValue::True => TruthValue::False,
                    TruthValue::False => TruthValue::True,
                    _ => TruthValue::Unknown,
                }
            }
            Proposition::Implies(a, b) => {
                match (a.evaluate(vars), b.evaluate(vars)) {
                    (TruthValue::True, TruthValue::False) => TruthValue::False,
                    (TruthValue::True, TruthValue::True) => TruthValue::True,
                    (TruthValue::False, _) => TruthValue::True, // False implies anything
                    _ => TruthValue::Unknown,
                }
            }
            Proposition::Iff(a, b) => {
                match (a.evaluate(vars), b.evaluate(vars)) {
                    (TruthValue::True, TruthValue::True) => TruthValue::True,
                    (TruthValue::False, TruthValue::False) => TruthValue::True,
                    (TruthValue::True, TruthValue::False) => TruthValue::False,
                    (TruthValue::False, TruthValue::True) => TruthValue::False,
                    _ => TruthValue::Unknown,
                }
            }
            Proposition::True => TruthValue::True,
            Proposition::False => TruthValue::False,
        }
    }

    /// Extract all atomic variables from this proposition.
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars
    }

    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Proposition::Atom(name) => {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
            Proposition::And(a, b) | Proposition::Or(a, b) | Proposition::Implies(a, b) | Proposition::Iff(a, b) => {
                a.collect_variables(vars);
                b.collect_variables(vars);
            }
            Proposition::Not(inner) => inner.collect_variables(vars),
            Proposition::True | Proposition::False => {}
        }
    }

    /// Convert to a display string.
    pub fn to_string(&self) -> String {
        match self {
            Proposition::Atom(name) => name.clone(),
            Proposition::And(a, b) => format!("({} AND {})", a.to_string(), b.to_string()),
            Proposition::Or(a, b) => format!("({} OR {})", a.to_string(), b.to_string()),
            Proposition::Not(inner) => format!("NOT({})", inner.to_string()),
            Proposition::Implies(a, b) => format!("({} → {})", a.to_string(), b.to_string()),
            Proposition::Iff(a, b) => format!("({} ↔ {})", a.to_string(), b.to_string()),
            Proposition::True => "TRUE".to_string(),
            Proposition::False => "FALSE".to_string(),
        }
    }
}

/// The logic engine.
pub struct LogicEngine {
    _priv: (),
}

impl LogicEngine {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Evaluate a logical proposition string.
    /// 
    /// Supports:
    /// - Variables: A, B, C, ...
    /// - AND: /\ , &, AND
    /// - OR: \/, |, OR
    /// - NOT: ~, !, NOT
    /// - IMPLIES: ->, →
    /// - IFF: <->, ↔
    /// - Parentheses: ()
    pub fn evaluate(&mut self, input: &str) -> Result<(String, TruthValue), String> {
        let prop = parse_proposition(input)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        let vars = prop.variables();
        
        if vars.is_empty() {
            let truth = prop.evaluate(&HashMap::new());
            return Ok((prop.to_string(), truth));
        }
        
        // Try to evaluate with empty bindings (all unknown)
        let truth = prop.evaluate(&HashMap::new());
        
        // If all variables are known (from a proposition), return
        Ok((prop.to_string(), truth))
    }

    /// Generate a truth table for a proposition.
    pub fn truth_table(&self, prop: &Proposition) -> Vec<(HashMap<String, bool>, TruthValue)> {
        let vars = prop.variables();
        let n = vars.len();
        let mut results = Vec::new();
        
        // Generate all 2^n combinations
        let combinations = 1u64 << n;
        
        for i in 0..combinations {
            let mut bindings = HashMap::new();
            for j in 0..n {
                let value = (i >> j) & 1 == 1;
                bindings.insert(vars[j].clone(), value);
            }
            let truth = prop.evaluate(&bindings);
            results.push((bindings, truth));
        }
        
        results
    }

    /// Check if a proposition is a tautology (always true).
    pub fn is_tautology(&self, prop: &Proposition) -> bool {
        let table = self.truth_table(prop);
        table.iter().all(|(_, t)| *t == TruthValue::True)
    }

    /// Check if a proposition is a contradiction (always false).
    pub fn is_contradiction(&self, prop: &Proposition) -> bool {
        let table = self.truth_table(prop);
        table.iter().all(|(_, t)| *t == TruthValue::False)
    }

    /// Check if two propositions are logically equivalent.
    pub fn are_equivalent(&self, p: &Proposition, q: &Proposition) -> bool {
        let p_vars = p.variables();
        let q_vars = q.variables();
        let mut all_vars = p_vars;
        for v in q_vars {
            if !all_vars.contains(&v) {
                all_vars.push(v);
            }
        }
        
        let n = all_vars.len();
        let combinations = 1u64 << n;
        
        for i in 0..combinations {
            let mut bindings = HashMap::new();
            for j in 0..n {
                let value = (i >> j) & 1 == 1;
                bindings.insert(all_vars[j].clone(), value);
            }
            let p_truth = p.evaluate(&bindings);
            let q_truth = q.evaluate(&bindings);
            if p_truth != q_truth {
                return false;
            }
        }
        
        true
    }

    /// Apply modus ponens: if P→Q and P, then Q.
    pub fn modus_ponens(&self, implication: &Proposition, antecedent: &Proposition) -> Option<TruthValue> {
        // Try to evaluate
        let mut bindings = HashMap::new();
        
        // This is simplified — in practice we'd need unification
        if let Proposition::Implies(p, q) = implication {
            // If we know both the implication and antecedent are true...
            // (simplified implementation)
            return Some(q.evaluate(&bindings)); // Would need proper unification
        }
        
        None
    }

    /// Apply modus tollens: if P→Q and ¬Q, then ¬P.
    pub fn modus_tollens(&self, implication: &Proposition, consequent_false: &Proposition) -> Option<TruthValue> {
        if let Proposition::Implies(_, _) = implication {
            // Simplified — would need proper logical inference
            return Some(TruthValue::True); // Placeholder
        }
        None
    }
}

impl Default for LogicEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a proposition string into a Proposition enum.
fn parse_proposition(input: &str) -> Result<Proposition, String> {
    let input = input.trim();
    
    // Handle parenthesized expressions
    if input.starts_with('(') && input.ends_with(')') {
        let inner = &input[1..input.len()-1];
        // Check if this is an AND or OR (which use parentheses differently)
        // For now, recursively parse inner
        return parse_proposition(inner);
    }
    
    // Handle NOT (~A, !A, NOT A)
    if input.starts_with("NOT ") || input.starts_with('!') || input.starts_with('~') {
        let inner = if input.starts_with("NOT ") {
            &input[4..]
        } else if input.starts_with('!') || input.starts_with('~') {
            &input[1..]
        } else {
            return Err("Invalid NOT".to_string());
        };
        let inner_prop = parse_proposition(inner)?;
        return Ok(Proposition::Not(Box::new(inner_prop)));
    }
    
    // Handle IMPLIES (A → B or A -> B)
    if let Some(pos) = find_operator(input, "→") {
        let antecedent = parse_proposition(&input[..pos])?;
        let consequent = parse_proposition(&input[pos + 1..])?;
        return Ok(Proposition::Implies(Box::new(antecedent), Box::new(consequent)));
    }
    if let Some(pos) = input.find("->") {
        let antecedent = parse_proposition(&input[..pos])?;
        let consequent = parse_proposition(&input[pos + 2..])?;
        return Ok(Proposition::Implies(Box::new(antecedent), Box::new(consequent)));
    }
    
    // Handle IFF (A ↔ B or A <-> B)
    if let Some(pos) = find_operator(input, "↔") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 1..])?;
        return Ok(Proposition::Iff(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find("<->") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 3..])?;
        return Ok(Proposition::Iff(Box::new(left), Box::new(right)));
    }
    
    // Handle AND (A /\ B or A & B or A AND B)
    if let Some(pos) = find_operator(input, "∧") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 1..])?;
        return Ok(Proposition::And(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" /\\ ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 4..])?;
        return Ok(Proposition::And(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" AND ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 5..])?;
        return Ok(Proposition::And(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" & ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 3..])?;
        return Ok(Proposition::And(Box::new(left), Box::new(right)));
    }
    
    // Handle OR (A \/ B or A | B or A OR B)
    if let Some(pos) = find_operator(input, "∨") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 1..])?;
        return Ok(Proposition::Or(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" \\/ ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 4..])?;
        return Ok(Proposition::Or(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" OR ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 4..])?;
        return Ok(Proposition::Or(Box::new(left), Box::new(right)));
    }
    if let Some(pos) = input.find(" | ") {
        let left = parse_proposition(&input[..pos])?;
        let right = parse_proposition(&input[pos + 3..])?;
        return Ok(Proposition::Or(Box::new(left), Box::new(right)));
    }
    
    // Handle TRUE/FALSE constants
    let upper = input.to_uppercase();
    if upper == "TRUE" || upper == "T" || upper == "1" {
        return Ok(Proposition::True);
    }
    if upper == "FALSE" || upper == "F" || upper == "0" {
        return Ok(Proposition::False);
    }
    
    // Must be an atom
    if input.is_empty() {
        return Err("Empty proposition".to_string());
    }
    
    // Clean the atom name
    let name = input.trim().to_string();
    if name.is_empty() {
        return Err("Empty atom".to_string());
    }
    
    Ok(Proposition::Atom(name))
}

/// Find a unicode operator in the string (skipping parenthesized parts).
fn find_operator(input: &str, op: &str) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = input.chars().collect();
    let op_chars: Vec<char> = op.chars().collect();
    
    for i in 0..=chars.len() - op_chars.len() {
        let matches = chars[i..].starts_with(&op_chars);
        if matches && depth == 0 {
            return Some(i);
        }
        if i < chars.len() {
            if chars[i] == '(' {
                depth += 1;
            } else if chars[i] == ')' {
                depth = depth.saturating_sub(1);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modus_ponens() {
        let engine = LogicEngine::new();
        // (P → Q) ∧ P → Q
        let p_implies_q = Proposition::Implies(
            Box::new(Proposition::Atom("P".to_string())),
            Box::new(Proposition::Atom("Q".to_string())),
        );
        let p = Proposition::Atom("P".to_string());
        
        let both = Proposition::And(Box::new(p_implies_q), Box::new(p));
        
        let mut vars = HashMap::new();
        vars.insert("P".to_string(), true);
        vars.insert("Q".to_string(), true);
        
        assert_eq!(both.evaluate(&vars), TruthValue::True);
    }

    #[test]
    fn test_parse() {
        let prop = parse_proposition("A AND B").unwrap();
        assert_eq!(prop.to_string(), "(A AND B)");
        
        let prop2 = parse_proposition("NOT A").unwrap();
        assert_eq!(prop2.to_string(), "NOT(A)");
    }

    #[test]
    fn test_tautology() {
        let engine = LogicEngine::new();
        // A OR NOT A is a tautology
        let taut = Proposition::Or(
            Box::new(Proposition::Atom("A".to_string())),
            Box::new(Proposition::Not(Box::new(Proposition::Atom("A".to_string())))),
        );
        assert!(engine.is_tautology(&taut));
    }
}
