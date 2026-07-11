# Infant–Starfire Fusion Research Plan

## Status

Experimental integration program. This work is isolated from `main` on the branch:

`experiment/infant-starfire-fusion`

The goal is **not** to copy the entire InfantGym repository into Starfire or immediately wire a learned model into the live runtime. The goal is to test whether Infant’s developmental learning substrate adds capabilities that Starfire’s current symbolic/causal closed-cycle architecture does not already provide, while preserving Starfire’s evidence discipline, replayability, and safety gates.

---

## Research Thesis

Starfire currently has a strong symbolic and causal research trajectory: persistent state, typed evidence, CHARGE-style pressures, PECS-style commitments, ontology/role induction, replay, and increasingly closed cognitive cycles.

Infant contributes a different substrate:

- learned multimodal encoders,
- a predictive world model,
- developmental curricula,
- concept formation under changing losses/modalities,
- dual-head stability ideas,
- developmental memory and identity experiments,
- embodied/action-conditioned learning experiments.

The central hypothesis is:

> A developmental learner can serve as Starfire’s learned perceptual and predictive substrate, while Starfire remains the deliberative, causal, evidence-governed organism.

This is a **hybrid cognition hypothesis**, not a claim that either system alone is AGI.

---

# Non-Negotiable Design Rules

1. **No blind end-to-end fusion.** Infant must not become an opaque authority over Starfire’s decisions.
2. **No live-runtime promotion without gates.** Learned concepts, latent roles, predictions, and policies remain advisory until independently validated.
3. **Preserve provenance.** Every learned representation used by Starfire must carry source, model version, confidence, and transformation history.
4. **Preserve falsifiability.** Every claimed gain must survive matched-budget baselines and ablations.
5. **Preserve replayability.** Given frozen inputs, checkpoints, and seeds, the integration experiment must be reproducible.
6. **Separate perception from belief.** Infant may propose observations or latent structure; Starfire decides whether evidence supports a persistent belief.
7. **No catastrophic identity rewriting.** Any learned identity or preference pathway must remain behind explicit write-protection and revision rules.
8. **No autonomous self-modification in Phase 1.** The integration may adapt model state under controlled training, but not rewrite Starfire code or promotion gates.

---

# Target Architecture

```text
Environment / Task / Conversation / Sensor Stream
                    |
                    v
        +-------------------------+
        | Infant Developmental    |
        | Substrate               |
        |-------------------------|
        | encoders                |
        | predictive world model  |
        | concept heads           |
        | action-conditioned state|
        +------------+------------+
                     |
            typed learned evidence
                     |
                     v
        +-------------------------+
        | Fusion Boundary         |
        |-------------------------|
        | provenance              |
        | confidence calibration  |
        | feature/latent adapters |
        | contradiction checks    |
        | replay serialization    |
        +------------+------------+
                     |
                     v
        +-------------------------+
        | Starfire Closed Cycle   |
        |-------------------------|
        | observation             |
        | reconciliation          |
        | CHARGE / pressure        |
        | PECS commitments         |
        | causal / role reasoning  |
        | operator attempts        |
        | independent verification|
        | memory / replay          |
        +------------+------------+
                     |
                     v
             Action / Response
```

The fusion boundary is the critical component. Infant outputs are **evidence-bearing proposals**, not truth.

---

# Phase 0 — Freeze Baselines and Inventory

## Goal

Establish what Starfire and Infant can each do independently before any integration.

## Work

### Starfire baseline

Record current performance on:

- closed-cycle reasoning tasks,
- CHARGE routing diagnostics,
- causal and reasoning-favored resolver tasks,
- ontology/role induction diagnostics,
- memory continuity tasks,
- held-out transfer tasks,
- replay determinism.

### Infant baseline

Record current performance on:

- prediction error,
- object/concept consistency,
- cross-modal grounding,
- action-effect prediction,
- continual-learning retention,
- dual-head stability,
- held-out environment transfer.

### Required artifact

Create a machine-readable baseline manifest containing:

- repository commit SHA,
- model checkpoint hashes,
- dataset/task versions,
- random seeds,
- hardware/runtime information,
- metrics,
- known failures.

## Exit gate

No integration work proceeds until both independent baselines are reproducible.

---

# Phase 1 — Define the Typed Fusion Contract

## Goal

Create a narrow interface by which Infant can provide learned observations to Starfire without contaminating Starfire’s core state.

## Proposed types

```rust
pub struct LearnedEvidence {
    pub source_model: String,
    pub source_version: String,
    pub observation_id: String,
    pub modality: LearnedModality,
    pub payload: LearnedPayload,
    pub confidence: f64,
    pub uncertainty: f64,
    pub provenance: Provenance,
    pub timestamp: i64,
}

pub enum LearnedPayload {
    ObjectSet(Vec<LearnedObject>),
    StateEmbedding(Vec<f32>),
    PredictedTransition(PredictedTransition),
    ConceptProposal(ConceptProposal),
    AnomalyScore(f64),
}
```

