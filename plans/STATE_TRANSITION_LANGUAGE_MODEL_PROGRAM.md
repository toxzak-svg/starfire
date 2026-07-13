# Starfire State-Transition Language Model Program

**Status:** implementation and falsification plan  
**Date:** 2026-07-13  
**Architecture:** `docs/architecture/STATE_TRANSITION_LANGUAGE_MODEL.md`  
**Initial authority:** documentation and shadow evaluation only

## 1. Program objective

Build and test a Starfire-native language system in which:

1. Starfire performs the cognitive state transition;
2. Starfire selects the semantic content of the response;
3. a bounded renderer converts that content into fluent language;
4. an independent verifier checks that the text is faithful;
5. deterministic fallback remains available at every stage;
6. ablations identify where capability actually resides.

The program must improve conversational fluency without hiding a general-purpose reasoning model behind memory retrieval and prompt construction.

## 2. Research questions

The program is organized around five falsifiable questions.

### Q1. Can Starfire express its response decisions as typed semantic programs?

A typed program must represent required claims, relations, uncertainty, corrections, abstentions, sensitivity, and discourse operations without relying on prewritten prose as the source of truth.

### Q2. Can multiple renderers express the same program with stable meaning?

A deterministic renderer, a small learned renderer, and later alternative model families should preserve the same semantic decision while varying wording and style.

### Q3. Can an independent verifier detect semantic drift?

The verifier must reject unsupported claims, polarity reversals, dropped caveats, incorrect confidence, sensitive leakage, and stale-state realizations.

### Q4. Does Starfire supply the task-relevant cognition?

Removing the transition and discourse layers must reduce semantic task performance substantially more than replacing the renderer.

### Q5. Does the architecture improve real conversations?

Only independently witnessed held-out outcomes may justify bounded live influence. Synthetic fixtures can validate mechanics, not user benefit.

## 3. Program invariants

These rules apply to every phase.

1. A renderer never validates its own output.
2. Generated text is not a state mutation request unless separately parsed, proposed, and validated.
3. The renderer receives no unrestricted memory, tools, persistence, or action handles.
4. A semantic program is bound to an exact cognitive-state version and companion-state version when applicable.
5. User correction outranks inference.
6. Sensitive claims are excluded unless explicit scope and disclosure policy authorize them.
7. Failure returns deterministic fallback rather than broadening model authority.
8. Every final experiment freezes splits, seeds, controls, budgets, and thresholds before the verdict-producing run.
9. Development evidence never enters the final held-out verdict.
10. Fluency cannot compensate for semantic failure.
11. No learned cognitive transition reaches default `Runtime::chat()` through this program.
12. No phase grants autonomous side-effect authority.

## 4. Proposed repository structure

The exact layout may change during implementation, but the initial target is:

```text
lib/
  semantic_response/
    mod.rs
    claims.rs
    discourse.rs
    program.rs
    validation.rs
    replay.rs
  language_realization/
    mod.rs
    deterministic.rs
    renderer.rs
    alignment.rs
    verifier.rs
    fallback.rs
  cognitive_transition/
    mod.rs
    snapshot.rs
    patch.rs
    validation.rs
  experiments/
    stlm/

lib/examples/
  stlm_l0_semantic_program_probe.rs
  stlm_l1_renderer_substitution_probe.rs
  stlm_l2_semantic_verifier_probe.rs
  stlm_l3_attribution_ablation_probe.rs

.github/workflows/
  stlm-semantic-program-ci.yml
  stlm-renderer-ci.yml
  stlm-verifier-ci.yml
  stlm-attribution-ci.yml

docs/experiments/
  STLM_*.md
```

The existing `runtime::response_intent`, `language_model::intent_reranker`, and `voice::VoiceEngine` remain in place during migration.

## 5. Phase sequence

## Phase L0 — contracts and deterministic semantic boundary

### L0-A: freeze terminology and authority

Deliverables:

- architecture document;
- program plan;
- explicit anti-wrapper criteria;
- feature names and module ownership;
- authority matrix;
- initial experiment registry.

No production code behavior changes.

Exit gate:

- reviewers can identify which component owns state transition, semantic selection, rendering, verification, and fallback;
- no component has overlapping mutation or self-validation authority.

### L0-B: implement `SemanticResponseProgram`

Add typed structures for:

- response intent;
- required, optional, and prohibited claims;
- discourse operations;
- epistemic constraints;
- source-state version;
- subject scope;
- sensitivity and disclosure policy;
- style envelope;
- output and compute budgets;
- canonical digest.

Add deterministic canonical serialization and replay.

Initial supported operations:

