# Starfire State-Transition Language Model Architecture

**Status:** proposed architecture  
**Date:** 2026-07-13  
**Authority:** documentation only; no runtime capability or scientific result is claimed  
**Related work:** `plans/CLOSED_COGNITIVE_CYCLE_AGI_PLAN.md`, `docs/INGEXUITY_STARFIRE_INTEGRATION.md`, `plans/VOICE_OVERHAUL.md`

## 1. Purpose

Starfire needs fluent language without surrendering cognition to an opaque general-purpose language model.

A conventional integration would place a large model behind `Runtime::chat()`, inject memories and instructions into a prompt, and treat the returned text as Starfire's answer. That would improve surface fluency quickly, but it would also make the external model the effective reasoner. Starfire would become an orchestration and memory wrapper around another system's cognition.

This proposal defines a different architecture:

> Starfire first performs a typed, replayable cognitive state transition; then it selects an authorized semantic response program; only then may a language model realize that program as text. The realized text must carry enough evidence to be independently checked against the state that authorized it.

The proposed system is called the **Starfire State-Transition Language Model** (`STLM`). Its user-visible language component is the **Proof-Carrying Renderer** (`PCR`).

## 2. Core hypothesis

A useful language system does not need to own all reasoning.

The research hypothesis is:

> A comparatively small trainable model can produce fluent, context-sensitive language from an explicit machine cognitive state while preserving claim identity, provenance, uncertainty, sensitivity, and discourse obligations; an independent verifier can reject outputs that introduce unsupported semantic content.

The architecture is successful only if cognition remains attributable to Starfire rather than to the renderer.

## 3. Anti-wrapper criterion

The system is not considered Starfire-native merely because the model runs in-process or is written in Rust.

The decisive question is where task-relevant semantic work occurs.

### 3.1 Wrapper failure mode

```text
user text
   |
   v
retrieve memories and construct prompt
   |
   v
general language model reasons and writes answer
   |
   v
store output and metadata
```

This is an LLM wrapper even when the surrounding framework has memory, tools, personality, or autonomous loops.

### 3.2 Starfire-native target

```text
user observation
   |
   v
validated observation event
   |
   v
Starfire cognitive transition
   |
   v
validated state patch
   |
   v
authorized semantic response program
   |
   v
proof-carrying surface realization
   |
   v
independent semantic verification
   |
   +--> accept
   +--> repair within the same authorization
   +--> deterministic neutral fallback
```

### 3.3 Required attribution tests

The architecture must eventually demonstrate all of the following:

1. **Renderer substitution:** replacing the renderer changes style more than semantic correctness.
2. **Cognition ablation:** disabling Starfire's transition and discourse layers causes a substantial correctness drop.
3. **Renderer restriction:** the renderer cannot access raw memory, tools, hidden state, or the unrestricted conversation transcript.
4. **Unsupported-claim rejection:** a renderer that adds an unlicensed proposition is rejected.
5. **Uncertainty preservation:** possibility, suspicion, confidence, and abstention survive realization.
6. **Sensitive-state noninterference:** sensitive claims absent from the authorization packet cannot leak into the output.
7. **Replay:** the semantic decision is reproducible independently of wording variation.

Until those tests pass under frozen held-out evaluation, STLM is a proposed architecture rather than evidence that Starfire is not a wrapper.

## 4. Three-layer model

STLM separates cognition, discourse, and language realization.

### 4.1 Cognitive Transition Model

The Cognitive Transition Model (`CTM`) maps a validated observation and prior cognitive state to a proposed typed state patch.

```text
(current state, validated observation, active goals, budget)
    -> proposed state patch
```

The CTM may initially be deterministic and hand-authored. A learned CTM is a later research stage and must remain proposal-only until independently validated.

The CTM does not emit user-facing prose.

### 4.2 Discourse Planner

The Discourse Planner (`DP`) decides what Starfire is authorized and obligated to communicate from the validated post-transition state.

```text
(validated state, user intent, interaction policy, disclosure policy)
    -> semantic response program
```

The program contains typed operations such as:

```text
Assert(claim_id)
Qualify(claim_id, epistemic_status)
Contrast(left_claim_id, right_claim_id)
Correct(previous_claim_id, replacement_claim_id)
Explain(causal_path_id)
Acknowledge(observation_id)
RequestEvidence(missing_variable_id)
Commit(prediction_id)
Abstain(reason_id)
```

