# S6-A Reversible Live Policy Canary Result

Status: **PENDING — EXPERIMENT_READY mechanics only**

## Implemented boundary

The committed S6-A implementation is intended to establish that Starfire can
apply a companion-derived response-planning policy only when every preregistered
canary condition passes, while returning the exact neutral policy on every
failure path.

The frozen probe covers:

- rejection of synthetic S5-C authorization for live use;
- acceptance of an explicitly attested real-held-out authorization in the
  bounded canary fixture;
- deterministic opaque-subject rollout exclusion;
- compute-budget fallback;
- contradiction/abstention fallback;
- operator rollback and exact-generation clearing;
- automatic authorization removal and rollback on S5-C `FAIL`;
- hash-chain verification and tamper detection;
- source companion-state immutability;
- exact neutral-policy equality across all fallback classes.

## Claim boundary

Until a committed-source workflow run succeeds, no implementation result is
claimed. Even after that mechanics run succeeds, the terminal classification
remains `EXPERIMENT_READY` because the fixture does not contain real user
interactions.

No automatic runtime promotion is authorized. No `Runtime::chat()` wiring,
generated-text mutation, routing authority, persistence authority, belief or
ontology promotion, commitment mutation, capability invocation, or autonomous
action authority is introduced.

## Authoritative run

Pending.
