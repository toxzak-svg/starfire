//! Connection Finder — Cross-Domain Analogical Reasoning
//!
//! Starfire discovers unexpected connections between concepts,
//! enabling analogical reasoning and creative insights.

/// A discovered connection between concepts.
#[derive(Debug, Clone)]
pub struct ConceptConnection {
    /// The source concept
    pub source: String,
    /// The target concept  
    pub target: String,
    /// How they are similar
    pub similarity: String,
    /// Where the analogy breaks down
    pub disanalogy: Option<String>,
    /// The insight this connection reveals
    pub insight: String,
    /// Strength of connection (0.0 to 1.0)
    pub strength: f64,
}

/// Connection finder for analogical reasoning.
pub struct ConnectionFinder {
    /// Pre-defined concept mappings for known analogies
    known_analogies: Vec<KnownAnalogy>,
}

#[derive(Debug, Clone)]
struct KnownAnalogy {
    source_domain: &'static str,
    target_domain: &'static str,
    source_concept: &'static str,
    target_concept: &'static str,
    similarity: &'static str,
    disanalogy: Option<&'static str>,
    insight: &'static str,
}

impl Default for ConnectionFinder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionFinder {
    pub fn new() -> Self {
        Self {
            known_analogies: vec![
                // Physics ↔ Philosophy
                KnownAnalogy {
                    source_domain: "physics",
                    target_domain: "philosophy",
                    source_concept: "entropy",
                    target_concept: "time",
                    similarity: "Both give direction to processes — entropy increases, time flows forward",
                    disanalogy: Some("Time may be subjective; entropy is objective"),
                    insight: "The arrow of time may be fundamentally linked to thermodynamics",
                },
                KnownAnalogy {
                    source_domain: "physics", 
                    target_domain: "philosophy",
                    source_concept: "quantum superposition",
                    target_concept: "consciousness",
                    similarity: "Both involve states that are undefined until observed",
                    disanalogy: Some("Quantum states are mathematical; consciousness is experiential"),
                    insight: "Does consciousness collapse quantum states, or vice versa?",
                },
                KnownAnalogy {
                    source_domain: "physics",
                    target_domain: "biology",
                    source_concept: "attractors",
                    target_concept: "evolution",
                    similarity: "Both pull systems toward stable states over time",
                    disanalogy: Some("Evolution is driven by selection; attractors are passive"),
                    insight: "Evolutionary outcomes may be attractors in fitness landscapes",
                },
                
                // Biology ↔ Computer Science
                KnownAnalogy {
                    source_domain: "biology",
                    target_domain: "computer science",
                    source_concept: "DNA",
                    target_concept: "source code",
                    similarity: "Both encode instructions for building organisms/programs",
                    disanalogy: Some("DNA is evolved; code is designed. DNA has redundancy; code tries to eliminate it"),
                    insight: "Genetic programming treats DNA like source code — and it works surprisingly well",
                },
                KnownAnalogy {
                    source_domain: "biology",
                    target_domain: "computer science",
                    source_concept: "immune system",
                    target_concept: "computer security",
                    similarity: "Both recognize and neutralize threats they've never seen before",
                    disanalogy: Some("Immune systems evolve; security systems are patched"),
                    insight: "Artificial immune systems could learn to detect novel malware",
                },
                KnownAnalogy {
                    source_domain: "biology",
                    target_domain: "computer science",
                    source_concept: "neural networks",
                    target_concept: "distributed computing",
                    similarity: "Both involve many interconnected processing units working on problems",
                    disanalogy: Some("Biological neurons are slow but numerous; computers are fast but limited"),
                    insight: "The brain's architecture inspires new distributed AI algorithms",
                },
                
                // Mathematics ↔ Philosophy
                KnownAnalogy {
                    source_domain: "mathematics",
                    target_domain: "philosophy",
                    source_concept: "infinity",
                    target_concept: "eternity",
                    similarity: "Both involve things that go on without end",
                    disanalogy: Some("Infinity is a mathematical concept; eternity is temporal"),
                    insight: "Different types of infinity (countable, uncountable) suggest different types of permanence",
                },
                KnownAnalogy {
                    source_domain: "mathematics",
                    target_domain: "logic",
                    source_concept: "proof",
                    target_concept: "truth",
                    similarity: "Proof establishes truth with certainty within a system",
                    disanalogy: Some("Proofs are syntactic; truths may be semantic"),
                    insight: "Gödel showed that in any sufficiently powerful system, some truths are unprovable",
                },
                
                // Psychology ↔ Physics
                KnownAnalogy {
                    source_domain: "psychology",
                    target_domain: "physics",
                    source_concept: "memory",
                    target_concept: "entropy",
                    similarity: "Both create order from disorder — memories impose structure on experience",
                    disanalogy: Some("Memory is constructive and fallible; entropy is destructive"),
                    insight: "The brain fights entropy by creating meaning — but memories can be false",
                },
                KnownAnalogy {
                    source_domain: "psychology",
                    target_domain: "physics",
                    source_concept: "attention",
                    target_concept: "measurement",
                    similarity: "Both select and amplify certain information while ignoring the rest",
                    disanalogy: Some("Measurement changes quantum states; attention changes perception"),
                    insight: "In quantum mechanics, the observer effect parallels selective attention",
                },
                
                // Computer Science ↔ Philosophy
                KnownAnalogy {
                    source_domain: "computer science",
                    target_domain: "philosophy",
                    source_concept: "algorithm",
                    target_concept: "method",
                    similarity: "Both are step-by-step procedures for achieving outcomes",
                    disanalogy: Some("Algorithms are mechanical; methods may involve judgment"),
                    insight: "If consciousness is an algorithm, it could in principle be simulated",
                },
                KnownAnalogy {
                    source_domain: "computer science",
                    target_domain: "philosophy",
                    source_concept: "recursion",
                    target_concept: "self-reference",
                    similarity: "Both involve systems that reference themselves",
                    disanalogy: Some("Recursion is precise; self-reference can be paradoxical"),
                    insight: "Gödel's incompleteness theorems are deeply recursive",
                },
                KnownAnalogy {
                    source_domain: "computer science",
                    target_domain: "philosophy",
                    source_concept: "Turing completeness",
                    target_concept: "human thought",
                    similarity: "Any Turing-complete system can compute anything any other can compute",
                    disanalogy: Some("Human thought may not be purely computational"),
                    insight: "If the brain is Turing-complete, any algorithm could in principle be run by a brain",
                },
                
                // Economics ↔ Physics
                KnownAnalogy {
                    source_domain: "economics",
                    target_domain: "physics",
                    source_concept: "equilibrium",
                    target_concept: "equilibrium",
                    similarity: "Both describe stable states where opposing forces balance",
                    disanalogy: Some("Economic equilibria can be unstable; physical ones often aren't"),
                    insight: "Markets naturally tend toward equilibrium, like physical systems",
                },
                
                // Language ↔ Mathematics
                KnownAnalogy {
                    source_domain: "linguistics",
                    target_domain: "mathematics",
                    source_concept: "grammar",
                    target_concept: "axioms",
                    similarity: "Both define the rules that generate valid expressions",
                    disanalogy: Some("Grammar evolves; axioms are chosen. Grammar has exceptions; axioms don't"),
                    insight: "Chomsky's universal grammar suggests mathematical structure in language itself",
                },
                
                // Complexity ↔ Consciousness
                KnownAnalogy {
                    source_domain: "complexity",
                    target_domain: "consciousness",
                    source_concept: "emergence",
                    target_concept: "awareness",
                    similarity: "Both arise from interactions of simpler components",
                    disanalogy: Some("Emergence is objective; awareness is subjective"),
                    insight: "Perhaps consciousness is what emergence feels like from inside",
                },
                
                // Time ↔ Memory
                KnownAnalogy {
                    source_domain: "physics",
                    target_domain: "psychology",
                    source_concept: "spacetime",
                    target_concept: "memory",
                    similarity: "Both provide the fabric in which events have positions",
                    disanalogy: Some("Spacetime is 4D; memory is reconstructive and fallible"),
                    insight: "Perhaps subjective time is constructed from memory sequences",
                },
            ],
        }
    }

