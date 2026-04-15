//! Reflex Layer — Sub-50ms Input Classification and Routing
//!
//! Pure regex domain classifier that runs before LLM/KV-cache work.
//! Classifies input into Domain + Authority + RetrievalPlan with zero I/O.
//!
//! Pattern: Input → Domain Classify → Authority Gate → Retrieval Plan

pub mod rules;

pub use rules::{Authority, Domain, ReflexResult, ReflexLayer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ego_domain() {
        let layer = ReflexLayer::new();
        let result = layer.process("what is your name?");
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
    fn test_self_response_pattern() {
        // "I'm" + capital name → ego or identity, not apology
        let layer = ReflexLayer::new();
        let result = layer.process("I'm Sarah");
        // Should NOT be Ego (would wrongly trigger self-response bypass)
        assert_ne!(result.domain, Domain::Ego);
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
}
