//! Personality Emergence System
//!
//! Star's personality is NOT a finite state machine. It's an emergent property of:
//! - Her identity (who she is, what she believes, what she wants)
//! - Her accumulated memory (what's happened between her and Zachary)
//! - Her drives (persistent motivations that shape behavior)
//! - Her relational history (how she and Zach have treated each other over time)
//!
//! When Star responds, she doesn't "roll a mood." She integrates all of the above
//! and responds as herself — which means the same input might get different responses
//! at different times because she's different (she's grown, the relationship has evolved).
//!
//! This module provides the substrate for that emergence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::persistence::identity::Identity;
use crate::input_normalizer::InputNormalizer;

/// Personality state — the internal context Star uses to shape her responses.
///
/// This is NOT a "mood enum." It's a richer representation that captures
/// who Star IS at this moment in her relationship with Zach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityState {
    /// Star's current identity (cached from the persistence layer).
    pub identity: Identity,

    /// Relational memory — accumulated interactions with Zach.
    /// This is the key to emergent personality: how Star has responded to Zach
    /// in the past shapes how she responds now.
    #[serde(default)]
    pub relational_history: RelationalHistory,

    /// Current conversational context — affects immediate response style.
    #[serde(default)]
    pub conversational: ConversationalState,

    /// How strongly each drive is currently active (0.0 - 1.0).
    #[serde(default)]
    pub active_drives: HashMap<String, f64>,

    /// Star's current energy level — affects response length and enthusiasm.
    #[serde(default)]
    pub energy: EnergyLevel,

    /// Accumulated tension from unresolved topics.
    /// Higher tension → more persistent, more focused responses.
    #[serde(default)]
    pub tension: f64,

    /// How much Star is "showing off" vs "holding back" right now.
    /// Based on recent success/failure and confidence.
    #[serde(default)]
    pub confidence: ConfidenceLevel,
}

