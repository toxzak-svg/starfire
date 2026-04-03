//! Symbolic Algebra Engine
//!
//! Handles algebraic expressions: parsing, simplification, and solving.

use std::collections::HashMap;

/// A symbolic algebraic expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal number
    Number(f64),
    /// A variable (e.g., x)
    Variable(String),
    /// Addition: a + b
    Add(Box<Expression>, Box<Expression>),
    /// Subtraction: a - b
    Sub(Box<Expression>, Box<Expression>),
    /// Multiplication: a * b
    Mul(Box<Expression>, Box<Expression>),
    /// Division: a / b
    Div(Box<Expression>, Box<Expression>),
    /// Power: a^b
    Pow(Box<Expression>, Box<Expression>),
    /// Negation: -a
    Neg(Box<Expression>),
    /// Parenthesized expression
    Group(Box<Expression>),
}

impl Expression {
    /// Simplify this expression.
    pub fn simplify(&self) -> Expression {
        match self {
            // Numbers stay the same
            Expression::Number(_) => self.clone(),
            Expression::Variable(_) => self.clone(),
            
            // Simplify additions
            Expression::Add(a, b) => {
                let a_s = a.simplify();
                let b_s = b.simplify();
                match (&a_s, &b_s) {
                    (Expression::Number(0.0), _) => b_s,
                    (_, Expression::Number(0.0)) => a_s,
                    (Expression::Number(x), Expression::Number(y)) => Expression::Number(x + y),
                    _ => Expression::Add(Box::new(a_s), Box::new(b_s)),
                }
            }
            
            // Simplify subtractions
            Expression::Sub(a, b) => {
                let a_s = a.simplify();
                let b_s = b.simplify();
                match (&a_s, &b_s) {
                    (_, Expression::Number(0.0)) => a_s,
                    (Expression::Number(x), Expression::Number(y)) => Expression::Number(x - y),
                    _ => Expression::Sub(Box::new(a_s), Box::new(b_s)),
                }
            }
            
            // Simplify multiplications
            Expression::Mul(a, b) => {
                let a_s = a.simplify();
                let b_s = b.simplify();
                match (&a_s, &b_s) {
                    (Expression::Number(0.0), _) => Expression::Number(0.0),
                    (_, Expression::Number(0.0)) => Expression::Number(0.0),
                    (Expression::Number(1.0), _) => b_s,
                    (_, Expression::Number(1.0)) => a_s,
                    (Expression::Number(x), Expression::Number(y)) => Expression::Number(x * y),
                    _ => Expression::Mul(Box::new(a_s), Box::new(b_s)),
                }
            }
            
            // Simplify divisions
            Expression::Div(a, b) => {
                let a_s = a.simplify();
                let b_s = b.simplify();
                match (&a_s, &b_s) {
                    (Expression::Number(0.0), _) => Expression::Number(0.0),
                    (_, Expression::Number(1.0)) => a_s,
                    (Expression::Number(x), Expression::Number(y)) if *y != 0.0 => Expression::Number(x / y),
                    _ => Expression::Div(Box::new(a_s), Box::new(b_s)),
                }
            }
            
            // Simplify powers
            Expression::Pow(a, b) => {
                let a_s = a.simplify();
                let b_s = b.simplify();
                match (&a_s, &b_s) {
                    (_, Expression::Number(0.0)) => Expression::Number(1.0),
                    (_, Expression::Number(1.0)) => a_s,
                    (Expression::Number(x), Expression::Number(y)) => {
                        Expression::Number(x.powf(*y))
                    }
                    _ => Expression::Pow(Box::new(a_s), Box::new(b_s)),
                }
            }
            
            // Simplify negation
            Expression::Neg(a) => {
                let a_s = a.simplify();
                match &a_s {
                    Expression::Number(x) => Expression::Number(-x),
                    Expression::Neg(inner) => *inner.clone(),
                    _ => Expression::Neg(Box::new(a_s)),
                }
            }
            
            // Groups just simplify inner
            Expression::Group(a) => a.simplify(),
        }
    }

