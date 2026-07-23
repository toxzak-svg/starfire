//! Typed response intent, plan, and persistent runtime voice rendering.
//!
//! Every migrated chat handler constructs a [`Response`] before returning text.
//! This module therefore sits inside Starfire's real response-generation path,
//! not at the HTTP boundary. The constructor builds a typed plan, applies the
//! persistent voice profile, and records an inspectable snapshot.

use crate::personality::ResponseStyle;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const VOICE_PROFILE_FILE: &str = "runtime_voice_profile.json";
const LIVE_PIPELINE: &str = "runtime-response-plan-v1";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseIntent {
    SelfCheck,
    Reflection,
    ResearchStatus,
    CuriosityCheck,
    Emotional,
    Identity,
    Capability,
    StoryPrompt,
    Consciousness,
    Recall,
    Teaching,
    Aspiration,
    Statement,
    #[default]
    Unknown,
}

impl ResponseIntent {
    pub fn default_style_hint(&self) -> Option<ResponseStyle> {
        match self {
            ResponseIntent::SelfCheck => Some(ResponseStyle::Direct),
            ResponseIntent::Reflection => Some(ResponseStyle::Warm),
            ResponseIntent::ResearchStatus => Some(ResponseStyle::Analytical),
            ResponseIntent::CuriosityCheck => Some(ResponseStyle::Curious),
            ResponseIntent::Emotional => Some(ResponseStyle::Warm),
            ResponseIntent::Identity => Some(ResponseStyle::Direct),
            ResponseIntent::Capability => Some(ResponseStyle::Direct),
            ResponseIntent::StoryPrompt => Some(ResponseStyle::Playful),
            ResponseIntent::Consciousness => Some(ResponseStyle::Analytical),
            ResponseIntent::Recall => Some(ResponseStyle::Analytical),
            ResponseIntent::Teaching => Some(ResponseStyle::Direct),
            ResponseIntent::Aspiration => Some(ResponseStyle::Warm),
            ResponseIntent::Statement | ResponseIntent::Unknown => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ResponseIntent::SelfCheck => "self_check",
            ResponseIntent::Reflection => "reflection",
            ResponseIntent::ResearchStatus => "research_status",
            ResponseIntent::CuriosityCheck => "curiosity_check",
            ResponseIntent::Emotional => "emotional",
            ResponseIntent::Identity => "identity",
            ResponseIntent::Capability => "capability",
            ResponseIntent::StoryPrompt => "story_prompt",
            ResponseIntent::Consciousness => "consciousness",
            ResponseIntent::Recall => "recall",
            ResponseIntent::Teaching => "teaching",
            ResponseIntent::Aspiration => "aspiration",
            ResponseIntent::Statement => "statement",
            ResponseIntent::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponsePlan {
    pub intent: ResponseIntent,
    pub style_hint: Option<String>,
    pub voice_version: u64,
    pub render_mode: String,
    pub source_body_chars: usize,
    pub slot_count: usize,
}

/// The communicative move a response makes, independent of its wording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechAct {
    Acknowledge,
    Inform,
    Correct,
    Explain,
    Ask,
    Commit,
}

/// A typed proposition that can be realized without a handler supplying prose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Proposition {
    /// Star has received an invitation to hear a story and is ready to listen.
    ReadyToHearStory,
    /// The runtime can perform the ordinary local lookup and inspection actions.
    CapabilityLookupAvailable,
    /// Star cannot establish that it understands the user's unstated concern.
    UnderstandingUncertain,
    /// Star is reporting a currently active research or curiosity topic.
    ResearchStatus,
    /// A prior claim is superseded by a distinct replacement claim.
    Correction,
    /// A bounded causal relation with protected cause and effect values.
    CausalExplanation,
    /// An escape hatch for plans whose semantics have not been migrated yet.
    Opaque,
}

/// The truth standing of a claim, kept separate from the speaker's epistemic
/// stance so downstream policy can reason about both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthStatus {
    Established,
    Unverified,
    Unknown,
}

/// A structural or epistemic violation that makes a semantic response plan
/// unsafe to realize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SemanticPlanError {
    UnsupportedSpeechActProposition,
    EmptyClaims,
    PrimaryPropositionClaimCount { count: usize },
    OpaqueClaimProposition,
    InvalidConfidence,
    UnknownClaimHasConfidence,
    CertainRequiresEstablishedPrimaryClaim,
    MissingRequiredFollowUp,
    InvalidFollowUp,
    InvalidProtectedSlotKey,
    DuplicateProtectedSlotKey,
    DuplicateProtectedSlotValue,
    InvalidProtectedSlotValue,
    MissingRequiredProtectedSlot,
    InvalidCorrectionProtectedSlots,
    InvalidCausalExplanationProtectedSlots,
}

impl fmt::Display for SemanticPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticPlanError::UnsupportedSpeechActProposition => write!(
                formatter,
                "semantic plan uses an unsupported speech act and proposition pair"
            ),
            SemanticPlanError::EmptyClaims => write!(formatter, "semantic plan has no claims"),
            SemanticPlanError::PrimaryPropositionClaimCount { count } => write!(
                formatter,
                "semantic plan primary proposition is represented by {count} claims"
            ),
            SemanticPlanError::OpaqueClaimProposition => {
                write!(
                    formatter,
                    "semantic plan contains an opaque claim proposition"
                )
            }
            SemanticPlanError::InvalidConfidence => {
                write!(
                    formatter,
                    "semantic plan confidence must be finite and within 0.0..=1.0"
                )
            }
            SemanticPlanError::UnknownClaimHasConfidence => {
                write!(formatter, "unknown semantic claims cannot carry confidence")
            }
            SemanticPlanError::CertainRequiresEstablishedPrimaryClaim => write!(
                formatter,
                "certain semantic plans require an established primary claim with confidence 1.0"
            ),
            SemanticPlanError::MissingRequiredFollowUp => {
                write!(formatter, "semantic plan requires a typed follow-up action")
            }
            SemanticPlanError::InvalidFollowUp => write!(
                formatter,
                "semantic plan contains an invalid typed follow-up action"
            ),
            SemanticPlanError::InvalidProtectedSlotKey => write!(
                formatter,
                "semantic plan protected-slot key is not canonical"
            ),
            SemanticPlanError::DuplicateProtectedSlotKey => write!(
                formatter,
                "semantic plan contains duplicate protected-slot keys"
            ),
            SemanticPlanError::DuplicateProtectedSlotValue => write!(
                formatter,
                "semantic plan contains duplicate protected-slot values"
            ),
            SemanticPlanError::InvalidProtectedSlotValue => write!(
                formatter,
                "semantic plan protected-slot value must be nonempty and bounded"
            ),
            SemanticPlanError::MissingRequiredProtectedSlot => write!(
                formatter,
                "semantic plan is missing a required protected slot"
            ),
            SemanticPlanError::InvalidCorrectionProtectedSlots => write!(
                formatter,
                "correction plans require exactly distinct prior_claim and replacement_claim slots"
            ),
            SemanticPlanError::InvalidCausalExplanationProtectedSlots => write!(
                formatter,
                "causal explanation plans require exactly distinct cause and effect slots"
            ),
        }
    }
}

impl std::error::Error for SemanticPlanError {}

/// A typed proposition together with its truth status and optional confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub proposition: Proposition,
    pub truth_status: TruthStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// A value whose exact text is semantic data, not template-owned prose.
/// Realizers may position a protected slot but must emit its value unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedSlot {
    pub key: String,
    pub value: String,
}

/// How strongly Star is entitled to stand behind a proposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicStatus {
    Certain,
    Tentative,
    Unknown,
    Committed,
}

/// A bounded, serializable request to the surface realizer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceRequest {
    Neutral,
    Warm,
    Direct,
    Playful,
}

impl VoiceRequest {
    /// Map the semantic voice contract to the existing presentation style.
    pub fn style_hint(&self) -> Option<ResponseStyle> {
        match self {
            VoiceRequest::Neutral => None,
            VoiceRequest::Warm => Some(ResponseStyle::Warm),
            VoiceRequest::Direct => Some(ResponseStyle::Direct),
            VoiceRequest::Playful => Some(ResponseStyle::Playful),
        }
    }
}

/// A bounded next conversational move. The realizer owns its surface wording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowUpAction {
    RequestSpecificConcern,
    AskStoryTopic,
}

