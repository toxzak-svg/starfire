# S6-A Reversible Live Policy Canary

## Purpose

S6-A is the first Starfire boundary permitted to return a companion-derived
response-planning policy for actual use by a caller. It does not wire itself
into `Runtime::chat()`. Instead, it exposes one explicit feature-gated canary
that a reviewed caller may invoke and whose result is either:

- the bounded companion-derived `InteractionPolicy`; or
- the exact neutral S5-A policy with a typed fallback reason.

This avoids introducing a second companion-state loader or a second persistence
authority while the runtime still has no native durable `CompanionState` field.

## Prerequisites

S6-A depends on the complete S0 through S5-C chain. In particular, an installed
promotion authorization must be derived from an S5-C report that:

- has verdict `PASS` and `promotion_eligible = true`;
- excludes development evidence from its verdict;
- contains exactly ten candidate-control comparisons: five controls on the
  opaque-subject holdout and the same five on the temporal holdout;
- passes every evidence and performance gate;
- claims no live, routing, belief-promotion, or action authority.

A synthetically conformant report may be installed for audit testing, but it can
never authorize live influence. Live influence requires an explicit
`RealHeldOut` evidence attestation and a canonical `sha256:` artifact digest.

## Frozen decision order

Every decision is evaluated in this order:

1. rollback latch;
2. configured mode (`Disabled`, `AuditOnly`, or `LiveCanary`);
3. deterministic opaque-subject rollout partition;
4. installed promotion authorization;
5. real-held-out evidence class;
6. S5-A candidate abstention;
7. sensitive-evidence exclusion;
8. source-claim budget;
9. positive compute observation within budget.

The first failed gate selects the exact S5-A neutral policy. There is no partial
policy merge and no best-effort degradation.

## Audit and rollback

The canary records hash-chained typed events for:

- authorization installation and removal;
- every candidate and effective-policy decision;
- rollback latching;
- explicit rollback clearing.

Each decision preserves the source companion version, source claim IDs, context
and subject digests, candidate and effective policy digests, selected arm,
compute observation, fallback reason, and rollback generation. Audit-chain
verification recomputes every event digest and rejects sequence, predecessor, or
payload tampering.

A failed or inconclusive evaluation removes the installed authorization and
latches rollback. Rollback clearing requires the exact current generation and a
nonzero operator digest. A passing later evaluation does not silently clear a
latched rollback.

## Authority boundary

S6-A may return an `InteractionPolicy` to a reviewed response-planning caller.
It does not:

- mutate generated text itself;
- modify `Runtime::chat()`;
- select models or tools;
- route CHARGE;
- persist companion state or audit events;
- promote beliefs or ontology entries;
- modify commitments;
- invoke capabilities or autonomous actions.

## Initial classification

The frozen probe terminal classification is `EXPERIMENT_READY`, not `PASS`.
The probe proves canary mechanics, exact fallback, audit integrity, and rollback.
It does not provide real held-out conversational evidence. A scientific S6
result requires separately collected, independently witnessed real interactions
and a subsequent S5-C evaluation under the frozen split and comparison contract.