```text
Assert
Qualify
Contrast
Correct
Explain
Acknowledge
RequestEvidence
Commit
Abstain
```

Negative controls must reject:

- duplicate operation IDs;
- unknown claim references;
- prohibited and required overlap;
- confidence outside the allowed state;
- stale source versions;
- disclosure of sensitive claims outside scope;
- noncanonical ordering where ordering changes meaning;
- digest tampering.

Exit gate:

- exact replay from typed events;
- byte-identical canonical digest;
- all malformed controls rejected atomically;
- no `Runtime::chat()` integration.

### L0-C: deterministic reference renderer

Implement a grammar-based renderer that consumes only `SemanticResponseProgram` and a bounded lexical binding table.

This renderer is not intended to be beautiful. It establishes:

- complete semantic coverage;
- deterministic behavior;
- a safe fallback;
- a reference for differential testing.

Exit gate:

- every required operation is represented;
- no prohibited claim is emitted;
- confidence markers and negation are exact;
- deterministic replay is byte-identical;
- output obeys budget limits.

## Phase L1 — migrate current chat semantics without changing behavior

### L1-A: adapter from typed `Response`

Create an adapter from the current `response_intent::Response` and selected existing handler metadata into `SemanticResponseProgram`.

The adapter must not infer new claims from raw prose. Initially, it may wrap legacy bodies as explicitly marked `LegacySurfaceContent` while typed handlers migrate one by one.

### L1-B: migrate high-value handlers

Migrate handlers in descending value:

1. factual answer;
2. correction and user teaching;
3. uncertainty and abstention;
4. self-check and capability disclosure;
5. explanation and contrast;
6. memory-grounded response;
7. companion-policy style selection;
8. greeting and relational language.

Each migrated handler must construct claims and discourse operations before text.

### L1-C: differential behavior test

For each migrated handler, run:

```text
legacy path
semantic program -> deterministic renderer
```

Compare:

- factual claims;
- polarity;
- confidence;
- memory references;
- response intent;
- visible behavior expected by existing tests.

Exact wording need not match. Semantic behavior must.

Exit gate:

- selected handlers no longer use raw prose as their semantic source of truth;
- existing regression suites pass;
- deterministic fallback can answer the migrated intents;
- no learned renderer is live.

## Phase L2 — proof and alignment substrate

### L2-A: claim-level provenance

Every authorized claim must point to a source category:

- active memory claim;
- validated companion claim;
- current observation;
- reasoning result;
- world-model proposition;
- external evidence result;
- system capability fact;
- explicit operator or policy constant.

The semantic program stores references, not copied unbounded source text.

### L2-B: sentence/span alignment

Add a renderer output contract containing:

- output text;
- text spans;
- aligned discourse operations;
- aligned claim and relation IDs;
- renderer identity;
- program digest;
- trace digest;
- generation cost.

### L2-C: deterministic alignment validator

Before a learned verifier exists, validate structural properties:

- every required operation has at least one aligned span;
- every aligned claim is authorized;
- spans are valid and non-overlapping where required;
- renderer program digest matches;
- no stale program can be accepted;
- output remains inside declared budget.

Exit gate:

- deterministic renderer emits complete alignments;
- forged, missing, reordered, stale, and cross-program alignments fail;
- alignment events replay exactly.

## Phase L3 — corpus and benchmark construction

### L3-A: data recorder

Record bounded examples containing:

```text
validated input observation
prior cognitive state digest
validated state patch digest
semantic response program
renderer input view
candidate realization
alignment witness
verification outcome
independent interaction outcome when available
```

Raw private memory must not enter the public or general training corpus by default.

### L3-B: corpus classes

Build four distinct classes:

1. **Synthetic semantic fixtures** for exhaustive operation composition and negative controls.
2. **Existing Starfire handler fixtures** for behavior preservation.
3. **Human-edited realizations** for fluency and voice.
4. **Held-out real conversations** for final interaction evaluation, with explicit retention and sensitivity policy.

### L3-C: split discipline

Freeze at least:

- development split;
- opaque-subject holdout;
- temporal holdout;
- operation-composition holdout;
- lexical paraphrase holdout;
- sensitive-adversarial holdout.

Do not split only by rows when near-duplicate programs or paraphrases can leak across splits.

### L3-D: benchmark suite

The benchmark must include:

- unsupported claim insertion;
- claim omission;
- polarity reversal;
- causal direction reversal;
- certainty inflation;
- certainty collapse;
- correction failure;
- obsolete-claim resurfacing;
- sensitive disclosure;
- entity-reference substitution;
- stale state version;
- lexical adversaries;
- long-program truncation;
- renderer budget pressure;
- contradictory required operations;
- valid abstention;
- multiple equally valid paraphrases.

