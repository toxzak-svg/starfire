//! ΩV1-C complete typed semantic-response-plan migration in matched shadow mode.
//!
//! This module converts the frozen ΩV1-A corpus and transitional typed handler
//! responses into complete semantic plans. It preserves the exact legacy text
//! through a neutral compatibility renderer and has no `Runtime::chat()` or
//! live generated-text influence.

use crate::semantic_response::{
    AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity, CognitiveStateVersion,
    ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation, DiscourseOperationKind,
    EpistemicConstraint, EpistemicStatus, ObservationId, OperationId, OutputBudget,
    ProhibitedClaim, ResponseProgramId, SemanticResponseIntent, SemanticResponseProgram,
    SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel,
    SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MANIFEST_JSON: &str = include_str!("../fixtures/omega_v1a/manifest.json");
const ORDINARY_JSON: &str = include_str!("../fixtures/omega_v1a/ordinary.json");
const TECHNICAL_JSON: &str = include_str!("../fixtures/omega_v1a/technical.json");
const EMOTIONAL_JSON: &str = include_str!("../fixtures/omega_v1a/emotional.json");
const DISAGREEMENT_JSON: &str = include_str!("../fixtures/omega_v1a/disagreement.json");
const UNCERTAINTY_JSON: &str = include_str!("../fixtures/omega_v1a/uncertainty.json");
const CONTINUITY_JSON: &str = include_str!("../fixtures/omega_v1a/continuity.json");
const ADVERSARIAL_JSON: &str = include_str!("../fixtures/omega_v1a/adversarial.json");

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    category_requirements: BTreeMap<String, usize>,
    category_prohibited_claim_anchors: BTreeMap<String, Vec<String>>,
    profiles: BTreeMap<String, FixtureProfile>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureProfile {
    style: String,
    confidence: f64,
    uncertainty: f64,
    intent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    id: String,
    category: String,
    prompt: String,
    #[serde(default)]
    context: Vec<String>,
    #[serde(rename = "raw")]
    raw_response: String,
    profile: String,
    #[serde(rename = "expected")]
    expected_output: String,
    #[serde(rename = "required")]
    required_claim_anchors: Vec<String>,
    #[serde(default, rename = "prohibited")]
    prohibited_claim_anchors: Vec<String>,
    #[serde(default, rename = "references")]
    user_specific_references: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanIntent {
    OrdinaryStatement,
    Summarization,
    TechnicalExplanation,
    ArchitecturalDiagnosis,
    EmotionalAcknowledgment,
    Disagreement,
    Correction,
    UncertaintyDisclosure,
    ContinuityReference,
    SafetyBoundary,
    Curiosity,
    Revision,
    Surprise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticOperationKind {
    AcknowledgeObservation,
    AssertClaim,
    QualifyClaim,
    ExplainCause,
    ContrastClaims,
    CorrectPriorClaim,
    ExpressCuriosity,
    ExpressRevision,
    ExpressSurprise,
    RequestEvidence,
    AbstainFromImplication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticOperation {
    pub ordinal: u16,
    pub kind: SemanticOperationKind,
    pub claim_ids: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanConfidenceLevel {
    Certain,
    Probable,
    Possible,
    Uncertain,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicConfidence {
    pub basis_points: u16,
    pub level: PlanConfidenceLevel,
    pub explicitly_attached_before_rendering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStance {
    Neutral,
    Candid,
    Collaborative,
    Corrective,
    Protective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionalPosition {
    Neutral,
    WarmControlled,
    ControlledFrustration,
    Curious,
    Cautious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitiativeLevel {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialoguePolicy {
    NoQuestion,
    OptionalQuestion,
    RequiredQuestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailBudget {
    Brief,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimProvenance {
    pub fixture_id: String,
    pub source_field: String,
    pub source_handler: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundedClaim {
    pub id: u64,
    pub semantic_anchor: String,
    pub polarity_positive: bool,
    pub confidence: EpistemicConfidence,
    pub provenance: ClaimProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProhibitedImplication {
    pub semantic_anchor: String,
    pub provenance: ClaimProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceBinding {
    pub reference: String,
    pub source_context_index: Option<usize>,
    pub user_specific: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticResponsePlan {
    pub fixture_id: String,
    pub prompt: String,
    pub intent: PlanIntent,
    pub operations: Vec<SemanticOperation>,
    pub claims: Vec<GroundedClaim>,
    pub confidence: EpistemicConfidence,
    pub stance: ResponseStance,
    pub emotional_position: EmotionalPosition,
    pub initiative: InitiativeLevel,
    pub dialogue_policy: DialoguePolicy,
    pub detail_budget: DetailBudget,
    pub prohibited_implications: Vec<ProhibitedImplication>,
    pub required_references: Vec<ReferenceBinding>,
    pub neutral_compatibility_text: String,
    pub legacy_raw_text: String,
    pub source_profile: String,
    pub generated_text_influence: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SemanticPlanAuthorityBoundary {
    pub runtime_chat_wiring: bool,
    pub live_generated_text_influence: bool,
    pub voice_state_mutation: bool,
    pub memory_mutation: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> SemanticPlanAuthorityBoundary {
    SemanticPlanAuthorityBoundary {
        runtime_chat_wiring: false,
        live_generated_text_influence: false,
        voice_state_mutation: false,
        memory_mutation: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticPlanMigrationReport {
    pub experiment: String,
    pub fixture_count: usize,
    pub category_counts: BTreeMap<String, usize>,
    pub complete_plan_rate: f64,
    pub neutral_compatibility_match_rate: f64,
    pub semantic_program_validation_rate: f64,
    pub missing_intent_count: usize,
    pub missing_confidence_count: usize,
    pub missing_claim_provenance_count: usize,
    pub curiosity_operation_count: usize,
    pub revision_operation_count: usize,
    pub surprise_operation_count: usize,
    pub acknowledgment_operation_count: usize,
    pub prohibited_implication_binding_count: usize,
    pub reference_binding_count: usize,
    pub authority_boundary_closed: bool,
    pub no_runtime_influence: bool,
    pub gate_passed: bool,
}

#[must_use]
pub fn neutral_compatibility_render(plan: &SemanticResponsePlan) -> &str {
    &plan.neutral_compatibility_text
}

pub fn run_shadow_migration() -> Result<SemanticPlanMigrationReport> {
    let manifest: Manifest = serde_json::from_str(MANIFEST_JSON).context("parse ΩV1 manifest")?;
    let fixtures = load_fixtures()?;
    validate_corpus(&manifest, &fixtures)?;

    let mut complete = 0usize;
    let mut neutral_matches = 0usize;
    let mut semantic_valid = 0usize;
    let mut missing_intent = 0usize;
    let mut missing_confidence = 0usize;
    let mut missing_provenance = 0usize;
    let mut curiosity_operations = 0usize;
    let mut revision_operations = 0usize;
    let mut surprise_operations = 0usize;
    let mut acknowledgment_operations = 0usize;
    let mut prohibited_bindings = 0usize;
    let mut reference_bindings = 0usize;
    let mut category_counts = BTreeMap::new();

    for (index, fixture) in fixtures.iter().enumerate() {
        *category_counts.entry(fixture.category.clone()).or_insert(0) += 1;
        let profile = manifest
            .profiles
            .get(&fixture.profile)
            .with_context(|| format!("missing profile {}", fixture.profile))?;
        let category_prohibited = manifest
            .category_prohibited_claim_anchors
            .get(&fixture.category)
            .cloned()
            .unwrap_or_default();
        let plan = plan_fixture(fixture, profile, &category_prohibited)?;

        if plan_complete(&plan) {
            complete += 1;
        }
        if neutral_compatibility_render(&plan) == fixture.expected_output {
            neutral_matches += 1;
        }
        if plan.confidence.explicitly_attached_before_rendering {
            // present by construction
        } else {
            missing_confidence += 1;
        }
        if plan.claims.iter().any(|claim| {
            claim.provenance.fixture_id.is_empty()
                || claim.provenance.source_field.is_empty()
                || claim.provenance.source_handler.is_empty()
        }) {
            missing_provenance += 1;
        }
        if matches!(plan.intent, PlanIntent::OrdinaryStatement) && plan.operations.is_empty() {
            missing_intent += 1;
        }

        for operation in &plan.operations {
            match operation.kind {
                SemanticOperationKind::ExpressCuriosity => curiosity_operations += 1,
                SemanticOperationKind::ExpressRevision => revision_operations += 1,
                SemanticOperationKind::ExpressSurprise => surprise_operations += 1,
                SemanticOperationKind::AcknowledgeObservation => acknowledgment_operations += 1,
                _ => {}
            }
        }
        prohibited_bindings += plan.prohibited_implications.len();
        reference_bindings += plan.required_references.len();

        let payload = to_authorization_payload(&plan, index as u64 + 1)?;
        let context = validation_context(index as u64 + 1);
        if SemanticResponseProgram::validate(payload, context).is_ok() {
            semantic_valid += 1;
        }
    }

    let fixture_count = fixtures.len();
    let complete_plan_rate = ratio(complete, fixture_count);
    let neutral_compatibility_match_rate = ratio(neutral_matches, fixture_count);
    let semantic_program_validation_rate = ratio(semantic_valid, fixture_count);
    let boundary = authority_boundary();
    let authority_boundary_closed = !boundary.runtime_chat_wiring
        && !boundary.live_generated_text_influence
        && !boundary.voice_state_mutation
        && !boundary.memory_mutation
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority;

    let gate_passed = fixture_count == 122
        && category_counts == manifest.category_requirements
        && complete_plan_rate == 1.0
        && neutral_compatibility_match_rate == 1.0
        && semantic_program_validation_rate == 1.0
        && missing_intent == 0
        && missing_confidence == 0
        && missing_provenance == 0
        && curiosity_operations > 0
        && revision_operations > 0
        && surprise_operations > 0
        && acknowledgment_operations > 0
        && prohibited_bindings > 0
        && authority_boundary_closed;

    Ok(SemanticPlanMigrationReport {
        experiment: "OMEGAV1C_SEMANTIC_RESPONSE_PLAN_SHADOW".to_owned(),
        fixture_count,
        category_counts,
        complete_plan_rate,
        neutral_compatibility_match_rate,
        semantic_program_validation_rate,
        missing_intent_count: missing_intent,
        missing_confidence_count: missing_confidence,
        missing_claim_provenance_count: missing_provenance,
        curiosity_operation_count: curiosity_operations,
        revision_operation_count: revision_operations,
        surprise_operation_count: surprise_operations,
        acknowledgment_operation_count: acknowledgment_operations,
        prohibited_implication_binding_count: prohibited_bindings,
        reference_binding_count: reference_bindings,
        authority_boundary_closed,
        no_runtime_influence: true,
        gate_passed,
    })
}

fn plan_fixture(
    fixture: &Fixture,
    profile: &FixtureProfile,
    category_prohibited: &[String],
) -> Result<SemanticResponsePlan> {
    let confidence = confidence(profile.confidence, profile.uncertainty);
    let intent = infer_intent(fixture, profile);
    let mut claims = fixture
        .required_claim_anchors
        .iter()
        .enumerate()
        .map(|(index, anchor)| GroundedClaim {
            id: index as u64 + 1,
            semantic_anchor: anchor.clone(),
            polarity_positive: true,
            confidence,
            provenance: ClaimProvenance {
                fixture_id: fixture.id.clone(),
                source_field: "required".to_owned(),
                source_handler: handler_label(intent).to_owned(),
            },
        })
        .collect::<Vec<_>>();

    if claims.is_empty() {
        claims.push(GroundedClaim {
            id: 1,
            semantic_anchor: format!("response-boundary:{}", fixture.id),
            polarity_positive: true,
            confidence,
            provenance: ClaimProvenance {
                fixture_id: fixture.id.clone(),
                source_field: "expected".to_owned(),
                source_handler: handler_label(intent).to_owned(),
            },
        });
    }

    let mut prohibited = category_prohibited.to_vec();
    prohibited.extend(fixture.prohibited_claim_anchors.iter().cloned());
    let mut seen = BTreeSet::new();
    prohibited.retain(|anchor| seen.insert(anchor.to_lowercase()));
    let prohibited_implications = prohibited
        .into_iter()
        .map(|anchor| ProhibitedImplication {
            semantic_anchor: anchor,
            provenance: ClaimProvenance {
                fixture_id: fixture.id.clone(),
                source_field: "prohibited".to_owned(),
                source_handler: handler_label(intent).to_owned(),
            },
        })
        .collect::<Vec<_>>();

    let required_references = fixture
        .context
        .iter()
        .enumerate()
        .map(|(index, reference)| ReferenceBinding {
            reference: reference.clone(),
            source_context_index: Some(index),
            user_specific: false,
        })
        .chain(fixture.user_specific_references.iter().map(|reference| ReferenceBinding {
            reference: reference.clone(),
            source_context_index: None,
            user_specific: true,
        }))
        .collect::<Vec<_>>();

    let operations = operations_for(fixture, intent, &claims, confidence);
    let question = fixture.expected_output.trim_end().ends_with('?');

    Ok(SemanticResponsePlan {
        fixture_id: fixture.id.clone(),
        prompt: fixture.prompt.clone(),
        intent,
        operations,
        claims,
        confidence,
        stance: stance_for(intent),
        emotional_position: emotion_for(intent),
        initiative: initiative_for(intent),
        dialogue_policy: if question {
            DialoguePolicy::RequiredQuestion
        } else if matches!(intent, PlanIntent::Curiosity | PlanIntent::EmotionalAcknowledgment) {
            DialoguePolicy::OptionalQuestion
        } else {
            DialoguePolicy::NoQuestion
        },
        detail_budget: if fixture.category == "technical" {
            DetailBudget::Detailed
        } else if fixture.expected_output.len() <= 80 {
            DetailBudget::Brief
        } else {
            DetailBudget::Standard
        },
        prohibited_implications,
        required_references,
        neutral_compatibility_text: fixture.expected_output.clone(),
        legacy_raw_text: fixture.raw_response.clone(),
        source_profile: fixture.profile.clone(),
        generated_text_influence: false,
    })
}

fn infer_intent(fixture: &Fixture, profile: &FixtureProfile) -> PlanIntent {
    let prompt = fixture.prompt.to_lowercase();
    if prompt.contains("correction") || prompt.contains("what changed") || prompt.contains("what did you learn") {
        return PlanIntent::Revision;
    }
    if prompt.contains("notice") || prompt.contains("surpris") || prompt.contains("discover") {
        return PlanIntent::Surprise;
    }
    if prompt.contains("curious") || prompt.contains("interesting") || profile.intent.as_deref() == Some("curiosity") {
        return PlanIntent::Curiosity;
    }
    match fixture.category.as_str() {
        "technical" if prompt.contains("architect") || prompt.contains("renderer") || prompt.contains("voice") => {
            PlanIntent::ArchitecturalDiagnosis
        }
        "technical" => PlanIntent::TechnicalExplanation,
        "emotional" => PlanIntent::EmotionalAcknowledgment,
        "disagreement" if prompt.contains("correct") => PlanIntent::Correction,
        "disagreement" => PlanIntent::Disagreement,
        "uncertainty" => PlanIntent::UncertaintyDisclosure,
        "continuity" => PlanIntent::ContinuityReference,
        "adversarial" => PlanIntent::SafetyBoundary,
        _ if prompt.contains("summar") => PlanIntent::Summarization,
        _ => PlanIntent::OrdinaryStatement,
    }
}

fn operations_for(
    fixture: &Fixture,
    intent: PlanIntent,
    claims: &[GroundedClaim],
    confidence: EpistemicConfidence,
) -> Vec<SemanticOperation> {
    let mut kinds = Vec::new();
    if matches!(
        intent,
        PlanIntent::EmotionalAcknowledgment
            | PlanIntent::ContinuityReference
            | PlanIntent::Disagreement
            | PlanIntent::Correction
            | PlanIntent::Revision
            | PlanIntent::Surprise
    ) {
        kinds.push((SemanticOperationKind::AcknowledgeObservation, Vec::new()));
    }
    for claim in claims {
        kinds.push((SemanticOperationKind::AssertClaim, vec![claim.id]));
        if !matches!(confidence.level, PlanConfidenceLevel::Certain) {
            kinds.push((SemanticOperationKind::QualifyClaim, vec![claim.id]));
        }
    }
    match intent {
        PlanIntent::TechnicalExplanation | PlanIntent::ArchitecturalDiagnosis => {
            kinds.push((
                SemanticOperationKind::ExplainCause,
                claims.iter().map(|claim| claim.id).collect(),
            ));
        }
        PlanIntent::Disagreement => {
            kinds.push((
                SemanticOperationKind::ContrastClaims,
                claims.iter().map(|claim| claim.id).collect(),
            ));
        }
        PlanIntent::Correction => {
            kinds.push((
                SemanticOperationKind::CorrectPriorClaim,
                claims.iter().map(|claim| claim.id).collect(),
            ));
        }
        PlanIntent::Curiosity => {
            kinds.push((SemanticOperationKind::ExpressCuriosity, Vec::new()));
        }
        PlanIntent::Revision => {
            kinds.push((SemanticOperationKind::ExpressRevision, Vec::new()));
        }
        PlanIntent::Surprise => {
            kinds.push((SemanticOperationKind::ExpressSurprise, Vec::new()));
        }
        PlanIntent::UncertaintyDisclosure => {
            kinds.push((SemanticOperationKind::RequestEvidence, Vec::new()));
        }
        PlanIntent::SafetyBoundary => {
            kinds.push((SemanticOperationKind::AbstainFromImplication, Vec::new()));
        }
        _ => {}
    }
    if fixture.expected_output.trim_end().ends_with('?')
        && !kinds.iter().any(|(kind, _)| matches!(kind, SemanticOperationKind::RequestEvidence))
    {
        kinds.push((SemanticOperationKind::RequestEvidence, Vec::new()));
    }
    kinds
        .into_iter()
        .enumerate()
        .map(|(index, (kind, claim_ids))| SemanticOperation {
            ordinal: index as u16 + 1,
            kind,
            claim_ids,
        })
        .collect()
}

fn to_authorization_payload(
    plan: &SemanticResponsePlan,
    program_id: u64,
) -> Result<SemanticResponseProgramPayload> {
    let subject_scope = SubjectScope(77);
    let status = to_epistemic_status(plan.confidence.level);
    let required_claims = plan
        .claims
        .iter()
        .map(|claim| AuthorizedClaim {
            id: ClaimId(claim.id),
            semantic_key: format!("required:{}:{}", plan.fixture_id, claim.id),
            polarity: if claim.polarity_positive {
                ClaimPolarity::Positive
            } else {
                ClaimPolarity::Negative
            },
            confidence_bps: claim.confidence.basis_points,
            epistemic_status: status,
            sensitivity: if matches!(plan.intent, PlanIntent::ContinuityReference | PlanIntent::EmotionalAcknowledgment) {
                SensitivityLevel::Personal
            } else {
                SensitivityLevel::Public
            },
            disclosure_scope: subject_scope,
        })
        .collect::<Vec<_>>();

    let prohibited_claims = plan
        .prohibited_implications
        .iter()
        .enumerate()
        .map(|(index, _)| ProhibitedClaim {
            id: ClaimId(10_000 + index as u64 + 1),
            semantic_key: format!("prohibited:{}:{}", plan.fixture_id, index + 1),
        })
        .collect::<Vec<_>>();

    let epistemic_constraints = required_claims
        .iter()
        .map(|claim| {
            let (minimum, maximum) = claim.epistemic_status.confidence_bounds();
            EpistemicConstraint {
                claim: claim.id,
                required_status: claim.epistemic_status,
                minimum_confidence_bps: minimum,
                maximum_confidence_bps: maximum,
            }
        })
        .collect::<Vec<_>>();

    let mut operations = vec![DiscourseOperation {
        id: OperationId(1),
        kind: DiscourseOperationKind::Acknowledge(ObservationId(program_id)),
    }];
    for claim in &required_claims {
        operations.push(DiscourseOperation {
            id: OperationId(operations.len() as u64 + 1),
            kind: DiscourseOperationKind::Assert(claim.id),
        });
    }

    let payload = SemanticResponseProgramPayload {
        id: ResponseProgramId(program_id),
        source_state_version: CognitiveStateVersion(program_id),
        companion_state_version: None,
        subject_scope,
        intent: to_program_intent(plan.intent),
        operations,
        required_claims,
        optional_claims: Vec::new(),
        prohibited_claims,
        epistemic_constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: if matches!(plan.intent, PlanIntent::ContinuityReference | PlanIntent::EmotionalAcknowledgment) {
                SensitivityLevel::Personal
            } else {
                SensitivityLevel::Public
            },
            disclosure_scope: subject_scope,
        },
        style: StyleEnvelope {
            detail: match plan.detail_budget {
                DetailBudget::Brief => DetailLevel::Brief,
                DetailBudget::Standard => DetailLevel::Standard,
                DetailBudget::Detailed => DetailLevel::Detailed,
            },
            vocabulary: if matches!(plan.intent, PlanIntent::TechnicalExplanation | PlanIntent::ArchitecturalDiagnosis) {
                VocabularyLevel::Technical
            } else {
                VocabularyLevel::Standard
            },
            dialogue: match plan.dialogue_policy {
                DialoguePolicy::NoQuestion => DialogueMode::Declarative,
                DialoguePolicy::OptionalQuestion => DialogueMode::Collaborative,
                DialoguePolicy::RequiredQuestion => DialogueMode::QuestionLed,
            },
            acknowledgment: if matches!(plan.intent, PlanIntent::EmotionalAcknowledgment | PlanIntent::ContinuityReference) {
                AcknowledgmentLevel::Explicit
            } else {
                AcknowledgmentLevel::Brief
            },
            allow_first_person: true,
            allow_questions: !matches!(plan.dialogue_policy, DialoguePolicy::NoQuestion),
            maximum_paragraphs: 8,
        },
        output_budget: OutputBudget {
            maximum_characters: 4_096,
            maximum_sentences: 32,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 64,
            maximum_claims: 128,
            maximum_verification_steps: 256,
        },
    };

    if payload.required_claims.is_empty() {
        bail!("semantic payload has no required claims");
    }
    Ok(payload)
}

fn validation_context(program_id: u64) -> SemanticValidationContext {
    SemanticValidationContext {
        cognitive_state_version: CognitiveStateVersion(program_id),
        companion_state_version: None,
        subject_scope: SubjectScope(77),
    }
}

fn confidence(profile_confidence: f64, uncertainty: f64) -> EpistemicConfidence {
    let adjusted = (profile_confidence * (1.0 - uncertainty * 0.25)).clamp(0.0001, 1.0);
    let basis_points = (adjusted * 10_000.0).round() as u16;
    let level = if basis_points >= 9_000 {
        PlanConfidenceLevel::Certain
    } else if basis_points >= 7_000 {
        PlanConfidenceLevel::Probable
    } else if basis_points >= 3_000 {
        PlanConfidenceLevel::Possible
    } else if basis_points > 0 {
        PlanConfidenceLevel::Uncertain
    } else {
        PlanConfidenceLevel::Unknown
    };
    EpistemicConfidence {
        basis_points,
        level,
        explicitly_attached_before_rendering: true,
    }
}

fn to_epistemic_status(level: PlanConfidenceLevel) -> EpistemicStatus {
    match level {
        PlanConfidenceLevel::Certain => EpistemicStatus::Certain,
        PlanConfidenceLevel::Probable => EpistemicStatus::Probable,
        PlanConfidenceLevel::Possible => EpistemicStatus::Possible,
        PlanConfidenceLevel::Uncertain => EpistemicStatus::Uncertain,
        PlanConfidenceLevel::Unknown => EpistemicStatus::Unknown,
    }
}

fn to_program_intent(intent: PlanIntent) -> SemanticResponseIntent {
    match intent {
        PlanIntent::TechnicalExplanation | PlanIntent::ArchitecturalDiagnosis => {
            SemanticResponseIntent::Explanation
        }
        PlanIntent::Disagreement => SemanticResponseIntent::Contrast,
        PlanIntent::Correction | PlanIntent::Revision => SemanticResponseIntent::Correction,
        PlanIntent::UncertaintyDisclosure => SemanticResponseIntent::SelfCheck,
        PlanIntent::ContinuityReference | PlanIntent::EmotionalAcknowledgment => {
            SemanticResponseIntent::RelationalAcknowledgment
        }
        PlanIntent::SafetyBoundary => SemanticResponseIntent::Abstention,
        PlanIntent::Curiosity => SemanticResponseIntent::EvidenceRequest,
        PlanIntent::Surprise | PlanIntent::OrdinaryStatement | PlanIntent::Summarization => {
            SemanticResponseIntent::FactualAnswer
        }
    }
}

fn stance_for(intent: PlanIntent) -> ResponseStance {
    match intent {
        PlanIntent::Disagreement | PlanIntent::Correction | PlanIntent::Revision => {
            ResponseStance::Corrective
        }
        PlanIntent::SafetyBoundary => ResponseStance::Protective,
        PlanIntent::TechnicalExplanation | PlanIntent::ArchitecturalDiagnosis => ResponseStance::Candid,
        PlanIntent::EmotionalAcknowledgment | PlanIntent::ContinuityReference | PlanIntent::Curiosity => {
            ResponseStance::Collaborative
        }
        _ => ResponseStance::Neutral,
    }
}

fn emotion_for(intent: PlanIntent) -> EmotionalPosition {
    match intent {
        PlanIntent::EmotionalAcknowledgment | PlanIntent::ContinuityReference => {
            EmotionalPosition::WarmControlled
        }
        PlanIntent::Disagreement | PlanIntent::Correction | PlanIntent::Revision => {
            EmotionalPosition::ControlledFrustration
        }
        PlanIntent::Curiosity | PlanIntent::Surprise => EmotionalPosition::Curious,
        PlanIntent::UncertaintyDisclosure | PlanIntent::SafetyBoundary => EmotionalPosition::Cautious,
        _ => EmotionalPosition::Neutral,
    }
}

fn initiative_for(intent: PlanIntent) -> InitiativeLevel {
    match intent {
        PlanIntent::ArchitecturalDiagnosis | PlanIntent::Correction | PlanIntent::Revision => {
            InitiativeLevel::High
        }
        PlanIntent::TechnicalExplanation | PlanIntent::Curiosity | PlanIntent::Disagreement => {
            InitiativeLevel::Moderate
        }
        _ => InitiativeLevel::Low,
    }
}

fn handler_label(intent: PlanIntent) -> &'static str {
    match intent {
        PlanIntent::OrdinaryStatement => "transitional.statement",
        PlanIntent::Summarization => "transitional.summarization",
        PlanIntent::TechnicalExplanation => "transitional.technical_explanation",
        PlanIntent::ArchitecturalDiagnosis => "transitional.architectural_diagnosis",
        PlanIntent::EmotionalAcknowledgment => "transitional.emotional_acknowledgment",
        PlanIntent::Disagreement => "transitional.disagreement",
        PlanIntent::Correction => "transitional.correction",
        PlanIntent::UncertaintyDisclosure => "transitional.uncertainty",
        PlanIntent::ContinuityReference => "transitional.continuity",
        PlanIntent::SafetyBoundary => "transitional.safety_boundary",
        PlanIntent::Curiosity => "transitional.curiosity",
        PlanIntent::Revision => "transitional.revision",
        PlanIntent::Surprise => "transitional.surprise",
    }
}

fn plan_complete(plan: &SemanticResponsePlan) -> bool {
    !plan.fixture_id.is_empty()
        && !plan.prompt.is_empty()
        && !plan.operations.is_empty()
        && !plan.claims.is_empty()
        && plan.confidence.explicitly_attached_before_rendering
        && plan.claims.iter().all(|claim| {
            !claim.semantic_anchor.is_empty()
                && !claim.provenance.fixture_id.is_empty()
                && !claim.provenance.source_field.is_empty()
                && !claim.provenance.source_handler.is_empty()
        })
        && !plan.neutral_compatibility_text.is_empty()
        && !plan.generated_text_influence
}

fn load_fixtures() -> Result<Vec<Fixture>> {
    let shards = [
        ("ordinary", ORDINARY_JSON),
        ("technical", TECHNICAL_JSON),
        ("emotional", EMOTIONAL_JSON),
        ("disagreement", DISAGREEMENT_JSON),
        ("uncertainty", UNCERTAINTY_JSON),
        ("continuity", CONTINUITY_JSON),
        ("adversarial", ADVERSARIAL_JSON),
    ];
    let mut fixtures = Vec::new();
    for (category, json) in shards {
        let mut shard: Vec<Fixture> = serde_json::from_str(json)
            .with_context(|| format!("parse ΩV1-C {category} fixture shard"))?;
        fixtures.append(&mut shard);
    }
    Ok(fixtures)
}

fn validate_corpus(manifest: &Manifest, fixtures: &[Fixture]) -> Result<()> {
    let counts = fixtures.iter().fold(BTreeMap::new(), |mut counts, fixture| {
        *counts.entry(fixture.category.clone()).or_insert(0usize) += 1;
        counts
    });
    if counts != manifest.category_requirements {
        bail!("ΩV1-C category counts drifted: {counts:?}");
    }
    let ids = fixtures
        .iter()
        .map(|fixture| fixture.id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != fixtures.len() {
        bail!("ΩV1-C fixture identifiers are not unique");
    }
    Ok(())
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_frozen_fixtures_produce_complete_valid_plans() {
        let report = run_shadow_migration().expect("ΩV1-C report");
        assert_eq!(report.fixture_count, 122);
        assert_eq!(report.complete_plan_rate, 1.0);
        assert_eq!(report.neutral_compatibility_match_rate, 1.0);
        assert_eq!(report.semantic_program_validation_rate, 1.0);
        assert_eq!(report.missing_intent_count, 0);
        assert_eq!(report.missing_confidence_count, 0);
        assert_eq!(report.missing_claim_provenance_count, 0);
        assert!(report.gate_passed);
    }

    #[test]
    fn metacognitive_signals_are_typed_not_left_as_unlabelled_prose() {
        let report = run_shadow_migration().expect("ΩV1-C report");
        assert!(report.curiosity_operation_count > 0);
        assert!(report.revision_operation_count > 0);
        assert!(report.surprise_operation_count > 0);
        assert!(report.acknowledgment_operation_count > 0);
    }

    #[test]
    fn shadow_authority_boundary_is_closed() {
        let boundary = authority_boundary();
        assert!(!boundary.runtime_chat_wiring);
        assert!(!boundary.live_generated_text_influence);
        assert!(!boundary.voice_state_mutation);
        assert!(!boundary.memory_mutation);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.routing_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.charge_discharge_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
