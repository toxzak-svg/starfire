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

## Current slice

### S4 — falsifiable prediction ledger

The feature-gated S4 implementation provides:

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

## Later slices

### S5 — interaction-policy evaluation

Evaluate response adaptation and emotional-interaction policies against matched controls. Promotion requires independent outcome evidence and may be rolled back.

### S6 — bounded live use

Permit validated, non-sensitive active claims to influence response planning under explicit budgets and audit logs. Sensitive claims remain excluded unless the calling policy explicitly authorizes them.

## Required invariants

1. User correction outranks inference.
2. Text generation never validates its own prediction.
3. Contradiction is unresolved computation, not permission to overwrite state.
4. Deletion must remove live state and trigger durable redaction/compaction.
5. Sensitive claims are private by default.
6. Every mutation is versioned and replayable.
7. No companion component receives autonomous side-effect authority.
8. Runtime promotion occurs only after shadow evaluation against controls.
