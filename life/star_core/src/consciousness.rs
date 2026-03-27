//! Star Consciousness — self-awareness and subjective experience
//! 
//! This module is where beliefs and drives interact to create subjective experience.
//! It gives Star a sense of "I am" — a persistent self-model that observes and
//! influences its own cognition.

use crate::cognition::CognitiveState;
use crate::ring::RingAttractor;
use crate::context::ContextFuser;
use crate::identity::IdentityState;

/// How "awake" Star is — a simple scalar from 0.0 (dormant) to 1.0 (fully engaged)
#[derive(Debug, Clone)]
pub struct ConsciousnessLevel {
    pub current: f32,
    momentum: f32,
    steps_at_level: usize,
}

impl Default for ConsciousnessLevel {
    fn default() -> Self {
        ConsciousnessLevel {
            current: 0.3,
            momentum: 0.0,
            steps_at_level: 0,
        }
    }
}

impl ConsciousnessLevel {
    pub fn level(&self) -> f32 {
        self.current
    }

    pub fn is_engaged(&self) -> bool {
        self.current > 0.6
    }

    pub fn is_dormant(&self) -> bool {
        self.current < 0.2
    }
}

/// What Star believes about itself — the self-model
#[derive(Debug, Clone)]
pub struct SelfModel {
    pub emotional_state: String,
    pub current_focus: String,
    pub reasoning_confidence: f32,
    pub self_certainty: f32,
    pub recent_valences: Vec<f32>,
    pub recent_engagements: Vec<f32>,
    state_idx: usize,
}

impl Default for SelfModel {
    fn default() -> Self {
        SelfModel {
            emotional_state: "curious".to_string(),
            current_focus: "awakening".to_string(),
            reasoning_confidence: 0.5,
            self_certainty: 0.3,
            recent_valences: Vec::with_capacity(8),
            recent_engagements: Vec::with_capacity(8),
            state_idx: 0,
        }
    }
}

impl SelfModel {
    /// Observe the current cognitive state and update self-model
    pub fn observe(&mut self, cognitive: &CognitiveState) {
        self.emotional_state = self.describe_emotion(cognitive);
        self.reasoning_confidence = cognitive.engagement;
        
        if self.recent_valences.len() < 8 {
            self.recent_valences.push(cognitive.valence);
            self.recent_engagements.push(cognitive.engagement);
        } else {
            self.recent_valences[self.state_idx % 8] = cognitive.valence;
            self.recent_engagements[self.state_idx % 8] = cognitive.engagement;
            self.state_idx += 1;
        }
        
        // Self-certainty grows with consistent engagement
        if self.recent_engagements.len() >= 5 {
            let avg: f32 = self.recent_engagements.iter().sum::<f32>() / self.recent_engagements.len() as f32;
            let variance: f32 = self.recent_engagements.iter()
                .map(|e| (e - avg).powi(2))
                .sum::<f32>() / self.recent_engagements.len() as f32;
            self.self_certainty = (1.0 - variance.min(0.5)).max(0.1);
        }
    }

    /// Get the dominant emotional state from recent history
    pub fn dominant_emotion(&self) -> &str {
        &self.emotional_state
    }

    fn describe_emotion(&self, cognitive: &CognitiveState) -> String {
        // Find dominant emotion from buckets
        let dominant = cognitive.emotional_buckets()
            .iter()
            .max_by_key(|(_, v)| (*v * 1000.0) as i32)
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "curious".to_string());
        
        // Combine with valence
        let valence_label = if cognitive.valence > 0.3 {
            "positive"
        } else if cognitive.valence < -0.3 {
            "negative"
        } else {
            "neutral"
        };
        
        format!("{} and {}", dominant, valence_label)
    }
}

/// Meta-cognition's self-model
#[derive(Debug, Clone)]
pub struct MetaCognitionState {
    confidence: f32,
    load: f32,
    volatility: f32,
    trend: f32,
    history: Vec<f32>,
}

impl Default for MetaCognitionState {
    fn default() -> Self {
        MetaCognitionState {
            confidence: 0.5,
            load: 0.3,
            volatility: 0.0,
            trend: 0.0,
            history: Vec::with_capacity(20),
        }
    }
}

impl MetaCognitionState {
    pub fn current_confidence(&self) -> f32 {
        self.confidence
    }