The initial interface should prefer **interpretable outputs** over raw embeddings.

## Rules

- Raw embeddings may be stored for diagnostics but cannot directly become persistent Starfire beliefs.
- Concept proposals must remain provisional.
- Predictions must be compared against observed outcomes.
- Calibration statistics must be tracked by model/checkpoint/task domain.

## Exit gate

Starfire can ingest and replay typed Infant evidence with zero behavioral change when the feature flag is disabled.

---

# Phase 2 — Offline Perception Adapter

## Goal

Use Infant as a perception/world-state proposal engine while keeping Starfire’s reasoning unchanged.

## First experiments

1. **Object extraction comparison**
   - Infant learned perception
   - deterministic connected-component baseline
   - Starfire-native parser/baseline

2. **State-change detection**
   - compare learned transition representations against simple pixel/state differences

3. **Anomaly proposal**
   - ask Infant to flag surprising transitions
   - measure whether those flags help Starfire create better CHARGE observations

## Metrics

- precision/recall of useful observations,
- downstream task success,
- false pressure generation,
- compute cost,
- calibration,
- held-out transfer.

## Exit gate

Infant evidence must improve at least one downstream task under matched compute without materially increasing false positives or destabilizing replay.

---

# Phase 3 — Predictive World Model as a Counterfactual Oracle

## Goal

Use Infant’s world model to generate **candidate predictions**, not decisions.

For a proposed action/operator:

```text
Starfire proposes action
        |
        v
Infant predicts likely next state(s)
        |
        v
Starfire evaluates predicted consequences
        |
        v
Action executes or remains hypothetical
        |
        v
Observed outcome compared with prediction
        |
        v
Prediction error becomes evidence
```

## Key research question

Does learned prediction help Starfire choose better operators or detect model mismatch earlier than symbolic reasoning alone?

## Required controls

- random predictor,
- frequency baseline,
- simple transition table,
- Infant world model,
- Starfire without prediction assistance.

## Exit gate

Prediction assistance must improve held-out action selection or contradiction detection, not merely training-domain fit.

---

# Phase 4 — Prediction Error to CHARGE

## Goal

Convert persistent, calibrated prediction error into a typed pressure signal.

This is the first deep architectural fusion.

## Proposed rule

Prediction error alone is insufficient. A CHARGE-like pressure should require some combination of:

- repeated mismatch,
- task relevance,
- confidence in the original prediction,
- independent observation support,
- persistence across episodes or contexts.

## Candidate pipeline

```text
predicted transition
    + observed transition
    -> residual / mismatch
    -> calibration
    -> persistence check
    -> relevance weighting
    -> typed discrepancy evidence
    -> possible CHARGE creation
```

## Failure modes to prevent

- noisy model creates endless false CHARGE,
- model uncertainty mistaken for contradiction,
- distribution shift treated as ontology failure,
- self-confirming feedback loops.

## Exit gate

The integrated system identifies genuine unresolved discrepancies earlier than Starfire alone while maintaining an acceptable false-pressure rate.

---

# Phase 5 — Developmental Concept Proposals

## Goal

Allow Infant to propose candidate concepts that Starfire can test.

This phase explicitly preserves the existing caution around automatic latent-concept promotion.

## Concept lifecycle

```text
latent regularity
    -> concept proposal
    -> canonical descriptor
    -> supporting examples
    -> counterexamples
    -> stability score
    -> intervention / transfer tests
    -> provisional symbolic handle
    -> promotion gate
```

## Promotion requirements

A proposed concept must survive:

- multiple seeds,
- held-out data,
- matched-width/random feature controls,
- task transfer,
- counterexample search,
- ablation showing downstream utility,
- independent verifier approval.

## Exit gate

At least one learned concept becomes a useful symbolic handle under frozen promotion rules and survives held-out transfer.

---

# Phase 6 — Shared Memory Without Shared Authority

## Goal

Unify useful memory while preserving distinct semantics.

## Proposed memory classes

- **Raw episodic observation** — environment/event record
- **Learned representation** — Infant-derived feature/concept state
- **Symbolic proposition** — Starfire interpretation
- **Verified belief** — independently supported proposition
- **Identity memory** — protected, explicitly governed

Infant-derived state must never silently overwrite verified beliefs or protected identity.

## Research questions

- Can learned embeddings improve retrieval?
- Can developmental consolidation improve long-horizon memory?
- Can Starfire’s evidence graph prevent learned-memory contamination?

## Exit gate

Long-horizon recall or transfer improves without increased contradiction or identity corruption.

---

# Phase 7 — Developmental Curriculum for the Closed Cognitive Cycle

## Goal

Use InfantGym’s strongest idea—the curriculum as the experimental variable—to train or expose the hybrid system in stages.

## Candidate curricula

### Curriculum A: world first

1. perception and prediction,
2. action-effect learning,
3. symbolic grounding,
4. causal reasoning,
5. language/interaction.

### Curriculum B: simultaneous

