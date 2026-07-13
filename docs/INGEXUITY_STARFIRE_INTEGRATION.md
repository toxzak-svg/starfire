# IngExuity → Starfire Native Integration

## Decision

Starfire remains the cognitive and execution substrate. IngExuity becomes the companion behavior and user-modeling layer inside the `star` crate.

This integration does **not** introduce an HTTP bridge, a second runtime, a second inference scheduler, or a second persistence authority.

## Architectural boundaries

Starfire continues to own:

- runtime and action authority;
- CHARGE accounting, routing, and independent discharge judgment;
- executable commitment state and representation growth;
- persistence transactions and replay;
- inference adapters and resource budgets;
- safety, capability, and autonomy boundaries.

The companion layer owns:

- provenance-aware claims about the user;
- supporting observations;
- explicit user correction, invalidation, and deletion;
- sensitivity and retention policy metadata;
- contradiction detection;
- falsifiable predictions and independently witnessed outcomes;
- shadow interaction-policy proposals and matched controls;
- response-policy inputs derived from validated companion state only after later evaluation gates.

## Implemented slices

### S0 — typed companion state

Merged in Starfire PR #45:

- `CompanionState` with optimistic versions;
- typed claims and observations;
- explicit source provenance;
- active, contested, superseded, and invalidated claim states;
- user corrections that supersede active claims;
- deletion of a claim and its supporting observations;
- sensitivity-aware and retention-aware query policy;
- deterministic event replay;
- CHARGE emission for unresolved contradictory claims;
- no live `Runtime::chat()` wiring.

The key authority rule is:

> An inferred contradiction may become a contested claim and emit CHARGE, but it may not silently replace an active user claim.

### S1 — Starfire persistence adapter

Merged in Starfire PR #47:

- one versioned companion journal inside Starfire's existing SQLite `Store`;
- SQLite compare-and-swap across threads, Store instances, and processes;
- restart recovery from a durable checkpoint;
- deterministic exclusion of `Retention::Session` claims and observations;
- source-version continuity across session-only transitions;
- durable event audit without making the audit tail a second state authority;
- deletion compaction that clears prior raw event history;
- repair of the pre-existing unclosed transaction in `Store::open()`;
- no second database, server, inference loop, or action authority.

### S2 — legacy user-model bridge

Merged in Starfire PR #48:

- deterministic read-only projection from `CompanionState` to `UserCognitionModel`;
- explicit confidence, sensitivity, and session-retention policy;
- active-claim-only semantics, preserving correction and invalidation precedence;
- provenance mapping from user statements/corrections to teaching and inferred/imported claims to observation;
- compatibility schema for strong/weak domains, answer-detail, brevity, questions, argument style, and response patterns;
- source claim IDs and unrecognized claim IDs preserved for audit;
- no reverse mutation from the legacy model into companion state;
- no `Runtime::chat()` or response-routing integration.

### S3 — bounded shadow observation

Completed by merged PR #50 after repairing the incomplete PR #49 integration:

- a default-off `companion-observer` feature;
- actual library export and compilation of the observer;
- sentence-boundary eligibility rather than substring extraction;
- matched controls for explicit, negated, quoted, third-person, hypothetical, and adversarial language;
- a frozen executable probe with 9 true-positive fixtures and 15 true-negative controls;
- zero observed false positives and zero observed false negatives on the frozen corpus;
- dedicated formatting, compilation, scoped lint, unit-test, and probe execution;
- no state mutation, persistence authority, response routing, `Runtime::chat()` wiring, or action authority.

S3 produces inert `ClaimInput` proposals only. A separate reviewed boundary must decide whether any proposal becomes a companion-state event.

### S4 — falsifiable prediction ledger

Merged in Starfire PR #53:

- typed monotonic prediction and abstention IDs;
- producer provenance and opaque subject scope;
- canonical multiclass outcome distributions in basis points;
- issue time, earliest valid witness time, expiration, and context digest;
- explicit abstention rather than forced prediction;
- pending, resolved, and expired status;
- rejection of response-generator self-grading;
- rejection of premature, post-expiration, unknown-label, and duplicate witnesses;
- exact multiclass Brier scoring and aggregate calibration buckets;
- deterministic typed events and exact replay validation;
- a frozen temporal probe against majority, recency, scrambled-scope, and oracle controls;
- no response-policy influence, routing authority, belief promotion, persistence authority, or autonomous side effects.

The synthetic S4 control result validates the ledger and evaluation harness. It does not establish that current companion predictions generalize to real conversations.

### S5-A — shadow interaction-policy proposals

Merged in Starfire PR #55:

