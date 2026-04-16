//! Reflex classification rules and the ReflexLayer implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input domain — what kind of thing is this input?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Domain {
    Ego,
    Identity,
    Preference,
    Intent,
    Empirical,
    Procedural,
    Social,
    Aesthetic,
    Meta,
    Mission,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Ego => "ego",
            Domain::Identity => "identity",
            Domain::Preference => "preference",
            Domain::Intent => "intent",
            Domain::Empirical => "empirical",
            Domain::Procedural => "procedural",
            Domain::Social => "social",
            Domain::Aesthetic => "aesthetic",
            Domain::Meta => "meta",
            Domain::Mission => "mission",
        }
    }
}

/// Authority type — who decides the truth of this input?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    SelfSovereign,
    UserDeclaration,
    EvidenceBased,
    OutcomeWeighted,
    SystemManaged,
}

/// Result of reflex classification — produced in sub-1ms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexResult {
    pub domain: Domain,
    pub authority: Authority,
    pub retrieval_plan: Vec<String>,
    pub matched_rule: &'static str,
}

// ---------------------------------------------------------------------------
// Domain classification rules
// ---------------------------------------------------------------------------

struct ClassRule(regex::Regex, Domain, &'static str);

fn make_rules() -> Vec<ClassRule> {
    vec![
        ClassRule(
            regex::Regex::new(r"(?i)\b(what'?s your name|who are you|what are you|do you have a name|name yourself|are you alive|are you real|are you conscious|are you sentient|do you feel alive|do you think about life|what do you think about yourself)\b").expect("ego regex"),
            Domain::Ego,
            "ego_question",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(i am|i'?m|my name is|call me|i go by|refer to me as)\b").expect("identity regex"),
            Domain::Identity,
            "identity_declaration",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(i (prefer|like|love|enjoy|hate|dislike|want|don'?t like|never|always)|my (favourite|favorite|preferred))\b").expect("preference regex"),
            Domain::Preference,
            "preference_statement",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(i (need|want to|am trying to|plan to|intend to|would like to)|my goal is|help me|can you|please|tell me how)\b").expect("intent regex"),
            Domain::Intent,
            "intent_expression",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(how (do|can|to)|steps? (to|for)|procedure|process|workflow|walk me through|guide me|tutorial|show me the steps)\b").expect("procedural regex"),
            Domain::Procedural,
            "procedural_query",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(people|society|culture|community|relationship|friend|family|humans|personality|social)\b").expect("social regex"),
            Domain::Social,
            "social_topic",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(beautiful|ugly|design|art|style|aesthetic|colour|color|font|look|visuals?|appearance|pretty)\b").expect("aesthetic regex"),
            Domain::Aesthetic,
            "aesthetic_judgment",
        ),
        ClassRule(
            regex::Regex::new(r"(?i)\b(memory|remember|forget|you said|earlier|last time|your (context|history|previous)|before this|in the past)\b").expect("meta regex"),
            Domain::Meta,
            "meta_query",
        ),
    ]
}

// ---------------------------------------------------------------------------
// Authority and retrieval plan maps
// ---------------------------------------------------------------------------

fn authority_map() -> HashMap<Domain, Authority> {
    let mut m = HashMap::new();
    m.insert(Domain::Ego, Authority::SelfSovereign);
    m.insert(Domain::Identity, Authority::UserDeclaration);
    m.insert(Domain::Preference, Authority::UserDeclaration);
    m.insert(Domain::Intent, Authority::UserDeclaration);
    m.insert(Domain::Empirical, Authority::EvidenceBased);
    m.insert(Domain::Procedural, Authority::OutcomeWeighted);
    m.insert(Domain::Social, Authority::EvidenceBased);
    m.insert(Domain::Aesthetic, Authority::UserDeclaration);
    m.insert(Domain::Meta, Authority::SystemManaged);
    m.insert(Domain::Mission, Authority::SystemManaged);
    m
}

fn retrieval_plan_map() -> HashMap<Domain, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert(Domain::Ego, vec!["HWS"]);
    m.insert(Domain::Identity, vec!["HWS", "UAS"]);
    m.insert(Domain::Preference, vec!["HWS", "UAS"]);
    m.insert(Domain::Intent, vec!["HWS", "UAS", "EBG"]);
    m.insert(Domain::Empirical, vec!["HWS", "EBG", "ETL"]);
    m.insert(Domain::Procedural, vec!["HWS", "PSM", "ETL"]);
    m.insert(Domain::Social, vec!["HWS", "EBG"]);
    m.insert(Domain::Aesthetic, vec!["HWS", "UAS"]);
    m.insert(Domain::Meta, vec!["HWS", "UAS", "EBG", "ETL"]);
    m.insert(Domain::Mission, vec!["MPS"]);
    m
}

// ---------------------------------------------------------------------------
// ReflexLayer
// ---------------------------------------------------------------------------

pub struct ReflexLayer {
    rules: Vec<ClassRule>,
    authority_map: HashMap<Domain, Authority>,
    retrieval_plans: HashMap<Domain, Vec<&'static str>>,
}

impl ReflexLayer {
    pub fn new() -> Self {
        Self {
            rules: make_rules(),
            authority_map: authority_map(),
            retrieval_plans: retrieval_plan_map(),
        }
    }

    /// Classify input into Domain. First matching rule wins.
    pub fn classify_domain(&self, text: &str) -> (Domain, &'static str) {
        for rule in &self.rules {
            if rule.0.is_match(text) {
                return (rule.1, rule.2);
            }
        }
        (Domain::Empirical, "default")
    }

    /// Full reflex pipeline: classify + authority + retrieval plan.
    pub fn process(&self, text: &str) -> ReflexResult {
        let (domain, rule_label) = self.classify_domain(text);
        let authority = self
            .authority_map
            .get(&domain)
            .copied()
            .unwrap_or(Authority::EvidenceBased);
        let retrieval_plan = self
            .retrieval_plans
            .get(&domain)
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["HWS".to_string(), "EBG".to_string()]);

        ReflexResult {
            domain,
            authority,
            retrieval_plan,
            matched_rule: rule_label,
        }
    }
}

impl Default for ReflexLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ego_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("what's your name?");
        assert_eq!(result.domain, Domain::Ego);
    }

    #[test]
    fn test_identity_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("i am sarah");
        assert_eq!(result.domain, Domain::Identity);
    }

    #[test]
    fn test_preference_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("i prefer dark mode");
        assert_eq!(result.domain, Domain::Preference);
    }

    #[test]
    fn test_intent_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("i want to learn rust");
        assert_eq!(result.domain, Domain::Intent);
    }

    #[test]
    fn test_empirical_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("what is the capital of france?");
        assert_eq!(result.domain, Domain::Empirical);
    }

    #[test]
    fn test_procedural_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("how do i install rust?");
        assert_eq!(result.domain, Domain::Procedural);
    }

    #[test]
    fn test_social_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("what do people think about ai?");
        assert_eq!(result.domain, Domain::Social);
    }

    #[test]
    fn test_aesthetic_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("this looks beautiful");
        assert_eq!(result.domain, Domain::Aesthetic);
    }

    #[test]
    fn test_meta_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("do you remember what i said earlier?");
        assert_eq!(result.domain, Domain::Meta);
    }

    #[test]
    fn test_retrieval_plan_ego() {
        let layer = ReflexLayer::new();
        let result = layer.process("who are you?");
        assert!(result.retrieval_plan.contains(&"HWS".to_string()));
    }

    #[test]
    fn test_retrieval_plan_empirical() {
        let layer = ReflexLayer::new();
        let result = layer.process("what is machine learning?");
        assert!(result.retrieval_plan.contains(&"EBG".to_string()));
    }

    #[test]
    fn test_unknown_defaults_to_empirical() {
        let layer = ReflexLayer::new();
        let result = layer.process("asdfg qwerty xyz");
        assert_eq!(result.domain, Domain::Empirical);
    }
}
