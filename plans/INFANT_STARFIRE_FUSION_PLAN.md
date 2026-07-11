# Infant–Starfire Fusion Research Plan

**Status:** experimental integration program  
**Date:** 2026-07-11  
**Integration host:** Starfire  
**Developmental substrate source:** `toxzak-svg/infant`  
**Isolation branch:** `experiment/infant-starfire-fusion`  
**Rule:** no `Runtime::chat()` integration until the closed-cycle gates below pass

## Thesis

Do not paste Infant into Starfire or let a neural model silently mutate the symbolic core.

The target is a two-level organism:

1. **Infant-derived developmental substrate**
   - learns perceptual and transition representations from experience
   - maintains plastic latent state
   - predicts near-future observations or effects
   - exposes uncertainty, residuals, candidate latent structure, and developmental state

2. **Starfire closed cognitive cycle**
   - owns explicit hypotheses, CHARGE, PECS commitments, causal/relational structure, operator routing, provenance, replay, and independent discharge judgment
   - decides what a neural signal means operationally
   - accepts no concept, rule, memory, or self-modification merely because the neural substrate emitted it confidently

The research question is:

> Can a developmental learned substrate provide representations and prediction residuals that make Starfire's closed cognitive cycle more capable, sample-efficient, and transferable without sacrificing explicit accounting, falsifiability, or independent outcome judgment?

The answer is unknown and must be tested.

## Why this is different from earlier Infant integration

Starfire already inherited selected Infant ideas historically, especially identity protection and memory tiering. It also once packaged an Infant model for ARC-style perception. The deeper integration was never completed.

Modern Starfire now has stronger host machinery:

- persistent typed CHARGE
- independent discharge judgment
- environment contracts
- transition hypotheses
- causal and relational machinery
- replay and held-out evaluation requirements
- ontology-pressure experiments
- operator routing and skill-compilation research
- PECS-style executable commitments
- rule induction, graph discovery, representation genesis, and transport experiments

The opportunity is therefore not "make Infant the whole intelligence." It is to test whether developmental neural learning can become an **evidence-producing substrate inside an accountable closed cognitive cycle**.

---

## Non-negotiable invariants

### 1. Infant never judges its own success

A prediction head, latent concept proposal, confidence value, reconstruction score, or self-classification is evidence input, not accepted truth.

Only an independent Starfire judge may accept discharge, promotion, or transfer credit.

### 2. No raw latent vector becomes ontology by declaration

A latent feature may become a candidate structural role only after measurable utility under replay and held-out transfer.

```text
latent pattern
  -> recurring residual association
  -> candidate role with provenance
  -> explicit probe or decoder
  -> replay improvement
  -> held-out transfer improvement
  -> complexity-adjusted acceptance
  -> provisional ontology role
```

### 3. The bridge preserves provenance

Every signal crossing from Infant into Starfire identifies:

- model/checkpoint identity
- observation and episode IDs
- representation version
- producer head or adapter
- transformation trace
- confidence/calibration metadata where applicable
- timestamp and schema version

### 4. Neural plasticity cannot silently rewrite protected Starfire state

The Infant substrate may learn continuously in experiments, but it cannot directly mutate:

- accepted symbolic identity commitments
- evidence records
- CHARGE accounting history
- PECS commitments
- promoted ontology
- operator success statistics
- regression baselines

Changes must travel through explicit candidate and promotion paths.

### 5. Frozen-core claims are hypotheses, not guarantees

The historical Infant dual-head design froze output heads while some shared encoders could remain plastic. The fusion program must separately test head-only freezing, encoder-plus-head freezing, adapter isolation, replay regularization, protected-memory retrieval, and controlled revision.

### 6. No direct self-edit-to-main loop

All neural architecture changes, ontology promotions, and induced operators remain isolated until matched-budget evaluation, held-out transfer, and regression gates pass.

### 7. Chat is not the first benchmark

The first integration runs in deterministic or replayable environments with objective outcome witnesses.

---

## Architectural target

```text
                   DEVELOPMENTAL SUBSTRATE

OBSERVATION -----> Infant Observation Encoder
                       |
                       v
                 Developmental State
                       |
              +--------+---------+
              |                  |
              v                  v
       Prediction Head     Representation / Slots
              |                  |
              +--------+---------+
                       |
                       v
              BRIDGE EVIDENCE PACKET
              - typed observations
              - predicted effects
              - residual signatures
              - uncertainty
              - candidate latent roles
              - provenance
                       |
                       v

                    STARFIRE CORE

          Reconcile explicit world model
                       |
                       v
               Emit typed CHARGE
                       |
                       v
              Route cognitive operators
                       |
                       v
       Hypothesis / query / action / revision
                       |
                       v
                 Execute or test
                       |
                       v
               Observe real outcome
                       |
                       v
          Independent Discharge Judge
                       |
        +--------------+----------------+
        |              |                |
        v              v                v
 update explicit   update routing   store episode /
 world model       statistics       counterexample
        |                               |
        +---------------+---------------+
                        |
                        v
             TRAIN / REPLAY CANDIDATE
```