All modalities and reasoning channels available from the beginning.

### Curriculum C: pressure-gated development

New capabilities activate only after the system demonstrates competence or unresolved pressure in prerequisite stages.

### Curriculum D: counterfactual-first

The system learns prediction and intervention before concept naming.

## Experimental rule

Keep architecture fixed while varying curriculum whenever possible.

## Exit gate

A curriculum produces repeatable gains over simultaneous training/exposure on held-out developmental tasks.

---

# Phase 8 — Closed Hybrid Cycle

## Goal

Run the complete experimental loop:

```text
observe
-> encode
-> predict
-> reconcile
-> create typed pressure
-> deliberate
-> propose operator
-> simulate candidate consequence
-> act
-> observe outcome
-> independently verify
-> update learned + symbolic state
-> replay
```

## Critical invariant

Learning and symbolic belief revision are related but not identical operations.

A neural weight update does not automatically constitute a belief update. A symbolic belief update does not automatically retrain the neural substrate.

## Exit gate

The hybrid loop outperforms both isolated systems on a predefined held-out suite while remaining reproducible and debuggable.

---

# Phase 9 — Safety and Robustness Stress Tests

Required tests:

- corrupted observation stream,
- adversarial concept proposal,
- overconfident wrong world-model prediction,
- checkpoint swap,
- stale embedding/model version,
- catastrophic forgetting pressure,
- identity overwrite attempt,
- contradictory multimodal evidence,
- reward shortcut / action-effect exploitation,
- distribution shift.

The system must fail visibly rather than silently promote corrupted structure.

---

# Phase 10 — Decision Point

After the experimental program, choose one of four outcomes:

1. **Reject integration** — no robust gain.
2. **Keep Infant as an external sensor/world-model service** — useful but not organism-level integration.
3. **Adopt selected mechanisms only** — specific developmental components graduate into Starfire.
4. **Promote the hybrid architecture** — only if the full closed cycle shows robust held-out gains and remains interpretable enough to debug.

No result is considered failure if it cleanly falsifies a hypothesis.

---

# Initial Repository Layout

Proposed Starfire-side layout:

```text
crates/ or lib/
  developmental/
    mod.rs
    evidence.rs
    adapter.rs
    calibration.rs
    replay.rs
    prediction.rs
    concepts.rs

experiments/
  infant_fusion/
    phase0_baseline/
    phase1_contract/
    phase2_perception/
    phase3_prediction/
    phase4_charge/
    phase5_concepts/

plans/
  INFANT_STARFIRE_FUSION_PLAN.md
```

The exact path should follow the current repository’s actual module/workspace conventions rather than forcing this structure mechanically.

---

# First Implementation Milestone: H-Infant-0

## Objective

Build the integration boundary with **zero live behavioral authority**.

### Deliverables

1. `LearnedEvidence` typed contract.
2. Versioned JSON serialization for replay.
3. Feature flag for developmental evidence ingestion.
4. Offline adapter capable of reading exported Infant observations.
5. Baseline manifest format.
6. Determinism/replay tests.
7. No changes to live routing, concept promotion, or autonomous action selection.

### Success criteria

- existing Starfire tests remain green,
- integration feature disabled by default,
- replay of the same evidence produces the same Starfire-side observation sequence,
- malformed or stale evidence is rejected explicitly,
- no Infant output can directly create a persistent verified belief.

---

# Second Milestone: H-Infant-1

## Objective

Evaluate whether Infant-derived perception adds downstream value.

### Experiments

- learned object proposals vs deterministic baseline,
- learned anomaly proposals vs transition-difference baseline,
- downstream CHARGE usefulness under frozen gates,
- held-out ARC-like or synthetic environment transfer.

### Promotion criterion

Proceed only if the learned path demonstrates a statistically and operationally meaningful gain under matched compute and ablation controls.

---

# Third Milestone: H-Infant-2

## Objective

Test Infant’s predictive world model as a counterfactual assistant.

No direct policy control.

### Promotion criterion

Proceed only if predictions improve operator ranking, contradiction detection, or sample efficiency on held-out tasks.

---

# Immediate Next Actions

1. Freeze and document current Starfire and Infant commit SHAs.
2. Add the `H-Infant-0` typed evidence contract in Starfire.
3. Add a minimal export format in Infant for observations, predictions, concept proposals, and model metadata.
4. Build an offline replay fixture with synthetic learned evidence.
5. Run Starfire tests with the feature disabled.
6. Add malformed/stale/version-mismatch tests.
7. Only then begin perception experiments.

---

# Working Principle

The intended fusion is not:

> “Put a neural model inside Starfire and hope intelligence emerges.”

It is:

> “Give Starfire a developmental, learned model of the world, but force every learned contribution to cross an explicit evidence boundary where it can be measured, contradicted, replayed, and rejected.”

That preserves the strongest idea from Infant—development through learned experience—while preserving the strongest idea from modern Starfire—proof-oriented, typed, falsifiable cognitive structure.