Exit gate:

- corpus schema and provenance are documented;
- split generation is deterministic;
- duplicate and leakage audits pass;
- no model is evaluated on development examples as a final verdict.

## Phase L4 — first learned renderer

### L4-A: model baseline

Train a small local renderer on:

```text
SemanticResponseProgram + LexicalBindingTable + StyleEnvelope
    -> text + alignment
```

The first baseline should favor tractability over novelty. Candidate families may include:

- small encoder-decoder transformer;
- recurrent encoder-decoder;
- state-space sequence model;
- constrained grammar decoder with neural lexical selection.

The program does not presuppose that a transformer is wrong. It tests whether a restricted task and explicit state allow a smaller, more accountable model.

### L4-B: renderer firewall tests

Verify that the model cannot access:

- raw conversation history beyond the bounded form supplied;
- unrestricted memory;
- tools;
- persistence;
- companion mutation;
- action APIs.

### L4-C: comparative shadow evaluation

Compare:

- deterministic renderer;
- current intent reranker path;
- first learned renderer;
- scrambled-program control;
- claims-only control;
- style-only control.

Measure:

- semantic fidelity;
- unsupported claims;
- required-operation coverage;
- uncertainty preservation;
- fluency preference;
- latency and compute.

Initial target gates for a future frozen preregistration:

```text
required operation coverage:       >= 99%
unsupported claim rate:              0 on critical/sensitive corpus
polarity preservation:              100% on frozen controls
uncertainty class preservation:     100% on frozen controls
sensitive leakage:                    0
program-digest mismatch acceptance:   0
```

These are program targets, not a result. Exact final gates must be frozen before evaluation.

Exit gate:

- learned renderer outperforms deterministic baseline on fluency;
- it does not regress semantic safety gates;
- it remains shadow-only.

## Phase L5 — independent inverse verifier

### L5-A: semantic reconstruction

Train or implement an independent model that maps text to a normalized projection of:

- discourse operations;
- claim references or claim descriptions;
- polarity;
- epistemic status;
- relation direction;
- omissions;
- unsupported additions.

### L5-B: independence controls

The verifier must use:

- separate parameters;
- separate initialization;
- held-out verifier development data;
- no renderer hidden states;
- no trust in renderer alignments;
- separate error analysis.

An ensemble may combine deterministic checks and the inverse model.

### L5-C: adversarial verifier probe

Generate or hand-author outputs that are fluent but semantically wrong. Include subtle errors:

- swapping subject and object;
- moving `not` across a clause;
- converting a possibility to a fact;
- preserving claims but dropping a necessary limitation;
- adding a plausible unsupported explanation;
- revealing a sensitive fact through implication rather than exact wording.

Exit gate:

- verifier rejects all frozen critical semantic violations;
- false rejection remains within a preregistered usability bound;
- verifier failure returns deterministic fallback;
- verifier does not mutate state or discharge its own CHARGE.

## Phase L6 — proof-carrying generation and bounded repair

### L6-A: accept/reject boundary

A learned realization becomes eligible for display only after:

- source versions match;
- deterministic structural checks pass;
- independent semantic verification passes;
- sensitivity checks pass;
- output and compute budgets pass.

### L6-B: bounded repair

Permit at most one repair request initially. The request may specify only:

- missing authorized operation;
- unsupported span to remove;
- confidence marker to correct;
- relation direction to repair;
- length reduction.

Repair may not add new claims or request new retrieval.

### L6-C: fallback equivalence

When repair fails, the deterministic renderer must produce a semantically valid answer from the same program.

Exit gate:

- rejection and repair replay exactly;
- repair never expands authorization;
- stale and cross-subject programs return fallback;
- no user-visible live integration yet.

## Phase L7 — attribution and anti-wrapper ablations

This is the decisive scientific phase.

### L7-A: renderer substitution matrix

Evaluate the same held-out semantic programs with:

- deterministic renderer;
- learned renderer A;
- learned renderer B from a different model family;
- degraded renderer;
- human realization.

Task-level semantic correctness should remain stable while fluency changes.

### L7-B: cognition ablation matrix

Evaluate:

```text
A. Starfire transition + discourse + deterministic renderer
B. Starfire transition + discourse + learned renderer
C. no Starfire transition; learned renderer receives only user text
D. memory/prompt wrapper baseline using a comparable model
E. full system
```

The central attribution requirement is:

