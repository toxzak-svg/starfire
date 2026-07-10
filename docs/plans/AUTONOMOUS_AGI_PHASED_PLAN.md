# Autonomous AGI-oriented phased plan

## Status

Plan begun. This is not an AGI claim.

Current evidence says Starfire may proceed only to the next diagnostic gate, not
to a stronger autonomy or AGI claim. The latest H5-C primary shadow ontology
diagnostic now passes the local primary acceptance criteria after the H5-B
task-profiled prerequisite passed.

Latest local diagnostic command:

```text
cargo run -p star --example h5_non_memory_ontology_probe --locked
```

Latest local verdict:

- final verdict: `PASS`
- reproducibility repeats: `3`
- passing repeats: `3`
- H5-C gates: `17/17`
- promoted non-memory concepts: `1`
- proposal evaluations: `203`
- future route evaluations: `96`
- induced versus non-memory parent: about `1.9883x`
- induced versus parent plus frozen H4 memory baseline: about `1.4956x`
- matched random and permuted fixed-feature controls: passed

This supports only a primary shadow H5-C proof-of-mechanism candidate. It is not
replicated H5-C support, not AGI evidence, and not a live autonomy result.

Working principle:

> Push toward autonomous general intelligence by building bounded autonomy,
> durable goals, memory, self-evaluation, and self-correction under diagnostics
> that can fail honestly.

---

## Phase 0 - Claim hygiene and safety boundary

Goal: keep the project honest while autonomy increases.

Build:

- A written autonomy ladder with concrete levels.
- A distinction between diagnostic support, publishable evidence, and speculative direction.
- Explicit safety gates for destructive actions, external writes, credential use, and messaging.
- A standard experiment report template with hypothesis, controls, failure modes, and verdict.

Exit criteria:

- Every autonomy experiment has a falsifiable pass/fail condition.
- No experiment result is described as AGI evidence unless the claim is explicitly defined and earned.
- External side effects remain gated.

---

## Phase 1 - Stabilize H5 before expanding the claim

Goal: make the current non-memory diagnostic either pass cleanly or fail with a
specific causal explanation.

Status: complete for the current local diagnostic slice.

Build:

- Per-window failure attribution for `h5_task_profiled_nonmemory_diagnostic`.
- Resolver-level traces for windows that fail positive-margin directionality.
- Separate reports for resolver behavior, verifier scoring, fixture construction, and H4 filtering.
- A minimal patch loop that changes one suspected cause at a time.

Exit criteria:

- Canonical task-profiled arm reaches at least three stable directional windows while all controls remain degraded; or
- The H5 task-profiled hypothesis is rejected with a documented reason.

Current result:

- Per-window failure attribution is emitted in the diagnostic report.
- Canonical task-profiled future directionality reaches `4/4` windows.
- Matched controls remain degraded.
- The contradiction-correction fix is scoped to the canonical
  `TaskProfiled + PredictionContradiction` arm so surface, permuted-profile,
  and profile-blind controls do not inherit the canonical correction behavior.

Do not:

- Loosen the H5 gates just to pass.
- Treat surface-verifier failure as sufficient evidence for H5-C.
- Mix the H4 memory split back into the non-memory claim.

---

## Phase 2 - H5-C ontology induction gate

Goal: test whether Starfire can recover useful latent regimes without being
handed task profiles.

Build only after Phase 1 passes:

- An ontology-induction diagnostic over H4-retained non-memory observations.
- Hidden-label-free feature construction.
- Matched-budget controls for random features, permuted features, surface-only verifier scoring, and profile-blind scoring.
- Post-hoc cluster interpretation that reports alignment with hidden task families without using those labels during induction.

Exit criteria:

- Discovered clusters transfer to future windows.
- Controls fail or materially degrade.
- Hidden labels are used only for post-hoc reporting.
- Matched budgets are preserved.

Failure meaning:

- If induced clusters do not transfer, Starfire has not shown autonomous
  non-memory ontology discovery under this harness.

Current status:

- The primary `h5_non_memory_ontology_probe` verdict is `PASS`.
- The executable consumes H4-retained non-memory observations only, uses
  hidden-label-free fixed-width features, and reports exact matched-budget
  random and permuted fixed-feature controls.
