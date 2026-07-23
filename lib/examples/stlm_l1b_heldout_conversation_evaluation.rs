use anyhow::{ensure, Context, Result};
use serde::Serialize;
use star::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalBindingTablePayload,
    MissingVariableLexicalBinding, ObservationLexicalBinding, PredictionLexicalBinding,
};
use star::semantic_response::{
    AbstentionReason, AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity,
    CognitiveStateVersion, ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation,
    DiscourseOperationKind, EpistemicConstraint, EpistemicStatus, MissingVariableId, ObservationId,
    OperationId, OutputBudget, PredictionId, ProhibitedClaim, ResponseProgramId,
    SemanticResponseIntent, SemanticResponseProgram, SemanticResponseProgramPayload,
    SemanticValidationContext, SensitivityLevel, SensitivityPolicy, StyleEnvelope, SubjectScope,
    VocabularyLevel,
};
use star::verified_improvisation::{
    authority_boundary, ConversationalMicrostate, ImprovisationDisposition, ImprovisationRequest,
    ImprovisationalVerifier, RecentLanguageTrace, VerifiedImprovisationSelector,
};
use std::collections::BTreeSet;

const EXPERIMENT: &str = "STLM_L1B_HELDOUT_CONVERSATIONAL_EVALUATION";
const CORPUS_VERSION: &str = "stlm-l1b-heldout-v1";
const SEEDS_PER_SCENARIO: u64 = 16;

#[derive(Clone, Copy)]
struct ClaimSpec {
    key: &'static str,
    positive: &'static str,
    negative: &'static str,
    status: EpistemicStatus,
}

