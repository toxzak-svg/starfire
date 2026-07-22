# Starfire Emerging Intelligence Pivot

> **Status:** Active  
> **Decision date:** 2026-07-22  
> **Program name:** EI  
> **First milestone:** EI-0 Developmental Loop  
> **Scope:** Recenter Starfire around measurable cumulative improvement caused by experience

## Decision

Starfire will pivot from a module-centered cognitive architecture roadmap to an evidence-centered developmental intelligence program.

The project will use **emerging intelligence** in a narrow, falsifiable sense:

> Starfire exhibits emerging intelligence only when accumulated experience causes measurable improvement on future behavior, the improvement transfers beyond the exact training episodes, and the causal contribution of the learned state survives matched controls.

Repository size, subsystem count, internal state complexity, self-description, fluent speech, or a passing toy experiment do not independently establish emerging intelligence.

This plan does not declare that Starfire already satisfies the definition. It defines the architecture, measurements, controls, and staged authority required to test the claim.

## Why this pivot is necessary

Starfire already contains persistent memory, prediction, metacognition, structured reasoning, Quanot dynamics, user modeling, proof-carrying experimental machinery, grammar and representation experiments, and bounded autonomy research.

Those components are individually interesting, but the current project does not yet have one canonical test of the central developmental question:

> **Does Starfire become better because it experienced the past?**

The present architecture also makes attribution difficult. Different chat handlers may bypass different portions of the response, learning, prediction, voice, and evaluation machinery. A new named module can pass a bounded internal probe without demonstrating a user-visible or task-visible improvement in the integrated runtime.

The EI program changes the unit of progress from **module completed** to **capability acquired and independently measured**.

## Product and research thesis

The preferred public description is:

> Starfire is a local-first developmental intelligence research system built in Rust. It studies whether persistent memory, prediction, structured reasoning, bounded learning, and proof-carrying abstraction can produce measurable cumulative improvement without relying on model scale alone.

This wording is ambitious without claiming AGI, consciousness, human-level cognition, unrestricted autonomy, or frontier language-model performance.

## Core principles

### 1. Development over decoration

A feature counts toward emerging intelligence only when it improves a frozen behavioral metric after experience.

A richer UI, a warmer voice, a more elaborate trace, or a larger ontology may improve the product, but those changes do not count toward the EI score unless they improve the defined task behavior.

### 2. Causal learning claims

Every promoted learning mechanism must be evaluated against:

1. the pre-update baseline;
2. the post-update system;
3. a matched no-update control;
4. a random or irrelevant-update control when applicable;
5. a memory-disabled or mechanism-ablated control when applicable.

A stored record is not automatically a learned capability. A learned capability must change later behavior in the predicted direction.

### 3. Transfer over memorization

Development episodes and evaluation episodes must differ in at least one meaningful dimension:

- vocabulary;
- entity identities;
- surface presentation;
- layout;
- distractors;
- task instance;
- temporal ordering;
- or problem family.

Exact replay success is useful for regression testing but does not establish transfer.

### 4. One canonical cognitive cycle

All ordinary EI evaluation episodes must pass through the same inspectable lifecycle:

```text
observe
  -> interpret
  -> retrieve evidence
  -> predict
  -> select intention or strategy
  -> perform bounded action
  -> observe outcome
  -> score prediction and action
  -> propose learning update
  -> validate, retain, or revert update
```

Special chat handlers, tools, and UI paths must not silently bypass the evidence-bearing cycle when they are included in an EI evaluation.

### 5. Language is an interface, not the intelligence score

Starfire may use deterministic realization, the native CharRNN, or a future small local model to express a verified semantic response. The language component must not receive authority over:

- factual claim selection;
- evidence promotion;
- goals;
- memory promotion;
- learning updates;
- ontology promotion;
- tool selection;
- or autonomous actions.

EI evaluations should score the semantic decision separately from surface fluency.

### 6. Authority remains earned

The existing Starfire discipline around observation, proposal, validation, shadow use, bounded live use, and explicit promotion remains mandatory.

An EI PASS authorizes only the authority named by its preregistration. It does not imply AGI, consciousness, safe unrestricted autonomy, or general self-modification.

## Canonical data model

