# S5-A — Shadow Interaction-Policy Proposals

## Purpose

S5-A converts eligible companion claims into typed interaction-policy proposals and enrolls every candidate and control as an unresolved S4 prediction before any outcome window begins.

This slice does not alter generated text. It establishes the experimental boundary required to test whether personalization adds information beyond trivial shortcuts.

## Policy dimensions

Each proposal contains a complete, bounded policy over:

- answer detail: brief, standard, or detailed;
- explanation style: concrete, abstract, or adaptive;
- dialogue mode: direct or question-led;
- vocabulary: plain, standard, or technical;
- acknowledgment: minimal or standard.

The candidate policy may use only eligible active companion claims. Every contributing claim ID, confidence, update time, key, sensitivity, and source companion version is preserved in the proposal.

## Eligibility and abstention

Default policy excludes:

- sensitive claims;
- expired claims;
- contested, superseded, or invalidated claims;
- claims below 7,000 basis points confidence.

Session claims remain eligible because S3 deliberately emits session-scoped proposals. Contradictory active detail and brevity claims, or simultaneous strong-domain and weak-domain claims for the same context, force candidate abstention. Insufficient evidence also forces candidate abstention.

Abstention is written to S4 rather than converted into a low-confidence prediction.

## Matched arms

Every eligible context produces six deterministic arms:

1. companion-derived candidate;
2. neutral default;
3. recency-only;
4. majority prior;
5. context-only;
6. scrambled-scope control.

All non-abstaining arms declare the same two outcome labels and are enrolled before the response outcome can be observed. The scrambled arm uses the same bounded policy dimensions and candidate evidence budget while deterministically reversing policy choices.

## S4 enrollment

Enrollment is atomic with respect to the in-memory S4 ledger: a working clone is updated first and replaces the caller's ledger only if all six transitions succeed.

Each arm receives:

- a typed producer ID;
- an opaque subject scope;
- issue, earliest witness, and expiration times;
- a combined context-and-policy digest;
- a pending S4 prediction or explicit abstention.

S5-A does not resolve outcomes. Resolution remains subject to S4's delayed independent-witness rules.

## Frozen probe

The frozen fixture requires:

- all six arms to be present exactly once;
- deterministic repeated planning;
- complete candidate provenance;
- sensitive claims excluded by default;
- contradictory preferences to produce one candidate abstention;
- all ordinary predictions to remain pending;
- exact S4 replay equality;
- source companion state to remain unchanged;
- unique policy digests across arms;
- no runtime, response, routing, belief, or action authority.

## What this does not prove

Passing S5-A proves that Starfire can construct and account for a controlled personalization experiment. It does not prove that the companion-derived policy improves user outcomes.

S5-B must collect independently witnessed outcomes on temporal and held-out splits. S5-C must compare Brier score, calibration, correction rate, clarification burden, completion, abstention quality, and compute overhead before any live response influence is considered.
