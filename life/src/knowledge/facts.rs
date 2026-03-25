//! Fact Extraction
//!
//! Parses natural language text into structured facts.
//! Uses pattern matching and basic NLP to extract subject-predicate-object triplets.
//!
//! "Fire is hot" → Fact { subject: "fire", predicate: "is", object: "hot" }
//! "Water boils at 100C" → Fact { subject: "water", predicate: "boils at", object: "100C" }
//! "Light travels fast" → Fact { subject: "light", predicate: "travels", object: "fast" }

use super::Fact;
use std::collections::HashSet;

/// Extracts facts from natural language text.
pub struct FactExtractor {
    /// Subjects we've already seen in this text (avoid duplicates)
    seen_subjects: HashSet<String>,
}

impl FactExtractor {
    pub fn new() -> Self {
        Self {
            seen_subjects: HashSet::new(),
        }
    }

    /// Extract all facts from a text.
    pub fn extract(&mut self, text: &str) -> Vec<Fact> {
        self.seen_subjects.clear();
        let mut facts = Vec::new();
        
        // Split into sentences
        let sentences = split_into_sentences(text);
        
        for sentence in sentences {
            if let Some(fact) = self.extract_sentence(&sentence) {
                // Avoid exact duplicates
                let key = fact.to_string().to_lowercase();
                if !self.seen_subjects.contains(&key) {
                    self.seen_subjects.insert(key);
                    facts.push(fact);
                }
            }
        }
        
        facts
    }

