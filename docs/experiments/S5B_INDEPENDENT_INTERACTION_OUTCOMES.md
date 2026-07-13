# S5-B — Independent Interaction Outcomes

## Purpose

S5-B adds a replayable outcome boundary for S5-A policy trials. It prevents a common counterfactual error: treating one observed user reaction as evidence about every policy arm.

The rule is:

> A witness may resolve only the response it actually observed.

## Direct observed outcomes

A trial may declare one delivered arm. Later direct evidence can resolve only that arm.

Accepted direct evidence includes:

- explicit positive or negative user ratings;
- user corrections;
- clarification requests;
- independently observed task completion;
- independently observed abandonment;
- neutral follow-ups, which are recorded but remain inconclusive.

User-originated signals require a `UserObservation` witness. Task completion and abandonment may also use an `Environment` witness. `ResponseGenerator` and `ExternalEvaluator` sources are rejected on this channel.

A pure-shadow trial has no delivered arm and therefore cannot be resolved from direct user or environment evidence.

## Counterfactual arms

Unshown arms remain pending unless an external evaluator explicitly compares two rendered alternatives.

A paired evaluation records:

- two distinct policy variants;
- distinct render digests proving two alternatives existed;
- evaluator and evidence digests;
- a left, right, or tie judgment;
- an observation time inside the declared S4 outcome window.

A decisive comparison resolves exactly two predictions: the preferred alternative as satisfactory and the rejected alternative as unsatisfactory. A tie records evidence without resolving either prediction.

This is offline evaluator evidence, not user evidence. It must not be represented as though the user observed both alternatives.

## Atomicity and replay

`InteractionOutcomeLedger` mirrors the S4 prediction ledger and records typed events:

- trial registration with the original S5-A enrollment transitions;
- direct observed signals and optional S4 resolution;
- paired external evaluations and zero or two S4 resolutions.

Every operation is clone-then-commit. A failed second resolution, stale version, invalid witness, or diverged S4 ledger leaves both ledgers unchanged.

Replay begins from the captured base S4 ledger and re-applies every recorded S4 transition. IDs, versions, labels, witnesses, timing, and scores must match exactly.

## Frozen probe

The executable probe requires:

- a neutral follow-up to leave S4 unchanged;
- direct evidence to resolve only the declared delivered arm;
- direct evidence on a pure-shadow trial to be rejected atomically;
- response-generator self-grading to be rejected;
- external evaluator evidence to be rejected on the direct channel;
- one paired comparison to resolve exactly two arms;
- all unobserved counterfactual arms to remain pending;
- exact S5-B and S4 replay equality;
- source companion state to remain unchanged.

## Authority boundary

S5-B has no:

- `Runtime::chat()` integration;
- response-generation or response-selection authority;
- routing or CHARGE authority;
- companion-state mutation;
- persistence adapter in this slice;
- belief or ontology promotion;
- autonomous side-effect authority.

S5-C may aggregate these independently witnessed outcomes and test whether the companion-derived arm beats matched controls on held-out data. No live promotion follows from S5-B alone.