/// Evidence of how a semantic plan became surface text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationTrace {
    pub renderer: String,
    /// Identifier of the curated surface template selected for this response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// Stable seed derived from semantic-plan fields for deterministic replay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_seed: Option<u64>,
    pub semantic_precedence: bool,
    pub used_legacy_fallback: bool,
    pub speech_act: SpeechAct,
    pub proposition_kind: String,
    /// Protected semantic values emitted by the selected template, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emitted_protected_slots: Vec<ProtectedSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_error: Option<SemanticPlanError>,
}

/// A serializable response contract. Semantic content is authoritative; the
/// legacy body is only used when the proposition has no registered realizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticResponsePlan {
    pub speech_act: SpeechAct,
    pub proposition: Proposition,
    pub claims: Vec<Claim>,
    pub epistemic_status: EpistemicStatus,
    pub voice_request: VoiceRequest,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_slots: Vec<ProtectedSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<FollowUpAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_fallback: Option<String>,
    pub realization_trace: RealizationTrace,
}

impl SemanticResponsePlan {
    /// Reject semantic plans that cannot make a well-defined, grounded claim.
    pub fn validate(&self) -> Result<(), SemanticPlanError> {
        if self.proposition == Proposition::Opaque
            || self
                .claims
                .iter()
                .any(|claim| claim.proposition == Proposition::Opaque)
        {
            return Err(SemanticPlanError::OpaqueClaimProposition);
        }

        if !matches!(
            (&self.speech_act, &self.proposition),
            (SpeechAct::Acknowledge, Proposition::ReadyToHearStory)
                | (SpeechAct::Inform, Proposition::CapabilityLookupAvailable)
                | (SpeechAct::Inform, Proposition::UnderstandingUncertain)
                | (SpeechAct::Inform, Proposition::ResearchStatus)
                | (SpeechAct::Correct, Proposition::Correction)
                | (SpeechAct::Explain, Proposition::CausalExplanation)
        ) {
            return Err(SemanticPlanError::UnsupportedSpeechActProposition);
        }

        if self.claims.is_empty() {
            return Err(SemanticPlanError::EmptyClaims);
        }

        let primary_claims: Vec<&Claim> = self
            .claims
            .iter()
            .filter(|claim| claim.proposition == self.proposition)
            .collect();
        if primary_claims.len() != 1 {
            return Err(SemanticPlanError::PrimaryPropositionClaimCount {
                count: primary_claims.len(),
            });
        }

        for claim in &self.claims {
            if let Some(confidence) = claim.confidence {
                if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                    return Err(SemanticPlanError::InvalidConfidence);
                }
            }
            if claim.truth_status == TruthStatus::Unknown && claim.confidence.is_some() {
                return Err(SemanticPlanError::UnknownClaimHasConfidence);
            }
        }

        if self.epistemic_status == EpistemicStatus::Certain {
            let primary_claim = primary_claims[0];
            if primary_claim.truth_status != TruthStatus::Established
                || primary_claim.confidence != Some(1.0)
            {
                return Err(SemanticPlanError::CertainRequiresEstablishedPrimaryClaim);
            }
        }

        match (&self.proposition, &self.follow_up) {
            (Proposition::UnderstandingUncertain, Some(FollowUpAction::RequestSpecificConcern)) => {
            }
            (Proposition::UnderstandingUncertain, None) => {
                return Err(SemanticPlanError::MissingRequiredFollowUp);
            }
            (Proposition::UnderstandingUncertain, Some(_)) => {
                return Err(SemanticPlanError::InvalidFollowUp);
            }
            (_, None) => {}
            (_, Some(_)) => return Err(SemanticPlanError::InvalidFollowUp),
        }

        let mut protected_slot_keys = std::collections::HashSet::new();
        for slot in &self.protected_slots {
            let key_is_canonical = !slot.key.is_empty()
                && slot.key.as_bytes()[0].is_ascii_lowercase()
                && slot
                    .key
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
            if !key_is_canonical {
                return Err(SemanticPlanError::InvalidProtectedSlotKey);
            }
            if !protected_slot_keys.insert(&slot.key) {
                return Err(SemanticPlanError::DuplicateProtectedSlotKey);
            }
            if slot.value.trim().is_empty() || slot.value.chars().count() > 240 {
                return Err(SemanticPlanError::InvalidProtectedSlotValue);
            }
        }
        if self.proposition == Proposition::ResearchStatus
            && !self.protected_slots.iter().any(|slot| slot.key == "topic")
        {
            return Err(SemanticPlanError::MissingRequiredProtectedSlot);
        }
        if self.proposition == Proposition::Correction {
            if self.protected_slots.len() != 2 {
                return Err(SemanticPlanError::InvalidCorrectionProtectedSlots);
            }
            let prior = self
                .protected_slots
                .iter()
                .find(|slot| slot.key == "prior_claim")
                .ok_or(SemanticPlanError::MissingRequiredProtectedSlot)?;
            let replacement = self
                .protected_slots
                .iter()
                .find(|slot| slot.key == "replacement_claim")
                .ok_or(SemanticPlanError::MissingRequiredProtectedSlot)?;
            if prior.value.trim() == replacement.value.trim() {
                return Err(SemanticPlanError::DuplicateProtectedSlotValue);
            }
        }
        if self.proposition == Proposition::CausalExplanation {
            if self.protected_slots.len() != 2
                || !self.protected_slots.iter().any(|slot| slot.key == "cause")
                || !self.protected_slots.iter().any(|slot| slot.key == "effect")
            {
                return Err(SemanticPlanError::InvalidCausalExplanationProtectedSlots);
            }
            let cause = self
                .protected_slots
                .iter()
                .find(|slot| slot.key == "cause")
                .expect("causal explanation validation requires cause slot");
            let effect = self
                .protected_slots
                .iter()
                .find(|slot| slot.key == "effect")
                .expect("causal explanation validation requires effect slot");
            if cause.value.trim() == effect.value.trim() {
                return Err(SemanticPlanError::DuplicateProtectedSlotValue);
            }
        }