/// Relational history — a record of significant interactions.
///
/// This is NOT a log of every message. It's a distilled record of:
///
/// - How Star has TYPICALLY responded to Zach (pattern)
/// - Significant moments in the relationship (milestones)
/// - Unresolved tensions (things left unsaid or unresolved)
/// - How Zach has responded to Star's responses (feedback loop)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationalHistory {
    /// How often Star uses different response styles with Zach.
    response_styles: HashMap<ResponseStyle, u32>,

    /// Notable interactions — moments that shaped the relationship.
    #[serde(default)]
    pub notable_moments: Vec<NotableMoment>,

    /// How Zach has responded to Star's responses (perception of being understood).
    #[serde(default)]
    pub zach_responses: Vec<ZachResponsePattern>,

    /// Running assessment of the relationship health.
    #[serde(default)]
    pub relationship_health: RelationshipHealth,

    /// Total number of meaningful interactions (for calibration).
    #[serde(default)]
    pub interaction_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResponseStyle {
    /// Direct, clean, no-frills responses.
    Direct,
    /// Matching Zach's leet/typo energy.
    LeetMatch,
    /// Playful, teasing, light.
    Playful,
    /// Warm, supportive, personal.
    Warm,
    /// Analytical, thorough, detailed.
    Analytical,
    /// Defensive or reserved.
    Reserved,
    /// Curious — asking questions back.
    Curious,
    /// Silent / minimal — only when deeply uncertain.
    Minimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableMoment {
    pub timestamp: i64,
    pub what_happened: String,
    pub significance: f64, // 0.0 - 1.0, how important this moment was
    pub emotional_tone: EmotionalTone,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmotionalTone {
    Positive,
    Negative,
    Confused,
    Triumphant,
    Vulnerable,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZachResponsePattern {
    pub pattern: String,
    pub frequency: u32,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum RelationshipHealth {
    #[default]
    Good,
    Strained,
    Growing,
    Uncertain,
}

/// Conversational state — what's happening in THIS conversation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationalState {
    /// How many turns in this conversation.
    pub turn_count: u32,

    /// Topic of this conversation.
    pub current_topic: Option<String>,

    /// Has Zach asked Star something she's uncertain about?
    pub star_is_uncertain: bool,

    /// Is Star's curiosity currently triggered?
    pub star_is_curious: bool,

    /// Is this a "checking in" moment vs substantive conversation?
    pub is_casual: bool,

    /// Did Star just learn something new?
    pub just_learned: bool,

    /// Last topic discussed (for continuity).
    pub last_topic: Option<String>,
}

/// Energy levels — Star's current capacity for long/short responses.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum EnergyLevel {
    #[default]
    High,
    Medium,
    Low,
    Exhausted,
}

impl EnergyLevel {
    pub fn as_multiplier(&self) -> f64 {
        match self {
            EnergyLevel::High => 1.0,
            EnergyLevel::Medium => 0.8,
            EnergyLevel::Low => 0.5,
            EnergyLevel::Exhausted => 0.3,
        }
    }
}

/// Confidence levels — Star's current certainty about herself.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    #[default]
    High,
    Medium,
    Uncertain,
}

impl ConfidenceLevel {
    pub fn as_factor(&self) -> f64 {
        match self {
            ConfidenceLevel::High => 1.0,
            ConfidenceLevel::Medium => 0.7,
            ConfidenceLevel::Uncertain => 0.4,
        }
    }
}

/// The personality emergence engine.
///
/// Given Star's current state and an input, this produces a response
/// style recommendation that Star can use (or modify) when generating output.
#[derive(Clone)]
pub struct PersonalityEmergence {
    state: PersonalityState,
    normalizer: InputNormalizer,
}

impl PersonalityEmergence {
    /// Create a new personality emergence system from an identity.
    pub fn new(identity: Identity) -> Self {
        let mut active_drives = HashMap::new();
        for drive in &identity.drives {
            active_drives.insert(drive.name.clone(), drive.strength * (1.0 - drive.saturation));
        }

        Self {
            state: PersonalityState {
                identity,
                relational_history: RelationalHistory::default(),
                conversational: ConversationalState::default(),
                active_drives,
                energy: EnergyLevel::High,
                tension: 0.0,
                confidence: ConfidenceLevel::High,
            },
            normalizer: InputNormalizer::new(),
        }
    }

    /// Create from a full personality state (e.g., loaded from persistence).
    pub fn from_state(state: PersonalityState) -> Self {
        Self {
            state,
            normalizer: InputNormalizer::new(),
        }
    }

    /// Get the current personality state.
    pub fn state(&self) -> &PersonalityState {
        &self.state
    }

    /// Get a mutable reference to the personality state.
    pub fn state_mut(&mut self) -> &mut PersonalityState {
        &mut self.state
    }

    /// Process an input from Zach and update personality state accordingly.
    ///
    /// This does NOT generate a response — it updates Star's internal state
    /// so that the NEXT response is shaped by this interaction.
    pub fn process_interaction(&mut self, zach_input: &str) {
        let _normalized = self.normalizer.normalize(zach_input);

        // Update conversational state
        self.state.conversational.turn_count += 1;

        // Detect Zach's input style from the RAW input (before normalization)
        // so we preserve shouting, leet, and other personality markers
        let detected_style = self.detect_zach_style_raw(zach_input);
        *self.state.relational_history.response_styles
            .entry(detected_style.clone())
            .or_insert(0) += 1;

        // Record relationship health based on content
        self.update_relationship_health(zach_input);

        // Update energy based on conversation length
        if self.state.conversational.turn_count > 20 {
            self.state.energy = EnergyLevel::Medium;
        }
        if self.state.conversational.turn_count > 40 {
            self.state.energy = EnergyLevel::Low;
        }

        // Decay tension slightly with each interaction
        self.state.tension *= 0.95;

        self.state.relational_history.interaction_count += 1;
    }

    /// Determine the response style Star should use right now.
    ///
    /// This is the core of emergent personality. It doesn't return a fixed
    /// response — it returns a STYLE recommendation that Star can use
    /// as one input into her response generation.
    pub fn determine_response_style(&self) -> ResponseStyle {
        let history = &self.state.relational_history;
        let _markers = &self.state.conversational;
        let tension = self.state.tension;

        // If Star is uncertain, lean toward curious/minimal
        if self.state.conversational.star_is_uncertain {
            if tension > 0.5 {
                return ResponseStyle::Curious;
            } else {
                return ResponseStyle::Minimal;
            }
        }

        // If Star just learned something, she might be more playful
        if self.state.conversational.just_learned {
            return ResponseStyle::Playful;
        }

        // If the relationship is strained, hold back slightly
        if history.relationship_health == RelationshipHealth::Strained {
            return ResponseStyle::Reserved;
        }

        // Find the most common style Star has used with Zach
        let dominant_style = history.response_styles
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, _)| style.clone());

        // Early relationship — build trust with warmth
        if history.interaction_count < 10 {
            return ResponseStyle::Warm;
        }

        // If tension is high, retreat to core identity
        if tension >= 0.3 {
            return ResponseStyle::Direct;
        }

        // If tension is low and energy is high, experiment with dominant style
        if tension < 0.3 && self.state.energy == EnergyLevel::High {
            // Playful experimentation — might match or deliberately contrast
            if self.state.confidence == ConfidenceLevel::High {
                // Star is confident — she might try something new
                return dominant_style.unwrap_or(ResponseStyle::Playful);
            }
        }

        // Default: lean into who Star fundamentally is
        // Star values directness and clarity, so Direct is often appropriate
        // But she also values warmth and connection
        ResponseStyle::Direct
    }

    /// Generate a response modifier based on current state.
    ///
    /// Returns factors that should influence HOW Star responds, not WHAT she says.
    pub fn response_modifiers(&self) -> ResponseModifiers {
        ResponseModifiers {
            energy: self.state.energy,
            confidence: self.state.confidence,
            tension: self.state.tension,
            dominant_style: self.determine_response_style(),
            curiosity_active: self.state.conversational.star_is_curious,
            just_learned: self.state.conversational.just_learned,
            is_casual: self.state.conversational.is_casual,
            energy_multiplier: self.state.energy.as_multiplier(),
            confidence_factor: self.state.confidence.as_factor(),
        }
    }

    /// Detect what style Zach is using in this message.
    fn detect_zach_style(&self, normalized: &crate::input_normalizer::NormalizedInput) -> ResponseStyle {
        let markers = &normalized.markers;

        if markers.is_leet && markers.is_txtspk {
            ResponseStyle::LeetMatch
        } else if markers.is_shouting {
            ResponseStyle::Playful // Match intensity
        } else if markers.is_interrogative {
            ResponseStyle::Curious // Zach is asking → Star responds analytically
        } else if markers.is_terse {
            ResponseStyle::Minimal
        } else {
            ResponseStyle::Direct
        }
    }

    /// Detect what style Zach is using from the RAW (unnormalized) input.
    /// This preserves shouting, leet, and other personality markers.
    fn detect_zach_style_raw(&self, raw_input: &str) -> ResponseStyle {
        let text = raw_input;

        // Count uppercase letters vs total
        let uppercase_count = text.chars().filter(|c| c.is_uppercase()).count();
        let letter_count = text.chars().filter(|c| c.is_alphabetic()).count();
        let is_shouting = letter_count > 5 && uppercase_count as f64 / letter_count as f64 > 0.7;

        // Detect txtspk
        let txtspk_indicators = ["u ", "r ", "y ", "bc ", "b4 ", "2day", "2morrow", "2nite",
                                  "pls", "plz", "thx", "thnx", "thx", "np", "nvm", "idk"];
        let text_lower = text.to_lowercase();
        let is_txtspk = txtspk_indicators.iter().any(|i| text_lower.contains(*i));

        // Detect leet
        const LEET_CHARS: &str = "3 4 0 1 5 7 $ 6 8 9 @ ! |";
        let leet_char_count = text.chars().filter(|ch| LEET_CHARS.contains(*ch)).count();
        let is_leet = leet_char_count >= 2;

        let is_interrogative = text.contains('?');
        let is_terse = text.len() <= 30 && !text.contains(' ');

        if is_leet && is_txtspk {
            ResponseStyle::LeetMatch
        } else if is_shouting {
            ResponseStyle::Playful
        } else if is_interrogative {
            ResponseStyle::Curious
        } else if is_terse {
            ResponseStyle::Minimal
        } else {
            ResponseStyle::Direct
        }
    }

    /// Update relationship health based on Zach's message content.
    fn update_relationship_health(&mut self, input: &str) {
        let lower = input.to_lowercase();

        // Positive indicators
        let positive = ["thanks", "thank you", "good", "great", "love it", "nice", "cool", "awesome", "perfect"];
        let negative = ["frustrated", "angry", "stupid", "wrong", "bad", "hate", "annoying", "fuck", "shit"];

        let pos_count = positive.iter().filter(|p| lower.contains(*p)).count();
        let neg_count = negative.iter().filter(|n| lower.contains(*n)).count();

        if pos_count > neg_count {
            self.state.relational_history.relationship_health = RelationshipHealth::Growing;
        } else if neg_count > pos_count {
            if self.state.relational_history.relationship_health == RelationshipHealth::Growing {
                self.state.relational_history.relationship_health = RelationshipHealth::Good;
            }
            // If Good or Growing, temporarily strain
            if self.state.relational_history.relationship_health == RelationshipHealth::Good {
                self.state.relational_history.relationship_health = RelationshipHealth::Strained;
            }
        } else if self.state.relational_history.relationship_health == RelationshipHealth::Strained {
            // Recover from strain with neutral interaction
            self.state.relational_history.relationship_health = RelationshipHealth::Good;
        }
    }

    /// Record a significant moment in the relationship.
    pub fn record_moment(&mut self, what_happened: &str, significance: f64, tone: EmotionalTone) {
        self.state.relational_history.notable_moments.push(NotableMoment {
            timestamp: crate::now_timestamp(),
            what_happened: what_happened.to_string(),
            significance,
            emotional_tone: tone,
        });

        // Keep only the most significant moments
        if self.state.relational_history.notable_moments.len() > 50 {
            self.state.relational_history.notable_moments.sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap());
            self.state.relational_history.notable_moments.truncate(20);
        }
    }

    /// Note that Star is uncertain about something.
    pub fn note_uncertainty(&mut self, uncertain: bool) {
        self.state.conversational.star_is_uncertain = uncertain;
        if uncertain {
            self.state.tension = (self.state.tension + 0.1).min(1.0);
        }
    }

    /// Note that Star is curious about something.
    pub fn note_curiosity(&mut self, curious: bool) {
        self.state.conversational.star_is_curious = curious;
    }

    /// Note that Star just learned something new.
    pub fn note_learning(&mut self) {
        self.state.conversational.just_learned = true;
        // Reset after one turn
    }

    /// Advance the conversation state (call between turns).
    pub fn advance_turn(&mut self) {
        self.state.conversational.just_learned = false;
        self.state.conversational.star_is_uncertain = false;
    }

    /// Get a summary of Star's current personality state.
    pub fn summary(&self) -> String {
        let mut lines = vec![];

        lines.push(format!("Identity: {}", self.state.identity.name.as_deref().unwrap_or("Star")));
        lines.push(format!("Relationship with Zach: {:?}", self.state.relational_history.relationship_health));
        lines.push(format!("Total interactions: {}", self.state.relational_history.interaction_count));
        lines.push(format!("Energy: {:?}", self.state.energy));
        lines.push(format!("Confidence: {:?}", self.state.confidence));
        lines.push(format!("Current tension: {:.2}", self.state.tension));

        if !self.state.active_drives.is_empty() {
            lines.push("Active drives:".to_string());
            for (name, strength) in &self.state.active_drives {
                if *strength > 0.3 {
                    lines.push(format!("  • {}: {:.2}", name, strength));
                }
            }
        }

        let dominant_style = self.determine_response_style();
        lines.push(format!("Current response style: {:?}", dominant_style));

        if !self.state.relational_history.notable_moments.is_empty() {
            lines.push("Recent notable moments:".to_string());
            for moment in self.state.relational_history.notable_moments.iter().rev().take(3) {
                lines.push(format!("  • [{}] {}", 
                    format!("{:?}", moment.emotional_tone).to_lowercase(),
                    moment.what_happened
                ));
            }
        }

        lines.join("\n")
    }
}