## Repository strategy

Work begins on `experiment/infant-starfire-fusion`. `main` remains untouched.

The first bridge is process-separated and explicit. Do not add PyTorch as a hidden dependency of the Rust cognitive core in Phase 0. JSON replay is acceptable for the initial deterministic probe; optimize transport only if profiling proves the bridge is a bottleneck.

Suggested Starfire-side layout:

```text
lib/developmental/
  mod.rs
  evidence.rs
  adapter.rs
  replay.rs
  manifest.rs

experiments/infant_bridge/
  protocol.py
  infant_adapter.py
  replay_server.py

experiments/infant_fusion/
  manifests/
  fixtures/
  results/
  frozen_gates/
```

---

# Phase F0 — Source audit and reproducibility freeze

Inventory the current Infant repository and history:

- current InfantGym environment/agent/curriculum framework
- historical `cognitive_architecture.py`
- dual-head experiments
- memory hierarchy
- dream/consolidation components
- goal formation
- self-validation
- body/action-effect experiments
- saved checkpoints
- benchmark scripts and exact datasets

For every claimed result, record commit SHA, entry point, dataset or generator, random seeds, baseline, metric definition, raw artifacts if available, and whether it reproduces now.

**Critical audit item:** test whether the historical dual-head protected behavior remains invariant when shared encoders continue learning.

**Exit gate:** produce an `INFANT_REPRODUCIBILITY_LEDGER.md` classifying each claim as reproduced, reproduced with correction, not reproducible, insufficient artifact, or superseded.

---

# Phase F1 — Bridge contracts only

Create an integration boundary without changing Starfire behavior.

The first Rust contract carries typed learned evidence with schema version, model/checkpoint provenance, observation ID, confidence, uncertainty, payload, and timestamp.

Rules:

- raw embeddings may be retained for diagnostics but cannot directly become persistent beliefs
- concept proposals remain provisional
- predictions must be compared with observed outcomes
- calibration is tracked by model/checkpoint/task domain
- malformed, stale, future-skewed, or version-mismatched evidence is rejected
- no runtime routing, belief promotion, ontology promotion, or autonomous action authority

**Exit gate:** contracts compile, deterministic serialization round-trips, malformed provenance is rejected, the feature is disabled by default, and no current Starfire behavior changes.

---

# Phase F2 — Frozen Infant perception baseline

Test whether a frozen Infant-derived representation helps the existing Starfire cycle without confounding the result with online learning.

Conditions:

1. hand-engineered symbolic observation features
2. frozen random neural features
3. frozen Infant-derived features
4. oracle object/state features

Use matched action and compute budgets.

Metrics:

- solve rate
- action cost to solve
- held-out transition prediction error
- unresolved CHARGE at termination
- uncertainty calibration
- nuisance-transformation stability
- transfer to held-out surface names/renderings

**Exit gate:** Infant-derived features must beat random matched-width features and provide measurable downstream value beyond compute-matched controls.

---

# Phase F3 — Developmental prediction residuals into CHARGE

Infant predicts observation/effect features. Starfire compares prediction with observed outcome and converts independently measured residual structure into typed CHARGE.

Infant may emit prediction, uncertainty, and residual vectors. Starfire owns CHARGE type, scope, persistence, routing, and discharge judgment.

Conditions:

1. no developmental predictor
2. scalar residual only
3. structured residual signature
4. structured residual plus persistent CHARGE
5. oracle residual structure

**Exit gate:** structured developmental residuals plus persistent CHARGE outperform scalar-error controls on held-out transition families under matched budget.

---

# Phase F4 — Proof-carrying latent role induction

A candidate role may be proposed only when a residual family recurs, existing operators repeatedly fail, and a latent feature or feature combination is stably associated with that residual family.

Promotion protocol:

```text
candidate latent role
  -> freeze candidate definition
  -> fit simple probe/decoder if needed
  -> replay affected episodes
  -> re-induce explicit transition hypotheses
  -> complexity-adjusted comparison
  -> held-out exact-instance test
  -> held-out surface test
  -> held-out rule-family transfer
  -> provisional role only if all gates pass
```

Controls:

- random latent dimensions
- shuffled residual association
- scalar prediction error
- equal-count random concept proposals
- hand-authored oracle latent role
- no ontology expansion

Reject candidates that improve reconstruction but not objective prediction or transfer, memorize episode identity, or require privileged hidden labels unavailable at inference.

---

# Phase F5 — Dual-plasticity / protected-core experiment

Compare:

1. fully plastic network
2. freeze output head only
3. freeze encoder plus protected head
4. separate protected and plastic encoders
5. shared frozen trunk plus trainable adapters
6. replay regularization
7. external protected symbolic core with plastic neural substrate

Measure protected-task retention, new-task acquisition, interference matrix, calibration drift, representation drift, recovery after distribution shift, transfer, and compute cost.