- complete bounded policies over detail, explanation style, dialogue mode, vocabulary, and acknowledgment;
- source claim IDs, confidence, update times, sensitivity, and companion version preserved in every candidate;
- default exclusion of sensitive, expired, low-confidence, and non-active claims;
- explicit candidate abstention for insufficient or contradictory evidence;
- six deterministic arms: companion-derived, neutral, recency-only, majority prior, context-only, and scrambled scope;
- deterministic policy digests and opaque subject scopes;
- atomic clone-then-commit enrollment into the S4 ledger;
- pending predictions or explicit abstentions only;
- exact S4 replay equality in the frozen fixture;
- no `Runtime::chat()` wiring, generated-text influence, routing authority, belief promotion, persistence authority, or autonomous actions.

The frozen S5-A fixture validates controlled experiment construction. It does not establish that the companion-derived arm improves user outcomes.

### S5-B — independently witnessed outcome collection

Merged in Starfire PR #58:

- typed trial registration over the exact six S5-A arms;
- one optional declared delivered arm per trial;
- direct user or environment evidence restricted to the delivered arm;
- pure-shadow trials that reject direct observed evidence;
- explicit positive, negative, correction, clarification, completion, abandonment, and neutral signals;
- inconclusive neutral signals without forced S4 resolution;
- rejection of response-generator self-grading;
- rejection of external evaluator evidence on the direct-observation channel;
- offline paired evaluation requiring two distinct policy arms and two distinct render digests;
- zero S4 resolutions for a tie and exactly two for a decisive comparison;
- atomic clone-then-commit updates across the S5-B ledger and its mirrored S4 ledger;
- deterministic replay from a captured base S4 state with exact transition equality;
- no live response influence, routing, companion-state mutation, belief promotion, persistence authority, or autonomous actions.

The central counterfactual rule is:

> A witness may resolve only the response it actually observed. Unshown arms remain pending unless an external evaluator explicitly compares rendered alternatives.

### S5-C — comparative policy evaluation

Merged in Starfire PR #61:

- deterministic development, opaque-subject holdout, and temporal-holdout assignment from pre-outcome metadata only;
- complete per-arm accounting for predictions, resolutions, pending outcomes, expirations, abstentions, Brier score, and calibration error;
- delivered-arm correction, clarification, completion, and abandonment rates;
- candidate-relative pairwise wins, losses, ties, and signed win margins;
- mandatory positive compute observations for every arm in every trial;
- separate candidate-versus-control comparisons for all five controls on both holdouts;
- minimum evidence gates before performance is judged;
- Brier improvement, calibration, burden, completion, abstention, and compute non-regression gates;
- explicit `PASS`, `FAIL`, and `INCONCLUSIVE` verdicts;
- structural exclusion of development evidence from the verdict;
- deterministic repeated evaluation and a frozen synthetic probe;
- no live response influence, routing, companion-state mutation, belief promotion, persistence authority, or autonomous actions.

The synthetic S5-C `PASS` validates the evaluator and gate composition only. Real promotion eligibility requires frozen real-world held-out evidence under the same preregistered contract.

## Current slice

### S6-A — reversible live policy canary

The feature-gated S6-A implementation provides:

- default-disabled, audit-only, and bounded live-canary modes;
- explicit promotion authorization tied to one passing S5-C report fingerprint, one canonical artifact digest, and the exact evaluated companion-state version;
- structural refusal to treat synthetic evaluator conformance as real live-use evidence;
- deterministic opaque-subject rollout admission;
- exact neutral fallback for missing or stale authorization, rollout exclusion, abstention, sensitive evidence, claim-budget failure, compute-budget failure, and rollback;
- hash-chained authorization, decision, and rollback audit events;
- source companion version, source claim IDs, context and subject digests, candidate/effective policy digests, delivered arm, compute, and fallback reason in every decision record;
- rollback latching on failed or inconclusive evaluation;
- exact-generation, operator-audited rollback clearing;
- no direct `Runtime::chat()` wiring, generated-text mutation, routing, persistence, belief or ontology promotion, capability invocation, or autonomous action authority.

The frozen S6-A probe has terminal classification `EXPERIMENT_READY`. It validates canary mechanics only. A scientific result requires independently witnessed real interactions and a subsequent frozen S5-C held-out evaluation.

## Later slices

### S6-B — runtime-owned companion response planning

After S6-A receives real held-out support, add one reviewed runtime-owned adapter that loads `CompanionState` through Starfire's existing persistence authority, invokes the canary before response rendering, records the delivered arm through S5-B, and preserves unconditional neutral fallback. No second state loader, database, inference loop, or action authority is permitted.

## Required invariants

1. User correction outranks inference.
2. Text generation never validates its own prediction.
3. Contradiction is unresolved computation, not permission to overwrite state.
4. Deletion must remove live state and trigger durable redaction/compaction.
5. Sensitive claims are private by default.
6. Every mutation is versioned and replayable.
7. No companion component receives autonomous side-effect authority.
8. Runtime promotion occurs only after shadow evaluation against controls.
9. Synthetic evaluator conformance never authorizes live response influence.
10. Every S6 live decision has an exact neutral fallback and auditable rollback path.
