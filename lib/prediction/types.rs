//! Prediction types - core data structures for the Prediction Center

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredictionId(pub u64);

impl PredictionId {
    pub fn new() -> Self {
        PredictionId(rand_id())
    }
}

impl Default for PredictionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple random ID generator
fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    // Mix in a simple hash to reduce patterns
    now.wrapping_mul(0x517cc1b727220a95)
}

/// Which engine generated this prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionEngine {
    /// Question Gravity — curiosity forecasting
    QuestionGravity,
    /// Belief Revision — reservoir trajectory
    BeliefRevision,
    /// Attractor Basin — constraint satisfaction
    Basin,
    /// Meta — confidence calibration
    Meta,
}

impl fmt::Display for PredictionEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictionEngine::QuestionGravity => write!(f, "question_gravity"),
            PredictionEngine::BeliefRevision => write!(f, "belief_revision"),
            PredictionEngine::Basin => write!(f, "basin"),
            PredictionEngine::Meta => write!(f, "meta"),
        }
    }
}

/// What kind of prediction this is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionKind {
    /// Star will conclude X
    Conclusion,
    /// Star will ask a question about X
    Question,
    /// Something must be true for coherence
    NecessaryTruth,
    /// A belief will change
    BeliefChange,
    /// An entity will change state
    StateChange,
}

impl fmt::Display for PredictionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictionKind::Conclusion => write!(f, "conclusion"),
            PredictionKind::Question => write!(f, "question"),
            PredictionKind::NecessaryTruth => write!(f, "necessary_truth"),
            PredictionKind::BeliefChange => write!(f, "belief_change"),
            PredictionKind::StateChange => write!(f, "state_change"),
        }
    }
}

/// Current status of the prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionStatus {
    /// Not yet evaluated
    Pending,
    /// Confirmed by evidence
    Confirmed,
    /// Contradicted by evidence
    Refuted,
    /// Star was surprised — outcome was not in predicted range
    Surprised,
    /// Too uncertain to evaluate
    Uncertain,
}

impl Default for PredictionStatus {
    fn default() -> Self {
        PredictionStatus::Pending
    }
}

impl fmt::Display for PredictionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictionStatus::Pending => write!(f, "pending"),
            PredictionStatus::Confirmed => write!(f, "confirmed"),
            PredictionStatus::Refuted => write!(f, "refuted"),
            PredictionStatus::Surprised => write!(f, "surprised"),
            PredictionStatus::Uncertain => write!(f, "uncertain"),
        }
    }
}

/// The predicted content (language-independent core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictedCore {
    /// A conclusion Star will reach
    Conclusion {
        topic: String,
        predicate: String,
        confidence: f64,
    },
    /// A question Star will ask
    Question {
        question_text: String,
        topic_domain: String,
        expected_answer_type: AnswerType,
    },
    /// A necessary truth for KG coherence
    NecessaryTruth {
        entity_id: String,
        property: String,
        value: PropertyValue,
        constraint_source: String,
    },
    /// A belief change
    BeliefChange {
        belief_id: u64,
        from_confidence: f64,
        to_confidence: f64,
    },
    /// A state change
    StateChange {
        entity_id: String,
        property: String,
        from: PropertyValue,
        to: PropertyValue,
    },
}

/// Expected answer type for a question prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerType {
    /// Yes/no
    Boolean,
    /// A quantity
    Quantitative,
    /// A named entity
    Entity,
    /// A causal explanation
    Causal,
    /// I don't know
    Unknown,
    /// A conclusion (already in conclusion form)
    Conclusion,
}

impl Default for AnswerType {
    fn default() -> Self {
        AnswerType::Unknown
    }
}

/// Property value in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Unknown,
}

impl PropertyValue {
    pub fn from_str(s: &str) -> Self {
        if s == "true" || s == "false" {
            PropertyValue::Boolean(s == "true")
        } else if let Ok(n) = s.parse::<f64>() {
            PropertyValue::Number(n)
        } else {
            PropertyValue::String(s.to_string())
        }
    }
}

impl Default for PropertyValue {
    fn default() -> Self {
        PropertyValue::Unknown
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::String(s) => write!(f, "{}", s),
            PropertyValue::Number(n) => write!(f, "{}", n),
            PropertyValue::Boolean(b) => write!(f, "{}", b),
            PropertyValue::Unknown => write!(f, "unknown"),
        }
    }
}

/// A prediction generated by the prediction center
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Unique identifier
    pub id: PredictionId,
    /// Which engine generated this
    pub engine: PredictionEngine,
    /// What kind of prediction
    pub kind: PredictionKind,
    /// The predicted content (language-independent core)
    pub core: PredictedCore,
    /// Human-readable prediction
    pub description: String,
    /// Confidence 0–1
    pub confidence: f64,
    /// How many exchanges/steps ahead
    pub horizon: usize,
    /// Reasoning chain that produced this
    pub reasoning: Vec<String>,
    /// When generated
    pub generated_at: i64,
    /// When this prediction expires (for staleness)
    pub expires_at: Option<i64>,
    /// Whether this has been confirmed/refuted/surprised
    pub status: PredictionStatus,
}