The planner decides the semantic content. It does not need to decide exact phrasing.

### 4.3 Proof-Carrying Renderer

The Proof-Carrying Renderer maps the semantic response program to natural language plus an alignment witness.

```text
semantic response program
    -> rendered text + sentence/span alignment + renderer trace digest
```

The renderer may be neural, symbolic, recurrent, transformer-based, state-space based, grammar-based, or hybrid. The architecture intentionally does not require one model family.

The renderer has no authority to:

- add a claim;
- promote a belief;
- alter confidence;
- read unrestricted memory;
- invoke tools;
- select actions;
- mutate companion state;
- discharge CHARGE;
- judge its own semantic fidelity.

## 5. Independent verifier

The Independent Language Verifier (`ILV`) evaluates whether the rendered text remains within the semantic authorization.

It must not simply trust renderer-provided alignments.

The verifier reconstructs a normalized semantic interpretation from the text and compares it against the program:

```text
rendered text
    -> reconstructed acts, claims, qualifiers, references, and omissions
```

It checks:

- required semantic operations were expressed;
- no unsupported claims were introduced;
- claim polarity was preserved;
- relationships and causal direction were preserved;
- epistemic status was preserved;
- required caveats were not dropped;
- prohibited claims were not implied;
- sensitive identifiers did not appear;
- references resolve to authorized entities;
- output remained inside length and compute budgets.

The ILV is an independent judge under the same principle used elsewhere in Starfire: a resolver never judges its own success.

Acceptable verifier implementations include:

- deterministic typed parsers for constrained forms;
- a separately trained inverse model with disjoint parameters and data controls;
- an ensemble requiring agreement between symbolic and learned checks;
- an external evaluator in offline experiments only.

The production verifier must fail closed to a deterministic neutral renderer when semantic equivalence cannot be established.

## 6. Native data contracts

The following Rust sketches define the intended boundary. Names and exact fields are provisional, but authority separation is not.

```rust
pub struct CognitiveSnapshot {
    pub version: CognitiveVersion,
    pub active_claims: Vec<ClaimRef>,
    pub unresolved_charge: Vec<ChargeRef>,
    pub active_goals: Vec<GoalRef>,
    pub conversation_state: ConversationStateRef,
    pub companion_version: Option<CompanionVersion>,
    pub disclosure_scope: DisclosureScope,
    pub budget: CognitiveBudget,
}

pub struct ProposedStatePatch {
    pub base_version: CognitiveVersion,
    pub operations: Vec<StateOperation>,
    pub evidence: Vec<EvidenceRef>,
    pub predicted_effects: Vec<PredictedEffect>,
    pub proposer: TransitionProposer,
    pub digest: PatchDigest,
}

pub struct ValidatedStatePatch {
    pub prior_version: CognitiveVersion,
    pub next_version: CognitiveVersion,
    pub operations: Vec<ValidatedStateOperation>,
    pub evidence: Vec<EvidenceRef>,
    pub validation: ValidationCertificate,
}
```

The discourse boundary:

```rust
pub struct SemanticResponseProgram {
    pub id: ResponseProgramId,
    pub source_state_version: CognitiveVersion,
    pub subject_scope: SubjectScope,
    pub intent: ResponseIntent,
    pub operations: Vec<DiscourseOperation>,
    pub required_claims: Vec<AuthorizedClaim>,
    pub optional_claims: Vec<AuthorizedClaim>,
    pub prohibited_claims: Vec<ProhibitedClaim>,
    pub uncertainty: Vec<EpistemicConstraint>,
    pub sensitivity: SensitivityPolicy,
    pub style: StyleEnvelope,
    pub output_budget: OutputBudget,
    pub digest: ResponseProgramDigest,
}

pub enum DiscourseOperation {
    Assert(ClaimId),
    Qualify { claim: ClaimId, status: EpistemicStatus },
    Contrast { left: ClaimId, right: ClaimId },
    Correct { prior: ClaimId, replacement: ClaimId },
    Explain { path: ExplanationPathId },
    Acknowledge(ObservationId),
    RequestEvidence(MissingVariableId),
    Commit(PredictionId),
    Abstain(AbstentionReason),
}
```

