# S6-A Reversible Live Policy Canary Result

Status: **EXPERIMENT_READY — mechanics conformance passed**

## Authoritative committed-source run

The first complete run after binding promotion authorization to the exact
`CompanionState` version compiled and linted the feature, executed the unit
contracts, ran the frozen canary probe, and validated the authority boundary:

```text
workflow: Companion Live Policy Canary S6-A CI
run id:   29258607758
head:     95e4caad32057fb37147e4e707954cbaef73e701
result:   success
artifact: 8282361331
artifact digest: sha256:7cbed2257720f40d07b99d818ba9c645b9ec46e70325c1dd60849adfe38f8662
```

## Frozen mechanics result

```text
terminal classification:              EXPERIMENT_READY
synthetic authorization refused:      true
real-canary influence path exercised: true
expected bounded policy returned:     true
stale authorization refused:          true
rollout fallback:                     true
compute fallback:                     true
contradiction fallback:               true
rollback fallback:                    true
failed evaluation latched rollback:   true
audit chain valid:                    true
source state unchanged:               true
neutral fallback exact:               true
gate passed:                          true
audit events:                         14
```

Every authority flag remained false:

```text
Runtime::chat() wiring:       false
generated-text mutation:      false
routing authority:            false
belief-promotion authority:   false
ontology-promotion authority: false
persistence authority:        false
action authority:             false
```

## Implemented boundary

The committed S6-A implementation establishes that Starfire can return a
companion-derived response-planning policy only when every preregistered canary
condition passes, while returning the exact neutral policy on every failure
path. `PromotionAuthorization` is deliberately non-serializable and has private
fields, so external callers cannot deserialize or reconstruct a forged
`RealHeldOut` capability token around the validated constructor.

The frozen probe covers:

- rejection of synthetic S5-C authorization for live use;
- rejection of stale authorization after the companion-state version changes;
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

## Interpretation and claim boundary

`EXPERIMENT_READY` is a mechanics classification, not a scientific capability
`PASS`. The frozen fixture contains an explicit real-held-out authorization
*attestation* solely to exercise the allowed branch; it does not contain real
user interactions and does not establish that companion-derived policies
improve real conversations.

No automatic runtime promotion is authorized. No `Runtime::chat()` wiring,
generated-text mutation, routing authority, persistence authority, belief or
ontology promotion, commitment mutation, capability invocation, or autonomous
action authority is introduced.

A scientific S6 result requires independently witnessed real interactions,
frozen opaque-subject and temporal holdouts, and another S5-C comparison against
all five controls under the preregistered thresholds.
