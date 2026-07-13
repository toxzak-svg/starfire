# S6-B Adversarial Live-Policy Stress Result

Status: **PASS — synthetic controller and audit/replay conformance only**

## Scientific provenance

The frozen preregistration was committed before implementation:

```text
e9511adc5b0df591f2a876a9429e4db59560f8aa
```

The authoritative committed-source run was:

```text
workflow: Companion Live Policy S6-B CI
run id:   29266883291
head:     68e1f518d08ffd61f38fe51b03484ad70c957cdc
result:   success
artifact: 8285821999
artifact digest: sha256:e7b455d349f3ded469e93c11d345ac49a8f4c03ec5aa260f5dd9c7e7f7919e63
```

The run enforced canonical formatting, compiled the hardened controller and both
probes, completed scoped Clippy without S6 findings, ran the deterministic unit
contracts, reran the frozen S6-A regression, and executed the frozen S6-B
adversarial probe.

## Frozen S6-B result

```text
exact proposal mismatch rejected:       true
production simulation rejected:         true
cross-subject request neutral:           true
sensitive context neutral:               true
disallowed intent neutral:               true
not-yet-valid lease neutral:             true
expired lease neutral:                   true
companion-version drift neutral:         true
stale plan-version mutation atomic:      true
stale revoke-version mutation atomic:    true
revocation immediate:                    true
post-revocation planning neutral:        true
trusted replay exact:                    true
replay without authorization rejected:   true
forged policy replay rejected:           true
forged evidence class rejected:          true
malformed lease rejected:                true
reordered events rejected:               true
reordered audit chain rejected:          true
deleted chain record rejected:           true
neutral-plan tampering rejected:         true
applied-plan tampering rejected:         true
replay deterministic:                    true
applied turns:                              2
neutral fallbacks:                          7
unauthorized applied turns:                 0
gate passed:                             true
```

## S6-A regression

The hardened replay and authorization boundary preserved the complete S6-A
contract:

```text
promotion gate validated:                 true
production simulation rejected:           true
explicit simulation override required:    true
activation mismatch rejected:             true
version-drift neutral fallback:            true
bounded metadata-only applied path:        true
brief reranker budget respected:           true
sensitive-context neutral fallback:        true
duplicate-turn neutral fallback:           true
disallowed-intent neutral fallback:        true
turn budget enforced:                      true
revocation immediate:                      true
exact replay:                              true
S6-A gate passed:                          true
```

## Hardening established

S6-B closes the demonstrated activation-replay gap by:

- retaining the version-bound S5-C promotion gate as a non-serializable
  capability;
- deriving a second non-serializable authorization bound to the exact companion
  proposal, including policy axes, policy digest, confidence, claim IDs, evidence
  class, and companion-state version;
- requiring trusted exact-proposal authorization for activation and replay;
- rejecting malformed or authorization-mismatched leases before controller
  mutation;
- exporting sequence-checked, hash-chained audit records;
- validating replayed fallback reasons and bounded planning metadata against the
  reconstructed controller state;
- reconstructing into a fresh candidate controller so rejected replay cannot
  partially mutate an existing controller.

The FNV-1a audit chain supplies deterministic ordering and accidental-tamper
integrity for this experiment. It is not cryptographic authentication and is not
represented as such.

## Authority boundary

Every authority flag remained false:

```text
default Runtime::chat() wiring: false
routing authority:              false
belief-promotion authority:     false
ontology-promotion authority:   false
persistence authority:          false
action authority:               false
```

## Interpretation

This PASS establishes adversarial conformance of the bounded live-policy
controller and its trusted audit/replay boundary under the frozen synthetic
regime. It does not establish that companion-derived policy improves real
conversations, authorize default runtime integration, grant persistent live
personalization, validate open-world safety, or demonstrate autonomous agency or
AGI.