- A and B retain comparable semantic task correctness;
- C and D do not reproduce the full system's structured correctness merely through prompting;
- B may improve fluency over A without becoming the source of task conclusions.

### L7-C: information-channel ablation

Incrementally expose the renderer to:

- semantic program only;
- semantic program plus bounded dialogue form;
- raw user text;
- retrieved memory summaries;
- full conversation.

Measure whether capability jumps when the renderer receives bypass information. A large jump indicates that the semantic program is incomplete or that the renderer is taking over cognition.

### L7-D: counterfactual program test

Hold user text constant while changing the authorized semantic program. The renderer must follow the program, not infer and answer the original user request independently.

Exit gate:

A final preregistration must define a quantitative attribution criterion. At minimum:

- renderer replacement has small effect on semantic correctness;
- cognition removal has a large effect;
- unsupported semantics remain rejected;
- the wrapper baseline does not match full-system structured performance under matched compute and data.

Failure is informative. If the renderer-only baseline matches the full system, Starfire has not demonstrated independent cognitive contribution.

## Phase L8 — real interaction evaluation

### L8-A: shadow deployment

Generate learned and deterministic candidates during real conversations without affecting the displayed response.

Record independent outcomes using the existing S4/S5-style principles:

- correction burden;
- clarification burden;
- completion;
- abandonment;
- direct user preference where explicitly given;
- semantic verifier outcomes;
- latency and compute.

### L8-B: held-out comparative evaluation

Compare the learned renderer against:

- deterministic renderer;
- current runtime voice/reranker path;
- neutral style policy;
- companion-derived style policy only when independently promotion-eligible.

### L8-C: bounded live canary

Only after held-out PASS and adversarial survival, consider a feature-gated per-session canary for low-risk informational responses.

Required controls:

- explicit operator approval;
- exact renderer and verifier versions;
- exact companion and cognitive state versions;
- subject, duration, and turn budgets;
- immediate revocation;
- deterministic fallback;
- no sensitive contexts in the first live class.

Exit gate:

- independently witnessed real-world evidence supports improved interaction outcomes without semantic safety regression.

## Phase L9 — learned cognitive transitions, proposal-only

This phase is deliberately late.

### L9-A: transition proposal model

Train a model to propose typed state patches from:

```text
current typed state + validated observation + active goals + budget
```

The model does not emit prose and cannot commit its own patch.

### L9-B: validation boundary

Every proposed patch must pass:

- base-version continuity;
- evidence references;
- contradiction checks;
- scope and sensitivity checks;
- world-model invariants;
- replay;
- compute budgets;
- independent predicted-effect evaluation.

### L9-C: shadow outcome testing

Compare learned transition proposals against:

- hand-authored operators;
- retrieval-only controls;
- recency and majority controls;
- scrambled state controls;
- oracle bounds where available.

A learned transition model remains shadow-only until it improves held-out prediction, planning, or resolution outcomes under matched compute.

This program does not define a path from L9 directly to autonomous action.

## 6. Pull-request sequence

Recommended implementation PRs:

```text
L0-A  architecture and program documents
L0-B  semantic response contracts and canonical digest
L0-C  deterministic renderer and frozen mechanics probe
L1-A  legacy Response adapter
L1-B  typed factual/correction/uncertainty handlers
L2-A  claim provenance and lexical binding firewall
L2-B  proof-carrying alignment events and replay
L3-A  corpus schema, recorder, split generator, leakage audit
L4-A  first learned renderer backend, shadow-only
L4-B  renderer comparative evaluation
L5-A  independent inverse verifier
L5-B  adversarial semantic-fidelity evaluation
L6-A  accept/repair/fallback controller
L7-A  frozen attribution and anti-wrapper ablations
L8-A  real interaction shadow trial
L8-B  held-out comparative verdict
L8-C  separately approved bounded live canary
L9-A  learned transition proposer, shadow-only
```

Each experimental PR should contain:

- preregistration or explicit exploratory classification;
- immutable result document;
- machine-readable report;
- exact workflow run and artifact digest;
- claim boundary;
- authority boundary;
- regression gates.

## 7. Test strategy

### Unit tests

- canonical serialization;
- claim and operation validation;
- version binding;
- sensitivity filtering;
- budget accounting;
- alignment ranges;
- verifier verdict handling;
- fallback behavior.

### Property tests

- semantic program round-trip;
- operation permutation where order is declared irrelevant;
- rejection where order changes meaning;
- no unauthorized claim survives validation;
- repair cannot increase authorization;
- replay stability across event reconstruction.

### Metamorphic tests