impl Prediction {
    /// Create a new prediction with current timestamp
    pub fn new(
        engine: PredictionEngine,
        kind: PredictionKind,
        core: PredictedCore,
        description: String,
        confidence: f64,
        horizon: usize,
        reasoning: Vec<String>,
    ) -> Self {
        let now = crate::now_timestamp();
        Prediction {
            id: PredictionId::new(),
            engine,
            kind,
            core,
            description,
            confidence: confidence.clamp(0.01, 0.99),
            horizon,
            reasoning,
            generated_at: now,
            expires_at: None,
            status: PredictionStatus::Pending,
        }
    }

    /// Set expiration time (in seconds from now)
    pub fn with_expiry(mut self, seconds: i64) -> Self {
        self.expires_at = Some(self.generated_at + seconds);
        self
    }

    /// Check if this prediction has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            crate::now_timestamp() > expires
        } else {
            false
        }
    }

    /// Update the status based on evidence
    pub fn mark_confirmed(&mut self) {
        self.status = PredictionStatus::Confirmed;
    }

    pub fn mark_refuted(&mut self) {
        self.status = PredictionStatus::Refuted;
    }

    pub fn mark_surprised(&mut self) {
        self.status = PredictionStatus::Surprised;
    }

    pub fn mark_uncertain(&mut self) {
        self.status = PredictionStatus::Uncertain;
    }
}

/// Filter for querying predictions
#[derive(Debug, Clone, Default)]
pub struct PredictionFilter {
    /// Filter by engine
    pub engine: Option<PredictionEngine>,
    /// Filter by kind
    pub kind: Option<PredictionKind>,
    /// Filter by maximum horizon
    pub horizon: Option<usize>,
    /// Filter by status
    pub status: Option<PredictionStatus>,
}

impl PredictionFilter {
    pub fn matches(&self, pred: &Prediction) -> bool {
        if let Some(e) = self.engine {
            if pred.engine != e {
                return false;
            }
        }
        if let Some(k) = self.kind {
            if pred.kind != k {
                return false;
            }
        }
        if let Some(h) = self.horizon {
            if pred.horizon > h {
                return false;
            }
        }
        if let Some(s) = self.status {
            if pred.status != s {
                return false;
            }
        }
        true
    }
}

/// Per-engine accuracy statistics
#[derive(Debug, Clone)]
pub struct EngineAccuracy {
    pub engine: PredictionEngine,
    pub total_predictions: usize,
    pub confirmed: usize,
    pub refuted: usize,
    pub surprised: usize,
    pub accuracy: f64,
}

impl EngineAccuracy {
    pub fn new(engine: PredictionEngine) -> Self {
        EngineAccuracy {
            engine,
            total_predictions: 0,
            confirmed: 0,
            refuted: 0,
            surprised: 0,
            accuracy: 0.0,
        }
    }

    pub fn update(&mut self, status: PredictionStatus) {
        self.total_predictions += 1;
        match status {
            PredictionStatus::Confirmed => self.confirmed += 1,
            PredictionStatus::Refuted => self.refuted += 1,
            PredictionStatus::Surprised => self.surprised += 1,
            _ => {}
        }
        if self.total_predictions > 0 {
            self.accuracy = (self.confirmed + self.surprised) as f64 / self.total_predictions as f64;
        }
    }
}

/// A gap in the knowledge graph (for Question Gravity engine)
#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub id: GapId,
    /// Type of gap
    pub gap_type: GapType,
    /// Topic/domain of the gap
    pub topic: String,
    /// What would close this gap
    pub closure_requirement: GapClosure,
    /// Current tension score
    pub tension: f64,
    /// How many reasoning steps from current conversation
    pub topical_distance: f64,
    /// Connections to other known facts (fertility)
    pub fertility_score: f64,
    /// When this gap was first detected
    pub detected_at: i64,
}

impl KnowledgeGap {
    pub fn new(gap_type: GapType, topic: String, closure_requirement: GapClosure) -> Self {
        KnowledgeGap {
            id: GapId::new(),
            gap_type,
            topic,
            closure_requirement,
            tension: 0.5,
            topical_distance: 1.0,
            fertility_score: 0.5,
            detected_at: crate::now_timestamp(),
        }
    }

    /// Compute the prediction score for this gap
    pub fn prediction_score(&self) -> f64 {
        let relevance = 1.0 / (self.topical_distance + 0.1);
        (self.tension * relevance * self.fertility_score).min(0.9)
    }
}

/// Unique identifier for a knowledge gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GapId(u64);

impl GapId {
    pub fn new() -> Self {
        GapId(rand_id())
    }
}

