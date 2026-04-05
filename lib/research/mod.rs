//! Research Walkabout — Autonomous Knowledge Gathering
//!
//! When Star doesn't know something, she can go on a "research walkabout" to
//! explore the topic, gather data, and integrate it into her beliefs/knowledge.

use crate::persistence::BeliefState;
use crate::reasoning::ReasoningEngine;
use std::sync::{Arc, Mutex};

/// Research walkabout engine — handles autonomous exploration of unknown topics.
pub struct ResearchWalkabout {
    /// Reasoning engine to add discovered knowledge to
    reasoning: Arc<Mutex<ReasoningEngine>>,
    /// Whether a research walkabout is currently in progress
    is_researching: bool,
    /// Topics currently being researched
    active_research: Vec<ResearchTopic>,
    /// Topics that have been researched and integrated
    completed_research: Vec<ResearchTopic>,
}

/// A topic being researched
#[derive(Debug, Clone)]
pub struct ResearchTopic {
    /// Original question/topic that triggered research
    pub original_topic: String,
    /// What we discovered during research
    pub findings: Vec<ResearchFinding>,
    /// When we started researching
    pub started_at: i64,
    /// When research was completed
    pub completed_at: Option<i64>,
    /// Confidence in the findings
    pub confidence: BeliefState,
}

/// A single finding from research
#[derive(Debug, Clone)]
pub struct ResearchFinding {
    /// The discovered fact or insight
    pub content: String,
    /// Source of the finding (if known)
    pub source: Option<String>,
    /// Confidence in this specific finding
    pub confidence: f64,
}

impl ResearchWalkabout {
    /// Create a new research walkabout engine.
    pub fn new(reasoning: Arc<Mutex<ReasoningEngine>>) -> Self {
        Self {
            reasoning,
            is_researching: false,
            active_research: Vec::new(),
            completed_research: Vec::new(),
        }
    }

    /// Check if Star is currently researching.
    pub fn is_researching(&self) -> bool {
        self.is_researching
    }

    /// Start a research walkabout on a topic.
    /// Returns the initial response Star should give before going on the walkabout.
    pub fn start_research(&mut self, topic: &str) -> String {
        self.is_researching = true;
        
        let research_topic = ResearchTopic {
            original_topic: topic.to_string(),
            findings: Vec::new(),
            started_at: crate::now_timestamp(),
            completed_at: None,
            confidence: BeliefState::Unknown,
        };
        
        self.active_research.push(research_topic);
        
        // Generate varied "one moment" responses
        let responses = [
            "One moment — I want to understand this better.".to_string(),
            "Hold on — let me dig into that.".to_string(),
            "One moment. This is worth exploring.".to_string(),
            "Give me a moment — I'm going to research this.".to_string(),
            "Let me look into that.".to_string(),
            "One moment — I need to explore this.".to_string(),
            "I'm not sure yet. Let me research it.".to_string(),
            "I don't know enough about this. One moment — I'm going to explore it.".to_string(),
        ];
        
        let idx = (topic.len() + crate::now_timestamp() as usize) % responses.len();
        responses[idx].clone()
    }

    /// Run the actual research — this is where Star "walks" through the topic.
    /// In this implementation, we reason about the topic and generate plausible
    /// findings based on the knowledge graph and reasoning.
    pub fn conduct_research(&mut self, topic: &str) {
        if let Some(research) = self.active_research.iter_mut().find(|r| r.original_topic == topic) {
            let mut reasoning = match self.reasoning.lock() {
                Ok(r) => r,
                Err(_) => return,
            };
            
            // Try to reason about the topic from different angles
            let angles = [
                format!("What is {}", topic),
                format!("How does {} work", topic),
                format!("Why is {} important", topic),
                format!("What are examples of {}", topic),
            ];
            
            for angle in &angles {
                let result = reasoning.reason(angle, &[]);
                
                if let Some(answer) = &result.answer {
                    // Only add if it's not already a "don't know" response
                    let lower = answer.to_lowercase();
                    if !lower.contains("don't know") && !lower.contains("i don't") && !lower.contains("not sure") {
                        research.findings.push(ResearchFinding {
                            content: answer.clone(),
                            source: None,
                            confidence: result.confidence_score.unwrap_or(0.5),
                        });
                    }
                }
                
                for chain_item in &result.reasoning_chain {
                    let lower = chain_item.to_lowercase();
                    if !lower.contains("don't know") && !lower.contains("i don't") {
                        research.findings.push(ResearchFinding {
                            content: chain_item.clone(),
                            source: None,
                            confidence: 0.4,
                        });
                    }
                }
            }
            
            // Also check what knowledge graph already has
            let kg = reasoning.knowledge();
            if let Some(entity) = kg.get_entity(&topic.to_lowercase()) {
                if let Some(desc) = &entity.description {
                    research.findings.push(ResearchFinding {
                        content: format!("I have some knowledge: {}", desc),
                        source: Some("knowledge graph".to_string()),
                        confidence: 0.6,
                    });
                }
            }
        }
    }