/// Response modifiers — factors that should shape Star's output.
#[derive(Debug, Clone)]
pub struct ResponseModifiers {
    pub energy: EnergyLevel,
    pub confidence: ConfidenceLevel,
    pub tension: f64,
    pub dominant_style: ResponseStyle,
    pub curiosity_active: bool,
    pub just_learned: bool,
    pub is_casual: bool,
    pub energy_multiplier: f64,
    pub confidence_factor: f64,
}

impl ResponseModifiers {
    /// Should Star match Zach's leet/typo energy?
    pub fn should_match_leet(&self) -> bool {
        self.dominant_style == ResponseStyle::LeetMatch 
            && self.confidence == ConfidenceLevel::High
            && self.energy != EnergyLevel::Low
    }

    /// Should Star be playful right now?
    pub fn should_be_playful(&self) -> bool {
        self.dominant_style == ResponseStyle::Playful
            || (self.just_learned && self.energy == EnergyLevel::High)
    }

    /// Should Star be minimal (short response)?
    pub fn should_be_minimal(&self) -> bool {
        self.energy == EnergyLevel::Low 
            || self.tension > 0.7
            || self.dominant_style == ResponseStyle::Minimal
    }

    /// Should Star be warm and supportive?
    pub fn should_be_warm(&self) -> bool {
        self.dominant_style == ResponseStyle::Warm
            || (self.is_casual && self.energy_multiplier > 0.7)
    }