    /// Extract a fact from a single sentence.
    fn extract_sentence(&self, sentence: &str) -> Option<Fact> {
        let sentence = clean_sentence(sentence);
        let lower = sentence.to_lowercase();
        
        // Skip questions, commands, and uncertain statements
        if lower.starts_with('?') || lower.starts_with("what is") || lower.starts_with("how do") 
            || lower.starts_with("why do") || lower.starts_with("is ") 
            || lower.contains(" i think ") || lower.contains(" i believe ")
            || lower.contains(" might be ") || lower.contains(" may be ") 
            || lower.contains(" perhaps ") || lower.contains(" possibly ")
            || lower.contains(" i don't know") || lower.contains(" i dont know")
            || lower.contains(" unknown ") {
            return None;
        }
        
        // Pattern: "X is Y" or "X are Y"
        if let Some(fact) = self.match_is_are(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X does Y" or "X can Y" or "X will Y"
        if let Some(fact) = self.match_verb_pattern(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X requires Y" / "X needs Y"
        if let Some(fact) = self.match_requires_needs(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X causes Y"
        if let Some(fact) = self.match_causes(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X consists of Y" / "X contains Y"
        if let Some(fact) = self.match_consists_contains(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X is a type of Y" / "X is an example of Y"
        if let Some(fact) = self.match_type_of(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X happens when Y"
        if let Some(fact) = self.match_happens(&sentence) {
            return Some(fact);
        }
        
        // Pattern: "X uses Y" / "X works with Y"
        if let Some(fact) = self.match_uses(&sentence) {
            return Some(fact);
        }
        
        None
    }

    /// Match "X is Y" or "X are Y" patterns.
    fn match_is_are(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        let (subject_lower, rest) = if lower.starts_with("the ") {
            split_once(&lower, "the ")? // "The fire is hot"
        } else if lower.starts_with("a ") {
            split_once(&lower, "a ")?  // "A fire is hot"
        } else if lower.starts_with("an ") {
            split_once(&lower, "an ")? // "An fire is hot" (grammatically wrong but possible)
        } else {
            split_once(&lower, " is ")? // "Fire is hot"
        };
        
        let subject = capitalize(&title_case(trim_punct(subject_lower)));
        
        // Check for compound predicates
        let predicates = [
            (" is ", "is"),
            (" are ", "are"),
            (" was ", "was"),
            (" were ", "were"),
        ];
        
        for (pattern, pred) in &predicates {
            if let Some((_, after)) = lower.split_once(*pattern) {
                let parts: Vec<&str> = after.split(',').next().unwrap_or(after).split(" but ").next().unwrap_or(after).split(" and ").next().unwrap_or(after).split(" which ").next().unwrap_or(after).split(" that ").next().unwrap_or(after).split(" because ").next().unwrap_or(after).trim().split('(').next().unwrap_or(after).trim().split_whitespace().collect::<Vec<_>>();
                
                if !parts.is_empty() && parts[0].len() > 1 {
                    let obj = parts[1..].join(" ");
                    let obj_clean = trim_punct(&obj);
                    if obj_clean.len() > 1 && obj_clean != subject.to_lowercase() {
                        return Some(Fact {
                            subject,
                            predicate: pred.to_string(),
                            object: Some(capitalize(obj_clean)),
                            confidence: 0.8,
                            source: "text".to_string(),
                        });
                    }
                }
            }
        }
        
        None
    }

    /// Match verb patterns: "X travels", "X burns", "X moves"
    fn match_verb_pattern(&self, sentence: &str) -> Option<Fact> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.len() < 3 {
            return None;
        }
        
        // Look for subject + verb + object
        for i in 1..words.len() - 1 {
            let verb = words[i].trim_matches(|c: char| c.is_ascii_punctuation() || c == 's');
            let verb_past = words[i].to_lowercase();
            
            // Common action verbs
            let action_verbs = [
                "travels", "travel", "burns", "burn", "moves", "move", 
                "requires", "needs", "produces", "produce", "creates", "create",
                "absorbs", "absorb", "reflects", "reflect", "emits", "emit",
                "contains", "happens", "occurs", "causes", "means", "connects",
                "changes", "grows", "dies", "falls", "rises", "combines", "separates",
            ];
            
            let verb_lower = verb_past.to_lowercase();
            if action_verbs.contains(&verb_lower.as_str()) || action_verbs.contains(&verb) {
                let subject = capitalize(trim_punct(words[0]));
                let object = if i + 1 < words.len() {
                    let obj = words[i+1..].join(" ");
                    Some(capitalize(trim_punct(&obj)))
                } else {
                    None
                };
                
                // Skip if subject is too short or is a pronoun
                let skip_pronouns = ["it", "he", "she", "they", "we", "you", "this", "that", "i"];
                if skip_pronouns.contains(&subject.to_lowercase().as_str()) {
                    continue;
                }
                
                return Some(Fact {
                    subject,
                    predicate: verb_lower,
                    object,
                    confidence: 0.75,
                    source: "text".to_string(),
                });
            }
        }
        
        None
    }

    /// Match "X requires Y" or "X needs Y"
    fn match_requires_needs(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" requires ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            Some(Fact {
                subject,
                predicate: "requires".to_string(),
                object: Some(object),
                confidence: 0.85,
                source: "text".to_string(),
            })
        } else if let Some((before, after)) = lower.split_once(" needs ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            Some(Fact {
                subject,
                predicate: "needs".to_string(),
                object: Some(object),
                confidence: 0.85,
                source: "text".to_string(),
            })
        } else {
            None
        }
    }

