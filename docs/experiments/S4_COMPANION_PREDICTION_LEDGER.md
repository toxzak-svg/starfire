# S4 — Falsifiable Companion Prediction Ledger

## Purpose

S4 turns companion expectations into replayable commitments that can be proven wrong. A prediction is recorded before its outcome window and remains unresolved until later evidence arrives from an independent witness.

This is an epistemic-accounting substrate. It does not select responses, route cognition, promote beliefs, or authorize actions.

## Implemented contract

Each prediction records:

- a monotonic prediction ID and ledger version;
- opaque subject scope;
- typed producer provenance;
- two or more canonical outcome labels;
- an exact probability distribution in basis points summing to 10,000;
- issue time, earliest valid witness time, and expiration;
- a context digest rather than raw conversational evidence;
- pending, resolved, or expired status.

The ledger also records explicit abstentions when evidence is insufficient.

## Resolution rules

A prediction may be resolved exactly once. Resolution is rejected when:

- the witness is the response generator;
- evidence arrives before the declared outcome window;
- evidence arrives after expiration;
- the witnessed label was not declared at issue time;
- the prediction was already resolved or expired;
- replayed scoring differs from the deterministic recomputation.

Accepted outcomes receive an exact multiclass Brier score. Aggregate summaries report pending, resolved, expired, and abstained counts plus mean Brier score. Top-label calibration buckets compare declared confidence with observed accuracy.

## Replay

Every mutation is represented as a typed event:

- `Issued`;
- `Resolved`;
- `Expired`;
- `Abstained`.

Replaying the ordered events from an empty ledger must reconstruct the exact state. Replay validates IDs, canonical fields, witness timing, finalization status, and scores rather than trusting serialized results.

## Frozen S4 probe

The executable probe uses a temporal six-case fixture and requires:

- all real predictions to remain pending until later independent evidence;
- rejection of response-generator self-grading;
- rejection of premature evidence;
- rejection of duplicate resolution;
- exact live-versus-replay equality;
- one explicit abstention and one expiration;
- candidate Brier score better than matched majority, recency, and scrambled-scope controls;
- oracle performance no worse than the candidate.

The synthetic control result validates the ledger and evaluation harness. It does not establish real-world personalization capability.

## Authority boundary

S4 has no:

- `Runtime::chat()` integration;
- response-policy influence;
- CHARGE routing authority;
- belief or ontology promotion;
- persistence adapter in this slice;
- autonomous side-effect authority.

S5 may evaluate interaction policies only after this ledger is stable and only against matched controls with independently observed outcomes.