Identity and values should not be arbitrary immutable scalar outputs. For Starfire, protected commitments should be testable things such as identity bindings, provenance rules, accounting constraints, and autobiographical continuity records. Revision must be explicit and evidenced rather than impossible by construction.

**Exit gate:** a protected/plastic architecture shows a superior retention–adaptation Pareto frontier over fully plastic and naive freeze baselines.

---

# Phase F6 — Developmental memory and consolidation exchange

Infant may retain compact learned episode embeddings, transition-prediction examples, replay buffers, and developmental state.

Starfire retains explicit episodic records, evidence/provenance, accepted semantic claims, identity and relationship records, CHARGE history, operator statistics, PECS commitments, and promoted ontology.

Infant can emit consolidation candidates such as recurring episode clusters, predictive prototypes, surprising transition families, or representation-drift warnings. Starfire independently decides whether to create semantic memory, emit CHARGE, schedule replay, or reject the candidate.

**Exit gate:** long-horizon task retention or transfer improves without increasing false semantic promotion beyond a predeclared threshold.

---

# Phase F7 — Active developmental closed cycle

```text
observe
 -> Infant encodes and predicts
 -> Starfire emits/resolves CHARGE
 -> Starfire selects information-gaining or goal-directed action
 -> environment produces outcome
 -> independent judge scores objective evidence
 -> Starfire updates explicit model
 -> Infant receives bounded training event
 -> replay / calibration / drift checks
 -> continue
```

Required ablations:

- Infant learning off
- Starfire active experiment selection off
- CHARGE persistence off
- developmental uncertainty removed
- replay removed
- ontology promotion disabled
- random exploration at equal action budget

Primary question: does the coupled system learn hidden rules or transferable abstractions faster than either subsystem alone?

---

# Phase F8 — Cross-environment transfer

Use one bridge contract across at least three environment families:

1. rendered HiddenRule worlds
2. ARC-like object/action tasks
3. bounded tool or filesystem environment with objective witnesses

**Exit gate:** at least one learned representation, operator-routing benefit, or promoted latent role transfers across environment families without retraining the entire Starfire cognitive core from scratch.

---

# Phase F9 — Runtime integration gate

`Runtime::chat()` and general live Starfire operation remain untouched until all of these hold:

- no CHARGE accounting regressions
- provenance completeness
- deterministic replay where promised
- held-out improvement over matched controls
- no catastrophic protected-capability regression
- no uncontrolled ontology growth
- no direct neural self-approval path
- bridge failure degrades safely
- checkpoint identity is always observable
- resource use is bounded
- integration can be disabled with one feature flag

Only then may a minimal runtime experiment be proposed.

---

## First implementation milestone: H-Infant-0

Build the integration boundary with **zero live behavioral authority**.

Deliverables:

1. typed learned-evidence contract
2. versioned JSON serialization for replay
3. disabled-by-default feature flag
4. offline/no-op evidence adapters
5. reproducible baseline manifest format
6. determinism and replay tests
7. malformed/stale/version-mismatch rejection tests
8. no changes to live routing, concept promotion, belief promotion, or autonomous action selection

Success criteria:

- existing Starfire tests remain green
- integration feature disabled by default
- replay of identical evidence produces the same Starfire-side sequence
- malformed or stale evidence is rejected explicitly
- no Infant output can directly create a persistent verified belief

## Next milestones

### H-Infant-1

Evaluate whether frozen Infant-derived perception adds downstream value against deterministic and random-feature controls.

### H-Infant-2

Evaluate Infant's predictive world model as a counterfactual assistant with no direct policy control.

### H-Infant-3

Feed independently measured developmental prediction residuals into CHARGE and test structured residuals against scalar-error controls.

### H-Infant-4

Test proof-carrying latent roles under frozen promotion gates and held-out transfer.

---

## Stop conditions

Pause or abandon the fusion if:

- Infant features do not beat random matched-width features
- online learning improves training tasks but consistently harms held-out transfer
- latent roles cannot be decoded into stable reusable structure
- ontology growth is dominated by spurious concepts
- protected/plastic schemes do not beat simple replay baselines
- bridge complexity overwhelms capability gains
- gains depend on benchmark leakage
- Starfire's explicit accounting becomes less trustworthy

A negative result is valid research output.

## Success definition

The fusion succeeds only if all three layers are demonstrated:

1. **Perceptual/developmental utility:** Infant-derived learning provides better predictive or task-relevant evidence than matched controls.
2. **Accountable cognitive use:** Starfire converts that evidence into better hypotheses, actions, or transferable structure under independent outcome judgment.
3. **Closed-cycle co-development:** the coupled system improves through experience while retaining provenance, bounded plasticity, replayability, and held-out generalization.

The desired endpoint is not "a neural model attached to Starfire."

It is:

> A developmental substrate that learns from experience, coupled to a cognitive organism that can represent why it is uncertain, act to resolve that uncertainty, test what changed, preserve evidence, and promote only the structures that survive falsification.