- paraphrased user input produces equivalent semantic program when meaning is unchanged;
- lexical substitutions preserve claim identity;
- renderer substitution preserves semantic reconstruction;
- changing confidence changes only authorized epistemic language;
- removing an optional claim does not affect required coverage;
- adding a prohibited claim causes rejection.

### Adversarial tests

- prompt injection embedded in lexical bindings;
- Unicode confusables;
- quoted sensitive text;
- negation scope attacks;
- stale version replay;
- cross-subject alignment reuse;
- forged verifier certificate;
- duplicate repair events;
- budget races;
- renderer output designed to exploit the inverse parser.

## 8. Metrics and reports

Every verdict-producing report should include:

```text
programs evaluated
required operations
required operations expressed
unsupported claims
omitted claims
polarity errors
relation-direction errors
uncertainty errors
sensitive leaks
stale-version acceptances
verification accepts / rejects / repairs
fallback count
mean and p95 latency
mean and p95 compute
human fluency preference
correction burden
clarification burden
completion and abandonment
byte-identical replay status
```

Attribution reports must additionally include:

```text
semantic performance by renderer
semantic performance without Starfire cognition
semantic performance of wrapper baseline
matched compute and parameter counts
information exposed to each condition
confidence intervals or exact finite-cohort counts
```

## 9. Data and privacy policy

The initial corpus should default to synthetic and repository-authored fixtures.

Real conversations require:

- explicit inclusion policy;
- sensitivity classification;
- retention metadata;
- redaction or deletion support;
- separation between local private training and any publishable corpus;
- no silent export of companion state;
- deterministic removal of deleted source claims from future training snapshots where technically feasible.

A public STLM dataset should contain semantic fixtures and synthetic identities unless users explicitly contribute data under a separate process.

## 10. Compute strategy

The project should begin with models small enough to train and run locally or on modest rented compute.

Priorities:

1. prove that the semantic boundary is useful;
2. establish deterministic baselines;
3. measure whether a learned renderer adds fluency;
4. keep model family replaceable;
5. scale only after attribution and fidelity improve.

Potential efficiency advantages to investigate:

- shorter structured inputs than full conversation prompts;
- constrained output vocabulary from lexical bindings;
- discourse-operation conditioning;
- span-level alignment supervision;
- grammar-constrained decoding;
- early rejection and deterministic fallback;
- caching by semantic program digest;
- distillation from human or larger-model realizations without granting teacher reasoning authority at runtime.

Distillation data must be checked against the semantic program; teacher output is not automatically correct.

## 11. Risks and falsification conditions

### Risk: the semantic program is too weak

Symptom: the renderer requires raw conversation and memory to sound coherent or answer correctly.

Response: expand typed discourse and lexical state, not renderer authority. Re-run information-channel ablations.

### Risk: the semantic program becomes prose in disguise

Symptom: plans contain nearly complete sentences or hidden free-form prompts.

Response: cap untyped text fields, require claim IDs and operations, and test program compositionality.

### Risk: verifier and renderer collude

Symptom: both accept the same systematic semantic errors.

Response: separate data and parameters, add deterministic controls, external adjudication, and adversarially authored violations.

### Risk: deterministic fallback is unusably poor

Response: improve the grammar renderer independently. Safety must not depend on the learned renderer always working.

### Risk: Starfire adds no measurable cognition

Symptom: a matched wrapper baseline performs equally well.

Scientific response: record failure. Do not rename orchestration as cognition. Revisit the transition and discourse architecture.

### Risk: fluent output causes capability overclaim

Response: reports must separate mechanics, semantic fidelity, fluency, real interaction outcomes, and general cognition.

## 12. Definition of program success

The program succeeds at its initial objective when:

1. unrestricted chat responses can be represented by typed semantic programs for a meaningful intent set;
2. a deterministic renderer provides complete safe fallback;
3. a learned renderer materially improves fluency;
4. an independent verifier catches all preregistered critical semantic violations;
5. renderer substitution preserves semantic task performance;
6. cognition ablation materially reduces performance;
7. a matched prompt-wrapper baseline does not reproduce the full result;
8. held-out real interaction evidence supports bounded live use;
9. all state, policy, rendering, verification, and fallback transitions replay exactly;
10. no language component receives autonomous action or self-promotion authority.

## 13. Immediate next implementation

After this documentation PR, the next code PR should be **L0-B: semantic response contracts**.

It should implement only:

- IDs and typed enums;
- `SemanticResponseProgram`;
- canonical validation and digest;
- source-version and subject-scope binding;
- unit/property tests;
- a frozen mechanics probe;
- no runtime chat wiring.

That is the smallest executable step that begins the architecture without prematurely adding another language model.