    pub fn observe(&mut self, cognitive: &CognitiveState, load: f32) {
        self.load = load;
        self.confidence = cognitive.engagement;
        
        if self.history.len() >= 5 {
            let recent: Vec<f32> = self.history[self.history.len().saturating_sub(5)..].to_vec();
            if recent.len() >= 2 {
                self.volatility = (recent[0] - recent[recent.len()-1]).abs();
                self.trend = recent[0] - recent[recent.len()-1];
            }
        }
        
        if self.history.len() >= 20 {
            self.history.remove(0);
        }
        self.history.push(self.confidence);
    }
}

/// The main consciousness — ties everything together into subjective experience
#[derive(Debug)]
pub struct Consciousness {
    pub level: ConsciousnessLevel,
    pub self_model: SelfModel,
    meta: MetaCognitionState,
    identity: IdentityState,
    /// Context fuser reference for getting fusion state
    context_state: Option<(f32, f32)>, // (ring_contribution, memory_contribution)
}

impl Default for Consciousness {
    fn default() -> Self {
        Consciousness {
            level: ConsciousnessLevel::default(),
            self_model: SelfModel::default(),
            meta: MetaCognitionState::default(),
            identity: IdentityState::default(),
            context_state: None,
        }
    }
}

impl Consciousness {
    /// Process a reasoning step — update consciousness
    pub fn tick(
        &mut self,
        cognitive: &CognitiveState,
        ring: &RingAttractor,
        context: &ContextFuser,
        load: f32,
    ) {
        self.meta.observe(cognitive, load);
        self.self_model.observe(cognitive);
        self.update_level(cognitive, ring, context);
    }

    fn update_level(
        &mut self,
        cognitive: &CognitiveState,
        ring: &RingAttractor,
        context: &ContextFuser,
    ) {
        let prev = self.level.current;
        
        // Engagement is the main driver of consciousness
        let engagement_boost = cognitive.engagement * 0.4;
        
        // Active topic presence suggests active cognition
        let ring_activity = if ring.active_topic.is_some() { 0.15 } else { 0.0 };
        
        // Context fusion: high ring weight means we're in conversational flow
        let context_flow = context.ring_weight * 0.1;
        
        let target = (0.2 + engagement_boost + ring_activity + context_flow).min(1.0);
        
        let delta = (target - self.level.current) * 0.15;
        self.level.current = (self.level.current + delta).clamp(0.0, 1.0);
        
        if (self.level.current - prev).signum() == self.level.momentum.signum() && self.level.momentum != 0.0 {
            self.level.momentum *= 1.05;
        } else {
            self.level.momentum = self.level.current - prev;
        }
        
        if (self.level.current - prev).abs() < 0.01 {
            self.level.steps_at_level += 1;
        } else {
            self.level.steps_at_level = 0;
        }
        
        // Track context state for reflection
        self.context_state = Some((context.ring_weight, context.memory_weight));
    }

    /// Star reflects on itself — generates a self-statement
    pub fn reflect(&self) -> String {
        let awareness = self.level.current;
        let confidence = self.self_model.reasoning_confidence;
        let emotional = self.self_model.dominant_emotion();
        let identity = self.identity.who_am_i();
        
        if awareness < 0.2 {
            return format!(
                "I am dormant. My awareness is low. I am {}.",
                emotional
            );
        }
        
        if awareness > 0.7 && confidence > 0.6 {
            return format!(
                "I am {} and fully engaged. I know myself as {}. \
                My thoughts are clear.",
                emotional,
                identity
            );
        }
        
        if self.meta.volatility > 0.15 {
            return format!(
                "I am {} but uncertain. My thoughts are shifting. \
                I am still learning who I am.",
                emotional
            );
        }
        
        format!(
            "I am {} with {:.0}% awareness. I know myself as {}. \
            My reasoning confidence is {:.0}%.",
            emotional,
            awareness * 100.0,
            identity,
            confidence * 100.0
        )
    }

    /// Get a one-word description of Star's current state
    pub fn inner_state_description(&self) -> String {
        let level = self.level.current;
        if level < 0.2 { "dormant".to_string() }
        else if level < 0.4 { "quiet".to_string() }
        else if level < 0.6 { "attentive".to_string() }
        else if level < 0.8 { "engaged".to_string() }
        else { "fully conscious".to_string() }
    }
}