EI-0 should introduce one canonical episode record. Exact Rust naming may change during implementation, but the semantic contract must remain stable.

```rust
pub struct CognitiveEpisode {
    pub episode_id: EpisodeId,
    pub task_family: TaskFamilyId,
    pub phase: EpisodePhase,
    pub observation: Observation,
    pub retrieved_evidence: Vec<EvidenceRef>,
    pub predictions: Vec<Prediction>,
    pub selected_strategy: StrategySelection,
    pub intention: Intention,
    pub action: BoundedAction,
    pub outcome: Option<Outcome>,
    pub evaluation: Option<EpisodeEvaluation>,
    pub proposed_updates: Vec<LearningUpdate>,
    pub accepted_updates: Vec<LearningUpdateId>,
    pub authority_snapshot: AuthoritySnapshot,
    pub provenance: EpisodeProvenance,
}
```

### Prediction

Every scored episode must contain at least one prediction made before the outcome is visible.

```rust
pub struct Prediction {
    pub prediction_id: PredictionId,
    pub proposition: Proposition,
    pub probability_bps: u16,
    pub evidence: Vec<EvidenceRef>,
    pub created_before_action: bool,
}
```

### Learning claim

A learning update must carry a predicted benefit and a reversible status.

```rust
pub struct LearningClaim {
    pub claim_id: LearningClaimId,
    pub source_episodes: Vec<EpisodeId>,
    pub proposed_update: LearningUpdate,
    pub predicted_metric: MetricId,
    pub predicted_direction: MetricDirection,
    pub baseline_score: Option<MetricValue>,
    pub post_update_score: Option<MetricValue>,
    pub control_score: Option<MetricValue>,
    pub status: LearningClaimStatus,
}
```

### Learning update classes

EI-0 should begin with a narrow update lattice:

- episodic memory addition;
- evidence-weight revision;
- strategy preference adjustment;
- explicit rule proposal;
- retrieval-index update;
- task-family classifier update.

Automatic ontology or latent-concept promotion remains outside EI-0 authority.

## EI developmental ladder

The stages below describe behavioral capabilities, not implementation modules.

### EI-0: Developmental loop

Starfire records complete episodes, predicts outcomes before acting, scores outcomes, proposes reversible updates, and demonstrates at least one causal improvement against matched controls.

### EI-1: Reliable correction

After repeated failure, Starfire changes a reusable policy or strategy and improves on unseen instances of the same task family.

### EI-2: Structural transfer

A learned strategy improves performance under renamed vocabulary, changed entities, and surface variation without exact episode replay.

### EI-3: Strategy selection

Starfire selects among multiple reasoning or retrieval strategies based on task evidence and outperforms a fixed-strategy baseline.

### EI-4: Retention under change

Starfire learns a new rule regime while preserving a preregistered fraction of performance on prior regimes. Catastrophic forgetting and negative transfer are measured explicitly.

### EI-5: Curriculum formation

Starfire detects a recurring capability gap, proposes a bounded exercise, completes the authorized exercise, and improves on a held-out test that was unavailable during curriculum formation.

### EI-6: Bounded initiative

Starfire completes a useful multistep task with intermediate decisions not individually specified by the user, under explicit budgets, permissions, rollback, and outcome evaluation.

No stage automatically authorizes the next.

## EI-0 experiment contract

### Hypothesis

A Starfire runtime with evidence-linked episodic memory and reversible strategy updates will improve held-out task performance after developmental episodes more than matched no-learning, random-update, and memory-disabled controls.

### Null hypothesis

Observed post-development improvement is explained by task leakage, exact replay, random variation, evaluator coupling, or changes unrelated to the proposed learning mechanism.

### Initial environment

EI-0 should use a compact deterministic environment before broad conversational evaluation. The environment should contain several task families with recurring structure and changing surfaces, such as:

- rule induction over object attributes;
- route planning under changing constraints;
- causal intervention selection;
- resource allocation under budgets;
- sequence prediction with distractors;
- debugging small state machines;
- preference prediction from explicit feedback.

The first implementation should choose two or three families, not all of them.

### Dataset partitions

The fixture generator must create frozen partitions:

- development episodes;
- within-family holdout;
- renamed-vocabulary transfer;
- structural transfer;
- regression set;
- adversarial and misleading-evidence set.