    /// Evaluate this expression given variable bindings.
    pub fn evaluate(&self, vars: &HashMap<String, f64>) -> Option<f64> {
        match self {
            Expression::Number(n) => Some(*n),
            Expression::Variable(v) => vars.get(v).copied(),
            Expression::Add(a, b) => Some(a.evaluate(vars)? + b.evaluate(vars)?),
            Expression::Sub(a, b) => Some(a.evaluate(vars)? - b.evaluate(vars)?),
            Expression::Mul(a, b) => Some(a.evaluate(vars)? * b.evaluate(vars)?),
            Expression::Div(a, b) => {
                let denom = b.evaluate(vars)?;
                if denom == 0.0 { None } else { Some(a.evaluate(vars)? / denom) }
            }
            Expression::Pow(a, b) => {
                Some(a.evaluate(vars)?.powf(b.evaluate(vars)?))
            }
            Expression::Neg(a) => Some(-a.evaluate(vars)?),
            Expression::Group(a) => a.evaluate(vars),
        }
    }

    /// Convert to a display string.
    pub fn to_string(&self) -> String {
        match self {
            Expression::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Expression::Variable(v) => v.clone(),
            Expression::Add(a, b) => format!("({} + {})", a.to_string(), b.to_string()),
            Expression::Sub(a, b) => format!("({} - {})", a.to_string(), b.to_string()),
            Expression::Mul(a, b) => format!("({} * {})", a.to_string(), b.to_string()),
            Expression::Div(a, b) => format!("({} / {})", a.to_string(), b.to_string()),
            Expression::Pow(a, b) => format!("({} ^ {})", a.to_string(), b.to_string()),
            Expression::Neg(a) => format!("-{}", a.to_string()),
            Expression::Group(a) => format!("({})", a.to_string()),
        }
    }
}

/// A solved equation result.
#[derive(Debug, Clone)]
pub struct SolvedResult {
    pub variable: String,
    pub solution: String,
    pub steps: Vec<String>,
}

/// Algebra engine for parsing and solving equations.
pub struct AlgebraEngine {
    _priv: (),
}

impl AlgebraEngine {
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Try to solve a linear equation (ax + b = c).
    pub fn solve_linear(&self, a: f64, b: f64, c: f64) -> Option<SolvedResult> {
        if a == 0.0 {
            return None; // Not actually linear in x
        }
        
        let mut steps = Vec::new();
        
        // ax + b = c
        steps.push(format!("Starting with: {}x + {} = {}", a, b, c));
        
        // Subtract b: ax = c - b
        let c_minus_b = c - b;
        steps.push(format!("Subtract {} from both sides: {}x = {}", b, a, c_minus_b));
        
        // Divide by a: x = (c - b) / a
        let solution = c_minus_b / a;
        steps.push(format!("Divide both sides by {}: x = {} / {} = {}", a, c_minus_b, a, solution));
        
        Some(SolvedResult {
            variable: "x".to_string(),
            solution: format!("{:.4}", solution).trim_end_matches('0').trim_end_matches('.').to_string(),
            steps,
        })
    }