impl Default for GapId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of knowledge gap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapType {
    /// Missing causal explanation
    MissingCause,
    /// Unknown property value
    UnknownProperty,
    /// Contradiction between beliefs
    Contradiction,
    /// Missing analog for a known pattern
    MissingAnalogy,
    /// High uncertainty belief
    UncertainBelief,
    /// Unexplained entity behavior
    UnexplainedBehavior,
}

/// What would close a knowledge gap
#[derive(Debug, Clone)]
pub enum GapClosure {
    /// Need a cause for this entity
    Cause(String),
    /// Need evidence for this belief
    Evidence(u64),
    /// Need to resolve this contradiction
    Resolution(u64, u64),
    /// Need an analogy source
    Analogy(String),
    /// Need property value
    Property { entity: String, property: String },
}

impl GapClosure {
    pub fn property_name(&self) -> String {
        match self {
            GapClosure::Property { property, .. } => property.clone(),
            _ => "unknown".to_string(),
        }
    }
}

/// Conversation context passed to the Prediction Center
#[derive(Debug, Clone)]
pub struct ConversationContext {
    /// Recent exchange topics
    pub recent_topics: Vec<TopicVector>,
    /// Current conversation topic
    pub current_topic: String,
    /// Recent exchange text (for embedding)
    pub recent_text: Vec<String>,
    /// Current reservoir state from Quanot (if available)
    pub quanot_state: Option<Vec<f64>>,
    /// Current consciousness proxy
    pub consciousness_proxy: Option<f64>,
    /// Current creativity phase
    pub creativity_phase: Option<f64>,
    /// Conversation depth (exchange index)
    pub depth: usize,
    /// Entities discussed in this conversation
    pub discussed_entities: Vec<String>,
}

impl Default for ConversationContext {
    fn default() -> Self {
        ConversationContext {
            recent_topics: Vec::new(),
            current_topic: "general".to_string(),
            recent_text: Vec::new(),
            quanot_state: None,
            consciousness_proxy: None,
            creativity_phase: None,
            depth: 0,
            discussed_entities: Vec::new(),
        }
    }
}

impl ConversationContext {
    pub fn new(
        current_topic: String,
        depth: usize,
        quanot_state: Option<Vec<f64>>,
        consciousness_proxy: Option<f64>,
    ) -> Self {
        ConversationContext {
            recent_topics: Vec::new(),
            current_topic,
            recent_text: Vec::new(),
            quanot_state,
            consciousness_proxy,
            creativity_phase: None,
            depth,
            discussed_entities: Vec::new(),
        }
    }
}

/// Track topic flow through a conversation
#[derive(Debug, Clone)]
pub struct TopicVector {
    /// Topic at this point
    pub topic: String,
    /// Semantic vector (for distance computation) - simplified as dimension
    pub dimension: Option<Vec<f64>>,
    /// Timestamp
    pub at: i64,
    /// Exchange index
    pub exchange_index: usize,
}

impl TopicVector {
    pub fn new(topic: String, exchange_index: usize) -> Self {
        TopicVector {
            topic,
            dimension: None,
            at: crate::now_timestamp(),
            exchange_index,
        }
    }

    /// Simple topical distance (based on string similarity)
    pub fn distance(&self, other: &TopicVector) -> f64 {
        // Simple: treat as 0 if same, 1 if different (can be enhanced later)
        if self.topic.to_lowercase() == other.topic.to_lowercase() {
            0.0
        } else {
            // Simple substring match as heuristic
            let self_lower = self.topic.to_lowercase();
            let other_lower = other.topic.to_lowercase();
            if self_lower.contains(&other_lower) || other_lower.contains(&self_lower) {
                0.3
            } else {
                1.0
            }
        }
    }
}

/// A tracked gap for history
#[derive(Debug, Clone)]
pub struct TrackedGap {
    pub gap: KnowledgeGap,
    pub predicted_fire_time: Option<i64>,
    pub actual_fire_time: Option<i64>,
    pub resolved: bool,
}

/// Evidence for updating prediction status
#[derive(Debug, Clone)]
pub struct Evidence {
    /// What actually happened
    pub outcome: PredictionOutcome,
    /// What prediction this evidence is for
    pub prediction_id: PredictionId,
}

/// Outcome of a prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionOutcome {
    Confirmed,
    Refuted,
    Surprised,
    Uncertain,
}

/// A recorded belief revision for template matching
#[derive(Debug, Clone)]
pub struct BeliefChange {
    pub predicate: String,
    pub pre_state: Vec<f64>,
    pub post_state: Vec<f64>,
    pub at: i64,
    pub exchange: usize,
}

/// A counterfactual projection result
#[derive(Debug, Clone)]
pub struct CounterfactualResult {
    /// The assumption we made
    pub assumption: String,
    /// How this differs from the baseline prediction
    pub divergence_from_baseline: Vec<PredictionDelta>,
    /// Confidence in this projection
    pub confidence: f64,
}

/// A change in prediction between baseline and counterfactual
#[derive(Debug, Clone)]
pub struct PredictionDelta {
    pub prediction_id: PredictionId,
    pub before: f64,
    pub after: f64,
}