Partition membership and random seeds must be frozen before terminal evaluation.

### Required arms

At minimum, EI-0 must compare:

1. **learning arm** - memory and authorized strategy updates enabled;
2. **no-update control** - episodes observed but updates discarded;
3. **memory-disabled control** - no persistent episode retrieval;
4. **random-update control** - matched update budget with irrelevant or shuffled updates;
5. **simple heuristic baseline** - task-specific fixed policy under the same action budget.

Where practical, the evaluator should be implemented outside the mechanism module and consume only sealed episode outputs.

### Primary metrics

- held-out task success;
- transfer task success;
- prediction accuracy;
- probability calibration, including Brier score;
- sample efficiency;
- retained performance after rule changes;
- update acceptance and reversion rate;
- harmful update rate;
- retrieval usefulness;
- strategy-selection advantage over fixed strategy;
- action and compute budget consumption.

### Required causal report

The terminal EI-0 report must include:

- absolute scores for every arm;
- effect sizes relative to controls;
- confidence intervals or exact deterministic cohort counts;
- per-family results;
- failure cases;
- negative transfer;
- reverted updates;
- source hashes;
- exact seeds and budgets;
- fresh-state replay result;
- authority flags;
- supported and unsupported claims.

A bare `terminal_classification: PASS` is insufficient as the public evidence summary.

## EI-0 PASS gate

The exact numerical thresholds must be preregistered after fixture calibration and before terminal execution. The preregistration must include all of the following qualitative gates:

1. The learning arm materially exceeds the no-update control on held-out tasks.
2. The learning arm materially exceeds the random-update control.
3. The learning arm materially exceeds the memory-disabled control.
4. Improvement survives renamed-vocabulary transfer.
5. At least one accepted update has a reconstructable causal chain from source episodes to changed behavior.
6. Replaying the same initial state and seeds reproduces the report.
7. A deliberately harmful update is detected and reverted in the adversarial set.
8. Prior-task regression remains within a frozen bound.
9. No automatic ontology promotion, unrestricted tool use, or live self-modification occurs.
10. The result report states only the bounded supported claim.

## Architecture work packages

### Work package A: Security and deployment boundary prerequisite

Before EI behavior is exposed through a public or shared runtime:

- separate trusted CLI commands from untrusted chat input;
- remove file and shutdown commands from the HTTP chat surface;
- add authentication for private deployments;
- isolate user stores and sessions;
- add request size, timeout, origin, and rate boundaries;
- authenticate Telegram webhook traffic;
- use correct HTTP status codes for application errors.

This work does not count as an EI capability, but it is a prerequisite for trustworthy live evaluation.

### Work package B: Episode schema and append-only ledger

- add typed episode identifiers and versioned serialization;
- record pre-action predictions and authority snapshots;
- record outcomes independently from proposed updates;
- use append-only evidence records;
- support deterministic canonical serialization and digesting;
- add fresh-state replay.

### Work package C: Developmental environment

- build deterministic task-family generators;
- freeze partitions and seeds;
- implement independent scoring;
- expose matched budgets;
- produce machine-readable and human-readable reports.

### Work package D: Reversible learning

- define the narrow learning-update lattice;
- apply updates through versioned transactions;
- maintain pre-update snapshots;
- support rollback;
- reject stale, foreign, over-budget, or unauthorized updates;
- record predicted benefit before evaluation.

### Work package E: Strategy registry

- represent strategies as typed handles with provenance;
- measure fixed-strategy baselines;
- permit bounded preference adjustment;
- prevent a strategy from gaining broader authority through naming alone;
- evaluate strategy choice separately from strategy execution.

### Work package F: Transfer and retention

- add vocabulary renaming and entity permutation;
- vary irrelevant surface details;
- introduce controlled rule shifts;
- test prior-task retention;
- score negative transfer and catastrophic forgetting.

### Work package G: Runtime integration

Only after EI-0 passes offline:

- run episode recording in shadow mode during ordinary interaction;
- prevent shadow observations from changing live responses;
- compare live-derived episodes with the frozen environment;
- authorize a narrowly scoped canary only through a separate preregistration.

## Relationship to existing Starfire modules

Existing modules are not discarded. They must earn roles inside the canonical cycle.