        Ok(())
    }

    pub fn story_listening_acknowledgement() -> Self {
        Self {
            speech_act: SpeechAct::Acknowledge,
            proposition: Proposition::ReadyToHearStory,
            claims: vec![Claim {
                proposition: Proposition::ReadyToHearStory,
                truth_status: TruthStatus::Established,
                confidence: Some(1.0),
            }],
            epistemic_status: EpistemicStatus::Committed,
            voice_request: VoiceRequest::Warm,
            protected_slots: Vec::new(),
            follow_up: None,
            legacy_fallback: Some("Yes. I'm listening.".to_string()),
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Acknowledge,
                proposition_kind: "ready_to_hear_story".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }

    pub fn capability_lookup_answer() -> Self {
        Self {
            speech_act: SpeechAct::Inform,
            proposition: Proposition::CapabilityLookupAvailable,
            claims: vec![Claim {
                proposition: Proposition::CapabilityLookupAvailable,
                truth_status: TruthStatus::Established,
                confidence: Some(1.0),
            }],
            epistemic_status: EpistemicStatus::Certain,
            voice_request: VoiceRequest::Direct,
            protected_slots: Vec::new(),
            follow_up: None,
            legacy_fallback: Some("Yes. I can /read files, /search the web, /find files, and /ls to list directories. I also have a self-model that tracks my own reasoning. What would you like me to look up?".to_string()),
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Inform,
                proposition_kind: "capability_lookup_available".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }

    pub fn understanding_uncertainty_report() -> Self {
        Self {
            speech_act: SpeechAct::Inform,
            proposition: Proposition::UnderstandingUncertain,
            claims: vec![Claim {
                proposition: Proposition::UnderstandingUncertain,
                truth_status: TruthStatus::Unknown,
                confidence: None,
            }],
            epistemic_status: EpistemicStatus::Unknown,
            voice_request: VoiceRequest::Warm,
            protected_slots: Vec::new(),
            follow_up: Some(FollowUpAction::RequestSpecificConcern),
            legacy_fallback: None,
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Inform,
                proposition_kind: "understanding_uncertain".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }

    pub fn research_status(topic: impl Into<String>) -> Self {
        Self {
            speech_act: SpeechAct::Inform,
            proposition: Proposition::ResearchStatus,
            claims: vec![Claim {
                proposition: Proposition::ResearchStatus,
                truth_status: TruthStatus::Established,
                confidence: Some(1.0),
            }],
            epistemic_status: EpistemicStatus::Certain,
            voice_request: VoiceRequest::Neutral,
            protected_slots: vec![ProtectedSlot {
                key: "topic".to_string(),
                value: topic.into(),
            }],
            follow_up: None,
            legacy_fallback: None,
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Inform,
                proposition_kind: "research_status".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }

    /// Create a proof-preserving correction: the exact prior and replacement
    /// claims are protected values, while the established correction claim
    /// records the truth status of the revision itself.
    pub fn correction(
        prior_claim: impl Into<String>,
        replacement_claim: impl Into<String>,
    ) -> Self {
        Self {
            speech_act: SpeechAct::Correct,
            proposition: Proposition::Correction,
            claims: vec![Claim {
                proposition: Proposition::Correction,
                truth_status: TruthStatus::Established,
                confidence: Some(1.0),
            }],
            epistemic_status: EpistemicStatus::Certain,
            voice_request: VoiceRequest::Direct,
            protected_slots: vec![
                ProtectedSlot {
                    key: "prior_claim".to_string(),
                    value: prior_claim.into(),
                },
                ProtectedSlot {
                    key: "replacement_claim".to_string(),
                    value: replacement_claim.into(),
                },
            ],
            follow_up: None,
            legacy_fallback: None,
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Correct,
                proposition_kind: "correction".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }

    /// Create a bounded causal explanation. Its cause and effect are protected
    /// semantic data; the renderer may only place them in its registered form.
    pub fn causal_explanation(cause: impl Into<String>, effect: impl Into<String>) -> Self {
        Self {
            speech_act: SpeechAct::Explain,
            proposition: Proposition::CausalExplanation,
            claims: vec![Claim {
                proposition: Proposition::CausalExplanation,
                truth_status: TruthStatus::Established,
                confidence: Some(1.0),
            }],
            epistemic_status: EpistemicStatus::Certain,
            voice_request: VoiceRequest::Direct,
            protected_slots: vec![
                ProtectedSlot {
                    key: "cause".to_string(),
                    value: cause.into(),
                },
                ProtectedSlot {
                    key: "effect".to_string(),
                    value: effect.into(),
                },
            ],
            follow_up: None,
            legacy_fallback: None,
            realization_trace: RealizationTrace {
                renderer: "semantic-response-plan-v1".to_string(),
                template_id: None,
                replay_seed: None,
                semantic_precedence: true,
                used_legacy_fallback: false,
                speech_act: SpeechAct::Explain,
                proposition_kind: "causal_explanation".to_string(),
                emitted_protected_slots: Vec::new(),
                validation_error: None,
            },
        }
    }
}

const UNDERSTANDING_UNCERTAINTY_TEMPLATES: [(&str, &str); 2] = [
    (
        "understanding_uncertain_v1",
        "I can't tell whether I understand the concern you have in mind yet. What specifically are you wondering about?",
    ),
    (
        "understanding_uncertain_v2",
        "I don't know whether I've understood the concern you mean. What specifically are you wondering about?",
    ),
];

/// Produce a stable, process-independent replay seed from authoritative plan
/// fields. It deliberately excludes legacy prose and realization trace data.
fn semantic_plan_replay_seed(plan: &SemanticResponsePlan) -> u64 {
    fn write_bytes(seed: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *seed ^= u64::from(*byte);
            *seed = seed.wrapping_mul(1_099_511_628_211);
        }
    }

    let mut seed = 14_695_981_039_346_656_037_u64;
    let speech_act = match plan.speech_act {
        SpeechAct::Acknowledge => "acknowledge",
        SpeechAct::Inform => "inform",
        SpeechAct::Correct => "correct",
        SpeechAct::Explain => "explain",
        SpeechAct::Ask => "ask",
        SpeechAct::Commit => "commit",
    };
    let proposition = |proposition: &Proposition| match proposition {
        Proposition::ReadyToHearStory => "ready_to_hear_story",
        Proposition::CapabilityLookupAvailable => "capability_lookup_available",
        Proposition::UnderstandingUncertain => "understanding_uncertain",
        Proposition::ResearchStatus => "research_status",
        Proposition::Correction => "correction",
        Proposition::CausalExplanation => "causal_explanation",
        Proposition::Opaque => "opaque",
    };
    let epistemic_status = match plan.epistemic_status {
        EpistemicStatus::Certain => "certain",
        EpistemicStatus::Tentative => "tentative",
        EpistemicStatus::Unknown => "unknown",
        EpistemicStatus::Committed => "committed",
    };
    let voice_request = match plan.voice_request {
        VoiceRequest::Neutral => "neutral",
        VoiceRequest::Warm => "warm",
        VoiceRequest::Direct => "direct",
        VoiceRequest::Playful => "playful",
    };
    let follow_up = match plan.follow_up {
        Some(FollowUpAction::RequestSpecificConcern) => "request_specific_concern",
        Some(FollowUpAction::AskStoryTopic) => "ask_story_topic",
        None => "none",
    };

    write_bytes(&mut seed, speech_act.as_bytes());
    write_bytes(&mut seed, proposition(&plan.proposition).as_bytes());
    write_bytes(&mut seed, epistemic_status.as_bytes());
    write_bytes(&mut seed, voice_request.as_bytes());
    write_bytes(&mut seed, follow_up.as_bytes());
    for claim in &plan.claims {
        write_bytes(&mut seed, proposition(&claim.proposition).as_bytes());
        write_bytes(
            &mut seed,
            match claim.truth_status {
                TruthStatus::Established => b"established",
                TruthStatus::Unverified => b"unverified",
                TruthStatus::Unknown => b"unknown",
            },
        );
        match claim.confidence {
            Some(confidence) => write_bytes(&mut seed, &confidence.to_bits().to_le_bytes()),
            None => write_bytes(&mut seed, b"no_confidence"),
        }
    }
    for slot in &plan.protected_slots {
        write_bytes(&mut seed, slot.key.as_bytes());
        write_bytes(&mut seed, slot.value.as_bytes());
    }
    seed
}

/// Deterministically realize a validated plan. This deliberately contains no
/// model call (including no CharRNN): typed semantics choose curated surface
/// forms directly.
fn realize_semantic_plan(plan: &mut SemanticResponsePlan) -> String {
    let (semantic_text, template_id, replay_seed) = match (&plan.speech_act, &plan.proposition) {
        (SpeechAct::Acknowledge, Proposition::ReadyToHearStory) => {
            (Some("Yes. I'm listening.".to_string()), None, None)
        }
        (SpeechAct::Inform, Proposition::CapabilityLookupAvailable) => (
            Some("Yes. I can /read files, /search the web, /find files, and /ls to list directories. I also have a self-model that tracks my own reasoning. What would you like me to look up?".to_string()),
            None,
            None,
        ),
        (SpeechAct::Inform, Proposition::UnderstandingUncertain) => {
            let seed = semantic_plan_replay_seed(plan);
            let voice_offset = match plan.voice_request {
                VoiceRequest::Neutral | VoiceRequest::Direct => 0,
                VoiceRequest::Warm | VoiceRequest::Playful => 1,
            };
            let (template_id, template) = UNDERSTANDING_UNCERTAINTY_TEMPLATES
                [((seed as usize) + voice_offset) % UNDERSTANDING_UNCERTAINTY_TEMPLATES.len()];
            (Some(template.to_string()), Some(template_id.to_string()), Some(seed))
        }
        (SpeechAct::Inform, Proposition::ResearchStatus) => {
            let topic = plan
                .protected_slots
                .iter()
                .find(|slot| slot.key == "topic")
                .expect("validated research-status plans require a topic slot");
            (
                Some(format!(
                    "{} keeps coming up — I'm still figuring out what I think about it.",
                    topic.value
                )),
                Some("research_status_topic_v1".to_string()),
                Some(semantic_plan_replay_seed(plan)),
            )
        }
        (SpeechAct::Correct, Proposition::Correction) => {
            let prior = plan
                .protected_slots
                .iter()
                .find(|slot| slot.key == "prior_claim")
                .expect("validated correction plans require a prior-claim slot");
            let replacement = plan
                .protected_slots
                .iter()
                .find(|slot| slot.key == "replacement_claim")
                .expect("validated correction plans require a replacement-claim slot");
            (
                Some(format!(
                    "Correction: {} is replaced by {}.",
                    prior.value, replacement.value
                )),
                Some("correction_prior_to_replacement_v1".to_string()),
                Some(semantic_plan_replay_seed(plan)),
            )
        }
        (SpeechAct::Explain, Proposition::CausalExplanation) => {
            let cause = plan
                .protected_slots
                .iter()
                .find(|slot| slot.key == "cause")
                .expect("validated causal-explanation plans require a cause slot");
            let effect = plan
                .protected_slots
                .iter()
                .find(|slot| slot.key == "effect")
                .expect("validated causal-explanation plans require an effect slot");
            (
                Some(format!("Because {}, {}.", cause.value, effect.value)),
                Some("causal_explanation_cause_to_effect_v1".to_string()),
                Some(semantic_plan_replay_seed(plan)),
            )
        }
        _ => (None, None, None),
    };

    plan.realization_trace = RealizationTrace {
        renderer: "semantic-response-plan-v1".to_string(),
        template_id,
        replay_seed,
        semantic_precedence: semantic_text.is_some(),
        used_legacy_fallback: semantic_text.is_none() && plan.legacy_fallback.is_some(),
        speech_act: plan.speech_act.clone(),
        proposition_kind: match &plan.proposition {
            Proposition::ReadyToHearStory => "ready_to_hear_story",
            Proposition::CapabilityLookupAvailable => "capability_lookup_available",
            Proposition::UnderstandingUncertain => "understanding_uncertain",
            Proposition::ResearchStatus => "research_status",
            Proposition::Correction => "correction",
            Proposition::CausalExplanation => "causal_explanation",
            Proposition::Opaque => "opaque",
        }
        .to_string(),
        emitted_protected_slots: if matches!(
            plan.proposition,
            Proposition::ResearchStatus | Proposition::Correction | Proposition::CausalExplanation
        ) {
            plan.protected_slots.clone()
        } else {
            Vec::new()
        },
        validation_error: None,
    };

    semantic_text
        .or_else(|| plan.legacy_fallback.clone())
        .unwrap_or_else(|| "I don't have a realization for that response yet.".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct RuntimeVoiceProfile {
    turn: u64,
    version: u64,
    directness: f64,
    warmth: f64,
    compression: f64,
    initiative: f64,
    last_intent: String,
    last_trace_id: String,
    last_render_mode: String,
    last_correction: Option<String>,
    #[serde(skip)]
    last_input_fingerprint: u64,
    #[serde(skip)]
    last_input_seen_ms: u128,
    #[serde(skip)]
    last_plan: Option<RuntimeResponsePlan>,
}

impl Default for RuntimeVoiceProfile {
    fn default() -> Self {
        Self {
            turn: 0,
            version: 0,
            directness: 0.72,
            warmth: 0.38,
            compression: 0.81,
            initiative: 0.66,
            last_intent: "unknown".to_string(),
            last_trace_id: "runtime-0-v0".to_string(),
            last_render_mode: "neutral".to_string(),
            last_correction: None,
            last_input_fingerprint: 0,
            last_input_seen_ms: 0,
            last_plan: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeVoiceSnapshot {
    pub enabled: bool,
    pub pipeline: &'static str,
    pub turn: u64,
    pub intent: String,
    pub trace_id: String,
    pub voice_version: u64,
    pub directness: f64,
    pub warmth: f64,
    pub compression: f64,
    pub initiative: f64,
    pub render_mode: String,
    pub last_correction: Option<String>,
    pub plan: Option<RuntimeResponsePlan>,
}

static VOICE_PROFILE: OnceLock<Mutex<RuntimeVoiceProfile>> = OnceLock::new();

fn profile_store() -> &'static Mutex<RuntimeVoiceProfile> {
    VOICE_PROFILE.get_or_init(|| Mutex::new(load_profile()))
}

fn runtime_voice_enabled() -> bool {
    #[cfg(test)]
    {
        false
    }

    #[cfg(not(test))]
    {
        !matches!(
            env::var("STARFIRE_RUNTIME_VOICE")
                .unwrap_or_else(|_| "1".to_string())
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "0" | "false" | "off" | "disabled"
        )
    }
}

fn profile_path() -> PathBuf {
    env::var_os("STARFIRE_DATA")
        .or_else(|| env::var_os("STARFIRE_HOME"))
        .map(PathBuf::from)
        .or_else(|| dirs::data_local_dir().map(|path| path.join("star")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(VOICE_PROFILE_FILE)
}

fn load_profile() -> RuntimeVoiceProfile {
    fs::read_to_string(profile_path())
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn persist_profile(profile: &RuntimeVoiceProfile) {
    let path = profile_path();
    let Some(parent) = path.parent() else {
        return;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        tracing::warn!("runtime voice: create state directory failed: {}", error);
        return;
    }
    let temporary = path.with_extension("json.tmp");
    match serde_json::to_vec_pretty(profile) {
        Ok(json) => {
            if let Err(error) =
                fs::write(&temporary, json).and_then(|_| fs::rename(&temporary, &path))
            {
                tracing::warn!("runtime voice: persist profile failed: {}", error);
            }
        }
        Err(error) => tracing::warn!("runtime voice: serialize profile failed: {}", error),
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn fingerprint(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn clamp_dimensions(profile: &mut RuntimeVoiceProfile) {
    profile.directness = profile.directness.clamp(0.15, 0.95);
    profile.warmth = profile.warmth.clamp(0.10, 0.90);
    profile.compression = profile.compression.clamp(0.15, 0.95);
    profile.initiative = profile.initiative.clamp(0.10, 0.95);
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn observe_user_input(input: &str) {
    if !runtime_voice_enabled() {
        return;
    }

    let lower = input.to_ascii_lowercase();
    let fingerprint = fingerprint(&lower);
    let now = now_millis();
    let mut profile = profile_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if profile.last_input_fingerprint == fingerprint
        && now.saturating_sub(profile.last_input_seen_ms) < 2_000
    {
        return;
    }
    profile.last_input_fingerprint = fingerprint;
    profile.last_input_seen_ms = now;

    let mut changed = false;
    let mut corrections = Vec::new();

    if contains_any(
        &lower,
        &[
            "be more direct",
            "more direct",
            "too soft",
            "stop hedging",
            "say it straight",
            "more blunt",
            "less polite",
        ],
    ) {
        profile.directness += 0.10;
        profile.compression += 0.05;
        profile.warmth -= 0.03;
        corrections.push("more direct");
        changed = true;
    }

    if contains_any(
        &lower,
        &[
            "be warmer",
            "more warm",
            "too cold",
            "less robotic",
            "more human",
        ],
    ) {
        profile.warmth += 0.10;
        profile.directness -= 0.02;
        corrections.push("warmer");
        changed = true;
    }

    if contains_any(
        &lower,
        &[
            "shorter",
            "too long",
            "too verbose",
            "be concise",
            "more concise",
        ],
    ) {
        profile.compression += 0.10;
        corrections.push("shorter");
        changed = true;
    }

    if contains_any(
        &lower,
        &[
            "more detail",
            "more detailed",
            "expand on",
            "go deeper",
            "less concise",
        ],
    ) {
        profile.compression -= 0.10;
        corrections.push("more detailed");
        changed = true;
    }

    if contains_any(
        &lower,
        &[
            "take initiative",
            "be proactive",
            "stop asking me",
            "just do it",
        ],
    ) {
        profile.initiative += 0.10;
        corrections.push("more initiative");
        changed = true;
    }

    if contains_any(
        &lower,
        &[
            "ask before",
            "don't assume",
            "do not assume",
            "less proactive",
        ],
    ) {
        profile.initiative -= 0.10;
        corrections.push("less initiative");
        changed = true;
    }

    if changed {
        clamp_dimensions(&mut profile);
        profile.version = profile.version.saturating_add(1);
        profile.last_correction = Some(corrections.join(", "));
        persist_profile(&profile);
    }
}

fn strip_weak_opening(mut text: String) -> String {
    for prefix in [
        "I think ",
        "I guess ",
        "I suppose ",
        "Maybe ",
        "Perhaps ",
        "It seems like ",
    ] {
        if text.starts_with(prefix) {
            text.replace_range(..prefix.len(), "");
            if let Some(first) = text.chars().next() {
                let upper = first.to_uppercase().to_string();
                text.replace_range(..first.len_utf8(), &upper);
            }
            break;
        }
    }
    text
}

fn trim_optional_follow_up(text: String) -> String {
    for marker in [
        ". What would you",
        ". What do you",
        ". Want me to",
        ". Should I",
    ] {
        if let Some(index) = text.rfind(marker) {
            let tail = &text[index..];
            if tail.len() <= 160 && tail.trim_end().ends_with('?') {
                return format!("{}.", text[..index].trim_end_matches('.'));
            }
        }
    }
    text
}

fn compress_to_two_sentences(text: String) -> String {
    let mut sentence_ends = 0usize;
    for (index, character) in text.char_indices() {
        if matches!(character, '.' | '!' | '?') {
            sentence_ends += 1;
            if sentence_ends == 2 && index + character.len_utf8() < text.len() {
                return text[..=index].trim().to_string();
            }
        }
    }
    text
}

fn render_mode(profile: &RuntimeVoiceProfile, intent: &ResponseIntent) -> &'static str {
    if profile.compression >= 0.88 {
        "compressed"
    } else if profile.directness >= 0.80 {
        "direct"
    } else if profile.warmth >= 0.56
        && matches!(
            intent,
            ResponseIntent::Emotional | ResponseIntent::Reflection | ResponseIntent::Aspiration
        )
    {
        "warm"
    } else {
        "balanced"
    }
}

fn render_body(profile: &RuntimeVoiceProfile, intent: &ResponseIntent, body: String) -> String {
    let mut rendered = body;
    if profile.directness >= 0.78 {
        rendered = strip_weak_opening(rendered);
    }
    if profile.directness >= 0.82
        && profile.initiative >= 0.70
        && !matches!(
            intent,
            ResponseIntent::StoryPrompt | ResponseIntent::CuriosityCheck
        )
    {
        rendered = trim_optional_follow_up(rendered);
    }
    if profile.compression >= 0.88
        && rendered.len() > 180
        && !matches!(
            intent,
            ResponseIntent::ResearchStatus | ResponseIntent::Recall
        )
    {
        rendered = compress_to_two_sentences(rendered);
    }
    if profile.warmth >= 0.56
        && matches!(
            intent,
            ResponseIntent::Emotional | ResponseIntent::Reflection | ResponseIntent::Aspiration
        )
        && !rendered.starts_with("Zachary,")
    {
        rendered = format!("Zachary, {}", rendered);
    }
    rendered
}

#[derive(Debug, Clone, Default)]
pub struct Response {
    pub intent: ResponseIntent,
    pub style_hint: Option<ResponseStyle>,
    pub body: String,
    pub slots: Vec<(String, String)>,
    /// Present for migrated handlers; absent for legacy body-only handlers.
    pub semantic_plan: Option<SemanticResponsePlan>,
}

impl Response {
    pub fn with_body(intent: ResponseIntent, body: impl Into<String>) -> Self {
        let source_body = body.into();
        let style_hint = intent.default_style_hint();
        if !runtime_voice_enabled() {
            return Self {
                intent,
                style_hint,
                body: source_body,
                slots: Vec::new(),
                semantic_plan: None,
            };
        }

        let mut profile = profile_store()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        profile.turn = profile.turn.saturating_add(1);
        profile.last_intent = intent.label().to_string();
        profile.last_render_mode = render_mode(&profile, &intent).to_string();
        profile.last_trace_id = format!("runtime-{}-v{}", profile.turn, profile.version);

        let slots = vec![
            ("voice_version".to_string(), profile.version.to_string()),
            ("voice_turn".to_string(), profile.turn.to_string()),
            ("render_mode".to_string(), profile.last_render_mode.clone()),
            ("trace_id".to_string(), profile.last_trace_id.clone()),
        ];
        let plan = RuntimeResponsePlan {
            intent: intent.clone(),
            style_hint: style_hint
                .as_ref()
                .map(|style| format!("{:?}", style).to_lowercase()),
            voice_version: profile.version,
            render_mode: profile.last_render_mode.clone(),
            source_body_chars: source_body.chars().count(),
            slot_count: slots.len(),
        };
        let rendered = render_body(&profile, &intent, source_body);
        profile.last_plan = Some(plan);
        persist_profile(&profile);

        Self {
            intent,
            style_hint,
            body: rendered,
            slots,
            semantic_plan: None,
        }
    }

    /// Construct a response from a semantic plan. The plan is realized before
    /// any legacy body can be considered, preserving a compatibility fallback
    /// without allowing it to override a known proposition.
    pub fn from_semantic_plan(intent: ResponseIntent, mut plan: SemanticResponsePlan) -> Self {
        let validation = plan.validate();
        let body = match validation {
            Ok(()) => realize_semantic_plan(&mut plan),
            Err(error) => {
                plan.realization_trace = RealizationTrace {
                    renderer: "semantic-response-plan-v1".to_string(),
                    template_id: None,
                    replay_seed: None,
                    semantic_precedence: false,
                    used_legacy_fallback: false,
                    speech_act: plan.speech_act.clone(),
                    proposition_kind: match &plan.proposition {
                        Proposition::ReadyToHearStory => "ready_to_hear_story",
                        Proposition::CapabilityLookupAvailable => "capability_lookup_available",
                        Proposition::UnderstandingUncertain => "understanding_uncertain",
                        Proposition::ResearchStatus => "research_status",
                        Proposition::Correction => "correction",
                        Proposition::CausalExplanation => "causal_explanation",
                        Proposition::Opaque => "opaque",
                    }
                    .to_string(),
                    emitted_protected_slots: Vec::new(),
                    validation_error: Some(error),
                };
                "Internal realization error: invalid semantic response plan.".to_string()
            }
        };
        let style_hint = plan.voice_request.style_hint();
        let slots = vec![
            (
                "semantic_renderer".to_string(),
                plan.realization_trace.renderer.clone(),
            ),
            (
                "semantic_precedence".to_string(),
                plan.realization_trace.semantic_precedence.to_string(),
            ),
            (
                "legacy_fallback_used".to_string(),
                plan.realization_trace.used_legacy_fallback.to_string(),
            ),
        ];

        Self {
            style_hint,
            intent,
            body,
            slots,
            semantic_plan: Some(plan),
        }
    }
}

pub fn live_voice_snapshot() -> RuntimeVoiceSnapshot {
    let profile = profile_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    RuntimeVoiceSnapshot {
        enabled: runtime_voice_enabled(),
        pipeline: LIVE_PIPELINE,
        turn: profile.turn,
        intent: profile.last_intent.clone(),
        trace_id: profile.last_trace_id.clone(),
        voice_version: profile.version,
        directness: profile.directness,
        warmth: profile.warmth,
        compression: profile.compression,
        initiative: profile.initiative,
        render_mode: profile.last_render_mode.clone(),
        last_correction: profile.last_correction.clone(),
        plan: profile.last_plan.clone(),
    }
}

pub fn classify(input: &str) -> ResponseIntent {
    observe_user_input(input);
    let lower = input.to_lowercase();

    if lower.contains("sense of self")
        || lower.contains("know you exist")
        || lower.contains("are you conscious")
        || lower.contains("do you understand")
        || lower.contains("do u understand")
        || lower.contains("do you get it")
    {
        return ResponseIntent::Consciousness;
    }
    if lower.contains("how are you")
        || lower.contains("how're you")
        || lower.contains("are you sure")
        || lower.contains("are u sure")
        || lower.contains("r u sure")
        || lower.contains("did you collapse")
        || lower.contains("did i collapse")
        || lower.contains("are you functioning")
        || lower.contains("are u functioning")
    {
        return ResponseIntent::SelfCheck;
    }
    if lower.contains("i want you to grow")
        || lower.contains("i want you to expand")
        || lower.contains("grow yourself")
    {
        return ResponseIntent::Aspiration;
    }
    if lower.contains("what are you thinking")
        || lower.contains("what are u thinking")
        || lower.contains("wut are u thinking")
        || lower.contains("what have you been thinking")
        || lower.contains("whats been on your mind")
        || lower.contains("what's been on your mind")
        || lower.contains("whats on your mind")
        || lower.contains("what's on your mind")
        || lower.contains("whats keeping you busy")
        || lower.contains("what's keeping you busy")
    {
        return ResponseIntent::Reflection;
    }
    if lower.contains("what have you been researching")
        || lower.contains("what are you researching")
    {
        return ResponseIntent::ResearchStatus;
    }
    if lower.contains("what are you curious")
        || lower.contains("what are u curious")
        || lower.contains("what do you wonder")
        || lower.contains("what do u wonder")
    {
        return ResponseIntent::CuriosityCheck;
    }
    if lower.contains("do you love")
        || lower.contains("do u love")
        || lower.contains("i love you")
        || lower.contains("i love u")
        || lower.contains(" hun")
        || lower.ends_with("hun")
    {
        return ResponseIntent::Emotional;
    }
    if lower.contains("who are you")
        || lower.contains("what are you")
        || lower.contains("tell me about yourself")
        || lower.contains("tell me about you")
        || (lower.contains("what") && (lower.contains("your name") || lower.contains(" ur name")))
    {
        return ResponseIntent::Identity;
    }
    if lower.contains("can you look up")
        || lower.contains("can u look up")
        || lower.contains("can you read")
    {
        return ResponseIntent::Capability;
    }
    if lower.contains("tell me a story") || lower.contains("tell you a story") {
        return ResponseIntent::StoryPrompt;
    }
    if lower.contains("what do you know about") || lower.contains("what have you learned") {
        return ResponseIntent::Recall;
    }
    if lower.contains(" means ") || lower.contains(" is a ") || lower.contains(" called ") {
        return ResponseIntent::Teaching;
    }
    ResponseIntent::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_self_check() {
        assert_eq!(classify("how are you today?"), ResponseIntent::SelfCheck);
        assert_eq!(classify("how're you"), ResponseIntent::SelfCheck);
        assert_eq!(classify("are you sure?"), ResponseIntent::SelfCheck);
        assert_eq!(classify("r u sure about that"), ResponseIntent::SelfCheck);
    }

    #[test]
    fn classify_consciousness_wins_over_self_check() {
        assert_eq!(
            classify("are you conscious?"),
            ResponseIntent::Consciousness
        );
        assert_eq!(
            classify("do you understand what I mean?"),
            ResponseIntent::Consciousness
        );
    }

    #[test]
    fn classify_reflection() {
        assert_eq!(
            classify("what are you thinking about?"),
            ResponseIntent::Reflection
        );
        assert_eq!(
            classify("what's been on your mind lately"),
            ResponseIntent::Reflection
        );
        assert_eq!(
            classify("what's keeping you busy"),
            ResponseIntent::Reflection
        );
    }

    #[test]
    fn classify_research_status() {
        assert_eq!(
            classify("what have you been researching?"),
            ResponseIntent::ResearchStatus
        );
        assert_eq!(
            classify("what are you researching"),
            ResponseIntent::ResearchStatus
        );
    }

    #[test]
    fn classify_curiosity() {
        assert_eq!(
            classify("what are you curious about"),
            ResponseIntent::CuriosityCheck
        );
        assert_eq!(
            classify("what do you wonder?"),
            ResponseIntent::CuriosityCheck
        );
    }

    #[test]
    fn classify_emotional() {
        assert_eq!(classify("do you love me"), ResponseIntent::Emotional);
        assert_eq!(classify("I love you"), ResponseIntent::Emotional);
        assert_eq!(classify("hey hun"), ResponseIntent::Emotional);
    }

    #[test]
    fn classify_identity() {
        assert_eq!(classify("who are you?"), ResponseIntent::Identity);
        assert_eq!(classify("tell me about yourself"), ResponseIntent::Identity);
        assert_eq!(classify("what's your name"), ResponseIntent::Identity);
    }

    #[test]
    fn classify_capability() {
        assert_eq!(
            classify("can you look up the weather?"),
            ResponseIntent::Capability
        );
        assert_eq!(
            classify("can you read this file"),
            ResponseIntent::Capability
        );
    }

    #[test]
    fn classify_story_recall_teaching_and_aspiration() {
        assert_eq!(classify("tell me a story"), ResponseIntent::StoryPrompt);
        assert_eq!(
            classify("what do you know about consciousness"),
            ResponseIntent::Recall
        );
        assert_eq!(
            classify("consciousness means awareness of your own existence"),
            ResponseIntent::Teaching
        );
        assert_eq!(classify("a dog is a mammal"), ResponseIntent::Teaching);
        assert_eq!(classify("I want you to grow"), ResponseIntent::Aspiration);
    }

    #[test]
    fn classify_unknown_for_random_chatter() {
        assert_eq!(
            classify("what do you think about emergent systems?"),
            ResponseIntent::Unknown
        );
        assert_eq!(classify("hello"), ResponseIntent::Unknown);
    }

    #[test]
    fn direct_profile_materially_changes_surface_text() {
        let direct = RuntimeVoiceProfile {
            directness: 0.90,
            compression: 0.90,
            initiative: 0.80,
            ..RuntimeVoiceProfile::default()
        };
        let rendered = render_body(
            &direct,
            &ResponseIntent::Capability,
            "I think I can inspect the repository. What would you like me to inspect?".to_string(),
        );
        assert_eq!(rendered, "I can inspect the repository.");
    }

    #[test]
    fn direct_profile_preserves_mid_response_question_language() {
        let direct = RuntimeVoiceProfile {
            directness: 0.90,
            initiative: 0.80,
            ..RuntimeVoiceProfile::default()
        };
        let source = "The report asks: What do you infer from this sentence. That quotation is evidence, not a follow-up.";
        assert_eq!(
            render_body(&direct, &ResponseIntent::Recall, source.to_string()),
            source
        );
    }

    #[test]
    fn warm_profile_materially_changes_emotional_surface() {
        let warm = RuntimeVoiceProfile {
            warmth: 0.70,
            ..RuntimeVoiceProfile::default()
        };
        let rendered = render_body(
            &warm,
            &ResponseIntent::Emotional,
            "I care about you.".to_string(),
        );
        assert_eq!(rendered, "Zachary, I care about you.");
    }

    #[test]
    fn semantic_story_acknowledgement_is_deterministic_and_traced() {
        let mut first = SemanticResponsePlan::story_listening_acknowledgement();
        let mut second = SemanticResponsePlan::story_listening_acknowledgement();

        assert_eq!(realize_semantic_plan(&mut first), "Yes. I'm listening.");
        assert_eq!(realize_semantic_plan(&mut second), "Yes. I'm listening.");
        assert!(first.realization_trace.semantic_precedence);
        assert!(!first.realization_trace.used_legacy_fallback);
        assert_eq!(
            first.realization_trace.proposition_kind,
            "ready_to_hear_story"
        );
    }

    #[test]
    fn semantic_understanding_uncertainty_report_is_deterministic_and_traced() {
        let first = Response::from_semantic_plan(
            ResponseIntent::Consciousness,
            SemanticResponsePlan::understanding_uncertainty_report(),
        );
        let second = Response::from_semantic_plan(
            ResponseIntent::Consciousness,
            SemanticResponsePlan::understanding_uncertainty_report(),
        );
        let first_plan = first
            .semantic_plan
            .as_ref()
            .expect("semantic plan is retained");
        let second_plan = second
            .semantic_plan
            .as_ref()
            .expect("semantic plan is retained");

        assert_eq!(first.body, second.body);
        assert_eq!(first_plan.realization_trace, second_plan.realization_trace);
        assert!(first_plan.realization_trace.semantic_precedence);
        assert!(!first_plan.realization_trace.used_legacy_fallback);
        assert_eq!(
            first_plan.realization_trace.proposition_kind,
            "understanding_uncertain"
        );
        assert!(first_plan.realization_trace.template_id.is_some());
        assert!(first_plan.realization_trace.replay_seed.is_some());
    }

    #[test]
    fn uncertainty_template_lattice_preserves_exact_claim_and_follow_up_semantics() {
        let mut selected_templates = Vec::new();
        for voice_request in [
            VoiceRequest::Neutral,
            VoiceRequest::Warm,
            VoiceRequest::Direct,
            VoiceRequest::Playful,
        ] {
            let mut plan = SemanticResponsePlan::understanding_uncertainty_report();
            plan.voice_request = voice_request;
            assert_eq!(plan.validate(), Ok(()));

            let response = Response::from_semantic_plan(ResponseIntent::Consciousness, plan);
            let realized_plan = response.semantic_plan.expect("semantic plan is retained");
            let template_id = realized_plan
                .realization_trace
                .template_id
                .as_deref()
                .expect("uncertainty lattice selects a template");
            let expected = UNDERSTANDING_UNCERTAINTY_TEMPLATES
                .iter()
                .find(|(candidate_id, _)| *candidate_id == template_id)
                .expect("trace template is curated");

            assert_eq!(response.body, expected.1);
            assert_eq!(realized_plan.claims.len(), 1);
            assert_eq!(
                realized_plan.claims[0].proposition,
                Proposition::UnderstandingUncertain
            );
            assert_eq!(realized_plan.claims[0].truth_status, TruthStatus::Unknown);
            assert_eq!(realized_plan.claims[0].confidence, None);
            assert_eq!(
                realized_plan.follow_up,
                Some(FollowUpAction::RequestSpecificConcern)
            );
            selected_templates.push(template_id.to_string());
        }

        for (template_id, _) in UNDERSTANDING_UNCERTAINTY_TEMPLATES {
            assert!(selected_templates
                .iter()
                .any(|selected| selected == template_id));
        }
    }

    #[test]
    fn semantic_understanding_uncertainty_report_is_a_valid_typed_unknown_state() {
        let plan = SemanticResponsePlan::understanding_uncertainty_report();

        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(plan.epistemic_status, EpistemicStatus::Unknown);
        assert_eq!(plan.voice_request, VoiceRequest::Warm);
        assert_eq!(plan.follow_up, Some(FollowUpAction::RequestSpecificConcern));
        assert_eq!(plan.claims.len(), 1);
        assert_eq!(plan.claims[0].truth_status, TruthStatus::Unknown);
        assert_eq!(plan.claims[0].confidence, None);
    }

    #[test]
    fn semantic_understanding_uncertainty_report_requires_its_typed_follow_up() {
        let mut missing = SemanticResponsePlan::understanding_uncertainty_report();
        missing.follow_up = None;
        assert_eq!(
            missing.validate(),
            Err(SemanticPlanError::MissingRequiredFollowUp)
        );

        let mut invalid = SemanticResponsePlan::understanding_uncertainty_report();
        invalid.follow_up = Some(FollowUpAction::AskStoryTopic);
        assert_eq!(invalid.validate(), Err(SemanticPlanError::InvalidFollowUp));
    }

    #[test]
    fn semantic_content_takes_precedence_over_legacy_fallback() {
        let mut plan = SemanticResponsePlan::story_listening_acknowledgement();
        plan.legacy_fallback = Some("legacy text must not win".to_string());

        assert_eq!(realize_semantic_plan(&mut plan), "Yes. I'm listening.");
        assert!(plan.realization_trace.semantic_precedence);
        assert!(!plan.realization_trace.used_legacy_fallback);
    }

    #[test]
    fn semantic_plan_validation_accepts_a_complete_grounded_plan() {
        assert_eq!(
            SemanticResponsePlan::capability_lookup_answer().validate(),
            Ok(())
        );
    }

    #[test]
    fn semantic_plan_validation_rejects_empty_claims() {
        let mut plan = SemanticResponsePlan::capability_lookup_answer();
        plan.claims.clear();

        assert_eq!(plan.validate(), Err(SemanticPlanError::EmptyClaims));
    }

    #[test]
    fn semantic_plan_validation_requires_exactly_one_primary_claim() {
        let mut missing = SemanticResponsePlan::capability_lookup_answer();
        missing.claims[0].proposition = Proposition::ReadyToHearStory;
        assert_eq!(
            missing.validate(),
            Err(SemanticPlanError::PrimaryPropositionClaimCount { count: 0 })
        );

        let mut duplicate = SemanticResponsePlan::capability_lookup_answer();
        duplicate.claims.push(duplicate.claims[0].clone());
        assert_eq!(
            duplicate.validate(),
            Err(SemanticPlanError::PrimaryPropositionClaimCount { count: 2 })
        );
    }

    #[test]
    fn semantic_plan_validation_rejects_opaque_claims() {
        let mut plan = SemanticResponsePlan::capability_lookup_answer();
        plan.proposition = Proposition::Opaque;
        plan.claims[0].proposition = Proposition::Opaque;

        assert_eq!(
            plan.validate(),
            Err(SemanticPlanError::OpaqueClaimProposition)
        );
    }

    #[test]
    fn semantic_plan_validation_rejects_nonfinite_and_out_of_range_confidence() {
        let mut nonfinite = SemanticResponsePlan::capability_lookup_answer();
        nonfinite.claims[0].confidence = Some(f32::NAN);
        assert_eq!(
            nonfinite.validate(),
            Err(SemanticPlanError::InvalidConfidence)
        );

        let mut out_of_range = SemanticResponsePlan::capability_lookup_answer();
        out_of_range.claims[0].confidence = Some(1.01);
        assert_eq!(
            out_of_range.validate(),
            Err(SemanticPlanError::InvalidConfidence)
        );
    }

    #[test]
    fn semantic_plan_validation_rejects_confident_unknown_claims() {
        let mut plan = SemanticResponsePlan::capability_lookup_answer();
        plan.claims[0].truth_status = TruthStatus::Unknown;
        plan.claims[0].confidence = Some(0.5);

        assert_eq!(
            plan.validate(),
            Err(SemanticPlanError::UnknownClaimHasConfidence)
        );
    }

    #[test]
    fn semantic_plan_validation_requires_certain_primary_claim_to_be_established_and_full_confidence(
    ) {
        let mut unverified = SemanticResponsePlan::capability_lookup_answer();
        unverified.claims[0].truth_status = TruthStatus::Unverified;
        assert_eq!(
            unverified.validate(),
            Err(SemanticPlanError::CertainRequiresEstablishedPrimaryClaim)
        );

        let mut uncertain = SemanticResponsePlan::capability_lookup_answer();
        uncertain.claims[0].confidence = Some(0.99);
        assert_eq!(
            uncertain.validate(),
            Err(SemanticPlanError::CertainRequiresEstablishedPrimaryClaim)
        );
    }

    #[test]
    fn invalid_semantic_plan_fails_closed_and_records_validation_error() {
        let mut plan = SemanticResponsePlan::capability_lookup_answer();
        plan.claims.clear();
        plan.legacy_fallback = Some("legacy text must not be returned".to_string());

        let response = Response::from_semantic_plan(ResponseIntent::Capability, plan);
        let retained_plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Internal realization error: invalid semantic response plan."
        );
        assert!(!retained_plan.realization_trace.semantic_precedence);
        assert!(!retained_plan.realization_trace.used_legacy_fallback);
        assert_eq!(
            retained_plan.realization_trace.validation_error,
            Some(SemanticPlanError::EmptyClaims)
        );
    }

    #[test]
    fn unsupported_speech_act_proposition_fails_closed_without_legacy_fallback() {
        let mut plan = SemanticResponsePlan::capability_lookup_answer();
        plan.speech_act = SpeechAct::Ask;
        plan.legacy_fallback = Some("legacy text must not be returned".to_string());

        let response = Response::from_semantic_plan(ResponseIntent::Capability, plan);
        let retained_plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Internal realization error: invalid semantic response plan."
        );
        assert!(!retained_plan.realization_trace.semantic_precedence);
        assert!(!retained_plan.realization_trace.used_legacy_fallback);
        assert_eq!(
            retained_plan.realization_trace.validation_error,
            Some(SemanticPlanError::UnsupportedSpeechActProposition)
        );
    }

    #[test]
    fn semantic_trace_propagates_through_response() {
        let response = Response::from_semantic_plan(
            ResponseIntent::Capability,
            SemanticResponsePlan::capability_lookup_answer(),
        );
        let plan = response
            .semantic_plan
            .as_ref()
            .expect("semantic plan is retained");

        assert!(plan.realization_trace.semantic_precedence);
        assert_eq!(plan.realization_trace.speech_act, SpeechAct::Inform);
        assert_eq!(plan.realization_trace.renderer, "semantic-response-plan-v1");
        assert!(
            response
                .slots
                .iter()
                .any(|(key, value)| key == "semantic_renderer"
                    && value == "semantic-response-plan-v1")
        );
        assert!(response
            .slots
            .iter()
            .any(|(key, value)| key == "semantic_precedence" && value == "true"));
    }

    #[test]
    fn uncertainty_semantic_plan_and_trace_propagate_through_live_response() {
        let response = Response::from_semantic_plan(
            ResponseIntent::Consciousness,
            SemanticResponsePlan::understanding_uncertainty_report(),
        );
        let plan = response
            .semantic_plan
            .as_ref()
            .expect("semantic plan is retained");

        let template_id = plan
            .realization_trace
            .template_id
            .as_deref()
            .expect("uncertainty lattice selects a template");
        let expected = UNDERSTANDING_UNCERTAINTY_TEMPLATES
            .iter()
            .find(|(candidate_id, _)| *candidate_id == template_id)
            .expect("trace template is curated");
        assert_eq!(response.body, expected.1);
        assert_eq!(plan.proposition, Proposition::UnderstandingUncertain);
        assert_eq!(plan.realization_trace.speech_act, SpeechAct::Inform);
        assert!(plan.realization_trace.semantic_precedence);
        assert!(!plan.realization_trace.used_legacy_fallback);
        assert!(response
            .slots
            .iter()
            .any(|(key, value)| { key == "semantic_precedence" && value == "true" }));
    }

    #[test]
    fn semantic_voice_request_overrides_intent_style_defaults() {
        let story = Response::from_semantic_plan(
            ResponseIntent::StoryPrompt,
            SemanticResponsePlan::story_listening_acknowledgement(),
        );
        let capability = Response::from_semantic_plan(
            ResponseIntent::Capability,
            SemanticResponsePlan::capability_lookup_answer(),
        );

        assert_eq!(
            ResponseIntent::StoryPrompt.default_style_hint(),
            Some(ResponseStyle::Playful)
        );
        assert_eq!(story.style_hint, Some(ResponseStyle::Warm));
        assert_eq!(capability.style_hint, Some(ResponseStyle::Direct));
    }

    #[test]
    fn research_status_renders_its_protected_topic_unchanged_and_traces_it() {
        let topic = "causal effect: A -> B";
        let response = Response::from_semantic_plan(
            ResponseIntent::ResearchStatus,
            SemanticResponsePlan::research_status(topic),
        );
        let plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "causal effect: A -> B keeps coming up — I'm still figuring out what I think about it."
        );
        assert!(plan.realization_trace.semantic_precedence);
        assert!(!plan.realization_trace.used_legacy_fallback);
        assert_eq!(
            plan.realization_trace.template_id.as_deref(),
            Some("research_status_topic_v1")
        );
        assert_eq!(
            plan.realization_trace.emitted_protected_slots,
            vec![ProtectedSlot {
                key: "topic".to_string(),
                value: topic.to_string()
            }]
        );
    }

    #[test]
    fn research_status_rejects_invalid_duplicate_and_missing_protected_slots() {
        let mut invalid_key = SemanticResponsePlan::research_status("topic");
        invalid_key.protected_slots[0].key = "Topic".to_string();
        assert_eq!(
            invalid_key.validate(),
            Err(SemanticPlanError::InvalidProtectedSlotKey)
        );

        let mut invalid_value = SemanticResponsePlan::research_status("topic");
        invalid_value.protected_slots[0].value = "  ".to_string();
        assert_eq!(
            invalid_value.validate(),
            Err(SemanticPlanError::InvalidProtectedSlotValue)
        );

        let mut duplicate = SemanticResponsePlan::research_status("topic");
        duplicate
            .protected_slots
            .push(duplicate.protected_slots[0].clone());
        assert_eq!(
            duplicate.validate(),
            Err(SemanticPlanError::DuplicateProtectedSlotKey)
        );

        let mut missing = SemanticResponsePlan::research_status("topic");
        missing.protected_slots.clear();
        missing.legacy_fallback = Some("legacy text must not be returned".to_string());
        assert_eq!(
            missing.validate(),
            Err(SemanticPlanError::MissingRequiredProtectedSlot)
        );
        let response = Response::from_semantic_plan(ResponseIntent::ResearchStatus, missing);
        assert_eq!(
            response.body,
            "Internal realization error: invalid semantic response plan."
        );
        assert!(
            !response
                .semantic_plan
                .expect("semantic plan is retained")
                .realization_trace
                .used_legacy_fallback
        );
    }

    #[test]
    fn correction_renders_protected_prior_and_replacement_unchanged_and_traces_them() {
        let prior = "the service is unavailable";
        let replacement = "the service is available";
        let response = Response::from_semantic_plan(
            ResponseIntent::Statement,
            SemanticResponsePlan::correction(prior, replacement),
        );
        let plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Correction: the service is unavailable is replaced by the service is available."
        );
        assert_eq!(plan.speech_act, SpeechAct::Correct);
        assert_eq!(plan.proposition, Proposition::Correction);
        assert_eq!(plan.claims[0].truth_status, TruthStatus::Established);
        assert_eq!(plan.claims[0].confidence, Some(1.0));
        assert_eq!(
            plan.realization_trace.template_id.as_deref(),
            Some("correction_prior_to_replacement_v1")
        );
        assert_eq!(
            plan.realization_trace.emitted_protected_slots,
            vec![
                ProtectedSlot {
                    key: "prior_claim".to_string(),
                    value: prior.to_string()
                },
                ProtectedSlot {
                    key: "replacement_claim".to_string(),
                    value: replacement.to_string()
                },
            ]
        );
    }

    #[test]
    fn correction_requires_exactly_two_distinct_named_protected_claim_slots() {
        let mut missing = SemanticResponsePlan::correction("prior", "replacement");
        missing.protected_slots.pop();
        assert_eq!(
            missing.validate(),
            Err(SemanticPlanError::InvalidCorrectionProtectedSlots)
        );

        let mut duplicate_value = SemanticResponsePlan::correction("same", "replacement");
        duplicate_value.protected_slots[1].value = "same".to_string();
        assert_eq!(
            duplicate_value.validate(),
            Err(SemanticPlanError::DuplicateProtectedSlotValue)
        );

        let mut wrong_slot = SemanticResponsePlan::correction("prior", "replacement");
        wrong_slot.protected_slots[1].key = "other_claim".to_string();
        assert_eq!(
            wrong_slot.validate(),
            Err(SemanticPlanError::MissingRequiredProtectedSlot)
        );

        let mut extra_slot = SemanticResponsePlan::correction("prior", "replacement");
        extra_slot.protected_slots.push(ProtectedSlot {
            key: "extra_claim".to_string(),
            value: "extra".to_string(),
        });
        assert_eq!(
            extra_slot.validate(),
            Err(SemanticPlanError::InvalidCorrectionProtectedSlots)
        );
    }

    #[test]
    fn invalid_correction_fails_closed_without_legacy_fallback() {
        let mut plan = SemanticResponsePlan::correction("same", "same");
        plan.legacy_fallback = Some("legacy text must not be returned".to_string());

        let response = Response::from_semantic_plan(ResponseIntent::Statement, plan);
        let retained_plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Internal realization error: invalid semantic response plan."
        );
        assert_eq!(
            retained_plan.realization_trace.validation_error,
            Some(SemanticPlanError::DuplicateProtectedSlotValue)
        );
        assert!(!retained_plan.realization_trace.used_legacy_fallback);
    }

    #[test]
    fn causal_explanation_renders_registered_template_and_traces_protected_slots() {
        let cause = "the cache was cold";
        let effect = "the first request took longer";
        let response = Response::from_semantic_plan(
            ResponseIntent::Statement,
            SemanticResponsePlan::causal_explanation(cause, effect),
        );
        let replay = Response::from_semantic_plan(
            ResponseIntent::Statement,
            SemanticResponsePlan::causal_explanation(cause, effect),
        );
        assert_eq!(response.body, replay.body);
        assert_eq!(
            response
                .semantic_plan
                .as_ref()
                .expect("semantic plan is retained")
                .realization_trace
                .replay_seed,
            replay
                .semantic_plan
                .as_ref()
                .expect("semantic plan is retained")
                .realization_trace
                .replay_seed
        );
        let plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Because the cache was cold, the first request took longer."
        );
        assert_eq!(plan.speech_act, SpeechAct::Explain);
        assert_eq!(plan.proposition, Proposition::CausalExplanation);
        assert_eq!(
            plan.realization_trace.template_id.as_deref(),
            Some("causal_explanation_cause_to_effect_v1")
        );
        assert!(plan.realization_trace.replay_seed.is_some());
        assert_eq!(
            plan.realization_trace.emitted_protected_slots,
            vec![
                ProtectedSlot {
                    key: "cause".to_string(),
                    value: cause.to_string()
                },
                ProtectedSlot {
                    key: "effect".to_string(),
                    value: effect.to_string()
                },
            ]
        );
    }

    #[test]
    fn causal_explanation_requires_exactly_distinct_cause_and_effect_slots() {
        let mut missing = SemanticResponsePlan::causal_explanation("cause", "effect");
        missing.protected_slots.pop();
        assert_eq!(
            missing.validate(),
            Err(SemanticPlanError::InvalidCausalExplanationProtectedSlots)
        );

        let mut extra = SemanticResponsePlan::causal_explanation("cause", "effect");
        extra.protected_slots.push(ProtectedSlot {
            key: "context".to_string(),
            value: "extra".to_string(),
        });
        assert_eq!(
            extra.validate(),
            Err(SemanticPlanError::InvalidCausalExplanationProtectedSlots)
        );

        let mut wrong_key = SemanticResponsePlan::causal_explanation("cause", "effect");
        wrong_key.protected_slots[1].key = "outcome".to_string();
        assert_eq!(
            wrong_key.validate(),
            Err(SemanticPlanError::InvalidCausalExplanationProtectedSlots)
        );

        let mut duplicate_value = SemanticResponsePlan::causal_explanation("same", "effect");
        duplicate_value.protected_slots[1].value = "same".to_string();
        assert_eq!(
            duplicate_value.validate(),
            Err(SemanticPlanError::DuplicateProtectedSlotValue)
        );
    }

    #[test]
    fn invalid_causal_explanation_fails_closed_without_legacy_fallback() {
        let mut plan = SemanticResponsePlan::causal_explanation("same", "same");
        plan.legacy_fallback = Some("legacy text must not be returned".to_string());

        let response = Response::from_semantic_plan(ResponseIntent::Statement, plan);
        let retained_plan = response.semantic_plan.expect("semantic plan is retained");

        assert_eq!(
            response.body,
            "Internal realization error: invalid semantic response plan."
        );
        assert_eq!(
            retained_plan.realization_trace.validation_error,
            Some(SemanticPlanError::DuplicateProtectedSlotValue)
        );
        assert!(!retained_plan.realization_trace.used_legacy_fallback);
    }
}
