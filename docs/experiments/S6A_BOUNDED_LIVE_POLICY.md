# S6-A — Bounded Live Companion Policy

## Purpose

S6-A is the first opt-in response-planning boundary after S5-C. It does not make companion policy part of default `Runtime::chat()` behavior. It provides a separately invoked controller that can adjust only the typed response metadata already consumed by the reranker:

- response style hint;
- maximum output characters;
- structured companion-policy slots.

The response body is not changed by the controller. Tools, routing, memories, beliefs, ontology, persistence, and actions are outside its authority.

## Activation gate

A lease requires all of the following:

- a structurally complete S5-C `PASS` report;
- exactly five controls on both opaque-subject and temporal holdouts;
- every evidence and performance gate passing;
- all S5-C authority flags remaining false;
- an evaluation artifact digest;
- the exact positive companion-state version evaluated by that artifact;
- an explicit operator-approval digest;
- a non-abstaining `CompanionDerived` proposal;
- non-sensitive source evidence;
- minimum policy confidence;
- bounded activation duration and turn count.

The promotion gate is non-serializable, and activation rejects any proposal whose source companion version differs from the evaluated version. Synthetic evaluator conformance is rejected by the production-default controller. A dedicated configuration flag exists only so the frozen S6-A simulation can exercise the applied path.

## Eligibility boundary

S6-A applies only to low-risk informational intents:

- recall;
- teaching;
- capability explanation;
- research status.

All other intents receive exact neutral fallback. Sensitive-context turns also receive exact neutral fallback even when the source policy itself is non-sensitive.

## Policy effects

The five S5-A policy axes map to response-planning metadata:

- detail controls a bounded reranker character budget;
- explanation style controls the typed style hint;
- dialogue, vocabulary, and acknowledgment are emitted as structured slots.

The controller never edits the raw response body. The existing reranker may consume the style, slots, and character budget through its normal interface.

## Budgets and reversal

Each activation is a lease with:

- one opaque subject scope;
- a valid-from and expiration time;
- a maximum number of applied turns;
- source companion version, claim IDs, confidence, and policy digest;
- promotion-gate and operator-approval digests.

Only applied turns consume the turn budget. Neutral fallbacks do not. Any later companion-state version change immediately forces exact neutral fallback without consuming budget. Revocation is immediate and replayable.

## Audit and replay

Every activation, planned turn, fallback, and revocation is recorded as a typed event. Turn records contain digests and planning metadata, not raw user input or raw response text.

Replay reconstructs the active lease, remaining budget, seen turn digests, event history, and controller version exactly.

## Frozen probe

The deterministic probe requires:

- structural S5-C promotion-gate validation;
- production-default rejection of simulated evidence;
- rejection of activation from a mismatched companion version;
- exact neutral fallback after companion-version drift;
- an explicit simulation override for the synthetic applied path;
- unchanged response body before reranking;
- bounded style and character metadata changes;
- actual reranker output respecting the brief budget;
- exact neutral fallback for sensitive context;
- exact neutral fallback for a disallowed emotional intent;
- turn-budget exhaustion;
- immediate revocation;
- exact replay.

## Authority boundary

S6-A has no:

- default `Runtime::chat()` wiring;
- tool or action selection;
- response routing;
- CHARGE authority;
- companion-state mutation;
- persistence authority;
- belief or ontology promotion;
- autonomous side-effect authority.

S6-B should add adversarial rollback, malformed-policy, cross-subject, and unsafe-context stress testing before any default runtime integration is considered.