- The primary result remains shadow-only and must be replicated before any
  stronger H5-C claim.

---

## Phase 3 - Bounded autonomous task loop

Goal: turn Starfire from a reactive system into a bounded actor with explicit
observe-plan-act-verify-memory cycles.

Build:

- A durable loop with these phases:
  - observe current state,
  - propose goal candidates,
  - select one goal,
  - form a plan,
  - execute local actions,
  - verify outcome,
  - write memory,
  - revise the next action.
- A local-only execution policy for safe actions such as reading files, running tests, writing reports, and proposing patches.
- A stop condition for each loop so autonomy does not become unbounded spinning.

Exit criteria:

- Starfire can complete a local multi-step task without being given every next command.
- The loop records enough state to explain what it did and why.
- Failed actions trigger a bounded retry or a clear stop, not silent drift.

---

## Phase 4 - Self-evaluation harness

Goal: measure agency as behavior, not vibes.

Build benchmark tasks for:

- Debugging.
- Planning.
- Memory recall.
- Causal explanation.
- Contradiction correction.
- Tool use.
- Long-horizon project continuation.

Each task must produce:

- Structured input.
- Expected success criteria.
- Machine-readable result.
- Human-readable trace.
- Failure classification.

Exit criteria:

- Starfire has a repeatable scorecard across the benchmark set.
- Regressions are visible.
- Improvements can be attributed to a change in memory, planning, tool use, or verifier logic.

---

## Phase 5 - Memory separation and ablation

Goal: make memory useful without confusing memory with reasoning.

Build:

- Episodic memory for what happened.
- Semantic memory for stable facts and preferences.
- Procedural memory for how to do recurring tasks.
- Retrieval tests that verify whether the right memory type is used.
- Ablation tests with memory removed, corrupted, delayed, or partially hidden.

Exit criteria:

- Starfire can resume tasks from durable memory.
- It can explain which memory type influenced a decision.
- It does not pass reasoning tasks only by leaking labels or rote memory.

---

## Phase 6 - Reflective error correction

Goal: make failures produce durable improvement.

Build:

- A structured failure record containing:
  - predicted cause,
  - evidence,
  - proposed correction,
  - minimal retest,
  - generalization check.
- A correction queue ranked by severity and confidence.
- A rule that every accepted correction must be tested against at least one near-neighbor task.

Exit criteria:

- Repeated failures decrease on the self-evaluation harness.
- Corrections generalize beyond the exact failed example.
- Bad corrections can be rolled back.

---

## Phase 7 - Long-horizon goal persistence

Goal: make Starfire able to pursue a project across sessions.

Build:

- A durable goal ledger with:
  - active goal,
  - subgoals,
  - blockers,
  - evidence,
  - last action,
  - next action,
  - stop condition.
- Resume tests where Starfire must continue a task after context loss.
- Conflict handling when new user instructions supersede an older goal.

Exit criteria:

- Starfire can resume a multi-day task without being re-ppecified from scratch.
- It distinguishes stale goals from current user intent.
- It preserves evidence and uncertainty across sessions.

---

## Phase 8 - Autonomy ladder

Goal: track progress with concrete behavior levels.

Levels:

- Level 0: answers prompts.
- Level 1: chooses among provided actions.
- Level 2: plans and executes local actions.
- Level 3: notices failures and retries.
- Level 4: preserves goals across sessions.
- Level 5: proposes useful new experiments.
- Level 6: improves its own procedures under test.
- Level 7: coordinates multiple tools and subsystems safely.

Exit criteria:

- Each level has at least one passing benchmark.
- Advancement requires repeatable performance, not a one-off demo.
- Any regression lowers the current supported level.

---

## Immediate next slice

Run Phase 2 replication as a shadow-only diagnostic, not a claim of discovery:

- preserve the primary H5-C verdict without tuning
- run the eight predeclared H5-C replication seeds
- report seed-level pass/fail outcomes, promoted concepts, control ratios, and
  margin-direction purity
- treat replication failure as scientific evidence, not as an implementation
  defect
- only after replicated success discuss shadow automatic promotion research
