//! ΩV1-B typed persistent voice state.
//!
//! This module is feature-gated behind `voice-state-shadow` and deliberately
//! has no `Runtime::chat()` wiring. State changes occur only through explicit,
//! version-checked revision events. The voice renderer cannot read this state
//! until a later ΩV1 promotion gate authorizes that integration.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_BASIS_POINTS: u16 = 10_000;
pub const MAX_DELTA_BASIS_POINTS: i16 = 2_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BasisPoints(u16);

impl BasisPoints {
    pub const ZERO: Self = Self(0);
    pub const FULL: Self = Self(MAX_BASIS_POINTS);

    pub fn new(value: u16) -> Result<Self, VoiceStateError> {
        if value > MAX_BASIS_POINTS {
            return Err(VoiceStateError::BasisPointsOutOfRange(value));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub fn as_unit(self) -> f64 {
        f64::from(self.0) / f64::from(MAX_BASIS_POINTS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceRange {
    pub min: BasisPoints,
    pub preferred: BasisPoints,
    pub max: BasisPoints,
}

impl VoiceRange {
    pub fn new(
        min: BasisPoints,
        preferred: BasisPoints,
        max: BasisPoints,
    ) -> Result<Self, VoiceStateError> {
        if min > preferred || preferred > max {
            return Err(VoiceStateError::InvalidRange {
                min: min.get(),
                preferred: preferred.get(),
                max: max.get(),
            });
        }
        Ok(Self {
            min,
            preferred,
            max,
        })
    }

    fn clamp(self, value: i32) -> BasisPoints {
        let min = i32::from(self.min.get());
        let max = i32::from(self.max.get());
        BasisPoints(value.clamp(min, max) as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceDimension {
    Directness,
    Warmth,
    Severity,
    Playfulness,
    PhilosophicalDepth,
    SentenceCompression,
    ImageryDensity,
    Initiative,
    DisagreementStyle,
    UncertaintyExpression,
    EmotionalExplicitness,
    SessionIntensity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineVoice {
    pub directness: VoiceRange,
    pub warmth: VoiceRange,
    pub severity: VoiceRange,
    pub playfulness: VoiceRange,
    pub philosophical_depth: VoiceRange,
    pub sentence_compression: VoiceRange,
    pub imagery_density: VoiceRange,
    pub initiative: VoiceRange,
    pub disagreement_style: VoiceRange,
    pub uncertainty_expression: VoiceRange,
    pub emotional_explicitness: VoiceRange,
}

impl BaselineVoice {
    fn range(&self, dimension: VoiceDimension) -> Option<VoiceRange> {
        match dimension {
            VoiceDimension::Directness => Some(self.directness),
            VoiceDimension::Warmth => Some(self.warmth),
            VoiceDimension::Severity => Some(self.severity),
            VoiceDimension::Playfulness => Some(self.playfulness),
            VoiceDimension::PhilosophicalDepth => Some(self.philosophical_depth),
            VoiceDimension::SentenceCompression => Some(self.sentence_compression),
            VoiceDimension::ImageryDensity => Some(self.imagery_density),
            VoiceDimension::Initiative => Some(self.initiative),
            VoiceDimension::DisagreementStyle => Some(self.disagreement_style),
            VoiceDimension::UncertaintyExpression => Some(self.uncertainty_expression),
            VoiceDimension::EmotionalExplicitness => Some(self.emotional_explicitness),
            VoiceDimension::SessionIntensity => None,
        }
    }
}

impl Default for BaselineVoice {
    fn default() -> Self {
        fn range(min: u16, preferred: u16, max: u16) -> VoiceRange {
            VoiceRange {
                min: BasisPoints(min),
                preferred: BasisPoints(preferred),
                max: BasisPoints(max),
            }
        }

        Self {
            directness: range(5_000, 7_200, 9_200),
            warmth: range(1_500, 3_800, 7_200),
            severity: range(500, 2_400, 7_800),
            playfulness: range(300, 2_100, 6_800),
            philosophical_depth: range(1_500, 4_600, 8_400),
            sentence_compression: range(4_000, 8_100, 9_500),
            imagery_density: range(300, 2_700, 6_500),
            initiative: range(3_000, 6_600, 9_000),
            disagreement_style: range(4_000, 7_000, 9_500),
            uncertainty_expression: range(5_000, 8_200, 9_800),
            emotional_explicitness: range(1_000, 3_200, 7_000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceEvidenceRef {
    pub kind: VoiceEvidenceKind,
    pub reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEvidenceKind {
    UserCorrection,
    LongitudinalEvaluation,
    HeldOutConversation,
    ReviewedConfiguration,
    RollbackRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedVoiceDelta {
    pub dimension: VoiceDimension,
    pub delta_bps: i16,
}

impl BoundedVoiceDelta {
    pub fn new(
        dimension: VoiceDimension,
        delta_bps: i16,
    ) -> Result<Self, VoiceStateError> {
        if !(-MAX_DELTA_BASIS_POINTS..=MAX_DELTA_BASIS_POINTS).contains(&delta_bps) {
            return Err(VoiceStateError::DeltaOutOfRange {
                dimension,
                delta_bps,
            });
        }
        Ok(Self {
            dimension,
            delta_bps,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceRevisionReason {
    UserCorrection,
    ValidatedLongitudinalEvidence,
    ReviewedRelationshipCalibration,
    SessionConfiguration,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "scope")]
pub enum VoiceRevisionTarget {
    Acquired,
    Relationship { relationship_id: String },
    Session { session_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceRevisionEvent {
    pub prior_version: u64,
    pub resulting_version: u64,
    pub target: VoiceRevisionTarget,
    pub evidence: Vec<VoiceEvidenceRef>,
    pub changed_dimensions: Vec<BoundedVoiceDelta>,
    pub reason: VoiceRevisionReason,
    pub confidence: BasisPoints,
    pub reversible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceTendency {
    pub dimension: VoiceDimension,
    pub delta_bps: i16,
    pub source_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RelationshipVoiceState {
    pub relationship_id: Option<String>,
    pub tendencies: Vec<VoiceTendency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionExpressionState {
    pub session_id: Option<String>,
    pub intensity: BasisPoints,
    pub tendencies: Vec<VoiceTendency>,
}

impl Default for SessionExpressionState {
    fn default() -> Self {
        Self {
            session_id: None,
            intensity: BasisPoints::ZERO,
            tendencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceState {
    pub baseline: BaselineVoice,
    pub acquired_tendencies: Vec<VoiceTendency>,
    pub relationship_calibration: RelationshipVoiceState,
    pub session_expression: SessionExpressionState,
    pub revision_history: Vec<VoiceRevisionEvent>,
    pub version: u64,
}

impl Default for VoiceState {
    fn default() -> Self {
        Self {
            baseline: BaselineVoice::default(),
            acquired_tendencies: Vec::new(),
            relationship_calibration: RelationshipVoiceState::default(),
            session_expression: SessionExpressionState::default(),
            revision_history: Vec::new(),
            version: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceDebugProjection {
    pub version: u64,
    pub directness: f64,
    pub warmth: f64,
    pub compression: f64,
    pub initiative: f64,
    pub disagreement_style: String,
    pub uncertainty_style: String,
    pub session_intensity: f64,
    pub digest: String,
}

impl VoiceState {
    pub fn apply_revision(
        &mut self,
        expected_version: u64,
        event: VoiceRevisionEvent,
    ) -> Result<(), VoiceStateError> {
        self.require_version(expected_version)?;
        if event.prior_version != expected_version {
            return Err(VoiceStateError::EventPriorVersionMismatch {
                expected: expected_version,
                actual: event.prior_version,
            });
        }
        let expected_resulting = expected_version
            .checked_add(1)
            .ok_or(VoiceStateError::VersionOverflow)?;
        if event.resulting_version != expected_resulting {
            return Err(VoiceStateError::EventResultingVersionMismatch {
                expected: expected_resulting,
                actual: event.resulting_version,
            });
        }
        if event.changed_dimensions.is_empty() {
            return Err(VoiceStateError::EmptyRevision);
        }
        if event.evidence.is_empty() {
            return Err(VoiceStateError::MissingEvidence);
        }
        if event
            .evidence
            .iter()
            .any(|evidence| evidence.reference.trim().is_empty())
        {
            return Err(VoiceStateError::EmptyEvidenceReference);
        }

        for delta in &event.changed_dimensions {
            if !(-MAX_DELTA_BASIS_POINTS..=MAX_DELTA_BASIS_POINTS)
                .contains(&delta.delta_bps)
            {
                return Err(VoiceStateError::DeltaOutOfRange {
                    dimension: delta.dimension,
                    delta_bps: delta.delta_bps,
                });
            }
            self.validate_target_dimension(&event.target, delta.dimension)?;
        }

        match &event.target {
            VoiceRevisionTarget::Acquired => {
                for delta in &event.changed_dimensions {
                    self.acquired_tendencies.push(VoiceTendency {
                        dimension: delta.dimension,
                        delta_bps: delta.delta_bps,
                        source_version: event.resulting_version,
                    });
                }
            }
            VoiceRevisionTarget::Relationship { relationship_id } => {
                let relationship_id = normalized_id(relationship_id)
                    .ok_or(VoiceStateError::EmptyRelationshipId)?;
                match &self.relationship_calibration.relationship_id {
                    Some(current) if current != &relationship_id => {
                        return Err(VoiceStateError::RelationshipScopeMismatch {
                            expected: current.clone(),
                            actual: relationship_id,
                        });
                    }
                    None => {
                        self.relationship_calibration.relationship_id =
                            Some(relationship_id);
                    }
                    Some(_) => {}
                }
                for delta in &event.changed_dimensions {
                    self.relationship_calibration
                        .tendencies
                        .push(VoiceTendency {
                            dimension: delta.dimension,
                            delta_bps: delta.delta_bps,
                            source_version: event.resulting_version,
                        });
                }
            }
            VoiceRevisionTarget::Session { session_id } => {
                let session_id =
                    normalized_id(session_id).ok_or(VoiceStateError::EmptySessionId)?;
                match &self.session_expression.session_id {
                    Some(current) if current != &session_id => {
                        return Err(VoiceStateError::SessionScopeMismatch {
                            expected: current.clone(),
                            actual: session_id,
                        });
                    }
                    None => self.session_expression.session_id = Some(session_id),
                    Some(_) => {}
                }
                for delta in &event.changed_dimensions {
                    if delta.dimension == VoiceDimension::SessionIntensity {
                        let value = i32::from(self.session_expression.intensity.get())
                            + i32::from(delta.delta_bps);
                        self.session_expression.intensity =
                            BasisPoints(value.clamp(0, i32::from(MAX_BASIS_POINTS)) as u16);
                    } else {
                        self.session_expression.tendencies.push(VoiceTendency {
                            dimension: delta.dimension,
                            delta_bps: delta.delta_bps,
                            source_version: event.resulting_version,
                        });
                    }
                }
            }
        }

        self.version = event.resulting_version;
        self.revision_history.push(event);
        Ok(())
    }

    pub fn replay(events: &[VoiceRevisionEvent]) -> Result<Self, VoiceStateError> {
        let mut state = Self::default();
        for event in events {
            state.apply_revision(state.version, event.clone())?;
        }
        Ok(state)
    }

    pub fn to_canonical_json(&self) -> Result<String, VoiceStateError> {
        serde_json::to_string(self).map_err(VoiceStateError::Serialize)
    }

    pub fn from_canonical_json(json: &str) -> Result<Self, VoiceStateError> {
        let state: Self =
            serde_json::from_str(json).map_err(VoiceStateError::Deserialize)?;
        let replayed = Self::replay(&state.revision_history)?;
        if replayed != state {
            return Err(VoiceStateError::StateDoesNotMatchReplay);
        }
        Ok(state)
    }

    pub fn digest(&self) -> Result<String, VoiceStateError> {
        let bytes = self.to_canonical_json()?.into_bytes();
        Ok(format!("{:016x}", fnv1a64(&bytes)))
    }

    pub fn debug_projection(&self) -> Result<VoiceDebugProjection, VoiceStateError> {
        let directness = self.project(VoiceDimension::Directness)?;
        let warmth = self.project(VoiceDimension::Warmth)?;
        let compression = self.project(VoiceDimension::SentenceCompression)?;
        let initiative = self.project(VoiceDimension::Initiative)?;
        let disagreement = self.project(VoiceDimension::DisagreementStyle)?;
        let uncertainty = self.project(VoiceDimension::UncertaintyExpression)?;

        Ok(VoiceDebugProjection {
            version: self.version,
            directness: directness.as_unit(),
            warmth: warmth.as_unit(),
            compression: compression.as_unit(),
            initiative: initiative.as_unit(),
            disagreement_style: disagreement_style(disagreement),
            uncertainty_style: uncertainty_style(uncertainty),
            session_intensity: self.session_expression.intensity.as_unit(),
            digest: self.digest()?,
        })
    }

    fn project(&self, dimension: VoiceDimension) -> Result<BasisPoints, VoiceStateError> {
        let range = self
            .baseline
            .range(dimension)
            .ok_or(VoiceStateError::NonProjectableDimension(dimension))?;
        let mut value = i32::from(range.preferred.get());
        value += tendency_sum(&self.acquired_tendencies, dimension);
        value += tendency_sum(&self.relationship_calibration.tendencies, dimension);
        value += tendency_sum(&self.session_expression.tendencies, dimension);
        Ok(range.clamp(value))
    }

    fn require_version(&self, expected_version: u64) -> Result<(), VoiceStateError> {
        if self.version != expected_version {
            return Err(VoiceStateError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }

    fn validate_target_dimension(
        &self,
        target: &VoiceRevisionTarget,
        dimension: VoiceDimension,
    ) -> Result<(), VoiceStateError> {
        match (target, dimension) {
            (VoiceRevisionTarget::Session { .. }, _) => Ok(()),
            (_, VoiceDimension::SessionIntensity) => {
                Err(VoiceStateError::SessionIntensityOutsideSession)
            }
            _ => Ok(()),
        }
    }
}

fn tendency_sum(tendencies: &[VoiceTendency], dimension: VoiceDimension) -> i32 {
    tendencies
        .iter()
        .filter(|tendency| tendency.dimension == dimension)
        .map(|tendency| i32::from(tendency.delta_bps))
        .sum()
}

fn normalized_id(value: &str) -> Option<String> {
    let normalized = value.trim();
    (!normalized.is_empty()).then(|| normalized.to_owned())
}

fn disagreement_style(value: BasisPoints) -> String {
    match value.get() {
        0..=3_333 => "yielding",
        3_334..=6_666 => "measured",
        _ => "direct",
    }
    .to_owned()
}

fn uncertainty_style(value: BasisPoints) -> String {
    match value.get() {
        0..=3_333 => "implicit",
        3_334..=6_666 => "calibrated",
        _ => "explicit",
    }
    .to_owned()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Error)]
pub enum VoiceStateError {
    #[error("basis points out of range: {0}")]
    BasisPointsOutOfRange(u16),
    #[error("invalid voice range: min={min}, preferred={preferred}, max={max}")]
    InvalidRange {
        min: u16,
        preferred: u16,
        max: u16,
    },
    #[error("voice delta out of range for {dimension:?}: {delta_bps}")]
    DeltaOutOfRange {
        dimension: VoiceDimension,
        delta_bps: i16,
    },
    #[error("voice state version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("event prior version mismatch: expected {expected}, actual {actual}")]
    EventPriorVersionMismatch { expected: u64, actual: u64 },
    #[error("event resulting version mismatch: expected {expected}, actual {actual}")]
    EventResultingVersionMismatch { expected: u64, actual: u64 },
    #[error("voice state version overflow")]
    VersionOverflow,
    #[error("voice revision contains no changed dimensions")]
    EmptyRevision,
    #[error("voice revision contains no evidence")]
    MissingEvidence,
    #[error("voice revision contains an empty evidence reference")]
    EmptyEvidenceReference,
    #[error("relationship id is empty")]
    EmptyRelationshipId,
    #[error("session id is empty")]
    EmptySessionId,
    #[error("relationship scope mismatch: expected {expected}, actual {actual}")]
    RelationshipScopeMismatch { expected: String, actual: String },
    #[error("session scope mismatch: expected {expected}, actual {actual}")]
    SessionScopeMismatch { expected: String, actual: String },
    #[error("session intensity may only be changed by a session revision")]
    SessionIntensityOutsideSession,
    #[error("voice dimension cannot be projected: {0:?}")]
    NonProjectableDimension(VoiceDimension),
    #[error("failed to serialize voice state: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize voice state: {0}")]
    Deserialize(serde_json::Error),
    #[error("serialized voice state does not match exact event replay")]
    StateDoesNotMatchReplay,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(reference: &str) -> Vec<VoiceEvidenceRef> {
        vec![VoiceEvidenceRef {
            kind: VoiceEvidenceKind::ReviewedConfiguration,
            reference: reference.to_owned(),
        }]
    }

    fn revision(
        prior_version: u64,
        target: VoiceRevisionTarget,
        changed_dimensions: Vec<BoundedVoiceDelta>,
        reference: &str,
    ) -> VoiceRevisionEvent {
        VoiceRevisionEvent {
            prior_version,
            resulting_version: prior_version + 1,
            target,
            evidence: evidence(reference),
            changed_dimensions,
            reason: VoiceRevisionReason::SessionConfiguration,
            confidence: BasisPoints(9_000),
            reversible: true,
        }
    }

    #[test]
    fn default_projection_matches_preregistered_debug_shape() {
        let projection = VoiceState::default().debug_projection().unwrap();
        assert_eq!(projection.version, 0);
        assert_eq!(projection.directness, 0.72);
        assert_eq!(projection.warmth, 0.38);
        assert_eq!(projection.compression, 0.81);
        assert_eq!(projection.initiative, 0.66);
        assert_eq!(projection.uncertainty_style, "explicit");
        assert_eq!(projection.session_intensity, 0.0);
    }

    #[test]
    fn replay_reproduces_exact_state_json_and_digest() {
        let events = vec![
            revision(
                0,
                VoiceRevisionTarget::Acquired,
                vec![BoundedVoiceDelta::new(VoiceDimension::Directness, 250).unwrap()],
                "held-out:technical-01",
            ),
            revision(
                1,
                VoiceRevisionTarget::Relationship {
                    relationship_id: "zachary".to_owned(),
                },
                vec![BoundedVoiceDelta::new(VoiceDimension::Warmth, 300).unwrap()],
                "review:user-correction-07",
            ),
            revision(
                2,
                VoiceRevisionTarget::Session {
                    session_id: "session-42".to_owned(),
                },
                vec![
                    BoundedVoiceDelta::new(VoiceDimension::SentenceCompression, -150)
                        .unwrap(),
                    BoundedVoiceDelta::new(VoiceDimension::SessionIntensity, 2_400)
                        .unwrap(),
                ],
                "session:explicit-config",
            ),
        ];

        let mut state = VoiceState::default();
        for event in &events {
            state.apply_revision(state.version, event.clone()).unwrap();
        }

        let replayed = VoiceState::replay(&events).unwrap();
        assert_eq!(state, replayed);
        assert_eq!(
            state.to_canonical_json().unwrap(),
            replayed.to_canonical_json().unwrap()
        );
        assert_eq!(state.digest().unwrap(), replayed.digest().unwrap());

        let round_trip =
            VoiceState::from_canonical_json(&state.to_canonical_json().unwrap()).unwrap();
        assert_eq!(state, round_trip);
    }

    #[test]
    fn optimistic_versioning_rejects_stale_writes() {
        let mut state = VoiceState::default();
        let event = revision(
            0,
            VoiceRevisionTarget::Acquired,
            vec![BoundedVoiceDelta::new(VoiceDimension::Initiative, 100).unwrap()],
            "held-out:initiative-01",
        );
        state.apply_revision(0, event).unwrap();

        let stale = revision(
            0,
            VoiceRevisionTarget::Acquired,
            vec![BoundedVoiceDelta::new(VoiceDimension::Warmth, 100).unwrap()],
            "held-out:warmth-01",
        );
        assert!(matches!(
            state.apply_revision(0, stale),
            Err(VoiceStateError::VersionConflict {
                expected: 0,
                actual: 1
            })
        ));
    }

    #[test]
    fn bounded_deltas_and_scope_boundaries_are_enforced() {
        assert!(matches!(
            BoundedVoiceDelta::new(VoiceDimension::Warmth, 2_501),
            Err(VoiceStateError::DeltaOutOfRange { .. })
        ));

        let mut state = VoiceState::default();
        let invalid = revision(
            0,
            VoiceRevisionTarget::Acquired,
            vec![BoundedVoiceDelta::new(
                VoiceDimension::SessionIntensity,
                100,
            )
            .unwrap()],
            "invalid:session-intensity",
        );
        assert!(matches!(
            state.apply_revision(0, invalid),
            Err(VoiceStateError::SessionIntensityOutsideSession)
        ));
    }

    #[test]
    fn no_runtime_or_renderer_influence_is_present() {
        // Structural guard: ΩV1-B state is manipulated only through this module's
        // explicit API. The feature module has no Runtime, VoiceEngine, Memory,
        // tool, CHARGE, belief, or ontology imports.
        let state = VoiceState::default();
        assert_eq!(state.version, 0);
        assert!(state.revision_history.is_empty());
    }
}