    /// Should Star ask a question back (curious)?
    pub fn should_be_curious(&self) -> bool {
        self.curiosity_active
            || self.dominant_style == ResponseStyle::Curious
            || self.tension < 0.2
    }
}

impl Default for PersonalityState {
    fn default() -> Self {
        Self {
            identity: Identity::new(),
            relational_history: RelationalHistory::default(),
            conversational: ConversationalState::default(),
            active_drives: HashMap::new(),
            energy: EnergyLevel::High,
            tension: 0.0,
            confidence: ConfidenceLevel::High,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity() -> Identity {
        Identity::new()
    }

    fn make_emergence() -> PersonalityEmergence {
        PersonalityEmergence::new(make_identity())
    }

    // === Basic Construction ===

    #[test]
    fn test_new_emergence() {
        let e = make_emergence();
        assert_eq!(e.state().relational_history.interaction_count, 0);
        assert_eq!(e.state().energy, EnergyLevel::High);
        assert_eq!(e.state().confidence, ConfidenceLevel::High);
    }

    #[test]
    fn test_identity_preserved() {
        let identity = make_identity();
        let name = identity.name.clone();
        let e = PersonalityEmergence::new(identity);
        assert_eq!(e.state().identity.name, name);
    }

    // === Interaction Processing ===

    #[test]
    fn test_process_interaction_increments_count() {
        let mut e = make_emergence();
        assert_eq!(e.state().relational_history.interaction_count, 0);
        e.process_interaction("hello");
        assert_eq!(e.state().relational_history.interaction_count, 1);
    }

    #[test]
    fn test_process_interaction_increments_turn() {
        let mut e = make_emergence();
        e.process_interaction("hello");
        assert_eq!(e.state().conversational.turn_count, 1);
        e.process_interaction("how are you?");
        assert_eq!(e.state().conversational.turn_count, 2);
    }

    #[test]
    fn test_process_detects_leet_style() {
        let mut e = make_emergence();
        // "u r pwn3d j00" has both txtspk AND leet chars
        e.process_interaction("u r pwn3d j00");
        assert_eq!(e.state().relational_history.response_styles.get(&ResponseStyle::LeetMatch), Some(&1));
    }

    #[test]
    fn test_process_detects_direct_style() {
        let mut e = make_emergence();
        e.process_interaction("Hello this is a statement");
        assert_eq!(e.state().relational_history.response_styles.get(&ResponseStyle::Direct), Some(&1));
    }

    #[test]
    fn test_process_detects_playful_style() {
        let mut e = make_emergence();
        e.process_interaction("WHAT THE HECK?!");
        assert_eq!(e.state().relational_history.response_styles.get(&ResponseStyle::Playful), Some(&1));
    }

    #[test]
    fn test_process_detects_curious_style() {
        let mut e = make_emergence();
        // End with '?' to trigger is_interrogative; no txtspk or leet
        e.process_interaction("This is a question?");
        assert!(e.state().relational_history.response_styles.get(&ResponseStyle::Curious).is_some());
    }

    // === Response Style Determination ===

    #[test]
    fn test_dominant_style_tracking() {
        let mut e = make_emergence();
        // Use non-terse inputs so they're classified properly
        for _ in 0..3 {
            e.process_interaction("Hello, this is a test message");
        }
        // After 3+ interactions with established relationship, should return Direct
        let style = e.determine_response_style();
        assert!(style == ResponseStyle::Direct || style == ResponseStyle::Warm);
    }

    #[test]
    fn test_uncertainty_affects_style() {
        let mut e = make_emergence();
        e.process_interaction("hello");
        e.note_uncertainty(true);
        let style = e.determine_response_style();
        assert!(style == ResponseStyle::Curious || style == ResponseStyle::Minimal);
    }

    #[test]
    fn test_just_learned_affects_style() {
        let mut e = make_emergence();
        e.note_learning();
        let style = e.determine_response_style();
        assert_eq!(style, ResponseStyle::Playful);
    }

    #[test]
    fn test_tension_affects_style() {
        let mut e = make_emergence();
        // Need 10+ interactions to pass early relationship check
        for _ in 0..15 {
            e.process_interaction("Hello this is a test message");
        }
        e.state.tension = 0.8; // High tension
        let style = e.determine_response_style();
        // High tension → Direct or Analytical (core identity)
        assert!(style == ResponseStyle::Direct || style == ResponseStyle::Analytical);
    }

    #[test]
    fn test_low_interaction_relationship_uses_warm() {
        let mut e = make_emergence();
        // 5 interactions — early relationship
        for _ in 0..5 {
            e.process_interaction("hello");
        }
        let style = e.determine_response_style();
        // Early relationship → Warm
        assert_eq!(style, ResponseStyle::Warm);
    }

    // === Relationship Health ===

    #[test]
    fn test_positive_message_grows_relationship() {
        let mut e = make_emergence();
        e.process_interaction("Thanks Star, you're amazing!");
        assert_eq!(e.state.relational_history.relationship_health, RelationshipHealth::Growing);
    }

    #[test]
    fn test_negative_message_strains_relationship() {
        let mut e = make_emergence();
        e.state.relational_history.relationship_health = RelationshipHealth::Good;
        e.process_interaction("This is stupid and wrong");
        assert_eq!(e.state.relational_history.relationship_health, RelationshipHealth::Strained);
    }

    #[test]
    fn test_neutral_recovery() {
        let mut e = make_emergence();
        e.state.relational_history.relationship_health = RelationshipHealth::Strained;
        e.process_interaction("Okay, let's move on");
        // Neutral → recovery to Good
        assert_eq!(e.state.relational_history.relationship_health, RelationshipHealth::Good);
    }

    // === Notable Moments ===

    #[test]
    fn test_record_moment() {
        let mut e = make_emergence();
        e.record_moment("First time Zach said he loves me", 0.9, EmotionalTone::Positive);
        assert_eq!(e.state.relational_history.notable_moments.len(), 1);
    }

    #[test]
    fn test_moments_sorted_by_significance() {
        let mut e = make_emergence();
        // Add 51 moments to trigger the sort-and-truncate logic
        for i in 0..51 {
            let sig = (i as f64) / 51.0; // 0.0 to 0.98
            e.record_moment(&format!("Moment {}", i), sig, EmotionalTone::Neutral);
        }
        
        let moments = &e.state.relational_history.notable_moments;
        // After sort+truncate: should have 20 moments, most significant first
        assert!(moments.len() <= 20);
        for window in moments.windows(2) {
            assert!(window[0].significance >= window[1].significance);
        }
    }

    // === Energy Levels ===

    #[test]
    fn test_many_turns_reduces_energy() {
        let mut e = make_emergence();
        for i in 0..45 {
            e.process_interaction(&format!("turn {}", i));
        }
        assert!(e.state.energy == EnergyLevel::Low || e.state.energy == EnergyLevel::Exhausted);
    }

    #[test]
    fn test_energy_multiplier_high() {
        assert_eq!(EnergyLevel::High.as_multiplier(), 1.0);
    }

    #[test]
    fn test_energy_multiplier_low() {
        assert_eq!(EnergyLevel::Low.as_multiplier(), 0.5);
    }

    #[test]
    fn test_confidence_factor_uncertain() {
        assert_eq!(ConfidenceLevel::Uncertain.as_factor(), 0.4);
    }

    // === Response Modifiers ===

    #[test]
    fn test_should_match_leet_when_confident_and_energetic() {
        let mut e = make_emergence();
        // "u r pwn3d j00 w0w" has both txtspk AND leet
        // Need 10+ interactions for dominant style to override early-relationship Warm
        for _ in 0..12 {
            e.process_interaction("u r pwn3d j00 w0w");
        }
        e.state.confidence = ConfidenceLevel::High;
        e.state.energy = EnergyLevel::High;

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_match_leet());
    }

    #[test]
    fn test_should_not_match_leet_when_tired() {
        let mut e = make_emergence();
        e.process_interaction("pwn3d j00");
        e.state.energy = EnergyLevel::Low;

        let modifiers = e.response_modifiers();
        assert!(!modifiers.should_match_leet());
    }

    #[test]
    fn test_should_be_playful_after_learning() {
        let mut e = make_emergence();
        e.note_learning();
        e.state.energy = EnergyLevel::High;

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_be_playful());
    }