The realization boundary:

```rust
pub struct SurfaceRealization {
    pub program_digest: ResponseProgramDigest,
    pub text: String,
    pub alignments: Vec<SemanticAlignment>,
    pub renderer: RendererIdentity,
    pub renderer_trace_digest: RendererTraceDigest,
    pub generation_cost: ComputeCost,
}

pub struct SemanticAlignment {
    pub text_span: TextSpan,
    pub operation_index: usize,
    pub claim_ids: Vec<ClaimId>,
    pub relation_ids: Vec<RelationId>,
}

pub enum VerificationVerdict {
    Accept(VerifiedRealization),
    Repairable(RepairRequest),
    Reject(VerificationFailure),
}
```

No renderer output becomes a chat response until it receives an `Accept` verdict or the system uses an authorized deterministic fallback.

## 7. Renderer information firewall

Preventing bypass is essential. A powerful renderer given the full conversation and memory can reconstruct the task and perform hidden reasoning even when a semantic packet exists.

The renderer therefore receives only:

- the semantic response program;
- opaque lexical handles needed to name authorized entities;
- bounded style state;
- bounded local dialogue form, when necessary for pronouns or ellipsis;
- no raw sensitive memory not selected for disclosure;
- no tool handles;
- no world-model mutation handle;
- no persistence handle;
- no unrestricted retrieval interface.

A renderer may receive a `LexicalBindingTable`:

```rust
pub struct LexicalBindingTable {
    pub entities: Vec<LexicalEntityBinding>,
    pub terms: Vec<LexicalTermBinding>,
    pub forbidden_surface_forms: Vec<String>,
}
```

The table contains only surface forms already authorized by the planner. Opaque internal IDs remain the source of truth.

## 8. Semantic response program example

For the question, "Isn't that just wrapping an LLM?", the planner might authorize:

```text
intent: architectural_challenge
source_state_version: 184
required operations:
  1. Acknowledge(observation: wrapper_risk_is_valid)
  2. Assert(claim: unrestricted_llm_would_own_cognition)
  3. Contrast(
       left: unrestricted_reasoning_model,
       right: constrained_surface_renderer
     )
  4. Qualify(
       claim: current_starfire_independent_semantics_are_limited,
       status: high_confidence
     )
prohibited claims:
  - current_starfire_has_general_language_understanding
  - fluency_proves_cognition
style:
  directness: high
  detail: medium
  warmth: neutral
budget:
  max_chars: 1800
```

A valid realization can vary in wording while preserving those operations. A fluent answer that omits the current limitation or claims that Starfire is already independent must fail verification.

## 9. Training formulation

The initial learned component should be the renderer, not the cognitive transition model.

### 9.1 Forward task

```text
SemanticResponseProgram + LexicalBindingTable + StyleEnvelope
    -> SurfaceRealization
```

### 9.2 Inverse task

```text
SurfaceRealization
    -> reconstructed SemanticResponseProgram projection
```

### 9.3 Candidate objective

A future training objective may combine:

```text
L =
    lambda_text        * L_text_fluency
  + lambda_coverage    * L_required_operation_coverage
  + lambda_claim       * L_claim_fidelity
  + lambda_relation    * L_relation_fidelity
  + lambda_uncertainty * L_epistemic_preservation
  + lambda_alignment   * L_span_alignment
  + lambda_cycle       * L_forward_inverse_cycle
  + lambda_privacy     * L_sensitive_leakage
  + lambda_budget      * L_compute_and_length
  + lambda_extra       * L_unsupported_semantics
```

Text similarity is deliberately not the sole objective. Multiple valid wordings should be accepted when they preserve the semantic program.

## 10. Relationship to existing Starfire components

### 10.1 `runtime::response_intent`

The existing typed `ResponseIntent` becomes one input to `SemanticResponseProgram`, not the complete semantic plan.

### 10.2 `language_model::intent_reranker`

The existing reranker is an evolutionary predecessor. It currently rewrites an assembled body. STLM moves the trust boundary earlier: future renderers should receive a semantic program rather than an already written answer plus unrestricted context.

The existing reranker remains available as a control and migration adapter.

### 10.3 `voice::VoiceEngine`

Voice becomes a bounded style realization layer. Personality may influence `StyleEnvelope`; it may not create factual claims.