    /// Complete the research walkabout and return what Star learned.
    pub fn complete_research(&mut self, topic: &str) -> Option<ResearchCompletion> {
        self.is_researching = false;
        
        if let Some(pos) = self.active_research.iter().position(|r| r.original_topic == topic) {
            let mut research = self.active_research.remove(pos);
            research.completed_at = Some(crate::now_timestamp());
            
            // Calculate overall confidence based on findings
            let final_confidence = if !research.findings.is_empty() {
                let total_conf: f64 = research.findings.iter().map(|f| f.confidence).sum::<f64>()
                    / research.findings.len() as f64;
                let conf = if total_conf > 0.6 {
                    BeliefState::Thinks
                } else if total_conf > 0.3 {
                    BeliefState::Believes
                } else {
                    BeliefState::Suspects
                };
                
                // Add findings to the knowledge graph
                if let Ok(mut reasoning) = self.reasoning.lock() {
                    for finding in &research.findings {
                        reasoning.add_knowledge(&topic, &finding.content);
                    }
                }
                
                conf
            } else {
                BeliefState::Unknown
            };
            
            let topic_owned = topic.to_string();
            let findings_owned = research.findings.clone();
            
            self.completed_research.push(research);
            
            Some(ResearchCompletion {
                topic: topic_owned,
                findings: findings_owned,
                confidence: final_confidence,
            })
        } else {
            None
        }
    }

    /// Get the most recent research topic if any.
    pub fn last_research_topic(&self) -> Option<&str> {
        self.completed_research.last().map(|r| r.original_topic.as_str())
    }

    /// Get count of completed research walkabouts.
    pub fn completed_count(&self) -> usize {
        self.completed_research.len()
    }
}

/// Result of completing a research walkabout
#[derive(Debug)]
pub struct ResearchCompletion {
    /// The topic that was researched
    pub topic: String,
    /// What was discovered
    pub findings: Vec<ResearchFinding>,
    /// Overall confidence in the findings
    pub confidence: BeliefState,
}

impl ResearchCompletion {
    /// Convert findings to a natural response.
    /// Returns (response_text, curiosity_about_topic)
    pub fn to_response(&self) -> (String, Option<String>) {
        if self.findings.is_empty() {
            return (
                "I explored {} but didn't find clear answers yet.".to_string(),
                Some(self.topic.clone()),
            );
        }

        // Take up to 2 findings to present
        let findings_to_present: Vec<&ResearchFinding> = self.findings.iter().take(2).collect();
        
        let content = if findings_to_present.len() == 1 {
            let f = findings_to_present[0];
            format!("I found something: {}", f.content)
        } else {
            let first = findings_to_present[0].content.clone();
            let second = findings_to_present[1].content.clone();
            format!("I found a couple of things: {} Also, {}.", first, second)
        };

        // Generate curiosity if confidence is low
        let curiosity = if self.confidence == BeliefState::Suspects || self.confidence == BeliefState::Unknown {
            Some(format!("I'd like to understand {} better.", self.topic))
        } else {
            None
        };

        (content, curiosity)
    }
}

impl Clone for ResearchWalkabout {
    fn clone(&self) -> Self {
        Self {
            reasoning: Arc::clone(&self.reasoning),
            is_researching: self.is_researching,
            active_research: self.active_research.clone(),
            completed_research: self.completed_research.clone(),
        }
    }
}