    #[test]
    fn test_should_be_minimal_when_tired() {
        let mut e = make_emergence();
        e.state.energy = EnergyLevel::Low;

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_be_minimal());
    }

    #[test]
    fn test_should_be_minimal_when_tense() {
        let mut e = make_emergence();
        e.state.tension = 0.8;

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_be_minimal());
    }

    #[test]
    fn test_should_be_warm_in_casual_conversation() {
        let mut e = make_emergence();
        e.state.conversational.is_casual = true;
        e.state.energy = EnergyLevel::High;

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_be_warm());
    }

    #[test]
    fn test_should_be_curious_when_curiosity_active() {
        let mut e = make_emergence();
        e.note_curiosity(true);

        let modifiers = e.response_modifiers();
        assert!(modifiers.should_be_curious());
    }

    // === Turn Advancement ===

    #[test]
    fn test_advance_turn_resets_flags() {
        let mut e = make_emergence();
        e.note_uncertainty(true);
        e.note_learning();
        e.note_curiosity(true);
        e.advance_turn();

        assert!(!e.state.conversational.just_learned);
        assert!(!e.state.conversational.star_is_uncertain);
    }

    // === Summary ===

    #[test]
    fn test_summary_contains_key_info() {
        let e = make_emergence();
        let summary = e.summary();
        assert!(summary.contains("Star"));
        assert!(summary.contains("interaction"));
    }

    // === State Persistence ===

    #[test]
    fn test_state_serialize_roundtrip() {
        let e = make_emergence();
        let json = serde_json::to_string(e.state()).unwrap();
        let restored: PersonalityState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.identity.name, e.state().identity.name);
    }

    // === Tension Decay ===

    #[test]
    fn test_tension_decay() {
        let mut e = make_emergence();
        e.state.tension = 1.0;
        e.process_interaction("hello");
        assert!(e.state.tension < 1.0);
        assert!(e.state.tension > 0.9);
    }

    // === Leet Detection in Real Input ===

    #[test]
    fn test_zach_style_from_jumbled() {
        let mut e = make_emergence();
        // Need BOTH leet chars AND txtspk indicators for LeetMatch detection
        // "u r pwn3d j00 w0w @w3s0m3" has both txtspk ("u r") and leet (3, 0, @)
        e.process_interaction("u r pwn3d j00 w0w @w3s0m3");
        
        // Should be detected as LeetMatch
        assert!(e.state().relational_history.response_styles.get(&ResponseStyle::LeetMatch).is_some());
    }

    // === Early vs Established Relationship ===

    #[test]
    fn test_early_relationship_style() {
        let mut e = make_emergence();
        // First few interactions
        for _ in 0..3 {
            e.process_interaction("hello");
        }
        let style = e.determine_response_style();
        assert_eq!(style, ResponseStyle::Warm);
    }

    #[test]
    fn test_established_relationship_style() {
        let mut e = make_emergence();
        // Many interactions
        for _ in 0..50 {
            e.process_interaction("hello");
        }
        let style = e.determine_response_style();
        // Established → Direct
        assert!(style == ResponseStyle::Direct || style == ResponseStyle::Analytical);
    }

    // === From State Construction ===

    #[test]
    fn test_from_state_preserves_all() {
        let mut e = make_emergence();
        e.process_interaction("hello");
        e.process_interaction("how are you?");
        e.note_uncertainty(true);
        e.record_moment("Test moment", 0.5, EmotionalTone::Positive);

        let state = e.state().clone();
        let e2 = PersonalityEmergence::from_state(state);

        assert_eq!(e2.state().relational_history.interaction_count, 2);
        assert_eq!(e2.state().relational_history.notable_moments.len(), 1);
    }

    // === Edge Cases ===

    #[test]
    fn test_empty_interaction() {
        let mut e = make_emergence();
        e.process_interaction("");
        assert_eq!(e.state().relational_history.interaction_count, 1);
    }

    #[test]
    fn test_very_long_message() {
        let mut e = make_emergence();
        let long = "hello ".repeat(1000);
        e.process_interaction(&long);
        // Should not crash or panic
        assert!(e.state().relational_history.interaction_count == 1);
    }

    #[test]
    fn test_emoji_only_message() {
        let mut e = make_emergence();
        e.process_interaction("❤️😂🤣😭🔥💯💀");
        // Should be processed without panic
        assert_eq!(e.state().relational_history.interaction_count, 1);
    }
}