### 10.4 Companion S0-S6

Validated companion state may influence policy dimensions such as detail, vocabulary, acknowledgment, and dialogue mode after the existing promotion gates. It does not authorize factual claims about the user unless those claims are independently active, in scope, non-sensitive for the context, and explicitly selected by the discourse planner.

S6-A remains a bounded response-planning predecessor. Default runtime integration still requires the separately evaluated S6-B and S6-C boundaries.

### 10.5 CHARGE and closed cognitive cycle

Language failures may emit typed unresolved computation, for example:

- repeated inability to express a relation within the current discourse grammar;
- verifier rejection due to unsupported semantic drift;
- ambiguity that cannot be resolved from authorized state;
- inability to preserve uncertainty under the current renderer.

The renderer cannot discharge its own failure charge. Independent evidence must show improved semantic fidelity or successful held-out realization.

### 10.6 Representation research

H12/H13-C-style proof-carrying roles may later supply opaque semantic structures to the discourse planner. Their presence does not grant automatic language or runtime authority. A separately validated lexicalization bridge is required.

## 11. Deployment modes

STLM should support four explicit modes.

### Mode A: deterministic baseline

```text
semantic program -> deterministic grammar renderer -> verifier
```

This is the reference implementation and safety fallback.

### Mode B: local learned renderer

```text
semantic program -> local small model -> verifier
```

The intended first learned deployment.

### Mode C: comparative shadow rendering

Multiple renderers produce candidates, but none affects the user-visible response. Offline or independently witnessed evaluation compares semantic fidelity, fluency, burden, latency, and compute.

### Mode D: bounded live canary

A renderer that passed held-out evaluation may influence a limited class of low-risk responses under explicit duration, subject, and turn budgets with immediate deterministic fallback.

No mode grants action authority.

## 12. Failure and fallback behavior

The system must return a safe response even when the renderer or verifier fails.

```text
renderer load failure
    -> deterministic renderer

renderer timeout or budget exhaustion
    -> deterministic renderer

verification rejection
    -> one bounded repair attempt using verifier constraints
    -> deterministic renderer if repair fails

source state version drift
    -> reject stale program
    -> replan or neutral fallback

sensitivity mismatch
    -> omit sensitive operation or abstain
```

Repair cannot expand the original semantic authorization.

## 13. Evaluation dimensions

Future experiments should measure at least:

- required-operation coverage;
- unsupported-claim rate;
- claim polarity preservation;
- relation-direction preservation;
- uncertainty calibration preservation;
- correction and supersession fidelity;
- sensitive-information leakage;
- reference resolution accuracy;
- deterministic replay of semantic decisions;
- renderer substitution invariance;
- fluency and human preference;
- clarification burden;
- latency, memory, and compute;
- neutral-fallback equivalence.

Fluency alone is insufficient.

## 14. Non-goals

This proposal does not claim:

- that a new neural primitive has already been invented;
- that transformers must be rejected;
- that typed cognition is sufficient for general intelligence;
- that the renderer cannot learn useful linguistic abstractions;
- that Starfire currently produces complete semantic plans for unrestricted conversation;
- that a learned CTM is safe to run live;
- that fluent language demonstrates consciousness or AGI.

## 15. Public value

If successful, the architecture could support systems where language is accountable to explicit machine state:

- local personal assistants with user-owned memory;
- clinical, legal, or financial drafting with claim-level authorization;
- robots whose explanations correspond to validated plans;
- educational systems that preserve uncertainty and cite internal evidence;
- low-parameter language systems paired with structured cognition;
- agents whose wording model can be replaced without replacing their memory and reasoning authority.

The potential contribution is not merely a Starfire-specific voice model. It is a general interface between explicit cognition and verifiable natural language.

## 16. Architectural claim boundary

This document defines a research target.

A future implementation may be called a **Starfire-native language model** only after frozen ablations demonstrate that:

1. Starfire determines the task-relevant semantic content before rendering;
2. the renderer cannot bypass the semantic boundary;
3. an independent verifier rejects unsupported content;
4. renderer replacement preserves semantic task performance;
5. removing Starfire cognition materially reduces performance;
6. held-out real interaction evidence supports live use.

Before those gates pass, the correct description is:

> A proposed state-transition and proof-carrying language architecture under development inside Starfire.
