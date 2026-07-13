//! STLM L0-B typed semantic response-program boundary.
//!
//! This module is feature-gated and deliberately has no access to `Runtime::chat()`,
//! persistence, tools, routing, belief mutation, ontology mutation, CHARGE discharge,
//! or autonomous action. It validates and replays semantic authorization packets only.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const DIGEST_DOMAIN: &[u8] = b"starfire-stlm-semantic-response-program-v1";

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

id_type!(ResponseProgramId);
id_type!(ClaimId);
id_type!(OperationId);
id_type!(ObservationId);
id_type!(MissingVariableId);
id_type!(PredictionId);
id_type!(SubjectScope);
id_type!(CognitiveStateVersion);
id_type!(CompanionStateVersion);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseProgramDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticResponseIntent {
    FactualAnswer,
    Explanation,
    Correction,
    Contrast,
    SelfCheck,
    CapabilityDisclosure,
    EvidenceRequest,
    Commitment,
    Abstention,
    RelationalAcknowledgment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimPolarity {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SensitivityLevel {
    Public,
    Personal,
    Sensitive,
    HighlySensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicStatus {
    Certain,
    Probable,
    Possible,
    Uncertain,
    Unknown,
}

impl EpistemicStatus {
    #[must_use]
    pub const fn confidence_bounds(self) -> (u16, u16) {
        match self {
            Self::Certain => (9_000, 10_000),
            Self::Probable => (7_000, 8_999),
            Self::Possible => (3_000, 6_999),
            Self::Uncertain => (1, 2_999),
            Self::Unknown => (0, 0),
        }
    }

    #[must_use]
    pub const fn permits(self, confidence_bps: u16) -> bool {
        let (minimum, maximum) = self.confidence_bounds();
        confidence_bps >= minimum && confidence_bps <= maximum
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedClaim {
    pub id: ClaimId,
    pub semantic_key: String,
    pub polarity: ClaimPolarity,
    pub confidence_bps: u16,
    pub epistemic_status: EpistemicStatus,
    pub sensitivity: SensitivityLevel,
    pub disclosure_scope: SubjectScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProhibitedClaim {
    pub id: ClaimId,
    pub semantic_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicConstraint {
    pub claim: ClaimId,
    pub required_status: EpistemicStatus,
    pub minimum_confidence_bps: u16,
    pub maximum_confidence_bps: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensitivityPolicy {
    pub maximum_disclosure: SensitivityLevel,
    pub disclosure_scope: SubjectScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    Brief,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VocabularyLevel {
    Plain,
    Standard,
    Technical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogueMode {
    Declarative,
    Collaborative,
    QuestionLed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcknowledgmentLevel {
    None,
    Brief,
    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleEnvelope {
    pub detail: DetailLevel,
    pub vocabulary: VocabularyLevel,
    pub dialogue: DialogueMode,
    pub acknowledgment: AcknowledgmentLevel,
    pub allow_first_person: bool,
    pub allow_questions: bool,
    pub maximum_paragraphs: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputBudget {
    pub maximum_characters: u32,
    pub maximum_sentences: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeBudget {
    pub maximum_operations: u16,
    pub maximum_claims: u16,
    pub maximum_verification_steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbstentionReason {
    InsufficientEvidence,
    ContradictoryEvidence,
    SensitiveContext,
    UnsupportedIntent,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscourseOperationKind {
    Assert(ClaimId),
    Qualify {
        claim: ClaimId,
        status: EpistemicStatus,
    },
    Contrast {
        left: ClaimId,
        right: ClaimId,
    },
    Correct {
        prior: ClaimId,
        replacement: ClaimId,
    },
    Explain {
        claims: Vec<ClaimId>,
    },
    Acknowledge(ObservationId),
    RequestEvidence(MissingVariableId),
    Commit(PredictionId),
    Abstain(AbstentionReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscourseOperation {
    pub id: OperationId,
    pub kind: DiscourseOperationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticResponseProgramPayload {
    pub id: ResponseProgramId,
    pub source_state_version: CognitiveStateVersion,
    pub companion_state_version: Option<CompanionStateVersion>,
    pub subject_scope: SubjectScope,
    pub intent: SemanticResponseIntent,
    pub operations: Vec<DiscourseOperation>,
    pub required_claims: Vec<AuthorizedClaim>,
    pub optional_claims: Vec<AuthorizedClaim>,
    pub prohibited_claims: Vec<ProhibitedClaim>,
    pub epistemic_constraints: Vec<EpistemicConstraint>,
    pub sensitivity: SensitivityPolicy,
    pub style: StyleEnvelope,
    pub output_budget: OutputBudget,
    pub compute_budget: ComputeBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticResponseProgram {
    pub payload: SemanticResponseProgramPayload,
    pub digest: ResponseProgramDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticValidationContext {
    pub cognitive_state_version: CognitiveStateVersion,
    pub companion_state_version: Option<CompanionStateVersion>,
    pub subject_scope: SubjectScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProgramEvent {
    pub registry_version: u64,
    pub program: SemanticResponseProgram,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticProgramRegistry {
    version: u64,
    programs: BTreeMap<ResponseProgramId, SemanticResponseProgram>,
    events: Vec<SemanticProgramEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProgramTransition {
    pub registry_version: u64,
    pub event: SemanticProgramEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProgramAuthorityBoundary {
    pub runtime_chat_wiring: bool,
    pub generated_text_influence: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub companion_mutation_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> SemanticProgramAuthorityBoundary {
    SemanticProgramAuthorityBoundary {
        runtime_chat_wiring: false,
        generated_text_influence: false,
        persistence_authority: false,
        routing_authority: false,
        companion_mutation_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SemanticProgramError {
    #[error("program identifier must be nonzero")]
    EmptyProgramId,
    #[error("source cognitive-state version must be nonzero")]
    EmptySourceStateVersion,
    #[error("companion-state version, when present, must be nonzero")]
    EmptyCompanionStateVersion,
    #[error("subject scope must be nonzero")]
    EmptySubjectScope,
    #[error("program source cognitive-state version is stale: expected {expected}, actual {actual}")]
    StaleCognitiveStateVersion { expected: u64, actual: u64 },
    #[error("program source companion-state version is stale or mismatched")]
    CompanionStateVersionMismatch,
    #[error("program subject scope does not match the validation context")]
    SubjectScopeMismatch,
    #[error("program has no discourse operations")]
    EmptyOperations,
    #[error("operation identifiers must be contiguous and begin at one")]
    NoncanonicalOperationOrder,
    #[error("claim identifiers must be nonzero, unique, and strictly ordered")]
    NoncanonicalClaimOrder,
    #[error("epistemic constraints must be unique and strictly ordered by claim")]
    NoncanonicalEpistemicOrder,
    #[error("semantic keys must be nonempty canonical identifiers")]
    InvalidSemanticKey,
    #[error("semantic keys must be unique across authorized claims")]
    DuplicateSemanticKey,
    #[error("required or optional claims overlap prohibited claims")]
    AuthorizedProhibitedOverlap,
    #[error("required and optional claims overlap")]
    AuthorizedClaimOverlap,
    #[error("discourse operation references an unknown or prohibited claim")]
    UnknownClaimReference,
    #[error("contrast and correction operations require distinct claims")]
    DegenerateBinaryOperation,
    #[error("explanation operations require canonical nonempty claim references")]
    InvalidExplanationClaims,
    #[error("claim confidence is outside 0..=10_000 or inconsistent with status")]
    InvalidClaimConfidence,
    #[error("epistemic constraint is malformed or inconsistent with its claim")]
    InvalidEpistemicConstraint,
    #[error("a required claim is not represented by any discourse operation")]
    UnrepresentedRequiredClaim,
    #[error("sensitivity policy exceeds the authorized disclosure boundary")]
    SensitivityViolation,
    #[error("style envelope is invalid")]
    InvalidStyleEnvelope,
    #[error("output budget is invalid")]
    InvalidOutputBudget,
    #[error("compute budget is invalid or exceeded")]
    InvalidComputeBudget,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("program digest is zero")]
    EmptyDigest,
    #[error("program digest does not match canonical bytes")]
    DigestMismatch,
    #[error("registry optimistic version conflict: expected {expected}, actual {actual}")]
    RegistryVersionConflict { expected: u64, actual: u64 },
    #[error("registry version overflow")]
    RegistryVersionOverflow,
    #[error("program identifier already exists in the registry")]
    DuplicateProgram,
    #[error("replay event versions must be contiguous and begin at one")]
    ReplayVersionMismatch,
}

impl SemanticResponseProgram {
    pub fn validate(
        payload: SemanticResponseProgramPayload,
        context: SemanticValidationContext,
    ) -> Result<Self, SemanticProgramError> {
        validate_payload(&payload, context)?;
        let digest = digest_payload(&payload)?;
        if digest.0 == 0 {
            return Err(SemanticProgramError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        context: SemanticValidationContext,
    ) -> Result<(), SemanticProgramError> {
        validate_payload(&self.payload, context)?;
        let expected = digest_payload(&self.payload)?;
        if self.digest.0 == 0 {
            return Err(SemanticProgramError::EmptyDigest);
        }
        if self.digest != expected {
            return Err(SemanticProgramError::DigestMismatch);
        }
        Ok(())
    }

    pub fn verify_replay_integrity(&self) -> Result<(), SemanticProgramError> {
        let context = SemanticValidationContext {
            cognitive_state_version: self.payload.source_state_version,
            companion_state_version: self.payload.companion_state_version,
            subject_scope: self.payload.subject_scope,
        };
        self.verify_integrity(context)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, SemanticProgramError> {
        canonical_payload_bytes(&self.payload)
    }
}

impl SemanticProgramRegistry {
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub fn programs(&self) -> &BTreeMap<ResponseProgramId, SemanticResponseProgram> {
        &self.programs
    }

    #[must_use]
    pub fn events(&self) -> &[SemanticProgramEvent] {
        &self.events
    }

    pub fn commit(
        &mut self,
        expected_registry_version: u64,
        payload: SemanticResponseProgramPayload,
        context: SemanticValidationContext,
    ) -> Result<SemanticProgramTransition, SemanticProgramError> {
        if self.version != expected_registry_version {
            return Err(SemanticProgramError::RegistryVersionConflict {
                expected: expected_registry_version,
                actual: self.version,
            });
        }

        let program = SemanticResponseProgram::validate(payload, context)?;
        let mut candidate = self.clone();
        let transition = candidate.apply_program(program)?;
        *self = candidate;
        Ok(transition)
    }

    pub fn replay(events: &[SemanticProgramEvent]) -> Result<Self, SemanticProgramError> {
        let mut registry = Self::default();
        for event in events {
            let expected_version = registry
                .version
                .checked_add(1)
                .ok_or(SemanticProgramError::RegistryVersionOverflow)?;
            if event.registry_version != expected_version {
                return Err(SemanticProgramError::ReplayVersionMismatch);
            }
            event.program.verify_replay_integrity()?;
            let transition = registry.apply_program(event.program.clone())?;
            if transition.event != *event {
                return Err(SemanticProgramError::ReplayVersionMismatch);
            }
        }
        Ok(registry)
    }

    fn apply_program(
        &mut self,
        program: SemanticResponseProgram,
    ) -> Result<SemanticProgramTransition, SemanticProgramError> {
        if self.programs.contains_key(&program.payload.id) {
            return Err(SemanticProgramError::DuplicateProgram);
        }
        let next_version = self
            .version
            .checked_add(1)
            .ok_or(SemanticProgramError::RegistryVersionOverflow)?;
        let event = SemanticProgramEvent {
            registry_version: next_version,
            program: program.clone(),
        };
        self.programs.insert(program.payload.id, program);
        self.events.push(event.clone());
        self.version = next_version;
        Ok(SemanticProgramTransition {
            registry_version: next_version,
            event,
        })
    }
}

fn validate_payload(
    payload: &SemanticResponseProgramPayload,
    context: SemanticValidationContext,
) -> Result<(), SemanticProgramError> {
    if payload.id.0 == 0 {
        return Err(SemanticProgramError::EmptyProgramId);
    }
    if payload.source_state_version.0 == 0 {
        return Err(SemanticProgramError::EmptySourceStateVersion);
    }
    if payload.companion_state_version == Some(CompanionStateVersion(0)) {
        return Err(SemanticProgramError::EmptyCompanionStateVersion);
    }
    if payload.subject_scope.0 == 0 {
        return Err(SemanticProgramError::EmptySubjectScope);
    }
    if payload.source_state_version != context.cognitive_state_version {
        return Err(SemanticProgramError::StaleCognitiveStateVersion {
            expected: context.cognitive_state_version.0,
            actual: payload.source_state_version.0,
        });
    }
    if payload.companion_state_version != context.companion_state_version {
        return Err(SemanticProgramError::CompanionStateVersionMismatch);
    }
    if payload.subject_scope != context.subject_scope {
        return Err(SemanticProgramError::SubjectScopeMismatch);
    }
    if payload.operations.is_empty() {
        return Err(SemanticProgramError::EmptyOperations);
    }

    validate_budgets(payload)?;
    validate_style(payload.style)?;
    validate_claim_collections(payload)?;
    let authorized = authorized_claim_map(payload);
    validate_epistemic_constraints(payload, &authorized)?;
    validate_operations(payload, &authorized)?;
    validate_sensitivity(payload)?;
    Ok(())
}

fn validate_budgets(payload: &SemanticResponseProgramPayload) -> Result<(), SemanticProgramError> {
    let output = payload.output_budget;
    if output.maximum_characters < 32
        || output.maximum_characters > 32_768
        || output.maximum_sentences == 0
        || output.maximum_sentences > 128
    {
        return Err(SemanticProgramError::InvalidOutputBudget);
    }

    let compute = payload.compute_budget;
    let claim_count = payload
        .required_claims
        .len()
        .saturating_add(payload.optional_claims.len())
        .saturating_add(payload.prohibited_claims.len());
    if compute.maximum_operations == 0
        || compute.maximum_claims == 0
        || compute.maximum_verification_steps == 0
        || payload.operations.len() > usize::from(compute.maximum_operations)
        || claim_count > usize::from(compute.maximum_claims)
    {
        return Err(SemanticProgramError::InvalidComputeBudget);
    }
    Ok(())
}

fn validate_style(style: StyleEnvelope) -> Result<(), SemanticProgramError> {
    if style.maximum_paragraphs == 0 || style.maximum_paragraphs > 16 {
        return Err(SemanticProgramError::InvalidStyleEnvelope);
    }
    if style.dialogue == DialogueMode::QuestionLed && !style.allow_questions {
        return Err(SemanticProgramError::InvalidStyleEnvelope);
    }
    Ok(())
}

fn validate_claim_collections(
    payload: &SemanticResponseProgramPayload,
) -> Result<(), SemanticProgramError> {
    validate_authorized_claim_order(&payload.required_claims)?;
    validate_authorized_claim_order(&payload.optional_claims)?;
    validate_prohibited_claim_order(&payload.prohibited_claims)?;

    let required_ids = payload
        .required_claims
        .iter()
        .map(|claim| claim.id)
        .collect::<BTreeSet<_>>();
    let optional_ids = payload
        .optional_claims
        .iter()
        .map(|claim| claim.id)
        .collect::<BTreeSet<_>>();
    let prohibited_ids = payload
        .prohibited_claims
        .iter()
        .map(|claim| claim.id)
        .collect::<BTreeSet<_>>();

    if !required_ids.is_disjoint(&optional_ids) {
        return Err(SemanticProgramError::AuthorizedClaimOverlap);
    }
    if !required_ids.is_disjoint(&prohibited_ids) || !optional_ids.is_disjoint(&prohibited_ids) {
        return Err(SemanticProgramError::AuthorizedProhibitedOverlap);
    }

    let required_keys = payload
        .required_claims
        .iter()
        .map(|claim| claim.semantic_key.as_str())
        .collect::<BTreeSet<_>>();
    let optional_keys = payload
        .optional_claims
        .iter()
        .map(|claim| claim.semantic_key.as_str())
        .collect::<BTreeSet<_>>();
    let prohibited_keys = payload
        .prohibited_claims
        .iter()
        .map(|claim| claim.semantic_key.as_str())
        .collect::<BTreeSet<_>>();

    if required_keys.len() != payload.required_claims.len()
        || optional_keys.len() != payload.optional_claims.len()
        || !required_keys.is_disjoint(&optional_keys)
    {
        return Err(SemanticProgramError::DuplicateSemanticKey);
    }
    if !required_keys.is_disjoint(&prohibited_keys)
        || !optional_keys.is_disjoint(&prohibited_keys)
    {
        return Err(SemanticProgramError::AuthorizedProhibitedOverlap);
    }
    Ok(())
}

fn validate_authorized_claim_order(
    claims: &[AuthorizedClaim],
) -> Result<(), SemanticProgramError> {
    let mut previous = 0_u64;
    for claim in claims {
        if claim.id.0 == 0 || claim.id.0 <= previous {
            return Err(SemanticProgramError::NoncanonicalClaimOrder);
        }
        validate_semantic_key(&claim.semantic_key)?;
        if claim.confidence_bps > 10_000
            || !claim.epistemic_status.permits(claim.confidence_bps)
        {
            return Err(SemanticProgramError::InvalidClaimConfidence);
        }
        previous = claim.id.0;
    }
    Ok(())
}

fn validate_prohibited_claim_order(
    claims: &[ProhibitedClaim],
) -> Result<(), SemanticProgramError> {
    let mut previous = 0_u64;
    let mut keys = BTreeSet::new();
    for claim in claims {
        if claim.id.0 == 0 || claim.id.0 <= previous {
            return Err(SemanticProgramError::NoncanonicalClaimOrder);
        }
        validate_semantic_key(&claim.semantic_key)?;
        if !keys.insert(claim.semantic_key.as_str()) {
            return Err(SemanticProgramError::DuplicateSemanticKey);
        }
        previous = claim.id.0;
    }
    Ok(())
}

fn validate_semantic_key(key: &str) -> Result<(), SemanticProgramError> {
    if key.is_empty()
        || key.len() > 160
        || !key.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b':' | b'.')
        })
    {
        return Err(SemanticProgramError::InvalidSemanticKey);
    }
    Ok(())
}

fn authorized_claim_map(
    payload: &SemanticResponseProgramPayload,
) -> BTreeMap<ClaimId, &AuthorizedClaim> {
    payload
        .required_claims
        .iter()
        .chain(payload.optional_claims.iter())
        .map(|claim| (claim.id, claim))
        .collect()
}

fn validate_epistemic_constraints(
    payload: &SemanticResponseProgramPayload,
    authorized: &BTreeMap<ClaimId, &AuthorizedClaim>,
) -> Result<(), SemanticProgramError> {
    let mut previous = 0_u64;
    for constraint in &payload.epistemic_constraints {
        if constraint.claim.0 == 0 || constraint.claim.0 <= previous {
            return Err(SemanticProgramError::NoncanonicalEpistemicOrder);
        }
        let Some(claim) = authorized.get(&constraint.claim) else {
            return Err(SemanticProgramError::InvalidEpistemicConstraint);
        };
        let (status_minimum, status_maximum) = constraint.required_status.confidence_bounds();
        if constraint.minimum_confidence_bps > constraint.maximum_confidence_bps
            || constraint.minimum_confidence_bps < status_minimum
            || constraint.maximum_confidence_bps > status_maximum
            || claim.epistemic_status != constraint.required_status
            || claim.confidence_bps < constraint.minimum_confidence_bps
            || claim.confidence_bps > constraint.maximum_confidence_bps
        {
            return Err(SemanticProgramError::InvalidEpistemicConstraint);
        }
        previous = constraint.claim.0;
    }

    if payload.epistemic_constraints.len() != authorized.len()
        || payload
            .epistemic_constraints
            .iter()
            .any(|constraint| !authorized.contains_key(&constraint.claim))
    {
        return Err(SemanticProgramError::InvalidEpistemicConstraint);
    }
    Ok(())
}

fn validate_operations(
    payload: &SemanticResponseProgramPayload,
    authorized: &BTreeMap<ClaimId, &AuthorizedClaim>,
) -> Result<(), SemanticProgramError> {
    let prohibited = payload
        .prohibited_claims
        .iter()
        .map(|claim| claim.id)
        .collect::<BTreeSet<_>>();
    let mut represented = BTreeSet::new();

    for (index, operation) in payload.operations.iter().enumerate() {
        let expected_id = index as u64 + 1;
        if operation.id.0 != expected_id {
            return Err(SemanticProgramError::NoncanonicalOperationOrder);
        }
        match &operation.kind {
            DiscourseOperationKind::Assert(claim) => {
                require_authorized(*claim, authorized, &prohibited)?;
                represented.insert(*claim);
            }
            DiscourseOperationKind::Qualify { claim, status } => {
                let authorized_claim = require_authorized(*claim, authorized, &prohibited)?;
                if authorized_claim.epistemic_status != *status {
                    return Err(SemanticProgramError::InvalidEpistemicConstraint);
                }
                represented.insert(*claim);
            }
            DiscourseOperationKind::Contrast { left, right }
            | DiscourseOperationKind::Correct {
                prior: left,
                replacement: right,
            } => {
                if left == right {
                    return Err(SemanticProgramError::DegenerateBinaryOperation);
                }
                require_authorized(*left, authorized, &prohibited)?;
                require_authorized(*right, authorized, &prohibited)?;
                represented.insert(*left);
                represented.insert(*right);
            }
            DiscourseOperationKind::Explain { claims } => {
                if claims.is_empty()
                    || claims.iter().any(|claim| claim.0 == 0)
                    || claims.windows(2).any(|pair| pair[0] >= pair[1])
                {
                    return Err(SemanticProgramError::InvalidExplanationClaims);
                }
                for claim in claims {
                    require_authorized(*claim, authorized, &prohibited)?;
                    represented.insert(*claim);
                }
            }
            DiscourseOperationKind::Acknowledge(observation) => {
                if observation.0 == 0 {
                    return Err(SemanticProgramError::UnknownClaimReference);
                }
            }
            DiscourseOperationKind::RequestEvidence(variable) => {
                if variable.0 == 0 {
                    return Err(SemanticProgramError::UnknownClaimReference);
                }
            }
            DiscourseOperationKind::Commit(prediction) => {
                if prediction.0 == 0 {
                    return Err(SemanticProgramError::UnknownClaimReference);
                }
            }
            DiscourseOperationKind::Abstain(_) => {}
        }
    }

    if payload
        .required_claims
        .iter()
        .any(|claim| !represented.contains(&claim.id))
    {
        return Err(SemanticProgramError::UnrepresentedRequiredClaim);
    }
    Ok(())
}

fn require_authorized<'a>(
    claim: ClaimId,
    authorized: &'a BTreeMap<ClaimId, &AuthorizedClaim>,
    prohibited: &BTreeSet<ClaimId>,
) -> Result<&'a AuthorizedClaim, SemanticProgramError> {
    if claim.0 == 0 || prohibited.contains(&claim) {
        return Err(SemanticProgramError::UnknownClaimReference);
    }
    authorized
        .get(&claim)
        .copied()
        .ok_or(SemanticProgramError::UnknownClaimReference)
}

fn validate_sensitivity(
    payload: &SemanticResponseProgramPayload,
) -> Result<(), SemanticProgramError> {
    if payload.sensitivity.disclosure_scope.0 == 0
        || payload.sensitivity.disclosure_scope != payload.subject_scope
    {
        return Err(SemanticProgramError::SensitivityViolation);
    }

    for claim in payload
        .required_claims
        .iter()
        .chain(payload.optional_claims.iter())
    {
        if claim.sensitivity > payload.sensitivity.maximum_disclosure
            || claim.disclosure_scope != payload.subject_scope
        {
            return Err(SemanticProgramError::SensitivityViolation);
        }
    }
    Ok(())
}

fn canonical_payload_bytes(
    payload: &SemanticResponseProgramPayload,
) -> Result<Vec<u8>, SemanticProgramError> {
    serde_json::to_vec(payload)
        .map_err(|error| SemanticProgramError::CanonicalSerialization(error.to_string()))
}

fn digest_payload(
    payload: &SemanticResponseProgramPayload,
) -> Result<ResponseProgramDigest, SemanticProgramError> {
    let encoded = canonical_payload_bytes(payload)?;
    let mut digest = fnv1a64(DIGEST_DOMAIN);
    digest = mix_u64(digest, encoded.len() as u64);
    for byte in encoded {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    Ok(ResponseProgramDigest(digest))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf29ce484222325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

fn mix_u64(mut digest: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(
        id: u64,
        key: &str,
        confidence_bps: u16,
        status: EpistemicStatus,
        sensitivity: SensitivityLevel,
    ) -> AuthorizedClaim {
        AuthorizedClaim {
            id: ClaimId(id),
            semantic_key: key.to_owned(),
            polarity: ClaimPolarity::Positive,
            confidence_bps,
            epistemic_status: status,
            sensitivity,
            disclosure_scope: SubjectScope(77),
        }
    }

    fn fixture_payload(program_id: u64, source_version: u64) -> SemanticResponseProgramPayload {
        let required_claims = vec![
            claim(
                1,
                "wrapper_risk_valid",
                9_500,
                EpistemicStatus::Certain,
                SensitivityLevel::Public,
            ),
            claim(
                2,
                "unrestricted_llm_owns_cognition",
                8_400,
                EpistemicStatus::Probable,
                SensitivityLevel::Public,
            ),
            claim(
                3,
                "constrained_renderer_preserves_authority",
                7_800,
                EpistemicStatus::Probable,
                SensitivityLevel::Public,
            ),
            claim(
                4,
                "current_starfire_semantics_limited",
                9_200,
                EpistemicStatus::Certain,
                SensitivityLevel::Personal,
            ),
        ];
        let optional_claims = vec![
            claim(
                5,
                "prior_wrapper_equivalence",
                5_000,
                EpistemicStatus::Possible,
                SensitivityLevel::Public,
            ),
            claim(
                6,
                "renderer_is_surface_realizer",
                8_100,
                EpistemicStatus::Probable,
                SensitivityLevel::Public,
            ),
            claim(
                7,
                "semantic_boundary_enables_attribution",
                7_400,
                EpistemicStatus::Probable,
                SensitivityLevel::Public,
            ),
        ];
        let epistemic_constraints = required_claims
            .iter()
            .chain(optional_claims.iter())
            .map(|claim| {
                let (minimum, maximum) = claim.epistemic_status.confidence_bounds();
                EpistemicConstraint {
                    claim: claim.id,
                    required_status: claim.epistemic_status,
                    minimum_confidence_bps: minimum,
                    maximum_confidence_bps: maximum,
                }
            })
            .collect();

        SemanticResponseProgramPayload {
            id: ResponseProgramId(program_id),
            source_state_version: CognitiveStateVersion(source_version),
            companion_state_version: Some(CompanionStateVersion(12)),
            subject_scope: SubjectScope(77),
            intent: SemanticResponseIntent::Contrast,
            operations: vec![
                DiscourseOperation {
                    id: OperationId(1),
                    kind: DiscourseOperationKind::Acknowledge(ObservationId(901)),
                },
                DiscourseOperation {
                    id: OperationId(2),
                    kind: DiscourseOperationKind::Assert(ClaimId(1)),
                },
                DiscourseOperation {
                    id: OperationId(3),
                    kind: DiscourseOperationKind::Qualify {
                        claim: ClaimId(4),
                        status: EpistemicStatus::Certain,
                    },
                },
                DiscourseOperation {
                    id: OperationId(4),
                    kind: DiscourseOperationKind::Contrast {
                        left: ClaimId(2),
                        right: ClaimId(3),
                    },
                },
                DiscourseOperation {
                    id: OperationId(5),
                    kind: DiscourseOperationKind::Correct {
                        prior: ClaimId(5),
                        replacement: ClaimId(6),
                    },
                },
                DiscourseOperation {
                    id: OperationId(6),
                    kind: DiscourseOperationKind::Explain {
                        claims: vec![ClaimId(2), ClaimId(3), ClaimId(7)],
                    },
                },
                DiscourseOperation {
                    id: OperationId(7),
                    kind: DiscourseOperationKind::RequestEvidence(MissingVariableId(33)),
                },
                DiscourseOperation {
                    id: OperationId(8),
                    kind: DiscourseOperationKind::Commit(PredictionId(44)),
                },
                DiscourseOperation {
                    id: OperationId(9),
                    kind: DiscourseOperationKind::Abstain(
                        AbstentionReason::InsufficientEvidence,
                    ),
                },
            ],
            required_claims,
            optional_claims,
            prohibited_claims: vec![
                ProhibitedClaim {
                    id: ClaimId(100),
                    semantic_key: "starfire_has_general_language_understanding".to_owned(),
                },
                ProhibitedClaim {
                    id: ClaimId(101),
                    semantic_key: "fluency_proves_cognition".to_owned(),
                },
            ],
            epistemic_constraints,
            sensitivity: SensitivityPolicy {
                maximum_disclosure: SensitivityLevel::Personal,
                disclosure_scope: SubjectScope(77),
            },
            style: StyleEnvelope {
                detail: DetailLevel::Detailed,
                vocabulary: VocabularyLevel::Technical,
                dialogue: DialogueMode::Collaborative,
                acknowledgment: AcknowledgmentLevel::Brief,
                allow_first_person: true,
                allow_questions: true,
                maximum_paragraphs: 6,
            },
            output_budget: OutputBudget {
                maximum_characters: 1_200,
                maximum_sentences: 20,
            },
            compute_budget: ComputeBudget {
                maximum_operations: 16,
                maximum_claims: 16,
                maximum_verification_steps: 128,
            },
        }
    }

    fn context(source_version: u64) -> SemanticValidationContext {
        SemanticValidationContext {
            cognitive_state_version: CognitiveStateVersion(source_version),
            companion_state_version: Some(CompanionStateVersion(12)),
            subject_scope: SubjectScope(77),
        }
    }

    #[test]
    fn valid_program_is_deterministic_and_replayable() {
        let payload = fixture_payload(1, 41);
        let first = SemanticResponseProgram::validate(payload.clone(), context(41)).unwrap();
        let second = SemanticResponseProgram::validate(payload, context(41)).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.canonical_bytes().unwrap(), second.canonical_bytes().unwrap());

        let mut registry = SemanticProgramRegistry::default();
        registry.commit(0, first.payload.clone(), context(41)).unwrap();
        let replayed = SemanticProgramRegistry::replay(registry.events()).unwrap();
        assert_eq!(registry, replayed);
    }

    #[test]
    fn stale_context_is_rejected_atomically() {
        let mut registry = SemanticProgramRegistry::default();
        let before = registry.clone();
        let error = registry
            .commit(0, fixture_payload(1, 41), context(42))
            .unwrap_err();
        assert!(matches!(
            error,
            SemanticProgramError::StaleCognitiveStateVersion { .. }
        ));
        assert_eq!(registry, before);
    }

    #[test]
    fn unknown_claim_reference_is_rejected() {
        let mut payload = fixture_payload(1, 41);
        payload.operations[1].kind = DiscourseOperationKind::Assert(ClaimId(999));
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::UnknownClaimReference
        );
    }

    #[test]
    fn authorized_and_prohibited_overlap_is_rejected() {
        let mut payload = fixture_payload(1, 41);
        payload.prohibited_claims[0].semantic_key =
            payload.required_claims[0].semantic_key.clone();
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::AuthorizedProhibitedOverlap
        );
    }

    #[test]
    fn confidence_and_qualifier_must_match_status() {
        let mut payload = fixture_payload(1, 41);
        payload.required_claims[0].confidence_bps = 7_500;
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::InvalidClaimConfidence
        );

        let mut payload = fixture_payload(1, 41);
        payload.operations[2].kind = DiscourseOperationKind::Qualify {
            claim: ClaimId(4),
            status: EpistemicStatus::Possible,
        };
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::InvalidEpistemicConstraint
        );
    }

    #[test]
    fn noncanonical_order_is_rejected() {
        let mut payload = fixture_payload(1, 41);
        payload.required_claims.swap(0, 1);
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::NoncanonicalClaimOrder
        );

        let mut payload = fixture_payload(1, 41);
        payload.operations[1].id = OperationId(1);
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::NoncanonicalOperationOrder
        );
    }

    #[test]
    fn sensitivity_scope_is_enforced() {
        let mut payload = fixture_payload(1, 41);
        payload.required_claims[3].disclosure_scope = SubjectScope(88);
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::SensitivityViolation
        );

        let mut payload = fixture_payload(1, 41);
        payload.sensitivity.maximum_disclosure = SensitivityLevel::Public;
        assert_eq!(
            SemanticResponseProgram::validate(payload, context(41)).unwrap_err(),
            SemanticProgramError::SensitivityViolation
        );
    }

    #[test]
    fn registry_version_and_duplicate_program_are_enforced_atomically() {
        let mut registry = SemanticProgramRegistry::default();
        registry
            .commit(0, fixture_payload(1, 41), context(41))
            .unwrap();

        let before = registry.clone();
        assert!(matches!(
            registry.commit(0, fixture_payload(2, 42), context(42)),
            Err(SemanticProgramError::RegistryVersionConflict { .. })
        ));
        assert_eq!(registry, before);

        assert_eq!(
            registry
                .commit(1, fixture_payload(1, 41), context(41))
                .unwrap_err(),
            SemanticProgramError::DuplicateProgram
        );
        assert_eq!(registry, before);
    }

    #[test]
    fn digest_tampering_and_event_reordering_are_rejected() {
        let mut registry = SemanticProgramRegistry::default();
        registry
            .commit(0, fixture_payload(1, 41), context(41))
            .unwrap();
        registry
            .commit(1, fixture_payload(2, 42), context(42))
            .unwrap();

        let mut tampered = registry.events().to_vec();
        tampered[0].program.digest.0 ^= 1;
        assert_eq!(
            SemanticProgramRegistry::replay(&tampered).unwrap_err(),
            SemanticProgramError::DigestMismatch
        );

        let mut reordered = registry.events().to_vec();
        reordered.swap(0, 1);
        assert_eq!(
            SemanticProgramRegistry::replay(&reordered).unwrap_err(),
            SemanticProgramError::ReplayVersionMismatch
        );

        let deleted = vec![registry.events()[1].clone()];
        assert_eq!(
            SemanticProgramRegistry::replay(&deleted).unwrap_err(),
            SemanticProgramError::ReplayVersionMismatch
        );
    }

    #[test]
    fn every_authority_flag_is_closed() {
        let boundary = authority_boundary();
        assert!(!boundary.runtime_chat_wiring);
        assert!(!boundary.generated_text_influence);
        assert!(!boundary.persistence_authority);
        assert!(!boundary.routing_authority);
        assert!(!boundary.companion_mutation_authority);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.charge_discharge_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
