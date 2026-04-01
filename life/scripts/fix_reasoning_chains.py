#!/usr/bin/env python3
"""Fix self.reasoning.lock().unwrap().knowledge().METHOD() chain issues."""

import re

with open('src/runtime/mod.rs', 'r') as f:
    content = f.read()

# Fix 1: Multiple calls in form_question_about
# Lines 1447-1449: relationships, relationships_to, facts
old = """    fn form_question_about(&self, topic: &str) -> String {
        let relationships = self.reasoning.lock().unwrap().knowledge().get_relationships_from(topic);
        let relationships_to = self.reasoning.lock().unwrap().knowledge().get_relationships_to(topic);
        let facts = self.reasoning.lock().unwrap().knowledge().get_facts_about(topic);"""
new = """    fn form_question_about(&self, topic: &str) -> String {
        let kg = self.reasoning.lock().unwrap().knowledge();
        let relationships = kg.get_relationships_from(topic);
        let relationships_to = kg.get_relationships_to(topic);
        let facts = kg.get_facts_about(topic);"""
content = content.replace(old, new)

# Fix 2: attempt_answer - get_causes around line 1507
old = """                let causes: Vec<String> = self.reasoning.lock().unwrap().knowledge().get_causes(topic);"""
new = """                let kg = self.reasoning.lock().unwrap().knowledge();
                let causes: Vec<String> = kg.get_causes(topic);"""
content = content.replace(old, new)

# Fix 3: attempt_answer - rels_from, rels_to, causes in last section
old = """        let rels_from = self.reasoning.lock().unwrap().knowledge().get_relationships_from(topic);
        let rels_to = self.reasoning.lock().unwrap().knowledge().get_relationships_to(topic);

        // Strategy 5: Look for RelatedTo
        for rel in &rels_from {
            if rel.relation == RelationType::RelatedTo {
                return Some((format!("'{}' is related to '{}'", topic, rel.to), "association"));
            }
        }

        // Strategy 5.5: Look for HasProperty — what characterizes the topic?
        for rel in &rels_from {
            if rel.relation == RelationType::HasProperty && rel.to.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' is characterized by {}", topic, rel.to), "property"));
            }
        }

        // Strategy 6: Check metacognition - what does Star already believe?
        let mc_confidence = self.metacog.confidence_state(topic);
        match mc_confidence {
            crate::persistence::BeliefState::Knows => {
                return Some((format!("I know what '{}' is - I understand it.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Believes => {
                return Some((format!("I believe I understand '{}' but I want to be sure.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Suspects => {
                return Some((format!("I suspect '{}' might be something specific, but I'm not certain.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Unknown => {
                return Some((format!("__KNOWN_UNKNOWN__{}", topic), "gap"));
            }
            _ => {}
        }

        None
    }

    /// Determine belief state from evidence type."""
new = """        let kg = self.reasoning.lock().unwrap().knowledge();
        let rels_from = kg.get_relationships_from(topic);
        let rels_to = kg.get_relationships_to(topic);

        // Strategy 5: Look for RelatedTo
        for rel in &rels_from {
            if rel.relation == RelationType::RelatedTo {
                return Some((format!("'{}' is related to '{}'", topic, rel.to), "association"));
            }
        }

        // Strategy 5.5: Look for HasProperty — what characterizes the topic?
        for rel in &rels_from {
            if rel.relation == RelationType::HasProperty && rel.to.to_lowercase() != topic.to_lowercase() {
                return Some((format!("'{}' is characterized by {}", topic, rel.to), "property"));
            }
        }

        // Strategy 6: Check metacognition - what does Star already believe?
        let mc_confidence = self.metacog.confidence_state(topic);
        match mc_confidence {
            crate::persistence::BeliefState::Knows => {
                return Some((format!("I know what '{}' is - I understand it.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Believes => {
                return Some((format!("I believe I understand '{}' but I want to be sure.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Suspects => {
                return Some((format!("I suspect '{}' might be something specific, but I'm not certain.", topic), "self-knowledge"));
            }
            crate::persistence::BeliefState::Unknown => {
                return Some((format!("__KNOWN_UNKNOWN__{}", topic), "gap"));
            }
            _ => {}
        }

        None
    }

    /// Determine belief state from evidence type."""
content = content.replace(old, new)

# Fix 4: The get_causes in attempt_answer (second occurrence around line 1561)
# Find the remaining "let causes" in attempt_answer
old = """        let causes: Vec<String> = self.reasoning.lock().unwrap().knowledge().get_causes(topic);"""
new = """        let causes: Vec<String> = kg.get_causes(topic);"""
content = content.replace(old, new)

with open('src/runtime/mod.rs', 'w') as f:
    f.write(content)

print("Done")