    /// Try to solve a quadratic equation (ax² + bx + c = 0).
    /// Returns the positive and negative solutions.
    pub fn solve_quadratic(&self, a: f64, b: f64, c: f64) -> Option<(SolvedResult, SolvedResult)> {
        if a == 0.0 {
            return None; // Not quadratic
        }
        
        let mut steps = Vec::new();
        
        // Discriminant: b² - 4ac
        steps.push(format!("Using quadratic formula for {}x² + {}x + {} = 0", a, b, c));
        
        let discriminant = b * b - 4.0 * a * c;
        steps.push(format!("Discriminant: b² - 4ac = {}² - 4({})({}) = {}", b, a, c, discriminant));
        
        if discriminant < 0.0 {
            steps.push("Discriminant is negative — no real solutions.".to_string());
            return None;
        }
        
        if discriminant == 0.0 {
            let solution = -b / (2.0 * a);
            steps.push(format!("One repeated solution: x = -b / 2a = {} / {} = {}", -b, 2.0 * a, solution));
            let result = SolvedResult {
                variable: "x".to_string(),
                solution: format!("{:.4}", solution).trim_end_matches('0').trim_end_matches('.').to_string(),
                steps,
            };
            return Some((result.clone(), result));
        }
        
        let sqrt_d = discriminant.sqrt();
        steps.push(format!("√{} = {:.4}", discriminant, sqrt_d));
        
        let x1 = (-b + sqrt_d) / (2.0 * a);
        let x2 = (-b - sqrt_d) / (2.0 * a);
        
        let mut steps1 = steps.clone();
        steps1.push(format!("x = (-b + √D) / 2a = ({:.4} + {:.4}) / {} = {:.4}", -b, sqrt_d, 2.0 * a, x1));
        
        let mut steps2 = steps;
        steps2.push(format!("x = (-b - √D) / 2a = ({:.4} - {:.4}) / {} = {:.4}", -b, sqrt_d, 2.0 * a, x2));
        
        Some((
            SolvedResult {
                variable: "x".to_string(),
                solution: format!("{:.4}", x1).trim_end_matches('0').trim_end_matches('.').to_string(),
                steps: steps1,
            },
            SolvedResult {
                variable: "x".to_string(),
                solution: format!("{:.4}", x2).trim_end_matches('0').trim_end_matches('.').to_string(),
                steps: steps2,
            },
        ))
    }

    /// Try to solve an equation string like "2x + 3 = 7".
    pub fn solve_equation(&self, input: &str) -> Option<MathResult> {
        let input = input.trim();
        
        // Split on '='
        let parts: Vec<&str> = input.split('=').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let left = parts[0].trim();
        let right = parts[1].trim();
        
        // Try to parse as linear: ax + b = c
        // Simple parser for "ax + b" or "ax - b" or "a"
        let (a, b) = self.parse_linear_coeffs(left);
        let c: f64 = right.parse().ok()?;
        
        // Try linear solve first
        if let Some(result) = self.solve_linear(a, b, c) {
            return Some(MathResult::Solved(result));
        }
        
        // Try quadratic (ax^2 + bx + c = 0 form — parse right as constant, set c = 0)
        // This is limited; full quadratic parsing would need more work
        None
    }

    /// Parse "ax + b" or "ax - b" or "a" into (a, b) coefficients.
    fn parse_linear_coeffs(&self, expr: &str) -> (f64, f64) {
        let expr = expr.trim();
        
        // Handle simple cases
        if expr.is_empty() {
            return (0.0, 0.0);
        }
        
        // Check for x term
        let has_x = expr.contains('x');
        
        if !has_x {
            // Just a constant
            if let Ok(n) = expr.parse::<f64>() {
                return (0.0, n);
            }
            return (0.0, 0.0);
        }
        
        // Extract coefficient of x
        let x_pos = expr.find('x').unwrap();
        let before_x = &expr[..x_pos];
        
        let a = if before_x.is_empty() || before_x == "+" {
            1.0
        } else if before_x == "-" {
            -1.0
        } else {
            before_x.parse().unwrap_or(1.0)
        };
        
        // Extract constant term (after x)
        let after_x = &expr[x_pos + 1..];
        let b = if after_x.is_empty() {
            0.0
        } else if after_x.starts_with("+") {
            after_x[1..].trim().parse().unwrap_or(0.0)
        } else if after_x.starts_with("-") {
            after_x.parse().unwrap_or(0.0)
        } else {
            0.0
        };
        
        (a, b)
    }

    /// Simplify an expression string.
    pub fn simplify(&self, input: &str) -> Option<String> {
        let input = input.trim();
        
        // Try to evaluate numeric expressions
        if let Ok(result) = evaluate_simple(input) {
            return Some(result.to_string());
        }
        
        None
    }

    /// Evaluate a simple numeric expression.
    pub fn evaluate(&self, input: &str) -> Option<String> {
        let input = input.trim();
        if let Ok(result) = evaluate_simple(input) {
            if result.fract() == 0.0 {
                return Some(format!("{}", result as i64));
            }
            return Some(format!("{}", result));
        }
        None
    }
}

/// Evaluate a simple arithmetic expression (no variables).
fn evaluate_simple(expr: &str) -> Result<f64, ()> {
    let expr = expr.replace(" ", "");
    evaluate_add_sub(&expr, &mut 0)
}