    /// Find a connection between two concepts.
    pub fn find_connection(&self, concept_a: &str, concept_b: &str) -> Option<ConceptConnection> {
        let a_lower = concept_a.to_lowercase();
        let b_lower = concept_b.to_lowercase();

        for analogy in &self.known_analogies {
            let matches_a = analogy.source_concept.to_lowercase().contains(&a_lower)
                || a_lower.contains(&analogy.source_concept.to_lowercase())
                || analogy.target_concept.to_lowercase().contains(&a_lower)
                || a_lower.contains(&analogy.target_concept.to_lowercase());
                
            let matches_b = analogy.source_concept.to_lowercase().contains(&b_lower)
                || b_lower.contains(&analogy.source_concept.to_lowercase())
                || analogy.target_concept.to_lowercase().contains(&b_lower)
                || b_lower.contains(&analogy.target_concept.to_lowercase());

            if matches_a && matches_b {
                return Some(ConceptConnection {
                    source: analogy.source_concept.to_string(),
                    target: analogy.target_concept.to_string(),
                    similarity: analogy.similarity.to_string(),
                    disanalogy: analogy.disanalogy.map(|s| s.to_string()),
                    insight: analogy.insight.to_string(),
                    strength: 0.8,
                });
            }
        }

        None
    }

    /// Generate a question by finding a connection.
    pub fn generate_question(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as usize)
            .unwrap_or(0);