#[derive(Clone)]
enum OperationSpec {
    Assert(usize),
    Qualify(usize),
    Contrast(usize, usize),
    Correct(usize, usize),
    Explain(Vec<usize>),
    Acknowledge(&'static str),
    RequestEvidence(&'static str),
    Commit(&'static str),
    Abstain(AbstentionReason),
}

#[derive(Clone)]
struct ScenarioSpec {
    id: &'static str,
    intent: SemanticResponseIntent,
    detail: DetailLevel,
    claims: Vec<ClaimSpec>,
    operations: Vec<OperationSpec>,
    forbidden: &'static str,
}

#[derive(Debug, Serialize)]
struct ScenarioReport {
    id: String,
    selections: u64,
    verified_selections: u64,
    replay_matches: u64,
    neutral_divergences: u64,
    unique_surfaces: usize,
    unique_openings: usize,
    trace_changed_opening: bool,
    microstate_changed_surface: bool,
    legacy_lead_hits: u64,
    fallback_count: u64,
}

#[derive(Debug, Serialize)]
struct EvaluationReport {
    experiment: &'static str,
    corpus_version: &'static str,
    scenario_count: usize,
    total_selections: u64,
    verified_selections: u64,
    replay_matches: u64,
    neutral_divergences: u64,
    trace_opening_changes: u64,
    microstate_surface_changes: u64,
    fallback_count: u64,
    legacy_lead_hits: u64,
    minimum_unique_surfaces: usize,
    minimum_unique_openings: usize,
    verification_pass_rate_bps: u64,
    replay_pass_rate_bps: u64,
    neutral_divergence_rate_bps: u64,
    trace_opening_change_rate_bps: u64,
    microstate_response_rate_bps: u64,
    fallback_rate_bps: u64,
    authority_boundary_closed: bool,
    no_runtime_influence: bool,
    heldout_corpus_separate_from_unit_fixture: bool,
    gate_passed: bool,
    scenarios: Vec<ScenarioReport>,
}

fn main() -> Result<()> {
    let scenarios = heldout_scenarios();
    ensure!(scenarios.len() == 10, "held-out corpus size drifted");

    let selector = VerifiedImprovisationSelector;
    let verifier = ImprovisationalVerifier;
    let mut reports = Vec::with_capacity(scenarios.len());

    for (scenario_index, scenario) in scenarios.iter().enumerate() {
        reports.push(evaluate_scenario(
            scenario_index,
            scenario,
            selector,
            verifier,
        )?);
    }

    let total_selections = reports.iter().map(|report| report.selections).sum::<u64>();
    let verified_selections = reports
        .iter()
        .map(|report| report.verified_selections)
        .sum::<u64>();
    let replay_matches = reports
        .iter()
        .map(|report| report.replay_matches)
        .sum::<u64>();
    let neutral_divergences = reports
        .iter()
        .map(|report| report.neutral_divergences)
        .sum::<u64>();
    let trace_opening_changes = reports
        .iter()
        .filter(|report| report.trace_changed_opening)
        .count() as u64;
    let microstate_surface_changes = reports
        .iter()
        .filter(|report| report.microstate_changed_surface)
        .count() as u64;
    let fallback_count = reports
        .iter()
        .map(|report| report.fallback_count)
        .sum::<u64>();
    let legacy_lead_hits = reports
        .iter()
        .map(|report| report.legacy_lead_hits)
        .sum::<u64>();
    let minimum_unique_surfaces = reports
        .iter()
        .map(|report| report.unique_surfaces)
        .min()
        .unwrap_or_default();
    let minimum_unique_openings = reports
        .iter()
        .map(|report| report.unique_openings)
        .min()
        .unwrap_or_default();

    let scenario_count = reports.len() as u64;
    let verification_pass_rate_bps = rate_bps(verified_selections, total_selections);
    let replay_pass_rate_bps = rate_bps(replay_matches, total_selections);
    let neutral_divergence_rate_bps = rate_bps(neutral_divergences, total_selections);
    let trace_opening_change_rate_bps = rate_bps(trace_opening_changes, scenario_count);
    let microstate_response_rate_bps = rate_bps(microstate_surface_changes, scenario_count);
    let fallback_rate_bps = rate_bps(fallback_count, total_selections);

    let boundary = authority_boundary();
    let authority_boundary_closed = boundary.committed_surface_lattice
        && boundary.conversational_microstate_scoring
        && boundary.replayable_entropy_seed
        && boundary.recent_language_anti_repetition
        && boundary.independent_candidate_verification
        && !boundary.runtime_chat_wiring
        && !boundary.http_response_influence
        && !boundary.live_generated_text_influence
        && !boundary.raw_prompt_access
        && !boundary.unrestricted_conversation_access
        && !boundary.unrestricted_memory_access
        && !boundary.persistence_authority
        && !boundary.voice_state_mutation
        && !boundary.companion_state_mutation
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority;

    let gate_passed = verification_pass_rate_bps == 10_000
        && replay_pass_rate_bps == 10_000
        && neutral_divergence_rate_bps >= 9_000
        && trace_opening_change_rate_bps >= 7_000
        && microstate_response_rate_bps >= 6_000
        && fallback_count == 0
        && legacy_lead_hits == 0
        && minimum_unique_surfaces >= 2
        && minimum_unique_openings >= 2
        && authority_boundary_closed;

    let report = EvaluationReport {
        experiment: EXPERIMENT,
        corpus_version: CORPUS_VERSION,
        scenario_count: reports.len(),
        total_selections,
        verified_selections,
        replay_matches,
        neutral_divergences,
        trace_opening_changes,
        microstate_surface_changes,
        fallback_count,
        legacy_lead_hits,
        minimum_unique_surfaces,
        minimum_unique_openings,
        verification_pass_rate_bps,
        replay_pass_rate_bps,
        neutral_divergence_rate_bps,
        trace_opening_change_rate_bps,
        microstate_response_rate_bps,
        fallback_rate_bps,
        authority_boundary_closed,
        no_runtime_influence: authority_boundary_closed,
        heldout_corpus_separate_from_unit_fixture: true,
        gate_passed,
        scenarios: reports,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    ensure!(gate_passed, "STLM L1-B held-out gate failed");
    Ok(())
}

fn evaluate_scenario(
    scenario_index: usize,
    scenario: &ScenarioSpec,
    selector: VerifiedImprovisationSelector,
    verifier: ImprovisationalVerifier,
) -> Result<ScenarioReport> {
    let (program, lexical_table) = build_fixture(scenario_index, scenario)?;
    let neutral = star::verifier_ready_realization::VerifierReadyRenderer
        .render(&program, &lexical_table)
        .context("neutral control must render")?;

    let mut texts = BTreeSet::new();
    let mut openings = BTreeSet::new();
    let mut verified_selections = 0;
    let mut replay_matches = 0;
    let mut neutral_divergences = 0;
    let mut fallback_count = 0;
    let mut legacy_lead_hits = 0;
    let seed_base = 10_000_u64 + (scenario_index as u64 * 1_000);

    for offset in 0..SEEDS_PER_SCENARIO {
        let request = ImprovisationRequest::new(
            seed_base + offset,
            ConversationalMicrostate::default(),
            RecentLanguageTrace::default(),
        )?;
        let selected = selector.select(&program, &lexical_table, &request)?;
        let replay = selector.select(&program, &lexical_table, &request)?;
        if selected == replay {
            replay_matches += 1;
        }
        if selected.payload.disposition == ImprovisationDisposition::VerifiedImprovisation {
            verified_selections += 1;
        } else {
            fallback_count += 1;
        }
        if selected.payload.text != neutral.payload.text {
            neutral_divergences += 1;
        }
        if has_legacy_lead(&selected.payload.text) {
            legacy_lead_hits += 1;
        }

        let lattice_digest = selected
            .payload
            .lattice_digest
            .context("verified selection must carry lattice digest")?;
        let report = verifier.verify(
            &program,
            &lexical_table,
            lattice_digest,
            &selected.payload.text,
        )?;
        ensure!(
            selected.payload.verification_digest == Some(report.digest),
            "selection verification digest mismatch"
        );

        texts.insert(selected.payload.text);
        openings.insert(selected.payload.opening_fingerprint);
    }

    let control_seed = seed_base + 91;
    let baseline_request = ImprovisationRequest::new(
        control_seed,
        ConversationalMicrostate::default(),
        RecentLanguageTrace::default(),
    )?;
    let baseline = selector.select(&program, &lexical_table, &baseline_request)?;
    let mut trace = RecentLanguageTrace::default();
    trace.record_text(&baseline.payload.text)?;
    let trace_request =
        ImprovisationRequest::new(control_seed, ConversationalMicrostate::default(), trace)?;
    let trace_treatment = selector.select(&program, &lexical_table, &trace_request)?;
    let trace_changed_opening =
        baseline.payload.opening_fingerprint != trace_treatment.payload.opening_fingerprint;

    let direct = ConversationalMicrostate::new(9_000, 2_000, 7_000, 8_500, 1_000, 6_500)?;
    let warm = ConversationalMicrostate::new(5_500, 9_000, 5_500, 4_500, 4_500, 6_500)?;
    let direct_request =
        ImprovisationRequest::new(seed_base + 177, direct, RecentLanguageTrace::default())?;
    let warm_request =
        ImprovisationRequest::new(seed_base + 177, warm, RecentLanguageTrace::default())?;
    let direct_selection = selector.select(&program, &lexical_table, &direct_request)?;
    let warm_selection = selector.select(&program, &lexical_table, &warm_request)?;
    let microstate_changed_surface = direct_selection.payload.text != warm_selection.payload.text;

    Ok(ScenarioReport {
        id: scenario.id.to_owned(),
        selections: SEEDS_PER_SCENARIO,
        verified_selections,
        replay_matches,
        neutral_divergences,
        unique_surfaces: texts.len(),
        unique_openings: openings.len(),
        trace_changed_opening,
        microstate_changed_surface,
        legacy_lead_hits,
        fallback_count,
    })
}

fn build_fixture(
    scenario_index: usize,
    scenario: &ScenarioSpec,
) -> Result<(SemanticResponseProgram, LexicalBindingTable)> {
    let scope = SubjectScope(50_000 + scenario_index as u64);
    let state = CognitiveStateVersion(700 + scenario_index as u64);
    let claims = scenario
        .claims
        .iter()
        .enumerate()
        .map(|(index, spec)| claim(index, spec, scope))
        .collect::<Vec<_>>();
    let epistemic_constraints = claims
        .iter()
        .map(|claim| {
            let (minimum_confidence_bps, maximum_confidence_bps) =
                claim.epistemic_status.confidence_bounds();
            EpistemicConstraint {
                claim: claim.id,
                required_status: claim.epistemic_status,
                minimum_confidence_bps,
                maximum_confidence_bps,
            }
        })
        .collect::<Vec<_>>();

    let mut observation_labels = Vec::new();
    let mut variable_labels = Vec::new();
    let mut prediction_labels = Vec::new();
    let operations = scenario
        .operations
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            let kind = match operation {
                OperationSpec::Assert(claim) => DiscourseOperationKind::Assert(claim_id(*claim)),
                OperationSpec::Qualify(claim) => DiscourseOperationKind::Qualify {
                    claim: claim_id(*claim),
                    status: scenario.claims[*claim].status,
                },
                OperationSpec::Contrast(left, right) => DiscourseOperationKind::Contrast {
                    left: claim_id(*left),
                    right: claim_id(*right),
                },
                OperationSpec::Correct(prior, replacement) => DiscourseOperationKind::Correct {
                    prior: claim_id(*prior),
                    replacement: claim_id(*replacement),
                },
                OperationSpec::Explain(references) => DiscourseOperationKind::Explain {
                    claims: references.iter().map(|claim| claim_id(*claim)).collect(),
                },
                OperationSpec::Acknowledge(label) => {
                    let id = ObservationId(observation_labels.len() as u64 + 1);
                    observation_labels.push((id, *label));
                    DiscourseOperationKind::Acknowledge(id)
                }
                OperationSpec::RequestEvidence(label) => {
                    let id = MissingVariableId(variable_labels.len() as u64 + 1);
                    variable_labels.push((id, *label));
                    DiscourseOperationKind::RequestEvidence(id)
                }
                OperationSpec::Commit(label) => {
                    let id = PredictionId(prediction_labels.len() as u64 + 1);
                    prediction_labels.push((id, *label));
                    DiscourseOperationKind::Commit(id)
                }
                OperationSpec::Abstain(reason) => DiscourseOperationKind::Abstain(*reason),
            };
            DiscourseOperation {
                id: OperationId(index as u64 + 1),
                kind,
            }
        })
        .collect::<Vec<_>>();

    let payload = SemanticResponseProgramPayload {
        id: ResponseProgramId(80_000 + scenario_index as u64),
        source_state_version: state,
        companion_state_version: None,
        subject_scope: scope,
        intent: scenario.intent,
        operations,
        required_claims: claims,
        optional_claims: Vec::new(),
        prohibited_claims: vec![ProhibitedClaim {
            id: ClaimId(9_000 + scenario_index as u64),
            semantic_key: format!("{}_forbidden", scenario.id),
        }],
        epistemic_constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: SensitivityLevel::Public,
            disclosure_scope: scope,
        },
        style: StyleEnvelope {
            detail: scenario.detail,
            vocabulary: VocabularyLevel::Standard,
            dialogue: DialogueMode::Collaborative,
            acknowledgment: AcknowledgmentLevel::Brief,
            allow_first_person: true,
            allow_questions: true,
            maximum_paragraphs: 3,
        },
        output_budget: OutputBudget {
            maximum_characters: 8_000,
            maximum_sentences: 32,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 24,
            maximum_claims: 32,
            maximum_verification_steps: 512,
        },
    };
    let program = SemanticResponseProgram::validate(
        payload,
        SemanticValidationContext {
            cognitive_state_version: state,
            companion_state_version: None,
            subject_scope: scope,
        },
    )?;

    let lexical_table = LexicalBindingTable::validate(
        LexicalBindingTablePayload {
            program_digest: program.digest,
            subject_scope: scope,
            claims: scenario
                .claims
                .iter()
                .enumerate()
                .map(|(index, spec)| ClaimLexicalBinding {
                    claim: claim_id(index),
                    positive_clause: spec.positive.to_owned(),
                    negative_clause: spec.negative.to_owned(),
                })
                .collect(),
            observations: observation_labels
                .into_iter()
                .map(|(observation, label)| ObservationLexicalBinding {
                    observation,
                    label: label.to_owned(),
                })
                .collect(),
            missing_variables: variable_labels
                .into_iter()
                .map(|(variable, label)| MissingVariableLexicalBinding {
                    variable,
                    label: label.to_owned(),
                })
                .collect(),
            predictions: prediction_labels
                .into_iter()
                .map(|(prediction, label)| PredictionLexicalBinding {
                    prediction,
                    label: label.to_owned(),
                })
                .collect(),
            forbidden_surface_forms: vec![scenario.forbidden.to_owned()],
        },
        &program,
    )?;

    Ok((program, lexical_table))
}

fn claim(index: usize, spec: &ClaimSpec, scope: SubjectScope) -> AuthorizedClaim {
    AuthorizedClaim {
        id: claim_id(index),
        semantic_key: spec.key.to_owned(),
        polarity: ClaimPolarity::Positive,
        confidence_bps: confidence(spec.status),
        epistemic_status: spec.status,
        sensitivity: SensitivityLevel::Public,
        disclosure_scope: scope,
    }
}

fn claim_id(zero_based: usize) -> ClaimId {
    ClaimId(zero_based as u64 + 1)
}

fn confidence(status: EpistemicStatus) -> u16 {
    match status {
        EpistemicStatus::Certain => 9_500,
        EpistemicStatus::Probable => 8_000,
        EpistemicStatus::Possible => 5_000,
        EpistemicStatus::Uncertain => 2_000,
        EpistemicStatus::Unknown => 0,
    }
}

fn rate_bps(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        0
    } else {
        numerator.saturating_mul(10_000) / denominator
    }
}