fn evaluate_add_sub(expr: &str, pos: &mut usize) -> Result<f64, ()> {
    let mut left = evaluate_mul_div(expr, pos)?;
    
    while *pos < expr.len() {
        let c = expr.chars().nth(*pos);
        match c {
            Some('+') => {
                *pos += 1;
                let right = evaluate_mul_div(expr, pos)?;
                left += right;
            }
            Some('-') => {
                *pos += 1;
                let right = evaluate_mul_div(expr, pos)?;
                left -= right;
            }
            _ => break,
        }
    }
    
    Ok(left)
}

fn evaluate_mul_div(expr: &str, pos: &mut usize) -> Result<f64, ()> {
    let mut left = evaluate_pow(expr, pos)?;
    
    while *pos < expr.len() {
        let c = expr.chars().nth(*pos);
        match c {
            Some('*') => {
                *pos += 1;
                let right = evaluate_pow(expr, pos)?;
                left *= right;
            }
            Some('/') => {
                *pos += 1;
                let right = evaluate_pow(expr, pos)?;
                if right == 0.0 { return Err(()); }
                left /= right;
            }
            _ => break,
        }
    }
    
    Ok(left)
}

fn evaluate_pow(expr: &str, pos: &mut usize) -> Result<f64, ()> {
    let mut left = evaluate_unary(expr, pos)?;
    
    while *pos < expr.len() && expr.chars().nth(*pos) == Some('^') {
        *pos += 1;
        let right = evaluate_unary(expr, pos)?;
        left = left.powf(right);
    }
    
    Ok(left)
}

fn evaluate_unary(expr: &str, pos: &mut usize) -> Result<f64, ()> {
    if *pos < expr.len() && expr.chars().nth(*pos) == Some('-') {
        *pos += 1;
        let val = evaluate_atom(expr, pos)?;
        Ok(-val)
    } else {
        evaluate_atom(expr, pos)
    }
}

fn evaluate_atom(expr: &str, pos: &mut usize) -> Result<f64, ()> {
    if *pos >= expr.len() {
        return Err(());
    }
    
    let c = expr.chars().nth(*pos).unwrap();
    
    // Parentheses
    if c == '(' {
        *pos += 1;
        let val = evaluate_add_sub(expr, pos)?;
        if *pos < expr.len() && expr.chars().nth(*pos) == Some(')') {
            *pos += 1;
        }
        return Ok(val);
    }
    
    // Number
    let start = *pos;
    while *pos < expr.len() {
        let ch = expr.chars().nth(*pos).unwrap();
        if ch.is_digit(10) || ch == '.' {
            *pos += 1;
        } else {
            break;
        }
    }
    
    let num_str = &expr[start..*pos];
    num_str.parse().map_err(|_| ())
}

use super::MathResult;

impl SolvedResult {
    pub fn display(&self) -> String {
        if self.steps.is_empty() {
            format!("x = {}", self.solution)
        } else {
            let steps_str = self.steps.join("\n");
            format!("{}\n\n∴ x = {}", steps_str, self.solution)
        }
    }
}

impl Default for AlgebraEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_linear() {
        let engine = AlgebraEngine::new();
        // 2x + 3 = 7 → x = 2
        let result = engine.solve_linear(2.0, 3.0, 7.0).unwrap();
        assert!(result.solution.contains("2"));
    }

    #[test]
    fn test_parse_linear() {
        let engine = AlgebraEngine::new();
        assert_eq!(engine.parse_linear_coeffs("2x + 3"), (2.0, 3.0));
        assert_eq!(engine.parse_linear_coeffs("x - 5"), (1.0, -5.0));
        assert_eq!(engine.parse_linear_coeffs("-3x"), (-3.0, 0.0));
        assert_eq!(engine.parse_linear_coeffs("7"), (0.0, 7.0));
    }

    #[test]
    fn test_evaluate() {
        let engine = AlgebraEngine::new();
        assert_eq!(engine.evaluate("2 + 2"), Some("4".to_string()));
        assert_eq!(engine.evaluate("10 / 2"), Some("5".to_string()));
    }
}
