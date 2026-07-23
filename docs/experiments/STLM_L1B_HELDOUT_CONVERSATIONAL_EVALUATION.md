# STLM L1-B Held-Out Conversational Evaluation

## Status

Preregistered, offline-only evaluation of the STLM L1-A verified improvisation selector.

This experiment does not add runtime, HTTP, routing, memory, persistence, tool, CHARGE, belief, ontology, companion-state, or autonomous-action authority. A passing result is evidence for a later shadow-integration proposal only. It is not permission for live text influence.

## Question

Does the verifier-backed improvisation selector preserve exact semantic authorization while producing replayable, nontrivially varied conversational surfaces on scenarios that were not used by the L1-A unit fixture?

## Frozen corpus

The evaluator contains ten independent scenarios:

1. iteration tradeoff
2. fluency correction
3. causal explanation
4. uncertainty disclosure
5. relational acknowledgment
6. evidence request
7. bounded commitment
8. careful abstention
9. capability boundary
10. factual status

The corpus spans `Assert`, `Qualify`, `Contrast`, `Correct`, `Explain`, `Acknowledge`, `RequestEvidence`, `Commit`, and `Abstain` operations. Each scenario has its own semantic program, lexical table, prohibited claim, forbidden surface form, and subject scope.

## Matched controls

For every scenario, the evaluator runs:

- sixteen entropy seeds under the default conversational microstate
- exact replay of every seed
- an independently reconstructed verification pass over every selected surface
- comparison with the deterministic neutral realization
- a same-seed no-trace control and recent-language trace treatment
- a same-seed direct-versus-warm microstate comparison

## Frozen gates

The experiment passes only when all of the following hold:

- semantic verification pass rate is exactly 100%
- exact replay rate is exactly 100%
- neutral divergence rate is at least 90%
- recent-language treatment changes the opening in at least 70% of scenarios
- direct-versus-warm microstates change the selected surface in at least 60% of scenarios
- fallback count is zero
- legacy remediation leads occur zero times
- every scenario produces at least two unique complete surfaces
- every scenario produces at least two unique opening fingerprints
- the complete L1-A authority boundary remains closed

## Interpretation

A pass means the bounded selector survived a separate, heterogeneous replay corpus without semantic drift and showed measurable wording responsiveness. It would justify designing an L1-C shadow observation layer that records candidate and control outputs without returning learned text.

A failure freezes live integration. Thresholds must not be weakened after seeing the result; failures should be recorded and remediated through a separately preregistered experiment.
