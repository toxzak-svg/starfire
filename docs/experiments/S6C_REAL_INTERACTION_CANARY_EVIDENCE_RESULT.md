# S6-C Real-Interaction Canary Evidence Result

Status: **PASS — synthetic intake-mechanism conformance only**

## Frozen prerequisite

The preregistration is in
`docs/experiments/S6C_REAL_INTERACTION_CANARY_EVIDENCE.md` and was committed before
implementation at:

```text
469654d1ce2008888e6804d407d17d37379a88f1
```

A post-freeze review found that the original prose mistakenly included
`ExternalEvaluator` as an allowed direct witness, which conflicts with the already-frozen
S5-B channel contract. The recorded erratum narrows direct intake to `UserObservation` and
channel-compatible `Environment` evidence. External evaluators remain pairwise-only.
This correction is restrictive and changes no threshold or authority boundary.

## Authoritative verification

The source-producing workflow transformed, formatted, compiled, linted, tested, ran the
frozen probe, removed its one-shot machinery, and committed the verified permanent source.

```text
workflow:        Apply S6-C split-binding hardening
run id:          29273459233
input head:      6f45be2ed960f5a879421cb034131e16e5dd6c01
source commit:   be0870f442eedab11060b367248b30b6671d3a85
artifact:        8288435830
artifact digest: sha256:62f956494020c5c379d2f3fcff5ce0da0b073387c9cbcc31cd56fb15ef005bc8
```

The same input head also passed the repository's S4, S5-A, S5-B, S5-C, S6-A, S6-B,
CHARGE, Render smoke, H13-C, relational residual, infant fusion, and STLM regression
workflows. The S6-C permanent workflow on that pre-transformation head stopped at the
expected formatting gate; the source-producing workflow then formatted and verified the
transformed source before committing it.

After the verified source commit, GitHub Actions began rejecting every repository workflow
before the first setup step, including unrelated CHARGE, Render, S4-S6, STLM, H13-C,
relational, infant, and Omega jobs. Those zero-step failures contain no compiler or test
result and are recorded as an Actions execution outage rather than experiment evidence.

## Frozen probe outcome

```text
mechanism_gate_passed:             true
sealed_trials:                     33
direct_imports:                    18
pairwise_imports:                  15
independent_witnesses:             33
synthetic_evidence:                33
real_evidence:                     0
raw_content_retained:              false
canary_replay_equal:               true
s5b_replay_equal:                  true
source_state_unchanged:            true
underlying_s5c_verdict:            Pass
underlying_s5c_promotion_eligible: true
s6c_canary_promotion_eligible:     false
```

Every original preregistered adversarial control passed:

- production-default synthetic-evidence rejection;
- duplicate-seal rejection;
- stale-version atomic rejection;
- unsealed-trial rejection;
- response-generator witness rejection;
- same-identity producer/witness rejection;
- consent-mismatch rejection;
- early and expired evidence rejection;
- duplicate direct-evidence rejection;
- invalid and duplicate pairwise-evidence rejection.

All rejected operations preserved the canary, S5-B, and S4 ledgers exactly.

The channel erratum relies on the existing verified S5-B rule that rejects
`WitnessSource::ExternalEvaluator` on direct observed evidence as `WrongEvidenceChannel`.
A dedicated S6-C executable regression was added to assert that this rejection preserves
all three ledgers atomically. It changes no production implementation path; it exercises
the already-existing S5-B enforcement through S6-C's copy-on-write import boundary.

## What passed

S6-C now provides a typed, privacy-minimized intake boundary that:

- seals an existing S5-B trial and its frozen S5-C split before outcomes;
- recomputes split assignment from the frozen split policy during intake and replay;
- stores typed fields and opaque digests rather than raw conversational content;
- imports valid direct and blind pairwise evidence atomically through S5-B;
- distinguishes `RealInteraction` from `SyntheticFixture` in the type system;
- rejects synthetic fixtures in production-default mode;
- prevents synthetic evidence from granting S6-C promotion eligibility;
- keeps external evaluators on the S5-B pairwise-only channel;
- replays deterministically;
- retains no response, routing, belief, persistence, tool, or action authority.

## Claim boundary

This PASS validates the **engineering mechanism using synthetic fixtures**. It does not
establish real-user preference, statistical power, open-world safety, broader deployment
readiness, autonomous agency, or AGI.

Real-interaction promotion eligibility remains pending. It requires a later consented
canary dataset containing only `RealInteraction` attestations and a passing frozen S5-C
held-out evaluation.