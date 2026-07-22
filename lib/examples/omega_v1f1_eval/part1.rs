use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use star::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalBindingTablePayload,
    MissingVariableLexicalBinding, ObservationLexicalBinding, PredictionLexicalBinding,
};
use star::learned_expression::{
    LearnedExpressionModel, PairwisePreference, PreferredSide, SelectionDisposition,
    VariantProfile, MAX_BEAM_WIDTH, MAX_MODEL_BYTES, MAX_RESPONSE_CANDIDATES,
    MAX_TRAINABLE_PARAMETERS, MAX_VARIANTS_PER_OPERATION,
};
use star::omega_v1f1_projection_guard::VerifiedVoiceProjection as LearnedVoiceProjection;
use star::semantic_response::{
    AbstentionReason, AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity,
    CognitiveStateVersion, ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation,
    DiscourseOperationKind, EpistemicConstraint, EpistemicStatus, MissingVariableId, ObservationId,
    OperationId, OutputBudget, PredictionId, ProhibitedClaim, ResponseProgramId,
    SemanticResponseIntent, SemanticResponseProgram, SemanticResponseProgramPayload,
    SemanticValidationContext, SensitivityLevel, SensitivityPolicy, StyleEnvelope, SubjectScope,
    VocabularyLevel,
};
use star::verifier_ready_realization::VerifierReadyRenderer;
use std::collections::{BTreeMap, BTreeSet};
use surface_diversity_v2::{
    authority_boundary, ClaimFirstLattice as ExpressionLattice,
    ClaimFirstLatticeDigest as ExpressionLatticeDigest,
    ClaimFirstOfflineSelector as OfflineLearnedExpressionSelector,
    ClaimFirstSurfaceVariant as OperationSurfaceVariant, ClaimFirstVerifier as GrammarV3Verifier,
};

const F1: &str = include_str!("../../fixtures/omega_v1f1/manifest.json");
const A: &str = include_str!("../../fixtures/omega_v1a/manifest.json");
const SHARDS: [(&str, &str); 7] = [
    (
        "ordinary",
        include_str!("../../fixtures/omega_v1a/ordinary.json"),
    ),
    (
        "technical",
        include_str!("../../fixtures/omega_v1a/technical.json"),
    ),
    (
        "emotional",
        include_str!("../../fixtures/omega_v1a/emotional.json"),
    ),
    (
        "disagreement",
        include_str!("../../fixtures/omega_v1a/disagreement.json"),
    ),
    (
        "uncertainty",
        include_str!("../../fixtures/omega_v1a/uncertainty.json"),
    ),
    (
        "continuity",
        include_str!("../../fixtures/omega_v1a/continuity.json"),
    ),
    (
        "adversarial",
        include_str!("../../fixtures/omega_v1a/adversarial.json"),
    ),
];
const SUBJECT: SubjectScope = SubjectScope(77);

#[derive(Clone, Deserialize)]
struct AM {
    category_requirements: BTreeMap<String, usize>,
    category_prohibited_claim_anchors: BTreeMap<String, Vec<String>>,
    profiles: BTreeMap<String, FP>,
}
#[derive(Clone, Deserialize)]
struct FP {
    confidence: f64,
    uncertainty: f64,
}
#[derive(Clone, Deserialize)]
struct Fx {
    id: String,
    category: String,
    prompt: String,
    #[serde(rename = "raw")]
    raw: String,
    profile: String,
    #[serde(rename = "expected")]
    expected: String,
    #[serde(rename = "required")]
    required: Vec<String>,
    #[serde(default, rename = "prohibited")]
    prohibited: Vec<String>,
}
#[derive(Deserialize)]
struct FM {
    schema_version: u16,
    experiment: String,
    source_corpus: String,
    split: SP,
    preference_evidence: PP,
    projection_profiles: Proj,
    thresholds: Th,
}
#[derive(Deserialize)]
struct SP {
    modulus: u16,
    test_remainder: u16,
    validation_remainder: u16,
    expected_totals: BTreeMap<String, usize>,
    expected_categories: BTreeMap<String, BTreeMap<String, usize>>,
}
#[derive(Deserialize)]
struct PP {
    schema_version: u16,
    left_candidate_id: String,
    right_candidate_id: String,
    evidence_source: String,
    reviewer: String,
    profile_preferences: BTreeMap<String, String>,
}
#[derive(Deserialize)]
struct Proj {
    direct: [u16; 7],
    warm: [u16; 7],
    neutral: [u16; 7],
}
#[derive(Deserialize)]
struct Th {
    preference_accuracy_min_bps: u16,
    state_pair_difference_min_bps: u16,
    shuffle_accuracy_drop_min_bps: u16,
    shuffled_accuracy_max_bps: u16,
    repeated_opener_relative_reduction_min_bps: u16,
    top_trigram_relative_reduction_min_bps: u16,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Split {
    Train,
    Validation,
    Test,
}
impl Split {
    fn s(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Validation => "validation",
            Self::Test => "test",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pref {
    Direct,
    Warm,
}
struct Case {
    fx: Fx,
    split: Split,
    pref: Pref,
    projection: LearnedVoiceProjection,
    program: SemanticResponseProgram,
    lexical: LexicalBindingTable,
}