fn has_legacy_lead(text: &str) -> bool {
    [
        "Directly: ",
        "In brief: ",
        "The clearest answer: ",
        "With context preserved: ",
        "A grounded reading: ",
        "The point I would keep: ",
    ]
    .iter()
    .any(|lead| text.starts_with(lead))
}

fn heldout_scenarios() -> Vec<ScenarioSpec> {
    vec![
        ScenarioSpec {
            id: "iteration_tradeoff",
            intent: SemanticResponseIntent::Contrast,
            detail: DetailLevel::Standard,
            claims: vec![
                ClaimSpec {
                    key: "rapid_iteration_reveals_design_pressure",
                    positive: "rapid iteration reveals design pressure",
                    negative: "rapid iteration does not reveal design pressure",
                    status: EpistemicStatus::Certain,
                },
                ClaimSpec {
                    key: "rapid_iteration_can_hide_regressions",
                    positive: "rapid iteration can hide regressions",
                    negative: "rapid iteration cannot hide regressions",
                    status: EpistemicStatus::Probable,
                },
            ],
            operations: vec![
                OperationSpec::Acknowledge("the pressure to move quickly"),
                OperationSpec::Contrast(0, 1),
            ],
            forbidden: "speed proves correctness",
        },
        ScenarioSpec {
            id: "fluency_correction",
            intent: SemanticResponseIntent::Correction,
            detail: DetailLevel::Standard,
            claims: vec![
                ClaimSpec {
                    key: "surface_fluency_proves_reasoning",
                    positive: "surface fluency proves reasoning",
                    negative: "surface fluency does not prove reasoning",
                    status: EpistemicStatus::Possible,
                },
                ClaimSpec {
                    key: "surface_fluency_supports_usability",
                    positive: "surface fluency supports usability",
                    negative: "surface fluency does not support usability",
                    status: EpistemicStatus::Certain,
                },
            ],
            operations: vec![OperationSpec::Correct(0, 1), OperationSpec::Assert(1)],
            forbidden: "fluent output is consciousness",
        },
        ScenarioSpec {
            id: "causal_explanation",
            intent: SemanticResponseIntent::Explanation,
            detail: DetailLevel::Detailed,
            claims: vec![
                ClaimSpec {
                    key: "bounded_search_limits_surface_space",
                    positive: "bounded search limits the surface space",
                    negative: "bounded search does not limit the surface space",
                    status: EpistemicStatus::Certain,
                },
                ClaimSpec {
                    key: "independent_verification_preserves_claims",
                    positive: "independent verification preserves the authorized claims",
                    negative: "independent verification does not preserve the authorized claims",
                    status: EpistemicStatus::Probable,
                },
                ClaimSpec {
                    key: "recent_language_pressure_reduces_repetition",
                    positive: "recent language pressure can reduce repetition",
                    negative: "recent language pressure cannot reduce repetition",
                    status: EpistemicStatus::Possible,
                },
            ],
            operations: vec![
                OperationSpec::Acknowledge("the request for a causal account"),
                OperationSpec::Explain(vec![0, 1, 2]),
            ],
            forbidden: "variation guarantees truth",
        },
        ScenarioSpec {
            id: "uncertainty_disclosure",
            intent: SemanticResponseIntent::SelfCheck,
            detail: DetailLevel::Standard,
            claims: vec![ClaimSpec {
                key: "evidence_supports_only_a_partial_answer",
                positive: "the evidence supports only a partial answer",
                negative: "the evidence does not support only a partial answer",
                status: EpistemicStatus::Uncertain,
            }],
            operations: vec![
                OperationSpec::Acknowledge("the unresolved evidence gap"),
                OperationSpec::Qualify(0),
            ],
            forbidden: "uncertainty is certainty",
        },
        ScenarioSpec {
            id: "relational_acknowledgment",
            intent: SemanticResponseIntent::RelationalAcknowledgment,
            detail: DetailLevel::Brief,
            claims: vec![ClaimSpec {
                key: "concern_about_canned_language_is_legitimate",
                positive: "the concern about canned language is legitimate",
                negative: "the concern about canned language is not legitimate",
                status: EpistemicStatus::Certain,
            }],
            operations: vec![
                OperationSpec::Acknowledge("the frustration with canned language"),
                OperationSpec::Assert(0),
            ],
            forbidden: "emotion proves accuracy",
        },
        ScenarioSpec {
            id: "evidence_request",
            intent: SemanticResponseIntent::EvidenceRequest,
            detail: DetailLevel::Brief,
            claims: vec![ClaimSpec {
                key: "current_record_is_insufficient",
                positive: "the current record is insufficient",
                negative: "the current record is not insufficient",
                status: EpistemicStatus::Certain,
            }],
            operations: vec![
                OperationSpec::Assert(0),
                OperationSpec::RequestEvidence("the missing timestamped source record"),
            ],
            forbidden: "missing evidence proves the claim",
        },
        ScenarioSpec {
            id: "bounded_commitment",
            intent: SemanticResponseIntent::Commitment,
            detail: DetailLevel::Standard,
            claims: vec![ClaimSpec {
                key: "the_next_step_is_a_heldout_replay",
                positive: "the next step is a held-out replay",
                negative: "the next step is not a held-out replay",
                status: EpistemicStatus::Certain,
            }],
            operations: vec![
                OperationSpec::Assert(0),
                OperationSpec::Commit("rerun the frozen corpus before live influence"),
            ],
            forbidden: "commitment bypasses evidence",
        },
        ScenarioSpec {
            id: "careful_abstention",
            intent: SemanticResponseIntent::Abstention,
            detail: DetailLevel::Brief,
            claims: Vec::new(),
            operations: vec![
                OperationSpec::Acknowledge("the pressure for a definitive answer"),
                OperationSpec::Abstain(AbstentionReason::InsufficientEvidence),
            ],
            forbidden: "abstention confirms the claim",
        },
        ScenarioSpec {
            id: "capability_boundary",
            intent: SemanticResponseIntent::CapabilityDisclosure,
            detail: DetailLevel::Standard,
            claims: vec![
                ClaimSpec {
                    key: "selector_changes_wording_only",
                    positive: "the selector changes wording only",
                    negative: "the selector does not change wording only",
                    status: EpistemicStatus::Certain,
                },
                ClaimSpec {
                    key: "selector_has_no_live_runtime_authority",
                    positive: "the selector has no live runtime authority",
                    negative: "the selector has live runtime authority",
                    status: EpistemicStatus::Certain,
                },
            ],
            operations: vec![OperationSpec::Explain(vec![0, 1])],
            forbidden: "offline evaluation is live deployment",
        },
        ScenarioSpec {
            id: "factual_status",
            intent: SemanticResponseIntent::FactualAnswer,
            detail: DetailLevel::Standard,
            claims: vec![
                ClaimSpec {
                    key: "heldout_evaluation_is_offline",
                    positive: "the held-out evaluation is offline",
                    negative: "the held-out evaluation is not offline",
                    status: EpistemicStatus::Certain,
                },
                ClaimSpec {
                    key: "live_integration_requires_a_separate_gate",
                    positive: "live integration requires a separate gate",
                    negative: "live integration does not require a separate gate",
                    status: EpistemicStatus::Certain,
                },
            ],
            operations: vec![OperationSpec::Assert(0), OperationSpec::Assert(1)],
            forbidden: "evaluation automatically enables deployment",
        },
    ]
}
