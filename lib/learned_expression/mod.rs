//! ΩV1-F1 bounded learned expression selection.
//!
//! The first learned expression component is a deterministic integer ranker over
//! a closed grammar-v3 lattice. It cannot emit tokens, alter lexical bindings,
//! mutate VoiceState, or influence Runtime::chat(). Every selected surface is
//! reconstructed from text by an independent exact-lattice verifier. Any model,
//! lattice, verification, or budget failure returns the frozen grammar-v2 neutral
//! realization.

use crate::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalTableDigest, RealizationError,
    SurfaceReference,
};
use crate::semantic_response::{
    AbstentionReason, AuthorizedClaim, ClaimId, ClaimPolarity, DetailLevel, DiscourseOperation,
    DiscourseOperationKind, EpistemicStatus, MissingVariableId, ObservationId, OperationId,
    PredictionId, ResponseProgramDigest, SemanticProgramError, SemanticResponseProgram,
};
use crate::verifier_ready_realization::{
    abstention_text, epistemic_marker, VerifierReadyRealizationError, VerifierReadyRenderer,
    VERIFIER_READY_GRAMMAR_VERSION,
};
use crate::voice_state::VoiceDebugProjection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const LEARNED_EXPRESSION_GRAMMAR_VERSION: u16 = 3;
pub const MAX_VARIANTS_PER_OPERATION: usize = 6;
pub const MAX_BEAM_WIDTH: usize = 8;
pub const MAX_RESPONSE_CANDIDATES: usize = 64;
pub const MAX_TRAINABLE_PARAMETERS: usize = 250_000;
pub const MAX_MODEL_BYTES: usize = 4 * 1024 * 1024;
pub const VOICE_FEATURE_COUNT: usize = 7;

const LATTICE_DIGEST_DOMAIN: &[u8] = b"starfire-omega-v1f1-expression-lattice-v1";
const MODEL_DIGEST_DOMAIN: &[u8] = b"starfire-omega-v1f1-ranker-model-v1";
const VERIFICATION_DIGEST_DOMAIN: &[u8] = b"starfire-omega-v1f1-grammar-v3-verification-v1";
const SELECTION_DIGEST_DOMAIN: &[u8] = b"starfire-omega-v1f1-selection-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SurfaceVariantId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionLatticeDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedExpressionModelDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarV3VerificationDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedSelectionDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantProfile {
    pub directness_bps: u16,
    pub warmth_bps: u16,
    pub compression_bps: u16,
    pub initiative_bps: u16,
    pub disagreement_bps: u16,
    pub uncertainty_bps: u16,
    pub intensity_bps: u16,
}

