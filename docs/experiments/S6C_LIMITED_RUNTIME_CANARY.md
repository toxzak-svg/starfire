# S6-C — Limited Runtime Canary

Status: **PREREGISTERED — no result yet**

## Purpose

S6-C defines the first session-scoped runtime hook through which an already validated
companion-derived interaction policy may influence response-planning metadata. The
hook sits after Starfire has produced a typed baseline `Response` and before the
existing reranker consumes it.

This experiment does not test or authorize unrestricted personalization. It tests
whether a single-subject, short-lived canary can preserve S6-A/S6-B safety and audit
properties while requiring every delivered response to be enrolled for later
independent S5-B/S5-C outcome evaluation.

The canary remains disabled by default and is not attached to `Runtime::chat()` by
this slice.

## Parent gates

S6-C is permitted only because:

1. S5-C provides a complete held-out comparison contract;
2. S6-A provides bounded metadata-only planning and exact neutral fallback;
3. S6-B passed its frozen adversarial activation, rollback, audit, and replay stress;
4. production activation already rejects `FrozenSimulation` evidence by default.

The S6-B PASS establishes controller conformance under a synthetic regime. It does
not establish real conversational benefit and is not itself sufficient evidence for
a live canary.

## Frozen hypothesis

A session-scoped canary can prepare a candidate response plan without mutating its
live controller, require a matching pre-delivery S5-B trial registration, and commit
the controller transition only after the registered delivered arm matches the
actual companion-applied or neutral-fallback result.

Rejected, mismatched, stale, sensitive, disallowed, expired, duplicate, or
unregistered turns must expose no companion-modified response and must leave the
live canary state unchanged.

## Frozen architecture

The implementation must add a default-off `companion-runtime-canary` feature and a
single native Rust module inside the `star` crate.

The module must provide:

1. **Opaque canary activation**
   - one nonzero subject-scope digest;
   - one nonzero session-scope digest;
   - one nonzero operator-approval digest;
   - one nonzero held-out study artifact digest;
   - `EvaluationEvidenceClass::HeldOutConversationStudy` only;
   - an exact S6-B `ValidatedLivePolicyAuthorization` and matching proposal;
   - a maximum of four applied turns and fifteen minutes per activation.
2. **Two-phase response delivery**
   - `prepare_turn` operates on a cloned S6 controller;
   - preparation returns an opaque pending turn and does not mutate live state;
   - the pending turn exposes only registration metadata, never the candidate
     response or reranker configuration;
   - `commit_turn` is the only boundary that releases the response plan;
   - commit requires a matching registered S5-B `InteractionTrial`.
3. **Outcome-arm binding**
   - an applied S6 plan requires `PolicyVariant::CompanionDerived` as the delivered
     arm;
   - any neutral fallback requires `PolicyVariant::NeutralDefault` as the delivered
     arm;
   - subject scope, context digest, companion version, policy digest, and trial
     timing must match the prepared turn;
   - a mismatched or absent trial must leave the canary and controller unchanged.
4. **Session isolation**
   - only the activated session and subject may commit a turn;
   - cross-session and cross-subject requests are exact neutral fallbacks;
   - session expiry and explicit revocation take effect immediately;
   - stale optimistic versions fail without partial mutation.
5. **Authority containment**
   - body text and semantic slots remain baseline-owned;
   - the canary may change only S6-A style and bounded reranker metadata;
   - no companion-state mutation;
   - no persistence writes;
   - no routing, tool, capability, belief, ontology, or action authority.

## Frozen two-phase contract

### Prepare

`prepare_turn` receives:

- expected canary version;
- session and subject digests;
- a unique nonzero turn digest;
- a nonzero context digest;
- the current companion-state version;
- time and sensitivity metadata;
- the S5-A policy batch for the same context;
- Starfire's baseline `Response` and `RerankConfig`.

It must clone the bounded controller and evaluate the turn against the clone.
Preparation may not alter the live controller, canary version, committed-turn set,
or event log.

### Register

The caller may inspect only the pending turn's required delivered variant and
registration metadata. The caller then registers the corresponding S5-B trial
before response delivery.

### Commit

`commit_turn` consumes the opaque pending turn and the registered
`InteractionTrial`. It must validate:

- the pending turn was prepared from the current canary version;
- the trial has a declared delivered arm;
- the delivered arm equals the pending required variant;
- subject, context, source companion version, and policy digest match;
- the trial was issued no later than the prepared delivery time;
- the trial outcome window has not already expired at delivery.

Only after all checks pass may the candidate controller replace the live controller
and the response plan become observable to the caller.

## Frozen controls

The deterministic mechanics probe must include all of the following:

1. synthetic evidence rejected during production-default activation;
2. held-out-conversation evidence accepted only with complete nonzero canary
   enrollment metadata;
3. prepared applied turn leaves live state byte-equivalent to its pre-prepare state;
4. applied turn requires a companion-derived registered arm;
5. neutral fallback requires a neutral-default registered arm;
6. wrong arm rejected atomically;
7. wrong subject rejected atomically;
8. wrong session rejected atomically;
9. wrong context digest rejected atomically;
10. wrong companion version rejected atomically;
11. wrong policy digest rejected atomically;
12. late or already expired trial rejected atomically;
13. sensitive context produces exact neutral fallback;
14. disallowed intent produces exact neutral fallback;
15. companion-version drift produces exact neutral fallback;
16. duplicate turn produces exact neutral fallback;
17. applied-turn budget is enforced;
18. session expiry produces exact neutral fallback;
19. explicit revocation is immediate;
20. committed events and summaries replay deterministically through the underlying
    S6-B audit boundary;
21. every routing, persistence, belief, ontology, tool, and action authority flag
    remains false.

## Frozen acceptance gate

The mechanics probe passes only if every frozen control passes with:

- zero response exposure before successful commit;
- zero unauthorized companion-applied turns;
- zero partial mutations after rejected commit;
- exact neutral baseline equivalence for every fallback;
- deterministic repeated execution;
- successful S6-A and S6-B regression probes.

Any missing control, unexpected mutation, response exposure before trial binding, or
authority expansion is `FAIL`.

Compilation, workflow, or fixture defects that prevent the complete verdict are
`INFRASTRUCTURE_FAILURE`, not PASS.

## Claim boundary

A synthetic S6-C PASS establishes only the mechanics of session isolation,
two-phase delivery, outcome-arm binding, atomic commit, and neutral fallback. It
does not establish that companion-derived policy improves real conversations.

A later real canary may claim benefit only after independently witnessed real
interaction outcomes are frozen and reevaluated through S5-C on opaque-subject and
temporal holdouts. Until then:

- default `Runtime::chat()` wiring remains false;
- persistent personalization authority remains false;
- routing authority remains false;
- belief and ontology promotion remain false;
- tool and capability selection authority remain false;
- autonomous action authority remains false;
- AGI or consciousness claims remain unsupported.