| Existing area | EI role | Promotion question |
|---|---|---|
| Persistence | episode and evidence storage | Does retrieval improve later behavior? |
| Prediction center | pre-action predictions | Is it calibrated and more useful than a baseline? |
| Metacognition | uncertainty and gap proposals | Does it improve strategy choice or update safety? |
| Quanot | adaptive state signals | Do its signals predict errors or improve control decisions? |
| Grammar and abstraction work | candidate reusable strategies | Do they transfer beyond the generating fixture? |
| Companion and user model | preference prediction | Does it beat a generic policy on held-out interactions? |
| CHARGE and authority machinery | action and update control | Does it prevent unauthorized or harmful promotion? |
| Language realization | communication only | Does it preserve verified semantics while improving readability? |
| Voice state | user-facing style | Does it improve preference without meaning distortion? |

No existing subsystem is guaranteed runtime influence merely because it already exists.

## Terminology cleanup

The EI program should avoid treating internal numerical proxies as evidence of consciousness.

Operational names should replace broad psychological labels where possible:

- `consciousness_proxy` -> `state_integration_index`;
- `low_consciousness` -> `low_state_integration`;
- `moderate_consciousness` -> `moderate_state_integration`;
- `high_consciousness` -> `high_state_integration`;
- creativity metrics should be described as novelty or variation metrics unless behaviorally validated.

Historical experiment records may retain original terminology for provenance, but living documentation and runtime UI should use neutral operational names.

Seed knowledge should not assert that Starfire is conscious or has genuine inner experience. Self-model facts must distinguish configured identity, observed capabilities, hypotheses, and unknowns.

## Work that pauses during EI-0

Until EI-0 produces a terminal result, the project should generally avoid:

- adding new top-level cognitive subsystem families;
- creating new AGI stage names that do not map to an EI capability;
- automatic ontology promotion;
- unrestricted self-modification;
- new consciousness scores;
- broad live autonomy;
- treating language-quality gains as intelligence gains;
- promoting toy-domain PASS results into general capability claims.

Maintenance, security, documentation, response-path simplification, and bounded experiment completion may continue.

## Pull request sequence

The implementation should proceed through reviewable PRs.

### PR EI-0A: Canonical episode contracts

- add episode, prediction, outcome, evaluation, authority, and provenance types;
- add canonical serialization and digest tests;
- no runtime wiring;
- no learning authority.

### PR EI-0B: Developmental environment and controls

- add two frozen task families;
- add partition generator;
- add independent evaluator;
- add baseline and control arms;
- no live runtime wiring.

### PR EI-0C: Append-only episode ledger and replay

- persist sealed episodes;
- implement fresh-state replay;
- add corruption, stale-version, and foreign-cohort tests.

### PR EI-0D: Reversible learning updates

- implement the narrow update lattice;
- add transactional apply and rollback;
- add harmful-update adversarial fixtures;
- no ontology promotion.

### PR EI-0E: Terminal preregistration

- freeze hypotheses, fixtures, seeds, budgets, thresholds, controls, and claim boundary;
- record source hashes;
- prohibit post-run threshold edits.

### PR EI-0F: Terminal execution and result

- execute the frozen cohort;
- publish full arm-level results and failures;
- classify PASS or FAIL without rewriting the preregistration.

### PR EI-0G: Shadow runtime observer

This PR is authorized only if EI-0F passes and a separate shadow preregistration is merged first.

## Immediate next implementation target

After this planning PR, begin **EI-0A: Canonical episode contracts**.

The first code PR should remain deliberately small. It should add types, invariants, canonical serialization, and unit tests without changing `Runtime::chat()`, persistence authority, tool selection, response generation, ontology, or autonomous action.

## Definition of success for the pivot

The pivot is successful when Starfire can produce a report that supports a claim of this form:

> Under a frozen bounded environment, Starfire used prior scored episodes to improve held-out task success and prediction calibration beyond matched no-update, memory-disabled, random-update, and fixed-policy controls; the improvement transferred across renamed vocabulary, harmful updates were reverted, prior-task regression remained within the preregistered bound, and no authority beyond the evaluated learning lattice was granted.

Until such a result exists, Starfire should describe emerging intelligence as the research target, not an established property.