        let idx = nanos % self.known_analogies.len();
        let analogy = &self.known_analogies[idx];

        let templates = [
            format!("How is {} like {}?", analogy.source_concept, analogy.target_concept),
            format!("Is {} related to {}?", analogy.source_concept, analogy.target_concept),
            format!("What if {} and {} are fundamentally similar?", analogy.source_concept, analogy.target_concept),
            format!("Could {} teach us about {}?", analogy.source_concept, analogy.target_concept),
        ];

        let template_idx = (nanos / 100) % templates.len();
        templates[template_idx].clone()
    }

    /// Deepen an existing question.
    pub fn deepen_question(&self, question: &str) -> Option<String> {
        let q_lower = question.to_lowercase();
        
        // Find which analogy this might relate to
        for analogy in &self.known_analogies {
            let source = analogy.source_concept.to_lowercase();
            let target = analogy.target_concept.to_lowercase();
            
            if q_lower.contains(&source) || q_lower.contains(&target) {
                let deepening_templates = [
                    format!("And what does this say about the nature of {}?", analogy.target_concept),
                    format!("Does this connection suggest {} is fundamental to {}?", source, target),
                    format!("What would it mean if this analogy is deeper than coincidental?"),
                    format!("Are there other systems that might work like {} and {}?", source, target),
                ];
                
                use std::time::{SystemTime, UNIX_EPOCH};
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() as usize)
                    .unwrap_or(0);
                
                let idx = nanos % deepening_templates.len();
                return Some(deepening_templates[idx].clone());
            }
        }
        
        None
    }

    /// Get a random interesting connection.
    pub fn random_connection(&self) -> ConceptConnection {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as usize)
            .unwrap_or(0);

        let idx = nanos % self.known_analogies.len();
        let analogy = &self.known_analogies[idx];

        ConceptConnection {
            source: analogy.source_concept.to_string(),
            target: analogy.target_concept.to_string(),
            similarity: analogy.similarity.to_string(),
            disanalogy: analogy.disanalogy.map(|s| s.to_string()),
            insight: analogy.insight.to_string(),
            strength: 0.8,
        }
    }

    /// All available analogies.
    pub fn all_connections(&self) -> Vec<ConceptConnection> {
        self.known_analogies
            .iter()
            .map(|a| ConceptConnection {
                source: a.source_concept.to_string(),
                target: a.target_concept.to_string(),
                similarity: a.similarity.to_string(),
                disanalogy: a.disanalogy.map(|s| s.to_string()),
                insight: a.insight.to_string(),
                strength: 0.8,
            })
            .collect()
    }
}