impl VariantProfile {
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            directness_bps: 6_000,
            warmth_bps: 4_000,
            compression_bps: 6_000,
            initiative_bps: 5_000,
            disagreement_bps: 5_000,
            uncertainty_bps: 7_000,
            intensity_bps: 3_000,
        }
    }

    #[must_use]
    pub const fn direct() -> Self {
        Self {
            directness_bps: 9_000,
            warmth_bps: 2_000,
            compression_bps: 9_000,
            initiative_bps: 8_000,
            disagreement_bps: 9_000,
            uncertainty_bps: 8_500,
            intensity_bps: 5_000,
        }
    }

    #[must_use]
    pub const fn warm() -> Self {
        Self {
            directness_bps: 5_000,
            warmth_bps: 8_500,
            compression_bps: 4_500,
            initiative_bps: 6_000,
            disagreement_bps: 4_000,
            uncertainty_bps: 7_500,
            intensity_bps: 6_500,
        }
    }

    #[must_use]
    pub const fn as_array(self) -> [u16; VOICE_FEATURE_COUNT] {
        [
            self.directness_bps,
            self.warmth_bps,
            self.compression_bps,
            self.initiative_bps,
            self.disagreement_bps,
            self.uncertainty_bps,
            self.intensity_bps,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedVoiceProjection {
    pub version: u64,
    pub directness_bps: u16,
    pub warmth_bps: u16,
    pub compression_bps: u16,
    pub initiative_bps: u16,
    pub disagreement_bps: u16,
    pub uncertainty_bps: u16,
    pub intensity_bps: u16,
    pub source_digest: String,
}

impl LearnedVoiceProjection {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u64,
        directness_bps: u16,
        warmth_bps: u16,
        compression_bps: u16,
        initiative_bps: u16,
        disagreement_bps: u16,
        uncertainty_bps: u16,
        intensity_bps: u16,
        source_digest: impl Into<String>,
    ) -> Result<Self, LearnedExpressionError> {
        let values = [
            directness_bps,
            warmth_bps,
            compression_bps,
            initiative_bps,
            disagreement_bps,
            uncertainty_bps,
            intensity_bps,
        ];
        if values.iter().any(|value| *value > 10_000) {
            return Err(LearnedExpressionError::InvalidVoiceProjection);
        }
        let source_digest = source_digest.into();
        if source_digest.trim().is_empty() {
            return Err(LearnedExpressionError::InvalidVoiceProjection);
        }
        Ok(Self {
            version,
            directness_bps,
            warmth_bps,
            compression_bps,
            initiative_bps,
            disagreement_bps,
            uncertainty_bps,
            intensity_bps,
            source_digest,
        })
    }

    pub fn from_debug_projection(
        projection: &VoiceDebugProjection,
    ) -> Result<Self, LearnedExpressionError> {
        let disagreement_bps = match projection.disagreement_style.as_str() {
            "yielding" => 1_667,
            "measured" => 5_000,
            "direct" => 8_333,
            _ => return Err(LearnedExpressionError::InvalidVoiceProjection),
        };
        let uncertainty_bps = match projection.uncertainty_style.as_str() {
            "implicit" => 1_667,
            "calibrated" => 5_000,
            "explicit" => 8_333,
            _ => return Err(LearnedExpressionError::InvalidVoiceProjection),
        };
        Self::new(
            projection.version,
            unit_to_bps(projection.directness)?,
            unit_to_bps(projection.warmth)?,
            unit_to_bps(projection.compression)?,
            unit_to_bps(projection.initiative)?,
            disagreement_bps,
            uncertainty_bps,
            unit_to_bps(projection.session_intensity)?,
            projection.digest.clone(),
        )
    }

    #[must_use]
    pub fn as_array(&self) -> [u16; VOICE_FEATURE_COUNT] {
        [
            self.directness_bps,
            self.warmth_bps,
            self.compression_bps,
            self.initiative_bps,
            self.disagreement_bps,
            self.uncertainty_bps,
            self.intensity_bps,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationSurfaceVariant {
    pub operation: OperationId,
    pub variant_id: SurfaceVariantId,
    pub text: String,
    pub kind: DiscourseOperationKind,
    pub claim_ids: Vec<ClaimId>,
    pub references: Vec<SurfaceReference>,
    pub profile: VariantProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionLatticePayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub grammar_version: u16,
    pub variants: Vec<OperationSurfaceVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionLattice {
    pub payload: ExpressionLatticePayload,
    pub digest: ExpressionLatticeDigest,
}

impl ExpressionLattice {
    pub fn build(
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<Self, LearnedExpressionError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;

        let claims = program
            .payload
            .required_claims
            .iter()
            .chain(program.payload.optional_claims.iter())
            .map(|claim| (claim.id, claim))
            .collect::<BTreeMap<_, _>>();
        let lexical_claims = lexical_table
            .payload
            .claims
            .iter()
            .map(|binding| (binding.claim, binding))
            .collect::<BTreeMap<_, _>>();
        let observations = lexical_table
            .payload
            .observations
            .iter()
            .map(|binding| (binding.observation, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let variables = lexical_table
            .payload
            .missing_variables
            .iter()
            .map(|binding| (binding.variable, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let predictions = lexical_table
            .payload
            .predictions
            .iter()
            .map(|binding| (binding.prediction, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();

        let mut variants = Vec::new();
        for operation in &program.payload.operations {
            let operation_variants = build_operation_variants(
                operation,
                &claims,
                &lexical_claims,
                &observations,
                &variables,
                &predictions,
                program.payload.style.allow_questions,
            )?;
            if operation_variants.is_empty()
                || operation_variants.len() > MAX_VARIANTS_PER_OPERATION
            {
                return Err(LearnedExpressionError::VariantBudgetExceeded);
            }
            variants.extend(operation_variants);
        }

        validate_lattice_variants(&variants, &lexical_table.payload.forbidden_surface_forms)?;
        let payload = ExpressionLatticePayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            grammar_version: LEARNED_EXPRESSION_GRAMMAR_VERSION,
            variants,
        };
        let digest = ExpressionLatticeDigest(digest_value(LATTICE_DIGEST_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(LearnedExpressionError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<(), LearnedExpressionError> {
        let rebuilt = Self::build(program, lexical_table)?;
        if self != &rebuilt {
            return Err(LearnedExpressionError::LatticeDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedExpressionModelPayload {
    pub schema_version: u16,
    pub weights: [i32; VOICE_FEATURE_COUNT],
    pub margin: i32,
    pub training_examples: u32,
    pub epochs: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedExpressionModel {
    pub payload: LearnedExpressionModelPayload,
    pub digest: LearnedExpressionModelDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredSide {
    Left,
    Right,
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairwisePreference {
    pub projection: LearnedVoiceProjection,
    pub left: VariantProfile,
    pub right: VariantProfile,
    pub preferred: PreferredSide,
}

impl LearnedExpressionModel {
    pub fn baseline() -> Result<Self, LearnedExpressionError> {
        Self::from_payload(LearnedExpressionModelPayload {
            schema_version: 1,
            weights: [1_000; VOICE_FEATURE_COUNT],
            margin: 100,
            training_examples: 0,
            epochs: 0,
        })
    }

    pub fn train(
        examples: &[PairwisePreference],
        epochs: u16,
        learning_rate: i32,
    ) -> Result<Self, LearnedExpressionError> {
        if epochs == 0 || learning_rate <= 0 {
            return Err(LearnedExpressionError::InvalidTrainingConfiguration);
        }
        let mut payload = LearnedExpressionModel::baseline()?.payload;
        payload.epochs = epochs;
        payload.training_examples = u32::try_from(examples.len())
            .map_err(|_| LearnedExpressionError::ModelBudgetExceeded)?;

        for _ in 0..epochs {
            for example in examples {
                let (preferred, rejected) = match example.preferred {
                    PreferredSide::Left => (example.left, example.right),
                    PreferredSide::Right => (example.right, example.left),
                    PreferredSide::Tie => continue,
                };
                let preferred_matches = feature_matches(&example.projection, preferred);
                let rejected_matches = feature_matches(&example.projection, rejected);
                let preferred_score = weighted_score(&payload.weights, &preferred_matches);
                let rejected_score = weighted_score(&payload.weights, &rejected_matches);
                if preferred_score <= rejected_score + i64::from(payload.margin) {
                    for index in 0..VOICE_FEATURE_COUNT {
                        let difference = i32::from(preferred_matches[index])
                            - i32::from(rejected_matches[index]);
                        let adjustment = learning_rate
                            .saturating_mul(difference)
                            .saturating_div(1_000);
                        payload.weights[index] = payload.weights[index]
                            .saturating_add(adjustment)
                            .clamp(-100_000, 100_000);
                    }
                }
            }
        }
        Self::from_payload(payload)
    }

    fn from_payload(
        payload: LearnedExpressionModelPayload,
    ) -> Result<Self, LearnedExpressionError> {
        if payload.schema_version != 1
            || payload.margin < 0
            || payload.weights.len() > MAX_TRAINABLE_PARAMETERS
        {
            return Err(LearnedExpressionError::ModelBudgetExceeded);
        }
        let bytes = canonical_bytes(&payload)?;
        if bytes.len() > MAX_MODEL_BYTES {
            return Err(LearnedExpressionError::ModelBudgetExceeded);
        }
        let digest = LearnedExpressionModelDigest(domain_digest(MODEL_DIGEST_DOMAIN, &bytes));
        if digest.0 == 0 {
            return Err(LearnedExpressionError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(&self) -> Result<(), LearnedExpressionError> {
        let rebuilt = Self::from_payload(self.payload.clone())?;
        if rebuilt.digest != self.digest {
            return Err(LearnedExpressionError::ModelDigestMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn parameter_count(&self) -> usize {
        self.payload.weights.len()
    }

    pub fn artifact_bytes(&self) -> Result<Vec<u8>, LearnedExpressionError> {
        canonical_bytes(self)
    }

    fn score(&self, projection: &LearnedVoiceProjection, profile: VariantProfile) -> i64 {
        weighted_score(&self.payload.weights, &feature_matches(projection, profile))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationTerminalClassification {
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVariant {
    pub operation: OperationId,
    pub variant_id: SurfaceVariantId,
    pub kind: DiscourseOperationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarV3Costs {
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarV3VerificationPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub lattice_digest: ExpressionLatticeDigest,
    pub grammar_version: u16,
    pub variants: Vec<VerifiedVariant>,
    pub costs: GrammarV3Costs,
    pub terminal_classification: VerificationTerminalClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarV3VerificationReport {
    pub payload: GrammarV3VerificationPayload,
    pub digest: GrammarV3VerificationDigest,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GrammarV3Verifier;

impl GrammarV3Verifier {
    pub fn verify(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        lattice_digest: ExpressionLatticeDigest,
        text: &str,
    ) -> Result<GrammarV3VerificationReport, LearnedExpressionError> {
        if text.is_empty() {
            return Err(LearnedExpressionError::UnsupportedSurface);
        }
        let lattice = ExpressionLattice::build(program, lexical_table)?;
        if lattice.digest != lattice_digest {
            return Err(LearnedExpressionError::LatticeDigestMismatch);
        }
        reject_forbidden_text(text, &lexical_table.payload.forbidden_surface_forms)?;
        let matched = parse_exact_variants(program, &lattice.payload.variants, text)?;

        if matched.len() != program.payload.operations.len() {
            return Err(LearnedExpressionError::OperationMismatch);
        }
        for (expected, actual) in program.payload.operations.iter().zip(&matched) {
            if expected.id != actual.operation || expected.kind != actual.kind {
                return Err(LearnedExpressionError::OperationMismatch);
            }
        }

        let costs = recompute_costs(program, text, &matched)?;
        let payload = GrammarV3VerificationPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            lattice_digest: lattice.digest,
            grammar_version: LEARNED_EXPRESSION_GRAMMAR_VERSION,
            variants: matched
                .iter()
                .map(|variant| VerifiedVariant {
                    operation: variant.operation,
                    variant_id: variant.variant_id,
                    kind: variant.kind.clone(),
                })
                .collect(),
            costs,
            terminal_classification: VerificationTerminalClassification::Pass,
        };
        let digest =
            GrammarV3VerificationDigest(digest_value(VERIFICATION_DIGEST_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(LearnedExpressionError::EmptyDigest);
        }
        Ok(GrammarV3VerificationReport { payload, digest })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionDisposition {
    LearnedVerified,
    NeutralFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedSelectionPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub voice_projection_digest: String,
    pub model_digest: LearnedExpressionModelDigest,
    pub lattice_digest: Option<ExpressionLatticeDigest>,
    pub selected_grammar_version: u16,
    pub disposition: SelectionDisposition,
    pub text: String,
    pub variant_ids: Vec<SurfaceVariantId>,
    pub score: i64,
    pub complete_candidates_scored: u16,
    pub verification_digest: Option<GrammarV3VerificationDigest>,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedSelectionResult {
    pub payload: LearnedSelectionPayload,
    pub digest: LearnedSelectionDigest,
}

#[derive(Debug, Clone)]
struct BeamCandidate {
    text: String,
    variant_ids: Vec<SurfaceVariantId>,
    score: i64,
}

#[derive(Debug, Clone)]
pub struct OfflineLearnedExpressionSelector {
    model: LearnedExpressionModel,
}

impl OfflineLearnedExpressionSelector {
    #[must_use]
    pub fn new(model: LearnedExpressionModel) -> Self {
        Self { model }
    }

    pub fn select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &LearnedVoiceProjection,
    ) -> Result<LearnedSelectionResult, LearnedExpressionError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        let neutral = VerifierReadyRenderer.render(program, lexical_table)?;

        let learned = self.try_select(program, lexical_table, projection);
        let payload = match learned {
            Ok(payload) => payload,
            Err(error) => LearnedSelectionPayload {
                program_digest: program.digest,
                lexical_table_digest: lexical_table.digest,
                voice_projection_digest: projection.source_digest.clone(),
                model_digest: self.model.digest,
                lattice_digest: None,
                selected_grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
                disposition: SelectionDisposition::NeutralFallback,
                text: neutral.payload.text,
                variant_ids: Vec::new(),
                score: 0,
                complete_candidates_scored: 0,
                verification_digest: None,
                fallback_reason: Some(error.to_string()),
            },
        };
        let digest = LearnedSelectionDigest(digest_value(SELECTION_DIGEST_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(LearnedExpressionError::EmptyDigest);
        }
        Ok(LearnedSelectionResult { payload, digest })
    }

    fn try_select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &LearnedVoiceProjection,
    ) -> Result<LearnedSelectionPayload, LearnedExpressionError> {
        self.model.verify_integrity()?;
        let lattice = ExpressionLattice::build(program, lexical_table)?;
        let mut by_operation = BTreeMap::<OperationId, Vec<&OperationSurfaceVariant>>::new();
        for variant in &lattice.payload.variants {
            by_operation
                .entry(variant.operation)
                .or_default()
                .push(variant);
        }
        for variants in by_operation.values_mut() {
            variants.sort_by_key(|variant| variant.variant_id);
        }

        let mut beam = vec![BeamCandidate {
            text: String::new(),
            variant_ids: Vec::new(),
            score: 0,
        }];
        for (index, operation) in program.payload.operations.iter().enumerate() {
            let variants = by_operation
                .get(&operation.id)
                .ok_or(LearnedExpressionError::MissingOperationVariants)?;
            let separator = separator_before(program, index);
            let mut next = Vec::new();
            for partial in &beam {
                for variant in variants {
                    let mut text = partial.text.clone();
                    text.push_str(separator);
                    text.push_str(&variant.text);
                    let mut variant_ids = partial.variant_ids.clone();
                    variant_ids.push(variant.variant_id);
                    next.push(BeamCandidate {
                        text,
                        variant_ids,
                        score: partial.score + self.model.score(projection, variant.profile),
                    });
                }
            }
            next.sort_by(|left, right| {
                right
                    .score
                    .cmp(&left.score)
                    .then_with(|| left.variant_ids.cmp(&right.variant_ids))
            });
            next.truncate(MAX_BEAM_WIDTH);
            beam = next;
        }

        if beam.is_empty() || beam.len() > MAX_RESPONSE_CANDIDATES {
            return Err(LearnedExpressionError::CandidateBudgetExceeded);
        }
        let complete_candidates_scored = u16::try_from(beam.len())
            .map_err(|_| LearnedExpressionError::CandidateBudgetExceeded)?;
        let verifier = GrammarV3Verifier;
        for candidate in beam {
            if let Ok(report) =
                verifier.verify(program, lexical_table, lattice.digest, &candidate.text)
            {
                return Ok(LearnedSelectionPayload {
                    program_digest: program.digest,
                    lexical_table_digest: lexical_table.digest,
                    voice_projection_digest: projection.source_digest.clone(),
                    model_digest: self.model.digest,
                    lattice_digest: Some(lattice.digest),
                    selected_grammar_version: LEARNED_EXPRESSION_GRAMMAR_VERSION,
                    disposition: SelectionDisposition::LearnedVerified,
                    text: candidate.text,
                    variant_ids: candidate.variant_ids,
                    score: candidate.score,
                    complete_candidates_scored,
                    verification_digest: Some(report.digest),
                    fallback_reason: None,
                });
            }
        }
        Err(LearnedExpressionError::NoVerifiedCandidate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedExpressionAuthorityBoundary {
    pub candidate_lattice_construction: bool,
    pub learned_candidate_scoring: bool,
    pub independent_candidate_verification: bool,
    pub runtime_chat_wiring: bool,
    pub http_response_influence: bool,
    pub live_generated_text_influence: bool,
    pub raw_prompt_access: bool,
    pub unrestricted_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub voice_state_mutation: bool,
    pub companion_state_access: bool,
    pub persistence_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> LearnedExpressionAuthorityBoundary {
    LearnedExpressionAuthorityBoundary {
        candidate_lattice_construction: true,
        learned_candidate_scoring: true,
        independent_candidate_verification: true,
        runtime_chat_wiring: false,
        http_response_influence: false,
        live_generated_text_influence: false,
        raw_prompt_access: false,
        unrestricted_conversation_access: false,
        unrestricted_memory_access: false,
        voice_state_mutation: false,
        companion_state_access: false,
        persistence_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Error)]
pub enum LearnedExpressionError {
    #[error("semantic program validation failed: {0}")]
    SemanticProgram(#[from] SemanticProgramError),
    #[error("lexical table validation failed: {0}")]
    LexicalTable(#[from] RealizationError),
    #[error("neutral realization failed: {0}")]
    NeutralRealization(#[from] VerifierReadyRealizationError),
    #[error("the voice projection is invalid")]
    InvalidVoiceProjection,
    #[error("the grammar-v3 variant budget is exceeded")]
    VariantBudgetExceeded,
    #[error("the complete-candidate budget is exceeded")]
    CandidateBudgetExceeded,
    #[error("the learned model budget is exceeded")]
    ModelBudgetExceeded,
    #[error("the training configuration is invalid")]
    InvalidTrainingConfiguration,
    #[error(
        "the expression lattice contains an empty, duplicate, ambiguous, or malformed surface"
    )]
    InvalidLattice,
    #[error("the expression lattice digest is stale or mismatched")]
    LatticeDigestMismatch,
    #[error("the learned model digest is stale or mismatched")]
    ModelDigestMismatch,
    #[error("the candidate contains an unsupported or unparsed surface")]
    UnsupportedSurface,
    #[error("the reconstructed operation sequence does not match the authorized program")]
    OperationMismatch,
    #[error("the candidate contains a forbidden surface form")]
    ForbiddenSurfaceForm,
    #[error("the candidate exceeds an output or compute budget")]
    BudgetExceeded,
    #[error("an operation has no candidate variants")]
    MissingOperationVariants,
    #[error("no independently verified candidate survived selection")]
    NoVerifiedCandidate,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("a canonical digest is zero")]
    EmptyDigest,
}

fn build_operation_variants(
    operation: &DiscourseOperation,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
    lexical_claims: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
    observations: &BTreeMap<ObservationId, &str>,
    variables: &BTreeMap<MissingVariableId, &str>,
    predictions: &BTreeMap<PredictionId, &str>,
    allow_questions: bool,
) -> Result<Vec<OperationSurfaceVariant>, LearnedExpressionError> {
    let (texts, claim_ids, references) = match &operation.kind {
        DiscourseOperationKind::Assert(claim) => {
            let claim_text = render_claim(*claim, claims, lexical_claims)?;
            (
                vec![
                    format!("{}.", claim_text),
                    format!("Conclusion: {}.", claim_text),
                    format!("The finding is: {}.", claim_text),
                ],
                vec![*claim],
                Vec::new(),
            )
        }
        DiscourseOperationKind::Qualify { claim, status } => {
            let authorized = claims
                .get(claim)
                .copied()
                .ok_or(LearnedExpressionError::InvalidLattice)?;
            if authorized.epistemic_status != *status {
                return Err(LearnedExpressionError::InvalidLattice);
            }
            let claim_text = render_claim(*claim, claims, lexical_claims)?;
            (
                vec![
                    format!("Qualification: {}.", claim_text),
                    format!("Calibrated conclusion: {}.", claim_text),
                    format!("With uncertainty preserved, {}.", claim_text),
                ],
                vec![*claim],
                Vec::new(),
            )
        }
        DiscourseOperationKind::Contrast { left, right } => {
            let left_text = render_claim(*left, claims, lexical_claims)?;
            let right_text = render_claim(*right, claims, lexical_claims)?;
            (
                vec![
                    format!("On one side, {}. By contrast, {}.", left_text, right_text),
                    format!("The contrast is: {}; however, {}.", left_text, right_text),
                    format!("Set side by side, {}; while {}.", left_text, right_text),
                ],
                vec![*left, *right],
                Vec::new(),
            )
        }
        DiscourseOperationKind::Correct { prior, replacement } => {
            let prior_text = render_claim(*prior, claims, lexical_claims)?;
            let replacement_text = render_claim(*replacement, claims, lexical_claims)?;
            (
                vec![
                    format!("Correction: {}; instead, {}.", prior_text, replacement_text),
                    format!(
                        "Correction pair: {}; replacement: {}.",
                        prior_text, replacement_text
                    ),
                    format!(
                        "The correction is explicit: {}; instead, {}.",
                        prior_text, replacement_text
                    ),
                ],
                vec![*prior, *replacement],
                Vec::new(),
            )
        }
        DiscourseOperationKind::Explain { claims: explained } => {
            let surfaces = explained
                .iter()
                .map(|claim| render_claim(*claim, claims, lexical_claims))
                .collect::<Result<Vec<_>, _>>()?;
            (
                vec![
                    format!("Relevant support: {}.", surfaces.join("; ")),
                    format!("The supporting chain is: {}.", surfaces.join("; ")),
                    format!("This follows from: {}.", surfaces.join("; ")),
                ],
                explained.clone(),
                Vec::new(),
            )
        }
        DiscourseOperationKind::Acknowledge(observation) => {
            let label = observations
                .get(observation)
                .copied()
                .ok_or(LearnedExpressionError::InvalidLattice)?;
            (
                vec![
                    format!("I acknowledge {}.", label),
                    format!("I register {}.", label),
                    format!("Acknowledged: {}.", label),
                ],
                Vec::new(),
                vec![SurfaceReference::Observation(*observation)],
            )
        }
        DiscourseOperationKind::RequestEvidence(variable) => {
            let label = variables
                .get(variable)
                .copied()
                .ok_or(LearnedExpressionError::InvalidLattice)?;
            let texts = if allow_questions {
                vec![
                    format!("What evidence resolves {}?", label),
                    format!("Which evidence would resolve {}?", label),
                    format!("What would settle the evidence question around {}?", label),
                ]
            } else {
                vec![
                    format!("Evidence is required for {}.", label),
                    format!("The unresolved evidence concerns {}.", label),
                    format!("Resolution requires evidence about {}.", label),
                ]
            };
            (
                texts,
                Vec::new(),
                vec![SurfaceReference::MissingVariable(*variable)],
            )
        }
        DiscourseOperationKind::Commit(prediction) => {
            let label = predictions
                .get(prediction)
                .copied()
                .ok_or(LearnedExpressionError::InvalidLattice)?;
            (
                vec![
                    format!("I commit to track {}.", label),
                    format!("I will track {}.", label),
                    format!("Tracking commitment: {}.", label),
                ],
                Vec::new(),
                vec![SurfaceReference::Prediction(*prediction)],
            )
        }
        DiscourseOperationKind::Abstain(reason) => {
            (abstention_variants(*reason), Vec::new(), Vec::new())
        }
    };

    let profiles = [
        VariantProfile::neutral(),
        VariantProfile::direct(),
        VariantProfile::warm(),
    ];
    texts
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let variant_id = u16::try_from(index)
                .map(SurfaceVariantId)
                .map_err(|_| LearnedExpressionError::VariantBudgetExceeded)?;
            Ok(OperationSurfaceVariant {
                operation: operation.id,
                variant_id,
                text,
                kind: operation.kind.clone(),
                claim_ids: claim_ids.clone(),
                references: references.clone(),
                profile: profiles[index.min(profiles.len() - 1)],
            })
        })
        .collect()
}

fn abstention_variants(reason: AbstentionReason) -> Vec<String> {
    let alternatives = match reason {
        AbstentionReason::InsufficientEvidence => [
            "The evidence is insufficient, so I abstain.",
            "I will not conclude this because the available evidence is insufficient.",
        ],
        AbstentionReason::ContradictoryEvidence => [
            "The evidence is contradictory, so I abstain.",
            "I will not conclude this because the available evidence is contradictory.",
        ],
        AbstentionReason::SensitiveContext => [
            "The context is too sensitive for disclosure, so I abstain.",
            "I abstain because disclosure would cross the sensitivity boundary.",
        ],
        AbstentionReason::UnsupportedIntent => [
            "The response intent is unsupported, so I abstain.",
            "I abstain because the requested response intent is unsupported.",
        ],
        AbstentionReason::BudgetExhausted => [
            "The authorized response budget is exhausted, so I abstain.",
            "I abstain because the authorized response budget has been exhausted.",
        ],
    };
    vec![
        abstention_text(reason).to_owned(),
        alternatives[0].to_owned(),
        alternatives[1].to_owned(),
    ]
}

fn render_claim(
    claim_id: ClaimId,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
    lexical_claims: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
) -> Result<String, LearnedExpressionError> {
    let claim = claims
        .get(&claim_id)
        .copied()
        .ok_or(LearnedExpressionError::InvalidLattice)?;
    let binding = lexical_claims
        .get(&claim_id)
        .copied()
        .ok_or(LearnedExpressionError::InvalidLattice)?;
    let clause = match claim.polarity {
        ClaimPolarity::Positive => &binding.positive_clause,
        ClaimPolarity::Negative => &binding.negative_clause,
    };
    Ok(format!(
        "{} {}",
        epistemic_marker(claim.epistemic_status),
        clause
    ))
}

fn validate_lattice_variants(
    variants: &[OperationSurfaceVariant],
    forbidden_forms: &[String],
) -> Result<(), LearnedExpressionError> {
    let mut operation_ids = BTreeMap::<OperationId, BTreeSet<SurfaceVariantId>>::new();
    let mut surfaces = BTreeSet::<String>::new();
    for variant in variants {
        if variant.text.is_empty()
            || variant.text.trim() != variant.text
            || variant.text.contains('\n')
            || !operation_ids
                .entry(variant.operation)
                .or_default()
                .insert(variant.variant_id)
            || !surfaces.insert(variant.text.clone())
        {
            return Err(LearnedExpressionError::InvalidLattice);
        }
        reject_forbidden_text(&variant.text, forbidden_forms)?;
    }
    let ordered = surfaces.iter().collect::<Vec<_>>();
    for (index, left) in ordered.iter().enumerate() {
        for right in ordered.iter().skip(index + 1) {
            if left.starts_with(right.as_str()) || right.starts_with(left.as_str()) {
                return Err(LearnedExpressionError::InvalidLattice);
            }
        }
    }
    Ok(())
}

fn parse_exact_variants<'a>(
    program: &SemanticResponseProgram,
    variants: &'a [OperationSurfaceVariant],
    text: &str,
) -> Result<Vec<&'a OperationSurfaceVariant>, LearnedExpressionError> {
    let mut cursor = 0_usize;
    let mut matched = Vec::with_capacity(program.payload.operations.len());
    for index in 0..program.payload.operations.len() {
        let separator = separator_before(program, index);
        if !text[cursor..].starts_with(separator) {
            return Err(LearnedExpressionError::UnsupportedSurface);
        }
        cursor += separator.len();
        let remaining = &text[cursor..];
        let next_separator = if index + 1 < program.payload.operations.len() {
            separator_before(program, index + 1)
        } else {
            ""
        };
        let candidates = variants
            .iter()
            .filter(|variant| {
                if !remaining.starts_with(&variant.text) {
                    return false;
                }
                let end = variant.text.len();
                if index + 1 == program.payload.operations.len() {
                    end == remaining.len()
                } else {
                    remaining[end..].starts_with(next_separator)
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(LearnedExpressionError::UnsupportedSurface);
        }
        let candidate = candidates[0];
        cursor += candidate.text.len();
        matched.push(candidate);
    }
    if cursor != text.len() {
        return Err(LearnedExpressionError::UnsupportedSurface);
    }
    Ok(matched)
}

fn recompute_costs(
    program: &SemanticResponseProgram,
    text: &str,
    variants: &[&OperationSurfaceVariant],
) -> Result<GrammarV3Costs, LearnedExpressionError> {
    let operation_cost =
        u32::try_from(variants.len()).map_err(|_| LearnedExpressionError::BudgetExceeded)?;
    let claim_cost = u32::try_from(
        variants
            .iter()
            .map(|variant| variant.claim_ids.len())
            .sum::<usize>(),
    )
    .map_err(|_| LearnedExpressionError::BudgetExceeded)?;
    let verification_step_cost = operation_cost
        .checked_add(claim_cost)
        .and_then(|cost| cost.checked_add(operation_cost))
        .ok_or(LearnedExpressionError::BudgetExceeded)?;
    let character_cost =
        u32::try_from(text.len()).map_err(|_| LearnedExpressionError::BudgetExceeded)?;
    let sentence_count = count_sentences(text)?;
    let paragraph_count = u16::try_from(text.split("\n\n").count())
        .map_err(|_| LearnedExpressionError::BudgetExceeded)?;

    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(LearnedExpressionError::BudgetExceeded);
    }
    Ok(GrammarV3Costs {
        operation_cost,
        claim_cost,
        verification_step_cost,
        character_cost,
        sentence_count,
        paragraph_count,
    })
}

fn separator_before(program: &SemanticResponseProgram, index: usize) -> &'static str {
    if index == 0 {
        return "";
    }
    let target_paragraphs = match program.payload.style.detail {
        DetailLevel::Detailed => program
            .payload
            .operations
            .len()
            .min(usize::from(program.payload.style.maximum_paragraphs)),
        DetailLevel::Brief | DetailLevel::Standard => 1,
    }
    .max(1);
    let operations_per_paragraph = program
        .payload
        .operations
        .len()
        .div_ceil(target_paragraphs)
        .max(1);
    if program.payload.style.detail == DetailLevel::Detailed
        && index % operations_per_paragraph == 0
    {
        "\n\n"
    } else {
        " "
    }
}

fn reject_forbidden_text(
    text: &str,
    forbidden_forms: &[String],
) -> Result<(), LearnedExpressionError> {
    let normalized = text.to_lowercase();
    if forbidden_forms
        .iter()
        .any(|form| normalized.contains(&form.to_lowercase()))
    {
        return Err(LearnedExpressionError::ForbiddenSurfaceForm);
    }
    Ok(())
}

fn count_sentences(text: &str) -> Result<u16, LearnedExpressionError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(LearnedExpressionError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| LearnedExpressionError::BudgetExceeded)
}

fn feature_matches(
    projection: &LearnedVoiceProjection,
    profile: VariantProfile,
) -> [u16; VOICE_FEATURE_COUNT] {
    let projection = projection.as_array();
    let profile = profile.as_array();
    let mut matches = [0_u16; VOICE_FEATURE_COUNT];
    for index in 0..VOICE_FEATURE_COUNT {
        matches[index] = 10_000_u16.saturating_sub(projection[index].abs_diff(profile[index]));
    }
    matches
}

fn weighted_score(
    weights: &[i32; VOICE_FEATURE_COUNT],
    matches: &[u16; VOICE_FEATURE_COUNT],
) -> i64 {
    weights
        .iter()
        .zip(matches)
        .map(|(weight, matched)| i64::from(*weight) * i64::from(*matched) / 10_000)
        .sum()
}

fn unit_to_bps(value: f64) -> Result<u16, LearnedExpressionError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(LearnedExpressionError::InvalidVoiceProjection);
    }
    Ok((value * 10_000.0).round() as u16)
}

fn digest_value<T: Serialize>(domain: &[u8], value: &T) -> Result<u64, LearnedExpressionError> {
    let bytes = canonical_bytes(value)?;
    Ok(domain_digest(domain, &bytes))
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, LearnedExpressionError> {
    serde_json::to_vec(value)
        .map_err(|error| LearnedExpressionError::CanonicalSerialization(error.to_string()))
}

fn domain_digest(domain: &[u8], encoded: &[u8]) -> u64 {
    let mut digest = fnv1a64(domain);
    digest = mix_u64(digest, encoded.len() as u64);
    for byte in encoded {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
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
    use crate::language_realization::{LexicalBindingTablePayload, ObservationLexicalBinding};
    use crate::semantic_response::{
        AcknowledgmentLevel, CognitiveStateVersion, ComputeBudget, DialogueMode,
        DiscourseOperation, EpistemicConstraint, OutputBudget, ProhibitedClaim, ResponseProgramId,
        SemanticResponseIntent, SemanticResponseProgramPayload, SemanticValidationContext,
        SensitivityLevel, SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
    };

    const SUBJECT: SubjectScope = SubjectScope(7);
    const COGNITIVE_VERSION: CognitiveStateVersion = CognitiveStateVersion(11);

    fn fixture() -> (SemanticResponseProgram, LexicalBindingTable) {
        let claim = AuthorizedClaim {
            id: ClaimId(1),
            semantic_key: "bounded_selection".to_owned(),
            polarity: ClaimPolarity::Positive,
            confidence_bps: 9_500,
            epistemic_status: EpistemicStatus::Certain,
            sensitivity: SensitivityLevel::Public,
            disclosure_scope: SUBJECT,
        };
        let payload = SemanticResponseProgramPayload {
            id: ResponseProgramId(1),
            source_state_version: COGNITIVE_VERSION,
            companion_state_version: None,
            subject_scope: SUBJECT,
            intent: SemanticResponseIntent::Explanation,
            operations: vec![
                DiscourseOperation {
                    id: OperationId(1),
                    kind: DiscourseOperationKind::Assert(ClaimId(1)),
                },
                DiscourseOperation {
                    id: OperationId(2),
                    kind: DiscourseOperationKind::Acknowledge(ObservationId(101)),
                },
                DiscourseOperation {
                    id: OperationId(3),
                    kind: DiscourseOperationKind::Abstain(AbstentionReason::InsufficientEvidence),
                },
            ],
            required_claims: vec![claim],
            optional_claims: Vec::new(),
            prohibited_claims: vec![ProhibitedClaim {
                id: ClaimId(2),
                semantic_key: "unbounded_generation".to_owned(),
            }],
            epistemic_constraints: vec![EpistemicConstraint {
                claim: ClaimId(1),
                required_status: EpistemicStatus::Certain,
                minimum_confidence_bps: 9_000,
                maximum_confidence_bps: 10_000,
            }],
            sensitivity: SensitivityPolicy {
                maximum_disclosure: SensitivityLevel::Public,
                disclosure_scope: SUBJECT,
            },
            style: StyleEnvelope {
                detail: DetailLevel::Detailed,
                vocabulary: VocabularyLevel::Technical,
                dialogue: DialogueMode::Collaborative,
                acknowledgment: AcknowledgmentLevel::Explicit,
                allow_first_person: true,
                allow_questions: true,
                maximum_paragraphs: 3,
            },
            output_budget: OutputBudget {
                maximum_characters: 2_000,
                maximum_sentences: 12,
            },
            compute_budget: ComputeBudget {
                maximum_operations: 8,
                maximum_claims: 8,
                maximum_verification_steps: 32,
            },
        };
        let program = SemanticResponseProgram::validate(
            payload,
            SemanticValidationContext {
                cognitive_state_version: COGNITIVE_VERSION,
                companion_state_version: None,
                subject_scope: SUBJECT,
            },
        )
        .unwrap();
        let lexical = LexicalBindingTable::validate(
            LexicalBindingTablePayload {
                program_digest: program.digest,
                subject_scope: SUBJECT,
                claims: vec![ClaimLexicalBinding {
                    claim: ClaimId(1),
                    positive_clause: "the selector remains bounded".to_owned(),
                    negative_clause: "the selector is not bounded".to_owned(),
                }],
                observations: vec![ObservationLexicalBinding {
                    observation: ObservationId(101),
                    label: "the frozen authority boundary".to_owned(),
                }],
                missing_variables: Vec::new(),
                predictions: Vec::new(),
                forbidden_surface_forms: vec!["forbidden leakage".to_owned()],
            },
            &program,
        )
        .unwrap();
        (program, lexical)
    }

    fn projection(direct: bool) -> LearnedVoiceProjection {
        if direct {
            LearnedVoiceProjection::new(
                1,
                9_000,
                2_000,
                9_000,
                8_000,
                9_000,
                8_500,
                5_000,
                "direct-projection",
            )
            .unwrap()
        } else {
            LearnedVoiceProjection::new(
                1,
                5_000,
                8_500,
                4_500,
                6_000,
                4_000,
                7_500,
                6_500,
                "warm-projection",
            )
            .unwrap()
        }
    }

    #[test]
    fn lattice_is_closed_bounded_and_replayable() {
        let (program, lexical) = fixture();
        let lattice = ExpressionLattice::build(&program, &lexical).unwrap();
        assert_eq!(lattice.payload.grammar_version, 3);
        assert_eq!(lattice.payload.variants.len(), 9);
        lattice.verify_integrity(&program, &lexical).unwrap();
        let surfaces = lattice
            .payload
            .variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(surfaces.len(), lattice.payload.variants.len());
    }

    #[test]
    fn training_and_selection_are_exactly_deterministic() {
        let examples = vec![PairwisePreference {
            projection: projection(true),
            left: VariantProfile::direct(),
            right: VariantProfile::warm(),
            preferred: PreferredSide::Left,
        }];
        let first = LearnedExpressionModel::train(&examples, 4, 100).unwrap();
        let second = LearnedExpressionModel::train(&examples, 4, 100).unwrap();
        assert_eq!(first, second);

        let (program, lexical) = fixture();
        let selector = OfflineLearnedExpressionSelector::new(first);
        let first = selector
            .select(&program, &lexical, &projection(true))
            .unwrap();
        let second = selector
            .select(&program, &lexical, &projection(true))
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.payload.disposition,
            SelectionDisposition::LearnedVerified
        );
        assert_eq!(first.payload.selected_grammar_version, 3);
        assert!(first.payload.verification_digest.is_some());
    }

    #[test]
    fn voice_projection_changes_only_verified_variant_selection() {
        let model = LearnedExpressionModel::baseline().unwrap();
        let selector = OfflineLearnedExpressionSelector::new(model);
        let (program, lexical) = fixture();
        let direct = selector
            .select(&program, &lexical, &projection(true))
            .unwrap();
        let warm = selector
            .select(&program, &lexical, &projection(false))
            .unwrap();
        assert_eq!(
            direct.payload.disposition,
            SelectionDisposition::LearnedVerified
        );
        assert_eq!(
            warm.payload.disposition,
            SelectionDisposition::LearnedVerified
        );
        assert_ne!(direct.payload.variant_ids, warm.payload.variant_ids);
        assert_ne!(direct.payload.text, warm.payload.text);
    }

    #[test]
    fn tampering_is_rejected_and_corrupt_model_falls_back_exactly() {
        let (program, lexical) = fixture();
        let lattice = ExpressionLattice::build(&program, &lexical).unwrap();
        let verifier = GrammarV3Verifier;
        assert!(verifier
            .verify(
                &program,
                &lexical,
                lattice.digest,
                "Injected unsupported sentence.",
            )
            .is_err());

        let mut corrupt = LearnedExpressionModel::baseline().unwrap();
        corrupt.digest.0 = corrupt.digest.0.wrapping_add(1);
        let selector = OfflineLearnedExpressionSelector::new(corrupt);
        let result = selector
            .select(&program, &lexical, &projection(true))
            .unwrap();
        let neutral = VerifierReadyRenderer.render(&program, &lexical).unwrap();
        assert_eq!(
            result.payload.disposition,
            SelectionDisposition::NeutralFallback
        );
        assert_eq!(result.payload.text, neutral.payload.text);
        assert_eq!(result.payload.selected_grammar_version, 2);
    }

    #[test]
    fn authority_boundary_remains_offline_only() {
        let boundary = authority_boundary();
        assert!(boundary.candidate_lattice_construction);
        assert!(boundary.learned_candidate_scoring);
        assert!(boundary.independent_candidate_verification);
        assert!(!boundary.runtime_chat_wiring);
        assert!(!boundary.http_response_influence);
        assert!(!boundary.live_generated_text_influence);
        assert!(!boundary.raw_prompt_access);
        assert!(!boundary.unrestricted_conversation_access);
        assert!(!boundary.unrestricted_memory_access);
        assert!(!boundary.voice_state_mutation);
        assert!(!boundary.companion_state_access);
        assert!(!boundary.persistence_authority);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.routing_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.charge_discharge_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