    /// Match "X causes Y"
    fn match_causes(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" causes ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            Some(Fact {
                subject,
                predicate: "causes".to_string(),
                object: Some(object),
                confidence: 0.8,
                source: "text".to_string(),
            })
        } else if let Some((before, after)) = lower.split_once(" causes ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            Some(Fact {
                subject,
                predicate: "causes".to_string(),
                object: Some(object),
                confidence: 0.8,
                source: "text".to_string(),
            })
        } else {
            None
        }
    }

    /// Match "X consists of Y" / "X contains Y"
    fn match_consists_contains(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" consists of ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            return Some(Fact {
                subject,
                predicate: "consists of".to_string(),
                object: Some(object),
                confidence: 0.85,
                source: "text".to_string(),
            });
        }
        
        if let Some((before, after)) = lower.split_once(" contains ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            return Some(Fact {
                subject,
                predicate: "contains".to_string(),
                object: Some(object),
                confidence: 0.85,
                source: "text".to_string(),
            });
        }
        
        None
    }

    /// Match "X is a type of Y" / "X is an example of Y"
    fn match_type_of(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" is a ") {
            let subject = capitalize(trim_punct(before.trim()));
            let rest = after.trim();
            if rest.starts_with("type of ") {
                let object = capitalize(trim_punct(rest.trim_start_matches("type of ").split('.')
                    .next().unwrap_or(rest).trim()));
                return Some(Fact {
                    subject,
                    predicate: "is a type of".to_string(),
                    object: Some(object),
                    confidence: 0.85,
                    source: "text".to_string(),
                });
            }
        }
        
        if let Some((before, after)) = lower.split_once(" is an ") {
            let subject = capitalize(trim_punct(before.trim()));
            let rest = after.trim();
            if rest.starts_with("example of ") {
                let object = capitalize(trim_punct(rest.trim_start_matches("example of ").split('.')
                    .next().unwrap_or(rest).trim()));
                return Some(Fact {
                    subject,
                    predicate: "is an example of".to_string(),
                    object: Some(object),
                    confidence: 0.85,
                    source: "text".to_string(),
                });
            }
        }
        
        None
    }

    /// Match "X happens when Y"
    fn match_happens(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" happens when ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split('.')
                .next().unwrap_or(after).trim()));
            return Some(Fact {
                subject,
                predicate: "happens when".to_string(),
                object: Some(object),
                confidence: 0.8,
                source: "text".to_string(),
            });
        }
        
        None
    }

    /// Match "X uses Y" / "X works with Y"
    fn match_uses(&self, sentence: &str) -> Option<Fact> {
        let lower = sentence.to_lowercase();
        
        if let Some((before, after)) = lower.split_once(" uses ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            return Some(Fact {
                subject,
                predicate: "uses".to_string(),
                object: Some(object),
                confidence: 0.8,
                source: "text".to_string(),
            });
        }
        
        if let Some((before, after)) = lower.split_once(" works with ") {
            let subject = capitalize(trim_punct(before.trim()));
            let object = capitalize(trim_punct(after.split(',').next().unwrap_or(after).split('.')
                .next().unwrap_or(after).trim()));
            return Some(Fact {
                subject,
                predicate: "works with".to_string(),
                object: Some(object),
                confidence: 0.8,
                source: "text".to_string(),
            });
        }
        
        None
    }
}

impl Default for FactExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Clean a sentence for processing.
fn clean_sentence(s: &str) -> String {
    s.trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .trim_matches('*')
        .to_string()
}

/// Remove leading/trailing punctuation from a string.
fn trim_punct(s: &str) -> &str {
    s.trim().trim_end_matches('.').trim_end_matches(',')
        .trim_end_matches('?').trim_end_matches('!')
        .trim_end_matches(':').trim_end_matches(';')
        .trim()
}

/// Split a string once by a separator.
fn split_once<'a>(s: &'a str, sep: &str) -> Option<(&'a str, &'a str)> {
    let idx = s.find(sep)?;
    Some((&s[..idx], &s[idx + sep.len()..]))
}

/// Title-case a string (capitalize first letter).
fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Capitalize first letter only.
fn capitalize(s: &str) -> String {
    title_case(s)
}

/// Split text into sentences.
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    
    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let s = current.trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
            current.clear();
        }
    }
    
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_is() {
        let mut extractor = FactExtractor::new();
        let facts = extractor.extract("Fire is hot.");
        assert!(!facts.is_empty());
        assert_eq!(facts[0].subject, "Fire");
        assert_eq!(facts[0].predicate, "is");
        assert_eq!(facts[0].object.as_deref(), Some("Hot"));
    }

    #[test]
    fn test_extract_requires() {
        let mut extractor = FactExtractor::new();
        let facts = extractor.extract("Fire requires oxygen to burn.");
        assert!(!facts.is_empty());
        let fire_fact = facts.iter().find(|f| f.subject.to_lowercase() == "fire").cloned();
        assert!(fire_fact.is_some());
    }

    #[test]
    fn test_extract_causes() {
        let mut extractor = FactExtractor::new();
        let facts = extractor.extract("Heat causes water to evaporate.");
        assert!(!facts.is_empty());
    }
}
